use std::path::PathBuf;
use clap::Parser;
use code_archiver::{ArchiveConfig, CodeArchiver};
use std::process;

/// A tool for archiving code directories with filtering and formatting options
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The directory to archive (default: current directory)
    #[arg(short, long, default_value = ".")]
    dir: PathBuf,
    
    /// File patterns to include (supports glob format)
    #[arg(short, long)]
    include: Vec<String>,
    
    /// File patterns to exclude (supports glob format)
    #[arg(short, long)]
    exclude: Vec<String>,
    
    /// File extensions to include (without leading .)
    #[arg(long)]
    extensions: Vec<String>,
    
    /// Maximum file size in bytes
    #[arg(long)]
    max_size: Option<u64>,
    
    /// Follow symbolic links
    #[arg(short = 'L', long)]
    follow_links: bool,
    
    /// Include hidden files
    #[arg(short = 'H', long)]
    hidden: bool,
    
    /// Don't respect .gitignore files
    #[arg(long)]
    no_gitignore: bool,
    
    /// Output format (json, text)
    #[arg(short, long, default_value = "text")]
    format: String,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    // Parse command line arguments
    let args = Args::parse();
    
    // Initialize logging
    let log_level = if args.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();
    
    // Use max_size directly as it's now a u64
    let max_file_size = args.max_size;
    
    // Create archive configuration
    let config = ArchiveConfig {
        root_dir: args.dir,
        include: if args.include.is_empty() { None } else { Some(args.include) },
        exclude: if args.exclude.is_empty() { None } else { Some(args.exclude) },
        extensions: if args.extensions.is_empty() { None } else { Some(args.extensions) },
        max_size: max_file_size,
        follow_links: args.follow_links,
        hidden: args.hidden,
        gitignore: !args.no_gitignore,
        include_git_status: false,  // Default to false for CLI
        include_ignored: false,     // Default to false for CLI
    };
    
    // Create and run the archiver
    match CodeArchiver::new(config) {
        Ok(archiver) => {
            match args.format.as_str() {
                "json" => {
                    match archiver.archive_to_json() {
                        Ok(json) => println!("{}", json),
                        Err(e) => {
                            eprintln!("Error creating archive: {}", e);
                            process::exit(1);
                        }
                    }
                }
                "text" => {
                    match archiver.create_archive() {
                        Ok(entries) => {
                            let count = entries.len();
                            for entry in &entries {
                                println!("{:8}  {}  {}", 
                                    bytesize::to_string(entry.size, true),
                                    entry.modified,
                                    entry.path
                                );
                            }
                            println!("\nTotal: {} files", count);
                        }
                        Err(e) => {
                            eprintln!("Error creating archive: {}", e);
                            process::exit(1);
                        }
                    }
                }
                _ => {
                    eprintln!("Error: Unsupported format '{}'. Use 'json' or 'text'.", args.format);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error initializing archiver: {}", e);
            process::exit(1);
        }
    }
}

/// Parse a human-readable size string (e.g., 1K, 2M, 1G) into bytes
fn parse_size(size_str: &str) -> Result<u64, String> {
    let size_str = size_str.trim();
    if size_str.is_empty() {
        return Err("Empty size string".to_string());
    }
    
    // Find the split between number and unit
    let split_pos = size_str.find(|c: char| !c.is_ascii_digit() && !c.is_whitespace())
        .unwrap_or_else(|| size_str.len());
    
    // Parse the numeric part
    let (num_str, unit) = size_str.split_at(split_pos);
    let num: u64 = num_str.trim().parse().map_err(|e| format!("Invalid number '{}': {}", num_str.trim(), e))?;
    
    // Parse the unit (trim any whitespace)
    let unit = unit.trim();
    let multiplier = match unit.to_uppercase().as_str() {
        "" | "B" => 1,
        "K" | "KB" => 1024,
        "M" | "MB" => 1024 * 1024,
        "G" | "GB" => 1024 * 1024 * 1024,
        _ => return Err(format!("Invalid unit '{}'. Use K, M, or G.", unit)),
    };
    
    num.checked_mul(multiplier)
        .ok_or_else(|| "Size too large".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1024").unwrap(), 1024);
        assert_eq!(parse_size("1K").unwrap(), 1024);
        assert_eq!(parse_size("1KB").unwrap(), 1024);
        assert_eq!(parse_size("1M").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1MB").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1G").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_size("1GB").unwrap(), 1024 * 1024 * 1024);
        // Test with spaces
        assert_eq!(parse_size("1 K").unwrap(), 1024);
        assert_eq!(parse_size("1 M").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1 G").unwrap(), 1024 * 1024 * 1024);
        // Test with spaces and no unit (should be treated as bytes)
        assert_eq!(parse_size(" 1024 ").unwrap(), 1024);
        
        assert!(parse_size("").is_err());
        assert!(parse_size("abc").is_err());
        assert!(parse_size("1X").is_err());
    }
}
