//! Filesystem metadata utilities

use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::error::{Error, Result};

/// Filesystem metadata
#[derive(Debug, Clone)]
pub struct Metadata {
    path: PathBuf,
    len: u64,
    modified: SystemTime,
    is_file: bool,
    is_dir: bool,
}

impl Metadata {
    /// Get the path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the file size in bytes
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Check if the file is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the last modification time
    pub fn modified(&self) -> Result<SystemTime> {
        Ok(self.modified)
    }

    /// Check if the path is a file
    pub fn is_file(&self) -> bool {
        self.is_file
    }

    /// Check if the path is a directory
    pub fn is_dir(&self) -> bool {
        self.is_dir
    }
}

/// Get metadata for a path
pub fn metadata<P: AsRef<Path>>(path: P) -> Result<Metadata> {
    let path = path.as_ref();
    let meta = fs::metadata(path).map_err(|e| Error::path_error(path, e.to_string()))?;

    Ok(Metadata {
        path: path.to_path_buf(),
        len: meta.len(),
        modified: meta.modified()
            .map_err(|e| Error::Io(std::io::Error::new(e.kind(), format!("failed to get modified time for {}", path.display()))))?,
        is_file: meta.is_file(),
        is_dir: meta.is_dir(),
    })
}
