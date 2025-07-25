#![cfg(feature = "test-utils")]

use code_archiver::git::{GitContext, GitStatus};
use code_archiver::test_utils::TestGitRepo;
use std::fs;

#[test]
fn test_git_ignore() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test repository
    let test_repo = TestGitRepo::new();
    
    // Create some test files
    let tracked_file = test_repo.add_file("tracked.txt", "tracked content");
    let ignored_file = test_repo.add_file("ignored.txt", "ignored content");
    
    // Add .gitignore
    test_repo.add_to_gitignore("ignored.txt");
    
    // Commit initial files
    test_repo.commit("Initial commit");
    
    // Create GitContext
    let git_ctx = GitContext::open(&test_repo.temp_dir)?.unwrap();
    
    // Verify tracked file is not ignored
    assert!(!git_ctx.is_ignored(&tracked_file)?);
    
    // Verify ignored file is ignored
    assert!(git_ctx.is_ignored(&ignored_file)?);
    
    // Verify status
    assert_eq!(git_ctx.get_status(&tracked_file)?.unwrap(), GitStatus::Unmodified);
    
    // Modify the tracked file and check status
    fs::write(&tracked_file, "modified content")?;
    assert_eq!(git_ctx.get_status(&tracked_file)?.unwrap(), GitStatus::Modified);
    
    Ok(())
}
