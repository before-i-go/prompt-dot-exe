//! Path extension utilities

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

/// Extension methods for path extensions
pub trait ExtensionExt {
    /// Get the file extension as a string
    fn extension_str(&self) -> Option<&str>;

    /// Check if the path has the given extension
    fn has_extension(&self, ext: &str) -> bool;

    /// Replace the file extension
    fn with_extension<S: AsRef<OsStr>>(&self, extension: S) -> PathBuf;
}

impl ExtensionExt for Path {
    fn extension_str(&self) -> Option<&str> {
        self.extension().and_then(OsStr::to_str)
    }

    fn has_extension(&self, ext: &str) -> bool {
        self.extension_str().map_or(false, |e| e == ext)
    }

    fn with_extension<S: AsRef<OsStr>>(&self, extension: S) -> PathBuf {
        self.with_extension(extension)
    }
}
