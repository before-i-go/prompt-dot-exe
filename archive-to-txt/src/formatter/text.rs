use std::path::Path;
use chrono::Local;
use super::Formatter;

pub struct TextFormatter;

impl TextFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Formatter for TextFormatter {
    fn format_header(&self) -> String {
        format!("Archive created at {}\n\n", Local::now().to_rfc3339())
    }

    fn format_file(&self, path: &Path, content: &str) -> String {
        format!("\n=== File: {} ===\n{}\n", path.display(), content)
    }

    fn format_footer(&self, file_count: usize) -> String {
        format!("\n=== Summary ===\nTotal files processed: {}\n", file_count)
    }
}
