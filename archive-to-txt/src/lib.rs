//! A library for creating text-based archives of directory contents.
//!
//! This library provides functionality to recursively walk through directories,
//! read file contents, and generate formatted text archives.

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
pub struct ArchiveEngine {
    config: Config,
}

impl ArchiveEngine {
    /// Creates a new `ArchiveEngine` with the given configuration.
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Runs the archiving process.
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
    
    /// Process files in parallel using Rayon
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
    
    /// Process a single file and write to the given writer
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
    
    /// Process a single file into a buffer
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
    
    /// Format and write a file's content to the writer
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
