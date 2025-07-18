//! Universal Code Compression Module
//! 
//! This module provides frequency-based dictionary compression capabilities
//! for maximum codebase size reduction through intelligent pattern recognition
//! and hexadecimal token replacement.

pub mod analyzer;
pub mod builder;
pub mod compressor;
pub mod concurrent_analyzer;
pub mod config;
pub mod error;
pub mod generator;
pub mod replacer;
pub mod types;
pub mod zstd_compressor;

// Re-export main types for convenience
pub use analyzer::FrequencyAnalyzer;
pub use builder::DictionaryBuilder;
pub use compressor::UniversalCompressor;
pub use concurrent_analyzer::ConcurrentFrequencyAnalyzer;
pub use config::CompressionConfig;
pub use error::CompressionError;
pub use generator::{HexTokenGenerator, HexToken};
pub use replacer::PatternReplacer;
pub use types::{CompressionResult, CompressionStatistics, FileEntry};
pub use zstd_compressor::{ZstdCompressor, ZstdStreamCompressor, ZstdStreamDecompressor, ZstdCompressionStats};

/// Core trait for frequency analysis operations
pub trait FrequencyAnalysis {
    /// Analyze content for pattern frequencies
    fn analyze_content(&mut self, content: &str);
    
    /// Get patterns that meet frequency threshold
    fn get_frequent_patterns(&self) -> Vec<(String, usize)>;
    
    /// Check if pattern should be compressed
    fn should_compress_pattern(&self, pattern: &str) -> bool;
}

/// Core trait for dictionary building operations
pub trait DictionaryBuilding {
    /// Build dictionary from frequent patterns
    fn build_dictionary(&mut self, patterns: Vec<(String, usize)>) -> Result<(), CompressionError>;
    
    /// Get replacement token for pattern
    fn get_replacement_token(&self, pattern: &str) -> Option<&String>;
    
    /// Get all dictionary entries
    fn get_dictionary_entries(&self) -> Vec<(String, String)>;
    
    /// Validate dictionary integrity
    fn validate_dictionary(&self) -> Result<(), CompressionError>;
}

/// Core trait for pattern replacement operations
pub trait PatternReplacement {
    /// Replace patterns in content using dictionary
    fn replace_patterns(&self, content: &str) -> String;
    
    /// Calculate compression ratio
    fn calculate_compression_ratio(&self, original: &str, compressed: &str) -> f64;
}

/// Core trait for token generation
pub trait TokenGeneration {
    /// Generate next unique token
    fn next_token(&mut self) -> Result<String, CompressionError>;
    
    /// Reset token generator
    fn reset(&mut self);
}