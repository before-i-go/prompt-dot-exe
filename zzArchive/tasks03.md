# File-Splitter MVP Requirements

## MVP Test Cases

### 1. Core Functionality Tests
- [x] Test splitting a file into chunks of specified size
- [x] Test default chunk size (1MB) when not specified
- [x] Test custom chunk sizes with different units (K, M, G)
- [x] Test handling of empty input files
- [x] Test handling of files smaller than chunk size
- [x] Test handling of files that are exact multiples of chunk size
- [x] Test handling of files that are not exact multiples of chunk size

### 2. Output Configuration Tests
- [x] Test default output directory (same as input file)
- [x] Test custom output directory
- [x] Test default filename prefix (input filename stem)
- [x] Test custom filename prefix
- [x] Test number padding with different digit counts

### 3. Error Handling Tests
- [x] Test handling of non-existent input file
- [x] Test handling of invalid chunk size (zero)
- [x] Test handling of invalid chunk size (negative)
- [x] Test handling of invalid chunk size (invalid format)
- [x] Test handling of invalid output directory (non-existent, no permission)

## MVP Implementation Status

### Implemented Features
- [x] Basic file splitting functionality
- [x] Support for human-readable size units (K, M, G)
- [x] Configurable output directory
- [x] Configurable filename prefix
- [x] Configurable number of digits for chunk numbering
- [x] Error handling for common cases
- [x] Logging support

## MVP Documentation
- [x] Basic `--help` output
- [ ] Simple README with basic usage examples

## MVP Next Steps
1. Implement missing MVP test cases
2. Add basic usage examples to README
3. Verify all MVP requirements are met