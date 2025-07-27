use std::path::Path;
use super::Formatter;
use crate::tree::{generate_tree, generate_structure_summary, TreeConfig};
use crate::config::Config;

pub struct PlainTextFormatter;

impl PlainTextFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Formatter for PlainTextFormatter {
    fn format_header(&self, config: Option<&Config>) -> String {
        let mut header = String::new();
        header.push_str("Archive Contents\n");
        header.push_str("================\n\n");
        
        // Add directory tree if requested and config is available
        if let Some(config) = config {
            if config.include_tree {
                header.push_str(&self.generate_directory_tree(config));
                header.push_str("\n");
                header.push_str("================================================\n");
                header.push_str("FILE CONTENTS\n");
                header.push_str("================================================\n\n");
            }
        }
        
        header
    }

    fn format_file(&self, path: &Path, content: &str) -> String {
        format!("\n================================================\nFILE: {}\n================================================\n{}\n", path.display(), content)
    }

    fn format_footer(&self, file_count: usize) -> String {
        format!("\n================================================\nSUMMARY\n================================================\nTotal files processed: {}\n", file_count)
    }
}

impl PlainTextFormatter {
    fn generate_directory_tree(&self, config: &Config) -> String {
        let tree_config = TreeConfig {
            include_hidden: config.include_hidden,
            max_depth: config.max_depth,
            follow_links: config.follow_links,
            include_extensions: config.get_included_extensions().map(|set| {
                set.into_iter().collect()
            }),
            max_file_size: config.max_file_size,
        };
        
        match generate_tree(&config.input, &tree_config) {
            Ok(tree) => generate_structure_summary(&tree),
            Err(e) => format!("Error generating directory tree: {}\n", e),
        }
    }
}

// Keep the old name for backward compatibility
pub type TextFormatter = PlainTextFormatter;
