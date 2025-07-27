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

#### Installation and Running

##### Clone and Run (Recommended)
```bash
git clone https://github.com/yourusername/archive-to-txt.git
cd archive-to-txt

# Run directly with cargo
cargo run -p archive-to-txt --release -- --input ./path/to/src
```

##### As a Library
Add to your `Cargo.toml`:
```toml
[dependencies]
archive-to-txt = { path = "./archive-to-txt" }  # Local path to the crate
```

### Basic Usage
```bash
# From the workspace root
cd /path/to/archive-to-txt

# Create archive of a directory (auto-named with timestamp)
cargo run -p archive-to-txt --release -- --input ./src

# Specify a custom output path
cargo run -p archive-to-txt --release -- --input ./src --output ./archive.txt

# See all options
cargo run -p archive-to-txt --release -- --help
```

### Strapi Project Example

Here's how to use archive-to-txt with a Strapi project:

#### Basic Archive (Auto-named with timestamp)
```bash
# From the workspace root
cargo run -p archive-to-txt --release -- \
  --input ./path/to/your/strapi/project

# Creates: ./path/to/your/strapi/your-project_archive_20250727_143000.txt
```

#### Specify Output Directory
```bash
cargo run -p archive-to-txt --release -- \
  --input ./path/to/your/strapi/project \
  --output /output/path/

# Creates: /output/path/your-project_archive_20250727_143000.txt
```

#### Advanced Usage with Filters
```bash
cargo run -p archive-to-txt --release -- \
  --input ./path/to/your/strapi/project \
  --include "*.js,*.jsx,*.ts,*.tsx,*.json" \
  --exclude "node_modules/**,build/**,.git/**" \
  --max-file-size 5MB
```

#### Custom Output Filename
```bash
cargo run -p archive-to-txt --release -- \
  --input ./path/to/your/strapi/project \
  --output ./custom_archive.txt \
  --include "*.js,*.jsx,*.ts,*.tsx,*.json"
```

#### Quick Run (Without Installation)

Run directly using Cargo without installing:

```bash
# From the workspace root
cargo run -p archive-to-txt --release -- \
  --input /path/to/your/strapi/project \
  --output strapi_archive.txt \
  --exclude-hidden \
  --max-file-size 5MB

# With debug output
RUST_LOG=debug cargo run -p archive-to-txt --release -- \
  --input /path/to/your/strapi/project \
  --output debug_strapi_archive.txt
```

#### Example Output
```
=== File: /config/database.js ===
module.exports = ({ env }) => ({
  connection: {
    client: 'postgres',
    connection: {
      host: env('DATABASE_HOST', 'localhost'),
      port: env.int('DATABASE_PORT', 5432),
      // ... more config
    },
  },
});

=== File: /src/api/restaurant/controllers/restaurant.js ===
'use strict';

/**
 * restaurant controller
 */

const { createCoreController } = require('@strapi/strapi').factories;

module.exports = createCoreController('api::restaurant.restaurant');
```

### Running the Tool

#### From the Workspace Root (Recommended)
```bash
# Navigate to the workspace root
cd /path/to/interview-irodov

# Basic usage with Strapi project
cargo run -p archive-to-txt --release -- \
  --input /path/to/your/strapi/project \
  --output /path/to/output/strapi_archive.txt

# With additional options
cargo run -p archive-to-txt --release -- \
  --input /path/to/your/strapi/project \
  --output /path/to/output/strapi_filtered.txt \
  --exclude-hidden \
  --max-file-size 2MB

# With debug output
RUST_LOG=debug cargo run -p archive-to-txt --release -- \
  --input /path/to/your/strapi/project \
  --output /path/to/output/debug_strapi_archive.txt
```

#### From the archive-to-txt Directory
```bash
# Navigate to the archive-to-txt directory
cd /path/to/interview-irodov/archive-to-txt

# Run in release mode
cargo run --release -- \
  --input /path/to/your/strapi/project \
  --output debug_output.txt
```

> **Note:** When running from the workspace root, use `-p archive-to-txt` to specify the package. This is not needed when running from the package directory.

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
cargo run -p archive-to-txt --release -- --input ./src --output archive.txt

# Show all available options
cargo run -p archive-to-txt --release -- --help

# Common options:
# - Exclude hidden files and directories
cargo run -p archive-to-txt --release -- --input . --output archive.txt --exclude-hidden

# - Set maximum file size (supports KB, MB, GB)
cargo run -p archive-to-txt --release -- --input . --output archive.txt --max-file-size 5MB

# - Disable parallel processing (useful for debugging)
cargo run -p archive-to-txt --release -- --input . --output archive.txt --no-parallel

# - Include gitignored files
cargo run -p archive-to-txt --release -- --input . --output all-files.txt --include-gitignored
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