use git2::{Repository, Status};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("Git repository error: {0}")]
    Repository(#[from] git2::Error),
    
    #[error("Path is not in a git repository: {0}")]
    NotARepository(PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitStatus {
    Unmodified,
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Ignored,
}

impl From<Status> for GitStatus {
    fn from(status: Status) -> Self {
        if status.is_wt_new() {
            GitStatus::Untracked
        } else if status.is_index_new() {
            GitStatus::Added
        } else if status.is_wt_modified() || status.is_index_modified() {
            GitStatus::Modified
        } else if status.is_wt_deleted() || status.is_index_deleted() {
            GitStatus::Deleted
        } else if status.is_wt_renamed() || status.is_index_renamed() {
            GitStatus::Renamed
        } else if status.is_wt_typechange() || status.is_index_typechange() {
            GitStatus::Modified
        } else if status.is_ignored() {
            GitStatus::Ignored
        } else {
            GitStatus::Unmodified
        }
    }
}

impl std::fmt::Display for GitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitStatus::Unmodified => write!(f, "unmodified"),
            GitStatus::Modified => write!(f, "modified"),
            GitStatus::Added => write!(f, "added"),
            GitStatus::Deleted => write!(f, "deleted"),
            GitStatus::Renamed => write!(f, "renamed"),
            GitStatus::Copied => write!(f, "copied"),
            GitStatus::Untracked => write!(f, "untracked"),
            GitStatus::Ignored => write!(f, "ignored"),
        }
    }
}

pub struct GitContext {
    repo: Repository,
    workdir: PathBuf,
}

impl GitContext {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Option<Self>, GitError> {
        match Repository::discover(path) {
            Ok(repo) => {
                let workdir = repo.workdir()
                    .ok_or_else(|| GitError::Repository(git2::Error::from_str("Bare repositories are not supported")))?
                    .to_path_buf();
                
                Ok(Some(Self { repo, workdir }))
            }
            Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
            Err(e) => Err(GitError::Repository(e)),
        }
    }

    pub fn get_status(&self, path: &Path) -> Result<Option<GitStatus>, GitError> {
        let rel_path = path.strip_prefix(&self.workdir)
            .map_err(|_| GitError::NotARepository(path.to_path_buf()))?;
        
        let status = self.repo.status_file(rel_path)?;
        
        if status.is_empty() {
            // File is not ignored and has no changes
            Ok(Some(GitStatus::Unmodified))
        } else if status.is_ignored() {
            Ok(Some(GitStatus::Ignored))
        } else {
            Ok(Some(status.into()))
        }
    }

    pub fn is_ignored(&self, path: &Path) -> Result<bool, GitError> {
        let rel_path = path.strip_prefix(&self.workdir)
            .map_err(|_| GitError::NotARepository(path.to_path_buf()))?;
        
        self.repo.is_path_ignored(rel_path)
            .map_err(Into::into)
    }

    pub fn get_root(&self) -> &Path {
        &self.workdir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    
    // Test helper for git operations
    struct TestGitRepo {
        path: tempfile::TempDir,
        repo: Repository,
    }
    
    impl TestGitRepo {
        fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let repo = Repository::init(&dir).unwrap();
            
            // Set up git config
            let mut config = repo.config().unwrap();
            config.set_str("user.name", "Test User").unwrap();
            config.set_str("user.email", "test@example.com").unwrap();
            
            Self { 
                path: dir,
                repo,
            }
        }
        
        fn path(&self) -> &Path {
            self.path.path()
        }
        
        fn add_file(&self, path: &str, content: &str) -> std::path::PathBuf {
            use std::io::Write;
            
            let file_path = self.path.path().join(path);
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            
            let mut file = std::fs::File::create(&file_path).unwrap();
            write!(file, "{}", content).unwrap();
            file_path
        }
        
        fn commit(&self, message: &str) {
            // Stage all changes
            let mut index = self.repo.index().unwrap();
            index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None).unwrap();
            index.write().unwrap();
            
            // Get the tree from the index
            let tree_id = index.write_tree().unwrap();
            let tree = self.repo.find_tree(tree_id).unwrap();
            
            // Get the current HEAD as the parent commit, if it exists
            let parent_commit = self.repo.head().ok()
                .and_then(|head| head.target())
                .and_then(|oid| self.repo.find_commit(oid).ok());
            
            let parents: Vec<&_> = parent_commit.as_ref().into_iter().collect();
            
            // Create a commit
            let sig = self.repo.signature().unwrap();
            self.repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                message,
                &tree,
                parents.as_slice(),
            ).unwrap();
        }
    }

    #[test]
    fn test_git_context() -> Result<(), Box<dyn std::error::Error>> {
        // Setup test repository
        let test_repo = TestGitRepo::new();
        let file_path = test_repo.add_file("test.txt", "test content");
        test_repo.commit("Initial commit");

        // Create a git context
        let git_ctx = GitContext::open(test_repo.path())?.unwrap();
        
        // Test get_status on committed file
        let status = git_ctx.get_status(&file_path)?.unwrap();
        // After commit, the file should be Unmodified since it's already in the repository
        assert_eq!(status, GitStatus::Unmodified, "Committed file should be Unmodified");
        
        // Test is_ignored
        assert!(!git_ctx.is_ignored(&file_path)?);
        
        Ok(())
    }
}
