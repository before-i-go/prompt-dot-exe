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
    /// Include directory tree structure in output
    #[serde(default)]
    pub include_tree: bool,
}

/// Output format for the archive
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum OutputFormat {
    /// Plain text format
    Plain,
    /// JSON format
    Json,
    /// Markdown format
    Markdown,
    /// Rich text with syntax highlighting
    RichText,
}

fn default_format() -> OutputFormat {
    OutputFormat::Plain
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
            format: OutputFormat::Plain,
            include: None,
            exclude: None,
            llm_optimize: true,
            show_filter_stats: true,
            include_extensions: None,
            max_depth: None,
            follow_links: false,
            verbosity: 1,
            include_tree: true,
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

    /// Set whether to include directory tree structure in output
    pub fn with_include_tree(mut self, include_tree: bool) -> Self {
        self.include_tree = include_tree;
        self
    }

    /// Get comprehensive LLM ignore patterns for cleaner training data
    ///
    /// This method returns a comprehensive list of file patterns that should be
    /// excluded when preparing code for LLM training. The patterns are based on
    /// best practices from the AI/ML community and cover:
    ///
    /// - Build artifacts and compiled outputs
    /// - Dependencies and package manager files
    /// - Cache and temporary files
    /// - IDE and editor configuration files
    /// - OS-generated files
    /// - Version control metadata
    /// - Logs and databases
    /// - Environment and secret files
    /// - Binary media files
    /// - Archives and compressed files
    /// - Test coverage reports
    /// - Language-specific compiled files
    /// - Cloud and deployment configurations
    /// - Mobile development artifacts
    /// - Game development assets
    /// - Large data files and ML models
    ///
    /// These exclusions help create cleaner, more focused training datasets
    /// that contain primarily source code and documentation rather than
    /// generated artifacts or binary files.
    pub fn get_default_llm_ignore_patterns() -> Vec<&'static str> {
        vec![
            // Build artifacts and outputs
            "**/target/", "**/build/", "**/dist/", "**/out/", "**/bin/", "**/obj/", "**/output/", "**/release/", "**/debug/",
            "**/*.exe", "**/*.dll", "**/*.so", "**/*.dylib", "**/*.a", "**/*.lib", "**/*.pdb", "**/*.ilk", "**/*.exp", "**/*.map",
            
            // Dependencies and package managers
            "**/node_modules/", "**/vendor/", "**/deps/", "**/packages/", "**/bower_components/", "**/.pnp/", "**/.yarn/",
            "**/venv/", "**/env/", "**/.venv/", "**/.env/", "**/virtualenv/", "**/site-packages/",
            "**/pip-log.txt", "**/pip-delete-this-directory.txt",
            
            // Cache and temporary files
            "**/.cache/", "**/tmp/", "**/temp/", "**/.tmp/", "**/.temp/",
            "**/*.tmp", "**/*.temp", "**/*.swp", "**/*.swo", "**/*~", "**/*.bak", "**/*.backup", "**/*.orig", "**/*.rej",
            "**/.#*", "**/#*#",
            
            // IDE and editor files
            "**/.vscode/", "**/.idea/", "**/*.iml", "**/.project", "**/.classpath", "**/.settings/",
            "**/*.sublime-*", "**/.vs/", "**/.vscode-test/", "**/*.code-workspace", "**/.history/", "**/.ionide/",
            "**/*.iws", "**/.metadata/", "**/.recommenders/",
            
            // OS generated files
            "**/.DS_Store", "**/.DS_Store?", "**/._*", "**/.Spotlight-V100", "**/.Trashes",
            "**/ehthumbs.db", "**/Thumbs.db", "**/desktop.ini", "**/*.lnk", "**/$RECYCLE.BIN/",
            
            // Version control (beyond .git)
            "**/.git/", "**/.hg/", "**/.svn/", "**/.bzr/", "**/.fossil-settings/",
            
            // Logs and databases
            "**/*.log", "**/*.db", "**/*.sqlite", "**/*.sqlite3", "**/logs/", "**/log/",
            "**/*.ldf", "**/*.mdf", "**/*.ndf",
            
            // Environment and configuration files (may contain secrets)
            "**/.env", "**/.env.local", "**/.env.development", "**/.env.test", "**/.env.production", "**/.env.staging",
            "**/*.env", "**/config.json", "**/secrets.json", "**/*.key", "**/*.pem", "**/*.crt", "**/*.cer",
            "**/*.p12", "**/*.pfx", "**/*.jks", "**/*.keystore",
            
            // Documentation that's not code
            "**/*.pdf", "**/*.doc", "**/*.docx", "**/*.ppt", "**/*.pptx", "**/*.xls", "**/*.xlsx",
            "**/*.odt", "**/*.ods", "**/*.odp", "**/*.rtf", "**/*.pages", "**/*.numbers", "**/*.keynote",
            
            // Media files
            "**/*.png", "**/*.jpg", "**/*.jpeg", "**/*.gif", "**/*.bmp", "**/*.ico", "**/*.tiff", "**/*.tif",
            "**/*.webp", "**/*.svg", "**/*.eps", "**/*.ai", "**/*.psd", "**/*.sketch", "**/*.fig",
            "**/*.mp4", "**/*.avi", "**/*.mkv", "**/*.mov", "**/*.wmv", "**/*.flv", "**/*.webm",
            "**/*.m4v", "**/*.3gp", "**/*.ogv", "**/*.mp3", "**/*.wav", "**/*.flac", "**/*.aac",
            "**/*.ogg", "**/*.wma", "**/*.m4a", "**/*.opus",
            
            // Archives
            "**/*.zip", "**/*.tar", "**/*.tar.gz", "**/*.tgz", "**/*.rar", "**/*.7z", "**/*.bz2",
            "**/*.xz", "**/*.lzma", "**/*.gz", "**/*.Z", "**/*.deb", "**/*.rpm", "**/*.msi",
            "**/*.dmg", "**/*.pkg", "**/*.app",
            
            // Test coverage and reports
            "**/coverage/", "**/test-results/", "**/htmlcov/", "**/.nyc_output/", "**/.coverage",
            "**/*.cover", "**/*.py,cover", "**/.hypothesis/", "**/.pytest_cache/", "**/nosetests.xml",
            "**/coverage.xml", "**/*.lcov", "**/lcov.info",
            
            // Language-specific compiled/generated files
            "**/*.pyc", "**/*.pyo", "**/*.pyd", "**/__pycache__/", "**/*.class", "**/*.jar",
            "**/*.war", "**/*.ear", "**/*.nar", "**/*.o", "**/*.obj", "**/*.hi", "**/*.dyn_hi",
            "**/*.dyn_o", "**/*.beam", "**/*.native", "**/*.byte", "**/*.cmi", "**/*.cmo",
            "**/*.cmx", "**/*.cmxa", "**/*.cma", "**/*.cmxs",
            
            // Language-specific build directories
            "**/.stack-work/", "**/.cabal-sandbox/", "**/cabal.sandbox.config", "**/dist-newstyle/",
            "**/.gradle/", "**/gradlew", "**/gradlew.bat", "**/cmake-build-*/", "**/CMakeFiles/",
            "**/CMakeCache.txt", "**/cmake_install.cmake", "**/install_manifest.txt", "**/Makefile",
            
            // Lock files (usually generated)
            "**/package-lock.json", "**/yarn.lock", "**/pnpm-lock.yaml", "**/Cargo.lock",
            "**/Pipfile.lock", "**/composer.lock", "**/Gemfile.lock", "**/poetry.lock",
            "**/mix.lock", "**/pubspec.lock", "**/stack.yaml.lock", "**/flake.lock",
            
            // Cloud and deployment
            "**/.terraform/", "**/*.tfstate", "**/*.tfstate.*", "**/*.tfplan", "**/*.tfvars",
            "**/.pulumi/", "**/.serverless/", "**/.vercel/", "**/.netlify/", "**/.next/",
            "**/.nuxt/", "**/.output/", "**/.firebase/", "**/.gcloud/", "**/.aws/", "**/cdk.out/",
            
            // Docker
            "**/.dockerignore", "**/Dockerfile.*", "**/.docker/",
            
            // Mobile development
            "**/*.ipa", "**/*.apk", "**/*.aab", "**/*.dSYM/", "**/*.xcarchive/", "**/*.xcworkspace/",
            "**/*.xcodeproj/", "**/DerivedData/", "**/*.hmap", "**/*.xcuserstate", "**/project.xcworkspace",
            "**/xcuserdata/",
            
            // Unity
            "**/[Ll]ibrary/", "**/[Tt]emp/", "**/[Oo]bj/", "**/[Bb]uild/", "**/[Bb]uilds/",
            "**/[Ll]ogs/", "**/[Mm]emoryCaptures/", "**/[Uu]serSettings/", "**/*.user", "**/*.userprefs",
            "**/*.pidb", "**/*.booproj", "**/*.svd", "**/*.mdb", "**/*.opendb", "**/*.VC.db",
            
            // Game development
            "**/*.blend1", "**/*.blend2", "**/*.fbx", "**/*.max", "**/*.maya", "**/*.mb", "**/*.ma",
            "**/*.3ds", "**/*.dae", "**/*.mtl", "**/*.dds", "**/*.tga", "**/*.exr", "**/*.hdr",
            
            // Fonts
            "**/*.ttf", "**/*.otf", "**/*.woff", "**/*.woff2", "**/*.eot",
            
            // Data files that are typically large/binary
            "**/*.csv", "**/*.tsv", "**/*.parquet", "**/*.h5", "**/*.hdf5", "**/*.nc", "**/*.mat",
            "**/*.npz", "**/*.npy", "**/*.pickle", "**/*.pkl", "**/*.joblib", "**/*.model",
            "**/*.weights", "**/*.pt", "**/*.pth", "**/*.ckpt", "**/*.pb", "**/*.tflite",
            "**/*.onnx", "**/*.mlmodel", "**/*.coreml", "**/datasets/", "**/data/",
            "**/*.bin", "**/*.dat", "**/*.raw",
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
