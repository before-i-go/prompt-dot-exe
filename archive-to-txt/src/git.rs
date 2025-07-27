//! Git integration for the archive-to-txt library.
//!
//! This module provides functionality to extract and format Git repository information
//! for inclusion in the text archive.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use log::{debug, warn};
use serde::Serialize;

/// Information about a Git repository
#[derive(Debug, Clone, Serialize)]
pub struct GitInfo {
    /// The repository root directory
    pub root: PathBuf,
    /// The current branch name
    pub branch: Option<String>,
    /// The current commit hash
    pub commit: Option<String>,
    /// The commit author
    pub author: Option<String>,
    /// The commit date
    pub date: Option<DateTime<Local>>,
    /// Whether there are uncommitted changes
    pub has_uncommitted_changes: bool,
}

impl GitInfo {
    /// Attempt to create a new GitInfo instance from a path within a git repository.
    ///
    /// # Arguments
    /// * `path` - A path within a git repository
    ///
    /// # Returns
    /// `Some(GitInfo)` if the path is in a git repository, `None` otherwise.
    pub fn from_path(path: &Path) -> Result<Self> {
        // Find the git repository root
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(path)
            .output()
            .context("Failed to execute git command")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Not a git repository"));
        }

        let root = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim().to_string());

        // Get current branch
        let branch = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&root)
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    None
                }
            });

        // Get current commit
        let commit = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&root)
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    None
                }
            });

        // Get author and date from the latest commit
        let (author, date) = if let Some(commit_hash) = &commit {
            let author_output = Command::new("git")
                .args(["show", "-s", "--format=%an <%ae>|%ai", commit_hash])
                .current_dir(&root)
                .output()
                .ok();

            if let Some(output) = author_output {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    let mut parts = output_str.trim().split('|');
                    let author = parts.next().map(|s| s.to_string());
                    let date_str = parts.next();
                    let date = date_str
                        .and_then(|s| DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S %z").ok())
                        .map(|dt| dt.with_timezone(&Local));
                    (author, date)
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        // Check for uncommitted changes
        let has_uncommitted_changes = Command::new("git")
            .args(["diff-index", "--quiet", "HEAD", "--"])
            .current_dir(&root)
            .status()
            .map(|status| !status.success())
            .unwrap_or(false);

        Ok(Self {
            root,
            branch,
            commit,
            author,
            date,
            has_uncommitted_changes,
        })
    }

    /// Format the git information as a string for inclusion in the archive.
    pub fn format(&self) -> String {
        let mut parts = Vec::new();

        if let Some(branch) = &self.branch {
            parts.push(format!("Branch: {}", branch));
        }

        if let Some(commit) = &self.commit {
            parts.push(format!("Commit: {}", &commit[..8]));
        }

        if let Some(author) = &self.author {
            parts.push(format!("Author: {}", author));
        }

        if let Some(date) = &self.date {
            parts.push(format!("Date: {}", date.format("%Y-%m-%d %H:%M:%S %z")));
        }

        if self.has_uncommitted_changes {
            parts.push("Warning: Uncommitted changes present".to_string());
        }

        parts.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;

    #[test]
    fn test_git_info() {
        // Create a temporary directory and initialize a git repo
        let dir = tempdir().unwrap();
        let repo_path = dir.path();

        // Initialize git repository
        Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .status()
            .unwrap();

        // Create a test file and commit it
        let test_file = repo_path.join("test.txt");
        std::fs::write(&test_file, "test").unwrap();

        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(repo_path)
            .status()
            .unwrap();

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(repo_path)
            .status()
            .unwrap();

        Command::new("git")
            .args(["add", "test.txt"])
            .current_dir(repo_path)
            .status()
            .unwrap();

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(repo_path)
            .status()
            .unwrap();

        // Test GitInfo
        let git_info = GitInfo::from_path(repo_path).unwrap();
        assert_eq!(git_info.branch, Some("master".to_string()));
        assert!(git_info.commit.is_some());
        assert!(git_info.author.is_some());
        assert!(git_info.date.is_some());
        assert!(!git_info.has_uncommitted_changes);

        // Test with uncommitted changes
        std::fs::write(&test_file, "modified").unwrap();
        let git_info = GitInfo::from_path(repo_path).unwrap();
        assert!(git_info.has_uncommitted_changes);
    }
}
