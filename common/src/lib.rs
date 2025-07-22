//! Common utilities for code tools

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]

pub mod error;
pub mod fs;
pub mod path;

// Re-exports
pub use error::{Error, Result};
