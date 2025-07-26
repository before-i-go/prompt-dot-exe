# 📦 archive-to-txt

**Create searchable, version-controlled text archives of your codebase**

[![Crates.io](https://img.shields.io/crates/v/archive-to-txt)](https://crates.io/crates/archive-to-txt)
[![Documentation](https://docs.rs/archive-to-txt/badge.svg)](https://docs.rs/archive-to-txt)
[![License](https://img.shields.io/crates/l/archive-to-txt)](LICENSE-APACHE)
[![Rust](https://github.com/yourusername/archive-to-txt/actions/workflows/rust.yml/badge.svg)](https://github.com/yourusername/archive-to-txt/actions)

A high-performance Rust library and CLI tool for creating text-based archives of directory contents with parallel processing support.

## Why Use archive-to-txt?

1. **Efficient** - Processes large codebases quickly with parallel execution
2. **Simple** - Single command creates comprehensive text archives
3. **Flexible** - Customize output with various configuration options
4. **Smart** - Respects `.gitignore` and handles binary files intelligently

## Quick Start

### Installation

#### From Crates.io
```bash
cargo install archive-to-txt
```

#### From Source
```bash
git clone https://github.com/yourusername/archive-to-txt.git
cd archive-to-txt
cargo install --path .
```

#### As a Library
Add to your `Cargo.toml`:
```toml
[dependencies]
archive-to-txt = "1.0.0"
```

### Basic Usage
```bash
# Create archive of a directory
archive-to-txt --input ./src --output archive.txt

# See all options
archive-to-txt --help
```

### Quick Run (Without Installation)

Run directly using Cargo without installing. There are two ways to run the tool:

#### Option 1: From the workspace root (recommended)
```bash
# Navigate to the workspace root if you're not already there
cd /home/amuldotexe/Desktop/GitHub202410/interview-irodov

# Basic usage with the strapi project
cargo run -p archive-to-txt --release -- --input /home/amuldotexe/Desktop/GitHub202410/ab202507/strapi --output /home/amuldotexe/Desktop/GitHub202410/ab202507/strapi_archive.txt

# With additional options (exclude hidden files, limit file size)
cargo run -p archive-to-txt --release -- --input /home/amuldotexe/Desktop/GitHub202410/ab202507/strapi --output /home/amuldotexe/Desktop/GitHub202410/ab202507/strapi_filtered.txt --exclude-hidden --max-file-size 2MB

# Process with debug output (shows detailed processing information
RUST_LOG=debug cargo run -p archive-to-txt --release -- --input /home/amuldotexe/Desktop/GitHub202410/ab202507/strapi --output /home/amuldotexe/Desktop/GitHub202410/ab202507/debug_strapi_archive.txt
```

#### Option 2: From the archive-to-txt directory
```bash
# Navigate to the archive-to-txt directory
cd /home/amuldotexe/Desktop/GitHub202410/interview-irodov/archive-to-txt

# Run in debug mode (faster build, slower execution)
# Output will be saved as 'debug_output.txt' in the current directory
cargo run --release -- --input /home/amuldotexe/Desktop/GitHub202410/ab202507/strapi --output debug_output.txt

# To save in a specific directory, use an absolute or relative path:
cargo run --release -- --input /home/amuldotexe/Desktop/GitHub202410/ab202507/strapi --output /path/to/save/archive.txt
```

> **Note:** The `-p archive-to-txt` flag specifies which package to run from the workspace. This is required when running from the workspace root.

## 📦 Project Structure

This repository contains multiple Rust tools for code processing and analysis:

- **`archive-to-txt/`** - Main crate for creating text-based archives
- **`code-archiver/`** - Utility for analyzing and archiving code structure
- **`ts-compressor/`** - TypeScript code compression and optimization
- **`file-splitter/`** - Tool for splitting large files into smaller chunks
- **`common/`** - Shared utilities and libraries
- **`impRustIdioms/`** - Rust patterns and best practices documentation
- **`test-input/`** - Sample files for testing

## ✨ Features

### Core Features
- **Parallel Processing** - Utilizes Rayon for efficient multi-threaded execution
- **Smart Filtering** - Respects `.gitignore` and handles binary files intelligently
- **Configurable** - Control file size limits, hidden files, and output format
- **Lightweight** - Minimal dependencies, fast compilation

### Additional Tools
- **Code Analysis** - Detailed project structure analysis
- **File Operations** - Advanced file handling and processing
- **TypeScript Support** - Specialized TypeScript processing capabilities

## 📚 Documentation

For detailed documentation and usage examples:
- [Changelog](CHANGELOG.md)
- [Contributing Guide](CONTRIBUTING.md)
- [License Information](LICENSE-APACHE)

> **Note:** The crate is not yet published to crates.io. For now, please refer to the source code documentation by running `cargo doc --open` in the project directory.

## Contributing

Contributions are welcome! Please read our [Contributing Guidelines](CONTRIBUTING.md).

## License

Dual-licensed under:
- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

## 🚀 Advanced Usage

### Command Line Options

```bash
# Basic usage
archive-to-txt --input ./src --output archive.txt

# Show all available options
archive-to-txt --help

# Common options:
# - Exclude hidden files and directories
archive-to-txt --input . --output archive.txt --exclude-hidden

# - Set maximum file size (supports KB, MB, GB)
archive-to-txt --input . --output archive.txt --max-file-size 5MB

# - Disable parallel processing (useful for debugging)
archive-to-txt --input . --output archive.txt --no-parallel

# - Include gitignored files
archive-to-txt --input . --output all-files.txt --include-gitignored
```

### Configuration File

Create a `.archive-to-txt.toml` file in your project root to customize behavior:

```toml
[default]
parallel = true
exclude_hidden = true
max_file_size = "5MB"
output_format = "text"
include_gitignored = false

# File type specific settings
[file_types]
# Maximum size for specific file types (overrides global max_file_size)
max_size = { "*.rs" = "10MB", "*.ts" = "5MB" }

# Exclude specific file patterns
exclude = ["**/target/**", "**/node_modules/**"]
```

## 🛠 Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/interview-irodov.git
cd interview-irodov

# Build all tools in release mode
cargo build --release

# Build specific tool
cargo build -p archive-to-txt --release

# Install to Cargo's bin directory
cargo install --path archive-to-txt
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p archive-to-txt

# Run with detailed output
RUST_LOG=debug cargo test -- --nocapture
```

### Code Quality

We maintain high code quality standards. Before submitting changes:

```bash
# Format code according to style guidelines
cargo fmt --all

# Run linter
cargo clippy --all-targets -- -D warnings

# Check for unused dependencies
cargo udeps

# Check for security vulnerabilities
cargo audit
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details on how to:
- Report issues
- Submit pull requests
- Set up your development environment
- Run tests

## 📄 License

This project is dual-licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.