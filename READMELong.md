# 📚 archive-to-txt - Detailed Documentation

## 📦 Overview
A high-performance Rust library and CLI tool for creating text-based archives of directory structures. Perfect for creating searchable, version-controlled snapshots of your codebase.

## 🚀 Features

### Core Features
- **Parallel Processing**: Utilizes Rayon for efficient multi-threaded file processing
- **Configurable**: Control over hidden files, file size limits, and output format
- **Fast**: Optimized for large codebases with minimal memory overhead
- **Thread-Safe**: Safe concurrent file processing with proper error handling

### Smart File Handling
- Recursively processes directories
- Respects `.gitignore` by default
- Configurable file size limits
- Parallel processing for better performance

### Output Format
- Clean, readable text output
- File headers with relative paths
- File content separation
- Summary statistics

## 📋 Usage Examples

### Basic Usage
```bash
# Install
cargo install --path archive-to-txt

# Basic usage
archive-to-txt --input ./src --output archive.txt
```

### Advanced Usage
```bash
# With options
archive-to-txt \
  --input . \
  --output project.txt \
  --exclude-hidden \
  --max-file-size 1MB \
  --parallel true

# Run with debug output
RUST_LOG=debug archive-to-txt --input . --output debug.txt
```

## 🛠 Development

### Building
```bash
cargo build --release
```

### Testing
```bash
cargo test
cargo test --test integration
cargo bench
```

## 🤝 Contributing

### Development Setup
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License
This project is dual-licensed under:
- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
