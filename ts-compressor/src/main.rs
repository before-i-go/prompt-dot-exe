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
    common::{
        errors::Handler,
        source_map::SourceMap,
        Globals, GLOBALS, Mark,
    },
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
    },
    /// Archive code folder contents to timestamped text file
    Archive {
        /// Target folder to archive
        target_folder: PathBuf,
        /// Output directory for archive file (optional, defaults to parent of target)
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compress { input_dir, output_dir } => {
            compress_typescript(input_dir, output_dir)
        }
        Commands::Archive { target_folder, output_dir } => {
            archive_code_folder(target_folder, output_dir)
        }
    }
}

fn compress_typescript(input_dir: PathBuf, output_dir: PathBuf) -> Result<()> {
    fs::create_dir_all(&output_dir)?;

    for entry in WalkDir::new(&input_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.path().extension().map_or(false, |e| e == "ts" || e == "tsx") {
            let minified = minify_file(entry.path())?;
            let out_path = output_dir.join(entry.path().file_name().unwrap()).with_extension("js");
            let mut out_file = File::create(&out_path)?;
            out_file.write_all(minified.as_bytes())?;
        }
    }

    Ok(())
}

fn archive_code_folder(target_folder: PathBuf, output_dir: Option<PathBuf>) -> Result<()> {
    let archiver = CodeArchiver::new(target_folder, output_dir)?;
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
        let mut program = parser.parse_program().map_err(|e| anyhow::anyhow!("Parse failed: {:?}", e))?;

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
        emitter.emit_program(&program).context("Failed to emit code")?;
        
        Ok(String::from_utf8(buf).context("Invalid UTF-8")?)
    })
}

// Code Archiver Implementation following idiomatic Rust patterns
pub struct CodeArchiver {
    target_folder: PathBuf,
    output_dir: PathBuf,
    git_repo: Option<Repository>,
    is_git_repo: bool,
}

impl std::fmt::Debug for CodeArchiver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodeArchiver")
            .field("target_folder", &self.target_folder)
            .field("output_dir", &self.output_dir)
            .field("is_git_repo", &self.is_git_repo)
            .field("git_repo", &self.git_repo.is_some())
            .finish()
    }
}

impl CodeArchiver {
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
        })
    }

    /// Create the archive file (Pattern 4.1 - RAII pattern)
    pub fn create_archive(&self) -> Result<()> {
        let timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();
        let folder_name = self
            .target_folder
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        let output_file = self.output_dir.join(format!("{}-{}.txt", folder_name, timestamp));
        
        // Ensure output directory exists
        fs::create_dir_all(&self.output_dir)?;
        
        let mut file = File::create(&output_file)?;
        
        // Write header information
        self.write_header(&mut file)?;
        
        // Write directory structure
        self.write_directory_structure(&mut file)?;
        
        // Write file contents
        self.write_file_contents(&mut file)?;
        
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
        let rel_path = self.target_folder
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
        if let Ok(output) = Command::new("tree")
            .arg(&self.target_folder)
            .output()
        {
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
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        format!("{}├── {}", prefix, name)
    }

    /// Write file contents (Pattern 2.3 - Question mark operator chaining)
    fn write_file_contents(&self, file: &mut File) -> Result<()> {
        writeln!(file, "Processing files...")?;
        writeln!(file)?;
        
        if self.is_git_repo {
            self.write_git_file_contents(file)
        } else {
            self.write_all_file_contents(file)
        }
    }

    /// Write git-tracked file contents (Pattern 31.2 - Collection operations)
    fn write_git_file_contents(&self, file: &mut File) -> Result<()> {
        let repo = self.git_repo.as_ref().unwrap();
        let workdir = repo.workdir().unwrap_or(&self.target_folder);
        let rel_path = self.target_folder
            .strip_prefix(workdir)
            .unwrap_or(Path::new("."));
        
        let files = self.get_git_tracked_files(rel_path)?;
        
        for file_path in files {
            let full_path = workdir.join(&file_path);
            if full_path.is_file() {
                self.write_single_file_content(file, &full_path)?;
            }
        }
        
        Ok(())
    }

    /// Write all file contents (Pattern 15.1 - Custom iterators)
    fn write_all_file_contents(&self, file: &mut File) -> Result<()> {
        for entry in WalkDir::new(&self.target_folder) {
            let entry = entry?;
            if entry.file_type().is_file() {
                self.write_single_file_content(file, entry.path())?;
            }
        }
        Ok(())
    }

    /// Write content of a single file (Pattern 31.3 - Early returns and guards)
    fn write_single_file_content(&self, output_file: &mut File, file_path: &Path) -> Result<()> {
        writeln!(output_file, "Absolute path: {}", file_path.display())?;
        
        // Check if file is text or binary (Pattern 31.4 - Default values)
        let mime_type = from_path(file_path).first_or_octet_stream();
        let is_text = mime_type.type_() == mime::TEXT || 
                     mime_type == mime::APPLICATION_JSON ||
                     self.is_likely_text_file(file_path);
        
        if is_text {
            writeln!(output_file, "<text starts>")?;
            
            // Read and write file content (Pattern 4.1 - RAII pattern)
            match fs::read_to_string(file_path) {
                Ok(content) => {
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
            "rs", "toml", "md", "txt", "json", "yaml", "yml", 
            "js", "ts", "tsx", "jsx", "html", "css", "scss",
            "py", "rb", "go", "java", "c", "cpp", "h", "hpp",
            "sh", "bash", "zsh", "fish", "ps1", "bat", "cmd",
            "xml", "svg", "gitignore", "dockerfile", "makefile"
        ];
        
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| text_extensions.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
    }
}