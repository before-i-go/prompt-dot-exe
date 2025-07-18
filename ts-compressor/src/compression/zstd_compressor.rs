//! Zstd compression integration
//!
//! Provides idiomatic Rust wrapper around zstd compression with
//! RAII resource management and type-safe compression levels.

use crate::compression::config::ZstdLevel;
use crate::compression::error::{CompressionError, CompressionResult};
use std::io::{Read, Write};

/// Newtype wrapper for zstd compression context with RAII
pub struct ZstdCompressor {
    compression_level: ZstdLevel,
}

impl ZstdCompressor {
    /// Create a new zstd compressor with specified level
    pub fn new(level: ZstdLevel) -> CompressionResult<Self> {
        Ok(Self {
            compression_level: level,
        })
    }

    /// Compress data using zstd
    pub fn compress(&self, data: &[u8]) -> CompressionResult<Vec<u8>> {
        zstd::bulk::compress(data, self.compression_level.get())
            .map_err(|e| CompressionError::zstd_compression(e))
    }

    /// Decompress data using zstd
    #[allow(dead_code)]
    pub fn decompress(&self, data: &[u8]) -> CompressionResult<Vec<u8>> {
        zstd::bulk::decompress(data, 1024 * 1024) // 1MB limit for safety
            .map_err(|e| CompressionError::zstd_compression(e))
    }

    /// Compress string data
    pub fn compress_string(&self, data: &str) -> CompressionResult<Vec<u8>> {
        self.compress(data.as_bytes())
    }

    /// Decompress to string data
    #[allow(dead_code)]
    pub fn decompress_to_string(&self, data: &[u8]) -> CompressionResult<String> {
        let decompressed = self.decompress(data)?;
        String::from_utf8(decompressed).map_err(|e| {
            CompressionError::zstd_compression(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid UTF-8 in decompressed data: {}", e),
            ))
        })
    }

    /// Get compression level
    #[allow(dead_code)]
    pub fn compression_level(&self) -> ZstdLevel {
        self.compression_level
    }
}

/// Streaming zstd compressor for large data
#[allow(dead_code)]
pub struct ZstdStreamCompressor<W: Write> {
    encoder: zstd::stream::write::Encoder<'static, W>,
}

impl<W: Write> ZstdStreamCompressor<W> {
    /// Create a new streaming compressor
    #[allow(dead_code)]
    pub fn new(writer: W, level: ZstdLevel) -> CompressionResult<Self> {
        let encoder = zstd::stream::write::Encoder::new(writer, level.get())
            .map_err(|e| CompressionError::zstd_compression(e))?;

        Ok(Self { encoder })
    }

    /// Write data to the compressor
    #[allow(dead_code)]
    pub fn write(&mut self, data: &[u8]) -> CompressionResult<()> {
        self.encoder
            .write_all(data)
            .map_err(|e| CompressionError::zstd_compression(e))
    }

    /// Finish compression and return the underlying writer
    #[allow(dead_code)]
    pub fn finish(self) -> CompressionResult<W> {
        self.encoder
            .finish()
            .map_err(|e| CompressionError::zstd_compression(e))
    }
}

/// Streaming zstd decompressor for large data
pub struct ZstdStreamDecompressor<R: Read> {
    decoder: zstd::stream::read::Decoder<'static, std::io::BufReader<R>>,
}

impl<R: Read> ZstdStreamDecompressor<R> {
    /// Create a new streaming decompressor
    #[allow(dead_code)]
    pub fn new(reader: R) -> CompressionResult<Self> {
        // zstd::stream::read::Decoder::new() wraps the reader in BufReader automatically
        let decoder = zstd::stream::read::Decoder::new(reader)
            .map_err(|e| CompressionError::zstd_compression(e))?;

        Ok(Self { decoder })
    }
}

impl<R: Read> Read for ZstdStreamDecompressor<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.decoder.read(buf)
    }
}

/// Zstd compression statistics
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ZstdCompressionStats {
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_level: ZstdLevel,
    pub compression_time_ms: u128,
}

impl ZstdCompressionStats {
    /// Calculate compression ratio
    #[allow(dead_code)]
    pub fn compression_ratio(&self) -> f64 {
        if self.original_size == 0 {
            0.0
        } else {
            self.compressed_size as f64 / self.original_size as f64
        }
    }

    /// Calculate space saved
    #[allow(dead_code)]
    pub fn space_saved(&self) -> usize {
        self.original_size.saturating_sub(self.compressed_size)
    }

    /// Calculate compression percentage
    #[allow(dead_code)]
    pub fn compression_percentage(&self) -> f64 {
        (1.0 - self.compression_ratio()) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_zstd_compressor_creation() {
        let level = ZstdLevel::new(3).unwrap();
        let compressor = ZstdCompressor::new(level);
        assert!(compressor.is_ok());

        let compressor = compressor.unwrap();
        assert_eq!(compressor.compression_level().get(), 3);
    }

    #[test]
    fn test_zstd_compression_basic() {
        let level = ZstdLevel::new(1).unwrap();
        let compressor = ZstdCompressor::new(level).unwrap();

        let original_data = b"Hello, world! This is a test string for compression.";
        let compressed = compressor.compress(original_data);
        assert!(compressed.is_ok());

        let compressed_data = compressed.unwrap();
        assert!(!compressed_data.is_empty());

        // Test decompression
        let decompressed = compressor.decompress(&compressed_data);
        assert!(decompressed.is_ok());

        let decompressed_data = decompressed.unwrap();
        assert_eq!(decompressed_data, original_data);
    }

    #[test]
    fn test_zstd_string_compression() {
        let level = ZstdLevel::new(3).unwrap();
        let compressor = ZstdCompressor::new(level).unwrap();

        let original_string = "fn main() { println!(\"Hello, world!\"); }";
        let compressed = compressor.compress_string(original_string);
        assert!(compressed.is_ok());

        let compressed_data = compressed.unwrap();
        let decompressed = compressor.decompress_to_string(&compressed_data);
        assert!(decompressed.is_ok());

        assert_eq!(decompressed.unwrap(), original_string);
    }

    #[test]
    fn test_zstd_compression_levels() {
        let test_data =
            "This is a test string that should compress well with repetitive patterns. "
                .repeat(100);

        let level1 = ZstdLevel::new(1).unwrap();
        let level9 = ZstdLevel::new(9).unwrap();

        let compressor1 = ZstdCompressor::new(level1).unwrap();
        let compressor9 = ZstdCompressor::new(level9).unwrap();

        let compressed1 = compressor1.compress_string(&test_data).unwrap();
        let compressed9 = compressor9.compress_string(&test_data).unwrap();

        // Higher compression level should generally produce smaller output
        // (though this isn't guaranteed for all data)
        assert!(!compressed1.is_empty());
        assert!(!compressed9.is_empty());

        // Both should decompress correctly
        let decompressed1 = compressor1.decompress_to_string(&compressed1).unwrap();
        let decompressed9 = compressor9.decompress_to_string(&compressed9).unwrap();

        assert_eq!(decompressed1, test_data);
        assert_eq!(decompressed9, test_data);
    }

    #[test]
    fn test_zstd_compression_empty_data() {
        let level = ZstdLevel::new(3).unwrap();
        let compressor = ZstdCompressor::new(level).unwrap();

        let compressed = compressor.compress(b"");
        assert!(compressed.is_ok());

        let compressed_data = compressed.unwrap();
        let decompressed = compressor.decompress(&compressed_data).unwrap();
        assert!(decompressed.is_empty());
    }

    #[test]
    fn test_zstd_compression_large_data() {
        let level = ZstdLevel::new(3).unwrap();
        let compressor = ZstdCompressor::new(level).unwrap();

        // Create a large string with repetitive patterns
        let large_data = "function test() { return 'hello world'; }\n".repeat(1000);

        let start = Instant::now();
        let compressed = compressor.compress_string(&large_data).unwrap();
        let compression_time = start.elapsed();

        // Verify compression actually reduces size for repetitive data
        assert!(compressed.len() < large_data.len());

        let decompressed = compressor.decompress_to_string(&compressed).unwrap();
        assert_eq!(decompressed, large_data);

        // Basic performance check - should complete in reasonable time
        assert!(compression_time.as_secs() < 5);
    }

    #[test]
    fn test_zstd_compression_stats() {
        let level = ZstdLevel::new(5).unwrap();
        let compressor = ZstdCompressor::new(level).unwrap();

        let test_data = "Hello, world! ".repeat(50);
        let original_size = test_data.len();

        let start = Instant::now();
        let compressed = compressor.compress_string(&test_data).unwrap();
        let compression_time = start.elapsed();

        let stats = ZstdCompressionStats {
            original_size,
            compressed_size: compressed.len(),
            compression_level: level,
            compression_time_ms: compression_time.as_millis(),
        };

        assert!(stats.compression_ratio() > 0.0);
        assert!(stats.compression_ratio() < 1.0); // Should compress
        assert!(stats.space_saved() > 0);
        assert!(stats.compression_percentage() > 0.0);
        assert!(stats.compression_percentage() < 100.0);
    }

    #[test]
    fn test_zstd_streaming_compression() {
        let level = ZstdLevel::new(3).unwrap();
        let mut buffer = Vec::new();

        {
            let mut compressor = ZstdStreamCompressor::new(&mut buffer, level).unwrap();
            compressor.write(b"Hello, ").unwrap();
            compressor.write(b"world!").unwrap();
            compressor.finish().unwrap();
        }

        assert!(!buffer.is_empty());

        // Test decompression - the decoder will wrap the cursor in BufReader internally
        let cursor = std::io::Cursor::new(&buffer);
        let mut decompressor = ZstdStreamDecompressor::new(cursor).unwrap();
        let mut decompressed = Vec::new();
        decompressor.read_to_end(&mut decompressed).unwrap();

        assert_eq!(decompressed, b"Hello, world!");
    }

    #[test]
    fn test_zstd_error_handling() {
        let level = ZstdLevel::new(3).unwrap();
        let compressor = ZstdCompressor::new(level).unwrap();

        // Test decompression of invalid data
        let invalid_data = b"This is not compressed data";
        let result = compressor.decompress(invalid_data);
        assert!(result.is_err());

        // Verify error type
        match result.unwrap_err() {
            CompressionError::ZstdCompression { .. } => (),
            _ => panic!("Expected ZstdCompression error"),
        }
    }
}
