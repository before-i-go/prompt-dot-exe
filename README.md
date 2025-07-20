# Interview Irodov

A repository containing interview preparation materials and a Rust CLI tool for TypeScript compilation and code archival.

## ts-compressor Tool

### Compilation

```bash
cd ts-compressor
cargo build --release
```

Requires Rust 1.70+ and Git.

### Usage and Sample Output

**TypeScript Compilation:**
```bash
./target/release/ts-compressor compress src/ dist/
```

Converts TypeScript files to minified JavaScript. Processes `.ts` and `.tsx` files, strips types, applies minification.

**Code Archiving:**
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

### Default Behavior

- **LLM optimization enabled**: Automatically excludes build artifacts, dependencies, cache files
- **Git integration**: Respects `.gitignore` when present
- **Binary file detection**: Excludes binary files from text output
- **Timestamped output**: Creates uniquely named archive files

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

### What Gets Excluded (LLM Optimization)

- Build outputs: `target/`, `build/`, `dist/`, `*.exe`, `*.dll`
- Dependencies: `node_modules/`, `vendor/`, `venv/`
- Cache: `.cache/`, `*.tmp`, `*.bak`
- IDE files: `.vscode/`, `.idea/`
- OS files: `.DS_Store`, `Thumbs.db`
- Media: `*.png`, `*.jpg`, `*.mp4`
- Lock files: `package-lock.json`, `Cargo.lock`

### Usefulness

The tool is useful for:
- Creating complete project snapshots in text format
- Preparing code for LLM analysis (excludes irrelevant files)
- TypeScript minification for deployment
- Code review and documentation
- Backup and archival of project state

## Testing

```bash
cd ts-compressor
cargo test
cargo run -- --help
cargo run -- archive ../test-input
```

## Repository Structure

```
├── ts-compressor/           # Main Rust CLI tool
│   ├── src/main.rs         # Application code
│   ├── Cargo.toml          # Dependencies
│   └── tests/              # Integration tests
├── test-input/             # Sample files for testing
│   └── example.ts          # TypeScript test file
├── zzArchive/              # Jupyter notebooks
│   ├── RailsCrashCours202507.ipynb
│   └── RustCrashCourse202507.ipynb
├── Unclassified20250706.txt # Interview questions
├── i00-pattern-list.txt    # Interview patterns
└── split_large_file.sh     # File utility
```

## Dependencies

- swc_core - TypeScript compilation and minification
- clap - Command-line argument parsing
- git2 - Git repository integration
- walkdir - Directory traversal
- tracing - Structured logging
- mime_guess - File type detection

## Interview Materials

- `zzArchive/` - Jupyter notebooks for Rails and Rust
- `Unclassified20250706.txt` - Mixed interview questions
- `i00-pattern-list.txt` - Common interview patterns
- Various `.md` files with technical documentation