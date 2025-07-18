//! Configuration types for compression operations
//!
//! Provides type-safe configuration with validation and builder patterns
//! for flexible compression parameter management.

use crate::compression::error::{CompressionError, CompressionResult};
use std::fmt;

/// Newtype for minimum pattern length with validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MinPatternLength(usize);

impl MinPatternLength {
    /// Create a new MinPatternLength with validation
    pub fn new(length: usize) -> CompressionResult<Self> {
        if length < 2 {
            return Err(CompressionError::config_validation(
                "Minimum pattern length must be at least 2",
            ));
        }
        if length > 100 {
            return Err(CompressionError::config_validation(
                "Minimum pattern length cannot exceed 100",
            ));
        }
        Ok(Self(length))
    }

    /// Get the inner value
    pub fn get(&self) -> usize {
        self.0
    }
}

impl Default for MinPatternLength {
    fn default() -> Self {
        Self(4) // Safe default, no validation needed
    }
}

impl fmt::Display for MinPatternLength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Newtype for frequency threshold with validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FrequencyThreshold(usize);

impl FrequencyThreshold {
    /// Create a new FrequencyThreshold with validation
    pub fn new(threshold: usize) -> CompressionResult<Self> {
        if threshold < 2 {
            return Err(CompressionError::config_validation(
                "Frequency threshold must be at least 2",
            ));
        }
        if threshold > 1000 {
            return Err(CompressionError::config_validation(
                "Frequency threshold cannot exceed 1000",
            ));
        }
        Ok(Self(threshold))
    }

    /// Get the inner value
    pub fn get(&self) -> usize {
        self.0
    }
}

impl Default for FrequencyThreshold {
    fn default() -> Self {
        Self(3) // Safe default, no validation needed
    }
}

impl fmt::Display for FrequencyThreshold {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Newtype for zstd compression level with validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZstdLevel(i32);

impl ZstdLevel {
    /// Create a new ZstdLevel with validation
    pub fn new(level: i32) -> CompressionResult<Self> {
        if !(1..=22).contains(&level) {
            return Err(CompressionError::config_validation(
                "Zstd compression level must be between 1 and 22",
            ));
        }
        Ok(Self(level))
    }

    /// Get the inner value
    pub fn get(&self) -> i32 {
        self.0
    }
}

impl Default for ZstdLevel {
    fn default() -> Self {
        Self(3) // Safe default, no validation needed
    }
}

impl fmt::Display for ZstdLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Newtype for thread count with validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThreadCount(usize);

impl ThreadCount {
    /// Create a new ThreadCount with validation
    pub fn new(count: usize) -> CompressionResult<Self> {
        if count == 0 {
            return Err(CompressionError::config_validation(
                "Thread count must be at least 1",
            ));
        }
        if count > 256 {
            return Err(CompressionError::config_validation(
                "Thread count cannot exceed 256",
            ));
        }
        Ok(Self(count))
    }

    /// Get the inner value
    pub fn get(&self) -> usize {
        self.0
    }
}

impl Default for ThreadCount {
    fn default() -> Self {
        Self(num_cpus::get()) // Safe default based on system capabilities
    }
}

impl fmt::Display for ThreadCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Newtype for chunk size with validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkSize(usize);

impl ChunkSize {
    /// Create a new ChunkSize with validation
    pub fn new(size: usize) -> CompressionResult<Self> {
        if size < 1024 {
            return Err(CompressionError::config_validation(
                "Chunk size must be at least 1KB",
            ));
        }
        if size > 10 * 1024 * 1024 {
            return Err(CompressionError::config_validation(
                "Chunk size cannot exceed 10MB",
            ));
        }
        Ok(Self(size))
    }

    /// Get the inner value
    pub fn get(&self) -> usize {
        self.0
    }
}

impl Default for ChunkSize {
    fn default() -> Self {
        Self(64 * 1024) // 64KB default chunk size
    }
}

impl fmt::Display for ChunkSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}KB", self.0 / 1024)
    }
}

/// Newtype for channel buffer size with validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChannelBufferSize(usize);

impl ChannelBufferSize {
    /// Create a new ChannelBufferSize with validation
    pub fn new(size: usize) -> CompressionResult<Self> {
        if size == 0 {
            return Err(CompressionError::config_validation(
                "Channel buffer size must be at least 1",
            ));
        }
        if size > 10000 {
            return Err(CompressionError::config_validation(
                "Channel buffer size cannot exceed 10000",
            ));
        }
        Ok(Self(size))
    }

    /// Get the inner value
    pub fn get(&self) -> usize {
        self.0
    }
}

impl Default for ChannelBufferSize {
    fn default() -> Self {
        Self(100) // Reasonable default for backpressure control
    }
}

impl fmt::Display for ChannelBufferSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Newtype for memory map threshold with validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryMapThreshold(usize);

impl MemoryMapThreshold {
    /// Create a new MemoryMapThreshold with validation
    pub fn new(threshold: usize) -> CompressionResult<Self> {
        if threshold < 1024 {
            return Err(CompressionError::config_validation(
                "Memory map threshold must be at least 1KB",
            ));
        }
        if threshold > 1024 * 1024 * 1024 {
            return Err(CompressionError::config_validation(
                "Memory map threshold cannot exceed 1GB",
            ));
        }
        Ok(Self(threshold))
    }

    /// Get the inner value
    pub fn get(&self) -> usize {
        self.0
    }
}

impl Default for MemoryMapThreshold {
    fn default() -> Self {
        Self(1024 * 1024) // 1MB default threshold
    }
}

impl fmt::Display for MemoryMapThreshold {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}MB", self.0 / (1024 * 1024))
    }
}

/// Configuration for parallel processing parameters
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    pub max_threads: ThreadCount,
    pub chunk_size: ChunkSize,
    pub channel_buffer_size: ChannelBufferSize,
    pub memory_map_threshold: MemoryMapThreshold,
}

impl ParallelConfig {
    /// Create a new parallel configuration with builder pattern
    pub fn builder() -> ParallelConfigBuilder {
        ParallelConfigBuilder::new()
    }

    /// Validate the parallel configuration
    pub fn validate(&self) -> CompressionResult<()> {
        // Cross-field validation
        if self.max_threads.get() > 64 && self.channel_buffer_size.get() < 50 {
            return Err(CompressionError::config_validation(
                "High thread counts require larger channel buffers to prevent bottlenecks",
            ));
        }

        if self.chunk_size.get() > self.memory_map_threshold.get() {
            return Err(CompressionError::config_validation(
                "Chunk size cannot be larger than memory map threshold",
            ));
        }

        Ok(())
    }
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_threads: ThreadCount::default(),
            chunk_size: ChunkSize::default(),
            channel_buffer_size: ChannelBufferSize::default(),
            memory_map_threshold: MemoryMapThreshold::default(),
        }
    }
}

/// Builder for ParallelConfig with method chaining
#[derive(Debug, Default)]
pub struct ParallelConfigBuilder {
    max_threads: Option<usize>,
    chunk_size: Option<usize>,
    channel_buffer_size: Option<usize>,
    memory_map_threshold: Option<usize>,
}

impl ParallelConfigBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum thread count
    pub fn max_threads(mut self, count: usize) -> Self {
        self.max_threads = Some(count);
        self
    }

    /// Set chunk size in bytes
    pub fn chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = Some(size);
        self
    }

    /// Set channel buffer size
    pub fn channel_buffer_size(mut self, size: usize) -> Self {
        self.channel_buffer_size = Some(size);
        self
    }

    /// Set memory map threshold in bytes
    pub fn memory_map_threshold(mut self, threshold: usize) -> Self {
        self.memory_map_threshold = Some(threshold);
        self
    }

    /// Build the parallel configuration with validation
    pub fn build(self) -> CompressionResult<ParallelConfig> {
        let config = ParallelConfig {
            max_threads: match self.max_threads {
                Some(count) => ThreadCount::new(count)?,
                None => ThreadCount::default(),
            },
            chunk_size: match self.chunk_size {
                Some(size) => ChunkSize::new(size)?,
                None => ChunkSize::default(),
            },
            channel_buffer_size: match self.channel_buffer_size {
                Some(size) => ChannelBufferSize::new(size)?,
                None => ChannelBufferSize::default(),
            },
            memory_map_threshold: match self.memory_map_threshold {
                Some(threshold) => MemoryMapThreshold::new(threshold)?,
                None => MemoryMapThreshold::default(),
            },
        };

        config.validate()?;
        Ok(config)
    }
}

/// Main configuration structure for compression operations
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    pub min_pattern_length: MinPatternLength,
    pub min_frequency_threshold: FrequencyThreshold,
    pub enable_zstd_compression: bool,
    pub zstd_compression_level: ZstdLevel,
    #[allow(dead_code)]
    pub parallel_config: ParallelConfig,
}

impl CompressionConfig {
    /// Create a new configuration with builder pattern
    pub fn builder() -> CompressionConfigBuilder {
        CompressionConfigBuilder::new()
    }

    /// Validate the configuration
    pub fn validate(&self) -> CompressionResult<()> {
        // Additional cross-field validation can be added here
        if self.min_pattern_length.get() > 50 && self.min_frequency_threshold.get() < 5 {
            return Err(CompressionError::config_validation(
                "Large pattern lengths require higher frequency thresholds for efficiency",
            ));
        }
        Ok(())
    }
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            min_pattern_length: MinPatternLength::default(),
            min_frequency_threshold: FrequencyThreshold::default(),
            enable_zstd_compression: true,
            zstd_compression_level: ZstdLevel::default(),
            parallel_config: ParallelConfig::default(),
        }
    }
}

/// Builder for CompressionConfig with method chaining
#[derive(Debug, Default)]
pub struct CompressionConfigBuilder {
    min_pattern_length: Option<usize>,
    min_frequency_threshold: Option<usize>,
    enable_zstd_compression: Option<bool>,
    zstd_compression_level: Option<i32>,
    parallel_config: Option<ParallelConfig>,
}

impl CompressionConfigBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set minimum pattern length
    pub fn min_pattern_length(mut self, length: usize) -> Self {
        self.min_pattern_length = Some(length);
        self
    }

    /// Set frequency threshold
    pub fn min_frequency_threshold(mut self, threshold: usize) -> Self {
        self.min_frequency_threshold = Some(threshold);
        self
    }

    /// Enable or disable zstd compression
    pub fn enable_zstd_compression(mut self, enable: bool) -> Self {
        self.enable_zstd_compression = Some(enable);
        self
    }

    /// Set zstd compression level
    #[allow(dead_code)]
    pub fn zstd_compression_level(mut self, level: i32) -> Self {
        self.zstd_compression_level = Some(level);
        self
    }

    /// Set parallel configuration
    pub fn parallel_config(mut self, config: ParallelConfig) -> Self {
        self.parallel_config = Some(config);
        self
    }

    /// Build the configuration with validation
    pub fn build(self) -> CompressionResult<CompressionConfig> {
        let config = CompressionConfig {
            min_pattern_length: match self.min_pattern_length {
                Some(length) => MinPatternLength::new(length)?,
                None => MinPatternLength::default(),
            },
            min_frequency_threshold: match self.min_frequency_threshold {
                Some(threshold) => FrequencyThreshold::new(threshold)?,
                None => FrequencyThreshold::default(),
            },
            enable_zstd_compression: self.enable_zstd_compression.unwrap_or(true),
            zstd_compression_level: match self.zstd_compression_level {
                Some(level) => ZstdLevel::new(level)?,
                None => ZstdLevel::default(),
            },
            parallel_config: self.parallel_config.unwrap_or_default(),
        };

        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_pattern_length_validation() {
        assert!(MinPatternLength::new(1).is_err());
        assert!(MinPatternLength::new(4).is_ok());
        assert!(MinPatternLength::new(101).is_err());
    }

    #[test]
    fn test_frequency_threshold_validation() {
        assert!(FrequencyThreshold::new(1).is_err());
        assert!(FrequencyThreshold::new(3).is_ok());
        assert!(FrequencyThreshold::new(1001).is_err());
    }

    #[test]
    fn test_zstd_level_validation() {
        assert!(ZstdLevel::new(0).is_err());
        assert!(ZstdLevel::new(3).is_ok());
        assert!(ZstdLevel::new(23).is_err());
    }

    #[test]
    fn test_config_builder_pattern() {
        let config = CompressionConfig::builder()
            .min_pattern_length(5)
            .min_frequency_threshold(4)
            .enable_zstd_compression(false)
            .zstd_compression_level(6)
            .build()
            .unwrap();

        assert_eq!(config.min_pattern_length.get(), 5);
        assert_eq!(config.min_frequency_threshold.get(), 4);
        assert!(!config.enable_zstd_compression);
        assert_eq!(config.zstd_compression_level.get(), 6);
    }

    #[test]
    fn test_config_validation() {
        let config = CompressionConfig::builder()
            .min_pattern_length(60)
            .min_frequency_threshold(2)
            .build();

        assert!(config.is_err());
    }

    #[test]
    fn test_default_config() {
        let config = CompressionConfig::default();
        assert_eq!(config.min_pattern_length.get(), 4);
        assert_eq!(config.min_frequency_threshold.get(), 3);
        assert!(config.enable_zstd_compression);
        assert_eq!(config.zstd_compression_level.get(), 3);
        // Test that parallel config is included
        assert!(config.parallel_config.max_threads.get() > 0);
    }

    // TDD Tests for Parallel Processing Types

    #[test]
    fn test_thread_count_validation() {
        // Test invalid values
        assert!(ThreadCount::new(0).is_err());
        assert!(ThreadCount::new(257).is_err());

        // Test valid values
        assert!(ThreadCount::new(1).is_ok());
        assert!(ThreadCount::new(8).is_ok());
        assert!(ThreadCount::new(256).is_ok());

        // Test default is reasonable
        let default = ThreadCount::default();
        assert!(default.get() > 0);
        assert!(default.get() <= 256);
    }

    #[test]
    fn test_chunk_size_validation() {
        // Test invalid values
        assert!(ChunkSize::new(1023).is_err()); // Less than 1KB
        assert!(ChunkSize::new(11 * 1024 * 1024).is_err()); // More than 10MB

        // Test valid values
        assert!(ChunkSize::new(1024).is_ok()); // 1KB
        assert!(ChunkSize::new(64 * 1024).is_ok()); // 64KB
        assert!(ChunkSize::new(10 * 1024 * 1024).is_ok()); // 10MB

        // Test default
        let default = ChunkSize::default();
        assert_eq!(default.get(), 64 * 1024);
    }

    #[test]
    fn test_channel_buffer_size_validation() {
        // Test invalid values
        assert!(ChannelBufferSize::new(0).is_err());
        assert!(ChannelBufferSize::new(10001).is_err());

        // Test valid values
        assert!(ChannelBufferSize::new(1).is_ok());
        assert!(ChannelBufferSize::new(100).is_ok());
        assert!(ChannelBufferSize::new(10000).is_ok());

        // Test default
        let default = ChannelBufferSize::default();
        assert_eq!(default.get(), 100);
    }

    #[test]
    fn test_memory_map_threshold_validation() {
        // Test invalid values
        assert!(MemoryMapThreshold::new(1023).is_err()); // Less than 1KB
        assert!(MemoryMapThreshold::new(1024 * 1024 * 1024 + 1).is_err()); // More than 1GB

        // Test valid values
        assert!(MemoryMapThreshold::new(1024).is_ok()); // 1KB
        assert!(MemoryMapThreshold::new(1024 * 1024).is_ok()); // 1MB
        assert!(MemoryMapThreshold::new(1024 * 1024 * 1024).is_ok()); // 1GB

        // Test default
        let default = MemoryMapThreshold::default();
        assert_eq!(default.get(), 1024 * 1024);
    }

    #[test]
    fn test_parallel_config_builder_pattern() {
        let config = ParallelConfig::builder()
            .max_threads(8)
            .chunk_size(128 * 1024)
            .channel_buffer_size(200)
            .memory_map_threshold(2 * 1024 * 1024)
            .build()
            .unwrap();

        assert_eq!(config.max_threads.get(), 8);
        assert_eq!(config.chunk_size.get(), 128 * 1024);
        assert_eq!(config.channel_buffer_size.get(), 200);
        assert_eq!(config.memory_map_threshold.get(), 2 * 1024 * 1024);
    }

    #[test]
    fn test_parallel_config_validation() {
        // Test cross-field validation: high thread count with small buffer
        let config = ParallelConfig::builder()
            .max_threads(128)
            .channel_buffer_size(10)
            .build();

        assert!(config.is_err());

        // Test chunk size larger than memory map threshold
        let config = ParallelConfig::builder()
            .chunk_size(2 * 1024 * 1024)
            .memory_map_threshold(1024 * 1024)
            .build();

        assert!(config.is_err());
    }

    #[test]
    fn test_parallel_config_default() {
        let config = ParallelConfig::default();
        assert!(config.max_threads.get() > 0);
        assert_eq!(config.chunk_size.get(), 64 * 1024);
        assert_eq!(config.channel_buffer_size.get(), 100);
        assert_eq!(config.memory_map_threshold.get(), 1024 * 1024);

        // Validate default config is valid
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_compression_config_with_parallel_config() {
        let parallel_config = ParallelConfig::builder()
            .max_threads(4)
            .chunk_size(32 * 1024)
            .build()
            .unwrap();

        let config = CompressionConfig::builder()
            .min_pattern_length(5)
            .parallel_config(parallel_config)
            .build()
            .unwrap();

        assert_eq!(config.min_pattern_length.get(), 5);
        assert_eq!(config.parallel_config.max_threads.get(), 4);
        assert_eq!(config.parallel_config.chunk_size.get(), 32 * 1024);
    }

    #[test]
    fn test_display_formatting() {
        let thread_count = ThreadCount::new(8).unwrap();
        assert_eq!(format!("{}", thread_count), "8");

        let chunk_size = ChunkSize::new(128 * 1024).unwrap();
        assert_eq!(format!("{}", chunk_size), "128KB");

        let buffer_size = ChannelBufferSize::new(150).unwrap();
        assert_eq!(format!("{}", buffer_size), "150");

        let threshold = MemoryMapThreshold::new(2 * 1024 * 1024).unwrap();
        assert_eq!(format!("{}", threshold), "2MB");
    }
}
