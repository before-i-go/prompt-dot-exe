use std::path::Path;

use super::config::OutputFormat;

pub mod text;

pub trait Formatter: Send + Sync {
    fn format_header(&self) -> String;
    fn format_file(&self, path: &Path, content: &str) -> String;
    fn format_footer(&self, file_count: usize) -> String;
}

pub fn create_formatter(format: OutputFormat) -> Box<dyn Formatter> {
    match format {
        OutputFormat::Text => Box::new(text::TextFormatter::new()),
        _ => unimplemented!("Other formatters not implemented yet"),
    }
}
