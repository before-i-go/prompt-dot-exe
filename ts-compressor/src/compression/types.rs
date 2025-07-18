//! Data types for compression operations
//! 
//! Provides type-safe data models with domain-specific newtypes
//! and comprehensive statistics tracking.

use std::path::PathBuf;
use std::time::Duration;

/// Newtype for compression ratio with validation and display
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct CompressionRatio(f64);

impl CompressionRatio {
    /// Create a new compression ratio (0.0 to 1.0)
    pub fn new(ratio: f64) -> Option<Self> {
        if ratio >= 0.0 && ratio <= 1.0 && ratio.is_finite() {
            Some(Self(ratio))
        } else {
            None
        }
    }
    
    /// Get the ratio as a percentage
    pub fn as_percentage(&self) -> f64 {
        self.0 * 100.0
    }
    
    /// Get the raw ratio value
    pub fn get(&self) -> f64 {
        self.0
    }
}

impl std::fmt::Display for CompressionRatio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}%", self.as_percentage())
    }
}

/// Newtype for file size with human-readable display
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileSize(usize);

impl FileSize {
    /// Create a new file size
    pub fn new(size: usize) -> Self {
        Self(size)
    }
    
    /// Get the size in bytes
    pub fn bytes(&self) -> usize {
        self.0
    }
    
    /// Calculate space saved compared to another size
    pub fn space_saved(&self, original: FileSize) -> usize {
        original.0.saturating_sub(self.0)
    }
}

impl std::fmt::Display for FileSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = self.0 as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            write!(f, "{} {}", self.0, UNITS[unit_index])
        } else {
            write!(f, "{:.2} {}", size, UNITS[unit_index])
        }
    }
}

impl std::ops::Add for FileSize {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl std::ops::Sub for FileSize {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }
}

/// Represents a file in the compressed output
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub relative_path: PathBuf,
    pub original_content: String,
    pub compressed_content: Option<String>,
    pub is_binary: bool,
    pub original_size: FileSize,
    pub compressed_size: Option<FileSize>,
}

impl FileEntry {
    /// Create a new file entry
    pub fn new(path: PathBuf, content: String, is_binary: bool) -> Self {
        let original_size = FileSize::new(content.len());
        Self {
            relative_path: path,
            original_content: content,
            compressed_content: None,
            is_binary,
            original_size,
            compressed_size: None,
        }
    }
    
    /// Apply compression to this file entry
    pub fn apply_compression(&mut self, compressed_content: String) {
        let compressed_size = FileSize::new(compressed_content.len());
        self.compressed_content = Some(compressed_content);
        self.compressed_size = Some(compressed_size);
    }
    
    /// Get compression ratio for this file
    pub fn compression_ratio(&self) -> Option<CompressionRatio> {
        self.compressed_size.and_then(|compressed| {
            if self.original_size.bytes() == 0 {
                Some(CompressionRatio::new(0.0).unwrap())
            } else {
                let ratio = compressed.bytes() as f64 / self.original_size.bytes() as f64;
                CompressionRatio::new(ratio)
            }
        })
    }
    
    /// Check if this file was compressed
    pub fn is_compressed(&self) -> bool {
        self.compressed_content.is_some()
    }
}

/// Detailed statistics about the compression process
#[derive(Debug, Clone)]
pub struct CompressionStatistics {
    pub total_files_processed: usize,
    pub total_patterns_found: usize,
    pub dictionary_entries: usize,
    pub original_total_size: FileSize,
    pub compressed_total_size: FileSize,
    pub processing_time: Duration,
    pub files_compressed: usize,
    pub files_skipped: usize,
}

impl CompressionStatistics {
    /// Create new compression statistics
    pub fn new() -> Self {
        Self {
            total_files_processed: 0,
            total_patterns_found: 0,
            dictionary_entries: 0,
            original_total_size: FileSize::new(0),
            compressed_total_size: FileSize::new(0),
            processing_time: Duration::new(0, 0),
            files_compressed: 0,
            files_skipped: 0,
        }
    }
    
    /// Calculate overall compression ratio
    pub fn compression_ratio(&self) -> CompressionRatio {
        if self.original_total_size.bytes() == 0 {
            CompressionRatio::new(0.0).unwrap()
        } else {
            let ratio = self.compressed_total_size.bytes() as f64 / self.original_total_size.bytes() as f64;
            CompressionRatio::new(ratio).unwrap_or(CompressionRatio::new(1.0).unwrap())
        }
    }
    
    /// Calculate space saved
    pub fn space_saved(&self) -> FileSize {
        self.original_total_size - self.compressed_total_size
    }
    
    /// Calculate compression efficiency (patterns found per file)
    pub fn compression_efficiency(&self) -> f64 {
        if self.total_files_processed == 0 {
            0.0
        } else {
            self.total_patterns_found as f64 / self.total_files_processed as f64
        }
    }
}

impl Default for CompressionStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CompressionStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Compression Statistics:")?;
        writeln!(f, "  Files processed: {}", self.total_files_processed)?;
        writeln!(f, "  Files compressed: {}", self.files_compressed)?;
        writeln!(f, "  Files skipped: {}", self.files_skipped)?;
        writeln!(f, "  Patterns found: {}", self.total_patterns_found)?;
        writeln!(f, "  Dictionary entries: {}", self.dictionary_entries)?;
        writeln!(f, "  Original size: {}", self.original_total_size)?;
        writeln!(f, "  Compressed size: {}", self.compressed_total_size)?;
        writeln!(f, "  Space saved: {}", self.space_saved())?;
        writeln!(f, "  Compression ratio: {}", self.compression_ratio())?;
        writeln!(f, "  Processing time: {:.2}s", self.processing_time.as_secs_f64())?;
        write!(f, "  Efficiency: {:.2} patterns/file", self.compression_efficiency())
    }
}

/// Represents the result of a compression operation
#[derive(Debug, Clone)]
pub struct CompressionResult {
    pub output_file_path: PathBuf,
    pub statistics: CompressionStatistics,
    pub dictionary_size: usize,
    pub patterns_replaced: usize,
}

impl CompressionResult {
    /// Create a new compression result
    pub fn new(
        output_file_path: PathBuf,
        statistics: CompressionStatistics,
        dictionary_size: usize,
        patterns_replaced: usize,
    ) -> Self {
        Self {
            output_file_path,
            statistics,
            dictionary_size,
            patterns_replaced,
        }
    }
    
    /// Get compression percentage
    pub fn compression_percentage(&self) -> f64 {
        self.statistics.compression_ratio().as_percentage()
    }
    
    /// Get space saved in bytes
    pub fn space_saved(&self) -> FileSize {
        self.statistics.space_saved()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    
    #[test]
    fn test_compression_ratio() {
        let ratio = CompressionRatio::new(0.75).unwrap();
        assert_eq!(ratio.as_percentage(), 75.0);
        assert_eq!(format!("{}", ratio), "75.00%");
        
        assert!(CompressionRatio::new(-0.1).is_none());
        assert!(CompressionRatio::new(1.1).is_none());
    }
    
    #[test]
    fn test_file_size_display() {
        assert_eq!(format!("{}", FileSize::new(512)), "512 B");
        assert_eq!(format!("{}", FileSize::new(1536)), "1.50 KB");
        assert_eq!(format!("{}", FileSize::new(1048576)), "1.00 MB");
    }
    
    #[test]
    fn test_file_size_arithmetic() {
        let size1 = FileSize::new(1000);
        let size2 = FileSize::new(500);
        
        assert_eq!((size1 + size2).bytes(), 1500);
        assert_eq!((size1 - size2).bytes(), 500);
        assert_eq!(size2.space_saved(size1), 500);
    }
    
    #[test]
    fn test_file_entry() {
        let mut entry = FileEntry::new(
            Path::new("test.rs").to_path_buf(),
            "fn main() {}".to_string(),
            false,
        );
        
        assert!(!entry.is_compressed());
        assert!(entry.compression_ratio().is_none());
        
        entry.apply_compression("fn main(){}".to_string());
        assert!(entry.is_compressed());
        assert!(entry.compression_ratio().is_some());
    }
    
    #[test]
    fn test_compression_statistics() {
        let mut stats = CompressionStatistics::new();
        stats.total_files_processed = 10;
        stats.total_patterns_found = 50;
        stats.original_total_size = FileSize::new(1000);
        stats.compressed_total_size = FileSize::new(750);
        
        assert_eq!(stats.compression_efficiency(), 5.0);
        assert_eq!(stats.space_saved().bytes(), 250);
        assert_eq!(stats.compression_ratio().as_percentage(), 75.0);
    }
}