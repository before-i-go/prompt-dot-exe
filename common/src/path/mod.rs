//! Path manipulation utilities

mod extension;
mod name;

pub use extension::ExtensionExt;
pub use name::NameExt;

use std::path::{Path, PathBuf};

/// Extension trait for Path/PathBuf
pub trait PathExt {
    /// Check if path has any of the given extensions
    fn has_extension_in<S: AsRef<str>>(&self, exts: &[S]) -> bool;

    /// Get the canonical path
    fn canonicalize_path(&self) -> std::io::Result<PathBuf>;

    /// Get the absolute path
    fn absolute_path(&self) -> std::io::Result<PathBuf>;
}

impl PathExt for Path {
    fn has_extension_in<S: AsRef<str>>(&self, exts: &[S]) -> bool {
        self.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| exts.iter().any(|e| e.as_ref() == ext))
            .unwrap_or(false)
    }

    fn canonicalize_path(&self) -> std::io::Result<PathBuf> {
        dunce::canonicalize(self)
    }

    fn absolute_path(&self) -> std::io::Result<PathBuf> {
        if self.is_absolute() {
            Ok(self.to_path_buf())
        } else {
            std::env::current_dir().map(|cwd| cwd.join(self))
        }
    }
}

impl PathExt for PathBuf {
    fn has_extension_in<S: AsRef<str>>(&self, exts: &[S]) -> bool {
        self.as_path().has_extension_in(exts)
    }

    fn canonicalize_path(&self) -> std::io::Result<PathBuf> {
        self.as_path().canonicalize_path()
    }

    fn absolute_path(&self) -> std::io::Result<PathBuf> {
        self.as_path().absolute_path()
    }
}
