//! Common error types and utilities

use std::{
    io,
    path::Path,
};

use thiserror::Error;

/// Common result type
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Main error type for the application
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Git error
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    /// Path error
    #[error("Path error: {0}")]
    Path(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Custom error
    #[error("{0}")]
    Custom(String),
}

impl Error {
    /// Create a new path error
    pub fn path_error<P: AsRef<Path>>(path: P, message: impl Into<String>) -> Self {
        Self::Path(format!(
            "{}: {}",
            path.as_ref().display(),
            message.into()
        ))
    }

    /// Create a new invalid input error
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    /// Create a new custom error
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom(message.into())
    }

    /// Convert to an I/O error
    pub fn into_io_error(self) -> io::Error {
        match self {
            Error::Io(e) => e,
            _ => io::Error::new(io::ErrorKind::Other, self.to_string()),
        }
    }
}

impl From<Error> for io::Error {
    fn from(err: Error) -> Self {
        err.into_io_error()
    }
}

impl From<tempfile::PersistError> for Error {
    fn from(err: tempfile::PersistError) -> Self {
        Error::Io(err.error)
    }
}

/// Extension trait for adding context to Results
pub trait ResultExt<T, E> {
    /// Add context to an error
    fn context(self, context: impl Into<String>) -> Result<T>;
}

impl<T, E: Into<Error>> ResultExt<T, E> for std::result::Result<T, E> {
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|e| {
            let mut err: Error = e.into();
            match &mut err {
                Error::Custom(msg) => {
                    *msg = format!("{}: {}", context.into(), msg);
                }
                _ => {
                    err = Error::Custom(format!("{}: {}", context.into(), err));
                }
            }
            err
        })
    }
}

/// Extension trait for IO results
pub trait IoResultExt<T> {
    /// Add context to an IO error
    fn with_context<F, S>(self, context: F) -> Result<T>
    where
        F: FnOnce() -> S,
        S: Into<String>;
}

impl<T> IoResultExt<T> for std::result::Result<T, io::Error> {
    fn with_context<F, S>(self, context: F) -> Result<T>
    where
        F: FnOnce() -> S,
        S: Into<String>,
    {
        self.map_err(|e| Error::Io(io::Error::new(e.kind(), context().into())))
    }
}
