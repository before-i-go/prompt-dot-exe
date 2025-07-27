# archive-to-txt

A high-performance Rust library and CLI tool for creating text-based archives of directory contents with parallel processing support. Perfect for creating searchable, version-controlled snapshots of your codebase.

## Features

- 🚀 **Parallel Processing**: Utilizes Rayon for efficient multi-threaded execution
- 📁 **Smart Filtering**: Respects `.gitignore` and handles binary files intelligently
- 📝 **Configurable Output**: Multiple output formats and customization options
- 🔒 **Thread-Safe**: Safe concurrent file processing with proper error handling
- 🛠️ **Extensible**: Easy to integrate into your Rust projects

## Installation

### From Crates.io
```bash
cargo install archive-to-txt
```

### From Source
```bash
git clone https://github.com/yourusername/archive-to-txt.git
cd archive-to-txt
cargo install --path .
```

### As a Library
Add to your `Cargo.toml`:

```toml
[dependencies]
archive-to-txt = "0.2.0"
```

## Strapi Project Example

### Basic Usage

```rust
use archive_to_txt::{archive_directory, Config};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::default()
        .with_input("./path/to/your/strapi/project")
        .with_output("./strapi_archive.txt")
        .with_parallel(true)
        .with_include_extensions("js,jsx,ts,tsx,json")
        .with_exclude_patterns("node_modules/**,build/**");

    archive_directory(
        "./path/to/your/strapi/project",
        "./strapi_archive.txt",
        &config
    )?;
    
    Ok(())
}
```

### CLI Usage

```bash
# Basic archive of a Strapi project
archive-to-txt --input ./path/to/strapi --output strapi_archive.txt

# With advanced options
archive-to-txt \
  --input ./path/to/strapi \
  --output strapi_filtered.txt \
  --include "*.js,*.jsx,*.ts,*.tsx,*.json" \
  --exclude "node_modules/**,.git/**,.cache/**" \
  --max-file-size 5MB \
  --parallel true
```

### Example Output

```
=== File: /config/database.js ===
module.exports = ({ env }) => ({
  connection: {
    client: 'postgres',
    connection: {
      host: env('DATABASE_HOST', 'localhost'),
      port: env.int('DATABASE_PORT', 5432),
      database: env('DATABASE_NAME', 'strapi'),
      user: env('DATABASE_USERNAME', 'strapi'),
      password: env('DATABASE_PASSWORD', 'strapi'),
      ssl: env.bool('DATABASE_SSL', false) ? {
        rejectUnauthorized: env.bool('DATABASE_SSL_SELF', false),
      } : false,
    },
    debug: false,
  },
});

=== File: /src/api/restaurant/controllers/restaurant.js ===
'use strict';

/**
 * restaurant controller
 */

const { createCoreController } = require('@strapi/strapi').factories;

module.exports = createCoreController('api::restaurant.restaurant');
```

## Configuration Options

| Option | CLI Flag | Description | Default |
|--------|----------|-------------|---------|
| `input` | `--input` | Input directory path | Required |
| `output` | `--output` | Output file path | Required |
| `parallel` | `--parallel` | Enable parallel processing | `true` |
| `include_hidden` | `--include-hidden` | Include hidden files | `false` |
| `include_extensions` | `--include-extensions` | Comma-separated list of file extensions to include | All files |
| `exclude_patterns` | `--exclude` | Comma-separated glob patterns to exclude | None |
| `max_file_size` | `--max-file-size` | Skip files larger than this size (e.g., 5MB) | No limit |
| `git_info` | `--git-info` | Include git information | `false` |

## Building from Source

```bash
# Build in release mode
cargo build --release

# Run tests
cargo test

# Build documentation
cargo doc --open
```

## Performance

For a typical Strapi project with ~10,000 files:

- **Sequential processing**: ~15-20 seconds
- **Parallel processing (8 threads)**: ~3-5 seconds

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE).

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
