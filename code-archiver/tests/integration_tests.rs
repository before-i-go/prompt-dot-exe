use assert_fs::prelude::*;
use code_archiver::{ArchiveConfig, CodeArchiver};
use code_archiver::git::GitStatus;
use std::path::PathBuf;

#[test]
fn test_archive_with_nested_directories() -> anyhow::Result<()> {
    // Setup test directory structure
    let temp = assert_fs::TempDir::new()?;
    let dir1 = temp.child("dir1");
    dir1.create_dir_all()?;
    dir1.child("file1.txt").write_str("test content")?;
    
    let dir2 = dir1.child("nested");
    dir2.create_dir_all()?;
    dir2.child("file2.rs").write_str("fn main() {}")?;
    
    // Create archive
    let config = ArchiveConfig {
        root_dir: temp.path().to_path_buf(),
        extensions: Some(vec!["txt".to_string(), "rs".to_string()]),
        ..Default::default()
    };
    let archiver = CodeArchiver::new(config)?;
    let archive = archiver.create_archive()?;
    
    // Verify results
    assert_eq!(archive.len(), 2);
    
    let has_txt = archive.iter().any(|e| e.path.ends_with("file1.txt"));
    let has_rs = archive.iter().any(|e| e.path.ends_with("file2.rs"));
    
    assert!(has_txt, "Expected to find file1.txt in archive");
    assert!(has_rs, "Expected to find file2.rs in archive");
    
    Ok(())
}

#[test]
fn test_archive_with_size_filtering() -> anyhow::Result<()> {
    // Setup test files with different sizes
    let temp = assert_fs::TempDir::new()?;
    
    // Create files with different sizes
    temp.child("small.txt").write_str("small")?;  // 5 bytes
    temp.child("large.txt").write_str("this is a larger file that exceeds the minimum size")?;  // > 10 bytes
    
    // Test with maximum size filter
    let config = ArchiveConfig {
        root_dir: temp.path().to_path_buf(),
        max_size: Some(5),  // Only include files <= 5 bytes
        ..Default::default()
    };
    let archiver = CodeArchiver::new(config)?;
    let archive = archiver.create_archive()?;
    
    // Should only include the small file (5 bytes)
    assert_eq!(archive.len(), 1);
    assert!(archive[0].path.ends_with("small.txt"));
    
    // Test with maximum size filter
    let config = ArchiveConfig {
        root_dir: temp.path().to_path_buf(),
        max_size: Some(10),  // Files larger than 10 bytes will be excluded
        ..Default::default()
    };
    let archiver = CodeArchiver::new(config)?;
    let archive = archiver.create_archive()?;
    
    // Should only include the small file
    assert_eq!(archive.len(), 1);
    assert!(archive[0].path.ends_with("small.txt"));
    
    Ok(())
}

#[test]
fn test_archive_with_extension_filtering() -> anyhow::Result<()> {
    // Setup test files with different extensions
    let temp = assert_fs::TempDir::new()?;
    temp.child("file1.rs").touch()?;
    temp.child("file2.txt").touch()?;
    temp.child("file3.md").touch()?;
    
    // Include only .rs and .md files
    let config = ArchiveConfig {
        root_dir: temp.path().to_path_buf(),
        extensions: Some(vec!["rs".to_string(), "md".to_string()]),
        ..Default::default()
    };
    let archiver = CodeArchiver::new(config)?;
    let archive = archiver.create_archive()?;
    
    // Should only include .rs and .md files
    assert_eq!(archive.len(), 2);
    let has_rs = archive.iter().any(|e| e.path.ends_with(".rs"));
    let has_md = archive.iter().any(|e| e.path.ends_with(".md"));
    
    assert!(has_rs, "Expected to find .rs file in archive");
    assert!(has_md, "Expected to find .md file in archive");
    
    Ok(())
}

#[test]
fn test_archive_with_exclude_patterns() -> anyhow::Result<()> {
    // Setup test directory structure
    let temp = assert_fs::TempDir::new()?;
    temp.child("include.txt").touch()?;
    temp.child("exclude.txt").touch()?;
    temp.child("target/file.txt").touch()?;
    
    // Exclude specific files and directories
    let config = ArchiveConfig {
        root_dir: temp.path().to_path_buf(),
        exclude: Some(vec!["**/exclude.txt".to_string(), "**/target/**".to_string()]),
        ..Default::default()
    };
    let archiver = CodeArchiver::new(config)?;
    let archive = archiver.create_archive()?;
    
    // Should only include include.txt
    assert_eq!(archive.len(), 1);
    assert!(archive[0].path.ends_with("include.txt"));
    
    Ok(())
}

#[test]
fn test_archive_with_git_integration() -> anyhow::Result<()> {
    // Setup a git repository
    let temp = assert_fs::TempDir::new()?;
    let repo = git2::Repository::init(temp.path())?;
    
    // Create a test file and add it to git
    let file_path = temp.child("committed.txt");
    file_path.touch()?;
    
    let mut index = repo.index()?;
    let path = file_path.path().strip_prefix(temp.path())?;
    index.add_path(path)?;
    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;
    let sig = git2::Signature::now("Test User", "test@example.com")?;
    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;
    
    // Create an untracked file
    temp.child("untracked.txt").touch()?;
    
    // Create and commit a file
    let modified_path = temp.child("modified.txt");
    modified_path.write_str("initial content")?;
    
    // Add and commit the file
    let mut index = repo.index()?;
    let path = modified_path.path().strip_prefix(temp.path())?;
    index.add_path(path)?;
    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;
    let sig = git2::Signature::now("Test User", "test@example.com")?;
    repo.commit(Some("HEAD"), &sig, &sig, "Add modified.txt", &tree, &[&repo.head()?.peel_to_commit()?])?;
    
    // Now modify the file
    modified_path.write_str("\nmodified content")?;
    
    // Enable git integration
    let config = ArchiveConfig {
        root_dir: temp.path().to_path_buf(),
        include_git_status: true,
        ..Default::default()
    };
    let archiver = CodeArchiver::new(config)?;
    let archive = archiver.create_archive()?;
    
    // Should include all files with correct git status
    assert_eq!(archive.len(), 3);
    
    let committed = archive.iter().find(|e| e.path.ends_with("committed.txt"));
    let untracked = archive.iter().find(|e| e.path.ends_with("untracked.txt"));
    let modified = archive.iter().find(|e| e.path.ends_with("modified.txt"));
    
    assert!(committed.is_some(), "Expected to find committed.txt");
    assert!(untracked.is_some(), "Expected to find untracked.txt");
    assert!(modified.is_some(), "Expected to find modified.txt");
    
    // Verify all expected files are present
    assert!(committed.is_some(), "Expected to find committed.txt");
    assert!(untracked.is_some(), "Expected to find untracked.txt");
    assert!(modified.is_some(), "Expected to find modified.txt");
    
    // Verify the files have valid Git statuses (not None)
    assert!(committed.unwrap().git_status.is_some(), "committed.txt should have a Git status");
    assert!(untracked.unwrap().git_status.is_some(), "untracked.txt should have a Git status");
    assert!(modified.unwrap().git_status.is_some(), "modified.txt should have a Git status");
    
    Ok(())
}
