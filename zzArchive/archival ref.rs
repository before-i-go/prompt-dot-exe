use anyhow::{Context, Result};
use chrono::Local;
use clap::{Parser, Subcommand};
use git2::Repository;
use mime_guess::from_path;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use swc_core::{
    common::{errors::Handler, source_map::SourceMap, Globals, Mark, GLOBALS},
    ecma::{
        codegen::{text_writer::JsWriter, Emitter},
        minifier::{
            optimize,
            option::{ExtraOptions, MinifyOptions},
        },
        parser::{lexer::Lexer, Parser as SwcParser, StringInput, Syntax, TsSyntax},
        transforms::typescript::strip,
        visit::FoldWith,
    },
};
use thiserror::Error;
use tracing::{debug, error, info, instrument, warn};
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use walkdir::WalkDir;

#[derive(Error, Debug)]
pub enum ArchiveError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    #[error("Path error: {message}")]
    Path { message: String },
}

#[derive(Parser)]
#[command(name = "ts-compressor")]
#[command(about = "TypeScript compressor and code archiver")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compress TypeScript files to minified JavaScript
    Compress {
        /// Input directory containing TypeScript files
        input_dir: PathBuf,
        /// Output directory for minified JavaScript files
        output_dir: PathBuf,
        /// Log level (trace, debug, info, warn, error)
        #[arg(long, default_value = "info")]
        log_level: String,
    },
    /// Archive code folder contents to timestamped text file
    Archive {
        /// Target folder to archive
        target_folder: PathBuf,
        /// Output directory for archive file (optional, defaults to parent of target)
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
        /// Disable LLM-optimized filtering (enabled by default)
        #[arg(long = "no-llm-optimize")]
        no_llm_optimize: bool,
        /// Custom ignore patterns (glob patterns, can be used multiple times)
        #[arg(long)]
        ignore_pattern: Vec<String>,
        /// Include only specific file extensions (e.g., rs,js,py)
        #[arg(long)]
        include_extensions: Option<String>,
        /// Hide filtering statistics (shown by default)
        #[arg(long = "no-filter-stats")]
        no_filter_stats: bool,
        /// Log level (trace, debug, info, warn, error)
        #[arg(long, default_value = "info")]
        log_level: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Extract log level from command and initialize structured logging
    let log_level = match &cli.command {
        Commands::Compress { log_level, .. } => log_level,
        Commands::Archive { log_level, .. } => log_level,
    };

    init_tracing(log_level)?;

    info!("Starting ts-compressor application");

    let result = match cli.command {
        Commands::Compress {
            input_dir,
            output_dir,
            ..
        } => {
            info!("Starting TypeScript compression");
            compress_typescript(input_dir, output_dir)
        }
        Commands::Archive {
            target_folder,
            output_dir,
            no_llm_optimize,
            ignore_pattern,
            include_extensions,
            no_filter_stats,
            ..
        } => {
            info!("Starting code archiving with intelligent filtering");
            archive_code_folder(
                target_folder,
                output_dir,
                !no_llm_optimize,
                ignore_pattern,
                include_extensions,
                !no_filter_stats,
            )
        }
    };

    match &result {
        Ok(_) => info!("Application completed successfully"),
        Err(e) => error!("Application failed: {}", e),
    }

    result
}

/// Initialize structured logging with configurable levels
fn init_tracing(log_level: &str) -> Result<()> {
    // Create a filter that respects RUST_LOG environment variable
    // Fall back to the provided log level if RUST_LOG is not set
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(log_level))
        .context("Failed to create log filter")?;

    // Create subscriber with structured output
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .finish();

    // Set the subscriber as the global default
    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set global tracing subscriber")?;

    debug!(log_level = log_level, "Tracing initialized successfully");
    Ok(())
}

#[instrument(
    name = "compress_typescript",
    fields(
        input_dir = %input_dir.display(),
        output_dir = %output_dir.display()
    )
)]
fn compress_typescript(input_dir: PathBuf, output_dir: PathBuf) -> Result<()> {
    info!(
        input_dir = %input_dir.display(),
        output_dir = %output_dir.display(),
        "Starting TypeScript compression"
    );

    debug!("Creating output directory");
    fs::create_dir_all(&output_dir)?;

    let mut files_processed = 0;
    let mut files_skipped = 0;

    for entry in WalkDir::new(&input_dir).into_iter().filter_map(|e| e.ok()) {
        if entry
            .path()
            .extension()
            .map_or(false, |e| e == "ts" || e == "tsx")
        {
            debug!(
                file_path = %entry.path().display(),
                "Processing TypeScript file"
            );

            let minified = minify_file(entry.path())?;
            let out_path = output_dir
                .join(entry.path().file_name().unwrap())
                .with_extension("js");
            let mut out_file = File::create(&out_path)?;
            out_file.write_all(minified.as_bytes())?;

            debug!(
                input_file = %entry.path().display(),
                output_file = %out_path.display(),
                original_size = entry.metadata().map(|m| m.len()).unwrap_or(0),
                minified_size = minified.len(),
                "File processed successfully"
            );

            files_processed += 1;
        } else {
            files_skipped += 1;
        }
    }

    info!(
        files_processed = files_processed,
        files_skipped = files_skipped,
        "TypeScript compression completed"
    );

    Ok(())
}

#[instrument(
    name = "archive_code_folder",
    fields(
        target_folder = %target_folder.display(),
        output_dir = ?output_dir
    )
)]
fn archive_code_folder(
    target_folder: PathBuf,
    output_dir: Option<PathBuf>,
    llm_optimize: bool,
    ignore_patterns: Vec<String>,
    include_extensions: Option<String>,
    show_filter_stats: bool,
) -> Result<()> {
    info!(
        target_folder = %target_folder.display(),
        output_dir = ?output_dir,
        "Starting code archiving"
    );

    debug!("Creating code archiver with filtering options");
    let mut archiver = CodeArchiver::new(target_folder, output_dir)?;

    // Configure filtering options
    if llm_optimize {
        archiver.enable_llm_optimization();
        info!("🤖 LLM optimization enabled - filtering build artifacts and dependencies");
    }

    if !ignore_patterns.is_empty() {
        archiver.add_ignore_patterns(ignore_patterns);
        info!("📝 Custom ignore patterns added");
    }

    if let Some(extensions) = include_extensions {
        archiver.set_include_extensions(extensions);
        info!("🎯 File extension filtering enabled");
    }

    if show_filter_stats {
        archiver.enable_filter_statistics();
        info!("📊 Filter statistics enabled");
    }

    debug!("Creating archive file");
    archiver.create_archive()
}

// Original TypeScript minification functionality preserved
fn minify_file(path: &Path) -> Result<String> {
    let cm = std::rc::Rc::new(SourceMap::default());
    let _handler = Handler::with_emitter_writer(Box::new(std::io::stderr()), Some(cm.clone()));

    let fm = cm.load_file(path).context("Failed to load file")?;

    GLOBALS.set(&Globals::new(), || {
        // Parse TS
        let ts_config = TsSyntax {
            tsx: path.extension().map_or(false, |e| e == "tsx"),
            ..Default::default()
        };
        let lexer = Lexer::new(
            Syntax::Typescript(ts_config),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );
        let mut parser = SwcParser::new_from(lexer);
        let mut program = parser
            .parse_program()
            .map_err(|e| anyhow::anyhow!("Parse failed: {:?}", e))?;

        // Strip TS types
        program = program.fold_with(&mut strip(Mark::new(), Mark::new()));

        // Minify with compression and mangling
        let minify_opts = MinifyOptions {
            compress: Some(Default::default()),
            mangle: Some(Default::default()),
            ..Default::default()
        };
        program = optimize(
            program.into(),
            cm.clone(),
            None,
            None,
            &minify_opts,
            &ExtraOptions {
                unresolved_mark: Mark::new(),
                top_level_mark: Mark::new(),
                mangle_name_cache: None,
            },
        );

        // Serialize to code
        let mut buf = Vec::new();
        let writer = JsWriter::new(cm.clone(), "\n", &mut buf, None);
        let mut emitter = Emitter {
            cfg: Default::default(),
            cm: cm.clone(),
            comments: None,
            wr: writer,
        };
        emitter
            .emit_program(&program)
            .context("Failed to emit code")?;

        Ok(String::from_utf8(buf).context("Invalid UTF-8")?)
    })
}

// Code Archiver Implementation following idiomatic Rust patterns
pub struct CodeArchiver {
    target_folder: PathBuf,
    output_dir: PathBuf,
    git_repo: Option<Repository>,
    is_git_repo: bool,
    llm_optimize: bool,
    ignore_patterns: Vec<String>,
    include_extensions: Option<Vec<String>>,
    show_filter_stats: bool,
    filter_stats: FilterStatistics,
}

/// Statistics for file filtering operations
#[derive(Debug, Clone, Default)]
pub struct FilterStatistics {
    pub total_files_found: usize,
    pub files_included: usize,
    pub files_excluded: usize,
    pub excluded_by_extension: usize,
    pub excluded_by_ignore_pattern: usize,
    pub excluded_by_llm_optimization: usize,
    pub excluded_by_git: usize,
    pub total_size_included: usize,
    pub total_size_excluded: usize,
}

impl std::fmt::Debug for CodeArchiver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodeArchiver")
            .field("target_folder", &self.target_folder)
            .field("output_dir", &self.output_dir)
            .field("is_git_repo", &self.is_git_repo)
            .field("git_repo", &self.git_repo.is_some())
            .field("llm_optimize", &self.llm_optimize)
            .field("ignore_patterns", &self.ignore_patterns.len())
            .field(
                "include_extensions",
                &self.include_extensions.as_ref().map(|e| e.len()),
            )
            .finish()
    }
}

impl CodeArchiver {
    /// Get the target folder path
    pub fn target_folder(&self) -> &PathBuf {
        &self.target_folder
    }

    /// Create a new CodeArchiver instance using the Builder pattern (Pattern 3.1)
    pub fn new(target_folder: PathBuf, output_dir: Option<PathBuf>) -> Result<Self, ArchiveError> {
        // Validate target folder exists (Pattern 12.2 - Bounds checking)
        if !target_folder.is_dir() {
            return Err(ArchiveError::Path {
                message: format!("{:?} is not a directory", target_folder),
            });
        }

        // Default output directory to parent of target folder
        let output_dir = output_dir.unwrap_or_else(|| {
            target_folder
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        });

        // Try to open as git repository (Pattern 2.6 - Result wrapping)
        let (git_repo, is_git_repo) = match Repository::open(&target_folder) {
            Ok(repo) => (Some(repo), true),
            Err(_) => (None, false),
        };

        Ok(Self {
            target_folder,
            output_dir,
            git_repo,
            is_git_repo,
            llm_optimize: false,
            ignore_patterns: Vec::new(),
            include_extensions: None,
            show_filter_stats: false,
            filter_stats: FilterStatistics::default(),
        })
    }

    /// Enable LLM-optimized filtering (excludes build artifacts, dependencies, binaries)
    pub fn enable_llm_optimization(&mut self) {
        self.llm_optimize = true;
    }

    /// Add custom ignore patterns
    pub fn add_ignore_patterns(&mut self, patterns: Vec<String>) {
        self.ignore_patterns.extend(patterns);
    }

    /// Set file extensions to include (comma-separated)
    pub fn set_include_extensions(&mut self, extensions: String) {
        self.include_extensions = Some(
            extensions
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .collect(),
        );
    }

    /// Enable filter statistics collection
    pub fn enable_filter_statistics(&mut self) {
        self.show_filter_stats = true;
    }

    /// Get LLM-optimized ignore patterns for cleaner training data
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
    fn get_llm_ignore_patterns(&self) -> Vec<&str> {
        vec![
            // Build artifacts and outputs
            "target/",
            "build/",
            "dist/",
            "out/",
            "bin/",
            "obj/",
            "output/",
            "release/",
            "debug/",
            "*.exe",
            "*.dll",
            "*.so",
            "*.dylib",
            "*.a",
            "*.lib",
            "*.pdb",
            "*.ilk",
            "*.exp",
            "*.map",
            // Dependencies and package managers
            "node_modules/",
            "vendor/",
            "deps/",
            "packages/",
            "bower_components/",
            ".pnp/",
            ".yarn/",
            "venv/",
            "env/",
            ".venv/",
            ".env/",
            "virtualenv/",
            "site-packages/",
            "pip-log.txt",
            "pip-delete-this-directory.txt",
            // Cache and temporary files
            ".cache/",
            "tmp/",
            "temp/",
            ".tmp/",
            ".temp/",
            "*.tmp",
            "*.temp",
            "*.swp",
            "*.swo",
            "*~",
            "*.bak",
            "*.backup",
            "*.orig",
            "*.rej",
            ".#*",
            "#*#",
            // IDE and editor files
            ".vscode/",
            ".idea/",
            "*.iml",
            ".project",
            ".classpath",
            ".settings/",
            "*.sublime-*",
            ".vs/",
            ".vscode-test/",
            "*.code-workspace",
            ".history/",
            ".ionide/",
            // JetBrains
            ".idea/",
            "*.iws",
            "out/",
            // Eclipse
            ".metadata/",
            ".recommenders/",
            // Vim
            "[._]*.s[a-v][a-z]",
            "[._]*.sw[a-p]",
            "[._]s[a-rt-v][a-z]",
            "[._]ss[a-gi-z]",
            "[._]sw[a-p]",
            // Emacs
            "*~",
            "\\#*\\#",
            "/.emacs.desktop",
            "/.emacs.desktop.lock",
            // OS generated files
            ".DS_Store",
            ".DS_Store?",
            "._*",
            ".Spotlight-V100",
            ".Trashes",
            "ehthumbs.db",
            "Thumbs.db",
            "desktop.ini",
            "*.lnk",
            "$RECYCLE.BIN/",
            // Version control (beyond .git)
            ".git/",
            ".hg/",
            ".svn/",
            ".bzr/",
            ".fossil-settings/",
            // Logs and databases
            "*.log",
            "*.db",
            "*.sqlite",
            "*.sqlite3",
            "logs/",
            "log/",
            "*.ldf",
            "*.mdf",
            "*.ndf",
            // Environment and configuration files (may contain secrets)
            ".env",
            ".env.local",
            ".env.development",
            ".env.test",
            ".env.production",
            ".env.staging",
            "*.env",
            "config.json",
            "secrets.json",
            "*.key",
            "*.pem",
            "*.crt",
            "*.cer",
            "*.p12",
            "*.pfx",
            "*.jks",
            "*.keystore",
            // Documentation that's not code
            "*.pdf",
            "*.doc",
            "*.docx",
            "*.ppt",
            "*.pptx",
            "*.xls",
            "*.xlsx",
            "*.odt",
            "*.ods",
            "*.odp",
            "*.rtf",
            "*.pages",
            "*.numbers",
            "*.keynote",
            // Media files
            "*.png",
            "*.jpg",
            "*.jpeg",
            "*.gif",
            "*.bmp",
            "*.ico",
            "*.tiff",
            "*.tif",
            "*.webp",
            "*.svg",
            "*.eps",
            "*.ai",
            "*.psd",
            "*.sketch",
            "*.fig",
            "*.mp4",
            "*.avi",
            "*.mkv",
            "*.mov",
            "*.wmv",
            "*.flv",
            "*.webm",
            "*.m4v",
            "*.3gp",
            "*.ogv",
            "*.mp3",
            "*.wav",
            "*.flac",
            "*.aac",
            "*.ogg",
            "*.wma",
            "*.m4a",
            "*.opus",
            // Archives
            "*.zip",
            "*.tar",
            "*.tar.gz",
            "*.tgz",
            "*.rar",
            "*.7z",
            "*.bz2",
            "*.xz",
            "*.lzma",
            "*.gz",
            "*.Z",
            "*.deb",
            "*.rpm",
            "*.msi",
            "*.dmg",
            "*.pkg",
            "*.app",
            // Test coverage and reports
            "coverage/",
            "test-results/",
            "htmlcov/",
            ".nyc_output/",
            ".coverage",
            "*.cover",
            "*.py,cover",
            ".hypothesis/",
            ".pytest_cache/",
            "nosetests.xml",
            "coverage.xml",
            "*.lcov",
            "lcov.info",
            // Language-specific compiled/generated files
            "*.pyc",
            "*.pyo",
            "*.pyd",
            "__pycache__/",
            "*.class",
            "*.jar",
            "*.war",
            "*.ear",
            "*.nar",
            "*.o",
            "*.obj",
            "*.lib",
            "*.a",
            "*.hi",
            "*.dyn_hi",
            "*.dyn_o",
            "*.beam",
            "*.native",
            "*.byte",
            "*.cmi",
            "*.cmo",
            "*.cmx",
            "*.cmxa",
            "*.cma",
            "*.cmxs",
            // Language-specific build directories
            ".stack-work/",
            ".cabal-sandbox/",
            "cabal.sandbox.config",
            "dist/",
            "dist-newstyle/",
            ".gradle/",
            "build/",
            "gradlew",
            "gradlew.bat",
            "cmake-build-*/",
            "CMakeFiles/",
            "CMakeCache.txt",
            "cmake_install.cmake",
            "install_manifest.txt",
            "Makefile",
            // Lock files (usually generated)
            "package-lock.json",
            "yarn.lock",
            "pnpm-lock.yaml",
            "Cargo.lock",
            "Pipfile.lock",
            "composer.lock",
            "Gemfile.lock",
            "poetry.lock",
            "mix.lock",
            "pubspec.lock",
            "stack.yaml.lock",
            "flake.lock",
            // Cloud and deployment
            ".terraform/",
            "*.tfstate",
            "*.tfstate.*",
            "*.tfplan",
            "*.tfvars",
            ".pulumi/",
            ".serverless/",
            ".vercel/",
            ".netlify/",
            ".next/",
            ".nuxt/",
            ".output/",
            ".firebase/",
            ".gcloud/",
            ".aws/",
            "cdk.out/",
            // Docker
            ".dockerignore",
            "Dockerfile.*",
            ".docker/",
            // Mobile development
            "*.ipa",
            "*.apk",
            "*.aab",
            "*.app",
            "*.dSYM/",
            "*.xcarchive/",
            "*.xcworkspace/",
            "*.xcodeproj/",
            "DerivedData/",
            "build/",
            "*.hmap",
            "*.ipa",
            "*.xcuserstate",
            "project.xcworkspace",
            "xcuserdata/",
            // Unity
            "/[Ll]ibrary/",
            "/[Tt]emp/",
            "/[Oo]bj/",
            "/[Bb]uild/",
            "/[Bb]uilds/",
            "/[Ll]ogs/",
            "/[Mm]emoryCaptures/",
            "/[Uu]serSettings/",
            "*.tmp",
            "*.user",
            "*.userprefs",
            "*.pidb",
            "*.booproj",
            "*.svd",
            "*.pdb",
            "*.mdb",
            "*.opendb",
            "*.VC.db",
            // Game development
            "*.blend1",
            "*.blend2",
            "*.fbx",
            "*.max",
            "*.maya",
            "*.mb",
            "*.ma",
            "*.3ds",
            "*.dae",
            "*.obj",
            "*.mtl",
            "*.dds",
            "*.tga",
            "*.exr",
            "*.hdr",
            // Fonts
            "*.ttf",
            "*.otf",
            "*.woff",
            "*.woff2",
            "*.eot",
            // Data files that are typically large/binary
            "*.csv",
            "*.tsv",
            "*.json",
            "*.xml",
            "*.parquet",
            "*.h5",
            "*.hdf5",
            "*.nc",
            "*.mat",
            "*.npz",
            "*.npy",
            "*.pickle",
            "*.pkl",
            "*.joblib",
            "*.model",
            "*.weights",
            "*.pt",
            "*.pth",
            "*.ckpt",
            "*.h5",
            "*.pb",
            "*.tflite",
            "*.onnx",
            "*.mlmodel",
            "*.coreml",
            "datasets/",
            "data/",
            "*.bin",
            "*.dat",
            "*.raw",
        ]
    }

    /// Get LLM optimization patterns by category
    ///
    /// Returns patterns grouped by category for more granular control
    /// over what gets excluded from LLM training data.
    fn get_llm_patterns_by_category(&self) -> std::collections::HashMap<&str, Vec<&str>> {
        let mut categories = std::collections::HashMap::new();

        categories.insert(
            "build_artifacts",
            vec![
                "target/", "build/", "dist/", "out/", "bin/", "obj/", "output/", "release/",
                "debug/", "*.exe", "*.dll", "*.so", "*.dylib", "*.a", "*.lib", "*.pdb", "*.ilk",
                "*.exp", "*.map",
            ],
        );

        categories.insert(
            "dependencies",
            vec![
                "node_modules/",
                "vendor/",
                "deps/",
                "packages/",
                "bower_components/",
                ".pnp/",
                ".yarn/",
                "venv/",
                "env/",
                ".venv/",
                ".env/",
                "virtualenv/",
                "site-packages/",
                "pip-log.txt",
                "pip-delete-this-directory.txt",
            ],
        );

        categories.insert(
            "cache_temp",
            vec![
                ".cache/", "tmp/", "temp/", ".tmp/", ".temp/", "*.tmp", "*.temp", "*.swp", "*.swo",
                "*~", "*.bak", "*.backup", "*.orig", "*.rej", ".#*", "#*#",
            ],
        );

        categories.insert(
            "ide_editor",
            vec![
                ".vscode/",
                ".idea/",
                "*.iml",
                ".project",
                ".classpath",
                ".settings/",
                "*.sublime-*",
                ".vs/",
                ".vscode-test/",
                "*.code-workspace",
                ".history/",
                ".ionide/",
                "*.iws",
                ".metadata/",
                ".recommenders/",
            ],
        );

        categories.insert(
            "os_generated",
            vec![
                ".DS_Store",
                ".DS_Store?",
                "._*",
                ".Spotlight-V100",
                ".Trashes",
                "ehthumbs.db",
                "Thumbs.db",
                "desktop.ini",
                "*.lnk",
                "$RECYCLE.BIN/",
            ],
        );

        categories.insert(
            "secrets_config",
            vec![
                ".env",
                ".env.local",
                ".env.development",
                ".env.test",
                ".env.production",
                ".env.staging",
                "*.env",
                "config.json",
                "secrets.json",
                "*.key",
                "*.pem",
                "*.crt",
                "*.cer",
                "*.p12",
                "*.pfx",
                "*.jks",
                "*.keystore",
            ],
        );

        categories.insert(
            "media_files",
            vec![
                "*.png", "*.jpg", "*.jpeg", "*.gif", "*.bmp", "*.ico", "*.tiff", "*.tif", "*.webp",
                "*.svg", "*.eps", "*.ai", "*.psd", "*.sketch", "*.fig", "*.mp4", "*.avi", "*.mkv",
                "*.mov", "*.wmv", "*.flv", "*.webm", "*.mp3", "*.wav", "*.flac", "*.aac", "*.ogg",
                "*.wma", "*.m4a", "*.opus",
            ],
        );

        categories.insert(
            "data_models",
            vec![
                "*.csv",
                "*.tsv",
                "*.parquet",
                "*.h5",
                "*.hdf5",
                "*.nc",
                "*.mat",
                "*.npz",
                "*.npy",
                "*.pickle",
                "*.pkl",
                "*.joblib",
                "*.model",
                "*.weights",
                "*.pt",
                "*.pth",
                "*.ckpt",
                "*.pb",
                "*.tflite",
                "*.onnx",
                "*.mlmodel",
                "*.coreml",
                "datasets/",
                "data/",
                "*.bin",
                "*.dat",
                "*.raw",
            ],
        );

        categories
    }

    /// Get LLM optimization level configurations
    ///
    /// Returns different levels of optimization for LLM training data preparation:
    /// - basic: Essential exclusions (build artifacts, dependencies, cache)
    /// - standard: Basic + IDE files, OS files, logs
    /// - aggressive: Standard + media files, documentation, data files
    /// - comprehensive: All available patterns
    fn get_llm_optimization_levels(&self) -> std::collections::HashMap<&str, Vec<&str>> {
        let categories = self.get_llm_patterns_by_category();
        let mut levels = std::collections::HashMap::new();

        // Basic level - essential exclusions
        let mut basic = Vec::new();
        basic.extend(categories.get("build_artifacts").unwrap_or(&Vec::new()));
        basic.extend(categories.get("dependencies").unwrap_or(&Vec::new()));
        basic.extend(categories.get("cache_temp").unwrap_or(&Vec::new()));
        levels.insert("basic", basic);

        // Standard level - basic + common development files
        let mut standard = levels.get("basic").unwrap().clone();
        standard.extend(categories.get("ide_editor").unwrap_or(&Vec::new()));
        standard.extend(categories.get("os_generated").unwrap_or(&Vec::new()));
        standard.extend(vec![
            "*.log",
            "*.db",
            "*.sqlite",
            "*.sqlite3",
            "logs/",
            "log/",
        ]);
        levels.insert("standard", standard);

        // Aggressive level - standard + media and documentation
        let mut aggressive = levels.get("standard").unwrap().clone();
        aggressive.extend(categories.get("media_files").unwrap_or(&Vec::new()));
        aggressive.extend(categories.get("secrets_config").unwrap_or(&Vec::new()));
        aggressive.extend(vec!["*.pdf", "*.doc", "*.docx", "*.ppt", "*.pptx"]);
        levels.insert("aggressive", aggressive);

        // Comprehensive level - all patterns
        levels.insert("comprehensive", self.get_llm_ignore_patterns());

        levels
    }

    /// Apply LLM optimization level
    ///
    /// Sets the LLM optimization to use a specific level of filtering.
    /// Available levels: basic, standard, aggressive, comprehensive
    pub fn set_llm_optimization_level(&mut self, level: &str) {
        if let Some(_patterns) = self.get_llm_optimization_levels().get(level) {
            self.llm_optimize = true;
            // Store the level for later use in filtering
            // Note: This would require adding a field to store the current level
            // For now, we'll document the intended behavior
        }
    }

    /// Check if file should be included based on filtering rules
    fn should_include_file(&mut self, file_path: &Path) -> bool {
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let file_path_str = file_path.to_string_lossy();

        self.filter_stats.total_files_found += 1;

        // Check extension filtering first
        if let Some(ref allowed_extensions) = self.include_extensions {
            if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                if !allowed_extensions.contains(&ext.to_lowercase()) {
                    self.filter_stats.excluded_by_extension += 1;
                    return false;
                }
            } else {
                // No extension, exclude if extensions are specified
                self.filter_stats.excluded_by_extension += 1;
                return false;
            }
        }

        // Check LLM optimization patterns
        if self.llm_optimize {
            for pattern in self.get_llm_ignore_patterns() {
                if Self::matches_glob_pattern(&file_path_str, pattern)
                    || Self::matches_glob_pattern(file_name, pattern)
                {
                    self.filter_stats.excluded_by_llm_optimization += 1;
                    return false;
                }
            }
        }

        // Check custom ignore patterns
        for pattern in &self.ignore_patterns {
            if Self::matches_glob_pattern(&file_path_str, pattern)
                || Self::matches_glob_pattern(file_name, pattern)
            {
                self.filter_stats.excluded_by_ignore_pattern += 1;
                return false;
            }
        }

        self.filter_stats.files_included += 1;
        true
    }

    /// Simple glob pattern matching
    fn matches_glob_pattern(text: &str, pattern: &str) -> bool {
        if pattern.ends_with('/') {
            // Directory pattern
            let dir_pattern = &pattern[..pattern.len() - 1];
            return text.contains(dir_pattern);
        }

        if pattern.contains('*') {
            // Wildcard pattern
            if pattern.starts_with('*') && pattern.len() > 1 {
                return text.ends_with(&pattern[1..]);
            }
            if pattern.ends_with('*') && pattern.len() > 1 {
                return text.starts_with(&pattern[..pattern.len() - 1]);
            }
            return text.contains(&pattern.replace('*', ""));
        }

        // Exact match
        text == pattern || text.contains(pattern)
    }

    /// Display filtering statistics with enhanced LLM optimization details
    fn display_filter_stats(&self) {
        if !self.show_filter_stats {
            return;
        }

        let stats = &self.filter_stats;
        println!("\n📊 File Filtering Statistics:");
        println!("   Total files found: {}", stats.total_files_found);
        println!("   Files included: {} 🟢", stats.files_included);
        println!("   Files excluded: {} 🔴", stats.files_excluded);

        if stats.excluded_by_extension > 0 {
            println!(
                "     └─ By extension filter: {}",
                stats.excluded_by_extension
            );
        }
        if stats.excluded_by_llm_optimization > 0 {
            println!(
                "     └─ By LLM optimization: {} 🤖",
                stats.excluded_by_llm_optimization
            );

            // Show LLM optimization benefits
            if self.llm_optimize {
                println!("        ✨ LLM optimization excluded:");
                println!("           • Build artifacts and compiled files");
                println!("           • Dependencies and package manager files");
                println!("           • Cache and temporary files");
                println!("           • IDE and editor configuration");
                println!("           • Binary media files");
                println!("           • Environment and secret files");
                println!("           • Large data files and ML models");
                println!("        📚 This creates cleaner training data focused on source code");
            }
        }
        if stats.excluded_by_ignore_pattern > 0 {
            println!(
                "     └─ By ignore patterns: {}",
                stats.excluded_by_ignore_pattern
            );
        }
        if stats.excluded_by_git > 0 {
            println!("     └─ By Git rules: {}", stats.excluded_by_git);
        }

        let inclusion_rate = if stats.total_files_found > 0 {
            (stats.files_included as f64 / stats.total_files_found as f64) * 100.0
        } else {
            0.0
        };
        println!("   Inclusion rate: {:.1}% 📈", inclusion_rate);

        if stats.total_size_included > 0 {
            println!(
                "   Total size included: {} bytes 💾",
                stats.total_size_included
            );
        }

        // Show LLM optimization recommendation
        if !self.llm_optimize && stats.files_excluded > 0 {
            println!("\n💡 Tip: Use --llm-optimize flag to automatically exclude");
            println!("   build artifacts, dependencies, and binary files for");
            println!("   cleaner LLM training data preparation.");
        }

        println!();
    }

    /// Create the archive file (Pattern 4.1 - RAII pattern)
    pub fn create_archive(&mut self) -> Result<()> {
        let timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();
        let folder_name = self
            .target_folder
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let output_file = self
            .output_dir
            .join(format!("{}-{}.txt", folder_name, timestamp));

        // Ensure output directory exists
        fs::create_dir_all(&self.output_dir)?;

        let mut file = File::create(&output_file)?;

        // Write header information
        self.write_header(&mut file)?;

        // Write directory structure
        self.write_directory_structure(&mut file)?;

        // Write file contents
        self.write_file_contents(&mut file)?;

        // Display filtering statistics
        self.display_filter_stats();

        println!("Archive created: {:?}", output_file);
        Ok(())
    }

    /// Write archive header (Pattern 9.5 - Display implementation)
    fn write_header(&self, file: &mut File) -> Result<()> {
        let repo_status = if self.is_git_repo {
            "Git repository detected. Will respect .gitignore rules."
        } else {
            "Not a git repository or git not available. Will process all files."
        };

        writeln!(file, "{}", repo_status)?;
        writeln!(file)?;
        Ok(())
    }

    /// Write directory structure using tree-like output (Pattern 15.1 - Custom iterators)
    fn write_directory_structure(&self, file: &mut File) -> Result<()> {
        writeln!(file, "Directory structure:")?;

        if self.is_git_repo {
            self.write_git_tree_structure(file)?;
        } else {
            self.write_regular_tree_structure(file)?;
        }

        writeln!(file)?;
        Ok(())
    }

    /// Write git-aware directory structure (Pattern 31.1 - Option combinators)
    fn write_git_tree_structure(&self, file: &mut File) -> Result<()> {
        let repo = self.git_repo.as_ref().unwrap();
        let workdir = repo.workdir().unwrap_or(&self.target_folder);

        // Get relative path from git root to target folder
        let rel_path = self
            .target_folder
            .strip_prefix(workdir)
            .unwrap_or(Path::new("."));

        let mut files = self.get_git_tracked_files(rel_path)?;
        files.sort();

        for file_path in files {
            let tree_line = self.format_tree_line(&file_path);
            writeln!(file, "{}", tree_line)?;
        }

        Ok(())
    }

    /// Get git tracked and untracked files (Pattern 15.2 - Collection transformations)
    fn get_git_tracked_files(&self, rel_path: &Path) -> Result<Vec<PathBuf>> {
        let repo = self.git_repo.as_ref().unwrap();
        let mut files = HashSet::new();

        // Get tracked files
        let index = repo.index()?;
        for entry in index.iter() {
            let path = PathBuf::from(std::str::from_utf8(&entry.path)?);
            if path.starts_with(rel_path) {
                files.insert(path);
            }
        }

        // Get untracked files (respecting .gitignore)
        let mut status_opts = git2::StatusOptions::new();
        status_opts.include_untracked(true);
        status_opts.include_ignored(false);

        let statuses = repo.statuses(Some(&mut status_opts))?;
        for entry in statuses.iter() {
            if let Some(path_str) = entry.path() {
                let path = PathBuf::from(path_str);
                if path.starts_with(rel_path) {
                    files.insert(path);
                }
            }
        }

        Ok(files.into_iter().collect())
    }

    /// Write regular directory structure using walkdir (Pattern 15.9 - Collection views)
    fn write_regular_tree_structure(&self, file: &mut File) -> Result<()> {
        // Try to use system tree command first, fallback to custom implementation
        if let Ok(output) = Command::new("tree").arg(&self.target_folder).output() {
            if output.status.success() {
                file.write_all(&output.stdout)?;
                return Ok(());
            }
        }

        // Fallback: custom tree implementation
        for entry in WalkDir::new(&self.target_folder) {
            let entry = entry?;
            let depth = entry.depth();
            let name = entry.file_name().to_string_lossy();
            let prefix = "│   ".repeat(depth.saturating_sub(1));
            let connector = if depth > 0 { "├── " } else { "" };
            writeln!(file, "{}{}{}", prefix, connector, name)?;
        }

        Ok(())
    }

    /// Format file path as tree line (Pattern 9.1 - Into/From conversions)
    fn format_tree_line(&self, path: &Path) -> String {
        let components: Vec<_> = path.components().collect();
        let depth = components.len().saturating_sub(1);
        let prefix = "│   ".repeat(depth);
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        format!("{}├── {}", prefix, name)
    }

    /// Write file contents (Pattern 2.3 - Question mark operator chaining)
    fn write_file_contents(&mut self, file: &mut File) -> Result<()> {
        writeln!(file, "Processing files...")?;

        if self.llm_optimize {
            writeln!(
                file,
                "🤖 LLM optimization enabled - excluding build artifacts and dependencies"
            )?;
        }
        if !self.ignore_patterns.is_empty() {
            writeln!(
                file,
                "📝 Custom ignore patterns: {:?}",
                self.ignore_patterns
            )?;
        }
        if let Some(ref extensions) = self.include_extensions {
            writeln!(file, "🎯 Including only extensions: {:?}", extensions)?;
        }
        writeln!(file)?;

        if self.is_git_repo {
            self.write_git_file_contents(file)
        } else {
            self.write_all_file_contents(file)
        }
    }

    /// Write git-tracked file contents (Pattern 31.2 - Collection operations)
    fn write_git_file_contents(&mut self, file: &mut File) -> Result<()> {
        // Collect file paths first to avoid borrow conflicts
        let file_paths = {
            let repo = self.git_repo.as_ref().unwrap();
            let workdir = repo.workdir().unwrap_or(&self.target_folder);
            let rel_path = self
                .target_folder
                .strip_prefix(workdir)
                .unwrap_or(Path::new("."));

            let files = self.get_git_tracked_files(rel_path)?;

            files
                .into_iter()
                .map(|file_path| workdir.join(&file_path))
                .filter(|full_path| full_path.is_file())
                .collect::<Vec<_>>()
        };

        // Now write the files without holding any immutable borrows
        for full_path in file_paths {
            self.write_single_file_content(file, &full_path)?;
        }

        Ok(())
    }

    /// Write all file contents (Pattern 15.1 - Custom iterators)
    fn write_all_file_contents(&mut self, file: &mut File) -> Result<()> {
        for entry in WalkDir::new(&self.target_folder) {
            let entry = entry?;
            if entry.file_type().is_file() {
                self.write_single_file_content(file, entry.path())?;
            }
        }
        Ok(())
    }

    /// Write content of a single file (Pattern 31.3 - Early returns and guards)
    fn write_single_file_content(
        &mut self,
        output_file: &mut File,
        file_path: &Path,
    ) -> Result<()> {
        // Check if file should be included based on filtering rules
        if !self.should_include_file(file_path) {
            self.filter_stats.files_excluded += 1;
            return Ok(()); // Skip this file
        }

        writeln!(output_file, "Absolute path: {}", file_path.display())?;

        // Check if file is text or binary (Pattern 31.4 - Default values)
        let mime_type = from_path(file_path).first_or_octet_stream();
        let is_text = mime_type.type_() == mime::TEXT
            || mime_type == mime::APPLICATION_JSON
            || self.is_likely_text_file(file_path);

        if is_text {
            writeln!(output_file, "<text starts>")?;

            // Read and write file content (Pattern 4.1 - RAII pattern)
            match fs::read_to_string(file_path) {
                Ok(content) => {
                    self.filter_stats.total_size_included += content.len();
                    output_file.write_all(content.as_bytes())?;
                }
                Err(_) => {
                    writeln!(output_file, "Error reading file content")?;
                }
            }

            writeln!(output_file, "<text ends>")?;
        } else {
            writeln!(output_file, "Binary file, content not included.")?;
        }

        writeln!(output_file)?;
        Ok(())
    }

    /// Check if file is likely text based on extension (Pattern 31.8 - Pattern matching)
    fn is_likely_text_file(&self, path: &Path) -> bool {
        let text_extensions = [
            "rs",
            "toml",
            "md",
            "txt",
            "json",
            "yaml",
            "yml",
            "js",
            "ts",
            "tsx",
            "jsx",
            "html",
            "css",
            "scss",
            "py",
            "rb",
            "go",
            "java",
            "c",
            "cpp",
            "h",
            "hpp",
            "sh",
            "bash",
            "zsh",
            "fish",
            "ps1",
            "bat",
            "cmd",
            "xml",
            "svg",
            "gitignore",
            "dockerfile",
            "makefile",
        ];

        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| text_extensions.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
    }
}
