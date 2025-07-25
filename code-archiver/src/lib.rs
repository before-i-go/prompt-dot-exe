//! A library for archiving code directories with filtering and formatting options.

pub mod git;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;

use std::borrow::Cow;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use globset::{Glob, GlobSetBuilder};
use std::sync::{Arc, Mutex};
use ignore::WalkBuilder;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use tracing::{info, debug, instrument};

/// Custom error type for code archiving operations
#[derive(Error, Debug)]
pub enum ArchiveError {
    /// I/O error occurred
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    Config(String),
    
    /// Invalid path
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    
    /// Pattern matching error
    #[error("Pattern error: {0}")]
    Pattern(#[from] glob::PatternError),
    
    /// Ignore error
    #[error("Ignore error: {0}")]
    Ignore(#[from] ignore::Error),
}

/// Result type for archiving operations
pub type Result<T> = std::result::Result<T, ArchiveError>;

/// Configuration for the code archiver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveConfig {
    /// The root directory to archive
    pub root_dir: PathBuf,
    
    /// File patterns to include (supports glob format)
    pub include: Option<Vec<String>>,
    
    /// File patterns to exclude (supports glob format)
    pub exclude: Option<Vec<String>>,
    
    /// File extensions to include (without leading .)
    pub extensions: Option<Vec<String>>,
    
    /// Maximum file size in bytes
    pub max_size: Option<u64>,
    
    /// Whether to follow symbolic links
    pub follow_links: bool,
    
    /// Whether to include hidden files (starting with .)
    pub hidden: bool,
    
    /// Whether to respect .gitignore files (requires git to be installed)
    pub gitignore: bool,
    
    /// Whether to include Git status information in the output
    pub include_git_status: bool,
    
    /// Whether to include Git-ignored files
    pub include_ignored: bool,
}

impl Default for ArchiveConfig {
    fn default() -> Self {
        Self {
            root_dir: ".".into(),
            include: None,
            exclude: None,
            extensions: None,
            max_size: None,
            follow_links: false,
            hidden: false,
            gitignore: true,
            include_git_status: true,
            include_ignored: false,
        }
    }
}

/// Represents a file entry in the archive
#[derive(Debug, Serialize)]
pub struct FileEntry {
    /// Relative path from the root directory
    pub path: String,
    
    /// File size in bytes
    pub size: u64,
    
    /// Last modification time
    pub modified: String,
    
    /// File extension (without the dot)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension: Option<String>,
    
    /// Git status if available and enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_status: Option<String>,
}

/// The main archiver struct
#[derive(Debug)]
pub struct CodeArchiver {
    config: ArchiveConfig,
}

impl CodeArchiver {
    /// Create a new CodeArchiver with the given configuration
    pub fn new(config: ArchiveConfig) -> Result<Self> {
        if !config.root_dir.exists() {
            return Err(ArchiveError::InvalidPath(format!(
                "Root directory does not exist: {}",
                config.root_dir.display()
            )));
        }
        
        if !config.root_dir.is_dir() {
            return Err(ArchiveError::InvalidPath(format!(
                "Root path is not a directory: {}",
                config.root_dir.display()
            )));
        }
        
        // Validate include patterns
        if let Some(patterns) = &config.include {
            for pattern in patterns {
                glob::Pattern::new(pattern)?;
            }
        }
        
        // Validate exclude patterns
        if let Some(patterns) = &config.exclude {
            for pattern in patterns {
                glob::Pattern::new(pattern)?;
            }
        }
        
        Ok(Self { config })
    }
    
    /// Create an archive of the configured directory
    #[instrument(skip(self))]
    pub fn create_archive(&self) -> Result<Vec<FileEntry>> {
        let mut entries = Vec::new();
        
        // Clone configuration values needed for the filter
        let exclude_patterns = self.config.exclude.clone();
        let include_patterns = self.config.include.clone();
        let include_git_status = self.config.include_git_status;
        let include_ignored = self.config.include_ignored;
        let use_git = self.config.include_git_status || self.config.gitignore;

        // Configure the directory walker
        let mut walker = WalkBuilder::new(&self.config.root_dir);
        
        // Apply configuration to walker
        walker
            .hidden(!self.config.hidden)
            .follow_links(self.config.follow_links)
            .git_ignore(self.config.gitignore);

        // Include patterns are handled in the filter_entry closure below

        // Add exclude patterns for common directories
        let walker = walker.filter_entry(move |e| {
            let path = e.path();
            let path_str = path.to_string_lossy();
            
            // Skip common directories
            if path_str.contains("/target/") || 
               path_str.contains("/node_modules/") || 
               path_str.contains("/.git/")
            {
                return false;
            }
            
            // Skip root level directories
            if let Some(name) = path.file_name() {
                let name = name.to_string_lossy();
                if name == "target" || name == "node_modules" || name == ".git" {
                    return false;
                }
            }
            
            // For directories, always include them to allow traversal
            if e.file_type().map_or(false, |ft| ft.is_dir()) {
                tracing::debug!("Including directory '{}' for traversal", path_str);
                return true;
            }
            
            // For files, check against include patterns
            if let Some(includes) = &include_patterns {
                if includes.is_empty() {
                    return true; // No include patterns means include everything
                }
                
                let path = path.to_string_lossy();
                tracing::debug!("Checking include patterns for path: {}", path);
                
                // Check each pattern individually for better debugging
                let mut matched = false;
                
                for pattern in includes {
                    match Glob::new(pattern) {
                        Ok(glob) => {
                            let matcher = glob.compile_matcher();
                            let path_str = path.as_ref();
                            let matches = matcher.is_match(path_str);
                            
                            tracing::debug!("Pattern '{}' matches '{}': {}", pattern, path, matches);
                            
                            if matches {
                                matched = true;
                                break;
                            }
                            
                            // Also try with a leading "./"
                            let path_with_dot = format!("./{}", path);
                            let matches_with_dot = matcher.is_match(&path_with_dot);
                            
                            tracing::debug!("Pattern '{}' matches '{}': {}", pattern, path_with_dot, matches_with_dot);
                            
                            if matches_with_dot {
                                matched = true;
                                break;
                            }
                        },
                        Err(e) => {
                            tracing::warn!("Invalid glob pattern '{}': {}", pattern, e);
                        }
                    }
                }
                
                if !matched && !includes.is_empty() {
                    tracing::debug!("Excluding '{}' - no matching include patterns", path);
                    return false;
                }
                
                tracing::debug!("Including '{}' - matched include pattern", path);
            }
            
            // Apply custom exclude patterns
            if let Some(excludes) = &exclude_patterns {
                // Compile all exclude patterns
                let mut glob_builder = GlobSetBuilder::new();
                let mut has_valid_patterns = false;
                
                for pattern in excludes {
                    match Glob::new(pattern) {
                        Ok(glob) => {
                            glob_builder.add(glob);
                            has_valid_patterns = true;
                        },
                        Err(e) => {
                            tracing::warn!("Invalid exclude pattern '{}': {}", pattern, e);
                        }
                    }
                }
                
                // Only check patterns if we have at least one valid pattern
                if has_valid_patterns {
                    // Build the glob set
                    if let Ok(glob_set) = glob_builder.build() {
                        let path = Path::new(path_str.as_ref());
                        if glob_set.is_match(path) {
                            tracing::debug!("Excluding '{}' - matched exclude pattern", path_str);
                            return false;
                        }
                        
                        // Also check with a leading "./"
                        let path_with_dot = Path::new(".").join(path);
                        if glob_set.is_match(&path_with_dot) {
                            tracing::debug!("Excluding '{}' - matched exclude pattern with leading './'", path_str);
                            return false;
                        }
                    }
                }
            }
            
            true
        });
        
        // Handle Git ignore if needed
        let walker = if use_git && !include_ignored {
            match git::GitContext::open(&self.config.root_dir) {
                Ok(Some(git_ctx)) => {
                    let git_ctx = Arc::new(Mutex::new(git_ctx));
                    walker.filter_entry(move |e| {
                        if e.file_type().map_or(false, |ft| !ft.is_dir()) {
                            if let Ok(ctx) = git_ctx.lock() {
                                if let Ok(true) = ctx.is_ignored(e.path()) {
                                    return false;
                                }
                            }
                        }
                        true
                    })
                },
                Ok(None) => walker,
                Err(e) => {
                    tracing::warn!("Failed to initialize Git context: {}", e);
                    walker
                }
            }
        } else {
            walker
        };

        // Process each file in the directory
        for result in walker.build() {
            let entry = match result {
                Ok(entry) => entry,
                Err(err) => {
                    tracing::warn!("Error reading directory entry: {}", err);
                    continue;
                }
            };
            
            // Skip directories
            let file_type = match entry.file_type() {
                Some(ft) => ft,
                None => {
                    tracing::warn!("Could not determine file type for: {}", entry.path().display());
                    continue;
                }
            };
            
            if file_type.is_dir() {
                continue;
            }

            let path = entry.path();
            
            // Skip Git metadata directories
            if path.components().any(|c| c.as_os_str() == ".git") {
                continue;
            }

            // Get file metadata
            let metadata = entry.metadata().map_err(|e| {
                let io_err = std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string()
                );
                ArchiveError::Io(io_err)
            })?;
            
            // Skip if file is too large
            if let Some(max_size) = self.config.max_size {
                if metadata.len() > max_size {
                    continue;
                }
            }
            

            
            // Get file extension if any
            let extension = path.extension()
                .and_then(|ext| ext.to_str())
                .map(|s| s.to_lowercase());
            
            // Skip if extension filtering is enabled and file doesn't match
            if let Some(extensions) = &self.config.extensions {
                if let Some(ref ext) = extension {
                    if !extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                        continue;
                    }
                } else {
                    // No extension but extensions are required
                    continue;
                }
            }
            
            // Get Git status if enabled
            let git_status: Option<String> = if include_git_status {
                if let Ok(Some(git_ctx)) = git::GitContext::open(&self.config.root_dir) {
                    git_ctx.get_status(path).ok().flatten().map(|s| s.to_string())
                } else {
                    None
                }
            } else {
                None
            };
            
            // Get relative path
            let rel_path = path.strip_prefix(&self.config.root_dir)
                .map_err(|_| ArchiveError::InvalidPath("Failed to get relative path".to_string()))?;
            
            // Convert to string
            let path_str = rel_path.to_string_lossy().to_string();
            
            // Get modification time
            let modified = metadata.modified()
                .map_err(ArchiveError::Io)?
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| ArchiveError::InvalidPath("Failed to get modification time".to_string()))?;
            
            let modified = chrono::DateTime::<chrono::Utc>::from(
                std::time::UNIX_EPOCH + modified
            ).to_rfc3339();
            
            // Add to entries
            let file_entry = FileEntry {
                path: path_str,
                size: metadata.len(),
                modified,
                extension,
                git_status,
            };
            
            debug!("Adding file to archive: {}", path.display());
            entries.push(file_entry);
        }
        
        info!("Archive created with {} files", entries.len());
        Ok(entries)
    }
    
    /// Generate a JSON representation of the archive
    pub fn archive_to_json(&self) -> Result<String> {
        let entries = self.create_archive()?;
        serde_json::to_string_pretty(&entries)
            .map_err(|e| ArchiveError::Config(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    
    #[test]
    fn test_archive_empty_dir() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            
        let config = ArchiveConfig {
            root_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let archiver = CodeArchiver::new(config)?;
        let entries = archiver.create_archive()?;
        assert!(entries.is_empty());
        
        Ok(())
    }
    
    #[test]
    fn test_archive_with_files() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            
        let file1 = temp_dir.child("test1.txt");
        file1.touch().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        let subdir = temp_dir.child("subdir");
        subdir.create_dir_all().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        let file2 = subdir.child("test2.rs");
        file2.touch().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        let config = ArchiveConfig {
            root_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let archiver = CodeArchiver::new(config)?;
        let entries = archiver.create_archive()?;
        
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|e| e.path == "test1.txt"));
        assert!(entries.iter().any(|e| e.path == "subdir/test2.rs"));
        
        Ok(())
    }
    
    #[test]
    fn test_archive_with_extension_filter() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            
        let file1 = temp_dir.child("test1.txt");
        file1.touch().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        let file2 = temp_dir.child("test2.rs");
        file2.touch().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        let config = ArchiveConfig {
            root_dir: temp_dir.path().to_path_buf(),
            extensions: Some(vec!["rs".to_string()]),
            ..Default::default()
        };
        
        let archiver = CodeArchiver::new(config)?;
        let entries = archiver.create_archive()?;
        
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "test2.rs");
        
        Ok(())
    }
    
    #[test]
    fn test_archive_with_size_filter() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        // Create a small file
        let small_file = temp_dir.child("small.txt");
        small_file.write_str("small file")
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        // Create a larger file
        let large_file = temp_dir.child("large.txt");
        let content = "x".repeat(1024); // 1KB
        large_file.write_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        let config = ArchiveConfig {
            root_dir: temp_dir.path().to_path_buf(),
            max_size: Some(100), // 100 bytes
            ..Default::default()
        };
        
        let archiver = CodeArchiver::new(config)?;
        let entries = archiver.create_archive()?;
        
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "small.txt");
        
        Ok(())
    }
    
    #[test]
    fn test_archive_ignores_common_directories() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            
        // Create regular files
        let file1 = temp_dir.child("file1.txt");
        file1.touch().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        // Create common directories that should be ignored
        let target_dir = temp_dir.child("target");
        target_dir.create_dir_all().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let target_file = target_dir.child("lib.rs");
        target_file.touch().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        let node_modules = temp_dir.child("node_modules");
        node_modules.create_dir_all().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let node_file = node_modules.child("index.js");
        node_file.touch().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        let config = ArchiveConfig {
            root_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let archiver = CodeArchiver::new(config)?;
        let entries = archiver.create_archive()?;
        
        // Should only include the top-level file, not files in target/ or node_modules/
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "file1.txt");
        
        Ok(())
    }
}
