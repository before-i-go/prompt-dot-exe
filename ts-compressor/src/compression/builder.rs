//! Dictionary building for pattern-token mapping
//! 
//! Creates bidirectional mappings between frequent patterns and hexadecimal tokens
//! using idiomatic Rust patterns including Builder pattern, From/Into traits,
//! and RAII for resource management.

use crate::compression::{DictionaryBuilding, CompressionError, HexTokenGenerator, TokenGeneration};
use std::collections::HashMap;
use tracing::{info, warn, error, debug, instrument, span, Level};

/// Builder for creating compression dictionaries with bidirectional mapping
/// 
/// Uses the Builder pattern with method chaining and RAII for resource management.
/// Prioritizes shorter tokens for more frequent patterns to maximize compression.
#[derive(Debug, Clone)]
pub struct DictionaryBuilder {
    token_generator: HexTokenGenerator,
    dictionary: HashMap<String, String>,
    reverse_dictionary: HashMap<String, String>,
}

impl DictionaryBuilder {
    /// Create a new dictionary builder
    pub fn new() -> Self {
        Self {
            token_generator: HexTokenGenerator::new(),
            dictionary: HashMap::new(),
            reverse_dictionary: HashMap::new(),
        }
    }
    
    /// Create a dictionary builder with custom token generator
    pub fn with_token_generator(token_generator: HexTokenGenerator) -> Self {
        Self {
            token_generator,
            dictionary: HashMap::new(),
            reverse_dictionary: HashMap::new(),
        }
    }
    
    /// Get the number of dictionary entries
    pub fn entry_count(&self) -> usize {
        self.dictionary.len()
    }
    
    /// Check if the dictionary is empty
    pub fn is_empty(&self) -> bool {
        self.dictionary.is_empty()
    }
    
    /// Clear all dictionary entries and reset token generator
    pub fn clear(&mut self) {
        self.dictionary.clear();
        self.reverse_dictionary.clear();
        self.token_generator.reset();
    }
    
    /// Get pattern by token (reverse lookup)
    pub fn get_pattern_by_token(&self, token: &str) -> Option<&String> {
        self.reverse_dictionary.get(token)
    }
}

impl Default for DictionaryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DictionaryBuilding for DictionaryBuilder {
    /// Build dictionary from frequent patterns, prioritizing shorter tokens for higher frequency
    /// 
    /// Patterns are sorted by frequency (descending) to ensure most frequent patterns
    /// get the shortest tokens for maximum compression efficiency.
    #[instrument(name = "build_dictionary", skip(self, patterns), fields(pattern_count = patterns.len()))]
    fn build_dictionary(&mut self, patterns: Vec<(String, usize)>) -> Result<(), CompressionError> {
        if patterns.is_empty() {
            debug!("No patterns provided for dictionary building");
            return Ok(());
        }
        
        info!(
            pattern_count = patterns.len(),
            "Starting dictionary building from patterns"
        );
        
        // Clear existing dictionary
        debug!("Clearing existing dictionary");
        self.clear();
        
        // Sort patterns by frequency (descending) to prioritize frequent patterns
        debug!("Sorting patterns by frequency");
        let mut sorted_patterns = patterns;
        sorted_patterns.sort_by(|a, b| b.1.cmp(&a.1));
        
        if !sorted_patterns.is_empty() {
            let max_freq = sorted_patterns[0].1;
            let min_freq = sorted_patterns.last().unwrap().1;
            debug!(
                max_frequency = max_freq,
                min_frequency = min_freq,
                "Pattern frequency range"
            );
        }
        
        // Build dictionary entries
        let mut entries_created = 0;
        let mut patterns_skipped = 0;
        
        for (pattern, frequency) in sorted_patterns {
            // Skip empty patterns or patterns that are too short
            if pattern.is_empty() {
                debug!("Skipping empty pattern");
                patterns_skipped += 1;
                continue;
            }
            
            // Check for duplicate patterns
            if self.dictionary.contains_key(&pattern) {
                error!(pattern = %pattern, "Duplicate pattern found");
                return Err(CompressionError::dictionary_build(
                    format!("Duplicate pattern found: '{}'", pattern)
                ));
            }
            
            // Generate next token
            debug!(
                pattern = %pattern,
                frequency = frequency,
                "Generating token for pattern"
            );
            
            let token = self.token_generator.next_token()
                .map_err(|e| {
                    error!(
                        pattern = %pattern,
                        error = %e,
                        "Failed to generate token"
                    );
                    CompressionError::dictionary_build(
                        format!("Failed to generate token for pattern '{}': {}", pattern, e)
                    )
                })?;
            
            // Check for token collision (should not happen with proper generator)
            if self.reverse_dictionary.contains_key(&token) {
                error!(token = %token, "Token collision detected");
                return Err(CompressionError::dictionary_build(
                    format!("Token collision detected: '{}'", token)
                ));
            }
            
            debug!(
                pattern = %pattern,
                token = %token,
                frequency = frequency,
                "Adding dictionary entry"
            );
            
            // Add to both dictionaries
            self.dictionary.insert(pattern.clone(), token.clone());
            self.reverse_dictionary.insert(token, pattern);
            entries_created += 1;
        }
        
        info!(
            entries_created = entries_created,
            patterns_skipped = patterns_skipped,
            total_dictionary_size = self.dictionary.len(),
            "Dictionary building completed"
        );
        
        Ok(())
    }
    
    /// Get replacement token for a pattern
    fn get_replacement_token(&self, pattern: &str) -> Option<&String> {
        self.dictionary.get(pattern)
    }
    
    /// Get all dictionary entries as (pattern, token) pairs
    /// 
    /// Returns entries sorted by token for deterministic output
    fn get_dictionary_entries(&self) -> Vec<(String, String)> {
        let mut entries: Vec<_> = self.dictionary.iter()
            .map(|(pattern, token)| (pattern.clone(), token.clone()))
            .collect();
        
        // Sort by token for deterministic output
        entries.sort_by(|a, b| a.1.cmp(&b.1));
        entries
    }
    
    /// Validate dictionary integrity
    /// 
    /// Ensures bidirectional mapping consistency and no collisions
    fn validate_dictionary(&self) -> Result<(), CompressionError> {
        // Check that both dictionaries have the same size
        if self.dictionary.len() != self.reverse_dictionary.len() {
            return Err(CompressionError::dictionary_build(
                format!(
                    "Dictionary size mismatch: forward={}, reverse={}",
                    self.dictionary.len(),
                    self.reverse_dictionary.len()
                )
            ));
        }
        
        // Validate bidirectional consistency
        for (pattern, token) in &self.dictionary {
            match self.reverse_dictionary.get(token) {
                Some(reverse_pattern) => {
                    if reverse_pattern != pattern {
                        return Err(CompressionError::dictionary_build(
                            format!(
                                "Bidirectional mapping inconsistency: pattern '{}' -> token '{}' -> pattern '{}'",
                                pattern, token, reverse_pattern
                            )
                        ));
                    }
                }
                None => {
                    return Err(CompressionError::dictionary_build(
                        format!("Missing reverse mapping for token '{}'", token)
                    ));
                }
            }
        }
        
        // Validate that patterns and tokens are non-empty
        for (pattern, token) in &self.dictionary {
            if pattern.is_empty() {
                return Err(CompressionError::dictionary_build(
                    "Empty pattern found in dictionary".to_string()
                ));
            }
            if token.is_empty() {
                return Err(CompressionError::dictionary_build(
                    "Empty token found in dictionary".to_string()
                ));
            }
        }
        
        Ok(())
    }
}

// Implement From trait for easy conversion from pattern vectors
impl TryFrom<Vec<(String, usize)>> for DictionaryBuilder {
    type Error = CompressionError;
    
    fn try_from(patterns: Vec<(String, usize)>) -> Result<Self, Self::Error> {
        let mut builder = Self::new();
        builder.build_dictionary(patterns)?;
        Ok(builder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dictionary_builder_creation() {
        let builder = DictionaryBuilder::new();
        assert!(builder.is_empty());
        assert_eq!(builder.entry_count(), 0);
    }
    
    #[test]
    fn test_build_dictionary_basic() {
        let mut builder = DictionaryBuilder::new();
        let patterns = vec![
            ("function".to_string(), 5),
            ("return".to_string(), 3),
            ("const".to_string(), 4),
        ];
        
        builder.build_dictionary(patterns).unwrap();
        
        assert_eq!(builder.entry_count(), 3);
        assert!(!builder.is_empty());
        
        // Most frequent pattern should get first token
        assert_eq!(builder.get_replacement_token("function"), Some(&"T0000".to_string()));
        assert_eq!(builder.get_replacement_token("const"), Some(&"T0001".to_string()));
        assert_eq!(builder.get_replacement_token("return"), Some(&"T0002".to_string()));
    }
    
    #[test]
    fn test_build_dictionary_frequency_priority() {
        let mut builder = DictionaryBuilder::new();
        let patterns = vec![
            ("low_freq".to_string(), 1),
            ("high_freq".to_string(), 10),
            ("med_freq".to_string(), 5),
        ];
        
        builder.build_dictionary(patterns).unwrap();
        
        // Higher frequency patterns should get shorter tokens first
        assert_eq!(builder.get_replacement_token("high_freq"), Some(&"T0000".to_string()));
        assert_eq!(builder.get_replacement_token("med_freq"), Some(&"T0001".to_string()));
        assert_eq!(builder.get_replacement_token("low_freq"), Some(&"T0002".to_string()));
    }
    
    #[test]
    fn test_build_dictionary_empty_patterns() {
        let mut builder = DictionaryBuilder::new();
        let patterns = vec![];
        
        builder.build_dictionary(patterns).unwrap();
        assert!(builder.is_empty());
    }
    
    #[test]
    fn test_build_dictionary_duplicate_patterns() {
        let mut builder = DictionaryBuilder::new();
        let patterns = vec![
            ("duplicate".to_string(), 5),
            ("duplicate".to_string(), 3),
        ];
        
        let result = builder.build_dictionary(patterns);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate pattern"));
    }
    
    #[test]
    fn test_build_dictionary_skips_empty_patterns() {
        let mut builder = DictionaryBuilder::new();
        let patterns = vec![
            ("valid".to_string(), 5),
            ("".to_string(), 3),
            ("also_valid".to_string(), 4),
        ];
        
        builder.build_dictionary(patterns).unwrap();
        
        // Should only have 2 entries, empty pattern skipped
        assert_eq!(builder.entry_count(), 2);
        assert!(builder.get_replacement_token("valid").is_some());
        assert!(builder.get_replacement_token("also_valid").is_some());
        assert!(builder.get_replacement_token("").is_none());
    }
    
    #[test]
    fn test_get_dictionary_entries_sorted() {
        let mut builder = DictionaryBuilder::new();
        let patterns = vec![
            ("zebra".to_string(), 1),
            ("alpha".to_string(), 3),
            ("beta".to_string(), 2),
        ];
        
        builder.build_dictionary(patterns).unwrap();
        let entries = builder.get_dictionary_entries();
        
        // Should be sorted by token (T0000, T0001, T0002)
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].1, "T0000"); // alpha (highest freq)
        assert_eq!(entries[1].1, "T0001"); // beta
        assert_eq!(entries[2].1, "T0002"); // zebra (lowest freq)
    }
    
    #[test]
    fn test_bidirectional_mapping() {
        let mut builder = DictionaryBuilder::new();
        let patterns = vec![
            ("test_pattern".to_string(), 5),
        ];
        
        builder.build_dictionary(patterns).unwrap();
        
        let token = builder.get_replacement_token("test_pattern").unwrap();
        let reverse_pattern = builder.get_pattern_by_token(token).unwrap();
        
        assert_eq!(reverse_pattern, "test_pattern");
    }
    
    #[test]
    fn test_validate_dictionary_success() {
        let mut builder = DictionaryBuilder::new();
        let patterns = vec![
            ("valid_pattern".to_string(), 5),
            ("another_pattern".to_string(), 3),
        ];
        
        builder.build_dictionary(patterns).unwrap();
        builder.validate_dictionary().unwrap();
    }
    
    #[test]
    fn test_clear_dictionary() {
        let mut builder = DictionaryBuilder::new();
        let patterns = vec![
            ("pattern".to_string(), 5),
        ];
        
        builder.build_dictionary(patterns).unwrap();
        assert!(!builder.is_empty());
        
        builder.clear();
        assert!(builder.is_empty());
        assert_eq!(builder.entry_count(), 0);
        
        // Should be able to build again from scratch
        let new_patterns = vec![("new_pattern".to_string(), 3)];
        builder.build_dictionary(new_patterns).unwrap();
        assert_eq!(builder.get_replacement_token("new_pattern"), Some(&"T0000".to_string()));
    }
    
    #[test]
    fn test_try_from_patterns() {
        let patterns = vec![
            ("pattern1".to_string(), 5),
            ("pattern2".to_string(), 3),
        ];
        
        let builder = DictionaryBuilder::try_from(patterns).unwrap();
        assert_eq!(builder.entry_count(), 2);
        assert!(builder.get_replacement_token("pattern1").is_some());
        assert!(builder.get_replacement_token("pattern2").is_some());
    }
    
    #[test]
    fn test_try_from_patterns_with_error() {
        let patterns = vec![
            ("duplicate".to_string(), 5),
            ("duplicate".to_string(), 3),
        ];
        
        let result = DictionaryBuilder::try_from(patterns);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_token_overflow_handling() {
        // Create a generator with very limited capacity
        let limited_generator = HexTokenGenerator::with_max_tokens(2).unwrap();
        let mut builder = DictionaryBuilder::with_token_generator(limited_generator);
        
        let patterns = vec![
            ("pattern1".to_string(), 5),
            ("pattern2".to_string(), 4),
            ("pattern3".to_string(), 3), // This should cause overflow
        ];
        
        let result = builder.build_dictionary(patterns);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to generate token"));
    }
    
    #[test]
    fn test_with_custom_token_generator() {
        let custom_generator = HexTokenGenerator::with_max_tokens(10).unwrap();
        let mut builder = DictionaryBuilder::with_token_generator(custom_generator);
        
        let patterns = vec![("test".to_string(), 5)];
        builder.build_dictionary(patterns).unwrap();
        
        assert_eq!(builder.get_replacement_token("test"), Some(&"T0000".to_string()));
    }
}