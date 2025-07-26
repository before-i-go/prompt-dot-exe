//! A high-performance library for creating text-based archives of directory contents.
//!
//! This library provides functionality to recursively process directories, read file contents,
//! and generate formatted text archives with support for both sequential and parallel processing.
//!
//! # Features
//! - Recursive directory traversal
//! - Parallel processing using Rayon
//! - Configurable output formats
//! - Thread-safe operations
//! - Comprehensive error handling
//!
//! # Quick Start
//! ```no_run
//! use archive_to_txt::{archive_directory, Config};
//! use std::path::Path;
//!
//! let config = Config::default()
//!     .with_input("./src")
//!     .with_output("./archive.txt")
//!     .with_parallel(true);
//!
//! archive_directory("./src", "./archive.txt", &config).unwrap();
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod config;
pub mod error;
pub mod formatter;

use std::path::Path;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;
use rayon::prelude::*;
use crate::formatter::text::TextFormatter;
use crate::formatter::Formatter as FormatterTrait;

use crate::{
    config::Config,
    error::{ArchiveError, Result},
    formatter::create_formatter,
};

/// The main archive engine that handles the archiving process.
///
/// This struct provides methods to process directories and create text-based archives.
/// It supports both sequential and parallel processing modes.
///
/// # Examples
/// ```
/// use archive_to_txt::{ArchiveEngine, Config};
///
/// let config = Config::default()
///     .with_input("./src")
///     .with_output("./archive.txt");
///
/// let engine = ArchiveEngine::new(config);
/// engine.run().expect("Failed to create archive");
/// ```
pub struct ArchiveEngine {
    /// Configuration for the archiving process
    config: Config,
}

impl ArchiveEngine {
    /// Creates a new `ArchiveEngine` with the given configuration.
    ///
    /// # Arguments
    /// * `config` - Configuration for the archiving process
    ///
    /// # Returns
    /// A new instance of `ArchiveEngine`
    ///
    /// # Example
    /// ```
    /// use archive_to_txt::{ArchiveEngine, Config};
    ///
    /// let config = Config::default();
    /// let engine = ArchiveEngine::new(config);
    /// ```
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Runs the archiving process.
    ///
    /// This method processes all files in the input directory according to the
    /// configuration and writes the formatted output to the specified file.
    ///
    /// # Returns
    /// `Ok(())` if the operation was successful, or an `ArchiveError` if an error occurred.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The input directory doesn't exist or isn't accessible
    /// - The output file can't be created or written to
    /// - Any file processing fails
    ///
    /// # Example
    /// ```no_run
    /// # use archive_to_txt::{ArchiveEngine, Config};
    /// let config = Config::default()
    ///     .with_input("./src")
    ///     .with_output("./archive.txt");
    /// let engine = ArchiveEngine::new(config);
    /// engine.run().expect("Failed to create archive");
    /// ```
    pub fn run(&self) -> Result<()> {
        // Create output directory if it doesn't exist
        if let Some(parent) = self.config.output.parent() {
            std::fs::create_dir_all(parent).map_err(ArchiveError::Io)?;
        }
        let output_file = self.config.output.canonicalize()
            .unwrap_or_else(|_| self.config.output.clone());
        
        let mut output = File::create(&output_file)
            .map_err(|e| ArchiveError::Io(e))?;
        
        let mut writer = BufWriter::new(&mut output);
        let formatter = create_formatter(self.config.format);
        
        // Create the output file and writer
        let output_file = File::create(&self.config.output)
            .map_err(ArchiveError::Io)?;
        let writer = Arc::new(Mutex::new(BufWriter::new(output_file)));
        
        // Create the formatter and convert to a boxed trait object
        let formatter: Box<dyn FormatterTrait> = Box::new(TextFormatter::new());
        
        // Write header
        writer.lock().unwrap()
            .write_all(formatter.format_header().as_bytes())
            .map_err(ArchiveError::Io)?;
        
        // Create a closure for filtering entries
        let filter_entry = |e: &walkdir::DirEntry| -> bool {
            let name = e.file_name().to_string_lossy();
            self.config.include_hidden || !name.starts_with('.')
        };
        
        // Collect all file entries first
        let mut entries = Vec::new();
        for entry in WalkDir::new(&self.config.input)
            .into_iter()
            .filter_entry(|e| filter_entry(e))
        {
            let entry = entry.map_err(ArchiveError::from)?;
            if entry.file_type().is_file() {
                entries.push(entry.into_path());
            }
        }
            
        let file_count = AtomicUsize::new(0);
        
        // Process files in parallel or sequentially based on config
        if self.config.parallel {
            self.process_files_parallel(&entries, &*formatter, &writer, &file_count)?;
        } else {
            self.process_files_sequential(&entries, &*formatter, &writer, &file_count)?;
        }
        
        // Write footer with the total file count
        let footer = formatter.format_footer(file_count.into_inner());
        writer.lock().unwrap().write_all(footer.as_bytes())
            .map_err(ArchiveError::Io)?;
        
        Ok(())
    }
    /// Process files sequentially
    fn process_files_sequential(
        &self,
        entries: &[std::path::PathBuf],
        formatter: &dyn FormatterTrait,
        writer: &Arc<Mutex<impl Write>>,
        file_count: &AtomicUsize,
    ) -> Result<()> {
        for path in entries {
            self.process_single_file(path, formatter, &mut *writer.lock().unwrap(), file_count)?;
        }
        Ok(())
    }
    
    /// Process files in parallel using Rayon's work-stealing thread pool.
    ///
    /// This method distributes file processing across multiple threads for improved
    /// performance on multi-core systems. Each file is processed independently and
    /// results are written to the output in a thread-safe manner.
    ///
    /// # Arguments
    /// * `entries` - List of file paths to process
    /// * `formatter` - Formatter to use for output generation
    /// * `writer` - Thread-safe writer for output
    /// * `file_count` - Atomic counter to track processed files
    ///
    /// # Returns
    /// `Ok(())` if all files were processed successfully, or an error if any file processing fails.
    ///
    /// # Errors
    /// Returns an error if any file processing fails or if writing to the output fails.
    ///
    /// # Performance
    /// - Uses Rayon's work-stealing scheduler for optimal load balancing
    /// - Minimizes lock contention through buffered writes
    /// - Processes files in chunks to balance between parallelism and overhead
    fn process_files_parallel(
        &self,
        entries: &[std::path::PathBuf],
        formatter: &dyn FormatterTrait,
        writer: &Arc<Mutex<impl Write + Send>>,
        file_count: &AtomicUsize,
    ) -> Result<()> {
        // Create thread-safe references
        let self_ref = &self;
        let formatter_ref = formatter;
        
        // Process files in parallel
        entries.par_iter().try_for_each(|path| {
            let mut buffer = Vec::new();
            
            // Process the file into a buffer
            self_ref.process_single_file_to_buffer(path, formatter_ref, &mut buffer, file_count)?;
            
            // Write the buffer to the output file (synchronized)
            if !buffer.is_empty() {
                writer.lock().unwrap().write_all(&buffer).map_err(ArchiveError::Io)?;
            }
            
            Ok::<_, ArchiveError>(())
        })?;
        
        Ok(())
    }
    
    /// Process a single file and write its formatted content to the given writer.
    ///
    /// This method reads the file content, applies formatting using the provided formatter,
    /// and writes the result to the output writer. It also increments the file counter.
    ///
    /// # Arguments
    /// * `path` - Path to the file to process
    /// * `formatter` - Formatter to use for the file content
    /// * `writer` - Mutable reference to the output writer
    /// * `file_count` - Atomic counter to track the number of processed files
    ///
    /// # Returns
    /// `Ok(())` if the file was processed successfully, or an error if processing failed.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The file cannot be read
    /// - The file content cannot be converted to UTF-8
    /// - Writing to the output fails
    ///
    /// # Example
    /// ```no_run
    /// # use archive_to_txt::{ArchiveEngine, Config, formatter::TextFormatter};
    /// # use std::sync::atomic::{AtomicUsize, Ordering};
    /// # use std::fs::File;
    /// # use std::io::Write;
    /// # use std::path::Path;
    /// #
    /// let engine = ArchiveEngine::new(Config::default());
    /// let file_count = AtomicUsize::new(0);
    /// let mut output = Vec::new();
    /// let formatter = TextFormatter::new();
    ///
    /// engine.process_single_file(
    ///     Path::new("example.txt"),
    ///     &formatter,
    ///     &mut output,
    ///     &file_count
    /// ).unwrap();
    /// ```
    fn process_single_file<W: Write>(
        &self,
        path: &Path,
        formatter: &dyn crate::formatter::Formatter,
        writer: &mut W,
        file_count: &AtomicUsize,
    ) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .map_err(ArchiveError::Io)?;
        
        self.format_and_write(path, &content, formatter, writer)?;
        file_count.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }
    
    /// Process a single file and append its formatted content to the provided buffer.
    ///
    /// This method is similar to `process_single_file` but writes to a buffer instead of a writer.
    /// It's particularly useful for parallel processing where you want to collect output
    /// before writing to a shared resource.
    ///
    /// # Arguments
    /// * `path` - Path to the file to process
    /// * `formatter` - Formatter to use for the file content
    /// * `buffer` - Buffer to append the formatted content to
    /// * `file_count` - Atomic counter to track the number of processed files
    ///
    /// # Returns
    /// `Ok(())` if the file was processed successfully, or an error if processing failed.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The file cannot be read
    /// - The file content cannot be converted to UTF-8
    /// - The buffer cannot be written to
    ///
    /// # Example
    /// ```no_run
    /// # use archive_to_txt::{ArchiveEngine, Config, formatter::TextFormatter};
    /// # use std::sync::atomic::{AtomicUsize, Ordering};
    /// # use std::path::Path;
    /// #
    /// let engine = ArchiveEngine::new(Config::default());
    /// let file_count = AtomicUsize::new(0);
    /// let mut buffer = Vec::new();
    /// let formatter = TextFormatter::new();
    ///
    /// engine.process_single_file_to_buffer(
    ///     Path::new("example.txt"),
    ///     &formatter,
    ///     &mut buffer,
    ///     &file_count
    /// ).unwrap();
    /// ```
    fn process_single_file_to_buffer(
        &self,
        path: &Path,
        formatter: &dyn FormatterTrait,
        buffer: &mut Vec<u8>,
        file_count: &AtomicUsize,
    ) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .map_err(ArchiveError::Io)?;
        
        // Get the relative path and convert to string
        let relative_path = path.strip_prefix(&self.config.input)
            .unwrap_or_else(|_| path);
        
        // Format the file content
        let formatted = formatter.format_file(relative_path, &content);
        
        // Write to buffer
        buffer.extend_from_slice(formatted.as_bytes());
        file_count.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }
    
    /// Format a file's content and write it to the provided writer.
    ///
    /// This method handles the formatting of file content using the specified formatter
    /// and writes the result to the output writer. It's a lower-level method used by
    /// both `process_single_file` and `process_single_file_to_buffer`.
    ///
    /// # Arguments
    /// * `path` - Path to the file being processed (used for metadata in formatting)
    /// * `content` - The content of the file to format
    /// * `formatter` - Formatter to use for the content
    /// * `writer` - Writer to write the formatted output to
    ///
    /// # Returns
    /// `Ok(())` if the content was formatted and written successfully, or an error if writing failed.
    ///
    /// # Errors
    /// Returns an error if writing to the output writer fails.
    ///
    /// # Example
    /// ```no_run
    /// # use archive_to_txt::{ArchiveEngine, Config, formatter::TextFormatter};
    /// # use std::io::Write;
    /// # use std::path::Path;
    /// #
    /// let engine = ArchiveEngine::new(Config::default());
    /// let formatter = TextFormatter::new();
    /// let mut output = Vec::new();
    ///
    /// engine.format_and_write(
    ///     Path::new("example.txt"),
    ///     "File content",
    ///     &formatter,
    ///     &mut output
    /// ).unwrap();
    /// ```
    fn format_and_write(
        &self,
        path: &Path,
        content: &str,
        formatter: &dyn crate::formatter::Formatter,
        writer: &mut dyn Write,
    ) -> Result<()> {
        // Get the relative path and convert to string
        let relative_path = path.strip_prefix(&self.config.input)
            .unwrap_or_else(|_| path);
        
        // Format the file content
        let formatted = formatter.format_file(relative_path, content);
        
        // Write to output
        writer.write_all(formatted.as_bytes())
            .map_err(ArchiveError::Io)?;
            
        Ok(())
    }
}

/// Archives the given directory to the specified output file.
///
/// This is a convenience function that creates a new `ArchiveEngine` and runs it.
/// For more control over the archiving process, use `ArchiveEngine` directly.
///
/// # Arguments
/// * `input` - The input directory to archive
/// * `output` - The output file path
/// * `config` - Configuration for the archiving process
///
/// # Returns
/// `Ok(())` if the operation was successful, or an `ArchiveError` if an error occurred.
///
/// # Errors
/// Returns an error if:
/// - The input directory doesn't exist or isn't accessible
/// - The output file can't be created or written to
/// - Any file processing fails
///
/// # Example
/// ```no_run
/// use archive_to_txt::{archive_directory, Config};
///
/// let config = Config::default()
///     .with_parallel(true);
///
/// archive_directory("./src", "./archive.txt", &config).unwrap();
/// ```
pub fn archive_directory(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    config: &Config,
) -> Result<()> {
    let config = Config {
        input: input.as_ref().to_path_buf(),
        output: output.as_ref().to_path_buf(),
        ..config.clone()
    };
    
    ArchiveEngine::new(config).run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use predicates::prelude::*;
    use rstest::rstest;

    type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

    #[rstest]
    fn test_empty_directory() -> TestResult {
        let temp_dir = assert_fs::TempDir::new()?;
        let output_file = temp_dir.path().join("archive.txt");
        
        let config = Config::default()
            .with_input(temp_dir.path())
            .with_output(&output_file);
            
        let result = archive_directory(temp_dir.path(), &output_file, &config);
        assert!(result.is_ok());
        
        let content = std::fs::read_to_string(&output_file)?;
        assert!(content.contains("Archive created at"));
        assert!(content.contains("Total files processed: 0"));
        
        Ok(())
    }

    #[rstest]
    fn test_single_file() -> TestResult {
        // Create a temporary directory
        let temp_dir = assert_fs::TempDir::new()?;
        println!("Temp dir: {:?}", temp_dir.path());
        
        // Create a test file
        let file = temp_dir.child("test.txt");
        file.write_str("Hello, world!")?;
        println!("Created test file: {:?}", file.path());
        
        // Set up output file and config
        let output_file = temp_dir.path().join("archive.txt");
        println!("Output file: {:?}", output_file);
        
        let config = Config::default()
            .with_input(temp_dir.path())
            .with_output(&output_file)
            .with_include_hidden(true); // Include hidden files for testing
        
        // Run the archive function
        let result = archive_directory(temp_dir.path(), &output_file, &config);
        assert!(result.is_ok(), "archive_directory failed: {:?}", result.err());
        
        // Read the output file
        let content = std::fs::read_to_string(&output_file)?;
        println!("Archive content:\n{}", content);
        
        // Verify the content
        assert!(
            content.contains("test.txt"),
            "Expected to find 'test.txt' in: {} ",
            content
        );
        assert!(
            content.contains("Hello, world!"),
            "Expected to find 'Hello, world!' in: {}",
            content
        );
        
        // Additional debug: List all files in temp dir
        println!("Files in temp dir:");
        for entry in std::fs::read_dir(temp_dir.path())? {
            let entry = entry?;
            println!("- {:?}", entry.path());
        }
        
        Ok(())
    }
}
