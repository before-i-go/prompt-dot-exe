//! File operations

use std::{
    fs,
    path::Path,
};

use crate::error::{Error, Result};

/// Read the entire contents of a file into a string
pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    let path_ref = path.as_ref();
    fs::read_to_string(path_ref)
        .map_err(|e| Error::path_error(path_ref, e.to_string()))
}

/// Write a slice as the entire contents of a file
pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<()> {
    let path_ref = path.as_ref();
    fs::write(path_ref, contents)
        .map_err(|e| Error::path_error(path_ref, e.to_string()))
}

/// Create a directory and all of its parent components if they are missing
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
    let path_ref = path.as_ref();
    fs::create_dir_all(path_ref)
        .map_err(|e| Error::path_error(path_ref, e.to_string()))
}

/// Check if a path exists and is a file
pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_file()
}

/// Check if a path exists and is a directory
pub fn is_dir<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_dir()
}

/// Create a temporary directory
pub fn temp_dir() -> Result<tempfile::TempDir> {
    tempfile::tempdir().map_err(|e| Error::Io(e.into()))
}

/// Create a temporary file
pub fn temp_file() -> Result<tempfile::NamedTempFile> {
    tempfile::NamedTempFile::new().map_err(|e| Error::Io(e.into()))
}

/// Copy a file from one location to another
pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<u64> {
    let from_path = from.as_ref();
    let to_path = to.as_ref();
    fs::copy(from_path, to_path)
        .map_err(|e| Error::path_error(from_path, format!("failed to copy to {}: {}", to_path.display(), e)))
}

/// Move a file from one location to another
pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
    let from_path = from.as_ref();
    let to_path = to.as_ref();
    fs::rename(from_path, to_path)
        .map_err(|e| Error::path_error(from_path, format!("failed to move to {}: {}", to_path.display(), e)))
}
