# 🧙‍♂️ Interview Irodov

*"Ah, my dear student of the digital arts! Within these virtual halls lies a most curious collection of knowledge - both the ancient runes of interview preparation and a rather nifty collection of magical coding artifacts. Do tread carefully, for wisdom and magic await those who seek them."*

*Albus Dumbledore's Repository of Magical Coding Artifacts and Interview Enchantments*

## 🚀 Quick Start (No Installation Needed)

### 1. Compress TypeScript/JavaScript Files
```bash
# From repository root
cargo run --release -p ts-compressor -- compress ./src ./dist
```

### 2. Create a Code Archive (Great for LLM Context)
```bash
# Creates a text file with all your source code
cargo run --release -p ts-compressor -- archive ./your-project
```

### 3. Explore Project Structure
```bash
# View project files with sizes and types
cargo run --release -p code-archiver -- --root ./your-project
```

## 📋 Table of Contents
- [Quick Start](#-quick-start)
- [Core Tools](#-core-tools)
  - [TypeScript Compressor](#-typescript-compressor)
  - [Code Archiver](#-code-archiver)
  - [File Splitter](#-file-splitter)
- [Learning Resources](#-learning-resources)
- [Development](#-development)
- [License](#-license)

## 🛠 Installation (Optional)

Only needed if you want to install the tools globally:

```bash
# Clone the repository
git clone https://github.com/yourusername/interview-irodov.git
# Navigate into the project directory
cd interview-irodov
# Build all tools in release mode (optimized for speed)
cargo build --release

# Install ts-compressor globally (optional, adds 'ts-compressor' to your PATH)
cargo install --path ts-compressor
# Install code-archiver globally (optional, adds 'code-archiver' to your PATH)
cargo install --path code-archiver
```

## 🔧 Core Tools

### 📦 TypeScript Compressor

#### Basic Usage
```bash
# Compress TypeScript files
# -p ts-compressor: Select the ts-compressor package
# compress: The compression command
# ./src: Input directory containing TypeScript files
# ./dist: Output directory for minified JavaScript files
cargo run --release -p ts-compressor -- compress ./src ./dist

# Create a text archive of your project
# archive: The archive command
# ./your-project: Target folder to archive
cargo run --release -p ts-compressor -- archive ./your-project
```

#### Advanced Options
```bash
# Create an archive with custom options
# --output-dir: Specify output directory (defaults to parent of target)
# --ignore-pattern: Exclude files matching the glob pattern
# --include-extensions: Only include specific file extensions
cargo run --release -p ts-compressor -- archive ./project \
    --output-dir ./archives \
    --ignore-pattern "**/__tests__/**" \
    --ignore-pattern "**/node_modules/**" \
    --include-extensions rs,js,ts,py
```

### 🗃 Code Archiver

#### Basic Usage
```bash
# View project structure
# -d, --dir: Specify the directory to analyze (defaults to current directory)
# -v: Enable verbose output
cargo run --release -p code-archiver -- -d ./your-project -v

# Filter files by extension and exclude patterns
# -e, --extensions: Only show files with these extensions
# -e, --exclude: Exclude files matching the pattern
cargo run --release -p code-archiver -- -d ./your-project \
    --extensions rs,toml,md \
    --exclude "**/target/**" \
    --format json
```

### ✂️ File Splitter

```bash
# Split large files into smaller chunks
# -i, --input: Input file to split
# -o, --output-dir: Output directory (default: same as input file)
# -c, --chunk-size: Size of each chunk (e.g., 10M for 10MB)
# -p, --prefix: Custom prefix for output files
cargo run --release -p file-splitter -- \
    --input large-file.txt \
    --output-dir ./chunks \
    --chunk-size 10M \
    --prefix "part_" \
    --digits 4
```

## 📚 Learning Resources
- **Rust Idioms**: `impRustIdioms/` directory
- **Code Examples**: Explore the `examples/` directory

## 📜 Table of Contents

- [Project Components](#-project-components)
- [Core Tools](#-core-tools)
  - [The Marauder's Compressor](#-the-marauders-compressor) - Taming the wild TypeScript
  - [The Code Archiver](#-the-code-archiver) - Preserving your magical code repositories
  - [File Splitter](#-file-splitter) - Divide and conquer large files
- [Learning Resources](#-learning-resources)
  - [Rust Idioms](#-rust-idioms) - Master Rust patterns and best practices
- [Development](#-development)
  - [Building from Source](#-building-from-source)
  - [Running Tests](#-running-tests)
  - [Contributing](#-contributing)
- [License](#-license)

## 🧰 The Marauder's Compressor

*"A most ingenious contraption, wouldn't you agree? With a wave of your terminal and the right incantation, it transforms verbose TypeScript into something rather more... compact. Much like how Fawkes can fit into a small cage, yet remain a magnificent phoenix at heart."*

### 🎯 Features

- **TypeScript Compression**: Strip types and minify your TypeScript code
- **Smart Filtering**: Automatically excludes build artifacts and dependencies
- **Git Integration**: Respects `.gitignore` rules by default
- **LLM Optimization**: Prepares your code for AI analysis
- **Multiple Output Formats**: Text and JSON formats supported

### 🚀 Quick Run (No Installation Needed)

Run the tool directly using Cargo - no installation required:

```bash
# Compress TypeScript files from src/ to dist/
# This processes .ts and .tsx files, removing type annotations
# and minifying while preserving functionality
cargo run --release -p ts-compressor -- compress src/ dist/

# Create a text-based archive of the project
# Useful for sharing code with LLMs or documentation
cargo run --release -p ts-compressor -- archive my-project

# Create an archive with exclusions
# Use glob patterns to exclude temporary and test files
cargo run --release -p ts-compressor -- archive my-project \
    --ignore-pattern "*.tmp" \
    --ignore-pattern "test_*" \
    --ignore-pattern "**/__tests__/**"
```

### 🏗 Building for Production

If you want to install it globally:
```bash
cargo install --path ts-compressor
# Then you can run it directly:
ts-compressor --help
```

### 📊 Example Output

```
📊 File Filtering Statistics:
   Total files found: 247
   Files included: 23 🟢
   Files excluded: 224 🔴
   Inclusion rate: 9.3% 📈
   Total size included: 1.2 MB 💾
```

### 📜 The Spellbook of Commands

**The Condensing Charm (TypeScript Edition):**
```bash
./target/release/ts-compressor compress src/ dist/
```

*"A clever bit of magic that transforms your verbose TypeScript into something more... portable. It handles both `.ts` and `.tsx` scrolls, stripping away the type annotations and compressing the rest - much like how one might summarize a particularly long-winded prophecy."*

**The Archive Charm (Project Preservation Spell):**
```bash
./target/release/ts-compressor archive my-project
```

Sample output creates: `my-project-20250118142033.txt`

```
Git repository detected. Will respect .gitignore rules.

Directory structure:
├── src
│   ├── main.ts
│   └── utils.ts
├── package.json
└── README.md

Processing files...
🤖 LLM optimization enabled - excluding build artifacts and dependencies

Absolute path: /home/user/my-project/src/main.ts
<text starts>
interface User {
    name: string;
    age: number;
}
<text ends>

Absolute path: /home/user/my-project/package.json
<text starts>
{
  "name": "my-project",
  "version": "1.0.0"
}
<text ends>

📊 File Filtering Statistics:
   Total files found: 247
   Files included: 23 🟢
   Files excluded: 224 🔴
     └─ By LLM optimization: 224 🤖
   Inclusion rate: 9.3% 📈
   Total size included: 1.2 MB 💾

Archive created: "my-project-20250118142033.txt"
```

### 🏰 Default Enchantments

- **The Wisdom of the Ancients (LLM Optimization)**: Like the Room of Requirement, it knows what to hide - build artifacts, dependencies, and cache files vanish from sight
- **The Keeper's Memory (Git Integration)**: Respects the sacred `.gitignore` scrolls, just as we respect the boundaries of the Forbidden Forest
- **The Revealing Charm (Binary Detection)**: Spots and excludes binary files with the precision of a Niffler spotting gold
- **The Time-Turner Feature**: Creates uniquely timestamped archives, because even wizards need to keep track of their past exploits

### Command Options

```bash
# Disable LLM optimization (includes all files)
ts-compressor archive my-project --no-llm-optimize

# Custom ignore patterns
ts-compressor archive my-project --ignore-pattern "*.tmp" --ignore-pattern "test_*"

# Filter by extensions
ts-compressor archive my-project --include-extensions rs,js,ts,md

# Hide filtering statistics
ts-compressor archive my-project --no-filter-stats

# Custom output directory
ts-compressor archive my-project --output-dir ./archives
```

### 🚫 The Forbidden Files (Vanished by LLM Optimization)

*"Even the most powerful wizards know that not all files are created equal. These are banished to the depths of the Room of Requirement, never to trouble your archives:"*

- **The Build Cauldron's Residue**: `target/`, `build/`, `dist/`, `*.exe`, `*.dll`
- **Dependency Demons**: `node_modules/`, `vendor/`, `venv/` (No need to carry around other wizards' spellbooks)
- **Temporal Echoes**: `.cache/`, `*.tmp`, `*.bak` (The Pensieve has its limits)
- **Muggle Artifacts**: `.DS_Store`, `Thumbs.db` (We must respect the Statute of Secrecy)
- **Moving Portraits**: `*.png`, `*.jpg`, `*.mp4` (Alas, they don't move in text form)
- **Binding Contracts**: `package-lock.json`, `Cargo.lock` (Some things are better left unbound)

### 🌟 Practical Applications for the Discerning Wizard

*"While not quite as versatile as a wand that can turn teacups into turtles, this tool serves several rather useful purposes:"*

- **The Pensieve Effect**: Create complete project snapshots in text format for later perusal
- **Occlumency for Code**: Prepare your spells—er, code—for LLM analysis by removing the mental clutter
- **The Shrinking Solution**: Minify TypeScript for deployment (without the unfortunate side effects of actual shrinking)
- **The Mirror of Erised**: Review and document your code to see it as it truly is, not as you wish it to be
- **The Vanishing Cabinet**: Safely archive project states, ready to be recalled at a moment's notice

## 🧪 Experimental Charms (Testing)

*"Before unleashing any magical artifact upon the world, one must first test it thoroughly. The Department of Mysteries suggests the following incantations:"*

```bash
cd ts-compressor
cargo test  # The Standard Book of Spells, Testing Edition
cargo run -- --help  # Consult the ancient scrolls
cargo run -- archive ../test-input  # A small sacrifice to the testing gods
```

## 🏰 The Code Archiver

*"A most ingenious magical artifact for the modern witch or wizard! This enchanted tool allows you to capture and organize your magical code repositories with the flick of a wand—or rather, the press of a key. It's particularly useful for preparing your potions—er, projects—for the Wizarding Code Review Board."*

### ✨ Features

- **Smart File Filtering**: Include/exclude files using glob patterns
- **Git Integration**: Track file status in Git repositories
- **Size-based Filtering**: Filter files by size
- **Multiple Output Formats**: JSON and plain text output
- **Detailed Logging**: Built-in logging for debugging

### 🚀 Installation

```bash
# Build from source
cd code-archiver
cargo install --path .
```

### 🛠 Usage

```bash
# Basic usage: Scan all files in the my-project directory
# Outputs a tree view of the project structure with file sizes
code-archiver --root ./my-project

# With Git integration: Shows Git status for each file
# (M)odified, (A)dded, (D)eleted, etc.
code-archiver --root ./my-project --git

# Filter to only include Rust source files (*.rs) but exclude test files
# The '**/' matches in all subdirectories
code-archiver \
    --root ./my-project \
    --include '**/*.rs' \
    --exclude '**/test_*.rs' \
    --exclude '**/tests/'

# Filter files by size (in bytes)
# This example shows files between 100 bytes and 10KB
code-archiver \
    --root ./my-project \
    --min-size 100 \
    --max-size 10000
```

### 📂 Example Output

```
📦 Project: my-project
├── src/
│   ├── main.rs (1.2 KB)
│   └── lib.rs (0.8 KB)
└── Cargo.toml (0.3 KB)

Total files: 3 | Total size: 2.3 KB
```

### 🪄 The Archive Charm

```bash
cd code-archiver
cargo build --release  # Conjuring the archive's magical core
```

*"A word of caution: This spell requires Rust 1.70 or later, and a touch of Git magic - though I daresay you've already made their acquaintance."*

### 📜 The Archiver's Grimoire

**Basic Incantation:**
```bash
./target/release/code-archiver archive /path/to/your/repo
```

**With Git Integration (for tracking file statuses):**
```bash
./target/release/code-archiver archive /path/to/your/repo --git
```

**Filtering Spells:**
```bash
# Only include specific file extensions
./target/release/code-archiver archive /path/to/your/repo --extensions rs,toml,md

# Exclude certain directories
./target/release/code-archiver archive /path/to/your/repo --exclude "**/target" --exclude "**/node_modules"

# Filter by file size (in bytes)
./target/release/code-archiver archive /path/to/your/repo --min-size 100 --max-size 10000
```

### 🏆 Features Worthy of the House Cup

- **Git Integration**: Like the Marauder's Map, it reveals the hidden status of your files - tracked, modified, or untracked
- **Smart Filtering**: With the precision of a Seer, it can include or exclude files based on patterns and extensions
- **Size Matters**: Filter files by size, because even magic has its limits
- **JSON Output**: Structured like a well-organized potion ingredients cabinet
- **Respects .gitignore**: Because some things are meant to remain hidden, like the Chamber of Secrets

### 🧪 Testing Your Spells

*"Every great wizard tests their spells before using them in the field. The Department of Mysteries suggests:"*

```bash
cd code-archiver
cargo test  # The Standard Book of Spells, Testing Edition
cargo run -- --help  # Consult the ancient scrolls
```

## 🏰 The Castle Layout

*"Every great wizard's tower has its secrets, and this repository is no exception. Here's what lies within these digital walls:"*

```
├── code-archiver/          # The Pensieve of Code Archiving
│   ├── src/                # The very soul of the archiver
│   ├── tests/              # The Triwizard Tournament (challenges await!)
│   └── Cargo.toml          # The Potion Master's recipe book
├── ts-compressor/          # The Marauder's Map of Code Compression
│   ├── src/main.rs         # The Sorcerer's Stone (core logic)
│   ├── Cargo.toml          # The Potion Master's recipe book
│   └── tests/              # The Triwizard Tournament (challenges await!)
├── test-input/             # The Room of Requirement (for testing)
│   └── example.ts          # A prophecy yet to be fulfilled
├── zzArchive/              # The Restricted Section
│   ├── RailsCrashCours202507.ipynb    # The Tales of Beedle the Bard (Rails edition)
│   └── RustCrashCourse202507.ipynb    # Advanced Rune Studies
├── Unclassified20250706.txt # The Half-Blood Prince's Notes
├── i00-pattern-list.txt    # The Standard Book of Spells (Interview Edition)
└── split_large_file.sh     # The Sword of Gryffindor (for cutting large files)
```

## 🧙‍♂️ Magical Ingredients (Code Archiver Edition)

*"No spell is complete without the right components. These are the enchanted artifacts that make our magic possible:"*

- **swc_core**: The Elder Wand of TypeScript compilation
- **clap**: The Marauder's Map for command-line arguments
- **git2**: A loyal house-elf for Git repository integration
- **walkdir**: The Invisibility Cloak for directory traversal
- **tracing**: The Pensieve for structured logging
- **mime_guess**: The Sorting Hat of file type detection

## 📚 The Restricted Section

*"For your O.W.L.s and N.E.W.T.s in the magical arts of coding, I present these most valuable resources:"*

- `zzArchive/`: The collected works of modern arithmancy (Rails and Rust)
- `Unclassified20250706.txt`: Mysterious prophecies (interview questions) yet to be deciphered
- `i00-pattern-list.txt`: Ancient runes of coding patterns (handy for defeating your technical interviews)
- Various `.md` scrolls containing the collective wisdom of wizards past

*"Remember, it does not do to dwell on dreams and forget to live... but a little preparation never hurt anyone. Now, off you go - I believe you have some code to write?"*

## 🚀 Getting Started

### Prerequisites

- Rust 1.70 or later
- Git (for version control integration)
- Cargo (Rust's package manager)

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/interview-irodov.git
cd interview-irodov

# Build all tools
cargo build --release

# Install binaries to your Cargo bin directory
cargo install --path ts-compressor
cargo install --path code-archiver
```

## 📝 Usage Examples

### Using The Marauder's Compressor

```bash
# Compress a TypeScript project
ts-compressor compress src/ dist/

# Create a project archive with LLM optimization
ts-compressor archive my-project --llm-optimize

# Custom file inclusion
ts-compressor archive my-project --include-extensions rs,js,ts,md
```

### Using The Code Archiver

```bash
# Basic archive creation
code-archiver --root ./my-project

# With Git integration and size filtering
code-archiver --root ./my-project --git --min-size 100 --max-size 10000

# Output to JSON format
code-archiver --root ./my-project --format json > project-structure.json
```

## 🏗 Project Structure

```
interview-irodov/
├── code-archiver/      # The Code Archiver
│   ├── src/            # Source code
│   ├── tests/          # Test files
│   └── Cargo.toml      # Rust package configuration
├── common/             # Shared utilities
│   ├── src/            # Shared code
│   └── Cargo.toml
├── file-splitter/      # File splitting utility
│   ├── src/            # Source code
│   ├── tests/          # Test files
│   └── Cargo.toml
├── impRustIdioms/      # Rust patterns and practices
│   ├── i00-pattern-list.txt  # Comprehensive pattern list
│   └── Rust Idiomatic Patterns Deep Dive_.md
├── test-input/         # Test files for development
├── ts-compressor/      # The Marauder's Compressor
│   ├── src/            # Source code
│   ├── tests/          # Test files
│   └── Cargo.toml
├── Cargo.toml          # Workspace configuration
├── Cargo.lock          # Dependency lock file
└── README.md           # This documentation
```

## 🔍 File Splitter

*"A handy tool for dividing large files into smaller, more manageable pieces. Particularly useful when dealing with oversized log files or datasets."*

### Features
- Split files by size
- Preserve file extensions
- Configurable chunk size
- Progress reporting

### Usage

```bash
# Split a large file into 10MB chunks
# Output will be: large_file.txt.part1, large_file.txt.part2, etc.
file-splitter split large_file.txt 10M

# Split with progress reporting (shows percentage complete)
# Useful for very large files
file-splitter split --progress huge_file.bin 100M

# Alternative: Use the shell script (splits into 10MB chunks by default)
# First argument: input file
# Second argument: number of lines per output file (not size)
./split_large_file.sh large_log_file.log 100000  # 100k lines per file
```

## 📚 Rust Idioms

*"A collection of Rust patterns and best practices, carefully curated for the aspiring Rustacean."*

### Key Topics
- Ownership and borrowing patterns
- Error handling strategies
- Concurrency models
- Memory safety patterns
- Async programming
- And many more...

### Usage

```bash
# Browse the pattern list
cat impRustIdioms/i00-pattern-list.txt

# Or open in your favorite editor
code impRustIdioms/
```

## 🛠 Development

### Building from Source

```bash
# Build all components in release mode (optimized)
# Output binaries will be in target/release/
cargo build --release

# Build a specific component (faster for development)
# -p specifies the package name from Cargo.toml
cargo build -p code-archiver --release

# Example: Build and install just the ts-compressor
cargo install --path ts-compressor --force
```

### Running Tests

```bash
# Run all tests across all workspace members
cargo test --workspace

# Run tests for a specific component
# -p specifies the package to test
cargo test -p code-archiver

# Run a specific test by name
cargo test test_archive_creation -- --nocapture

# Run with detailed logging (useful for debugging)
# RUST_LOG=debug enables debug-level logging
# --nocapture shows println! output during tests
RUST_LOG=debug cargo test -- --nocapture
```

### Code Quality

```bash
# Format all Rust code according to style guidelines
# This modifies files in place to match the standard Rust style
cargo fmt --all

# Run Clippy for additional code quality checks
# -D warnings turns all warnings into errors
# --all-targets checks all targets (lib, bins, tests, examples, etc.)
cargo clippy --all-targets -- -D warnings

# Additional useful development commands:
# Check for unused dependencies
cargo udeps

# Update dependencies
cargo update

# Check for security vulnerabilities
cargo audit
```

## 🧪 Testing

Run all tests for both projects:

```bash
# Run all tests
cargo test --workspace

# Run with detailed logging
RUST_LOG=debug cargo test -- --nocapture

# Run tests for a specific tool
cd ts-compressor && cargo test
# or
cd code-archiver && cargo test
```

## 🤝 Contributing

We welcome contributions from wizards and Muggles alike! Here's how you can help:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style

Please follow these guidelines:
- Use `rustfmt` for consistent code formatting
- Document public APIs with Rust doc comments
- Write tests for new features
- Keep commits small and focused

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by the need for better code organization and sharing tools
- Built with the amazing Rust programming language
- Thanks to all contributors who've helped improve these tools