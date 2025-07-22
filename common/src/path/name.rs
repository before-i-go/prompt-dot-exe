//! Path name utilities

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

/// Extension methods for path names
pub trait NameExt: AsRef<Path> {
    /// Get the file name without extension
    fn file_stem_str(&self) -> Option<&str> {
        self.as_ref().file_stem().and_then(OsStr::to_str)
    }

    /// Get the file name with extension
    fn file_name_str(&self) -> Option<&str> {
        self.as_ref().file_name().and_then(OsStr::to_str)
    }

    /// Get the parent directory as a string
    fn parent_str(&self) -> Option<&str> {
        self.as_ref().parent().and_then(Path::to_str)
    }

    /// Get the parent directory as a Path
    fn parent_path(&self) -> Option<PathBuf> {
        self.as_ref().parent().map(ToOwned::to_owned)
    }

    /// Check if the path has the given file name
    fn has_file_name(&self, name: &str) -> bool {
        self.file_name_str().map_or(false, |n| n == name)
    }

    /// Check if the path has the given stem (file name without extension)
    fn has_stem(&self, stem: &str) -> bool {
        self.file_stem_str().map_or(false, |s| s == stem)
    }
}

// Implement for common path-like types
impl<T: AsRef<Path>> NameExt for T {}
