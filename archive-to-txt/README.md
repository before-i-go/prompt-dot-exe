# archive-to-txt

A high-performance Rust library for creating text-based archives of directory contents with parallel processing support.

## Features

- 🚀 Parallel processing using Rayon
- 📁 Recursive directory traversal
- 📝 Text-based output formatting
- 🔒 Thread-safe operations
- 🛠️ Configurable processing options

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
archive-to-txt = "0.1.0"
```

## Quick Start

```rust
use archive_to_txt::{archive_directory, Config};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::default()
        .with_input("./src")
        .with_output("./archive.txt")
        .with_parallel(true);

    archive_directory("./src", "./archive.txt", &config)?;
    Ok(())
}
```

## Configuration

| Option | Description | Default |
|--------|-------------|---------|
| `with_input` | Input directory path | Required |
| `with_output` | Output file path | Required |
| `with_parallel` | Enable parallel processing | `true` |
| `with_include_hidden` | Include hidden files | `false` |

## Building

```bash
# Build in release mode
cargo build --release

# Run tests
cargo test
```

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE).

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.
