//! A library for archiving code directories with filtering and formatting options.

use std::path::PathBuf;
use std::collections::HashSet;

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
    
    /// File patterns to include (default: all files)
    pub includes: Option<Vec<String>>,
    
    /// File patterns to exclude (default: none)
    pub excludes: Option<Vec<String>>,
    
    /// File extensions to include (default: all extensions)
    pub extensions: Option<Vec<String>>,
    
    /// Maximum file size in bytes (default: no limit)
    pub max_file_size: Option<u64>,
    
    /// Whether to follow symbolic links (default: false)
    pub follow_links: bool,
    
    /// Whether to include hidden files (default: false)
    pub include_hidden: bool,
    
    /// Whether to respect .gitignore files (default: true)
    pub respect_gitignore: bool,
}

impl Default for ArchiveConfig {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from("."),
            includes: None,
            excludes: None,
            extensions: None,
            max_file_size: None,
            follow_links: false,
            include_hidden: false,
            respect_gitignore: true,
        }
    }
}

/// Represents a file entry in the archive
#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    /// Relative path from the root directory
    pub path: String,
    
    /// File size in bytes
    pub size: u64,
    
    /// Last modified timestamp (ISO 8601 format)
    pub modified: String,
    
    /// File extension (without the dot)
    pub extension: Option<String>,
}

/// The main archiver struct
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
        
        Ok(Self { config })
    }
    
    /// Generate an archive of the configured directory
    #[instrument(skip(self))]
    pub fn create_archive(&self) -> Result<Vec<FileEntry>> {
        let mut entries = Vec::new();
        let root = self.config.root_dir.canonicalize()?;
        
        info!("Creating archive for directory: {}", root.display());
        
        let mut walker = WalkBuilder::new(&root);
        walker
            .hidden(!self.config.include_hidden)
            .ignore(self.config.respect_gitignore)
            .git_ignore(self.config.respect_gitignore)
            .follow_links(self.config.follow_links);
            
        // Apply includes and excludes if specified
        if let Some(includes) = &self.config.includes {
            for pattern in includes {
                walker.add_ignore(pattern);
            }
        }
        
        if let Some(excludes) = &self.config.excludes {
            for pattern in excludes {
                walker.add_ignore(pattern);
            }
        }
        
        // Convert extensions to a set for faster lookup
        let extensions: Option<HashSet<_>> = self.config.extensions
            .as_ref()
            .map(|exts| exts.iter().map(|s| s.to_lowercase()).collect());
        
        for result in walker.build() {
            let entry = result?;
            let path = entry.path();
            
            // Skip directories
            if !path.is_file() {
                continue;
            }
            
            // Skip files that don't match the extension filter
            if let Some(exts) = &extensions {
                if let Some(ext) = path.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_lowercase())
                {
                    if !exts.contains(&ext) {
                        continue;
                    }
                } else {
                    // Skip files without an extension if we're filtering by extension
                    continue;
                }
            }
            
            // Skip files larger than the maximum size
            let metadata = path.metadata()?;
            if let Some(max_size) = self.config.max_file_size {
                if metadata.len() > max_size {
                    debug!("Skipping large file: {} ({} bytes)", path.display(), metadata.len());
                    continue;
                }
            }
            
            // Create a file entry
            let rel_path = path.strip_prefix(&root)
                .map_err(|_| ArchiveError::InvalidPath(format!(
                    "Failed to get relative path for: {}",
                    path.display()
                )))?;
                
            let modified = metadata.modified()?;
            let modified_str = chrono::DateTime::<chrono::Local>::from(modified)
                .to_rfc3339();
                
            let extension = path.extension()
                .and_then(|ext| ext.to_str())
                .map(|s| s.to_string());
                
            let entry = FileEntry {
                path: rel_path.to_string_lossy().into_owned(),
                size: metadata.len(),
                modified: modified_str,
                extension,
            };
            
            debug!("Adding file to archive: {}", entry.path);
            entries.push(entry);
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
            max_file_size: Some(100), // 100 bytes
            ..Default::default()
        };
        
        let archiver = CodeArchiver::new(config)?;
        let entries = archiver.create_archive()?;
        
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "small.txt");
        
        Ok(())
    }
}
