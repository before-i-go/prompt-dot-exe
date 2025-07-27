//! A high-performance library for creating text-based archives of directory contents.
//!
//! This library provides functionality to recursively process directories, read file contents,
//! and generate formatted text archives with support for both sequential and parallel processing.

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
pub mod tree;

use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicUsize, Ordering, AtomicU64},
    Arc, Mutex,
};
use std::time::Instant;

use walkdir::WalkDir;
use log::{debug, error, info};
use rayon::prelude::*;

use crate::{
    config::{Config, OutputFormat},
    error::ArchiveError,
    formatter::{Formatter as FormatterTrait, text::PlainTextFormatter},
    git::GitInfo,
    stats::ArchiveStats,
};

/// Statistics for file filtering operations
#[derive(Debug, Clone, Default)]
pub struct FilterStatistics {
    pub total_files_found: usize,
    pub files_included: usize,
    pub files_excluded: usize,
    pub excluded_by_extension: usize,
    pub excluded_by_ignore_pattern: usize,
    pub excluded_by_llm_optimization: usize,
    pub excluded_by_size: usize,
    pub total_size_included: usize,
}

/// The main archive engine that handles the archiving process.
#[derive(Debug)]
pub struct ArchiveEngine {
    /// Configuration for the archiving process
    config: Config,
    /// Git repository information (if available)
    git_info: Option<GitInfo>,
    /// Statistics about the archiving process
    files_processed: AtomicUsize,
    files_skipped: AtomicUsize,
    error_count: AtomicUsize,
    total_size: AtomicU64,
}

impl ArchiveEngine {
    /// Creates a new `ArchiveEngine` with the given configuration.
    pub fn new(config: Config) -> Result<Self, ArchiveError> {
        // Initialize git info if this is a git repository
        let git_info = if config.llm_optimize {
            GitInfo::from_path(&config.input).ok()
        } else {
            None
        };

        Ok(Self {
            config,
            git_info,
            files_processed: AtomicUsize::new(0),
            files_skipped: AtomicUsize::new(0),
            error_count: AtomicUsize::new(0),
            total_size: AtomicU64::new(0),
        })
    }

    /// Runs the archiving process.
    pub fn run(&mut self) -> Result<ArchiveStats, ArchiveError> {
        let start_time = Instant::now();

        // Create output directory if it doesn't exist
        if let Some(parent) = Path::new(&self.config.output).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ArchiveError::io_error(e, "Failed to create output directory")
            })?;
        }

        // Create the output file
        let output_file = std::fs::File::create(&self.config.output).map_err(|e| {
            ArchiveError::io_error(e, "Failed to create output file")
        })?;

        // Create a thread-safe writer
        let writer = Arc::new(Mutex::new(BufWriter::new(output_file)));

        // Get the appropriate formatter based on configuration
        let formatter: Arc<dyn FormatterTrait> = match self.config.format {
            OutputFormat::Plain => Arc::new(PlainTextFormatter::new()),
            _ => Arc::new(PlainTextFormatter::new()), // Default to plain text for now
        };

        // Write the header with configuration
        {
            let header = formatter.format_header(Some(&self.config));
            let mut writer_guard = writer.lock().unwrap();
            writer_guard.write_all(header.as_bytes()).map_err(|e| {
                ArchiveError::io_error(e, "Failed to write header")
            })?;
        }

        // Process files based on configuration
        if self.config.parallel {
            self.process_files_parallel(&*formatter, &writer)?;
        } else {
            self.process_files_sequential(&*formatter, &writer)?;
        }

        // Write the footer
        let files_processed = self.files_processed.load(Ordering::SeqCst);
        {
            let footer = formatter.format_footer(files_processed);
            let mut writer_guard = writer.lock().unwrap();
            writer_guard.write_all(footer.as_bytes()).map_err(|e| {
                ArchiveError::io_error(e, "Failed to write footer")
            })?;
        }

        // Ensure all data is written to disk
        if let Err(e) = writer.lock().unwrap().flush() {
            return Err(ArchiveError::io_error(e, "Failed to flush output file"));
        }

        // Create and return statistics
        let duration = start_time.elapsed();
        let stats = ArchiveStats {
            files_processed: self.files_processed.load(Ordering::SeqCst),
            files_skipped: self.files_skipped.load(Ordering::SeqCst),
            error_count: self.error_count.load(Ordering::SeqCst),
            total_size: self.total_size.load(Ordering::SeqCst),
            duration,
            output_path: PathBuf::from(&self.config.output),
        };

        Ok(stats)
    }

    /// Process files sequentially.
    fn process_files_sequential<W: Write + Send + 'static>(
        &self,
        formatter: &dyn FormatterTrait,
        writer: &Arc<Mutex<W>>,
    ) -> Result<(), ArchiveError> {
        // Collect all files to process
        let files = self.collect_files()?;
        let total_files = files.len();
        
        // Process each file sequentially
        for (i, file) in files.into_iter().enumerate() {
            if let Err(e) = self.process_single_file(&file, formatter, writer) {
                error!("Error processing file {}: {}", file.display(), e);
                self.error_count.fetch_add(1, Ordering::SeqCst);
                continue;
            }
            
            // Update progress
            if i % 10 == 0 || i == total_files - 1 {
                info!("Processed {}/{} files", i + 1, total_files);
            }
        }
        
        Ok(())
    }

    /// Process files in parallel using Rayon's work-stealing thread pool.
    fn process_files_parallel<W: Write + Send + 'static>(
        &self,
        formatter: &dyn FormatterTrait,
        writer: &Arc<Mutex<W>>,
    ) -> Result<(), ArchiveError> {
        // Collect all files to process
        let files = self.collect_files()?;
        let total_files = files.len();
        
        if total_files == 0 {
            info!("No files to process");
            return Ok(());
        }

        info!("Starting parallel processing of {} files...", total_files);
        let processed_count = AtomicUsize::new(0);
        
        // Process files in parallel using Rayon
        files.par_iter()
            .map(|file| {
                // Process the file to a buffer
                let mut buffer = Vec::new();
                match self.process_single_file_to_buffer(file, formatter, &mut buffer, &processed_count) {
                    Ok(_) => {
                        debug!("Processed {}", file.display());
                        Ok((file.clone(), buffer))
                    },
                    Err(e) => {
                        error!("Error processing file {}: {}", file.display(), e);
                        self.error_count.fetch_add(1, Ordering::SeqCst);
                        Err((file.clone(), e))
                    }
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
            .try_for_each(|result| {
                match result {
                    Ok((file_path, buffer)) => {
                        // Write the buffer to the output file with a lock
                        let mut writer_guard = match writer.lock() {
                            Ok(guard) => guard,
                            Err(_) => return Err(ArchiveError::Other("Failed to acquire write lock".to_string())),
                        };
                        
                        if let Err(e) = writer_guard.write_all(&buffer) {
                            error!("Failed to write file {}: {}", file_path.display(), e);
                            self.error_count.fetch_add(1, Ordering::SeqCst);
                            return Err(ArchiveError::io_error(e, format!("Failed to write file {}", file_path.display())));
                        }
                        
                        Ok(())
                    },
                    Err((file_path, e)) => {
                        // Log the error but continue processing other files
                        error!("Skipping file due to error: {} - {}", file_path.display(), e);
                        Ok(())
                    }
                }
            })?;
        
        Ok(())
    }

    /// Collect all files that should be included in the archive based on the configuration.
    fn collect_files(&self) -> Result<Vec<PathBuf>, ArchiveError> {
        let mut entries = Vec::new();
        let mut filter_stats = FilterStatistics::default();
        
        // Get LLM ignore patterns if enabled
        let llm_patterns = if self.config.llm_optimize {
            crate::config::Config::get_default_llm_ignore_patterns()
        } else {
            Vec::new()
        };
        
        // Get custom exclude patterns
        let empty_patterns = Vec::new();
        let custom_exclude_patterns = self.config.exclude.as_ref().unwrap_or(&empty_patterns);
        
        // Get included extensions
        let included_extensions = self.config.get_included_extensions();
        
        // Configure the walker with all settings before starting iteration
        let mut walker = WalkDir::new(&self.config.input)
            .min_depth(1)
            .follow_links(self.config.follow_links);

        // Apply max depth if specified
        if let Some(max_depth) = self.config.max_depth {
            walker = walker.max_depth(max_depth as usize);
        }

        let walker = walker.into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                // Skip hidden files if not included
                if !self.config.include_hidden {
                    if let Some(name) = e.file_name().to_str() {
                        if name.starts_with('.') {
                            return false;
                        }
                    }
                }
                true
            });

        // Process each entry
        for entry in walker {
            if entry.file_type().is_file() {
                filter_stats.total_files_found += 1;
                let file_path = entry.path();
                let file_path_str = file_path.to_string_lossy();
                let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                
                // Check file extension filtering first
                if let Some(ref allowed_extensions) = included_extensions {
                    if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                        if !allowed_extensions.contains(&ext.to_lowercase()) {
                            filter_stats.excluded_by_extension += 1;
                            continue;
                        }
                    } else {
                        // No extension, exclude if extensions are specified
                        filter_stats.excluded_by_extension += 1;
                        continue;
                    }
                }
                
                // Check LLM optimization patterns
                if self.config.llm_optimize {
                    let mut excluded_by_llm = false;
                    for pattern in &llm_patterns {
                        if Self::matches_glob_pattern(&file_path_str, pattern)
                            || Self::matches_glob_pattern(file_name, pattern)
                        {
                            filter_stats.excluded_by_llm_optimization += 1;
                            excluded_by_llm = true;
                            break;
                        }
                    }
                    if excluded_by_llm {
                        continue;
                    }
                }
                
                // Check custom ignore patterns
                let mut excluded_by_custom = false;
                for pattern in custom_exclude_patterns {
                    if Self::matches_glob_pattern(&file_path_str, pattern)
                        || Self::matches_glob_pattern(file_name, pattern)
                    {
                        filter_stats.excluded_by_ignore_pattern += 1;
                        excluded_by_custom = true;
                        break;
                    }
                }
                if excluded_by_custom {
                    continue;
                }
                
                // Skip files larger than max_file_size if specified
                if let Some(max_size) = self.config.max_file_size {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.len() > max_size {
                            filter_stats.excluded_by_size += 1;
                            continue;
                        }
                    }
                }
                
                // File passed all filters
                filter_stats.files_included += 1;
                if let Ok(metadata) = entry.metadata() {
                    filter_stats.total_size_included += metadata.len() as usize;
                }
                entries.push(entry.into_path());
            }
        }

        // Calculate excluded count
        filter_stats.files_excluded = filter_stats.total_files_found - filter_stats.files_included;

        // Display filter statistics if enabled
        if self.config.show_filter_stats {
            self.display_filter_stats(&filter_stats);
        }

        Ok(entries)
    }

    /// Simple glob pattern matching
    fn matches_glob_pattern(text: &str, pattern: &str) -> bool {
        if pattern.starts_with("**/") {
            // Directory pattern - check if path contains the pattern
            let dir_pattern = &pattern[3..];
            if dir_pattern.ends_with('/') {
                let dir_name = &dir_pattern[..dir_pattern.len() - 1];
                return text.contains(&format!("/{}/", dir_name)) || text.contains(dir_name);
            } else {
                return text.contains(dir_pattern);
            }
        }

        if pattern.ends_with('/') {
            // Directory pattern
            let dir_pattern = &pattern[..pattern.len() - 1];
            return text.contains(dir_pattern);
        }

        if pattern.contains('*') {
            // Wildcard pattern
            if pattern.starts_with('*') && pattern.len() > 1 {
                return text.ends_with(&pattern[1..]);
            }
            if pattern.ends_with('*') && pattern.len() > 1 {
                return text.starts_with(&pattern[..pattern.len() - 1]);
            }
            return text.contains(&pattern.replace('*', ""));
        }

        // Exact match
        text == pattern || text.contains(pattern)
    }

    /// Display filtering statistics with enhanced LLM optimization details
    fn display_filter_stats(&self, stats: &FilterStatistics) {
        println!("\n📊 File Filtering Statistics:");
        println!("   Total files found: {}", stats.total_files_found);
        println!("   Files included: {} 🟢", stats.files_included);
        println!("   Files excluded: {} 🔴", stats.files_excluded);

        if stats.excluded_by_extension > 0 {
            println!("     └─ By extension filter: {}", stats.excluded_by_extension);
        }
        if stats.excluded_by_llm_optimization > 0 {
            println!("     └─ By LLM optimization: {} 🤖", stats.excluded_by_llm_optimization);

            // Show LLM optimization benefits
            if self.config.llm_optimize {
                println!("        ✨ LLM optimization excluded:");
                println!("           • Build artifacts and compiled files");
                println!("           • Dependencies and package manager files");
                println!("           • Cache and temporary files");
                println!("           • IDE and editor configuration");
                println!("           • Binary media files");
                println!("           • Environment and secret files");
                println!("           • Large data files and ML models");
                println!("        📚 This creates cleaner training data focused on source code");
            }
        }
        if stats.excluded_by_ignore_pattern > 0 {
            println!("     └─ By ignore patterns: {}", stats.excluded_by_ignore_pattern);
        }
        if stats.excluded_by_size > 0 {
            println!("     └─ By file size limit: {}", stats.excluded_by_size);
        }

        let inclusion_rate = if stats.total_files_found > 0 {
            (stats.files_included as f64 / stats.total_files_found as f64) * 100.0
        } else {
            0.0
        };
        println!("   Inclusion rate: {:.1}% 📈", inclusion_rate);

        if stats.total_size_included > 0 {
            println!("   Total size included: {:.2} MB 💾", 
                stats.total_size_included as f64 / (1024.0 * 1024.0));
        }

        // Show LLM optimization recommendation
        if !self.config.llm_optimize && stats.files_excluded > 0 {
            println!("\n💡 Tip: Use --llm-optimize flag to automatically exclude");
            println!("   build artifacts, dependencies, and binary files for");
            println!("   cleaner LLM training data preparation.");
        }

        println!();
    }

    /// Process a single file and write its contents to the output.
    fn process_single_file<W: Write + Send>(
        &self,
        path: &Path,
        formatter: &dyn FormatterTrait,
        writer: &Arc<Mutex<W>>,
    ) -> Result<(), ArchiveError> {
        // Check file size limit if specified
        if let Some(max_size) = self.config.max_file_size {
            let metadata = fs::metadata(path).map_err(|e| {
                self.error_count.fetch_add(1, Ordering::SeqCst);
                ArchiveError::io_error(e, format!("Failed to get metadata for {}", path.display()))
            })?;
            
            if metadata.len() > max_size {
                debug!("Skipping large file: {} ({} bytes)", path.display(), metadata.len());
                self.files_skipped.fetch_add(1, Ordering::SeqCst);
                return Ok(());
            }
        }

        // Read file content as binary first to handle both text and binary files
        let content = match fs::read(path) {
            Ok(bytes) => {
                // Try to convert to UTF-8, use lossy conversion if needed
                String::from_utf8_lossy(&bytes).into_owned()
            }
            Err(e) => {
                self.error_count.fetch_add(1, Ordering::SeqCst);
                return Err(ArchiveError::io_error(e, format!("Failed to read file: {}", path.display())));
            }
        };

        // Format the file content
        let formatted = formatter.format_file(path, &content);
        
        // Write the formatted content
        let mut writer_guard = writer.lock().map_err(|_| {
            self.error_count.fetch_add(1, Ordering::SeqCst);
            ArchiveError::Other("Failed to acquire lock on output file".to_string())
        })?;

        if let Err(e) = writer_guard.write_all(formatted.as_bytes()) {
            self.error_count.fetch_add(1, Ordering::SeqCst);
            return Err(ArchiveError::io_error(e, "Failed to write formatted content"));
        }

        // Update statistics atomically
        self.files_processed.fetch_add(1, Ordering::SeqCst);
        if let Ok(metadata) = fs::metadata(path) {
            self.total_size.fetch_add(metadata.len(), Ordering::SeqCst);
        }

        Ok(())
    }

    /// Process a single file and write its formatted content to the given buffer.
    fn process_single_file_to_buffer(
        &self,
        path: &Path,
        formatter: &dyn FormatterTrait,
        buffer: &mut Vec<u8>,
        _file_count: &AtomicUsize,
    ) -> Result<(), ArchiveError> {
        // Read file as binary
        let bytes = fs::read(path)
            .map_err(|e| ArchiveError::Other(e.to_string()))?;
        
        // Convert to string, replacing invalid UTF-8 sequences with replacement characters
        let content = String::from_utf8_lossy(&bytes);
        
        // Get the relative path and convert to string
        let relative_path = path.strip_prefix(&self.config.input)
            .unwrap_or(path);
        
        // Format the file content
        let formatted = formatter.format_file(relative_path, &content);
        
        // Write to buffer
        buffer.extend_from_slice(formatted.as_bytes());
        
        // Update statistics
        self.files_processed.fetch_add(1, Ordering::SeqCst);
        if let Ok(metadata) = fs::metadata(path) {
            self.total_size.fetch_add(metadata.len(), Ordering::SeqCst);
        }
        
        Ok(())
    }
}

/// Archives the given directory to the specified output file.
///
/// This is a convenience function that creates a new `ArchiveEngine` and runs it.
/// For more control over the archiving process, use `ArchiveEngine` directly.
pub fn archive_directory(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    config: &Config,
) -> Result<(), ArchiveError> {
    // Create a new configuration with the provided input and output
    let mut config = config.clone();
    config.input = input.as_ref().to_path_buf();
    config.output = output.as_ref().to_path_buf();
    
    // Create and run the archive engine
    let mut engine = ArchiveEngine::new(config)?;
    engine.run()?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_archive_engine_creation() {
        let config = Config::default();
        let engine = ArchiveEngine::new(config);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_collect_files() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content")?;

        let mut config = Config::default();
        config.input = temp_dir.path().to_path_buf();
        
        let engine = ArchiveEngine::new(config)?;
        let files = engine.collect_files()?;
        
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], test_file);
        
        Ok(())
    }
}