use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub input: PathBuf,
    pub output: PathBuf,
    pub include_hidden: bool,
    pub max_file_size: Option<u64>,
    pub parallel: bool,
    #[serde(default)]
    pub git_info: bool,
    #[serde(default = "default_format")]
    pub format: OutputFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
    Markdown,
}

fn default_format() -> OutputFormat {
    OutputFormat::Text
}

impl Default for Config {
    fn default() -> Self {
        Self {
            input: PathBuf::from("."),
            output: PathBuf::from("archive.txt"),
            include_hidden: false,
            max_file_size: None,
            parallel: true,
            git_info: false,
            format: OutputFormat::Text,
            include: None,
            exclude: None,
        }
    }
}

impl Config {
    pub fn with_input(mut self, input: impl Into<PathBuf>) -> Self {
        self.input = input.into();
        self
    }

    pub fn with_output(mut self, output: impl Into<PathBuf>) -> Self {
        self.output = output.into();
        self
    }

    pub fn with_include_hidden(mut self, include_hidden: bool) -> Self {
        self.include_hidden = include_hidden;
        self
    }

    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }
}
