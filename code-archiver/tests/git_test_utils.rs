use assert_fs::prelude::*;
use assert_fs::TempDir;
use std::fs;
use std::path::Path;

pub struct TestGitRepo {
    pub temp_dir: TempDir,
}

impl TestGitRepo {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let _ = std::process::Command::new("git")
            .arg("init")
            .current_dir(&temp_dir)
            .output()
            .expect("Failed to initialize git repository");
        
        Self { temp_dir }
    }

    pub fn add_file(&self, path: &str, content: &str) -> std::path::PathBuf {
        let file = self.temp_dir.child(path);
        file.write_str(content).unwrap();
        file.path().to_path_buf()
    }

    pub fn add_to_gitignore(&self, pattern: &str) {
        let gitignore = self.temp_dir.child(".gitignore");
        if gitignore.exists() {
            let mut content = fs::read_to_string(gitignore.path()).unwrap();
            content.push_str(pattern);
            content.push('\n');
            fs::write(gitignore.path(), content).unwrap();
        } else {
            fs::write(gitignore.path(), format!("{}\n", pattern)).unwrap();
        }
    }

    pub fn commit(&self, message: &str) {
        let _ = std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&self.temp_dir)
            .output()
            .expect("Failed to stage files");
        
        let _ = std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&self.temp_dir)
            .output()
            .expect("Failed to set git user email");
            
        let _ = std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&self.temp_dir)
            .output()
            .expect("Failed to set git user name");
            
        let _ = std::process::Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(&self.temp_dir)
            .output()
            .expect("Failed to commit");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_repo_creation() {
        let repo = TestGitRepo::new();
        assert!(repo.temp_dir.path().join(".git").exists());
    }
}
