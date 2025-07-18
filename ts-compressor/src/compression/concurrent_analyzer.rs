//! Concurrent frequency analysis for pattern detection
//! 
//! This module implements thread-safe frequency analysis using lock-free data structures
//! for maximum performance in concurrent scenarios.

use crate::compression::{FrequencyAnalysis, CompressionError};
use dashmap::DashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::collections::HashMap;

/// Thread-safe frequency analyzer using concurrent data structures
#[derive(Debug)]
pub struct ConcurrentFrequencyAnalyzer {
    min_pattern_length: usize,
    min_frequency_threshold: usize,
    pattern_frequencies: Arc<DashMap<String, AtomicUsize>>,
}

impl ConcurrentFrequencyAnalyzer {
    /// Create a new concurrent frequency analyzer
    pub fn new(min_length: usize, min_frequency: usize) -> Self {
        Self {
            min_pattern_length: min_length,
            min_frequency_threshold: min_frequency,
            pattern_frequencies: Arc::new(DashMap::new()),
        }
    }
    
    /// Analyze a chunk of content concurrently
    pub fn analyze_chunk(&self, content: &str) {
        if content.is_empty() {
            return;
        }
        
        // Extract patterns of minimum length or greater
        for window_size in self.min_pattern_length..=content.len().min(50) {
            for window in content.as_bytes().windows(window_size) {
                if let Ok(pattern) = std::str::from_utf8(window) {
                    // Skip patterns that are just whitespace or single repeated characters
                    if pattern.trim().is_empty() || pattern.chars().all(|c| c == pattern.chars().next().unwrap()) {
                        continue;
                    }
                    
                    // Skip patterns with too many non-alphanumeric characters
                    let alphanumeric_count = pattern.chars().filter(|c| c.is_alphanumeric()).count();
                    if alphanumeric_count < pattern.len() / 2 {
                        continue;
                    }
                    
                    // Atomically increment the frequency count
                    self.pattern_frequencies
                        .entry(pattern.to_string())
                        .or_insert_with(|| AtomicUsize::new(0))
                        .fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }
    
    /// Merge local pattern frequencies from a single-threaded analyzer
    pub fn merge_local_patterns(&self, local_patterns: HashMap<String, usize>) {
        for (pattern, count) in local_patterns {
            self.pattern_frequencies
                .entry(pattern)
                .or_insert_with(|| AtomicUsize::new(0))
                .fetch_add(count, Ordering::Relaxed);
        }
    }
    
    /// Get the frequency of a specific pattern
    pub fn get_pattern_frequency(&self, pattern: &str) -> usize {
        self.pattern_frequencies
            .get(pattern)
            .map(|entry| entry.load(Ordering::Relaxed))
            .unwrap_or(0)
    }
    
    /// Clone the analyzer for use in another thread
    pub fn clone_for_thread(&self) -> Self {
        Self {
            min_pattern_length: self.min_pattern_length,
            min_frequency_threshold: self.min_frequency_threshold,
            pattern_frequencies: Arc::clone(&self.pattern_frequencies),
        }
    }
}

impl FrequencyAnalysis for ConcurrentFrequencyAnalyzer {
    /// Analyze content for pattern frequencies (delegates to analyze_chunk)
    fn analyze_content(&mut self, content: &str) {
        self.analyze_chunk(content);
    }
    
    /// Get patterns that meet frequency threshold
    fn get_frequent_patterns(&self) -> Vec<(String, usize)> {
        let mut patterns: Vec<_> = self.pattern_frequencies
            .iter()
            .map(|entry| {
                let pattern = entry.key().clone();
                let frequency = entry.value().load(Ordering::Relaxed);
                (pattern, frequency)
            })
            .filter(|(_, freq)| *freq >= self.min_frequency_threshold)
            .collect();
        
        // Sort by frequency descending, then by pattern length descending for deterministic results
        patterns.sort_by(|a, b| {
            b.1.cmp(&a.1).then_with(|| b.0.len().cmp(&a.0.len())).then_with(|| a.0.cmp(&b.0))
        });
        
        patterns
    }
    
    /// Check if pattern should be compressed
    fn should_compress_pattern(&self, pattern: &str) -> bool {
        if pattern.len() < self.min_pattern_length {
            return false;
        }
        
        self.get_pattern_frequency(pattern) >= self.min_frequency_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;
    use std::collections::HashMap;
    
    #[test]
    fn test_concurrent_analyzer_creation() {
        let analyzer = ConcurrentFrequencyAnalyzer::new(4, 3);
        assert_eq!(analyzer.min_pattern_length, 4);
        assert_eq!(analyzer.min_frequency_threshold, 3);
        assert!(analyzer.pattern_frequencies.is_empty());
    }
    
    #[test]
    fn test_analyze_chunk_basic() {
        let analyzer = ConcurrentFrequencyAnalyzer::new(4, 2);
        let content = "function test() { function demo() { function main() { } } }";
        
        analyzer.analyze_chunk(content);
        
        let patterns = analyzer.get_frequent_patterns();
        assert!(!patterns.is_empty());
        
        // "function" should be detected as a frequent pattern
        assert!(patterns.iter().any(|(pattern, _)| pattern.contains("function")));
    }
    
    #[test]
    fn test_concurrent_pattern_analysis() {
        let analyzer = Arc::new(ConcurrentFrequencyAnalyzer::new(4, 3));
        let content_chunks = vec![
            "function test() { return 'test'; }",
            "function demo() { return 'demo'; }",
            "function main() { return 'main'; }",
            "function helper() { return 'help'; }",
        ];
        
        let mut handles = vec![];
        
        // Spawn multiple threads to analyze different chunks
        for chunk in content_chunks {
            let analyzer_clone = Arc::clone(&analyzer);
            let chunk_owned = chunk.to_string();
            let handle = thread::spawn(move || {
                analyzer_clone.analyze_chunk(&chunk_owned);
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify that patterns were detected across all threads
        let patterns = analyzer.get_frequent_patterns();
        assert!(!patterns.is_empty());
        
        // "function" should appear 4 times (once per chunk)
        assert!(analyzer.get_pattern_frequency("function") >= 4);
        assert!(analyzer.should_compress_pattern("function"));
    }
    
    #[test]
    fn test_frequency_merging() {
        let analyzer = ConcurrentFrequencyAnalyzer::new(4, 3);
        
        // Create local patterns as if from single-threaded analysis
        let mut local_patterns1 = HashMap::new();
        local_patterns1.insert("test".to_string(), 2);
        local_patterns1.insert("demo".to_string(), 1);
        
        let mut local_patterns2 = HashMap::new();
        local_patterns2.insert("test".to_string(), 2);
        local_patterns2.insert("main".to_string(), 3);
        
        // Merge patterns from different sources
        analyzer.merge_local_patterns(local_patterns1);
        analyzer.merge_local_patterns(local_patterns2);
        
        // Verify merged frequencies
        assert_eq!(analyzer.get_pattern_frequency("test"), 4);
        assert_eq!(analyzer.get_pattern_frequency("demo"), 1);
        assert_eq!(analyzer.get_pattern_frequency("main"), 3);
        
        // Only "test" and "main" should meet the threshold of 3
        let frequent_patterns = analyzer.get_frequent_patterns();
        assert_eq!(frequent_patterns.len(), 2);
        assert!(frequent_patterns.iter().any(|(p, f)| p == "test" && *f == 4));
        assert!(frequent_patterns.iter().any(|(p, f)| p == "main" && *f == 3));
    }
    
    #[test]
    fn test_thread_safety() {
        let analyzer = Arc::new(ConcurrentFrequencyAnalyzer::new(4, 2));
        let num_threads = 10;
        let iterations_per_thread = 100;
        
        let mut handles = vec![];
        
        // Spawn multiple threads that all increment the same pattern
        for thread_id in 0..num_threads {
            let analyzer_clone = Arc::clone(&analyzer);
            let handle = thread::spawn(move || {
                for i in 0..iterations_per_thread {
                    let content = format!("test pattern {} {}", thread_id, i);
                    analyzer_clone.analyze_chunk(&content);
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify that all increments were properly recorded
        // "test" should appear num_threads * iterations_per_thread times
        let expected_frequency = num_threads * iterations_per_thread;
        assert_eq!(analyzer.get_pattern_frequency("test"), expected_frequency);
    }
    
    #[test]
    fn test_clone_for_thread() {
        let original = ConcurrentFrequencyAnalyzer::new(5, 2);
        original.analyze_chunk("testing pattern analysis");
        
        let cloned = original.clone_for_thread();
        
        // Both should share the same underlying data
        assert_eq!(cloned.min_pattern_length, 5);
        assert_eq!(cloned.min_frequency_threshold, 2);
        assert_eq!(
            cloned.get_pattern_frequency("testing"),
            original.get_pattern_frequency("testing")
        );
        
        // Modifications to one should be visible in the other
        cloned.analyze_chunk("testing again");
        assert_eq!(
            original.get_pattern_frequency("testing"),
            cloned.get_pattern_frequency("testing")
        );
    }
    
    #[test]
    fn test_pattern_frequency_threshold() {
        let analyzer = ConcurrentFrequencyAnalyzer::new(4, 3);
        
        // Add patterns with different frequencies
        let mut local_patterns = HashMap::new();
        local_patterns.insert("high_freq".to_string(), 5);
        local_patterns.insert("med_freq".to_string(), 3);
        local_patterns.insert("low_freq".to_string(), 2);
        
        analyzer.merge_local_patterns(local_patterns);
        
        // Only patterns meeting threshold should be returned
        let frequent_patterns = analyzer.get_frequent_patterns();
        assert_eq!(frequent_patterns.len(), 2);
        
        // Verify threshold checking
        assert!(analyzer.should_compress_pattern("high_freq"));
        assert!(analyzer.should_compress_pattern("med_freq"));
        assert!(!analyzer.should_compress_pattern("low_freq"));
    }
    
    #[test]
    fn test_pattern_sorting() {
        let analyzer = ConcurrentFrequencyAnalyzer::new(4, 1);
        
        let mut local_patterns = HashMap::new();
        local_patterns.insert("medium_freq".to_string(), 5);
        local_patterns.insert("highest_freq".to_string(), 10);
        local_patterns.insert("lowest_freq".to_string(), 2);
        
        analyzer.merge_local_patterns(local_patterns);
        
        let patterns = analyzer.get_frequent_patterns();
        
        // Should be sorted by frequency descending
        assert_eq!(patterns.len(), 3);
        assert_eq!(patterns[0].0, "highest_freq");
        assert_eq!(patterns[0].1, 10);
        assert_eq!(patterns[1].0, "medium_freq");
        assert_eq!(patterns[1].1, 5);
        assert_eq!(patterns[2].0, "lowest_freq");
        assert_eq!(patterns[2].1, 2);
    }
}