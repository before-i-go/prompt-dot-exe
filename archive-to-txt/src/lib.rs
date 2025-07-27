//! A high-performance library for creating text-based archives of directory contents.
//!
//! This library provides functionality to recursively process directories, read file contents,
//! and generate formatted text archives with support for both sequential and parallel processing.
//! It includes advanced features like LLM-optimized filtering, git integration, and configurable
//! output formats.
//!
//! # Features
//! - Recursive directory traversal with configurable depth
//! - Parallel processing using Rayon for improved performance
//! - Multiple output formats: Plain text, JSON, Markdown, Rich text
//! - Thread-safe operations with minimal lock contention
//! - Comprehensive error handling with detailed error messages
//! - Git repository awareness (optional)
//! - LLM-optimized file filtering
//! - Configurable file inclusion/exclusion patterns
//! - File size limits and extension filtering
//!
//! # Quick Start
//! ```no_run
//! use archive_to_txt::{archive_directory, config::{Config, OutputFormat}};
//!
//! // Create a basic configuration
//! let config = Config::default()
//!     .with_input("./src")
//!     .with_output("./archive.txt")
//!     .with_parallel(true)
//!     .with_llm_optimize(true);
//!
//! // Create a text archive
//! archive_directory("./src", "./archive.txt", &config).unwrap();
//!
//! // Or create a JSON archive
//! let config = config
//!     .with_output("./archive.json")
//!     .with_format(OutputFormat::Json);
//! 
//! archive_directory("./src", "./archive.json", &config).unwrap();
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]

// Public modules
pub mod config;
pub mod error;
pub mod formatter;
pub mod git;
pub mod stats;
pub mod utils;
pub mod filter;

use std::fs::{self, File, Metadata};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::time::SystemTime;

use chrono::{DateTime, Local};
use log::{debug, error, info, warn};
use rayon::prelude::*;
use serde::Serialize;
use walkdir::WalkDir;

use crate::{
    config::Config,
    error::{ArchiveError, Result as ArchiveResult},
    formatter::{create_formatter, Formatter as FormatterTrait},
    git::GitInfo,
    utils::{format_file_size, format_path, format_timestamp},
};

/// Statistics about the archiving process
#[derive(Debug, Default, Clone, Serialize)]
pub struct ArchiveStats {
    /// Total number of files processed
    pub files_processed: usize,
    /// Total number of directories processed
    pub dirs_processed: usize,
    /// Total size of all processed files in bytes
    pub total_size: u64,
    /// Number of files skipped due to filtering
    pub files_skipped: usize,
    /// Number of files that couldn't be read
    pub read_errors: usize,
    /// Time taken for the archiving process in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
    /// Timestamp when the archive was created (ISO 8601 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// The main archive engine that handles the archiving process.
///
/// This struct provides methods to process directories and create text-based archives.
/// It supports both sequential and parallel processing modes, with advanced filtering
/// and formatting options.
///
/// # Examples
/// ```no_run
/// use archive_to_txt::{ArchiveEngine, config::{Config, OutputFormat}};
///
/// // Create a new archive engine with custom configuration
/// let config = Config::default()
///     .with_input("./src")
///     .with_output("./archive.txt")
///     .with_parallel(true)
///     .with_llm_optimize(true)
///     .with_include_extensions("rs,toml,md");
///
/// let engine = ArchiveEngine::new(config);
/// let stats = engine.run().expect("Failed to create archive");
///
/// println!("Archived {} files ({} bytes)", stats.files_processed, stats.total_size);
/// ```
#[derive(Debug)]
pub struct ArchiveEngine {
    /// Configuration for the archiving process
    config: Config,
    /// Git repository information (if available)
    git_info: Option<GitInfo>,
    /// Statistics about the archiving process
    stats: ArchiveStats,
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
    /// ```no_run
    /// use archive_to_txt::ArchiveEngine;
    /// use archive_to_txt::config::Config;
    ///
    /// // Create a new configuration with custom settings
    /// let config = Config::default()
    ///     .with_input("./src")
    ///     .with_output("./archive.txt")
    ///     .with_parallel(true)
    ///     .with_llm_optimize(true);
    ///
    /// // Create a new archive engine with the configuration
    /// let engine = ArchiveEngine::new(config).expect("Failed to initialize archive engine");
    /// ```
    pub fn new(config: Config) -> Result<Self> {
        // Initialize git info if git is enabled
        let git_info = if config.git_info {
            GitInfo::from_path(&config.input).ok()
        } else {
            None
        };

        Ok(Self {
            config,
            git_info,
            stats: ArchiveStats::default(),
        })
    }

    /// Runs the archiving process.
    ///
    /// This method processes all files in the input directory according to the
    /// configuration and writes the formatted output to the specified file.
    ///
    /// # Returns
    ///
    /// Returns `Ok(ArchiveStats)` containing statistics about the archiving process,
    /// or an `ArchiveError` if the operation fails.
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The input directory doesn't exist or isn't accessible
    /// - The output file can't be created or written to
    /// - Any file processing fails
    /// - Any I/O error occurs during processing
    ///
    /// # Example
    /// ```no_run
    /// use archive_to_txt::ArchiveEngine;
    /// use archive_to_txt::config::Config;
    ///
    /// let config = Config::default()
    ///     .with_input("./src")
    ///     .with_output("./archive.txt");
    ///
    /// let mut engine = ArchiveEngine::new(config)?;
    /// let stats = engine.run()?;
    ///
    /// println!("Archived {} files ({} bytes) in {:?}",
    ///     stats.files_processed,
    ///     stats.total_size,
    ///     stats.duration
    /// );
    /// ```
    pub fn run(&mut self) -> Result<ArchiveStats> {
        let start_time = Instant::now();
        let mut stats = ArchiveStats::default();

        // Create output directory if it doesn't exist
        if let Some(parent) = self.config.output.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| ArchiveError::io_error(e, "Failed to create output directory"))?;
            }
        }

        // Create the output file
        let output_file = File::create(&self.config.output)
            .map_err(|e| ArchiveError::io_error(e, format!("Failed to create output file: {:?}", self.config.output)))?;

        // Create a buffered writer for better performance
        let writer = BufWriter::new(output_file);
        let writer_mutex = Arc::new(Mutex::new(writer));

        // Select formatter based on output format
        let formatter = create_formatter(self.config.format);

        // Process files in parallel or sequentially based on configuration
        if self.config.parallel {
            self.process_files_parallel(&formatter, &writer_mutex)?;
        } else {
            self.process_files_sequential(&formatter, &writer_mutex)?;
        }

        // Flush any remaining output
        let mut writer_guard = writer_mutex.lock().map_err(|e| ArchiveError::Other(e.to_string()))?;
        writer_guard.flush().map_err(|e| ArchiveError::io_error(e, "Failed to flush output"))?;

        // Calculate and return statistics
        stats.duration = start_time.elapsed();
        Ok(stats)
    }

    /// Process files sequentially.
    ///
    /// This method processes files one by one in the current thread.
    ///
    /// # Arguments
    /// * `formatter` - The formatter to use for formatting file contents
    /// * `writer` - Thread-safe writer for output
    ///
    /// # Returns
    /// `Result<(), ArchiveError>` indicating success or failure
    fn process_files_sequential<W: Write + Send + 'static>(
        &self,
        formatter: &dyn FormatterTrait,
        writer: &Arc<Mutex<W>>,
    ) -> Result<()> {
        // Collect all files that match the criteria
        let entries = self.collect_files()?;
        let file_count = entries.len();
        let mut processed_count = 0;
        
        // Process each file
        for path in entries {
            match self.process_single_file(&path, formatter, writer) {
                Ok(_) => processed_count += 1,
                Err(e) => {
                    error!("Error processing file {}: {}", path.display(), e);
                    if !self.config.continue_on_error {
                        return Err(e);
                    }
                }
            }
        }
        
        info!("Processed {} of {} files sequentially", processed_count, file_count);
        
        if processed_count == 0 && file_count > 0 {
            return Err(ArchiveError::Other("No files were processed successfully".to_string()));
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
    /// * `formatter` - The formatter to use for formatting file contents
    /// * `writer` - Thread-safe writer for output
    ///
    /// # Returns
    /// `Result<(), ArchiveError>` indicating success or failure
    ///
    /// # Errors
    /// Returns an error if any file processing fails or if writing to the output fails.
    ///
    fn process_files_parallel<W: Write + Send + 'static>(
        &self,
        formatter: &dyn FormatterTrait,
        writer: &Arc<Mutex<W>>,
    ) -> Result<()> {
        use rayon::prelude::*;

        // Collect all files that match the criteria
        let entries = self.collect_files()?;
        let file_count = entries.len();
        let processed_count = AtomicUsize::new(0);
        
        // Process files in parallel using Rayon
        let result: Result<(), ArchiveError> = entries.par_iter().try_for_each(|path| {
            match self.process_single_file(path, formatter, writer) {
                Ok(_) => {
                    processed_count.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
                Err(e) => {
                    error!("Error processing file {}: {}", path.display(), e);
                    if !self.config.continue_on_error {
                        return Err(e);
                    }
                    Ok(())
                }
            }
        });
        
        let processed = processed_count.load(Ordering::Relaxed);
        info!("Processed {} of {} files in parallel", processed, file_count);
        
        if processed == 0 && file_count > 0 {
            return Err(ArchiveError::Other("No files were processed successfully".to_string()));
        }
        
        result
    }

    /// Collect all files that should be included in the archive based on the configuration.
    ///
    /// This method walks the input directory and applies all filters (include/exclude patterns,
    /// file extensions, size limits, etc.) to determine which files should be processed.
    ///
    /// # Returns
    /// A `Vec<PathBuf>` containing paths to all files that should be included in the archive.
    ///
    /// # Errors
    /// Returns an error if the input directory cannot be read or if any I/O error occurs.
    fn collect_files(&self) -> Result<Vec<PathBuf>> {
        let mut entries = Vec::new();
        let mut walker = WalkBuilder::new(&self.config.input);

        // Configure the walker based on the configuration
        walker
            .hidden(!self.config.include_hidden)
            .follow_links(self.config.follow_links)
            .git_ignore(self.config.llm_optimize)
            .git_global(self.config.llm_optimize)
            .git_exclude(self.config.llm_optimize);

        // Apply max depth if specified
        if let Some(max_depth) = self.config.max_depth {
            walker.max_depth(Some(max_depth));
        }

        // Add custom ignore patterns
        if let Some(patterns) = &self.config.exclude {
            for pattern in patterns {
                walker.add_custom_ignore_filename(pattern);
            }
        }

        // Add LLM ignore patterns if enabled
        if self.config.llm_optimize {
            for pattern in Config::get_default_llm_ignore_patterns() {
                walker.add_custom_ignore_filename(pattern);
            }
        }

        // Build the walker and process entries
        for entry in walker.build() {
            match entry {
                Ok(entry) => {
                    if entry.file_type().map_or(false, |ft| ft.is_file()) {
                        entries.push(entry.into_path());
                    } else if entry.file_type().map_or(false, |ft| ft.is_dir()) {
                        self.stats.dirs_processed += 1;
                    }
                }
                Err(e) => {
                    error!("Error reading directory entry: {}", e);
                    self.stats.read_errors += 1;
                }
            }
        }

        // Apply additional filters
        let included_extensions = self.config.get_included_extensions();
        let max_file_size = self.config.max_file_size;

        let filtered_entries: Vec<_> = entries
            .into_iter()
            .filter(|path| {
                // Filter by extension if specified
                if let Some(exts) = &included_extensions {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if !exts.contains(&ext.to_lowercase()) {
                            self.stats.files_skipped += 1;
                            return false;
                        }
                    } else {
                        self.stats.files_skipped += 1;
                        return false;
                    }
                }

                // Filter by file size if specified
                if let (Some(max_size), Ok(metadata)) = (max_file_size, path.metadata()) {
                    if metadata.len() > max_size {
                        self.stats.files_skipped += 1;
                        return false;
                    }
                }

                true
            })
            .collect();

        Ok(filtered_entries)
    }

    /// Display filtering statistics if verbose output is enabled.
    fn display_filter_stats(&self) {
        if self.config.verbosity > 0 {
            info!("Filtering statistics:");
            info!("  - Files processed: {}", self.stats.files_processed);
            info!("  - Directories processed: {}", self.stats.dirs_processed);
            info!("  - Files skipped: {}", self.stats.files_skipped);
            info!("  - Read errors: {}", self.stats.read_errors);
            info!("  - Total size: {} bytes", self.stats.total_size);
        }
    }

    /// Process a single file and write its contents to the output.
    ///
    /// This method reads the file, applies any necessary formatting, and writes
    /// the result to the output writer. It also updates the statistics.
    ///
    /// # Arguments
    /// * `path` - Path to the file to process
    /// * `formatter` - Formatter to use for the file content
    /// * `writer` - Thread-safe writer for output
    /// * `file_count` - Atomic counter for tracking processed files
    fn process_single_file<W: Write + Send>(
        &self,
        path: &Path,
        formatter: &dyn FormatterTrait,
        writer: &Arc<Mutex<W>>,
        file_count: &AtomicUsize,
    ) -> Result<()> {
        // Check file size limit if specified
        if let Some(max_size) = self.config.max_file_size {
            let metadata = std::fs::metadata(path).map_err(|e| {
                ArchiveError::io_error(e, format!("Failed to get metadata for: {}", path.display()))
            })?;
            
            if metadata.len() > max_size {
                debug!("Skipping large file: {} ({} bytes)", path.display(), metadata.len());
                return Ok(());
            }
        }

        // Read file content
        let content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
                // File is not valid UTF-8, read as binary
                let bytes = std::fs::read(path).map_err(|e| {
                    ArchiveError::io_error(e, format!("Failed to read file: {}", path.display()))
                })?;
                
                // Try to convert to UTF-8 with replacement characters for invalid sequences
                String::from_utf8_lossy(&bytes).into_owned()
            }
            Err(e) => {
                return Err(ArchiveError::io_error(
                    e,
                    format!("Failed to read file: {}", path.display()),
                ));
            }
        };

        // Format and write the file content
        let mut writer_guard = writer.lock().map_err(|e| {
            ArchiveError::io_error(
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
                "Failed to acquire write lock",
            )
        })?;

        formatter
            .format_file(path, &content, &mut *writer_guard)
            .map_err(|e| {
                ArchiveError::io_error(
                    e,
                    format!("Failed to format file: {}", path.display()),
                )
            })?;

        // Update statistics
        self.stats.files_processed += 1;
        self.stats.total_size += content.len() as u64;

        Ok(())
    }

    /// Process a single file and write its formatted content to the given writer.
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
    /// use archive_to_txt::archive_directory;
    /// use archive_to_txt::config::Config;
    ///
    /// // Create configuration with parallel processing enabled
    /// let config = Config::default()
    ///     .with_input("./src")
    ///     .with_output("./archive.txt")
    ///     .with_parallel(true);
    ///
    /// // Archive the directory
    /// archive_directory("./src", "./archive.txt", &config).unwrap();
    /// ```
    fn process_single_file_to_buffer(
        &self,
        path: &Path,
        formatter: &dyn FormatterTrait,
        buffer: &mut Vec<u8>,
        file_count: &AtomicUsize,
    ) -> Result<()> {
        // Read file as binary
        let bytes = std::fs::read(path)
            .map_err(ArchiveError::Io)?;
        
        // Convert to string, replacing invalid UTF-8 sequences with replacement characters
        let content = String::from_utf8_lossy(&bytes);
        
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
    /// use archive_to_txt::ArchiveEngine;
    /// use archive_to_txt::config::Config;
    ///
    /// // Create a new archive engine with custom configuration
    /// let config = Config::default()
    ///     .with_input("./src")
    ///     .with_output("./archive.txt")
    ///     .with_include_hidden(true);
    ///
    /// let engine = ArchiveEngine::new(config);
    ///
    /// // Process the files and create the archive
    /// engine.run().expect("Failed to create archive");
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
/// use archive_to_txt::{archive_directory, config::Config};
///
/// let config = Config::default()
///     .with_input("./src")
///     .with_output("./archive.txt")
///     .with_parallel(true);
///
/// archive_directory("./src", "./archive.txt", &config).unwrap();
/// ```
pub fn archive_directory(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    config: &Config,
) -> Result<()> {
    // Create a new configuration with the provided input and output
    let config = config.clone()
        .with_input(input)
        .with_output(output);
    
    // Create and run the archive engine
    let mut engine = ArchiveEngine::new(config)?;
    engine.run()?;
    
    Ok(())
}

/// Default formatter implementation for testing
#[cfg(test)]
mod test_utils {
    use super::*;
    use std::path::Path;

    pub struct TestFormatter;

    impl FormatterTrait for TestFormatter {
        fn format_file<W: std::io::Write>(
            &self,
            path: &Path,
            content: &str,
            writer: &mut W,
        ) -> std::io::Result<()> {
            writeln!(writer, "File: {}", path.display())?;
            writeln!(writer, "Content length: {} bytes", content.len())?;
            writeln!(writer, "---")?;
            writeln!(writer, "{}", content)?;
            writeln!(writer, "---\n")?;
            Ok(())
        }
    }

    pub fn create_test_file(dir: &tempfile::TempDir, path: &str, content: &str) -> PathBuf {
        let file_path = dir.path().join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&file_path, content).unwrap();
        file_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use pretty_assertions::assert_eq;
    use serial_test::serial;
    use std::io::Read;
    use test_utils::*;
    use predicates::prelude::*;
    use rstest::rstest;
    use std::io::Cursor;
    use std::sync::Arc;
    use std::sync::Mutex;

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
        
        // Create a test file
        let file = temp_dir.child("test.txt");
        file.write_str("Hello, world!")?;
        
        // Set up output file and config
        let output_file = temp_dir.path().join("archive.txt");
        
        let config = Config::default()
            .with_input(temp_dir.path())
            .with_output(&output_file)
            .with_include_hidden(true);
        
        // Run the archive function
        let result = archive_directory(temp_dir.path(), &output_file, &config);
        assert!(result.is_ok(), "archive_directory failed: {:?}", result.err());
        
        // Read the output file
        let content = std::fs::read_to_string(&output_file)?;
        
        // Verify the content
        assert!(content.contains("test.txt"), "Expected to find 'test.txt' in output");
        assert!(content.contains("Hello, world!"), "Expected to find file content in output");
        
        Ok(())
    }
    
    #[test]
    fn test_process_single_file() -> TestResult {
        let temp_dir = tempfile::tempdir()?;
        let file_path = create_test_file(&temp_dir, "test.txt", "Test content");
        
        let config = Config::default()
            .with_input(temp_dir.path().to_path_buf())
            .with_output(PathBuf::from("output.txt"));
            
        let engine = ArchiveEngine::new(config)?;
        let formatter = TestFormatter;
        let output = Arc::new(Mutex::new(Vec::new()));
        
        engine.process_single_file(&file_path, &formatter, &output)?;
        
        let output_str = String::from_utf8(output.lock().unwrap().clone())?;
        assert!(output_str.contains("test.txt"), "Output should contain filename");
        assert!(output_str.contains("Test content"), "Output should contain file content");
        Ok(())
    }
    
    #[test]
    fn test_binary_file_handling() -> TestResult {
        let temp_dir = tempfile::tempdir()?;
        let file_path = temp_dir.path().join("test.bin");
        std::fs::write(&file_path, b"\x00\x01\x02\x03\xFF")?;
        
        let config = Config::default();
        let engine = ArchiveEngine::new(config)?;
        let formatter = TestFormatter;
        let output = Arc::new(Mutex::new(Vec::new()));
        
        engine.process_single_file(&file_path, &formatter, &output)?;
        
        let output_str = String::from_utf8_lossy(&output.lock().unwrap());
        assert!(output_str.contains("test.bin"), "Should process binary files");
        Ok(())
    }
    
    #[test]
    fn test_large_file_skipping() -> TestResult {
        let temp_dir = tempfile::tempdir()?;
        let file_path = temp_dir.path().join("large.txt");
        let large_content = "a".repeat(2048); // 2KB file
        std::fs::write(&file_path, large_content)?;
        
        let config = Config::default()
            .with_max_file_size(Some(1024)); // 1KB limit
            
        let engine = ArchiveEngine::new(config)?;
        let formatter = TestFormatter;
        let output = Arc::new(Mutex::new(Vec::new()));
        
        engine.process_single_file(&file_path, &formatter, &output)?;
        
        // Output should be empty since the file was skipped
        assert!(output.lock().unwrap().is_empty(), "Large file should be skipped");
        Ok(())
    }
    
    #[test]
    fn test_file_filtering() -> TestResult {
        let temp_dir = tempfile::tempdir()?;
        
        // Create test files
        create_test_file(&temp_dir, "include.rs", "fn main() {}");
        create_test_file(&temp_dir, "exclude.txt", "Don't include me");
        create_test_file(&temp_dir, "subdir/another.rs", "mod test {}");
        
        let config = Config::default()
            .with_input(temp_dir.path().to_path_buf())
            .with_include(Some(vec!["**/*.rs".to_string()]))
            .with_exclude(Some(vec!["**/exclude.txt".to_string()]));
            
        let engine = ArchiveEngine::new(config)?;
        let files = engine.collect_files()?;
        
        // Should include .rs files but exclude exclude.txt
        assert_eq!(files.len(), 2, "Should find 2 .rs files");
        assert!(files.iter().any(|f| f.ends_with("include.rs")));
        assert!(files.iter().any(|f| f.ends_with("subdir/another.rs")));
        assert!(!files.iter().any(|f| f.ends_with("exclude.txt")), "exclude.txt should be excluded");
        
        Ok(())
    }
    
    #[test]
    fn test_parallel_processing() -> TestResult {
        let temp_dir = tempfile::tempdir()?;
        let mut expected_files = Vec::new();
        
        // Create multiple test files
        for i in 0..5 {
            let file_name = format!("file_{}.txt", i);
            expected_files.push(file_name.clone());
            create_test_file(&temp_dir, &file_name, &format!("Content {}", i));
        }
        
        let config = Config::default()
            .with_input(temp_dir.path().to_path_buf())
            .with_output(PathBuf::from("output.txt"))
            .with_parallel(true);
            
        let engine = ArchiveEngine::new(config)?;
        let formatter = TestFormatter;
        let output = Arc::new(Mutex::new(Vec::new()));
        
        // Collect and process files in parallel
        let files = engine.collect_files()?;
        engine.process_files_parallel(&files, &formatter, &output)?;
        
        // Verify all files were processed
        let output_str = String::from_utf8_lossy(&output.lock().unwrap());
        for file in &expected_files {
            assert!(
                output_str.contains(file),
                "Missing file in output: {}",
                file
            );
            assert!(
                output_str.contains(&format!("Content {}", file.chars().last().unwrap())),
                "Missing content for file: {}",
                file
            );
        }
        
        Ok(())
    }
    
    #[test]
    fn test_format_utilities() -> TestResult {
        // Test path formatting
        let path = PathBuf::from("path/to/file.txt");
        let formatted = format_path(&path);
        assert_eq!(formatted, "path/to/file.txt");
        
        // Test file size formatting
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(1023), "1023 B");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
        
        // Test timestamp formatting (just verify it produces some output)
        let timestamp = std::time::SystemTime::now();
        let formatted_time = format_timestamp(timestamp);
        assert!(!formatted_time.is_empty(), "Timestamp should not be empty");
        
        Ok(())
    }
}
