//! Integrity validation for compressed archives and reconstruction
//!
//! Provides checksum validation, dictionary integrity checks, and reconstruction
//! verification to ensure data integrity throughout the compression pipeline.

use crate::compression::{CompressionError, CompressionResult};
use std::collections::HashMap;
use std::path::Path;
use sha2::{Sha256, Digest};
use crc32fast::Hasher as Crc32Hasher;

/// Integrity validator for compression operations
pub struct IntegrityValidator {
    checksums: HashMap<String, FileChecksum>,
    dictionary_hash: Option<String>,
    validation_mode: ValidationMode,
}

/// File checksum with multiple hash algorithms
#[derive(Debug, Clone)]
pub struct FileChecksum {
    pub crc32: u32,
    pub sha256: String,
    pub size: usize,
}

/// Validation mode for different integrity levels
#[derive(Debug, Clone, Copy)]
pub enum ValidationMode {
    /// Fast validation using CRC32 only
    Fast,
    /// Standard validation using CRC32 and SHA256
    Standard,
    /// Thorough validation with additional checks
    Thorough,
}

/// Integrity check result
#[derive(Debug, Clone)]
pub struct IntegrityReport {
    pub files_checked: usize,
    pub files_valid: usize,
    pub files_invalid: usize,
    pub dictionary_valid: bool,
    pub validation_errors: Vec<String>,
}

impl IntegrityValidator {
    /// Create a new integrity validator
    pub fn new(mode: ValidationMode) -> Self {
        Self {
            checksums: HashMap::new(),
            dictionary_hash: None,
            validation_mode: mode,
        }
    }

    /// Calculate checksum for file content
    pub fn calculate_checksum(&self, content: &[u8]) -> FileChecksum {
        let mut crc32_hasher = Crc32Hasher::new();
        crc32_hasher.update(content);
        let crc32 = crc32_hasher.finalize();

        let sha256 = if matches!(self.validation_mode, ValidationMode::Fast) {
            // Skip SHA256 for fast mode
            String::new()
        } else {
            let mut sha256_hasher = Sha256::new();
            sha256_hasher.update(content);
            format!("{:x}", sha256_hasher.finalize())
        };

        FileChecksum {
            crc32,
            sha256,
            size: content.len(),
        }
    }

    /// Add file checksum to validation set
    pub fn add_file_checksum(&mut self, path: &str, checksum: FileChecksum) {
        self.checksums.insert(path.to_string(), checksum);
    }

    /// Calculate checksum for file at path
    pub fn calculate_file_checksum<P: AsRef<Path>>(&self, path: P) -> CompressionResult<FileChecksum> {
        let content = std::fs::read(path.as_ref())
            .map_err(|e| CompressionError::file_processing(
                path.as_ref().to_path_buf(),
                format!("Failed to read file for checksum: {}", e)
            ))?;

        Ok(self.calculate_checksum(&content))
    }

    /// Validate file against stored checksum
    pub fn validate_file(&self, path: &str, content: &[u8]) -> CompressionResult<bool> {
        let stored_checksum = self.checksums.get(path)
            .ok_or_else(|| CompressionError::integrity_check(
                format!("No checksum found for file: {}", path)
            ))?;

        let current_checksum = self.calculate_checksum(content);

        // Always check CRC32 and size
        if current_checksum.crc32 != stored_checksum.crc32 {
            return Ok(false);
        }

        if current_checksum.size != stored_checksum.size {
            return Ok(false);
        }

        // Check SHA256 if available and not in fast mode
        if !matches!(self.validation_mode, ValidationMode::Fast) &&
           !stored_checksum.sha256.is_empty() &&
           current_checksum.sha256 != stored_checksum.sha256 {
            return Ok(false);
        }

        Ok(true)
    }

    /// Set dictionary hash for validation
    pub fn set_dictionary_hash(&mut self, dictionary: &HashMap<String, String>) {
        let mut hasher = Sha256::new();

        // Sort dictionary entries for consistent hashing
        let mut entries: Vec<_> = dictionary.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));

        for (pattern, token) in entries {
            hasher.update(pattern.as_bytes());
            hasher.update(b":");
            hasher.update(token.as_bytes());
            hasher.update(b"\n");
        }

        self.dictionary_hash = Some(format!("{:x}", hasher.finalize()));
    }

    /// Validate dictionary integrity
    pub fn validate_dictionary(&self, dictionary: &HashMap<String, String>) -> CompressionResult<bool> {
        let stored_hash = self.dictionary_hash.as_ref()
            .ok_or_else(|| CompressionError::integrity_check(
                "No dictionary hash available for validation".to_string()
            ))?;

        // Calculate current hash
        let mut hasher = Sha256::new();
        let mut entries: Vec<_> = dictionary.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));

        for (pattern, token) in entries {
            hasher.update(pattern.as_bytes());
            hasher.update(b":");
            hasher.update(token.as_bytes());
            hasher.update(b"\n");
        }

        let current_hash = format!("{:x}", hasher.finalize());

        Ok(current_hash == *stored_hash)
    }

    /// Validate bidirectional dictionary mapping
    pub fn validate_dictionary_bidirectional(&self, dictionary: &HashMap<String, String>) -> CompressionResult<bool> {
        // Build reverse dictionary
        let mut reverse_dict = HashMap::new();

        for (pattern, token) in dictionary {
            // Check for token collisions
            if reverse_dict.contains_key(token) {
                return Err(CompressionError::integrity_check(
                    format!("Token collision detected: {} maps to multiple patterns", token)
                ));
            }
            reverse_dict.insert(token.clone(), pattern.clone());
        }

        // Validate sizes match
        if dictionary.len() != reverse_dict.len() {
            return Err(CompressionError::integrity_check(
                format!("Dictionary size mismatch: forward={}, reverse={}",
                       dictionary.len(), reverse_dict.len())
            ));
        }

        // Validate bidirectional consistency
        for (pattern, token) in dictionary {
            match reverse_dict.get(token) {
                Some(reverse_pattern) => {
                    if reverse_pattern != pattern {
                        return Err(CompressionError::integrity_check(
                            format!("Bidirectional mapping inconsistency: '{}' -> '{}' -> '{}'",
                                   pattern, token, reverse_pattern)
                        ));
                    }
                }
                None => {
                    return Err(CompressionError::integrity_check(
                        format!("Missing reverse mapping for token: {}", token)
                    ));
                }
            }
        }

        Ok(true)
    }

    /// Validate token format consistency
    pub fn validate_token_format(&self, dictionary: &HashMap<String, String>) -> CompressionResult<bool> {
        let token_regex = regex::Regex::new(r"^T[0-9A-F]{4}$").unwrap();

        for (pattern, token) in dictionary {
            if !token_regex.is_match(token) {
                return Err(CompressionError::integrity_check(
                    format!("Invalid token format '{}' for pattern '{}'", token, pattern)
                ));
            }
        }

        Ok(true)
    }

    /// Perform comprehensive integrity validation
    pub fn validate_comprehensive(&self,
        files: &[(String, Vec<u8>)],
        dictionary: &HashMap<String, String>
    ) -> CompressionResult<IntegrityReport> {
        let mut report = IntegrityReport {
            files_checked: 0,
            files_valid: 0,
            files_invalid: 0,
            dictionary_valid: true,
            validation_errors: Vec::new(),
        };

        // Validate dictionary first
        if let Err(e) = self.validate_dictionary_bidirectional(dictionary) {
            report.dictionary_valid = false;
            report.validation_errors.push(format!("Dictionary validation failed: {}", e));
        }

        if let Err(e) = self.validate_token_format(dictionary) {
            report.dictionary_valid = false;
            report.validation_errors.push(format!("Token format validation failed: {}", e));
        }

        if self.dictionary_hash.is_some() {
            if let Err(e) = self.validate_dictionary(dictionary) {
                report.dictionary_valid = false;
                report.validation_errors.push(format!("Dictionary hash validation failed: {}", e));
            }
        }

        // Validate files
        for (path, content) in files {
            report.files_checked += 1;

            match self.validate_file(path, content) {
                Ok(true) => {
                    report.files_valid += 1;
                }
                Ok(false) => {
                    report.files_invalid += 1;
                    report.validation_errors.push(format!("File validation failed: {}", path));
                }
                Err(e) => {
                    report.files_invalid += 1;
                    report.validation_errors.push(format!("File validation error for '{}': {}", path, e));
                }
            }
        }

        Ok(report)
    }

    /// Generate integrity manifest for archive
    pub fn generate_manifest(&self,
        files: &[(String, Vec<u8>)],
        dictionary: &HashMap<String, String>
    ) -> CompressionResult<String> {
        let mut manifest = String::new();

        // Add header
        manifest.push_str("# Integrity Manifest\n");
        manifest.push_str(&format!("# Generated: {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        manifest.push_str(&format!("# Validation Mode: {:?}\n", self.validation_mode));
        manifest.push_str("\n");

        // Add dictionary hash
        if let Some(hash) = &self.dictionary_hash {
            manifest.push_str(&format!("DICT_HASH:{}\n", hash));
        }
        manifest.push_str("\n");

        // Add file checksums
        manifest.push_str("# File Checksums\n");
        for (path, content) in files {
            let checksum = self.calculate_checksum(content);
            manifest.push_str(&format!("FILE:{}:{}:{}:{}\n",
                                      path, checksum.crc32, checksum.sha256, checksum.size));
        }

        Ok(manifest)
    }

    /// Parse integrity manifest
    pub fn parse_manifest(&mut self, manifest: &str) -> CompressionResult<()> {
        for line in manifest.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with("DICT_HASH:") {
                self.dictionary_hash = Some(line[10..].to_string());
            } else if line.starts_with("FILE:") {
                let parts: Vec<&str> = line[5..].split(':').collect();
                if parts.len() == 4 {
                    let path = parts[0].to_string();
                    let crc32 = parts[1].parse::<u32>()
                        .map_err(|_| CompressionError::integrity_check("Invalid CRC32 in manifest".to_string()))?;
                    let sha256 = parts[2].to_string();
                    let size = parts[3].parse::<usize>()
                        .map_err(|_| CompressionError::integrity_check("Invalid size in manifest".to_string()))?;

                    self.add_file_checksum(&path, FileChecksum { crc32, sha256, size });
                }
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for IntegrityReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Integrity Validation Report:")?;
        writeln!(f, "  Files checked: {}", self.files_checked)?;
        writeln!(f, "  Files valid: {}", self.files_valid)?;
        writeln!(f, "  Files invalid: {}", self.files_invalid)?;
        writeln!(f, "  Dictionary valid: {}", self.dictionary_valid)?;

        if !self.validation_errors.is_empty() {
            writeln!(f, "  Validation errors:")?;
            for error in &self.validation_errors {
                writeln!(f, "    - {}", error)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_calculation() {
        let validator = IntegrityValidator::new(ValidationMode::Standard);
        let content = b"Hello, World!";

        let checksum = validator.calculate_checksum(content);
        assert_eq!(checksum.size, 13);
        assert_ne!(check
