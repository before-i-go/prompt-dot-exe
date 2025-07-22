//! A library for splitting files into smaller chunks with various strategies.

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Read, Write};
use thiserror::Error;
use std::fmt;

/// Custom error type for file splitting operations
#[derive(Error, Debug)]
pub enum SplitError {
    /// I/O error occurred
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Invalid chunk size specified
    #[error("Invalid chunk size: {0}")]
    InvalidChunkSize(String),
    
    /// Invalid input path
    #[error("Invalid input path: {0}")]
    InvalidInputPath(String),
    
    /// Invalid output directory
    #[error("Invalid output directory: {0}")]
    InvalidOutputDir(String),
}

/// Result type for file splitting operations
pub type Result<T> = std::result::Result<T, SplitError>;

/// Configuration for file splitting
#[derive(Debug, Clone)]
pub struct SplitConfig {
    /// Path to the input file
    pub input_path: String,
    
    /// Directory to output chunks (defaults to same as input file)
    pub output_dir: Option<String>,
    
    /// Size of each chunk in bytes
    pub chunk_size: u64,
    
    /// Prefix for output chunk filenames (defaults to input filename)
    pub prefix: Option<String>,
    
    /// Number of digits to use in chunk numbering (default: 3)
    pub digits: u8,
}

impl Default for SplitConfig {
    fn default() -> Self {
        Self {
            input_path: String::new(),
            output_dir: None,
            chunk_size: 1024 * 1024, // 1MB default chunk size
            prefix: None,
            digits: 3,
        }
    }
}

/// Represents a chunk of a file
#[derive(Debug)]
pub struct FileChunk {
    /// The path to the chunk file
    pub path: PathBuf,
    /// The size of the chunk in bytes
    pub size: u64,
}

impl fmt::Display for FileChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, 
            "{} ({} bytes)", 
            self.path.display(),
            self.size
        )
    }
}

/// Result of a file split operation
#[derive(Debug)]
pub struct SplitResult {
    /// The original file path
    pub input_path: PathBuf,
    /// The output directory
    pub output_dir: PathBuf,
    /// The size of each chunk
    pub chunk_size: u64,
    /// Information about each created chunk
    pub chunks: Vec<FileChunk>,
    /// Total number of chunks created
    pub total_chunks: usize,
    /// Total size of the original file in bytes
    pub total_size: u64,
}

impl fmt::Display for SplitResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Split '{}' into {} chunks:", 
            self.input_path.display(), 
            self.total_chunks
        )?;
        
        for (i, chunk) in self.chunks.iter().enumerate() {
            writeln!(f, "  {:03}: {}", i + 1, chunk)?;
        }
        
        writeln!(f, "Total size: {} bytes ({} chunks)", 
            self.total_size, 
            self.total_chunks
        )
    }
}

/// Split a file into smaller chunks based on the provided configuration
pub fn split_file(config: &SplitConfig) -> Result<SplitResult> {
    // Input validation
    if config.chunk_size == 0 {
        return Err(SplitError::InvalidChunkSize("Chunk size must be greater than 0".into()));
    }
    
    // Verify input file exists and is a file
    let input_path = Path::new(&config.input_path).canonicalize()
        .map_err(|e| SplitError::InvalidInputPath(e.to_string()))?;
        
    if !input_path.is_file() {
        return Err(SplitError::InvalidInputPath("Input path is not a file".into()));
    }
    
    // Get or create output directory
    let output_dir = match &config.output_dir {
        Some(dir) => {
            let path = Path::new(dir);
            if !path.exists() {
                std::fs::create_dir_all(path).map_err(SplitError::Io)?;
            }
            path.canonicalize().map_err(|e| 
                SplitError::InvalidOutputDir(e.to_string())
            )?
        }
        None => {
            input_path.parent()
                .unwrap_or_else(|| Path::new("."))
                .canonicalize()
                .map_err(|e| SplitError::InvalidOutputDir(e.to_string()))?
        }
    };
    
    // Get file metadata
    let metadata = std::fs::metadata(&input_path)
        .map_err(|e| SplitError::Io(e))?;
    
    let file_size = metadata.len();
    if file_size == 0 {
        return Err(SplitError::InvalidInputPath("Input file is empty".into()));
    }
    
    // Calculate number of chunks needed
    let total_chunks = ((file_size as f64) / (config.chunk_size as f64)).ceil() as usize;
    
    // Determine the filename prefix
    let prefix = match &config.prefix {
        Some(p) => p.clone(),
        None => input_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("chunk")
            .to_string(),
    };
    
    // Open the input file
    let mut input_file = File::open(&input_path).map_err(SplitError::Io)?;
    
    // Buffer for reading chunks
    let mut buffer = vec![0u8; config.chunk_size as usize];
    let mut chunks = Vec::with_capacity(total_chunks);
    
    // Process each chunk
    for chunk_num in 0..total_chunks {
        let chunk_path = output_dir.join(format!(
            "{}.{:0width$}",
            prefix,
            chunk_num + 1,
            width = config.digits as usize
        ));
        
        // Read a chunk from the input file
        let bytes_read = input_file.read(&mut buffer).map_err(SplitError::Io)?;
        
        if bytes_read == 0 {
            break; // End of file
        }
        
        // Write the chunk to the output file
        let mut output_file = File::create(&chunk_path).map_err(SplitError::Io)?;
        output_file.write_all(&buffer[..bytes_read]).map_err(SplitError::Io)?;
        
        // Add chunk info to the result
        chunks.push(FileChunk {
            path: chunk_path,
            size: bytes_read as u64,
        });
    }
    
    // Build and return the result
    Ok(SplitResult {
        input_path,
        output_dir,
        chunk_size: config.chunk_size,
        chunks,
        total_chunks,
        total_size: file_size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;
    
    #[test]
    fn test_split_config_default() {
        let config = SplitConfig::default();
        assert_eq!(config.input_path, "");
        assert_eq!(config.chunk_size, 1024 * 1024);
        assert_eq!(config.digits, 3);
    }
    
    #[test]
    fn test_split_file_invalid_chunk_size() {
        let config = SplitConfig {
            input_path: "test.txt".to_string(),
            chunk_size: 0,
            ..Default::default()
        };
        
        let result = split_file(&config);
        assert!(matches!(result, Err(SplitError::InvalidChunkSize(_))));
    }
    
    #[test]
    fn test_split_file_nonexistent_input() {
        let config = SplitConfig {
            input_path: "nonexistent_file.txt".to_string(),
            chunk_size: 1024,
            ..Default::default()
        };
        
        let result = split_file(&config);
        assert!(matches!(result, Err(SplitError::InvalidInputPath(_))));
    }
    
    #[test]
    fn test_split_file_with_output_dir() -> Result<()> {
        // Create a temporary directory for testing
        let temp_dir = tempdir()?;
        let input_path = temp_dir.path().join("test.txt");
        let output_dir = temp_dir.path().join("output");
        
        // Create a test file
        let mut file = File::create(&input_path)?;
        file.write_all(b"This is a test file")?;
        
        // Test with output directory
        let config = SplitConfig {
            input_path: input_path.to_str().unwrap().to_string(),
            output_dir: Some(output_dir.to_str().unwrap().to_string()),
            chunk_size: 10, // Small chunk size for testing
            ..Default::default()
        };
        
        split_file(&config)?;
        
        // Verify output directory was created
        assert!(output_dir.exists());
        assert!(output_dir.is_dir());
        
        Ok(())
    }
}
