use code_archiver::{ArchiveConfig, CodeArchiver, ArchiveError};
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_nonexistent_root_dir() {
    let config = ArchiveConfig {
        root_dir: PathBuf::from("/nonexistent/directory"),
        ..Default::default()
    };
    
    let result = CodeArchiver::new(config);
    assert!(matches!(result, Err(ArchiveError::InvalidPath(_))));
}

#[test]
fn test_file_as_root_dir() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "test")?;
    
    let config = ArchiveConfig {
        root_dir: file_path,
        ..Default::default()
    };
    
    let result = CodeArchiver::new(config);
    assert!(matches!(result, Err(ArchiveError::InvalidPath(_))));
    
    Ok(())
}

#[test]
fn test_invalid_glob_pattern() -> Result<(), Box<dyn std::error::Error>> {
    // Create a test directory with a file
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "test")?;
    
    // Create a config with an invalid glob pattern
    let config = ArchiveConfig {
        root_dir: temp_dir.path().to_path_buf(),
        include: Some(vec![
            "**/invalid[.txt".to_string(),  // Invalid glob pattern - unmatched '['
            "valid-pattern-*.txt".to_string(),  // Valid pattern
        ]),
        ..Default::default()
    };
    
    // This should fail because of the invalid pattern
    let result = CodeArchiver::new(config);
    
    // Verify that we got an error about the invalid pattern
    match result {
        Ok(_) => panic!("Expected an error for invalid glob pattern, but got Ok"),
        Err(e) => {
            let err_str = e.to_string();
            assert!(
                err_str.contains("Pattern error") || 
                err_str.contains("PatternError") ||
                err_str.contains("invalid range pattern"),
                "Expected error about invalid glob pattern, but got: {}",
                err_str
            );
        }
    }
    
    Ok(())
}

#[test]
fn test_archive_empty_with_excludes() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "test")?;
    
    // Exclude everything
    let config = ArchiveConfig {
        root_dir: temp_dir.path().to_path_buf(),
        exclude: Some(vec!["**/*".to_string()]),
        ..Default::default()
    };
    
    let archiver = CodeArchiver::new(config)?;
    let entries = archiver.create_archive()?;
    assert!(entries.is_empty());
    
    Ok(())
}
