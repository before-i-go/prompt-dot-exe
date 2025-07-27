# Requirements Document

## Introduction

The "Interview Irodov" Code Processing Toolkit is a Rust Cargo workspace containing three command-line utilities: code-archiver, ts-compressor, and file-splitter. Analysis shows that most core functionality is already implemented with existing tests. This MVP completion plan focuses on finishing the remaining critical features to reach a functional release quickly.

## Requirements

### Requirement 1

**User Story:** As a developer completing the Interview Irodov MVP, I want to finish the missing TS-Compressor archive functionality, so that all three utilities are feature-complete.

#### Acceptance Criteria

1. WHEN the ts-compressor archive command is invoked THEN it SHALL create a single text file archive of the target project
2. WHEN LLM optimization is enabled (default) THEN it SHALL exclude files matching the 270+ predefined patterns
3. WHEN custom filtering options are provided THEN it SHALL respect ignore patterns and extension filters
4. WHEN binary files are encountered THEN it SHALL detect them and insert placeholder messages
5. WHEN the archive is complete THEN it SHALL provide statistics on files processed, included, and excluded

### Requirement 2

**User Story:** As a developer preparing for MVP release, I want comprehensive CLI integration tests, so that all tools work correctly as standalone binaries.

#### Acceptance Criteria

1. WHEN each tool is executed from the command line THEN it SHALL parse arguments correctly and provide helpful error messages
2. WHEN invalid arguments are provided THEN tools SHALL exit gracefully with appropriate error codes and messages
3. WHEN help is requested THEN tools SHALL display comprehensive usage information
4. WHEN tools are used in realistic workflows THEN they SHALL integrate seamlessly with shell scripts and CI/CD pipelines
5. WHEN tools process real-world data THEN they SHALL complete successfully within reasonable time and memory limits

### Requirement 3

**User Story:** As a developer ensuring MVP quality, I want end-to-end integration tests, so that the complete toolkit works together in realistic scenarios.

#### Acceptance Criteria

1. WHEN multiple tools are used in sequence THEN they SHALL work together without conflicts or data corruption
2. WHEN tools process the same project THEN they SHALL produce consistent results across runs
3. WHEN tools are used on different platforms THEN they SHALL behave identically on Windows, macOS, and Linux
4. WHEN tools encounter edge cases THEN they SHALL handle them gracefully without crashes or data loss
5. WHEN tools are used in automated environments THEN they SHALL provide reliable exit codes and error reporting