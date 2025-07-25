#![cfg(feature = "test-utils")]

use assert_fs::prelude::*;
use assert_fs::TempDir;
use std::fs;
use std::path::Path;

/// A test Git repository for integration testing
/// 
/// This struct provides utilities for creating and manipulating a test Git repository
/// in a temporary directory. It's only available when the "test-utils" feature is enabled.
#[derive(Debug)]
pub struct TestGitRepo {
    /// The temporary directory containing the Git repository
    pub temp_dir: TempDir,
}

impl TestGitRepo {
    /// Create a new test Git repository in a temporary directory
    /// 
    /// This initializes a new Git repository and sets up a test user configuration.
    /// The temporary directory will be automatically cleaned up when the TestGitRepo is dropped.
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let _ = std::process::Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .output()
            .expect("Failed to initialize git repository");
        
        // Set user config for the test repository
        let _ = std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&temp_dir)
            .status()
            .expect("Failed to set git user name");
            
        let _ = std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&temp_dir)
            .status()
            .expect("Failed to set git user email");
        
        Self { temp_dir }
    }

    /// Add a file to the test repository
    /// 
    /// # Arguments
    /// * `path` - The path to the file relative to the repository root
    /// * `content` - The content to write to the file
    /// 
    /// # Returns
    /// The full path to the created file
    pub fn add_file(&self, path: &str, content: &str) -> std::path::PathBuf {
        let file_path = self.temp_dir.path().join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent directory");
        }
        fs::write(&file_path, content).expect("Failed to write test file");
        file_path
    }

    /// Add a pattern to the .gitignore file in the test repository
    /// 
    /// # Arguments
    /// * `pattern` - The pattern to add to .gitignore
    pub fn add_to_gitignore(&self, pattern: &str) {
        let gitignore_path = self.temp_dir.path().join(".gitignore");
        fs::write(gitignore_path, pattern).expect("Failed to write .gitignore");
    }

    /// Commit all changes in the test repository
    /// 
    /// # Arguments
    /// * `message` - The commit message to use
    pub fn commit(&self, message: &str) {
        // Add all files to git
        let _ = std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&self.temp_dir)
            .status()
            .expect("Failed to add files to git");

        // Create initial commit
        let status = std::process::Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(&self.temp_dir)
            .status()
            .expect("Failed to commit files");

        if !status.success() {
            panic!("Git commit failed");
        }
    }
}
