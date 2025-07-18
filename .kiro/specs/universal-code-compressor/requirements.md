# Universal Code Compressor - Requirements Document

## Introduction

The Universal Code Compressor is a focused enhancement to the existing `ts-compressor` tool that implements frequency-based dictionary compression to achieve maximum codebase size reduction. This MVP feature transforms any software project into a highly compressed, single-file representation using intelligent pattern recognition and hexadecimal token replacement.

The primary use case is **maximum compression for archival and transfer** - taking any codebase and creating the smallest possible text file representation while maintaining perfect reconstruction capability. The system analyzes code patterns, builds frequency-based replacement dictionaries, and applies aggressive compression techniques to achieve compression ratios significantly better than traditional methods.

## Requirements

### Requirement 1: Single Command Maximum Compression

**User Story:** As a developer, I want to compress any codebase with a single command using frequency-based dictionary compression, so that I can achieve maximum size reduction for archival and transfer.

#### Acceptance Criteria

1. WHEN I execute `universal_code_compressor <input_folder>` THEN the system SHALL process the entire folder recursively using frequency-based dictionary compression
2. WHEN the compression completes THEN the system SHALL create a file named `{folder_name}_{timestamp}.txt` in the parent directory of the input folder
3. WHEN the input folder does not exist THEN the system SHALL display a clear error message and exit gracefully
4. WHEN the parent directory is not writable THEN the system SHALL fall back to the current working directory
5. WHEN the compression process encounters errors THEN the system SHALL log detailed error information while continuing with remaining files

### Requirement 2: Frequency-Based Dictionary Compression

**User Story:** As a developer, I want the system to automatically identify and replace frequently occurring code patterns with compact hexadecimal tokens, so that I can achieve maximum compression on codebases with high repetition.

#### Acceptance Criteria

1. WHEN analyzing the codebase THEN the system SHALL build frequency maps for strings, identifiers, and code patterns of length 4+ characters
2. WHEN a pattern occurs 3 or more times THEN the system SHALL assign it a hexadecimal replacement token (starting with shortest: A0, A1, A2... AA, AB...)
3. WHEN generating replacements THEN the system SHALL prioritize shorter tokens for more frequent patterns
4. WHEN creating the dictionary THEN the system SHALL ensure all replacements are reversible and collision-free
5. WHEN writing the output THEN the system SHALL embed the dictionary at the top in format `DICT:original_pattern=hex_token`
6. WHEN compression completes THEN the system SHALL apply final zstd compression to the entire output

### Requirement 3: Git-Aware Processing

**User Story:** As a developer, I want the compressor to respect my project's version control settings, so that it only processes relevant files and respects my .gitignore rules.

#### Acceptance Criteria

1. WHEN the input folder is a Git repository THEN the system SHALL respect .gitignore rules by default
2. WHEN processing Git repositories THEN the system SHALL include both tracked and untracked (but not ignored) files
3. WHEN Git metadata is unavailable THEN the system SHALL process all files in the directory tree
4. WHEN .gitignore parsing fails THEN the system SHALL log a warning and continue with full processing

### Requirement 4: Output Format

**User Story:** As a developer, I want the compressed output to contain all necessary information for perfect reconstruction, so that I can reliably restore the original codebase structure and content.

#### Acceptance Criteria

1. WHEN generating output THEN the system SHALL include a complete directory structure manifest
2. WHEN using dictionary compression THEN the system SHALL embed the complete replacement dictionary at the top
3. WHEN compression completes THEN the system SHALL include compression statistics (original size, compressed size, ratio)
4. WHEN the output file is created THEN the system SHALL verify its integrity before completion

### Requirement 5: Multi-Stage Pipeline Processing

**User Story:** As a developer, I want the compression process to utilize multiple CPU cores through pipeline parallelism, so that I can achieve faster processing times on large codebases.

#### Acceptance Criteria

1. WHEN processing large codebases (1000+ files) THEN the system SHALL utilize pipeline parallelism with concurrent stages
2. WHEN multiple files are available THEN the system SHALL process them through concurrent pipeline stages
3. WHEN pattern analysis is CPU-intensive THEN the system SHALL distribute analysis work across available CPU cores
4. WHEN applying compression THEN the system SHALL parallelize pattern replacement across multiple threads
5. WHEN pipeline stages have different throughput THEN the system SHALL use bounded channels to prevent memory overflow
6. WHEN processing completes THEN the system SHALL maintain identical compression results to sequential processing

### Requirement 6: Memory-Mapped File Processing

**User Story:** As a developer, I want the system to efficiently handle large codebases using memory-mapped I/O, so that memory usage remains bounded while maximizing throughput.

#### Acceptance Criteria

1. WHEN processing files larger than 1MB THEN the system SHALL use memory-mapped I/O for efficient access
2. WHEN memory mapping files THEN the system SHALL process chunks in parallel without loading entire files into memory
3. WHEN pattern analysis occurs THEN the system SHALL use zero-copy string processing where possible
4. WHEN memory is constrained THEN the system SHALL use streaming processing with bounded memory usage
5. WHEN very large files are encountered THEN the system SHALL chunk them for parallel processing
6. WHEN memory mapping fails THEN the system SHALL fall back to traditional file I/O gracefully

### Requirement 7: Resource-Aware Progress Indication

**User Story:** As a developer, I want real-time progress feedback with system resource monitoring during compression, so that I can track progress and ensure the process doesn't overwhelm my system.

#### Acceptance Criteria

1. WHEN compression starts THEN the system SHALL display a progress bar showing overall completion percentage
2. WHEN processing pipeline stages THEN the system SHALL show current stage name and stage-specific progress
3. WHEN system resources are under pressure THEN the system SHALL display CPU and memory usage warnings
4. WHEN processing large files THEN the system SHALL show file count progress and current file being processed
5. WHEN compression takes longer than 30 seconds THEN the system SHALL display estimated time remaining
6. WHEN system load is high THEN the system SHALL offer to throttle processing to reduce resource impact
7. WHEN progress updates occur THEN the system SHALL refresh the display without scrolling or cluttering the terminal