use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use archive_to_txt::{archive_directory, config::Config};

/// Command line interface for the archive-to-txt tool
///
/// This tool creates a text-based archive of a directory's contents, including file contents
/// and metadata. It includes a directory tree structure at the beginning of the output,
/// making it easy to understand the project layout. It's designed for creating searchable 
/// archives of codebases or documentation.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input directory to archive (required)
    #[arg(short, long, value_name = "DIRECTORY")]
    input: PathBuf,

    /// Output file path (default: archive.txt in current directory)
    #[arg(short, long, value_name = "FILE", default_value = "archive.txt")]
    output: PathBuf,

    /// Exclude hidden files and directories (those starting with '.')
    #[arg(long)]
    exclude_hidden: bool,

    /// Disable parallel processing (may be slower but uses less memory)
    #[arg(long)]
    no_parallel: bool,

    /// Disable directory tree structure in output
    #[arg(long)]
    no_tree: bool,

    /// Disable LLM-optimized filtering (enabled by default - excludes build artifacts, dependencies, binaries)
    #[arg(long = "no-llm-optimize")]
    no_llm_optimize: bool,

    /// Show filtering statistics (files included/excluded with reasons)
    #[arg(long)]
    show_filter_stats: bool,

    /// Custom ignore patterns (glob patterns, can be used multiple times)
    #[arg(long)]
    ignore_pattern: Vec<String>,

    /// Include only specific file extensions (e.g., rs,js,py)
    #[arg(long)]
    include_extensions: Option<String>,
}

fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Validate input directory exists
    if !args.input.exists() {
        anyhow::bail!("Input directory does not exist: {}", args.input.display());
    }

    if !args.input.is_dir() {
        anyhow::bail!("Input path is not a directory: {}", args.input.display());
    }

    // Build configuration from command line arguments
    let mut config = Config::default()
        .with_input(&args.input)
        .with_output(&args.output)
        .with_include_hidden(!args.exclude_hidden)
        .with_parallel(!args.no_parallel)
        .with_include_tree(!args.no_tree)
        .with_llm_optimize(!args.no_llm_optimize); // LLM optimization enabled by default

    // Configure filtering options
    if args.show_filter_stats {
        config.show_filter_stats = true;
    }

    if !args.ignore_pattern.is_empty() {
        config.exclude = Some(args.ignore_pattern);
    }

    if let Some(extensions) = args.include_extensions {
        config = config.with_include_extensions(&extensions);
    }

    // Run the archive process
    println!("Creating archive from: {}", args.input.display());
    println!("Output will be saved to: {}", args.output.display());
    
    if !args.no_llm_optimize {
        println!("🤖 LLM optimization enabled by default - filtering build artifacts and dependencies");
    } else {
        println!("⚠️  LLM optimization disabled - including all files (use default behavior for cleaner archives)");
    }
    if args.show_filter_stats {
        println!("📊 Filter statistics will be shown");
    }
    
    archive_directory(&args.input, &args.output, &config)?;

    println!("\n✅ Successfully created archive at: {}", args.output.display());
    if let Ok(metadata) = std::fs::metadata(&args.output) {
        println!("   Archive size: {:.2} MB", metadata.len() as f64 / (1024.0 * 1024.0));
    }
    
    Ok(())
}
