use serde::Serialize;
use std::path::PathBuf;
use std::time::Duration;

/// Statistics about the archiving process
#[derive(Debug, Default, Clone, Serialize)]
pub struct ArchiveStats {
    /// Number of files processed
    pub files_processed: usize,
    /// Number of files skipped
    pub files_skipped: usize,
    /// Number of errors encountered
    pub error_count: usize,
    /// Total size of processed files in bytes
    pub total_size: u64,
    /// Duration of the archiving process
    pub duration: Duration,
    /// Path to the output file
    pub output_path: PathBuf,
}

impl ArchiveStats {
    /// Create a new ArchiveStats instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of files processed
    pub fn with_files_processed(mut self, count: usize) -> Self {
        self.files_processed = count;
        self
    }

    /// Set the number of files skipped
    pub fn with_files_skipped(mut self, count: usize) -> Self {
        self.files_skipped = count;
        self
    }

    /// Set the number of errors encountered
    pub fn with_error_count(mut self, count: usize) -> Self {
        self.error_count = count;
        self
    }

    /// Set the total size of processed files
    pub fn with_total_size(mut self, size: u64) -> Self {
        self.total_size = size;
        self
    }

    /// Set the duration of the archiving process
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Set the output path
    pub fn with_output_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.output_path = path.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_archive_stats() {
        let stats = ArchiveStats::new()
            .with_files_processed(10)
            .with_files_skipped(2)
            .with_error_count(1)
            .with_total_size(1024)
            .with_duration(Duration::from_secs(1))
            .with_output_path("output.txt");

        assert_eq!(stats.files_processed, 10);
        assert_eq!(stats.files_skipped, 2);
        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.total_size, 1024);
        assert_eq!(stats.duration, Duration::from_secs(1));
        assert_eq!(stats.output_path, PathBuf::from("output.txt"));
    }
}
