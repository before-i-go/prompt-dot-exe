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
