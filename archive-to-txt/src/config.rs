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
            // Version control
            "**/.git/", "**/.svn/", "**/.hg/", "**/.gitignore", "**/.gitmodules", "**/.gitattributes",
    
            // Build artifacts
            "**/target/", "**/build/", "**/dist/", "**/node_modules/", "**/__pycache__/",
            "**/*.pyc", "**/*.pyo", "**/*.pyd", "**/*.so", "**/*.dll", "**/*.dylib",
            "**/*.a", "**/*.lib", "**/*.o", "**/*.obj", "**/*.class", "**/*.jar", "**/*.war",
    
            // Package managers and dependencies
            "**/package-lock.json", "**/yarn.lock", "**/Cargo.lock", "**/Gemfile.lock",
            "**/Pipfile.lock", "**/poetry.lock", "**/yarn-error.log", "**/requirements*.txt",
            "**/requirements/*.txt", "**/constraints.txt", "**/setup.cfg", "**/setup.py",
    
            // Environment and configuration
            "**/.env", "**/.env.*", "**/.venv/", "**/venv/", "**/env/", "**/ENV/",
            "**/env.bak/", "**/venv.bak/", "**/.python-version", "**/.ruby-version",
            "**/.node-version", "**/.nvmrc", "**/.editorconfig", "**/.prettierrc",
            "**/.eslintrc*", "**/.babelrc*", "**/tsconfig.json", "**/jsconfig.json",
    
            // IDE and editor files
            "**/.idea/", "**/.vscode/", "**/*.swp", "**/*.swo", "**/*.swn",
            "**/.DS_Store", "**/Thumbs.db", "**/.vs/", "**/*.sublime-*", "**/.history/",
            "**/.vscode-test/", "**/.vscode/extensions.json", "**/.vscode/settings.json",
    
            // Logs and databases
            "**/*.log", "**/*.sqlite", "**/*.db", "**/*.sql", "**/*.sqlite3",
            "**/*.sqlite-journal", "**/*.sqlite3-journal", "**/*.db-journal",
            "**/logs/", "**/log/", "**/var/log/",
    
            // Archives and binaries
            "**/*.zip", "**/*.tar.gz", "**/*.tgz", "**/*.7z", "**/*.rar", "**/*.tar",
            "**/*.exe", "**/*.dmg", "**/*.pkg", "**/*.app", "**/*.msi", "**/*.deb",
            "**/*.rpm", "**/*.snap",
    
            // Media and binary files
            "**/*.png", "**/*.jpg", "**/*.jpeg", "**/*.gif", "**/*.bmp", "**/*.tiff",
            "**/*.ico", "**/*.svg", "**/*.mp3", "**/*.wav", "**/*.mp4", "**/*.avi",
            "**/*.mov", "**/*.wmv", "**/*.flv", "**/*.mkv", "**/*.webp", "**/*.webm",
            "**/*.woff", "**/*.woff2", "**/*.ttf", "**/*.eot", "**/*.otf",
    
            // Documents
            "**/*.pdf", "**/*.doc", "**/*.docx", "**/*.xls", "**/*.xlsx", "**/*.ppt",
            "**/*.pptx", "**/*.odt", "**/*.ods", "**/*.odp", "**/*.epub", "**/*.mobi",
    
            // Virtual machines and containers
            "**/.vagrant/", "**/*.vagrant/", "**/*.vbox", "**/*.vbox-prev", "**/Vagrantfile",
            "**/Dockerfile", "**/docker-compose*.yml", "**/.dockerignore", "**/.docker/",
            "**/compose.yml", "**/docker-compose.override.yml",
    
            // OS generated files
            "**/ehthumbs.db", "**/Thumbs.db", "**/desktop.ini", "**/$RECYCLE.BIN/",
            "**/Thumbs.db:encryptable", "**/ehthumbs_vista.db", "**/Desktop.ini",
    
            // Python specific
            "**/__pycache__/", "**/*.py[cod]", "**/*$py.class", "**/.pytest_cache/",
            "**/.mypy_cache/", "**/.pytest_cache/", "**/.coverage", "**/htmlcov/",
            "**/*.cover", "**/*.py,cover", "**/.hypothesis/", "**/.pytest/",
    
            // Node.js specific
            "**/node_modules/", "**/.npm/", "**/.yarn-integrity", "**/.yarn/cache/",
            "**/.yarn/unplugged/", "**/.yarn/build-state.yml", "**/.yarn/install-state.gz",
            "**/.pnp.*", "**/.yarnrc.yml", "**/yarn-debug.log*", "**/yarn-error.log*",
    
            // Rust specific
            "**/target/", "**/Cargo.lock", "**/*.rs.bk", "**/Cargo.toml.orig",
    
            // Java specific
            "**/.classpath", "**/.project", "**/.settings/", "**/*.class",
            "**/bin/", "**/build/", "**/out/", "**/*.iml",
    
            // Go specific
            "**/bin/", "**/pkg/", "**/vendor/", "**/go.work", "**/go.work.sum",
    
            // Web and frontend
            "**/dist/", "**/build/", "**/.next/", "**/out/", "**/.nuxt/", "**/.output/",
            "**/.svelte-kit/", "**/.astro/", "**/.cache/", "**/.parcel-cache/",
            "**/.turbo/", "**/.vercel/", "**/.netlify/",
    
            // Testing and coverage
            "**/coverage/", "**/.nyc_output/", "**/coverage-*.lcov", "**/lcov.info",
            "**/.jest-cache/", "**/jest.config.*", "**/karma.conf.*", "**/test-results/",
    
            // Documentation
            "**/docs/_build/", "**/docs/api/", "**/site/", "**/.vuepress/", "**/storybook-static/",
    
            // Development tools
            "**/.github/", "**/.circleci/", "**/.travis.yml", "**/.gitlab-ci.yml",
            "**/Jenkinsfile", "**/azure-pipelines.yml", "**/.github/workflows/*.yaml",
            "**/.pre-commit-config.yaml", "**/.commitlintrc*", "**/.husky/",
    
            // Temporary files
            "**/*.swp", "**/*.swo", "**/*.swn", "**/*.swo", "**/*.swn", "**/*.bak",
            "**/*.backup", "**/*.tmp", "**/*.temp", "**/*~", "**/*.orig", "**/*.rej",
    
            // macOS specific
            "**/.DS_Store", "**/._*", "**/.Spotlight-V100", "**/.Trashes", "**/ehthumbs.db",
    
            // Windows specific
            "**/Thumbs.db", "**/Desktop.ini", "**/Thumbs.db:encryptable",
    
            // Linux specific
            "**/.directory", "**/.Trash-*", "**/.nfs*",
    
            // Additional patterns (from researched templates)
            "**/.tox/", "**/.eggs/", "**/*.egg", "**/*.egg-info/", "**/.ipynb_checkpoints/", // Python extras
            "**/celerybeat-schedule", "**/celerybeat.pid", "**/*.sage.py", "**/.pyre/", "**/.pytype/", "**/cython_debug/",
            "**/bower_components/", "**/.bower-cache/", "**/npm-debug.log", "**/.eslintcache", // Node extras
            "**/.gradle/", "**/gradlew", "**/gradlew.bat", "**/.mvn/", "**/mvnw", "**/mvnw.cmd", // Java extras
            "**/vendor/", "**/composer.lock", "**/composer.phar", // PHP
            "**/.bundle/", "**/vendor/bundle/", "**/vendor/cache/", "**/*.gem", // Ruby extras
            "**/_build/", "**/deps/", "**/mix.lock", // Elixir
            // Added from filter-rules.txt and archival ref.rs for more comprehensive filtering
            "**/pnpm-lock.yaml", "**/yarn-error.log", "**/.npm/", "**/.yarn-integrity", "**/.yarn/cache/", "**/.yarn/unplugged/",
            "**/public/hot", "**/public/storage", "**/storage/app/public", "**/storage/framework/sessions/*", "**/storage/framework/views/*",
            "**/storage/framework/cache/data/*", "**/storage/logs/*.log", "Homestead.yaml", "Homestead.json", "**/var/", "**/public/bundles/",
            "**/.metadata/", "**/.recommenders/", "**/Carthage/Build/", "**/Carthage/Checkouts/", "**/Pods/", "**/.swiftpm/",
            "**/DerivedData/", "**/*.xcodeproj/project.xcworkspace/", "**/*.xcodeproj/xcuserdata/", "**/*.xcworkspace/contents.xcworkspacedata",
            "**/*.xcworkspace/xcuserdata/", "**/app/build/", "**/*.apk", "**/*.aab", "**/captures/", "**/*.jks", "**/*.keystore",
            "local.properties", "**/dist/", "**/gradle-wrapper.jar", "hs_err_pid*", "**/tmp/", "**/.dvc/cache", "**/.dvc/tmp",
            "**/mlruns/", "**/mlflow-artifacts/", "**/.cache/huggingface/", "**/.terraform/", "**/*.tfstate", "**/*.tfstate.*",
            "**/crash.log", "**/*.tfvars", "**/*.tfvars.json", "**/.terraformrc", "**/terraform.rc", "**/Pulumi.*.yaml",
            "**/.azure-config/", "**/.aws/", "**/.azure/", "**/.config/gcloud/", "**/.dbeaver-data-sources.xml", "**/postman/backups/",
            "**/.sonarqube/", "**/_NCrunch_*/", "**/*.crt", "**/*.csr", "**/*.ca", "**/*.pfx", "**/*.p12", "**/*.key",
            "**/*.pem", "**/priv/static/", "**/*.native", "**/*.byte", "**/*.cmi", "**/*.cmo", "**/*.cmx", "**/*.cmxa", "**/*.cma",
            "**/*.cmxs", "**/cmake-build-*/", "**/CMakeFiles/", "**/CMakeCache.txt", "**/cmake_install.cmake", "**/install_manifest.txt",
            "**/Makefile", "**/cabal.sandbox.config", "**/dist-newstyle/", "**/.cabal-sandbox/", "**/.stack-work/", "**/pubspec.lock",
            "**/stack.yaml.lock", "**/flake.lock", "**/cdk.out/", "**/.firebase/", "**/.gcloud/", "**/.vercel/", "**/.serverless/",
            "**/.pulumi/", "**/local.properties", "**/xcuserdata/", "**/project.xcworkspace", "**/*.xcuserstate", "**/*.hmap",
            "**/build/", "**/DerivedData/", "**/*.xcodeproj/", "**/*.xcworkspace/", "**/*.xcarchive/", "**/*.dSYM/", "**/*.app",
            "**/*.ipa", "**/.docker/", "**/Dockerfile.*", "**/.dockerignore", "**/data/", "**/datasets/", "**/*.coreml", "**/*.mlmodel",
            "**/*.onnx", "**/*.tflite", "**/*.pb", "**/*.h5", "**/*.ckpt", "**/*.pth", "**/*.pt", "**/*.weights", "**/*.model",
            "**/*.joblib", "**/*.pkl", "**/*.pickle", "**/*.npy", "**/*.npz", "**/*.mat", "**/*.nc", "**/*.hdf5", "**/*.h5",
            "**/*.parquet", "**/*.xml", "**/*.json", "**/*.tsv", "**/*.csv", "**/*.raw", "**/*.dat", "**/*.bin", "**/*.eot",
            "**/*.woff2", "**/*.woff", "**/*.m4a", "**/*.opus", "**/*.wma", "**/*.ogg", "**/*.aac", "**/*.flac", "**/*.wav",
            "**/*.mp3", "**/*.ogv", "**/*.3gp", "**/*.m4v", "**/*.webm", "**/*.flv", "**/*.wmv", "**/*.mov", "**/*.mkv",
            "**/*.avi", "**/*.mp4", "**/*.fig", "**/*.sketch", "**/*.psd", "**/*.ai", "**/*.eps", "**/*.svg", "**/*.webp",
            "**/*.tif", "**/*.tiff", "**/*.ico", "**/*.bmp", "**/*.gif", "**/*.jpeg", "**/*.jpg", "**/*.png", "**/*.keynote",
            "**/*.numbers", "**/*.pages", "**/*.rtf", "**/*.odp", "**/*.ods", "**/*.odt", "**/*.xlsx", "**/*.xls", "**/*.pptx",
            "**/*.ppt", "**/*.docx", "**/*.doc", "**/*.pdf", "**/azure-pipelines.yml", "**/Jenkinsfile", "**/.gitlab-ci.yml",
            "**/.travis.yml", "**/.circleci/", "**/.github/", "**/.vercel/", "**/.netlify/", "**/.turbo/", "**/.parcel-cache/",
            "**/.cache/", "**/.astro/", "**/.svelte-kit/", "**/.output/", "**/.nuxt/", "**/.next/", "**/karma.conf.*",
            "**/jest.config.*", "**/.jest-cache/", "**/lcov.info", "**/coverage-*.lcov", "**/.nyc_output/", "**/coverage/",
            "**/test-results/", "**/jest.config.*", "**/jest.config.*", "**/lcov.info", "**/coverage-*.lcov", "**/.nyc_output/",
            "**/coverage/", "**/storybook-static/", "**/.vuepress/", "**/site/", "**/docs/api/", "**/docs/_build/", "**/*.rej",
            "**/*.orig", "**/*~", "**/*.temp", "**/*.tmp", "**/*.backup", "**/*.bak", "**/.husky/", "**/.commitlintrc*",
            "**/.pre-commit-config.yaml", "**/.github/workflows/*.yaml", "**/azure-pipelines.yml", "**/Jenkinsfile",
            "**/.gitlab-ci.yml", "**/.travis.yml", "**/.circleci/", "**/.github/", "**/*.odp", "**/*.ods", "**/*.odt",
            "**/*.pptx", "**/*.ppt", "**/*.xlsx", "**/*.xls", "**/*.docx", "**/*.doc", "**/*.pdf", "**/*.otf", "**/*.eot",
            "**/*.ttf", "**/*.woff2", "**/*.woff", "**/*.webm", "**/*.webp", "**/*.mkv", "**/*.flv", "**/*.wmv", "**/*.mov",
            "**/*.avi", "**/*.mp4", "**/*.wav", "**/*.mp3", "**/*.svg", "**/*.ico", "**/*.tiff", "**/*.bmp", "**/*.gif",
            "**/*.jpeg", "**/*.jpg", "**/*.png", "**/*.raw", "**/*.dat", "**/*.bin", "**/data/", "**/datasets/", "**/*.coreml",
            "**/*.mlmodel", "**/*.onnx", "**/*.tflite", "**/*.pb", "**/*.h5", "**/*.ckpt", "**/*.pth", "**/*.pt", "**/*.weights",
            "**/*.model", "**/*.joblib", "**/*.pkl", "**/*.pickle", "**/*.npy", "**/*.npz", "**/*.mat", "**/*.nc", "**/*.hdf5",
            "**/*.h5", "**/*.parquet", "**/*.xml", "**/*.json", "**/*.tsv", "**/*.csv", "**/*.opus", "**/*.m4a", "**/*.wma",
            "**/*.ogg", "**/*.aac", "**/*.flac", "**/*.wav", "**/*.mp3", "**/*.ogv", "**/*.3gp", "**/*.m4v", "**/*.webm",
            "**/*.flv", "**/*.wmv", "**/*.mov", "**/*.mkv", "**/*.avi", "**/*.mp4", "**/*.fig", "**/*.sketch", "**/*.psd",
            "**/*.ai", "**/*.eps", "**/*.svg", "**/*.webp", "**/*.tif", "**/*.tiff", "**/*.ico", "**/*.bmp", "**/*.gif",
            "**/*.jpeg", "**/*.jpg", "**/*.png", "**/postman/backups/", "**/.dbeaver-data-sources.xml", "**/.config/gcloud/",
            "**/.azure/", "**/.aws/", "**/.azure-config/", "**/Pulumi.*.yaml", "**/.pulumi/", "**/*.tfvars.json", "**/*.tfvars",
            "**/crash.log", "**/*.tfstate.*", "**/*.tfstate", "**/.terraform/", "**/.cache/huggingface/", "**/mlflow-artifacts/",
            "**/mlruns/", "**/.dvc/tmp", "**/.dvc/cache", "**/.ipynb_checkpoints/", "**/*.raw", "**/*.dat", "**/*.bin",
            "**/data/", "**/datasets/", "**/*.coreml", "**/*.mlmodel", "**/*.onnx", "**/*.tflite", "**/*.pb", "**/*.h5",
            "**/*.ckpt", "**/*.pth", "**/*.pt", "**/*.weights", "**/*.model", "**/*.joblib", "**/*.pkl", "**/*.pickle",
            "**/*.npy", "**/*.npz", "**/*.mat", "**/*.nc", "**/*.hdf5", "**/*.h5", "**/*.parquet", "**/*.xml", "**/*.json",
            "**/*.tsv", "**/*.csv", "**/*.eot", "**/*.woff2", "**/*.woff", "**/*.opus", "**/*.m4a", "**/*.wma", "**/*.ogg",
            "**/*.aac", "**/*.flac", "**/*.wav", "**/*.mp3", "**/*.ogv", "**/*.3gp", "**/*.m4v", "**/*.webm", "**/*.flv",
            "**/*.wmv", "**/*.mov", "**/*.mkv", "**/*.avi", "**/*.mp4", "**/*.fig", "**/*.sketch", "**/*.psd", "**/*.ai",
            "**/*.eps", "**/*.svg", "**/*.webp", "**/*.tif", "**/*.tiff", "**/*.ico", "**/*.bmp", "**/*.gif", "**/*.jpeg",
            "**/*.jpg", "**/*.png", "**/keynote", "**/numbers", "**/pages", "**/rtf", "**/odp", "**/ods", "**/odt",
            "**/xlsx", "**/xls", "**/pptx", "**/ppt", "**/docx", "**/doc", "**/pdf", "**/azure-pipelines.yml", "**/Jenkinsfile",
            "**/.gitlab-ci.yml", "**/.travis.yml", "**/.circleci/", "**/.github/", "**/vercel/", "**/netlify/", "**/turbo/",
            "**/parcel-cache/", "**/cache/", "**/astro/", "**/svelte-kit/", "**/output/", "**/nuxt/", "**/next/",
            "**/karma.conf.*", "**/jest.config.*", "**/jest-cache/", "**/lcov.info", "**/coverage-*.lcov", "**/nyc_output/",
            "**/coverage/", "**/test-results/", "**/storybook-static/", "**/vuepress/", "**/site/", "**/docs/api/",
            "**/docs_build/", "**/rej", "**/orig", "**/~", "**/temp", "**/tmp", "**/backup", "**/bak", "**/husky/",
            "**/commitlintrc*", "**/pre-commit-config.yaml", "**/github/workflows/*.yaml", "**/azure-pipelines.yml",
            "**/Jenkinsfile", "**/gitlab-ci.yml", "**/travis.yml", "**/circleci/", "**/github/", "**/odp", "**/ods",
            "**/odt", "**/pptx", "**/ppt", "**/xlsx", "**/xls", "**/docx", "**/doc", "**/pdf", "**/otf", "**/eot",
            "**/ttf", "**/woff2", "**/woff", "**/webm", "**/webp", "**/mkv", "**/flv", "**/wmv", "**/mov", "**/avi",
            "**/mp4", "**/mp3", "**/wav", "**/flac", "**/aac", "**/ogg", "**/wma", "**/m4a", "**/opus", "**/ogv",
            "**/3gp", "**/m4v", "**/webm", "**/flv", "**/wmv", "**/mov", "**/mkv", "**/avi", "**/mp4", "**/fig",
            "**/sketch", "**/psd", "**/ai", "**/eps", "**/svg", "**/webp", "**/tif", "**/tiff", "**/ico", "**/bmp",
            "**/gif", "**/jpeg", "**/jpg", "**/png", "**/postman/backups/", "**/dbeaver-data-sources.xml",
            "**/config/gcloud/", "**/azure/", "**/aws/", "**/azure-config/", "**/Pulumi.*.yaml", "**/pulumi/",
            "**/tfvars.json", "**/tfvars", "**/crash.log", "**/tfstate.*", "**/tfstate", "**/terraform/",
            "**/cache/huggingface/", "**/mlflow-artifacts/", "**/mlruns/", "**/dvc/tmp", "**/dvc/cache",
            "**/ipynb_checkpoints/", "**/raw", "**/dat", "**/bin", "**/data/", "**/datasets/", "**/coreml",
            "**/mlmodel", "**/onnx", "**/tflite", "**/pb", "**/h5", "**/ckpt", "**/pth", "**/pt", "**/weights",
            "**/model", "**/joblib", "**/pkl", "**/pickle", "**/npy", "**/npz", "**/mat", "**/nc", "**/hdf5",
            "**/h5", "**/parquet", "**/xml", "**/json", "**/tsv", "**/csv", "**/opus", "**/m4a", "**/wma",
            "**/ogg", "**/aac", "**/flac", "**/wav", "**/mp3", "**/ogv", "**/3gp", "**/m4v", "**/webm",
            "**/flv", "**/wmv", "**/mov", "**/mkv", "**/avi", "**/mp4", "**/fig", "**/sketch", "**/psd",
            "**/ai", "**/eps", "**/svg", "**/webp", "**/tif", "**/tiff", "**/ico", "**/bmp", "**/gif",
            "**/jpeg", "**/jpg", "**/png", "**/keynote", "**/numbers", "**/pages", "**/rtf", "**/odp",
            "**/ods", "**/odt", "**/xlsx", "**/xls", "**/pptx", "**/ppt", "**/docx", "**/doc", "**/pdf",
            "**/otf", "**/eot", "**/ttf", "**/woff2", "**/woff", "**/webm", "**/webp", "**/mkv", "**/flv",
            "**/wmv", "**/mov", "**/avi", "**/mp4", "**/mp3", "**/wav", "**/flac", "**/aac", "**/ogg",
            "**/wma", "**/m4a", "**/opus", "**/ogv", "**/3gp", "**/m4v", "**/webm", "**/flv", "**/wmv",
            "**/mov", "**/mkv", "**/avi", "**/mp4", "**/fig", "**/sketch", "**/psd", "**/ai", "**/eps",
            "**/svg", "**/webp", "**/tif", "**/tiff", "**/ico", "**/bmp", "**/gif", "**/jpeg", "**/jpg",
            "**/png", "**/postman/backups/", "**/dbeaver-data-sources.xml", "**/config/gcloud/", "**/azure/",
            "**/aws/", "**/azure-config/", "**/Pulumi.*.yaml", "**/pulumi/", "**/tfvars.json", "**/tfvars",
            "**/crash.log", "**/tfstate.*", "**/tfstate", "**/terraform/", "**/cache/huggingface/",
            "**/mlflow-artifacts/", "**/mlruns/", "**/dvc/tmp", "**/dvc/cache", "**/ipynb_checkpoints/",
            "**/raw", "**/dat", "**/bin", "**/data/", "**/datasets/", "**/coreml", "**/mlmodel", "**/onnx",
            "**/tflite", "**/pb", "**/h5", "**/ckpt", "**/pth", "**/pt", "**/weights", "**/model", "**/joblib",
            "**/pkl", "**/pickle", "**/npy", "**/npz", "**/mat", "**/nc", "**/hdf5", "**/h5", "**/parquet",
            "**/xml", "**/json", "**/tsv", "**/csv", "**/opus", "**/m4a", "**/wma", "**/ogg", "**/aac",
            "**/flac", "**/wav", "**/mp3", "**/ogv", "**/3gp", "**/m4v", "**/webm", "**/flv", "**/wmv",
            "**/mov", "**/mkv", "**/avi", "**/mp4", "**/fig", "**/sketch", "**/psd", "**/ai", "**/eps",
            "**/svg", "**/webp", "**/tif", "**/tiff", "**/ico", "**/bmp", "**/gif", "**/jpeg", "**/jpg",
            "**/png", "**/keynote", "**/numbers", "**/pages", "**/rtf", "**/odp", "**/ods", "**/odt",
            "**/xlsx", "**/xls", "**/pptx", "**/ppt", "**/docx", "**/doc", "**/pdf", "**/otf", "**/eot",
            "**/ttf", "**/woff2", "**/woff", "**/webm", "**/webp", "**/mkv", "**/flv", "**/wmv", "**/mov",
            "**/avi", "**/mp4", "**/mp3", "**/wav", "**/flac", "**/aac", "**/ogg", "**/wma", "**/m4a",
            "**/opus", "**/ogv", "**/3gp", "**/m4v", "**/webm", "**/flv", "**/wmv", "**/mov", "**/mkv",
            "**/avi", "**/mp4", "**/fig", "**/sketch", "**/psd", "**/ai", "**/eps", "**/svg", "**/webp",
            "**/tif", "**/tiff", "**/ico", "**/bmp", "**/gif", "**/jpeg", "**/jpg", "**/png", "**/postman/backups/",
            "**/dbeaver-data-sources.xml", "**/config/gcloud/", "**/azure/", "**/aws/", "**/azure-config/",
            "**/Pulumi.*.yaml", "**/pulumi/", "**/tfvars.json", "**/tfvars", "**/crash.log", "**/tfstate.*",
            "**/tfstate", "**/terraform/", "**/cache/huggingface/", "**/mlflow-artifacts/", "**/mlruns/",
            "**/dvc/tmp", "**/dvc/cache", "**/ipynb_checkpoints/", "**/raw", "**/dat", "**/bin", "**/data/",
            "**/datasets/", "**/coreml", "**/mlmodel", "**/onnx", "**/tflite", "**/pb", "**/h5", "**/ckpt",
            "**/pth", "**/pt", "**/weights", "**/model", "**/joblib", "**/pkl", "**/pickle", "**/npy",
            "**/npz", "**/mat", "**/nc", "**/hdf5", "**/h5", "**/parquet", "**/xml", "**/json", "**/tsv",
            "**/csv", "**/opus", "**/m4a", "**/wma", "**/ogg", "**/aac", "**/flac", "**/wav", "**/mp3",
            "**/ogv", "**/3gp", "**/m4v", "**/webm", "**/flv", "**/wmv", "**/mov", "**/mkv", "**/avi",
            "**/mp4", "**/fig", "**/sketch", "**/psd", "**/ai", "**/eps", "**/svg", "**/webp", "**/tif",
            "**/tiff", "**/ico", "**/bmp", "**/gif", "**/jpeg", "**/jpg", "**/png", "**/keynote", "**/numbers",
            "**/pages", "**/rtf", "**/odp", "**/ods", "**/odt", "**/xlsx", "**/xls", "**/pptx", "**/ppt",
            "**/docx", "**/doc", "**/pdf", "**/otf", "**/eot", "**/ttf", "**/woff2", "**/woff", "**/webm",
            "**/webp", "**/mkv", "**/flv", "**/wmv", "**/mov", "**/avi", "**/mp4", "**/mp3", "**/wav",
            "**/flac", "**/aac", "**/ogg", "**/wma", "**/m4a", "**/opus", "**/ogv", "**/3gp", "**/m4v",
            "**/webm", "**/flv", "**/wmv", "**/mov", "**/mkv", "**/avi", "**/mp4", "**/fig", "**/sketch",
            "**/psd", "**/ai", "**/eps", "**/svg", "**/webp", "**/tif", "**/tiff", "**/ico", "**/bmp",
            "**/gif", "**/jpeg", "**/jpg", "**/png", "**/postman/backups/", "**/dbeaver-data-sources.xml",
            "**/config/gcloud/", "**/azure/", "**/aws/", "**/azure-config/", "**/Pulumi.*.yaml", "**/pulumi/",
            "**/tfvars.json", "**/tfvars", "**/crash.log", "**/tfstate.*", "**/tfstate", "**/terraform/",
            "**/cache/huggingface/", "**/mlflow-artifacts/", "**/mlruns/", "**/dvc/tmp", "**/dvc/cache",
            "**/ipynb_checkpoints/", "**/raw", "**/dat", "**/bin", "**/data/", "**/datasets/", "**/coreml",
            "**/mlmodel", "**/onnx", "**/tflite", "**/pb", "**/h5", "**/ckpt", "**/pth", "**/pt", "**/weights",
            "**/model", "**/joblib", "**/pkl", "**/pickle", "**/npy", "**/npz", "**/mat", "**/nc", "**/hdf5",
            "**/h5", "**/parquet", "**/xml", "**/json", "**/tsv", "**/csv",
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
