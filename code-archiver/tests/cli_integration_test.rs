use anyhow::Result;
use std::process::Command;
use tempfile::TempDir;
use std::fs;

/// Test utilities for CLI integration testing
mod test_utils {
    use super::*;

    pub fn create_test_project() -> Result<TempDir> {
        let temp_dir = TempDir::new()?;
        
        // Create a diverse project structure for testing
        fs::create_dir_all(temp_dir.path().join("src"))?;
        fs::write(temp_dir.path().join("src/main.rs"), r#"
fn main() {
    println!("Hello, world!");
}
"#)?;
        
        fs::write(temp_dir.path().join("src/lib.rs"), r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
"#)?;
        
        fs::write(temp_dir.path().join("Cargo.toml"), r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
"#)?;
        
        fs::write(temp_dir.path().join("README.md"), r#"
# Test Project

This is a test project for CLI integration testing.

## Features

- Rust code
- Documentation
- Configuration files
"#)?;
        
        // Create some files that should be filtered
        fs::create_dir_all(temp_dir.path().join("target/debug"))?;
        fs::write(temp_dir.path().join("target/debug/test-project"), "binary content")?;
        
        fs::create_dir_all(temp_dir.path().join(".git"))?;
        fs::write(temp_dir.path().join(".git/config"), "[core]\n    repositoryformatversion = 0")?;
        
        fs::write(temp_dir.path().join(".gitignore"), "target/\n*.tmp\n")?;
        
        // Create a large file for testing
        let large_content = "x".repeat(10000);
        fs::write(temp_dir.path().join("large_file.txt"), &large_content)?;
        
        // Create a TypeScript file for ts-compressor testing
        fs::write(temp_dir.path().join("app.ts"), r#"
interface User {
    name: string;
    email: string;
}

class UserService {
    private users: User[] = [];
    
    addUser(user: User): void {
        this.users.push(user);
    }
    
    getUsers(): User[] {
        return this.users;
    }
}

export { UserService, User };
"#)?;
        
        Ok(temp_dir)
    }
    
    pub fn run_command(program: &str, args: &[&str]) -> Result<std::process::Output> {
        Command::new("cargo")
            .arg("run")
            .arg("-p")
            .arg(program)
            .arg("--")
            .args(args)
            .output()
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod code_archiver_cli_tests {
    use super::*;
    use test_utils::*;

    #[test]
    fn test_code_archiver_help() -> Result<()> {
        let output = run_command("code-archiver", &["--help"])?;
        
        assert!(output.status.success(), 
            "Help command failed: {}", 
            String::from_utf8_lossy(&output.stderr));
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("code-archiver"));
        assert!(stdout.contains("--dir"));
        assert!(stdout.contains("--format"));
        assert!(stdout.contains("--extensions"));
        
        Ok(())
    }
    
    #[test]
    fn test_code_archiver_basic_functionality() -> Result<()> {
        let temp_dir = create_test_project()?;
        
        let output = run_command("code-archiver", &[
            "--dir", temp_dir.path().to_str().unwrap(),
            "--format", "json"
        ])?;
        
        assert!(output.status.success(), 
            "Basic functionality failed: {}", 
            String::from_utf8_lossy(&output.stderr));
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Should be valid JSON
        let json: serde_json::Value = serde_json::from_str(&stdout)
            .expect("Output should be valid JSON");
        
        // Should contain files
        assert!(json.is_array());
        let files = json.as_array().unwrap();
        assert!(!files.is_empty(), "Should find some files");
        
        // Should contain expected files
        let file_paths: Vec<String> = files.iter()
            .filter_map(|f| f.get("path"))
            .filter_map(|p| p.as_str())
            .map(|s| s.to_string())
            .collect();
        
        assert!(file_paths.iter().any(|p| p.contains("main.rs")));
        assert!(file_paths.iter().any(|p| p.contains("Cargo.toml")));
        
        Ok(())
    }
    
    #[test]
    fn test_code_archiver_extension_filtering() -> Result<()> {
        let temp_dir = create_test_project()?;
        
        let output = run_command("code-archiver", &[
            "--dir", temp_dir.path().to_str().unwrap(),
            "--extensions", "rs",
            "--format", "json"
        ])?;
        
        assert!(output.status.success());
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&stdout)?;
        let files = json.as_array().unwrap();
        
        // All files should have .rs extension
        for file in files {
            let path = file.get("path").unwrap().as_str().unwrap();
            assert!(path.ends_with(".rs"), "File {} should have .rs extension", path);
        }
        
        Ok(())
    }
    
    #[test]
    fn test_code_archiver_text_format() -> Result<()> {
        let temp_dir = create_test_project()?;
        
        let output = run_command("code-archiver", &[
            "--dir", temp_dir.path().to_str().unwrap(),
            "--format", "text"
        ])?;
        
        assert!(output.status.success());
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Should contain file listings
        assert!(stdout.contains("main.rs"));
        assert!(stdout.contains("Total:"));
        
        // Should show file sizes
        assert!(stdout.contains("B") || stdout.contains("KB") || stdout.contains("MB"));
        
        Ok(())
    }
    
    #[test]
    fn test_code_archiver_invalid_format() -> Result<()> {
        let temp_dir = create_test_project()?;
        
        let output = run_command("code-archiver", &[
            "--dir", temp_dir.path().to_str().unwrap(),
            "--format", "invalid"
        ])?;
        
        assert!(!output.status.success());
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Unsupported format"));
        
        Ok(())
    }
    
    #[test]
    fn test_code_archiver_nonexistent_directory() -> Result<()> {
        let output = run_command("code-archiver", &[
            "--dir", "/nonexistent/directory"
        ])?;
        
        assert!(!output.status.success());
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Error"));
        
        Ok(())
    }
}

#[cfg(test)]
mod ts_compressor_cli_tests {
    use super::*;
    use test_utils::*;

    #[test]
    fn test_ts_compressor_help() -> Result<()> {
        let output = run_command("ts-compressor", &["--help"])?;
        
        assert!(output.status.success());
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("ts-compressor"));
        assert!(stdout.contains("compress"));
        assert!(stdout.contains("archive"));
        
        Ok(())
    }
    
    #[test]
    fn test_ts_compressor_compress_help() -> Result<()> {
        let output = run_command("ts-compressor", &["compress", "--help"])?;
        
        assert!(output.status.success());
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("input_dir"));
        assert!(stdout.contains("output_dir"));
        
        Ok(())
    }
    
    #[test]
    fn test_ts_compressor_archive_help() -> Result<()> {
        let output = run_command("ts-compressor", &["archive", "--help"])?;
        
        assert!(output.status.success());
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("target_folder"));
        assert!(stdout.contains("llm-optimize"));
        assert!(stdout.contains("ignore-pattern"));
        
        Ok(())
    }
    
    #[test]
    fn test_ts_compressor_compress_functionality() -> Result<()> {
        let temp_dir = create_test_project()?;
        let output_dir = TempDir::new()?;
        
        let output = run_command("ts-compressor", &[
            "compress",
            temp_dir.path().to_str().unwrap(),
            output_dir.path().to_str().unwrap()
        ])?;
        
        assert!(output.status.success(), 
            "Compress failed: {}", 
            String::from_utf8_lossy(&output.stderr));
        
        // Check that JavaScript file was created
        let js_file = output_dir.path().join("app.js");
        assert!(js_file.exists(), "JavaScript file should be created");
        
        let js_content = std::fs::read_to_string(js_file)?;
        // Should not contain TypeScript-specific syntax
        assert!(!js_content.contains("interface"));
        assert!(!js_content.contains(": string"));
        
        Ok(())
    }
    
    #[test]
    fn test_ts_compressor_archive_functionality() -> Result<()> {
        let temp_dir = create_test_project()?;
        let output_dir = TempDir::new()?;
        
        let output = run_command("ts-compressor", &[
            "archive",
            temp_dir.path().to_str().unwrap(),
            "--output-dir", output_dir.path().to_str().unwrap()
        ])?;
        
        assert!(output.status.success(), 
            "Archive failed: {}", 
            String::from_utf8_lossy(&output.stderr));
        
        // Check that archive file was created
        let archive_files: Vec<_> = std::fs::read_dir(output_dir.path())?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.file_name().to_string_lossy().ends_with(".txt")
            })
            .collect();
        
        assert!(!archive_files.is_empty(), "Archive file should be created");
        
        let archive_file = &archive_files[0];
        let archive_content = std::fs::read_to_string(archive_file.path())?;
        
        // Should contain directory structure
        assert!(archive_content.contains("Directory structure"));
        
        // Should contain file contents
        assert!(archive_content.contains("Absolute path:"));
        assert!(archive_content.contains("<text starts>"));
        assert!(archive_content.contains("<text ends>"));
        
        Ok(())
    }
    
    #[test]
    fn test_ts_compressor_archive_with_llm_optimization() -> Result<()> {
        let temp_dir = create_test_project()?;
        let output_dir = TempDir::new()?;
        
        let output = run_command("ts-compressor", &[
            "archive",
            temp_dir.path().to_str().unwrap(),
            "--output-dir", output_dir.path().to_str().unwrap(),
            // LLM optimization is enabled by default, so we test the default behavior
        ])?;
        
        assert!(output.status.success());
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Should show LLM optimization is enabled
        assert!(stdout.contains("LLM optimization") || stdout.contains("🤖"));
        
        // Should show filtering statistics
        assert!(stdout.contains("File Filtering Statistics") || stdout.contains("📊"));
        
        Ok(())
    }
    
    #[test]
    fn test_ts_compressor_archive_with_custom_patterns() -> Result<()> {
        let temp_dir = create_test_project()?;
        let output_dir = TempDir::new()?;
        
        let output = run_command("ts-compressor", &[
            "archive",
            temp_dir.path().to_str().unwrap(),
            "--output-dir", output_dir.path().to_str().unwrap(),
            "--ignore-pattern", "*.md",
            "--include-extensions", "rs,toml"
        ])?;
        
        assert!(output.status.success());
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Custom ignore patterns") || stdout.contains("📝"));
        
        Ok(())
    }
}

#[cfg(test)]
mod file_splitter_cli_tests {
    use super::*;
    use test_utils::*;

    #[test]
    fn test_file_splitter_help() -> Result<()> {
        let output = run_command("file-splitter", &["--help"])?;
        
        assert!(output.status.success());
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("file-splitter"));
        assert!(stdout.contains("--input"));
        assert!(stdout.contains("--chunk-size"));
        assert!(stdout.contains("--output-dir"));
        
        Ok(())
    }
    
    #[test]
    fn test_file_splitter_basic_functionality() -> Result<()> {
        let temp_dir = create_test_project()?;
        let output_dir = TempDir::new()?;
        
        // Use the large file we created
        let input_file = temp_dir.path().join("large_file.txt");
        
        let output = run_command("file-splitter", &[
            "--input", input_file.to_str().unwrap(),
            "--output-dir", output_dir.path().to_str().unwrap(),
            "--chunk-size", "1K"
        ])?;
        
        assert!(output.status.success(), 
            "File splitter failed: {}", 
            String::from_utf8_lossy(&output.stderr));
        
        // Check that chunk files were created
        let chunk_files: Vec<_> = std::fs::read_dir(output_dir.path())?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.file_name().to_string_lossy().contains("large_file")
            })
            .collect();
        
        assert!(!chunk_files.is_empty(), "Chunk files should be created");
        assert!(chunk_files.len() > 1, "Should create multiple chunks for large file");
        
        Ok(())
    }
    
    #[test]
    fn test_file_splitter_custom_prefix() -> Result<()> {
        let temp_dir = create_test_project()?;
        let output_dir = TempDir::new()?;
        
        let input_file = temp_dir.path().join("large_file.txt");
        
        let output = run_command("file-splitter", &[
            "--input", input_file.to_str().unwrap(),
            "--output-dir", output_dir.path().to_str().unwrap(),
            "--chunk-size", "2K",
            "--prefix", "custom_chunk"
        ])?;
        
        assert!(output.status.success());
        
        // Check that files with custom prefix were created
        let chunk_files: Vec<_> = std::fs::read_dir(output_dir.path())?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.file_name().to_string_lossy().starts_with("custom_chunk")
            })
            .collect();
        
        assert!(!chunk_files.is_empty(), "Custom prefix chunk files should be created");
        
        Ok(())
    }
    
    #[test]
    fn test_file_splitter_nonexistent_file() -> Result<()> {
        let output = run_command("file-splitter", &[
            "--input", "/nonexistent/file.txt"
        ])?;
        
        assert!(!output.status.success());
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Error") || stderr.contains("not found"));
        
        Ok(())
    }
    
    #[test]
    fn test_file_splitter_invalid_chunk_size() -> Result<()> {
        let temp_dir = create_test_project()?;
        let input_file = temp_dir.path().join("large_file.txt");
        
        let output = run_command("file-splitter", &[
            "--input", input_file.to_str().unwrap(),
            "--chunk-size", "invalid"
        ])?;
        
        assert!(!output.status.success());
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Error") || stderr.contains("Invalid"));
        
        Ok(())
    }
}

#[cfg(test)]
mod cross_tool_integration_tests {
    use super::*;
    use test_utils::*;

    #[test]
    fn test_workflow_code_archiver_then_ts_compressor() -> Result<()> {
        let temp_dir = create_test_project()?;
        
        // First, use code-archiver to analyze the project
        let archive_output = run_command("code-archiver", &[
            "--dir", temp_dir.path().to_str().unwrap(),
            "--format", "json"
        ])?;
        
        assert!(archive_output.status.success());
        
        // Then, use ts-compressor to create an archive
        let ts_output = run_command("ts-compressor", &[
            "archive",
            temp_dir.path().to_str().unwrap()
        ])?;
        
        assert!(ts_output.status.success());
        
        // Both should complete successfully and provide consistent information
        let archive_json: serde_json::Value = serde_json::from_str(
            &String::from_utf8_lossy(&archive_output.stdout)
        )?;
        
        assert!(archive_json.is_array());
        assert!(!archive_json.as_array().unwrap().is_empty());
        
        Ok(())
    }
    
    #[test]
    fn test_all_tools_handle_same_project_consistently() -> Result<()> {
        let temp_dir = create_test_project()?;
        
        // Test that all tools can process the same project without conflicts
        let code_archiver_result = run_command("code-archiver", &[
            "--dir", temp_dir.path().to_str().unwrap()
        ]);
        
        let ts_compressor_result = run_command("ts-compressor", &[
            "archive",
            temp_dir.path().to_str().unwrap()
        ]);
        
        let large_file = temp_dir.path().join("large_file.txt");
        let file_splitter_result = run_command("file-splitter", &[
            "--input", large_file.to_str().unwrap(),
            "--chunk-size", "5K"
        ]);
        
        // All tools should handle the project successfully
        assert!(code_archiver_result.is_ok());
        assert!(ts_compressor_result.is_ok());
        assert!(file_splitter_result.is_ok());
        
        if let Ok(output) = code_archiver_result {
            assert!(output.status.success());
        }
        
        if let Ok(output) = ts_compressor_result {
            assert!(output.status.success());
        }
        
        if let Ok(output) = file_splitter_result {
            assert!(output.status.success());
        }
        
        Ok(())
    }
}