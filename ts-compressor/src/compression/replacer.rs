//! Pattern replacement for content transformation
//! 
//! Implements PatternReplacer for applying dictionary replacements to content
//! using idiomatic Rust patterns including Cow<str> for zero-copy optimization,
//! functional programming patterns, and comprehensive error handling.

use crate::compression::PatternReplacement;
use std::borrow::Cow;
use std::collections::HashMap;
use tracing::{info, warn, error, debug, instrument, span, Level};

/// Pattern replacer for applying dictionary transformations
/// 
/// Uses efficient string replacement algorithms and zero-copy optimization
/// where possible to minimize memory allocations during pattern replacement.
#[derive(Debug, Clone)]
pub struct PatternReplacer {
    dictionary: HashMap<String, String>,
    /// Patterns sorted by length (descending) to handle overlapping patterns correctly
    sorted_patterns: Vec<String>,
}

impl PatternReplacer {
    /// Create a new pattern replacer with the given dictionary
    /// 
    /// Patterns are sorted by length (descending) to ensure longer patterns
    /// are replaced before shorter ones, preventing incorrect partial replacements.
    pub fn new(dictionary: HashMap<String, String>) -> Self {
        let mut sorted_patterns: Vec<String> = dictionary.keys().cloned().collect();
        // Sort by length descending, then alphabetically for deterministic behavior
        sorted_patterns.sort_by(|a, b| {
            b.len().cmp(&a.len()).then_with(|| a.cmp(b))
        });
        
        Self {
            dictionary,
            sorted_patterns,
        }
    }
    
    /// Create a pattern replacer from a vector of (pattern, token) pairs
    pub fn from_entries(entries: Vec<(String, String)>) -> Self {
        let dictionary: HashMap<String, String> = entries.into_iter().collect();
        Self::new(dictionary)
    }
    
    /// Get the number of patterns in the dictionary
    pub fn pattern_count(&self) -> usize {
        self.dictionary.len()
    }
    
    /// Check if the dictionary is empty
    pub fn is_empty(&self) -> bool {
        self.dictionary.is_empty()
    }
    
    /// Get the replacement token for a pattern
    pub fn get_token(&self, pattern: &str) -> Option<&String> {
        self.dictionary.get(pattern)
    }
    
    /// Check if a pattern exists in the dictionary
    pub fn contains_pattern(&self, pattern: &str) -> bool {
        self.dictionary.contains_key(pattern)
    }
    
    /// Get all patterns sorted by replacement priority (length descending)
    pub fn get_sorted_patterns(&self) -> &[String] {
        &self.sorted_patterns
    }
}

impl PatternReplacement for PatternReplacer {
    /// Replace patterns in content using the dictionary
    /// 
    /// Uses a greedy approach, replacing longer patterns first to avoid
    /// incorrect partial replacements. Returns the transformed content.
    #[instrument(name = "replace_patterns", skip(self, content), fields(content_size = content.len(), dictionary_size = self.dictionary.len()))]
    fn replace_patterns(&self, content: &str) -> String {
        if self.dictionary.is_empty() || content.is_empty() {
            debug!(
                dictionary_empty = self.dictionary.is_empty(),
                content_empty = content.is_empty(),
                "Skipping pattern replacement"
            );
            return content.to_string();
        }
        
        debug!(
            content_size = content.len(),
            pattern_count = self.dictionary.len(),
            "Starting pattern replacement"
        );
        
        let mut result = Cow::Borrowed(content);
        let mut replacements_made = 0;
        let mut patterns_matched = 0;
        
        // Apply replacements in order of pattern length (longest first)
        for pattern in &self.sorted_patterns {
            if let Some(token) = self.dictionary.get(pattern) {
                // Only allocate new string if replacement is needed
                if result.contains(pattern) {
                    let before_len = result.len();
                    result = Cow::Owned(result.replace(pattern, token));
                    let after_len = result.len();
                    
                    patterns_matched += 1;
                    let occurrences = (before_len - after_len + token.len() * (before_len - after_len) / pattern.len()) / pattern.len();
                    replacements_made += occurrences;
                    
                    debug!(
                        pattern = %pattern,
                        token = %token,
                        pattern_length = pattern.len(),
                        token_length = token.len(),
                        estimated_occurrences = occurrences,
                        "Pattern replaced"
                    );
                }
            }
        }
        
        let final_result = result.into_owned();
        let compression_ratio = self.calculate_compression_ratio(content, &final_result);
        
        debug!(
            original_size = content.len(),
            compressed_size = final_result.len(),
            patterns_matched = patterns_matched,
            total_replacements = replacements_made,
            compression_ratio = compression_ratio,
            "Pattern replacement completed"
        );
        
        final_result
    }
    
    /// Calculate compression ratio between original and compressed content
    /// 
    /// Returns a value between 0.0 and 1.0, where lower values indicate
    /// better compression (smaller compressed size relative to original).
    fn calculate_compression_ratio(&self, original: &str, compressed: &str) -> f64 {
        if original.is_empty() {
            return if compressed.is_empty() { 0.0 } else { f64::INFINITY };
        }
        
        compressed.len() as f64 / original.len() as f64
    }
}

impl AsRef<HashMap<String, String>> for PatternReplacer {
    fn as_ref(&self) -> &HashMap<String, String> {
        &self.dictionary
    }
}

impl From<HashMap<String, String>> for PatternReplacer {
    fn from(dictionary: HashMap<String, String>) -> Self {
        Self::new(dictionary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pattern_replacer_creation() {
        let dictionary = HashMap::new();
        let replacer = PatternReplacer::new(dictionary);
        
        assert!(replacer.is_empty());
        assert_eq!(replacer.pattern_count(), 0);
    }
    
    #[test]
    fn test_pattern_replacer_from_entries() {
        let entries = vec![
            ("function".to_string(), "A0".to_string()),
            ("return".to_string(), "A1".to_string()),
        ];
        
        let replacer = PatternReplacer::from_entries(entries);
        assert_eq!(replacer.pattern_count(), 2);
        assert_eq!(replacer.get_token("function"), Some(&"A0".to_string()));
        assert_eq!(replacer.get_token("return"), Some(&"A1".to_string()));
    }
    
    #[test]
    fn test_basic_pattern_replacement() {
        let mut dictionary = HashMap::new();
        dictionary.insert("function".to_string(), "A0".to_string());
        dictionary.insert("return".to_string(), "A1".to_string());
        
        let replacer = PatternReplacer::new(dictionary);
        let content = "function test() { return 42; }";
        let result = replacer.replace_patterns(content);
        
        assert_eq!(result, "A0 test() { A1 42; }");
    }
    
    #[test]
    fn test_pattern_replacement_priority() {
        let mut dictionary = HashMap::new();
        dictionary.insert("test".to_string(), "A0".to_string());
        dictionary.insert("testing".to_string(), "A1".to_string());
        dictionary.insert("test_function".to_string(), "A2".to_string());
        
        let replacer = PatternReplacer::new(dictionary);
        let content = "test_function and testing and test";
        let result = replacer.replace_patterns(content);
        
        // Longer patterns should be replaced first
        assert_eq!(result, "A2 and A1 and A0");
    }
    
    #[test]
    fn test_overlapping_patterns() {
        let mut dictionary = HashMap::new();
        dictionary.insert("abc".to_string(), "X".to_string());
        dictionary.insert("abcd".to_string(), "Y".to_string());
        dictionary.insert("bcd".to_string(), "Z".to_string());
        
        let replacer = PatternReplacer::new(dictionary);
        let content = "abcd abc bcd";
        let result = replacer.replace_patterns(content);
        
        // Should replace "abcd" first (longest), then "abc", then "bcd"
        assert_eq!(result, "Y X Z");
    }
    
    #[test]
    fn test_no_replacement_needed() {
        let mut dictionary = HashMap::new();
        dictionary.insert("function".to_string(), "A0".to_string());
        dictionary.insert("return".to_string(), "A1".to_string());
        
        let replacer = PatternReplacer::new(dictionary);
        let content = "const x = 42;";
        let result = replacer.replace_patterns(content);
        
        // Content should remain unchanged
        assert_eq!(result, content);
    }
    
    #[test]
    fn test_empty_content() {
        let mut dictionary = HashMap::new();
        dictionary.insert("test".to_string(), "A0".to_string());
        
        let replacer = PatternReplacer::new(dictionary);
        let result = replacer.replace_patterns("");
        
        assert_eq!(result, "");
    }
    
    #[test]
    fn test_empty_dictionary() {
        let dictionary = HashMap::new();
        let replacer = PatternReplacer::new(dictionary);
        let content = "function test() { return 42; }";
        let result = replacer.replace_patterns(content);
        
        assert_eq!(result, content);
    }
    
    #[test]
    fn test_multiple_occurrences() {
        let mut dictionary = HashMap::new();
        dictionary.insert("test".to_string(), "A0".to_string());
        
        let replacer = PatternReplacer::new(dictionary);
        let content = "test test test";
        let result = replacer.replace_patterns(content);
        
        assert_eq!(result, "A0 A0 A0");
    }
    
    #[test]
    fn test_pattern_sorting() {
        let mut dictionary = HashMap::new();
        dictionary.insert("a".to_string(), "1".to_string());
        dictionary.insert("abc".to_string(), "2".to_string());
        dictionary.insert("ab".to_string(), "3".to_string());
        dictionary.insert("abcd".to_string(), "4".to_string());
        
        let replacer = PatternReplacer::new(dictionary);
        let sorted = replacer.get_sorted_patterns();
        
        // Should be sorted by length descending, then alphabetically
        assert_eq!(sorted, vec!["abcd", "abc", "ab", "a"]);
    }
    
    #[test]
    fn test_compression_ratio_calculation() {
        let replacer = PatternReplacer::new(HashMap::new());
        
        // Test normal compression
        let ratio = replacer.calculate_compression_ratio("hello world", "hello");
        assert!((ratio - 5.0/11.0).abs() < f64::EPSILON);
        
        // Test no compression
        let ratio = replacer.calculate_compression_ratio("hello", "hello");
        assert!((ratio - 1.0).abs() < f64::EPSILON);
        
        // Test expansion
        let ratio = replacer.calculate_compression_ratio("hi", "hello");
        assert!((ratio - 2.5).abs() < f64::EPSILON);
        
        // Test empty original
        let ratio = replacer.calculate_compression_ratio("", "hello");
        assert!(ratio.is_infinite());
        
        // Test both empty
        let ratio = replacer.calculate_compression_ratio("", "");
        assert!((ratio - 0.0).abs() < f64::EPSILON);
    }
    
    #[test]
    fn test_contains_pattern() {
        let mut dictionary = HashMap::new();
        dictionary.insert("function".to_string(), "A0".to_string());
        dictionary.insert("return".to_string(), "A1".to_string());
        
        let replacer = PatternReplacer::new(dictionary);
        
        assert!(replacer.contains_pattern("function"));
        assert!(replacer.contains_pattern("return"));
        assert!(!replacer.contains_pattern("const"));
    }
    
    #[test]
    fn test_from_hashmap() {
        let mut dictionary = HashMap::new();
        dictionary.insert("test".to_string(), "A0".to_string());
        
        let replacer = PatternReplacer::from(dictionary);
        assert_eq!(replacer.pattern_count(), 1);
        assert!(replacer.contains_pattern("test"));
    }
    
    #[test]
    fn test_as_ref() {
        let mut dictionary = HashMap::new();
        dictionary.insert("test".to_string(), "A0".to_string());
        
        let replacer = PatternReplacer::new(dictionary);
        let dict_ref: &HashMap<String, String> = replacer.as_ref();
        
        assert_eq!(dict_ref.get("test"), Some(&"A0".to_string()));
    }
    
    #[test]
    fn test_complex_replacement_scenario() {
        let mut dictionary = HashMap::new();
        dictionary.insert("function".to_string(), "A0".to_string());
        dictionary.insert("const".to_string(), "A1".to_string());
        dictionary.insert("return".to_string(), "A2".to_string());
        dictionary.insert("()".to_string(), "A3".to_string());
        dictionary.insert("{}".to_string(), "A4".to_string());
        
        let replacer = PatternReplacer::new(dictionary);
        let content = "function test() { const x = 42; return x; }";
        let result = replacer.replace_patterns(content);
        
        assert_eq!(result, "A0 testA3 { A1 x = 42; A2 x; }");
    }
    
    #[test]
    fn test_edge_case_single_character_patterns() {
        let mut dictionary = HashMap::new();
        dictionary.insert("a".to_string(), "1".to_string());
        dictionary.insert("b".to_string(), "2".to_string());
        
        let replacer = PatternReplacer::new(dictionary);
        let content = "ababa";
        let result = replacer.replace_patterns(content);
        
        assert_eq!(result, "12121");
    }
    
    #[test]
    fn test_unicode_content() {
        let mut dictionary = HashMap::new();
        dictionary.insert("函数".to_string(), "A0".to_string());
        dictionary.insert("返回".to_string(), "A1".to_string());
        
        let replacer = PatternReplacer::new(dictionary);
        let content = "函数 test() { 返回 42; }";
        let result = replacer.replace_patterns(content);
        
        assert_eq!(result, "A0 test() { A1 42; }");
    }
}