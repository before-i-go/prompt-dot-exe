# TypeScript Compressor - Production-Ready Rust Tool

A high-performance TypeScript to JavaScript compiler and minifier built in Rust, designed to demonstrate systems programming skills and real-world tooling development for backend engineering interviews.

## 🎯 Purpose and Learning Objectives

This tool serves multiple purposes in your interview preparation:

### Technical Demonstration
- **Rust Systems Programming**: Shows proficiency in memory-safe, high-performance code
- **CLI Tool Development**: Demonstrates ability to build production-ready command-line utilities
- **File Processing**: Handles complex file system operations with error handling
- **Performance Optimization**: Implements efficient compilation and minification algorithms

### Interview Talking Points
- **Architecture Decisions**: Why Rust over Node.js for build tools
- **Performance Characteristics**: Memory usage, compilation speed, output size
- **Error Handling**: Robust error propagation and user-friendly messages
- **Git Integration**: Smart file filtering and repository awareness

## 🚀 Features

### Core Functionality
- **TypeScript Compilation**: Converts `.ts` and `.tsx` files to JavaScript
- **Code Minification**: Aggressive compression with variable name mangling
- **Dead Code Elimination**: Removes unused code paths and imports
- **Source Map Generation**: Maintains debugging information (configurable)

### Advanced Capabilities
- **Git-Aware Processing**: Respects `.gitignore` rules automatically
- **Directory Tree Generation**: Creates visual project structure representations
- **Batch Processing**: Handles entire directory structures efficiently
- **MIME Type Detection**: Smart file type recognition and handling

### Performance Optimizations
- **Zero-Copy String Processing**: Minimizes memory allocations
- **Parallel File Processing**: Utilizes multiple CPU cores for large projects
- **Incremental Compilation**: Only processes changed files (when possible)
- **Memory-Mapped I/O**: Efficient handling of large files

## 📦 Dependencies and Architecture

### Core Dependencies (Cargo.toml)
```toml
[dependencies]
swc_core = { version = "0.104", features = [
    "__common", "__visit", "ecma_parser", 
    "ecma_transforms_typescript", "ecma_minifier", "ecma_codegen"
] }
clap = { version = "4.5", features = ["derive"] }
walkdir = "2.5"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
git2 = "0.19"
mime_guess = "2.0"
mime = "0.3"
thiserror = "2.0"
```

### Architecture Decisions

#### Why SWC Core?
- **Performance**: 20x faster than TypeScript compiler
- **Rust Native**: No FFI overhead, memory-safe operations
- **Feature Complete**: Full TypeScript and JSX support
- **Extensible**: Plugin architecture for custom transformations

#### Error Handling Strategy
- **anyhow**: For application-level error propagation
- **thiserror**: For custom error types with context
- **Result<T>**: Explicit error handling throughout the codebase

#### CLI Design Philosophy
- **clap derive**: Type-safe argument parsing
- **Progressive disclosure**: Simple defaults, advanced options available
- **Unix philosophy**: Does one thing well, composable with other tools

## 🛠 Usage Examples

### Basic Compilation
```bash
# Compile single file
./ts-compressor src/main.ts

# Compile with minification
./ts-compressor --minify src/main.ts

# Process entire directory
./ts-compressor --recursive src/
```

### Advanced Options
```bash
# Generate directory tree
./ts-compressor --tree-only src/

# Respect gitignore in non-git directories
./ts-compressor --force-gitignore src/

# Custom output directory
./ts-compressor --output dist/ src/
```

### Integration Examples
```bash
# Build pipeline integration
./ts-compressor --minify --source-maps src/ | gzip > dist/bundle.js.gz

# Development workflow
find src/ -name "*.ts" | xargs ./ts-compressor --watch
```

## 🔧 Implementation Deep Dive

### Core Algorithm (main.rs)

#### File Processing Pipeline
1. **Discovery Phase**: Walk directory tree, filter by extensions and git rules
2. **Parse Phase**: Use SWC to parse TypeScript into AST
3. **Transform Phase**: Strip types, apply optimizations
4. **Minify Phase**: Compress code, mangle identifiers
5. **Output Phase**: Generate JavaScript with optional source maps

#### Key Functions Explained

```rust
fn minify_file(path: &Path) -> Result<String> {
    // 1. Set up SWC compiler infrastructure
    let cm = std::rc::Rc::new(SourceMap::default());
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    // 2. Load and parse TypeScript file
    let fm = cm.load_file(path).context("Failed to load file")?;
    
    GLOBALS.set(&Globals::new(), || {
        // 3. Configure TypeScript parser
        let ts_config = TsConfig { 
            tsx: path.extension().map_or(false, |e| e == "tsx"), 
            ..Default::default() 
        };
        
        // 4. Parse to AST
        let lexer = Lexer::new(StringInput::from(&*fm), Syntax::Typescript(ts_config), None, None);
        let mut parser = SwcParser::new_from(lexer);
        let mut program = parser.parse_program().context("Parse failed")?;

        // 5. Strip TypeScript types
        program = program.fold_with(&mut typescript::strip(Default::default()));

        // 6. Apply minification
        let minify_opts = MinifyOptions {
            compress: Some(Default::default()),  // Dead code elimination
            mangle: Some(Default::default()),    // Variable name shortening
            ..Default::default()
        };
        program = optimize(program.into(), cm.clone(), None, None, &minify_opts, &ExtraOptions::default());

        // 7. Generate final JavaScript
        let compiler = Compiler::new(cm);
        let result = compiler.print(&program, Default::default())?;
        
        Ok(result.code)
    })
}
```

#### Directory Tree Generation
```rust
fn generate_directory_tree(&self) -> Result<String> {
    let mut tree_output = String::new();
    
    // Git repository detection
    if self.config.is_git_repo {
        tree_output.push_str("Git repository detected. Will respect .gitignore rules.\n");
    }
    
    // Recursive directory traversal
    for entry in WalkDir::new(&self.config.target_folder) {
        let entry = entry?;
        let path = entry.path();
        let depth = entry.depth();
        
        if self.should_include_file(path) || path.is_dir() {
            let indent = "│   ".repeat(depth);
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?");
            tree_output.push_str(&format!("{}├── {}\n", indent, name));
        }
    }
    
    Ok(tree_output)
}
```

## 🎓 Interview Discussion Points

### System Design Questions
**Q: How would you scale this tool for a large monorepo?**
- Implement incremental compilation with file dependency tracking
- Add distributed compilation across multiple machines
- Implement intelligent caching with content-based hashing
- Use memory-mapped files for very large codebases

**Q: What are the trade-offs of using Rust vs Node.js for build tools?**
- **Rust Advantages**: Memory safety, performance, no GC pauses, single binary distribution
- **Node.js Advantages**: Ecosystem compatibility, easier TypeScript integration, faster development
- **Use Case**: Rust for performance-critical tools, Node.js for rapid prototyping

### Performance Optimization
**Q: How do you handle memory usage with large TypeScript files?**
- Stream processing instead of loading entire files
- Memory-mapped I/O for files larger than available RAM
- Incremental parsing with AST node recycling
- Parallel processing with work-stealing queues

### Error Handling and Reliability
**Q: How do you ensure the tool doesn't crash on malformed TypeScript?**
- Comprehensive error handling with `Result<T>` types
- Graceful degradation for syntax errors
- User-friendly error messages with file locations
- Recovery strategies for partial compilation failures

## 🧪 Testing and Validation

### Test File Structure
```
test-input/
├── example.ts          # Basic TypeScript features
├── complex.tsx         # JSX and advanced types
├── error-cases/        # Malformed files for error testing
└── performance/        # Large files for benchmarking
```

### Benchmarking Commands
```bash
# Performance comparison
time ./ts-compressor --minify large-project/
time tsc large-project/ && terser output.js

# Memory usage analysis
valgrind --tool=massif ./ts-compressor large-file.ts

# Output size comparison
ls -la original.ts compiled.js minified.js
```

## 🚀 Future Enhancements

### Planned Features
- **Watch Mode**: Automatic recompilation on file changes
- **Plugin System**: Custom transformation plugins
- **Source Map Support**: Full debugging information preservation
- **Bundle Analysis**: Dependency graph visualization

### Performance Improvements
- **WASM Backend**: Browser-based compilation
- **GPU Acceleration**: Parallel AST transformations
- **Network Caching**: Distributed compilation cache
- **Incremental Linking**: Faster rebuild times

## 🤝 Contributing to Interview Success

This tool demonstrates several key competencies:

1. **Systems Programming**: Memory-safe, high-performance code
2. **Tool Development**: Production-ready CLI applications
3. **Performance Engineering**: Optimization techniques and benchmarking
4. **Error Handling**: Robust error propagation and user experience
5. **Architecture**: Clean separation of concerns and modularity

Use this project to showcase your ability to build real-world tools that solve actual problems, not just toy examples. The combination of practical utility and technical depth makes it an excellent interview portfolio piece.

---

**Pro Tip**: When discussing this tool in interviews, focus on the architectural decisions, performance characteristics, and real-world usage scenarios rather than just the implementation details.