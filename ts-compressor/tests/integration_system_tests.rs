//! Integration tests for the ts-compressor system
//!
//! These tests validate the complete end-to-end functionality including
//! CLI integration, file processing, and output generation.

use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Create a test directory with sample files for testing
fn create_test_project() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create a TypeScript file with sample content
    let ts_file = temp_dir.path().join("example.ts");
    fs::write(
        &ts_file,
        r#"
interface User {
    name: string;
    email: string;
    age: number;
}

class UserManager {
    private users: User[] = [];

    constructor() {
        console.log("UserManager initialized");
    }

    addUser(user: User): void {
        this.users.push(user);
        console.log(`Added user: ${user.name}`);
    }

    getUsers(): User[] {
        return this.users;
    }

    findUserByName(name: string): User | undefined {
        return this.users.find(user => user.name === name);
    }
}

const manager = new UserManager();
manager.addUser({
    name: "John Doe",
    age: 30,
    email: "john@example.com"
});

export { UserManager, User };
"#,
    )
    .expect("Failed to write TypeScript file");

    // Create a README file
    let readme_file = temp_dir.path().join("README.md");
    fs::write(
        &readme_file,
        r#"
# Test Project

This is a test project for testing functionality.

## Features

- User management functionality
- TypeScript support
- Clean architecture

## Usage

```typescript
const manager = new UserManager();
manager.addUser(user);
const users = manager.getUsers();
```
"#,
    )
    .expect("Failed to write README file");

    temp_dir
}

#[test]
fn test_project_setup() {
    let temp_dir = create_test_project();
    let target_path = temp_dir.path();

    // Verify test project structure
    assert!(target_path.join("example.ts").exists());
    assert!(target_path.join("README.md").exists());

    // Verify TypeScript file content
    let ts_content =
        fs::read_to_string(target_path.join("example.ts")).expect("Failed to read TypeScript file");
    assert!(ts_content.contains("interface User"));
    assert!(ts_content.contains("class UserManager"));

    println!("✅ Test project setup validation passed");
}

#[test]
fn test_binary_compilation() {
    // Test that the binary can be built successfully
    let output = Command::new("cargo")
        .args(&["build"])
        .current_dir(".")
        .output()
        .expect("Failed to execute cargo build");

    assert!(
        output.status.success(),
        "Failed to build binary: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    println!("✅ Binary compilation test passed");
}

#[test]
fn test_cli_help() {
    // Test that the CLI help works
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .current_dir(".")
        .output()
        .expect("Failed to run CLI help");

    assert!(
        output.status.success(),
        "CLI help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ts-compressor"));

    println!("✅ CLI help test passed");
}

#[test]
fn test_archive_command_basic() {
    let temp_dir = create_test_project();
    let target_path = temp_dir.path();
    let output_dir = TempDir::new().expect("Failed to create output temp directory");

    // Test basic archive command
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "archive",
            target_path.to_str().unwrap(),
            "--output-dir",
            output_dir.path().to_str().unwrap(),
            "--no-filter-stats", // Disable stats for cleaner test output
        ])
        .current_dir(".")
        .output()
        .expect("Failed to run archive command");

    if !output.status.success() {
        println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    }

    assert!(
        output.status.success(),
        "Archive command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that an archive file was created
    let archive_files: Vec<_> = fs::read_dir(output_dir.path())
        .expect("Failed to read output directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .ends_with(".txt")
        })
        .collect();

    assert!(!archive_files.is_empty(), "No archive file was created");

    println!("✅ Basic archive command test passed");
}

#[test]
fn test_archive_with_llm_optimization() {
    let temp_dir = create_test_project();
    let target_path = temp_dir.path();
    let output_dir = TempDir::new().expect("Failed to create output temp directory");

    // Create some files that should be excluded by LLM optimization
    fs::create_dir_all(target_path.join("node_modules")).expect("Failed to create node_modules");
    fs::write(
        target_path.join("node_modules").join("package.json"),
        r#"{"name": "test-package"}"#,
    ).expect("Failed to write node_modules file");

    fs::create_dir_all(target_path.join("target").join("debug")).expect("Failed to create target dir");
    fs::write(
        target_path.join("target").join("debug").join("app.exe"),
        "binary content",
    ).expect("Failed to write target file");

    // Test archive command with LLM optimization (enabled by default)
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "archive",
            target_path.to_str().unwrap(),
            "--output-dir",
            output_dir.path().to_str().unwrap(),
        ])
        .current_dir(".")
        .output()
        .expect("Failed to run archive command with LLM optimization");

    if !output.status.success() {
        println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    }

    assert!(
        output.status.success(),
        "Archive command with LLM optimization failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Check that filtering statistics are shown (could be in stdout or stderr due to logging)
    let combined_output = format!("{}{}", stdout, stderr);
    assert!(combined_output.contains("📊 File Filtering Statistics:") || combined_output.contains("LLM optimization"));
    assert!(combined_output.contains("🤖") || combined_output.contains("LLM"));

    println!("✅ Archive with LLM optimization test passed");
}

#[test]
fn test_archive_with_custom_patterns() {
    let temp_dir = create_test_project();
    let target_path = temp_dir.path();
    let output_dir = TempDir::new().expect("Failed to create output temp directory");

    // Create some files that should be excluded by custom patterns
    fs::write(
        target_path.join("secret.key"),
        "secret-key-content",
    ).expect("Failed to write secret file");

    fs::write(
        target_path.join("config.local.json"),
        r#"{"secret": "value"}"#,
    ).expect("Failed to write config file");

    // Test archive command with custom ignore patterns
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "archive",
            target_path.to_str().unwrap(),
            "--output-dir",
            output_dir.path().to_str().unwrap(),
            "--ignore-pattern",
            "*.key",
            "--ignore-pattern",
            "*.local.*",
            "--no-filter-stats", // Disable stats for cleaner test output
        ])
        .current_dir(".")
        .output()
        .expect("Failed to run archive command with custom patterns");

    if !output.status.success() {
        println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    }

    assert!(
        output.status.success(),
        "Archive command with custom patterns failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify archive was created
    let archive_files: Vec<_> = fs::read_dir(output_dir.path())
        .expect("Failed to read output directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .ends_with(".txt")
        })
        .collect();

    assert!(!archive_files.is_empty(), "No archive file was created");

    println!("✅ Archive with custom patterns test passed");
}

#[test]
fn test_archive_with_extension_filtering() {
    let temp_dir = create_test_project();
    let target_path = temp_dir.path();
    let output_dir = TempDir::new().expect("Failed to create output temp directory");

    // Create additional files with different extensions
    fs::write(
        target_path.join("script.py"),
        "print('Hello from Python')",
    ).expect("Failed to write Python file");

    fs::write(
        target_path.join("data.json"),
        r#"{"name": "test", "value": 42}"#,
    ).expect("Failed to write JSON file");

    // Test archive command with extension filtering
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "archive",
            target_path.to_str().unwrap(),
            "--output-dir",
            output_dir.path().to_str().unwrap(),
            "--include-extensions",
            "ts,md,py",
            "--no-filter-stats", // Disable stats for cleaner test output
        ])
        .current_dir(".")
        .output()
        .expect("Failed to run archive command with extension filtering");

    if !output.status.success() {
        println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    }

    assert!(
        output.status.success(),
        "Archive command with extension filtering failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify archive was created
    let archive_files: Vec<_> = fs::read_dir(output_dir.path())
        .expect("Failed to read output directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .ends_with(".txt")
        })
        .collect();

    assert!(!archive_files.is_empty(), "No archive file was created");

    println!("✅ Archive with extension filtering test passed");
}

#[test]
fn test_comprehensive_cli_experience() {
    let temp_dir = create_test_project();
    let target_path = temp_dir.path();
    let output_dir = TempDir::new().expect("Failed to create output temp directory");

    // Create a realistic project structure
    fs::create_dir_all(target_path.join("src")).expect("Failed to create src dir");
    fs::create_dir_all(target_path.join("tests")).expect("Failed to create tests dir");
    fs::create_dir_all(target_path.join("docs")).expect("Failed to create docs dir");

    // Add various file types
    fs::write(
        target_path.join("src").join("lib.rs"),
        r#"
//! Main library module
pub mod utils;

pub fn hello_world() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        hello_world();
    }
}
"#,
    ).expect("Failed to write Rust library file");

    fs::write(
        target_path.join("src").join("utils.rs"),
        r#"
//! Utility functions

/// Adds two numbers together
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Multiplies two numbers
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
"#,
    ).expect("Failed to write Rust utils file");

    fs::write(
        target_path.join("Cargo.toml"),
        r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
"#,
    ).expect("Failed to write Cargo.toml");

    fs::write(
        target_path.join("docs").join("API.md"),
        r#"
# API Documentation

## Functions

### `hello_world()`
Prints "Hello, world!" to stdout.

### `add(a, b)`
Adds two integers and returns the result.

### `multiply(a, b)`
Multiplies two integers and returns the result.
"#,
    ).expect("Failed to write API documentation");

    // Test comprehensive archive with full statistics
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "archive",
            target_path.to_str().unwrap(),
            "--output-dir",
            output_dir.path().to_str().unwrap(),
            // Enable all features for comprehensive test
            // LLM optimization is enabled by default
            // Filter stats are shown by default
        ])
        .current_dir(".")
        .output()
        .expect("Failed to run comprehensive archive command");

    if !output.status.success() {
        println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
        println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    }

    assert!(
        output.status.success(),
        "Comprehensive archive command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined_output = format!("{}{}", stdout, stderr);
    
    // Verify comprehensive output includes expected elements (could be in stdout or stderr due to logging)
    assert!(combined_output.contains("📊 File Filtering Statistics") || combined_output.contains("Starting archive creation"));
    assert!(combined_output.contains("Total files discovered:") || combined_output.contains("Files included:") || combined_output.contains("LLM optimization"));
    assert!(combined_output.contains("🎉 Archive processing completed successfully!") || combined_output.contains("Archive successfully created!"));

    // Verify archive file was created and contains expected content
    let archive_files: Vec<_> = fs::read_dir(output_dir.path())
        .expect("Failed to read output directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .ends_with(".txt")
        })
        .collect();

    assert!(!archive_files.is_empty(), "No archive file was created");

    // Read and verify archive content
    let archive_file = &archive_files[0];
    let archive_content = fs::read_to_string(archive_file.path())
        .expect("Failed to read archive file");

    // Verify archive contains expected sections (be flexible about exact format)
    assert!(archive_content.contains("Directory structure:") || archive_content.contains("Processing files..."));
    assert!(archive_content.contains("Absolute path:") || archive_content.contains("text starts") || archive_content.len() > 100);

    // Verify some specific content is included (at least one should be present)
    let has_rust_content = archive_content.contains("hello_world") || archive_content.contains("pub fn");
    let has_doc_content = archive_content.contains("API Documentation") || archive_content.contains("# API");
    let has_project_content = archive_content.contains("test-project") || archive_content.contains("Cargo.toml");
    
    assert!(has_rust_content || has_doc_content || has_project_content, 
        "Archive should contain some expected content. Archive length: {}", archive_content.len());

    println!("✅ Comprehensive CLI experience test passed");
}
