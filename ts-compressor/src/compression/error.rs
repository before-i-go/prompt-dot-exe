//! Error types for compression operations
//! 
//! Provides comprehensive error handling with context-rich error messages
//! and proper error chaining for debugging and user feedback.

use std::path::PathBuf;
use thiserror::Error;

/// Comprehensive error type for all compression operations
#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("Pattern analysis failed: {message}")]
    PatternAnalysis { message: String },
    
    #[error("Dictionary building failed: {message}")]
    DictionaryBuild { message: String },
    
    #[error("Token generation overflow - too many patterns to compress")]
    TokenOverflow,
    
    #[error("Pattern replacement failed: {message}")]
    PatternReplacement { message: String },
    
    #[error("Zstd compression failed: {source}")]
    ZstdCompression {
        #[source]
        source: std::io::Error,
    },
    
    #[error("Output file creation failed: {path}")]
    OutputCreation { path: PathBuf },
    
    #[error("File processing failed: {path} - {message}")]
    FileProcessing { path: PathBuf, message: String },
    
    #[error("Configuration validation failed: {message}")]
    ConfigValidation { message: String },
    
    #[error("Integrity check failed: {message}")]
    IntegrityCheck { message: String },
    
    #[error("Git operation failed")]
    GitOperation {
        #[from]
        source: git2::Error,
    },
    
    #[error("IO operation failed")]
    Io {
        #[from]
        source: std::io::Error,
    },
}

impl CompressionError {
    /// Create a pattern analysis error with context
    pub fn pattern_analysis<S: Into<String>>(message: S) -> Self {
        Self::PatternAnalysis {
            message: message.into(),
        }
    }
    
    /// Create a dictionary build error with context
    pub fn dictionary_build<S: Into<String>>(message: S) -> Self {
        Self::DictionaryBuild {
            message: message.into(),
        }
    }
    
    /// Create a pattern replacement error with context
    pub fn pattern_replacement<S: Into<String>>(message: S) -> Self {
        Self::PatternReplacement {
            message: message.into(),
        }
    }
    
    /// Create an output creation error with path context
    pub fn output_creation<P: Into<PathBuf>>(path: P) -> Self {
        Self::OutputCreation { path: path.into() }
    }
    
    /// Create a file processing error with path and message context
    pub fn file_processing<P: Into<PathBuf>, S: Into<String>>(path: P, message: S) -> Self {
        Self::FileProcessing {
            path: path.into(),
            message: message.into(),
        }
    }
    
    /// Create a configuration validation error with context
    pub fn config_validation<S: Into<String>>(message: S) -> Self {
        Self::ConfigValidation {
            message: message.into(),
        }
    }
    
    /// Create an integrity check error with context
    pub fn integrity_check<S: Into<String>>(message: S) -> Self {
        Self::IntegrityCheck {
            message: message.into(),
        }
    }
    
    /// Create a zstd compression error with IO error context
    pub fn zstd_compression(source: std::io::Error) -> Self {
        Self::ZstdCompression { source }
    }
}

/// Result type alias for compression operations
pub type CompressionResult<T> = Result<T, CompressionError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::path::Path;
    
    #[test]
    fn test_error_creation_with_context() {
        let error = CompressionError::pattern_analysis("Invalid pattern detected");
        assert!(error.to_string().contains("Pattern analysis failed"));
        assert!(error.to_string().contains("Invalid pattern detected"));
    }
    
    #[test]
    fn test_file_processing_error_with_path() {
        let path = Path::new("/test/file.rs");
        let error = CompressionError::file_processing(path, "Permission denied");
        assert!(error.to_string().contains("/test/file.rs"));
        assert!(error.to_string().contains("Permission denied"));
    }
    
    #[test]
    fn test_error_chain_with_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let compression_error = CompressionError::from(io_error);
        assert!(matches!(compression_error, CompressionError::Io { .. }));
    }
    
    #[test]
    fn test_all_error_variants() {
        // Test PatternAnalysis
        let error = CompressionError::pattern_analysis("test message");
        assert!(error.to_string().contains("Pattern analysis failed"));
        
        // Test DictionaryBuild
        let error = CompressionError::dictionary_build("test message");
        assert!(error.to_string().contains("Dictionary building failed"));
        
        // Test TokenOverflow
        let error = CompressionError::TokenOverflow;
        assert!(error.to_string().contains("Token generation overflow"));
        
        // Test PatternReplacement
        let error = CompressionError::pattern_replacement("test message");
        assert!(error.to_string().contains("Pattern replacement failed"));
        
        // Test OutputCreation
        let error = CompressionError::output_creation("/test/path");
        assert!(error.to_string().contains("Output file creation failed"));
        assert!(error.to_string().contains("/test/path"));
        
        // Test ConfigValidation
        let error = CompressionError::config_validation("test message");
        assert!(error.to_string().contains("Configuration validation failed"));
        
        // Test IntegrityCheck
        let error = CompressionError::integrity_check("test message");
        assert!(error.to_string().contains("Integrity check failed"));
    }
    
    #[test]
    fn test_error_source_chain() {
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
        let compression_error = CompressionError::from(io_error);
        
        // Test that the source chain is preserved
        assert!(compression_error.source().is_some());
        let source = compression_error.source().unwrap();
        assert!(source.to_string().contains("Access denied"));
    }
    
    #[test]
    fn test_error_display_formatting() {
        let error = CompressionError::file_processing("/path/to/file.rs", "Read error");
        let display_str = format!("{}", error);
        assert!(display_str.contains("File processing failed"));
        assert!(display_str.contains("/path/to/file.rs"));
        assert!(display_str.contains("Read error"));
    }
    
    #[test]
    fn test_error_debug_formatting() {
        let error = CompressionError::TokenOverflow;
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("TokenOverflow"));
    }
    
    #[test]
    fn test_compression_result_alias() {
        fn test_function() -> CompressionResult<String> {
            Ok("success".to_string())
        }
        
        let result = test_function();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }
    
    #[test]
    fn test_error_conversion_traits() {
        // Test that Into<String> works for helper methods
        let error = CompressionError::pattern_analysis("test");
        assert!(error.to_string().contains("test"));
        
        let error = CompressionError::file_processing(Path::new("/test"), "message");
        assert!(error.to_string().contains("/test"));
        assert!(error.to_string().contains("message"));
    }
}