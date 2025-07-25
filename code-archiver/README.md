# 📜 The Code Archiver

*"A most ingenious magical artifact for the modern witch or wizard! This enchanted tool allows you to capture and organize your magical code repositories with the flick of a wand—or rather, the press of a key. It's particularly useful for preparing your potions—er, projects—for the Wizarding Code Review Board."*

## 🎩 Features

- **Magical Directory Traversal**: Glide through your project directories as gracefully as a Thestral in flight
- **Pattern Recognition Charms**: Powerful glob patterns to include or exclude files with precision
- **Git Familiar Integration**: Works seamlessly with your magical version control system
- **Size Matters Not**: Filter files by size, because even wizards need to mind their storage
- **Multiple Output Formats**: JSON for modern wizards, plain text for traditionalists

## 🔮 Installation

### Via Cargo (Recommended for All Wizards)

```bash
cargo install --path .
```

### For Muggles

1. Ensure you have Rust 1.70 or later installed
2. Clone this repository
3. Run `cargo build --release`
4. Find the executable in `target/release/code-archiver`

## 🧙‍♂️ Basic Usage

### Command Line Incantations

```bash
# Basic spell to archive your project
code-archiver --root ./my-spellbook

# For more selective wizards (filter by file patterns)
code-archiver --include '**/*.rs' --exclude '**/test_*.rs'

# For those who work with magical creatures (Git repositories)
code-archiver --git

# For wizards who like their potions measured precisely
code-archiver --min-size 100 --max-size 10000

# For the modern wizard who speaks in JSON
code-archiver --format json
```

## 🧪 Advanced Wizardry

### Library Usage

For those who wish to embed this magic in their own spells:

```rust
use code_archiver::{CodeArchiver, ArchiveConfig};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ArchiveConfig::new("./my-spellbook")
        .include(Some(vec![
            "**/*.rs".to_string(),
            "**/*.toml".to_string(),
        ]))
        .exclude(Some(vec![
            "**/target/**".to_string(),
            "**/node_modules/**".to_string(),
        ]))
        .git(true);
        
    let archiver = CodeArchiver::new(config)?;
    let archive = archiver.create_archive()?;
    
    println!("Captured {} magical artifacts", archive.len());
    
    // For advanced spellcasting (JSON output)
    let json = serde_json::to_string_pretty(&archive)?;
    println!("Your archive, in JSON form: {}", json);
    
    Ok(())
}
```

## 🔍 Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `root_dir` | `PathBuf` | `.` | The root of your magical repository |
| `include` | `Option<Vec<String>>` | `None` | Patterns to include (supports glob) |
| `exclude` | `Option<Vec<String>>` | `None` | Patterns to exclude (supports glob) |
| `extensions` | `Option<Vec<String>>` | `None` | File extensions to include |
| `min_size` | `Option<u64>` | `None` | Minimum file size in bytes |
| `max_size` | `Option<u64>` | `None` | Maximum file size in bytes |
| `follow_links` | `bool` | `false` | Follow symbolic links |
| `hidden` | `bool` | `false` | Include hidden files |
| `git` | `bool` | `false` | Enable Git integration |
| `gitignore` | `bool` | `true` | Respect .gitignore files |
| `include_git_status` | `bool` | `true` | Include Git status in output |
| `include_ignored` | `bool` | `false` | Include Git-ignored files |

## 🧪 Testing Your Spells

To ensure your magical artifacts work as intended:

```bash
cargo test -- --nocapture
```

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Albus Dumbledore** for his wisdom in magical education
- **Newt Scamander** for teaching us to care for magical code creatures
- **Hermione Granger** for proving that with enough preparation, any spell can be mastered

*"It does not do to dwell on dreams and forget to test your code."* - Albus Dumbledore (probably)
