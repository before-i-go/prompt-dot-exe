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

mod compression;

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
        /// Enable LLM-optimized filtering (excludes build artifacts, dependencies, binaries)
        #[arg(long)]
        llm_optimize: bool,
        /// Custom ignore patterns (glob patterns, can be used multiple times)
        #[arg(long)]
        ignore_pattern: Vec<String>,
        /// Include only specific file extensions (e.g., rs,js,py)
        #[arg(long)]
        include_extensions: Option<String>,
        /// Show filtering statistics
        #[arg(long)]
        show_filter_stats: bool,
        /// Log level (trace, debug, info, warn, error)
        #[arg(long, default_value = "info")]
        log_level: String,
    },
    /// Universal code compression with pattern analysis and dictionary building
    UniversalCompress {
        /// Target folder to compress
        target_folder: PathBuf,
        /// Output directory for compressed file (optional, defaults to parent of target)
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
        /// Minimum pattern length for analysis (default: 4)
        #[arg(long, default_value = "4")]
        min_pattern_length: usize,
        /// Minimum frequency threshold for patterns (default: 3)
        #[arg(long, default_value = "3")]
        min_frequency_threshold: usize,
        /// Enable final zstd compression
        #[arg(long)]
        enable_zstd: bool,
        /// Database path for pattern persistence (default: compression_patterns.db)
        #[arg(long, default_value = "compression_patterns.db")]
        database_path: PathBuf,
        /// Maximum number of threads for parallel processing (default: auto-detect)
        #[arg(long)]
        max_threads: Option<usize>,
        /// Chunk size for parallel processing in KB (default: 64)
        #[arg(long, default_value = "64")]
        chunk_size_kb: usize,
        /// Channel buffer size for parallel processing (default: 100)
        #[arg(long, default_value = "100")]
        channel_buffer_size: usize,
        /// Memory map threshold in MB for large files (default: 1)
        #[arg(long, default_value = "1")]
        memory_map_threshold_mb: usize,
        /// Log level (trace, debug, info, warn, error)
        #[arg(long, default_value = "info")]
        log_level: String,
    },
    /// Resume compression from a previous checkpoint
    Resume {
        /// Database path containing the checkpoint (default: compression_patterns.db)
        #[arg(long, default_value = "compression_patterns.db")]
        database_path: PathBuf,
        /// Output directory for compressed file (optional, defaults to parent of target)
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
        /// Maximum number of threads for parallel processing (default: auto-detect)
        #[arg(long)]
        max_threads: Option<usize>,
        /// Log level (trace, debug, info, warn, error)
        #[arg(long, default_value = "info")]
        log_level: String,
    },
    /// Manage compression checkpoints
    Checkpoint {
        #[command(subcommand)]
        action: CheckpointAction,
    },
}

#[derive(Debug, Subcommand)]
enum CheckpointAction {
    /// List available checkpoints
    List {
        /// Database path to check for checkpoints (default: compression_patterns.db)
        #[arg(long, default_value = "compression_patterns.db")]
        database_path: PathBuf,
    },
    /// Show details of a specific checkpoint
    Show {
        /// Database path containing the checkpoint (default: compression_patterns.db)
        #[arg(long, default_value = "compression_patterns.db")]
        database_path: PathBuf,
        /// Checkpoint ID to show (optional, shows latest if not specified)
        #[arg(long)]
        checkpoint_id: Option<i64>,
    },
    /// Delete a checkpoint
    Delete {
        /// Database path containing the checkpoint (default: compression_patterns.db)
        #[arg(long, default_value = "compression_patterns.db")]
        database_path: PathBuf,
        /// Checkpoint ID to delete
        #[arg(long)]
        checkpoint_id: i64,
    },
    /// Clean old checkpoints (keep only the latest N)
    Clean {
        /// Database path containing checkpoints (default: compression_patterns.db)
        #[arg(long, default_value = "compression_patterns.db")]
        database_path: PathBuf,
        /// Number of checkpoints to keep (default: 5)
        #[arg(long, default_value = "5")]
        keep_count: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Extract log level from command and initialize structured logging
    let log_level = match &cli.command {
        Commands::Compress { log_level, .. } => log_level,
        Commands::Archive { log_level, .. } => log_level,
        Commands::UniversalCompress { log_level, .. } => log_level,
        Commands::Resume { log_level, .. } => log_level,
        Commands::Checkpoint { .. } => "info", // Default for checkpoint commands
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
            llm_optimize,
            ignore_pattern,
            include_extensions,
            show_filter_stats,
            ..
        } => {
            info!("Starting code archiving with intelligent filtering");
            archive_code_folder(
                target_folder,
                output_dir,
                llm_optimize,
                ignore_pattern,
                include_extensions,
                show_filter_stats,
            )
        }
        Commands::UniversalCompress {
            target_folder,
            output_dir,
            min_pattern_length,
            min_frequency_threshold,
            enable_zstd,
            database_path,
            max_threads,
            chunk_size_kb,
            channel_buffer_size,
            memory_map_threshold_mb,
            ..
        } => {
            info!("Starting universal compression with enhanced configuration");
            universal_compress_enhanced(
                target_folder,
                output_dir,
                min_pattern_length,
                min_frequency_threshold,
                enable_zstd,
                database_path,
                max_threads,
                chunk_size_kb,
                channel_buffer_size,
                memory_map_threshold_mb,
            )
        }
        Commands::Resume {
            database_path,
            output_dir,
            max_threads,
            ..
        } => {
            info!("Resuming compression from checkpoint");
            resume_compression(database_path, output_dir, max_threads)
        }
        Commands::Checkpoint { action } => {
            info!("Managing compression checkpoints");
            handle_checkpoint_command(action)
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

#[instrument(
    name = "universal_compress",
    fields(
        target_folder = %target_folder.display(),
        min_pattern_length = min_pattern_length,
        min_frequency_threshold = min_frequency_threshold,
        enable_zstd = enable_zstd
    )
)]
fn universal_compress(
    target_folder: PathBuf,
    output_dir: Option<PathBuf>,
    min_pattern_length: usize,
    min_frequency_threshold: usize,
    enable_zstd: bool,
) -> Result<()> {
    use compression::{CompressionConfig, UniversalCompressor};
    use std::time::Instant;

    let start_time = Instant::now();

    info!(
        target_folder = %target_folder.display(),
        output_dir = ?output_dir,
        min_pattern_length = min_pattern_length,
        min_frequency_threshold = min_frequency_threshold,
        enable_zstd = enable_zstd,
        "Starting universal compression pipeline"
    );

    // Build configuration
    info!("Building compression configuration");
    let config = CompressionConfig::builder()
        .min_pattern_length(min_pattern_length)
        .min_frequency_threshold(min_frequency_threshold)
        .enable_zstd_compression(enable_zstd)
        .build()
        .context("Failed to build compression configuration")?;

    debug!(config = ?config, "Configuration built successfully");

    // Create compressor and execute pipeline
    info!("Creating universal compressor");
    let compressor =
        UniversalCompressor::with_config(target_folder.clone(), output_dir.clone(), config)
            .context("Failed to create universal compressor")?;

    // Execute the compression pipeline using typestate pattern
    info!("Starting pipeline phase: Configuration");
    let configured = compressor.configure();

    info!("Starting pipeline phase: Pattern Analysis");
    let analysis_start = Instant::now();
    let analyzed = configured.analyze().context("Failed to analyze patterns")?;
    let analysis_duration = analysis_start.elapsed();
    info!(
        duration_ms = analysis_duration.as_millis(),
        "Pattern analysis completed"
    );

    info!("Starting pipeline phase: Dictionary Building");
    let dict_start = Instant::now();
    let built = analyzed
        .build_dictionary()
        .context("Failed to build dictionary")?;
    let dict_duration = dict_start.elapsed();
    info!(
        duration_ms = dict_duration.as_millis(),
        "Dictionary building completed"
    );

    info!("Starting pipeline phase: Pattern Replacement Preparation");
    let prep_start = Instant::now();
    let mut ready_compressor = built
        .prepare_replacement()
        .context("Failed to prepare pattern replacement")?;
    let prep_duration = prep_start.elapsed();
    info!(
        duration_ms = prep_duration.as_millis(),
        "Pattern replacement preparation completed"
    );

    // Perform compression
    info!("Starting pipeline phase: Compression");
    let compression_start = Instant::now();
    let result = ready_compressor
        .compress()
        .context("Failed to perform compression")?;
    let compression_duration = compression_start.elapsed();
    info!(
        duration_ms = compression_duration.as_millis(),
        "Compression completed"
    );

    // Generate output file with embedded dictionary
    info!("Generating output file");
    let output_start = Instant::now();
    let output_path = generate_output_file(&target_folder, &output_dir, &ready_compressor, &result)
        .context("Failed to generate output file")?;
    let output_duration = output_start.elapsed();
    info!(
        duration_ms = output_duration.as_millis(),
        "Output file generation completed"
    );

    let total_duration = start_time.elapsed();

    info!(
        output_path = %output_path.display(),
        files_processed = result.statistics.total_files_processed,
        dictionary_entries = result.dictionary_size,
        pattern_replacements = result.patterns_replaced,
        total_duration_ms = total_duration.as_millis(),
        analysis_duration_ms = analysis_duration.as_millis(),
        dict_duration_ms = dict_duration.as_millis(),
        prep_duration_ms = prep_duration.as_millis(),
        compression_duration_ms = compression_duration.as_millis(),
        output_duration_ms = output_duration.as_millis(),
        "Universal compression pipeline completed successfully"
    );

    Ok(())
}

fn generate_output_file(
    target_folder: &PathBuf,
    output_dir: &Option<PathBuf>,
    compressor: &compression::UniversalCompressor<compression::compressor::ReadyState>,
    result: &compression::types::CompressionResult,
) -> Result<PathBuf> {
    use chrono::Local;
    use std::fs::File;
    use std::io::Write;

    // Create output directory
    let output_dir = output_dir.as_ref().map(|p| p.clone()).unwrap_or_else(|| {
        target_folder
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf()
    });

    fs::create_dir_all(&output_dir)?;

    // Generate timestamped filename
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let folder_name = target_folder
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let output_path = output_dir.join(format!("{}_{}.txt", folder_name, timestamp));
    let mut file = File::create(&output_path)?;

    // Write header
    writeln!(file, "# Universal Code Compression Output")?;
    writeln!(
        file,
        "# Generated: {}",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    )?;
    writeln!(file, "# Target: {:?}", target_folder)?;
    writeln!(file)?;

    // Write compression statistics
    write_compression_statistics(&mut file, result)?;

    // Write embedded dictionary
    write_embedded_dictionary(&mut file, compressor)?;

    // Write directory structure manifest
    write_directory_manifest(&mut file, target_folder)?;

    // Write compressed content
    write_compressed_content(&mut file, compressor)?;

    Ok(output_path)
}

fn write_compression_statistics(
    file: &mut File,
    result: &compression::types::CompressionResult,
) -> Result<()> {
    writeln!(file, "## Compression Statistics")?;
    writeln!(
        file,
        "Files processed: {}",
        result.statistics.total_files_processed
    )?;
    writeln!(
        file,
        "Original size: {} bytes",
        result.statistics.original_total_size.bytes()
    )?;
    writeln!(
        file,
        "Compressed size: {} bytes",
        result.statistics.compressed_total_size.bytes()
    )?;

    let compression_ratio = if result.statistics.original_total_size.bytes() > 0 {
        (result.statistics.compressed_total_size.bytes() as f64)
            / (result.statistics.original_total_size.bytes() as f64)
    } else {
        0.0
    };

    writeln!(
        file,
        "Compression ratio: {:.2}%",
        (1.0 - compression_ratio) * 100.0
    )?;
    writeln!(file, "Dictionary entries: {}", result.dictionary_size)?;
    writeln!(file, "Pattern replacements: {}", result.patterns_replaced)?;
    writeln!(
        file,
        "Processing time: {:?}",
        result.statistics.processing_time
    )?;
    writeln!(file)?;

    Ok(())
}

fn write_embedded_dictionary(
    file: &mut File,
    compressor: &compression::UniversalCompressor<compression::compressor::ReadyState>,
) -> Result<()> {
    writeln!(file, "## Embedded Dictionary")?;
    writeln!(file, "# Format: DICT:original_pattern=hex_token")?;

    // Get dictionary entries from the compressor
    let dictionary_entries = compressor.get_dictionary_entries();

    if dictionary_entries.is_empty() {
        writeln!(file, "# No dictionary entries found")?;
    } else {
        for (pattern, token) in dictionary_entries {
            writeln!(file, "DICT:{}={}", pattern, token)?;
        }
    }

    writeln!(file)?;
    Ok(())
}

fn write_directory_manifest(file: &mut File, target_folder: &PathBuf) -> Result<()> {
    writeln!(file, "## Directory Structure Manifest")?;

    // Use the same logic as CodeArchiver for consistency
    for entry in WalkDir::new(target_folder) {
        let entry = entry?;
        let relative_path = entry
            .path()
            .strip_prefix(target_folder)
            .unwrap_or(entry.path());

        if entry.file_type().is_file() {
            writeln!(file, "FILE: {}", relative_path.display())?;
        } else if entry.file_type().is_dir() && entry.depth() > 0 {
            writeln!(file, "DIR: {}", relative_path.display())?;
        }
    }

    writeln!(file)?;
    Ok(())
}

fn write_compressed_content(
    file: &mut File,
    compressor: &compression::UniversalCompressor<compression::compressor::ReadyState>,
) -> Result<()> {
    writeln!(file, "## Compressed Content")?;

    // Get compressed files from the compressor
    match compressor.get_compressed_files() {
        Ok(files) => {
            if files.is_empty() {
                writeln!(file, "# No files found to compress")?;
            } else {
                for file_entry in files {
                    writeln!(file, "### File: {}", file_entry.relative_path.display())?;
                    writeln!(
                        file,
                        "Original size: {} bytes",
                        file_entry.original_size.bytes()
                    )?;

                    if let Some(compressed_size) = file_entry.compressed_size {
                        writeln!(file, "Compressed size: {} bytes", compressed_size.bytes())?;
                        let ratio = if file_entry.original_size.bytes() > 0 {
                            (compressed_size.bytes() as f64)
                                / (file_entry.original_size.bytes() as f64)
                        } else {
                            0.0
                        };
                        writeln!(file, "Compression ratio: {:.2}%", (1.0 - ratio) * 100.0)?;
                    }

                    writeln!(file, "Content:")?;
                    if let Some(compressed_content) = &file_entry.compressed_content {
                        writeln!(file, "{}", compressed_content)?;
                    } else {
                        writeln!(file, "{}", file_entry.original_content)?;
                    }
                    writeln!(file, "---")?;
                }
            }
        }
        Err(e) => {
            writeln!(file, "# Error retrieving compressed files: {}", e)?;
        }
    }

    writeln!(file)?;
    Ok(())
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

    /// Get statistics about LLM optimization categories
    ///
    /// Returns detailed statistics about which categories of files were excluded
    /// during LLM-optimized filtering.
    fn get_llm_optimization_stats(&self) -> std::collections::HashMap<String, usize> {
        let mut stats = std::collections::HashMap::new();

        // This would be populated during filtering
        // For now, return the current basic stats
        stats.insert("build_artifacts_excluded".to_string(), 0);
        stats.insert("dependencies_excluded".to_string(), 0);
        stats.insert("cache_temp_excluded".to_string(), 0);
        stats.insert("ide_editor_excluded".to_string(), 0);
        stats.insert("os_generated_excluded".to_string(), 0);
        stats.insert("secrets_config_excluded".to_string(), 0);
        stats.insert("media_files_excluded".to_string(), 0);
        stats.insert("data_models_excluded".to_string(), 0);

        stats
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
/// Enhanced universal compression with full configuration support
#[instrument(
    name = "universal_compress_enhanced",
    fields(
        target_folder = %target_folder.display(),
        min_pattern_length = min_pattern_length,
        min_frequency_threshold = min_frequency_threshold,
        enable_zstd = enable_zstd,
        database_path = %database_path.display(),
        max_threads = ?max_threads,
        chunk_size_kb = chunk_size_kb,
        channel_buffer_size = channel_buffer_size,
        memory_map_threshold_mb = memory_map_threshold_mb
    )
)]
fn universal_compress_enhanced(
    target_folder: PathBuf,
    output_dir: Option<PathBuf>,
    min_pattern_length: usize,
    min_frequency_threshold: usize,
    enable_zstd: bool,
    database_path: PathBuf,
    max_threads: Option<usize>,
    chunk_size_kb: usize,
    channel_buffer_size: usize,
    memory_map_threshold_mb: usize,
) -> Result<()> {
    use compression::{config::ParallelConfig, CompressionConfig};
    use std::time::Instant;

    let _start_time = Instant::now();

    info!(
        target_folder = %target_folder.display(),
        output_dir = ?output_dir,
        min_pattern_length = min_pattern_length,
        min_frequency_threshold = min_frequency_threshold,
        enable_zstd = enable_zstd,
        database_path = %database_path.display(),
        max_threads = ?max_threads,
        chunk_size_kb = chunk_size_kb,
        channel_buffer_size = channel_buffer_size,
        memory_map_threshold_mb = memory_map_threshold_mb,
        "Starting enhanced universal compression pipeline"
    );

    // Validate configuration parameters
    info!("Validating configuration parameters");
    validate_compression_config(
        min_pattern_length,
        min_frequency_threshold,
        max_threads,
        chunk_size_kb,
        channel_buffer_size,
        memory_map_threshold_mb,
    )?;

    // Build parallel configuration
    info!("Building parallel processing configuration");
    let mut parallel_builder = ParallelConfig::builder()
        .chunk_size(chunk_size_kb * 1024) // Convert KB to bytes
        .channel_buffer_size(channel_buffer_size)
        .memory_map_threshold(memory_map_threshold_mb * 1024 * 1024); // Convert MB to bytes

    if let Some(threads) = max_threads {
        parallel_builder = parallel_builder.max_threads(threads);
    }

    let parallel_config = parallel_builder
        .build()
        .context("Failed to build parallel configuration")?;

    debug!(parallel_config = ?parallel_config, "Parallel configuration built successfully");

    // Build main compression configuration
    info!("Building compression configuration");
    let config = CompressionConfig::builder()
        .min_pattern_length(min_pattern_length)
        .min_frequency_threshold(min_frequency_threshold)
        .enable_zstd_compression(enable_zstd)
        .parallel_config(parallel_config)
        .build()
        .context("Failed to build compression configuration")?;

    debug!(config = ?config, "Configuration built successfully");

    // TODO: Implement database-aware compression pipeline
    // For now, fall back to the original implementation
    warn!("Database-aware compression pipeline not yet implemented, falling back to original implementation");

    universal_compress(
        target_folder,
        output_dir,
        min_pattern_length,
        min_frequency_threshold,
        enable_zstd,
    )
}

/// Resume compression from a checkpoint
#[instrument(
    name = "resume_compression",
    fields(
        database_path = %database_path.display(),
        output_dir = ?output_dir,
        max_threads = ?max_threads
    )
)]
fn resume_compression(
    database_path: PathBuf,
    output_dir: Option<PathBuf>,
    max_threads: Option<usize>,
) -> Result<()> {
    info!(
        database_path = %database_path.display(),
        output_dir = ?output_dir,
        max_threads = ?max_threads,
        "Attempting to resume compression from checkpoint"
    );

    // Check if database exists
    if !database_path.exists() {
        return Err(anyhow::anyhow!(
            "Database file not found: {}. Cannot resume compression.",
            database_path.display()
        ));
    }

    // TODO: Implement checkpoint resume functionality
    // This would involve:
    // 1. Loading checkpoint state from database
    // 2. Validating checkpoint integrity
    // 3. Resuming from the saved state
    // 4. Continuing with the compression pipeline

    error!("Resume functionality not yet implemented");
    Err(anyhow::anyhow!(
        "Resume functionality is not yet implemented. Please use the regular compression command."
    ))
}

/// Handle checkpoint management commands
#[instrument(name = "handle_checkpoint_command")]
fn handle_checkpoint_command(action: CheckpointAction) -> Result<()> {
    match action {
        CheckpointAction::List { database_path } => {
            info!(database_path = %database_path.display(), "Listing checkpoints");
            list_checkpoints(database_path)
        }
        CheckpointAction::Show {
            database_path,
            checkpoint_id,
        } => {
            info!(
                database_path = %database_path.display(),
                checkpoint_id = ?checkpoint_id,
                "Showing checkpoint details"
            );
            show_checkpoint(database_path, checkpoint_id)
        }
        CheckpointAction::Delete {
            database_path,
            checkpoint_id,
        } => {
            info!(
                database_path = %database_path.display(),
                checkpoint_id = checkpoint_id,
                "Deleting checkpoint"
            );
            delete_checkpoint(database_path, checkpoint_id)
        }
        CheckpointAction::Clean {
            database_path,
            keep_count,
        } => {
            info!(
                database_path = %database_path.display(),
                keep_count = keep_count,
                "Cleaning old checkpoints"
            );
            clean_checkpoints(database_path, keep_count)
        }
    }
}

/// List available checkpoints
fn list_checkpoints(database_path: PathBuf) -> Result<()> {
    if !database_path.exists() {
        println!("No database found at: {}", database_path.display());
        println!("No checkpoints available.");
        return Ok(());
    }

    // TODO: Implement database checkpoint listing
    // This would involve:
    // 1. Opening the database connection
    // 2. Querying the checkpoints table
    // 3. Formatting and displaying the results

    println!("Checkpoint listing not yet implemented.");
    println!("Database path: {}", database_path.display());

    Ok(())
}

/// Show details of a specific checkpoint
fn show_checkpoint(database_path: PathBuf, checkpoint_id: Option<i64>) -> Result<()> {
    if !database_path.exists() {
        return Err(anyhow::anyhow!(
            "Database file not found: {}",
            database_path.display()
        ));
    }

    // TODO: Implement checkpoint detail display
    // This would involve:
    // 1. Opening the database connection
    // 2. Querying for the specific checkpoint (or latest if none specified)
    // 3. Displaying detailed information about the checkpoint state

    match checkpoint_id {
        Some(id) => println!("Showing checkpoint {} from {}", id, database_path.display()),
        None => println!("Showing latest checkpoint from {}", database_path.display()),
    }

    println!("Checkpoint details not yet implemented.");

    Ok(())
}

/// Delete a specific checkpoint
fn delete_checkpoint(database_path: PathBuf, checkpoint_id: i64) -> Result<()> {
    if !database_path.exists() {
        return Err(anyhow::anyhow!(
            "Database file not found: {}",
            database_path.display()
        ));
    }

    // TODO: Implement checkpoint deletion
    // This would involve:
    // 1. Opening the database connection
    // 2. Verifying the checkpoint exists
    // 3. Deleting the checkpoint record
    // 4. Cleaning up any associated data

    println!(
        "Deleting checkpoint {} from {}",
        checkpoint_id,
        database_path.display()
    );
    println!("Checkpoint deletion not yet implemented.");

    Ok(())
}

/// Clean old checkpoints, keeping only the latest N
fn clean_checkpoints(database_path: PathBuf, keep_count: usize) -> Result<()> {
    if !database_path.exists() {
        println!("No database found at: {}", database_path.display());
        println!("No checkpoints to clean.");
        return Ok(());
    }

    // TODO: Implement checkpoint cleanup
    // This would involve:
    // 1. Opening the database connection
    // 2. Querying all checkpoints ordered by creation time
    // 3. Identifying checkpoints to delete (keeping only the latest N)
    // 4. Deleting the old checkpoints

    println!(
        "Cleaning checkpoints, keeping {} latest from {}",
        keep_count,
        database_path.display()
    );
    println!("Checkpoint cleanup not yet implemented.");

    Ok(())
}

/// Validate compression configuration parameters
fn validate_compression_config(
    min_pattern_length: usize,
    min_frequency_threshold: usize,
    max_threads: Option<usize>,
    chunk_size_kb: usize,
    channel_buffer_size: usize,
    memory_map_threshold_mb: usize,
) -> Result<()> {
    // Validate pattern length
    if min_pattern_length < 2 {
        return Err(anyhow::anyhow!(
            "Minimum pattern length must be at least 2, got: {}",
            min_pattern_length
        ));
    }
    if min_pattern_length > 100 {
        return Err(anyhow::anyhow!(
            "Minimum pattern length cannot exceed 100, got: {}",
            min_pattern_length
        ));
    }

    // Validate frequency threshold
    if min_frequency_threshold < 2 {
        return Err(anyhow::anyhow!(
            "Minimum frequency threshold must be at least 2, got: {}",
            min_frequency_threshold
        ));
    }
    if min_frequency_threshold > 1000 {
        return Err(anyhow::anyhow!(
            "Minimum frequency threshold cannot exceed 1000, got: {}",
            min_frequency_threshold
        ));
    }

    // Validate thread count
    if let Some(threads) = max_threads {
        if threads == 0 {
            return Err(anyhow::anyhow!(
                "Thread count must be at least 1, got: {}",
                threads
            ));
        }
        if threads > 256 {
            return Err(anyhow::anyhow!(
                "Thread count cannot exceed 256, got: {}",
                threads
            ));
        }
    }

    // Validate chunk size
    if chunk_size_kb == 0 {
        return Err(anyhow::anyhow!(
            "Chunk size must be at least 1KB, got: {}KB",
            chunk_size_kb
        ));
    }
    if chunk_size_kb > 10 * 1024 {
        return Err(anyhow::anyhow!(
            "Chunk size cannot exceed 10MB, got: {}KB",
            chunk_size_kb
        ));
    }

    // Validate channel buffer size
    if channel_buffer_size == 0 {
        return Err(anyhow::anyhow!(
            "Channel buffer size must be at least 1, got: {}",
            channel_buffer_size
        ));
    }
    if channel_buffer_size > 10000 {
        return Err(anyhow::anyhow!(
            "Channel buffer size cannot exceed 10000, got: {}",
            channel_buffer_size
        ));
    }

    // Validate memory map threshold
    if memory_map_threshold_mb == 0 {
        return Err(anyhow::anyhow!(
            "Memory map threshold must be at least 1MB, got: {}MB",
            memory_map_threshold_mb
        ));
    }
    if memory_map_threshold_mb > 1024 {
        return Err(anyhow::anyhow!(
            "Memory map threshold cannot exceed 1GB, got: {}MB",
            memory_map_threshold_mb
        ));
    }

    // Cross-validation
    if let Some(threads) = max_threads {
        if threads > 64 && channel_buffer_size < 50 {
            return Err(anyhow::anyhow!(
                "High thread counts ({}) require larger channel buffers (at least 50), got: {}",
                threads,
                channel_buffer_size
            ));
        }
    }

    let chunk_size_bytes = chunk_size_kb * 1024;
    let memory_map_threshold_bytes = memory_map_threshold_mb * 1024 * 1024;
    if chunk_size_bytes > memory_map_threshold_bytes {
        return Err(anyhow::anyhow!(
            "Chunk size ({}KB) cannot be larger than memory map threshold ({}MB)",
            chunk_size_kb,
            memory_map_threshold_mb
        ));
    }

    // Cross-field validation for pattern analysis efficiency
    if min_pattern_length > 50 && min_frequency_threshold < 5 {
        warn!(
            "Large pattern lengths ({}) with low frequency thresholds ({}) may be inefficient",
            min_pattern_length, min_frequency_threshold
        );
    }

    info!("Configuration validation passed successfully");
    Ok(())
}
