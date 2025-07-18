//! Frequency analysis for pattern detection
//! 
//! This module will implement the FrequencyAnalyzer struct for scanning content
//! and building frequency maps for patterns 4+ characters long.

use crate::compression::{FrequencyAnalysis, CompressionError};
use std::collections::HashMap;
use tracing::{info, warn, error, debug, instrument, span, Level};

/// Frequency analyzer for detecting common patterns in code
#[derive(Debug)]
pub struct FrequencyAnalyzer {
    min_pattern_length: usize,
    min_frequency_threshold: usize,
    pattern_frequencies: HashMap<String, usize>,
}

impl FrequencyAnalyzer {
    /// Create a new frequency analyzer
    pub fn new(min_length: usize, min_frequency: usize) -> Self {
        Self {
            min_pattern_length: min_length,
            min_frequency_threshold: min_frequency,
            pattern_frequencies: HashMap::new(),
        }
    }
}

impl FrequencyAnalysis for FrequencyAnalyzer {
    /// Analyze content for pattern frequencies
    /// 
    /// Scans the content for patterns of minimum length and tracks their frequencies
    #[instrument(name = "analyze_content", skip(self, content), fields(content_size = content.len()))]
    fn analyze_content(&mut self, content: &str) {
        if content.is_empty() {
            debug!("Skipping empty content");
            return;
        }
        
        debug!(
            content_size = content.len(),
            min_pattern_length = self.min_pattern_length,
            "Starting pattern analysis"
        );
        
        let initial_pattern_count = self.pattern_frequencies.len();
        let mut patterns_processed = 0;
        let mut patterns_skipped = 0;
        
        // Extract patterns of minimum length or greater
        for window_size in self.min_pattern_length..=content.len().min(50) {
            for window in content.as_bytes().windows(window_size) {
                if let Ok(pattern) = std::str::from_utf8(window) {
                    patterns_processed += 1;
                    
                    // Skip patterns that are just whitespace or single repeated characters
                    if pattern.trim().is_empty() || pattern.chars().all(|c| c == pattern.chars().next().unwrap()) {
                        patterns_skipped += 1;
                        continue;
                    }
                    
                    // Skip patterns with too many non-alphanumeric characters
                    let alphanumeric_count = pattern.chars().filter(|c| c.is_alphanumeric()).count();
                    if alphanumeric_count < pattern.len() / 2 {
                        patterns_skipped += 1;
                        continue;
                    }
                    
                    *self.pattern_frequencies.entry(pattern.to_string()).or_insert(0) += 1;
                }
            }
        }
        
        let final_pattern_count = self.pattern_frequencies.len();
        let new_patterns = final_pattern_count - initial_pattern_count;
        
        debug!(
            patterns_processed = patterns_processed,
            patterns_skipped = patterns_skipped,
            new_patterns_discovered = new_patterns,
            total_unique_patterns = final_pattern_count,
            "Pattern analysis completed"
        );
    }
    
    /// Get patterns that meet frequency threshold
    /// 
    /// Returns patterns sorted by frequency (descending) that meet the minimum threshold
    #[instrument(name = "get_frequent_patterns", skip(self))]
    fn get_frequent_patterns(&self) -> Vec<(String, usize)> {
        debug!(
            total_patterns = self.pattern_frequencies.len(),
            min_frequency_threshold = self.min_frequency_threshold,
            "Filtering patterns by frequency threshold"
        );
        
        let mut patterns: Vec<_> = self.pattern_frequencies
            .iter()
            .filter(|(_, &freq)| freq >= self.min_frequency_threshold)
            .map(|(pattern, &freq)| (pattern.clone(), freq))
            .collect();
        
        let filtered_count = patterns.len();
        
        // Sort by frequency descending, then by pattern length descending for deterministic results
        patterns.sort_by(|a, b| {
            b.1.cmp(&a.1).then_with(|| b.0.len().cmp(&a.0.len())).then_with(|| a.0.cmp(&b.0))
        });
        
        if !patterns.is_empty() {
            let max_freq = patterns[0].1;
            let min_freq = patterns.last().unwrap().1;
            debug!(
                frequent_patterns_found = filtered_count,
                max_frequency = max_freq,
                min_frequency = min_freq,
                "Pattern filtering and sorting completed"
            );
        } else {
            warn!("No patterns meet the frequency threshold");
        }
        
        patterns
    }
    
    /// Check if pattern should be compressed
    /// 
    /// Returns true if the pattern meets the frequency threshold and length requirements
    fn should_compress_pattern(&self, pattern: &str) -> bool {
        if pattern.len() < self.min_pattern_length {
            return false;
        }
        
        self.pattern_frequencies
            .get(pattern)
            .map(|&freq| freq >= self.min_frequency_threshold)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_frequency_analyzer_creation() {
        let analyzer = FrequencyAnalyzer::new(4, 3);
        assert_eq!(analyzer.min_pattern_length, 4);
        assert_eq!(analyzer.min_frequency_threshold, 3);
        assert!(analyzer.pattern_frequencies.is_empty());
    }
    
    #[test]
    fn test_analyze_content_basic() {
        let mut analyzer = FrequencyAnalyzer::new(4, 2);
        let content = "function test() { function demo() { function main() { } } }";
        
        analyzer.analyze_content(content);
        
        let patterns = analyzer.get_frequent_patterns();
        assert!(!patterns.is_empty());
        
        // "function" should be detected as a frequent pattern
        assert!(patterns.iter().any(|(pattern, _)| pattern.contains("function")));
    }
    
    #[test]
    fn test_analyze_content_empty() {
        let mut analyzer = FrequencyAnalyzer::new(4, 3);
        analyzer.analyze_content("");
        
        let patterns = analyzer.get_frequent_patterns();
        assert!(patterns.is_empty());
    }
    
    #[test]
    fn test_frequency_threshold_filtering() {
        let mut analyzer = FrequencyAnalyzer::new(4, 3);
        let content = "test test test rare rare";
        
        analyzer.analyze_content(content);
        
        let patterns = analyzer.get_frequent_patterns();
        
        // "test" appears 3 times, should be included
        assert!(analyzer.should_compress_pattern("test"));
        
        // "rare" appears 2 times, should not be included (threshold is 3)
        assert!(!analyzer.should_compress_pattern("rare"));
    }
    
    #[test]
    fn test_pattern_length_filtering() {
        let mut analyzer = FrequencyAnalyzer::new(5, 2);
        
        // Short patterns should not be compressed
        assert!(!analyzer.should_compress_pattern("test")); // length 4 < 5
        
        // Long enough patterns should be considered
        analyzer.pattern_frequencies.insert("testing".to_string(), 3);
        assert!(analyzer.should_compress_pattern("testing")); // length 7 >= 5
    }
    
    #[test]
    fn test_get_frequent_patterns_sorting() {
        let mut analyzer = FrequencyAnalyzer::new(4, 2);
        
        // Manually insert patterns with different frequencies
        analyzer.pattern_frequencies.insert("high_freq".to_string(), 10);
        analyzer.pattern_frequencies.insert("med_freq".to_string(), 5);
        analyzer.pattern_frequencies.insert("low_freq".to_string(), 2);
        analyzer.pattern_frequencies.insert("too_low".to_string(), 1); // Below threshold
        
        let patterns = analyzer.get_frequent_patterns();
        
        // Should be sorted by frequency descending
        assert_eq!(patterns.len(), 3); // too_low should be filtered out
        assert_eq!(patterns[0].0, "high_freq");
        assert_eq!(patterns[0].1, 10);
        assert_eq!(patterns[1].0, "med_freq");
        assert_eq!(patterns[1].1, 5);
        assert_eq!(patterns[2].0, "low_freq");
        assert_eq!(patterns[2].1, 2);
    }
    
    #[test]
    fn test_realistic_code_analysis() {
        let mut analyzer = FrequencyAnalyzer::new(4, 3);
        let code_content = r#"
            function main() {
                console.log("Hello World");
                return 0;
            }
            
            function test() {
                console.log("Test Message");
                return 1;
            }
            
            function demo() {
                console.log("Demo Message");
                return 2;
            }
        "#;
        
        analyzer.analyze_content(code_content);
        let patterns = analyzer.get_frequent_patterns();
        
        // Should find common patterns like "function", "return", "console.log"
        assert!(!patterns.is_empty());
        
        // Verify some expected patterns are found
        let pattern_strings: Vec<&String> = patterns.iter().map(|(p, _)| p).collect();
        assert!(pattern_strings.iter().any(|p| p.contains("function")));
        assert!(pattern_strings.iter().any(|p| p.contains("return")));
    }
}