use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashSet;
use chrono;

/// Configuration for the archiving process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Input directory to archive
    pub input: PathBuf,
    /// Output file path
    pub output: PathBuf,
    /// Include hidden files and directories
    pub include_hidden: bool,
    /// Maximum file size to include (in bytes)
    pub max_file_size: Option<u64>,
    /// Enable parallel processing
    pub parallel: bool,
    /// Include git information (if available)
    #[serde(default)]
    pub git_info: bool,
    /// Output format
    #[serde(default = "default_format")]
    pub format: OutputFormat,
    /// File patterns to include (glob format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
    /// File patterns to exclude (glob format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
    /// Enable LLM-optimized filtering
    #[serde(default)]
    pub llm_optimize: bool,
    /// Show filter statistics
    #[serde(default)]
    pub show_filter_stats: bool,
    /// File extensions to include (comma-separated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_extensions: Option<String>,
    /// Maximum depth for directory traversal
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<usize>,
    /// Follow symbolic links
    #[serde(default)]
    pub follow_links: bool,
    /// Output verbosity level
    #[serde(default = "default_verbosity")]
    pub verbosity: u8,
}

/// Output format for the archive
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum OutputFormat {
    /// Plain text format
    Text,
    /// JSON format
    Json,
    /// Markdown format
    Markdown,
    /// Rich text with syntax highlighting
    RichText,
}

fn default_format() -> OutputFormat {
    OutputFormat::Text
}

fn default_verbosity() -> u8 {
    1
}

impl Default for Config {
    fn default() -> Self {
        // Generate a timestamp string in the format YYYYMMDD_HHMMSS
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        
        // Default to current directory if input path isn't set yet
        let input = PathBuf::from(".");
        
        // Create output filename with timestamp
        let output = input.join(format!("archive_{}.txt", timestamp));
        
        Self {
            input,
            output,
            include_hidden: false,
            max_file_size: Some(10 * 1024 * 1024), // 10MB default max size
            parallel: true,
            git_info: true,
            format: OutputFormat::Text,
            include: None,
            exclude: None,
            llm_optimize: true,
            show_filter_stats: true,
            include_extensions: None,
            max_depth: None,
            follow_links: false,
            verbosity: 1,
        }
    }
}

impl Config {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the input directory and update output to be in the same directory with a timestamp
    pub fn with_input(mut self, input: impl Into<PathBuf>) -> Self {
        let input_path = input.into();
        self.input = input_path.clone();
        
        // Only update output if it's still using the default or if it's in a different directory
        let default_output = PathBuf::from("archive.txt");
        if self.output == default_output || self.output.parent() != Some(&input_path) {
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
            let dir_name = input_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("archive");
            self.output = input_path.join(format!("{}_archive_{}.txt", dir_name, timestamp));
        }
        
        self
    }

    /// Set the output file path
    /// If a directory is provided, creates a timestamped filename in that directory
    /// If a file is provided, uses that exact path
    pub fn with_output(mut self, output: impl Into<PathBuf>) -> Self {
        let output_path = output.into();
        
        // If the output path is a directory, create a timestamped filename in that directory
        if output_path.is_dir() || output_path.extension().is_none() {
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
            let dir_name = self.input.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("archive");
            self.output = output_path.join(format!("{}_archive_{}.txt", dir_name, timestamp));
        } else {
            self.output = output_path;
        }
        
        self
    }

    /// Set whether to include hidden files
    pub fn with_include_hidden(mut self, include_hidden: bool) -> Self {
        self.include_hidden = include_hidden;
        self
    }

    /// Set whether to use parallel processing
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Set whether to include git information
    pub fn with_git_info(mut self, git_info: bool) -> Self {
        self.git_info = git_info;
        self
    }

    /// Set the output format
    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    /// Set file patterns to include
    pub fn with_include_patterns(mut self, patterns: Vec<String>) -> Self {
        self.include = Some(patterns);
        self
    }

    /// Set file patterns to exclude
    pub fn with_exclude_patterns(mut self, patterns: Vec<String>) -> Self {
        self.exclude = Some(patterns);
        self
    }

    /// Enable or disable LLM optimization
    pub fn with_llm_optimize(mut self, enable: bool) -> Self {
        self.llm_optimize = enable;
        self
    }

    /// Set file extensions to include (comma-separated)
    pub fn with_include_extensions(mut self, extensions: &str) -> Self {
        self.include_extensions = Some(extensions.to_string());
        self
    }

    /// Set maximum depth for directory traversal
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    /// Set whether to follow symbolic links
    pub fn with_follow_links(mut self, follow: bool) -> Self {
        self.follow_links = follow;
        self
    }

    /// Set the output verbosity level (0-3)
    pub fn with_verbosity(mut self, level: u8) -> Self {
        self.verbosity = level.min(3);
        self
    }

    /// Get the default LLM ignore patterns
    pub fn get_default_llm_ignore_patterns() -> Vec<&'static str> {
        vec![
            // Build artifacts
            "**/target/", "**/build/", "**/dist/", "**/node_modules/", "**/__pycache__/",
            // Version control
            "**/.git/", "**/.svn/", "**/.hg/",
            // OS generated files
            "**/.DS_Store", "**/Thumbs.db",
            // Editor files
            "**/*.swp", "**/*.swo", "**/*.swn", "**/*.swo", "**/*.swn",
            // Logs and databases
            "**/*.log", "**/*.sqlite", "**/*.db",
        ]
    }

    /// Get the set of file extensions to include
    pub fn get_included_extensions(&self) -> Option<HashSet<String>> {
        self.include_extensions.as_ref().map(|exts| {
            exts.split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect()
        })
    }
}
