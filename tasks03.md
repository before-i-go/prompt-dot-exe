# archive-to-txt MVP Tasks

## Phase 1: Core Functionality (TDD Cycle)
### 1.1 File Processing
- [x] Basic file I/O for directory traversal
- [ ] Test: Handle large files (memory mapping)
- [ ] Implement: Binary file detection and handling
- [ ] Test: Symbolic link handling
- [ ] Implement: Safe symlink resolution

### 1.2 Filtering System
- [x] Basic file filtering by extension
- [x] Include/exclude patterns
- [x] File size limits
- [ ] Test: Complex pattern matching
- [ ] Implement: Gitignore-style pattern support
- [ ] Test: Performance with large numbers of patterns

### 1.3 Output Formatting
- [x] Basic text output
- [ ] Test: Different output formats (JSON, Markdown)
- [ ] Implement: Configurable formatters
- [ ] Test: Unicode and special character handling
- [ ] Implement: Progress reporting

## Phase 2: CLI & Integration
### 2.1 Command Line Interface
- [x] Basic CLI with clap
- [ ] Test: All command-line options
- [ ] Implement: Verbose/debug output
- [ ] Test: Error handling and user feedback
- [ ] Implement: Progress indicators

### 2.2 Git Integration
- [ ] Test: Git repository detection
- [ ] Implement: .gitignore support
- [ ] Test: Sparse checkout handling
- [ ] Implement: Git history inclusion (optional)

## Phase 3: Performance & Optimization
### 3.1 Parallel Processing
- [x] Basic parallel file processing
- [ ] Test: Thread pool sizing
- [ ] Implement: Work stealing for load balancing
- [ ] Test: Performance profiling
- [ ] Implement: Memory usage optimization

### 3.2 Caching
- [ ] Test: File modification detection
- [ ] Implement: Incremental updates
- [ ] Test: Cache invalidation
- [ ] Implement: Persistent cache storage

## Phase 4: Polish & Documentation
### 4.1 Error Handling
- [x] Basic error types
- [ ] Test: Error recovery
- [ ] Implement: Detailed error messages
- [ ] Test: Edge cases and error paths

### 4.2 Documentation
- [x] Basic README
- [ ] Write comprehensive user guide
- [ ] Document all public APIs
- [ ] Add examples
- [ ] Create man pages

## Phase 5: Testing & Quality
### 5.1 Test Coverage
- [ ] Unit tests for all modules
- [ ] Integration tests
- [ ] Performance benchmarks
- [ ] Fuzz testing

### 5.2 Linting & Formatting
- [x] Basic clippy setup
- [ ] Custom clippy lints
- [ ] Format with rustfmt
- [ ] Audit dependencies

## Implementation Notes:
- Follow TDD strictly (Red-Green-Refactor)
- Document all public APIs with examples
- Ensure all tests pass before committing
- Follow Rust idioms and best practices
- Keep functions small and focused
- Optimize for both small and large codebases
- Handle edge cases gracefully (permissions, symlinks, etc.)