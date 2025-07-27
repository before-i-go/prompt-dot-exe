# Design Document

## Overview

This design outlines the MVP completion strategy for the "Interview Irodov" Code Processing Toolkit. Analysis shows that most core functionality is already implemented with good test coverage. This focused approach targets the remaining gaps to achieve a functional MVP release quickly.

The design prioritizes completing the missing TS-Compressor archive functionality, ensuring robust CLI integration, and validating end-to-end workflows. This pragmatic approach builds on the substantial existing implementation rather than starting from scratch.

## Architecture

### Testing Layers

The testing architecture follows a three-tier approach:

1. **Unit Tests**: Test individual functions, methods, and modules in isolation
2. **Integration Tests**: Test component interactions and workflows within each utility
3. **End-to-End Tests**: Test complete user workflows across the entire system

### Test Organization Structure

```
├── code-archiver/
│   ├── src/
│   │   └── lib.rs (unit tests inline)
│   └── tests/
│       ├── unit/
│       │   ├── directory_scanning_test.rs
│       │   ├── file_filtering_test.rs
│       │   ├── git_integration_test.rs
│       │   └── output_formatting_test.rs
│       ├── integration/
│       │   ├── cli_integration_test.rs
│       │   └── workflow_integration_test.rs
│       └── fixtures/
│           └── test_data/
├── ts-compressor/
│   ├── src/
│   │   └── main.rs (unit tests inline)
│   └── tests/
│       ├── unit/
│       │   ├── compress_command_test.rs
│       │   ├── archive_command_test.rs
│       │   └── llm_filtering_test.rs
│       ├── integration/
│       │   ├── cli_integration_test.rs
│       │   └── workflow_integration_test.rs
│       └── fixtures/
│           └── test_projects/
├── file-splitter/
│   ├── src/
│   │   └── main.rs (unit tests inline)
│   └── tests/
│       ├── unit/
│       │   ├── file_splitting_test.rs
│       │   └── size_parsing_test.rs
│       ├── integration/
│       │   └── cli_integration_test.rs
│       └── fixtures/
│           └── test_files/
├── common/
│   ├── src/
│   │   └── lib.rs (unit tests inline)
│   └── tests/
│       ├── unit/
│       │   ├── fs_operations_test.rs
│       │   ├── path_utilities_test.rs
│       │   └── error_handling_test.rs
│       └── integration/
│           └── cross_platform_test.rs
└── tests/ (workspace-level)
    ├── end_to_end/
    │   ├── complete_workflows_test.rs
    │   └── performance_test.rs
    └── fixtures/
        └── sample_projects/
```

## Components and Interfaces

### Test Infrastructure Components

#### 1. Test Utilities Module
**Location**: `common/src/test_utils.rs`
**Purpose**: Shared testing utilities across all crates

```rust
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub git_repo: Option<TestGitRepo>,
}

pub struct TestGitRepo {
    pub repo: Repository,
    pub temp_dir: TempDir,
}

pub trait TestFixture {
    fn setup() -> Self;
    fn cleanup(&self);
}
```

#### 2. Mock Data Generators
**Location**: `common/src/test_data.rs`
**Purpose**: Generate consistent test data across all test suites

```rust
pub struct FileGenerator;
pub struct ProjectGenerator;
pub struct GitRepoGenerator;
```

#### 3. Assertion Helpers
**Location**: `common/src/test_assertions.rs`
**Purpose**: Custom assertions for domain-specific testing

```rust
pub fn assert_file_exists(path: &Path);
pub fn assert_archive_contains(archive: &Archive, expected_files: &[&str]);
pub fn assert_git_status(entry: &FileEntry, expected: GitStatus);
```

### Testing Strategy by Component

#### Code-Archiver Testing Strategy

**Unit Tests**:
- Directory traversal logic
- File filtering algorithms (glob patterns, extensions, size)
- Git integration functions
- Output formatting (text/JSON)
- Error handling for invalid inputs

**Integration Tests**:
- CLI argument parsing and validation
- Complete archiving workflows
- Git repository integration scenarios
- Performance with large directory structures

**Key Test Scenarios**:
- Empty directories
- Deeply nested structures
- Large files and many files
- Various Git states (modified, untracked, ignored)
- Complex filtering combinations
- Cross-platform path handling

#### TS-Compressor Testing Strategy

**Unit Tests**:
- TypeScript compilation logic
- Minification functionality
- LLM optimization filtering patterns
- Binary file detection
- Archive format generation
- Statistics calculation

**Integration Tests**:
- Complete compress workflows
- Complete archive workflows
- Custom filtering combinations
- Large project processing

**Key Test Scenarios**:
- Various TypeScript/JSX syntax patterns
- All 270+ LLM exclusion patterns
- Binary vs text file detection
- Custom ignore patterns
- Extension filtering
- Statistics accuracy

#### File-Splitter Testing Strategy

**Unit Tests**:
- Size parsing logic (K, M, G units)
- File splitting algorithms
- Chunk naming logic
- Output directory handling

**Integration Tests**:
- CLI workflows with various options
- Large file processing
- Edge cases (empty files, small files)

**Key Test Scenarios**:
- Files smaller than chunk size
- Files exactly matching chunk size
- Very large files
- Custom naming and directory options
- Invalid size specifications

#### Common Library Testing Strategy

**Unit Tests**:
- File system operations
- Path manipulation utilities
- Error type definitions and conversions
- Cross-platform compatibility

**Integration Tests**:
- File operations across different OS
- Path handling edge cases
- Error propagation scenarios

## Data Models

### Test Configuration Model

```rust
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub temp_dir_prefix: String,
    pub cleanup_on_drop: bool,
    pub git_enabled: bool,
    pub performance_tracking: bool,
}

#[derive(Debug)]
pub struct TestResult {
    pub passed: bool,
    pub duration: Duration,
    pub memory_usage: Option<usize>,
    pub error_message: Option<String>,
}
```

### Test Data Models

```rust
#[derive(Debug, Clone)]
pub struct TestFileSpec {
    pub path: PathBuf,
    pub content: String,
    pub size: Option<usize>,
    pub git_status: Option<GitStatus>,
}

#[derive(Debug, Clone)]
pub struct TestProjectSpec {
    pub name: String,
    pub files: Vec<TestFileSpec>,
    pub directories: Vec<PathBuf>,
    pub git_repo: bool,
}
```

## Error Handling

### Test Error Categories

1. **Setup Errors**: Issues creating test environments or fixtures
2. **Execution Errors**: Failures during test execution
3. **Assertion Errors**: Test expectations not met
4. **Cleanup Errors**: Issues cleaning up test resources

### Error Handling Strategy

```rust
#[derive(Debug, thiserror::Error)]
pub enum TestError {
    #[error("Test setup failed: {0}")]
    Setup(String),
    
    #[error("Test execution failed: {0}")]
    Execution(String),
    
    #[error("Assertion failed: expected {expected}, got {actual}")]
    Assertion { expected: String, actual: String },
    
    #[error("Test cleanup failed: {0}")]
    Cleanup(String),
}
```

### Error Recovery

- Automatic cleanup of temporary resources
- Graceful handling of platform-specific failures
- Detailed error reporting for debugging
- Retry mechanisms for flaky tests

## Testing Strategy

### Test Categories and Coverage Goals

#### 1. Functional Testing
- **Goal**: 100% coverage of all public APIs
- **Approach**: Test each function with valid inputs, edge cases, and error conditions
- **Tools**: Standard Rust test framework, custom assertions

#### 2. Integration Testing
- **Goal**: Verify component interactions work correctly
- **Approach**: Test realistic workflows end-to-end
- **Tools**: `assert_fs`, `tempfile`, custom test utilities

#### 3. Performance Testing
- **Goal**: Ensure acceptable performance characteristics
- **Approach**: Benchmark critical paths, memory usage monitoring
- **Tools**: `criterion` for benchmarking, custom memory tracking

#### 4. Compatibility Testing
- **Goal**: Verify cross-platform functionality
- **Approach**: Test on different OS, file systems, Git configurations
- **Tools**: Platform-specific test runners, Docker containers

### Test Data Management

#### Fixture Organization
- **Static Fixtures**: Pre-created test files and directories
- **Dynamic Fixtures**: Generated test data for specific scenarios
- **Git Fixtures**: Pre-configured Git repositories with various states

#### Test Data Generation
- Deterministic generation for reproducible tests
- Configurable data sizes for performance testing
- Realistic file content for integration testing

### Continuous Integration Strategy

#### Test Execution Pipeline
1. **Fast Tests**: Unit tests and basic integration tests
2. **Comprehensive Tests**: Full integration and compatibility tests
3. **Performance Tests**: Benchmarks and memory usage tests
4. **Coverage Analysis**: Code coverage reporting and analysis

#### Quality Gates
- Minimum 90% code coverage
- All tests must pass
- Performance benchmarks within acceptable ranges
- No memory leaks detected

### Test Maintenance Strategy

#### Test Organization Principles
- One test file per module/feature
- Clear test naming conventions
- Comprehensive documentation
- Regular test review and updates

#### Test Data Lifecycle
- Automatic cleanup of temporary resources
- Version control for static test fixtures
- Regular updates to reflect real-world scenarios

## Implementation Phases

### Phase 1: Foundation
- Set up test infrastructure and utilities
- Implement basic unit tests for core functionality
- Establish CI/CD pipeline for test execution

### Phase 2: Comprehensive Unit Testing
- Complete unit test coverage for all modules
- Implement custom assertions and test helpers
- Add performance benchmarks

### Phase 3: Integration Testing
- Implement integration tests for each utility
- Add cross-platform compatibility tests
- Implement end-to-end workflow tests

### Phase 4: Advanced Testing
- Add property-based testing for complex scenarios
- Implement stress testing for large inputs
- Add security and edge case testing

### Phase 5: Optimization and Maintenance
- Optimize test execution performance
- Implement test result analysis and reporting
- Establish ongoing maintenance procedures