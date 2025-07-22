# Code Tools Refactoring - TDD Implementation Plan

## Overview
This document outlines the TDD approach for refactoring the code tools into separate crates:
1. `code-archiver` - For archiving code directories (In Progress)
2. `file-splitter` - For splitting large files into smaller chunks (Completed ✅)
3. `common` - Shared utilities (Completed ✅)

## Project Structure
```
interview-irodov/
├── Cargo.toml                # Workspace configuration
├── code-archiver/            # Archive code directories
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       └── lib.rs
├── file-splitter/            # Split large files (Next)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       └── lib.rs
└── common/                   # Shared utilities (Completed)
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs
    │   ├── error.rs
    │   ├── fs/
    │   │   ├── mod.rs
    │   │   ├── file.rs
    │   │   └── metadata.rs
    │   └── path/
    │       ├── mod.rs
    │       ├── extension.rs
    │       └── name.rs
    └── tests/
        ├── basic_tests.rs
        └── integration_test.rs
```

## Implementation Checklist

### Phase 1: Setup Workspace
- [x] Create workspace Cargo.toml
- [x] Set up common crate structure
- [x] Set up file-splitter crate
- [ ] Set up code-archiver crate

### Phase 2: Common Crate (Completed ✅)
#### Error Handling
- [x] Define common error types
- [x] Implement `From` traits for error conversions
- [x] Add custom error messages and context
- [x] Implement `ResultExt` and `IoResultExt` traits

#### I/O Utilities
- [x] File operations with proper error handling
  - [x] Read/write files
  - [x] Create directories
  - [x] Handle temporary files/directories
  - [x] File metadata operations
- [x] Path manipulation utilities
  - [x] Extension handling
  - [x] File name and path operations
  - [x] Absolute/canonical path resolution
- [x] Comprehensive test coverage
  - [x] Unit tests
  - [x] Integration tests

## File Splitter Implementation (Completed ✅)

### Core Functionality
1. **Basic File Splitting**
   - [x] Define chunk size and naming strategy
   - [x] Implement sequential file splitting
   - [x] Handle file boundaries and chunk naming
   - [x] Support for both binary and text files

2. **Error Handling**
   - [x] Handle file system errors
   - [x] Validate input parameters
   - [x] Clean up partial files on failure

3. **CLI Interface**
   - [x] Command-line argument parsing
   - [x] Human-readable byte sizes (e.g., 1K, 2M, 1G)
   - [x] Verbose output option
   - [x] Progress reporting

### Testing Strategy
1. **Unit Tests**
   - [x] Test with small files
   - [x] Test with files smaller than chunk size
   - [x] Test with files exactly matching chunk size
   - [x] Test with files larger than chunk size

2. **Integration Tests**
   - [x] End-to-end file splitting
   - [x] Verification of split files
   - [x] Error scenarios

## Next Steps: Code Archiver Implementation

### Core Functionality
1. **Basic Archiving**
   - [ ] Traverse directory structure
   - [ ] Collect and filter files
   - [ ] Generate archive file
   - [ ] Support for different output formats

2. **Filtering Options**
   - [ ] Ignore patterns (.gitignore support)
   - [ ] File extension filtering
   - [ ] Size-based filtering
   - [ ] Last modified time filtering

3. **Output**
   - [ ] Human-readable output format
   - [ ] Progress reporting
   - [ ] Summary statistics

### Testing Strategy
1. **Unit Tests**
   - [ ] Test directory traversal
   - [ ] Test file filtering
   - [ ] Test archive generation

2. **Integration Tests**
   - [ ] Test with various directory structures
   - [ ] Test with different filter combinations
   - [ ] Test error handling

3. **Performance Tests**
   - [ ] Benchmark with large codebases
   - [ ] Memory usage monitoring
   - [ ] Parallel processing evaluation

### Phase 4: Code Archiver (In Progress)
#### Core Functionality
- [x] Basic directory traversal and file collection
- [x] File extension filtering
- [x] Size-based filtering
- [x] Basic output formatting (JSON and text)
- [ ] Archive file generation (in-memory only)
- [ ] Git integration
- [ ] Advanced output formatting (CSV, YAML)
- [ ] Progress reporting

#### Tests
- [x] Unit tests for basic functionality
- [x] Integration tests for core features
- [x] Tests for extension filtering
- [x] Tests for size-based filtering
- [ ] Tests for ignore patterns
- [ ] Tests for Git integration
- [ ] Performance benchmarks

### Phase 5: CLI Interfaces (Next Up)
- [x] Basic command-line argument parsing
- [x] Help and version information
- [ ] Progress reporting
- [x] Basic error handling
- [ ] Interactive mode
- [ ] Shell completion
- [ ] Verbose/debug output options

### Phase 6: Documentation (Next Up)
- [ ] Crate-level documentation
- [ ] Module documentation
- [ ] Function documentation
- [ ] Examples in documentation
- [ ] README files for each crate
- [ ] User guide
- [ ] API reference

### Phase 7: Performance Optimization
- [ ] Benchmark critical paths
- [ ] Optimize I/O operations
- [ ] Add parallel processing where beneficial
- [ ] Memory usage optimization

## TDD Workflow

### File Splitter Implementation
1. **Basic Splitting**
   - [ ] Test: Split empty file
   - [ ] Test: Split small file into chunks
   - [ ] Test: Split large file into chunks
   - [ ] Test: Handle file sizes not evenly divisible by chunk size

2. **Error Handling**
   - [ ] Test: Non-existent input file
   - [ ] Test: Invalid chunk size
   - [ ] Test: Read-only output directory
   - [ ] Test: Disk full scenario

3. **Verification**
   - [ ] Test: Verify split files match original
   - [ ] Test: Detect corrupted chunks
   - [ ] Test: Handle verification errors

### Code Archiver Implementation (In Progress)
1. **Basic Archiving**
   - [x] Test: Archive empty directory
   - [x] Test: Archive directory with files
   - [x] Test: Archive nested directory structure

2. **Filtering**
   - [x] Test: Extension filtering
   - [x] Test: Size-based filtering
   - [ ] Test: Ignore patterns (.gitignore)
   - [ ] Test: Last modified time filtering

3. **Output Formats**
   - [x] Text output
   - [x] JSON output
   - [ ] YAML output
   - [ ] CSV output
   - [ ] Custom template output
   - [ ] Test: Extension filtering
   - [ ] Test: Hidden files handling

3. **Output**
   - [ ] Test: Output file creation
   - [ ] Test: Output file formatting
   - [ ] Test: Large archive handling

## Rust Idiomatic Patterns to Follow

1. **Error Handling**
   - Use `thiserror` for custom error types
   - Implement proper error conversion traits
   - Provide context for errors

2. **API Design**
   - Use builder pattern for complex configurations
   - Follow Rust API guidelines
   - Use appropriate visibility (pub vs crate)

3. **Testing**
   - Follow Rust's testing conventions
   - Use test modules
   - Include doc tests

4. **Documentation**
   - Document all public APIs
   - Include examples
   - Follow Rustdoc conventions

## Progress Tracking

### Week 1 (Completed)
- [x] Set up workspace and basic crates
- [x] Implement common error handling
- [x] Complete file-splitter implementation
- [x] Comprehensive testing and documentation

### Week 2 (Current)
- [ ] Start code-archiver implementation
- [ ] Implement basic directory traversal
- [ ] Add file filtering capabilities
- [ ] Design archive format and structure

### Week 2
- [ ] Complete file splitter with tests
- [ ] Start code archiver implementation
- [ ] Add CLI interfaces

### Week 3
- [ ] Complete code archiver
- [ ] Add integration tests
- [ ] Performance optimization

### Week 4
- [ ] Documentation
- [ ] Final testing
- [ ] Release preparation

## Notes
- Always write tests first (Red-Green-Refactor)
- Keep commits small and focused
- Document design decisions
- Follow Rust's formatting and linting guidelines