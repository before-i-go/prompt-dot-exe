use std::path::{Path, PathBuf};
use std::fmt;
use thiserror::Error;
use std::error::Error as StdError;

/// Main error type for the archive-to-txt library
/// 
/// This error type provides detailed information about what went wrong during
/// the archiving process, with support for chaining multiple errors together.
#[derive(Debug, Error)]
pub enum ArchiveError {
    /// I/O operation failed
    #[error("I/O error: {context}")]
    Io {
        /// The underlying I/O error
        #[source]
        source: std::io::Error,
        /// Context about where the error occurred
        context: String,
    },

    /// Directory walking error
    #[error("Failed to walk directory: {path:?}")]
    WalkDir {
        /// The underlying walkdir error
        #[source]
        source: walkdir::Error,
        /// The path being walked when the error occurred
        path: PathBuf,
    },

    /// File exceeds maximum allowed size
    #[error("File '{path:?}' exceeds maximum size of {} bytes (was {} bytes)", max_size, actual_size)]
    FileTooLarge {
        /// Path to the file that's too large
        path: PathBuf,
        /// Maximum allowed file size in bytes
        max_size: u64,
        /// Actual file size in bytes
        actual_size: u64,
    },

    /// Invalid or inaccessible path
    #[error("Invalid path: {path:?} - {reason}")]
    InvalidPath {
        /// The invalid path
        path: PathBuf,
        /// Reason why the path is invalid
        reason: String,
    },

    /// Git operation failed (only available with `git` feature)
    #[cfg(feature = "git")]
    #[error("Git operation failed: {message}")]
    Git {
        /// The underlying git error
        #[source]
        source: git2::Error,
        /// Context about the git operation
        message: String,
    },

    /// Serialization/Deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid configuration
    #[error("Configuration error: {0}")]
    Config(String),

    /// Pattern matching error
    #[error("Pattern error: {0}")]
    Pattern(String),

    /// Multiple errors occurred
    #[error("Multiple errors occurred:
{errors}")]
    Multiple {
        /// List of errors that occurred
        errors: Vec<ArchiveError>,
    },

    /// Other kinds of errors
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// A specialized `Result` type for archive operations
pub type Result<T> = std::result::Result<T, ArchiveError>;

impl ArchiveError {
    /// Create a new I/O error with context
    pub fn io_error(source: std::io::Error, context: impl Into<String>) -> Self {
        ArchiveError::Io {
            source,
            context: context.into(),
        }
    }

    /// Create a new path validation error
    pub fn invalid_path(path: impl AsRef<Path>, reason: impl Into<String>) -> Self {
        ArchiveError::InvalidPath {
            path: path.as_ref().to_path_buf(),
            reason: reason.into(),
        }
    }

    /// Create a new file too large error
    pub fn file_too_large(path: impl AsRef<Path>, max_size: u64, actual_size: u64) -> Self {
        ArchiveError::FileTooLarge {
            path: path.as_ref().to_path_buf(),
            max_size,
            actual_size,
        }
    }

    /// Convert multiple errors into a single `Multiple` error
    pub fn multiple(errors: impl IntoIterator<Item = Self>) -> Self {
        let errors: Vec<_> = errors.into_iter().collect();
        if errors.len() == 1 {
            errors.into_iter().next().unwrap()
        } else {
            ArchiveError::Multiple { errors }
        }
    }
}

// Conversion from std::io::Error to ArchiveError
impl From<std::io::Error> for ArchiveError {
    fn from(err: std::io::Error) -> Self {
        ArchiveError::Io {
            source: err,
            context: "I/O operation failed".to_string(),
        }
    }
}

// Conversion from walkdir::Error to ArchiveError
impl From<walkdir::Error> for ArchiveError {
    fn from(err: walkdir::Error) -> Self {
        let path = err.path().unwrap_or_else(|| Path::new("")).to_path_buf();
        ArchiveError::WalkDir { source: err, path }
    }
}

// Conversion from glob::PatternError to ArchiveError
impl From<glob::PatternError> for ArchiveError {
    fn from(err: glob::PatternError) -> Self {
        ArchiveError::InvalidPath {
            path: PathBuf::new(),
            reason: err.to_string(),
        }
    }
}

// Conversion from glob::GlobError to ArchiveError
impl From<glob::GlobError> for ArchiveError {
    fn from(err: glob::GlobError) -> Self {
        ArchiveError::InvalidPath {
            path: err.path().unwrap_or_else(|| Path::new("")).to_path_buf(),
            reason: err.error().to_string(),
        }
    }
}

// Conversion from anyhow::Error to ArchiveError
impl From<anyhow::Error> for ArchiveError {
    fn from(err: anyhow::Error) -> Self {
        if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
            return ArchiveError::io_error(io_err.clone(), "I/O operation failed");
        }
        ArchiveError::Other(err.to_string())
    }
}

// Helper to format multiple errors with indentation
pub(crate) fn format_multiple_errors(errors: &[ArchiveError]) -> String {
    let mut result = String::new();
    for (i, error) in errors.iter().enumerate() {
        let prefix = format!("{}. ", i + 1);
        let indented = textwrap::indent(&error.to_string(), "  ");
        result.push_str(&format!("{}{}", prefix, indented.trim_start()));
        if i < errors.len() - 1 {
            result.push('\n');
        }
    }
    result
}
