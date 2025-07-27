

# **Product Requirements Document: "Interview Irodov" Code Processing Toolkit MVP**

## **1.0 Introduction**

### **1.1 Project Purpose & Scope**

This document specifies the functional and non-functional requirements for the Minimum Viable Product (MVP) of the "Interview Irodov" project. This project is a suite of command-line utilities, implemented as a Rust Cargo workspace, designed for code processing, analysis, and archival.1 The scope of this PRD is strictly limited to features confirmed to be implemented by the provided codebase. The primary audience for this document is an automated system (LLM) tasked with generating a comprehensive Test-Driven Development (TDD) plan based on these specifications.

The toolkit provides a collection of focused, high-performance tools that address common developer workflows. These include analyzing the structure of a codebase, preparing code for deployment by compiling and minifying TypeScript, creating comprehensive project archives, and managing large files by splitting them into smaller chunks. The design emphasizes utility, performance, and integration with modern development practices, particularly those involving version control and AI-driven code analysis.1

### **1.2 Core Architectural Principles**

The "Interview Irodov" project is built upon a set of core architectural principles that ensure consistency, maintainability, and extensibility across its component tools. These principles represent foundational, non-functional requirements that govern the overall design and behavior of the system.

* **Monorepo Workspace:** The project is structured as a Cargo workspace, a standard Rust pattern for managing multi-crate projects. This architecture contains multiple independent binary crates (code-archiver, ts-compressor, file-splitter) that share a common utility library (common). This structure promotes significant code reuse for tasks like file system interaction and error handling, ensures consistent behavior across the tool suite, and simplifies dependency management by using a single, workspace-level Cargo.lock file.1  
* **CLI-First Interface:** All system functionalities are exposed to the end-user through command-line interfaces. The project consistently utilizes the clap crate for parsing command-line arguments, which provides a robust and standardized way to define flags, options, and subcommands. This design choice ensures that all tools are easily scriptable, can be integrated into automated workflows such as CI/CD pipelines, and provide users with conventional help messages and argument validation.1  
* **Git-Aware Operations:** The system's tools are designed to be "Git-aware." They shall, where applicable, automatically detect if they are operating within a Git repository. This context is then used to provide more intelligent processing. A primary example of this is the default behavior of respecting a repository's .gitignore files to exclude irrelevant files from processing, a feature that aligns with developer expectations and significantly improves the quality of the output. This integration is a core principle that enhances the utility of the tools in real-world development environments.1

## **2.0 Feature Set: code-archiver Utility**

### **2.1 User Story**

"As a developer, I want to scan a code directory to understand its structure, file sizes, and types, so that I can quickly analyze a project's composition and identify large or numerous files."

### **2.2 Functional Requirements & Acceptance Criteria**

#### **FR-2.2.1 Directory Scanning**

The system MUST accept a root directory path for analysis and recursively scan its contents.

* **AC-2.2.1.1:** The tool shall be invoked via the code-archiver binary executable.1  
* **AC-2.2.1.2:** A root directory for scanning shall be specifiable via a \--dir flag (with a shorthand of \-d). If this flag is not provided, the tool MUST default to using the current working directory (.) as the root.1  
* **AC-2.2.1.3:** The tool MUST recursively traverse all subdirectories within the specified root path to build a complete representation of the project structure. This is accomplished using the walkdir crate or a similar directory traversal mechanism.1  
* **AC-2.2.1.4:** The system MUST validate the provided root path. If the path does not exist or points to a file instead of a directory, the tool MUST exit gracefully and report a clear error to the user. This behavior is validated by the tests in error\_handling\_test.rs, which confirm that CodeArchiver::new returns an ArchiveError::InvalidPath in these scenarios.1

#### **FR-2.2.2 File Filtering**

The system MUST provide multiple, combinable mechanisms for filtering the files to be included in the analysis, allowing for precise control over the output.

* **AC-2.2.2.1 (Include Globs):** The \--include flag shall accept one or more glob patterns as arguments. When provided, only files whose paths match at least one of these patterns will be included in the final output. This allows for targeted analysis of specific file types or locations within a project.1  
* **AC-2.2.2.2 (Exclude Globs):** The \--exclude flag shall accept one or more glob patterns as arguments. Any file whose path matches one of these patterns will be explicitly excluded from the output, even if it matches an include pattern. This is useful for filtering out test files, documentation, or other specific assets. The functionality and correctness of glob pattern matching are verified by the glob\_pattern\_test.rs test suite.1  
* **AC-2.2.2.3 (Extension Filter):** The \--extensions flag shall accept a comma-separated list of file extensions (e.g., js,ts,json). When this flag is used, only files with one of the specified extensions will be included in the analysis. This provides a simple and convenient alternative to more complex glob patterns for common use cases.1  
* **AC-2.2.2.4 (Size Filter):** The \--min-size and \--max-size flags shall accept integer values representing bytes. These flags enable filtering of files based on their size, allowing users to find files that are either smaller than the minimum threshold or larger than the maximum threshold.1  
* **AC-2.2.2.5 (Hidden Files):** By default, the tool MUST exclude hidden files and directories (those whose names begin with a .) from the scan. The \--hidden flag MUST override this default behavior and force the inclusion of these files in the analysis.1

#### **FR-2.2.3 Git Integration**

When operating within a directory that is part of a Git repository, the system MUST use repository information to refine its analysis and provide richer context.

* **AC-2.2.3.1 (Status Reporting):** The \--git flag shall enable the reporting of the Git status for each file in the output. The status can include states such as Modified, Added, Deleted, Untracked, or Ignored. This functionality is implemented using the git2 crate and the logic encapsulated within code-archiver/src/git.rs.1  
* **AC-2.2.3.2 (Default .gitignore Adherence):** The tool MUST, by default, detect and respect the rules defined in the repository's .gitignore files (and global gitignore configurations). Files and directories matching these rules MUST be automatically excluded from the analysis. This behavior is a core feature and is validated by git\_integration\_test.rs.1  
* **AC-2.2.3.3 (Default Directory Exclusions):** Independent of .gitignore rules, the tool MUST automatically exclude common directories that are typically not part of a project's source code. This list includes, at a minimum, .git/, node\_modules/, and target/. This default behavior ensures a cleaner, more relevant output for typical code analysis tasks.1

#### **FR-2.2.4 Output Formatting**

The system MUST support multiple output formats to cater to both human and machine consumption.

* **AC-2.2.4.1 (Text Format):** The default output format, triggered by \--format text or by omitting the flag, MUST be a human-readable, tree-like view of the directory structure. This view shall display the relative path of each included file along with its size in a human-friendly format (e.g., KB, MB).1  
* **AC-2.2.4.2 (JSON Format):** The \--format json flag MUST produce a machine-readable JSON output. The output shall be a JSON array where each element is an object representing a file. Based on the FileEntry struct defined in code-archiver/src/lib.rs, each object MUST contain the following keys: path (string), size (u64), modified (string, in RFC3339 format), and extension (string, optional). If the \--git flag is enabled, the object MUST also include the git\_status key (string, optional).1

### **2.3 Table: code-archiver CLI Reference**

| Flag | Shorthand | Type | Required | Default | Description |
| :---- | :---- | :---- | :---- | :---- | :---- |
| \--dir | \-d | Path | No | . | Specifies the root directory to scan. |
| \--include | \-i | String (Vec) | No | N/A | Glob pattern for files to include. Can be used multiple times. |
| \--exclude | \-x | String (Vec) | No | N/A | Glob pattern for files to exclude. Can be used multiple times. |
| \--extensions | \-e | String (CSV) | No | N/A | Comma-separated list of file extensions to include. |
| \--min-size | N/A | u64 | No | N/A | Minimum file size in bytes for inclusion. |
| \--max-size | N/A | u64 | No | N/A | Maximum file size in bytes for inclusion. |
| \--hidden | \-H | Flag | No | false | Includes hidden files (starting with .) in the scan. |
| \--git | N/A | Flag | No | false | Enables Git integration to show file status. |
| \--no-gitignore | N/A | Flag | No | false | Disables the default behavior of respecting .gitignore files. |
| \--format | \-f | String | No | text | Sets the output format. Supported values: text, json. |
| \--verbose | \-v | Flag | No | false | Enables verbose logging output. |

## **3.0 Feature Set: ts-compressor Utility**

The ts-compressor utility is a dual-functionality tool that operates via two distinct subcommands: compress and archive. The compress command serves as a build tool for TypeScript projects, while the archive command is a sophisticated data preparation tool designed for creating clean, LLM-ready codebase archives. This clear separation of concerns is critical for understanding the tool's purpose and for generating an effective test plan.

### **3.1 Sub-feature: compress Command**

#### **3.1.1 User Story**

"As a web developer, I want to compile and minify my TypeScript/JSX project into optimized JavaScript, so that I can prepare it for production deployment."

#### **3.1.2 Functional Requirements & Acceptance Criteria**

* **FR-3.1.2.1: Invocation:** The functionality MUST be invoked via the compress subcommand of the ts-compressor binary (e.g., cargo run \-p ts-compressor \-- compress...).1  
* **FR-3.1.2.2: Input/Output Specification:** The command MUST accept two required positional arguments in order: input\_dir and output\_dir. The input\_dir specifies the source directory containing the TypeScript files, and output\_dir specifies the destination for the compiled JavaScript files.1  
* **FR-3.1.2.3: File Processing:** The system MUST recursively scan the input\_dir and identify all files with .ts (TypeScript) and .tsx (TypeScript with JSX) extensions for processing.1  
* **FR-3.1.2.4: Transformation Logic:** For each identified file, the system MUST perform a series of transformations using the swc\_core library. This process includes parsing the TypeScript syntax, stripping all type annotations, and applying minification to the resulting JavaScript code to reduce its size.1  
* **FR-3.1.2.5: Output Generation:** The transformed and minified JavaScript content MUST be written to a corresponding file within the specified output\_dir. The original filename must be preserved, but the file extension must be changed to .js.1

### **3.2 Sub-feature: archive Command**

#### **3.2.1 User Story**

"As a machine learning engineer, I want to create a clean, text-based archive of a single project directory, excluding all irrelevant files, so that I can use it as a high-quality dataset for training a Large Language Model."

#### **3.2.2 Functional Requirements & Acceptance Criteria**

* **FR-3.2.2.1: Invocation:** The functionality MUST be invoked via the archive subcommand of the ts-compressor binary (e.g., cargo run \-p ts-compressor \-- archive...).1  
* **FR-3.2.2.2: Target Specification:** The command MUST accept one required positional argument: target\_folder, which specifies the root directory of the project to be archived.1  
* **FR-3.2.2.3: Output Format:** The system MUST generate a single, self-contained text file representing the entire project archive.  
  * **AC-3.2.2.3.1:** The output filename MUST be automatically generated and follow the format \<folder\_name\>-\<timestamp\>.txt, where the timestamp corresponds to the moment of creation in YYYYMMDDHHMMSS format. This ensures that each archive has a unique name.1  
  * **AC-3.2.2.3.2:** The output file's content MUST begin with a header that includes a tree-like representation of the project's directory structure, providing a manifest of the archived files.1  
  * **AC-3.2.2.3.3:** Following the header, the content of each included file MUST be concatenated into the archive. Each file's content must be clearly delimited by a header line specifying its absolute path (e.g., Absolute path: /path/to/file.ts) and enclosed by start and end markers (e.g., \<text starts\> and \<text ends\>).1  
* **FR-3.2.2.4: LLM Optimization Filtering (Default Behavior):** The system MUST, by default, apply an extensive set of filters designed to exclude files and directories that are generally considered irrelevant for LLM training datasets. This is the core value proposition of the archive command.1  
  * **AC-3.2.2.4.1:** The \--no-llm-optimize command-line flag MUST disable this default filtering behavior, causing the archiver to include a much wider set of files.1  
  * **AC-3.2.2.4.2:** The categories of files and directories excluded by this mode are extensive, numbering over 270 patterns. These are defined within the get\_llm\_ignore\_patterns function and MUST cover categories such as build artifacts (target/, build/), dependencies (node\_modules/), cache files, IDE configurations, OS-specific files, secrets (.env), binary media files, data files, and package manager lock files.1  
* **FR-3.2.2.5: Custom Filtering:** The system MUST provide mechanisms for the user to override or augment the default filtering logic.  
  * **AC-3.2.2.5.1:** The \--ignore-pattern flag, which can be specified multiple times, shall accept glob patterns. Any file or directory matching one of these patterns MUST be excluded.1  
  * **AC-3.2.2.5.2:** The \--include-extensions flag shall accept a comma-separated list of file extensions. If provided, only files with these extensions will be considered for inclusion in the archive.1  
* **FR-3.2.2.6: Binary File Handling:** The system MUST intelligently handle binary files to prevent corruption of the text-based archive.  
  * **AC-3.2.2.6.1:** The tool MUST use a mechanism like the mime\_guess crate to detect whether a file is likely to be binary or text-based.1  
  * **AC-3.2.2.6.2:** If a file is identified as binary, its content MUST NOT be written to the archive. Instead, a placeholder message (e.g., "Binary file, content not included.") MUST be inserted in its place.1  
* **FR-3.2.2.7: Statistics Reporting:** The system MUST, by default, provide a summary of its filtering actions to the user upon completion.  
  * **AC-3.2.2.7.1:** The statistics report printed to the console MUST include the total number of files found, the number of files included, and the number of files excluded. It should also provide a breakdown of the reasons for exclusion (e.g., filtered by LLM optimization, by custom pattern, by Git rules).1  
  * **AC-3.2.2.7.2:** The \--no-filter-stats command-line flag MUST suppress the printing of this statistical summary.1

### **3.3 Table: LLM Optimization Exclusion Categories**

The default LLM Optimization feature is implemented via a comprehensive set of exclusion patterns. For the purpose of generating a high-level test plan, these patterns can be grouped into the following semantic categories. The TDD plan should include test cases that create representative files/directories for each category and verify their exclusion.

| Category | Description | Example Patterns |
| :---- | :---- | :---- |
| Build Artifacts | Compiled code, libraries, and build tool outputs. | target/, build/, dist/, \*.exe, \*.class, \*.o |
| Dependencies | External packages and libraries managed by package managers. | node\_modules/, vendor/, venv/, packages/ |
| Lock Files | Generated files that lock dependency versions. | package-lock.json, Cargo.lock, yarn.lock |
| Cache & Temp Files | Temporary files and caches generated by tools or editors. | .cache/, \*.tmp, \*.swp, \*\~ |
| IDE/Editor Config | Configuration files specific to a developer's environment. | .vscode/, .idea/, \*.sublime-\* |
| OS-Generated Files | System-specific metadata files. | .DS\_Store, Thumbs.db, desktop.ini |
| Secrets & Keys | Files containing sensitive information like API keys or passwords. | .env, \*.key, secrets.json, \*.pem |
| Media & Binary Files | Non-text files such as images, videos, and audio. | \*.png, \*.jpg, \*.mp4, \*.mp3, \*.zip, \*.pdf |
| Data & Model Files | Large datasets, database files, or machine learning models. | \*.csv, \*.db, \*.sqlite, \*.pkl, \*.weights |
| Logs & Reports | Runtime logs and test coverage reports. | \*.log, coverage/, lcov.info |
| Version Control | Metadata directories for version control systems. | .git/, .hg/, .svn/ |

## **4.0 Feature Set: file-splitter Utility**

### **4.1 User Story**

"As a data analyst, I want to split a massive log file into smaller, manageable chunks with predictable naming, so that I can process them in parallel or with tools that have memory limitations."

### **4.2 Functional Requirements & Acceptance Criteria**

* **FR-4.2.1: Invocation:** The tool MUST be invoked via the file-splitter binary executable.1  
* **FR-4.2.2: Input Specification:** The \--input flag MUST be a required command-line argument that specifies the path to the file to be split.1  
* **FR-4.2.3: Chunk Size Configuration:** The \--chunk-size flag MUST define the maximum size of each output chunk file.  
  * **AC-4.2.3.1:** The flag's value must be a string that supports human-readable size units, including bytes (no suffix), kilobytes (K/KB), megabytes (M/MB), and gigabytes (G/GB). The parse\_size function in file-splitter/src/main.rs confirms the implementation of this parsing logic.1  
  * **AC-4.2.3.2:** If the \--chunk-size flag is omitted, its value MUST default to 1M (1 Megabyte).1  
* **FR-4.2.4: Output Configuration:** The system MUST provide a flexible set of options to control the location and naming convention of the generated output files.  
  * **AC-4.2.4.1 (Output Directory):** The \--output-dir flag shall specify the directory where the output chunk files will be saved. If this flag is omitted, the tool MUST default to saving the chunks in the same directory as the input file.1  
  * **AC-4.2.4.2 (Filename Prefix):** The \--prefix flag shall specify a custom string to be used as the base name for the output files. If this flag is omitted, the tool MUST default to using the filename stem (the name without the extension) of the input file as the prefix.1  
  * **AC-4.2.4.3 (Number Padding):** The \--digits flag shall specify the number of digits to use for the zero-padded sequential number appended to each chunk's filename. If this flag is omitted, it MUST default to 3 digits.1

## **5.0 Non-Functional Requirements (NFR)**

### **NFR-5.1 Determinism**

For a given set of inputs and a static file system state, all tools within the "Interview Irodov" suite MUST produce deterministic output. The content of archives generated by ts-compressor archive must be identical between runs (excluding the timestamp in the filename and any timestamps within the file content itself). This is critical for ensuring reproducibility and for use in automated testing and CI/CD environments.

### **NFR-5.2 Logging**

All tools MUST implement structured logging using the tracing crate. The verbosity of the logging output MUST be configurable at runtime. This is achieved through a \--log-level flag on each tool, which accepts values such as info, debug, or trace. Additionally, the logging level MUST be configurable via the standard RUST\_LOG environment variable, which takes precedence if set.1

### **NFR-5.3 Error Handling**

All tools MUST handle failure cases gracefully and provide clear, contextual error messages to the user. This includes, but is not limited to, errors from file system operations (e.g., file not found, permission denied), invalid user input (e.g., malformed arguments), and internal processing errors (e.g., code parsing failures). The consistent use of the anyhow and thiserror crates across the workspace indicates a commitment to robust error handling and propagation.1

### **NFR-5.4 Performance**

As the tools are implemented in Rust, they are expected to be highly performant. While no specific quantitative benchmarks are defined for the MVP, the tools should be capable of handling multi-megabyte files and directories containing thousands of files in a timeframe that is reasonable for interactive use on standard developer hardware. The implementation should avoid unnecessary memory allocations and use efficient algorithms for file traversal and processing.

### **NFR-5.5 Consistency**

All tools in the suite MUST exhibit consistent behavior for common operations, particularly file system access, path manipulation, and error reporting. This consistency is enforced architecturally through the use of the shared common library crate, which centralizes these utilities. This ensures a predictable user experience and simplifies maintenance and future development.1

## **6.0 Appendix: Consolidated CLI Command Reference**

This table provides a comprehensive, consolidated reference for the command-line interface of all tools within the "Interview Irodov" workspace. It is intended to serve as a single source of truth for the system's public API, facilitating the generation of a complete TDD plan.

| Tool | Command | Argument/Flag | Type | Required | Default | Description |
| :---- | :---- | :---- | :---- | :---- | :---- | :---- |
| code-archiver | (root) | \--dir \<PATH\> | Path | No | . | Specifies the root directory to scan. |
| code-archiver | (root) | \--include \<GLOB\> | String (Vec) | No | N/A | Glob pattern for files to include. |
| code-archiver | (root) | \--exclude \<GLOB\> | String (Vec) | No | N/A | Glob pattern for files to exclude. |
| code-archiver | (root) | \--extensions \<EXTS\> | String (CSV) | No | N/A | Comma-separated list of extensions to include. |
| code-archiver | (root) | \--min-size \<BYTES\> | u64 | No | N/A | Minimum file size in bytes for inclusion. |
| code-archiver | (root) | \--max-size \<BYTES\> | u64 | No | N/A | Maximum file size in bytes for inclusion. |
| code-archiver | (root) | \--hidden | Flag | No | false | Includes hidden files in the scan. |
| code-archiver | (root) | \--git | Flag | No | false | Enables Git integration to show file status. |
| code-archiver | (root) | \--no-gitignore | Flag | No | false | Disables respecting .gitignore files. |
| code-archiver | (root) | \--format \<FORMAT\> | String | No | text | Sets output format (text or json). |
| code-archiver | (root) | \--verbose | Flag | No | false | Enables verbose logging. |
| ts-compressor | compress | input\_dir | Path | Yes | N/A | Input directory with TypeScript files. |
| ts-compressor | compress | output\_dir | Path | Yes | N/A | Output directory for JavaScript files. |
| ts-compressor | compress | \--log-level \<LEVEL\> | String | No | info | Sets the logging level. |
| ts-compressor | archive | target\_folder | Path | Yes | N/A | The project directory to archive. |
| ts-compressor | archive | \--output-dir \<PATH\> | Path | No | Parent of target | Directory to save the archive file. |
| ts-compressor | archive | \--no-llm-optimize | Flag | No | false | Disables default LLM-centric file filtering. |
| ts-compressor | archive | \--ignore-pattern \<GLOB\> | String (Vec) | No | N/A | Custom glob pattern for excluding files. |
| ts-compressor | archive | \--include-extensions \<E\> | String (CSV) | No | N/A | Comma-separated list of extensions to include. |
| ts-compressor | archive | \--no-filter-stats | Flag | No | false | Hides the filtering statistics summary. |
| ts-compressor | archive | \--log-level \<LEVEL\> | String | No | info | Sets the logging level. |
| file-splitter | (root) | \--input \<FILE\> | Path | Yes | N/A | The input file to be split. |
| file-splitter | (root) | \--output-dir \<DIR\> | Path | No | Input dir | Directory to save the output chunks. |
| file-splitter | (root) | \--chunk-size \<SIZE\> | String | No | 1M | Size of each chunk (e.g., 10K, 50M). |
| file-splitter | (root) | \--prefix \<PREFIX\> | String | No | Input stem | Prefix for output chunk filenames. |
| file-splitter | (root) | \--digits \<NUM\> | u8 | No | 3 | Number of digits for chunk numbering. |
| file-splitter | (root) | \--verbose | Flag | No | false | Enables verbose logging. |

#### **Works cited**

1. before-i-go-interview-irodov.txt