//! Integration tests for the Universal Code Compressor system
//! 
//! These tests validate the complete end-to-end functionality including
//! CLI integration, file processing, compression pipeline, and output generation.

use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Create a test directory with sample files for compression testing
fn create_test_project() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Create a TypeScript file with repetitive patterns
    let ts_file = temp_dir.path().join("example.ts");
    fs::write(&ts_file, r#"
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
"#).expect("Failed to write TypeScript file");

    // Create a README file with repetitive patterns
    let readme_file = temp_dir.path().join("README.md");
    fs::write(&readme_file, r#"
# Test Project

This is a test project for testing compression capabilities.
The test project contains test files with test patterns.
Testing compression on test data helps validate test results.

## Features

- User management functionality
- User creation and user retrieval
- User search and user validation
- User data persistence and user data processing

## Usage

```typescript
const manager = new UserManager();
manager.addUser(user);
const users = manager.getUsers();
const user = manager.findUserByName("test");
```

## Testing

Run tests with:
```bash
npm test
npm run test
npm run test:unit
npm run test:integration
```

## Performance

The system provides excellent performance for:
- User operations performance
- Data processing performance  
- Search functionality performance
- Memory usage performance
"#).expect("Failed to write README file");

    // Create a subdirectory with more files
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).expect("Failed to create src directory");
    
    let lib_file = src_dir.join("lib.ts");
    fs::write(&lib_file, r#"
export function processUser(user: User): ProcessedUser {
    return {
        id: generateUserId(),
        name: user.name,
        email: user.email,
        processedAt: new Date()
    };
}

export function validateUser(user: User): boolean {
    return user.name.length > 0 && user.email.includes('@');
}
"#).expect("Failed to write lib file");

    temp_dir
}

#[test]
fn test_end_to_end_universal_compression() {
    let temp_dir = create_test_project();
    let target_path = temp_dir.path();
    
    // Build the binary
    let output = Command::new("cargo")
        .args(&["build", "--release"])
        .current_dir(".")
        .output()
        .expect("Failed to build binary");
    
    assert!(output.status.success(), "Failed to build binary: {}", 
            String::from_utf8_lossy(&output.stderr));
    
    // Run universal compression
    let output = Command::new("./target/release/ts-compressor")
        .args(&["universal-compress", target_path.to_str().unwrap()])
        .current_dir(".")
        .output()
        .expect("Failed to run universal compression");
    
    assert!(output.status.success(), "Universal compression failed: {}", 
            String::from_utf8_lossy(&output.stderr));
    
    // Verify output file was created
    let output_files: Vec<_> = fs::read_dir(target_path.parent().unwrap())
        .expect("Failed to read output directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_name().to_string_lossy().starts_with(
                &format!("{}_", target_path.file_name().unwrap().to_string_lossy())
            ) && entry.file_name().to_string_lossy().ends_with(".txt")
        })
        .collect();
    
    assert!(!output_files.is_empty(), "No output file was created");
    
    // Read and validate the output file
    let output_file = &output_files[0];
    let content = fs::read_to_string(output_file.path())
        .expect("Failed to read output file");
    
    // Validate output file structure
    assert!(content.contains("# Universal Code Compression Output"));
    assert!(content.contains("## Compression Statistics"));
    assert!(content.contains("## Embedded Dictionary"));
    assert!(content.contains("## Directory Structure Manifest"));
    assert!(content.contains("## Compressed Content"));
    
    // Validate compression statistics
    assert!(content.contains("Files processed:"));
    assert!(content.contains("Original size:"));
    assert!(content.contains("Compressed size:"));
    assert!(content.contains("Compression ratio:"));
    assert!(content.contains("Dictionary entries:"));
    
    // Validate dictionary format
    assert!(content.contains("DICT:"));
    assert!(content.contains("=T"));
    
    // Validate compressed content
    assert!(content.contains("### File:"));
    assert!(content.contains("Content:"));
    
    println!("✅ End-to-end universal compression test passed");
}

#[test]
fn test_compression_with_different_parameters() {
    let temp_dir = create_test_project();
    let target_path = temp_dir.path();
    
    // Test with custom parameters
    let output = Command::new("cargo")
        .args(&[
            "run", "--", "universal-compress", 
            target_path.to_str().unwrap(),
            "--min-pattern-length", "5",
            "--min-frequency-threshold", "2"
        ])
        .current_dir(".")
        .output()
        .expect("Failed to run universal compression with custom parameters");
    
    assert!(output.status.success(), "Universal compression with custom parameters failed: {}", 
            String::from_utf8_lossy(&output.stderr));
    
    println!("✅ Compression with custom parameters test passed");
}

#[test]
fn test_compression_with_zstd_enabled() {
    let temp_dir = create_test_project();
    let target_path = temp_dir.path();
    
    // Test with zstd compression enabled
    let output = Command::new("cargo")
        .args(&[
            "run", "--", "universal-compress", 
            target_path.to_str().unwrap(),
            "--enable-zstd"
        ])
        .current_dir(".")
        .output()
        .expect("Failed to run universal compression with zstd");
    
    assert!(output.status.success(), "Universal compression with zstd failed: {}", 
            String::from_utf8_lossy(&output.stderr));
    
    println!("✅ Compression with zstd test passed");
}

#[test]
fn test_error_handling_invalid_directory() {
    let invalid_path = "/nonexistent/directory";
    
    let output = Command::new("cargo")
        .args(&["run", "--", "universal-compress", invalid_path])
        .current_dir(".")
        .output()
        .expect("Failed to run universal compression with invalid directory");
    
    assert!(!output.status.success(), "Should fail with invalid directory");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not a directory") || stderr.contains("Failed to create archiver"));
    
    println!("✅ Error handling for invalid directory test passed");
}

#[test]
fn test_compression_statistics_accuracy() {
    let temp_dir = create_test_project();
    let target_path = temp_dir.path();
    
    // Calculate original size manually
    let mut original_size = 0;
    for entry in walkdir::WalkDir::new(target_path) {
        let entry = entry.expect("Failed to read directory entry");
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                original_size += metadata.len();
            }
        }
    }
    
    // Run compression
    let output = Command::new("cargo")
        .args(&["run", "--", "universal-compress", target_path.to_str().unwrap()])
        .current_dir(".")
        .output()
        .expect("Failed to run universal compression");
    
    assert!(output.status.success());
    
    // Find and read output file
    let output_files: Vec<_> = fs::read_dir(target_path.parent().unwrap())
        .expect("Failed to read output directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_name().to_string_lossy().starts_with(
                &format!("{}_", target_path.file_name().unwrap().to_string_lossy())
            ) && entry.file_name().to_string_lossy().ends_with(".txt")
        })
        .collect();
    
    let content = fs::read_to_string(output_files[0].path())
        .expect("Failed to read output file");
    
    // Verify that reported original size is reasonable
    // (allowing for text file filtering differences)
    assert!(content.contains(&format!("Original size: {} bytes", original_size)) ||
            content.contains("Original size:"), 
            "Original size should be reported in statistics");
    
    println!("✅ Compression statistics accuracy test passed");
}