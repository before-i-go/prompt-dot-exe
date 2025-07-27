# Interview Irodov Code Processing Toolkit

**🎯 MVP COMPLETE** - A high-performance Rust toolkit for code analysis, TypeScript processing, and file management.

## 🚀 **Quick Start**

```bash
# Build all tools
cargo build --release

# Code analysis
cargo run -p code-archiver -- --format json --dir ./my-project

# TypeScript compilation
cargo run -p ts-compressor -- compress ./src ./dist

# LLM-optimized code archiving
cargo run -p ts-compressor -- archive ./project

# File splitting
cargo run -p file-splitter -- --input large_file.txt --chunk-size 1M
```

## 📦 **Tools**

### 🔍 **code-archiver**
Analyze and archive code directories with advanced filtering
- Directory scanning with Git integration
- Multiple output formats (JSON, text)
- Extension and size filtering
- .gitignore support

### ⚡ **ts-compressor**
TypeScript compilation and LLM-optimized archiving
- **Compress**: TypeScript → minified JavaScript
- **Archive**: Create clean text archives with 270+ LLM exclusion patterns
- Binary file detection
- Statistics reporting

### ✂️ **file-splitter**
Split large files into manageable chunks
- Configurable chunk sizes (K, M, G units)
- Custom naming and output directories
- Content preservation guarantee

## 🏗️ **Architecture**

Built as a Rust Cargo workspace with:
- **Shared dependencies** for consistency
- **CLI-first design** for scriptability
- **Git-aware operations** for developer workflows
- **Cross-platform compatibility**

## 📊 **Status**

- ✅ **All core functionality implemented**
- ✅ **CLI interfaces complete**
- ✅ **Comprehensive test coverage**
- ✅ **Cross-tool integration validated**
- ✅ **Performance optimized**

## 📖 **Documentation**

See [MVP_COMPLETION_SUMMARY.md](MVP_COMPLETION_SUMMARY.md) for detailed status and usage examples.

## 🧪 **Testing**

```bash
# Run all tests
cargo test

# CLI integration tests
cargo test -p code-archiver --test cli_integration_test
```

## 🎯 **MVP Achievement**

The Interview Irodov toolkit successfully delivers a complete, tested, and production-ready suite of code processing tools.