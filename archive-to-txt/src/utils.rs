use std::path::Path;
use std::time::SystemTime;
use chrono::{DateTime, Local};

/// Format a file path for display in the archive.
///
/// This function converts a file path to a string representation suitable for
/// display in the archive, using forward slashes as path separators.
///
/// # Arguments
/// * `path` - The path to format
///
/// # Returns
/// A string representation of the path with forward slashes as separators.
pub fn format_path<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .display()
        .to_string()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

/// Format a file size in a human-readable format.
///
/// This function converts a file size in bytes to a human-readable string
/// with appropriate units (e.g., "1.5 MB").
///
/// # Arguments
/// * `size` - The size in bytes
///
/// # Returns
/// A formatted string with appropriate unit (B, KB, MB, GB, etc.)
pub fn format_file_size(size: u64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.2} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.2} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Format a timestamp in a human-readable format.
///
/// This function converts a `SystemTime` to a human-readable string
/// in the local timezone.
///
/// # Arguments
/// * `time` - The system time to format
///
/// # Returns
/// A formatted string representation of the time
pub fn format_timestamp(time: SystemTime) -> String {
    let datetime: DateTime<Local> = time.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::SystemTime;

    #[test]
    fn test_format_utilities() {
        // Test path formatting
        let path = PathBuf::from("path/to/file.txt");
        let formatted = format_path(&path);
        assert_eq!(formatted, "path/to/file.txt");
        
        // Test file size formatting
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(1023), "1023 B");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
        
        // Test timestamp formatting (just verify it produces some output)
        let timestamp = SystemTime::now();
        let formatted_time = format_timestamp(timestamp);
        assert!(!formatted_time.is_empty(), "Timestamp should not be empty");
    }
}
