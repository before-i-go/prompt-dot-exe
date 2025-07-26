use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use archive_to_txt::{archive_directory, config::Config};

/// Command line interface for the archive-to-txt tool
///
/// This tool creates a text-based archive of a directory's contents, including file contents
/// and metadata. It's designed for creating searchable archives of codebases or documentation.
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
    let config = Config::default()
        .with_input(&args.input)
        .with_output(&args.output)
        .with_include_hidden(!args.exclude_hidden)
        .with_parallel(!args.no_parallel);

    // Run the archive process
    println!("Creating archive from: {}", args.input.display());
    println!("Output will be saved to: {}", args.output.display());
    
    archive_directory(&args.input, &args.output, &config)?;

    println!("\n✅ Successfully created archive at: {}", args.output.display());
    if let Ok(metadata) = std::fs::metadata(&args.output) {
        println!("   Archive size: {:.2} MB", metadata.len() as f64 / (1024.0 * 1024.0));
    }
    
    Ok(())
}
