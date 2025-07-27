use std::path::Path;

use super::config::{OutputFormat, Config};

pub mod text;

pub trait Formatter: Send + Sync {
    fn format_header(&self, config: Option<&Config>) -> String;
    fn format_file(&self, path: &Path, content: &str) -> String;
    fn format_footer(&self, file_count: usize) -> String;
}

pub fn create_formatter(format: OutputFormat) -> Box<dyn Formatter> {
    match format {
        OutputFormat::Plain => Box::new(text::PlainTextFormatter::new()),
        _ => Box::new(text::PlainTextFormatter::new()), // Default to plain text
    }
}
