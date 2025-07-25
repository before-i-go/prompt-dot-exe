use code_archiver::{ArchiveConfig, CodeArchiver};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_archive_validates_file_metadata() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory with test files
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("test.txt");
    let mut file = File::create(&file_path)?;
    writeln!(file, "Test content")?;

    // Get file metadata for verification
    let metadata = file.metadata()?;
    let expected_size = metadata.len();
    
    // Create archive
    let config = ArchiveConfig {
        root_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let archiver = CodeArchiver::new(config)?;
    let entries = archiver.create_archive()?;

    // Verify metadata
    assert_eq!(entries.len(), 1, "Should find exactly one file");
    let entry = &entries[0];
    assert_eq!(entry.path, "test.txt", "File path should match");
    assert_eq!(entry.size, expected_size, "File size should match");
    assert_eq!(entry.extension, Some("txt".to_string()), "File extension should be 'txt'");
    
    // Verify the modified time is a valid RFC3339 timestamp
    let entry_modified = chrono::DateTime::parse_from_rfc3339(&entry.modified)
        .map_err(|e| format!("Invalid RFC3339 timestamp in entry.modified: {}", e))?;
    
    // Verify the modified time is recent (within 5 minutes to be safe)
    let now = chrono::Utc::now();
    let time_diff = now.signed_duration_since(entry_modified);
    
    assert!(
        time_diff.num_seconds() < 300, 
        "Modified time should be recent (within 5 minutes). Time difference: {} seconds",
        time_diff.num_seconds()
    );

    Ok(())
}

#[test]
fn test_archive_handles_symlinks() -> Result<(), Box<dyn std::error::Error>> {
    // Skip on Windows where symlink creation might require admin privileges
    if cfg!(windows) {
        return Ok(());
    }

    let temp_dir = tempdir()?;
    
    // Create a target file
    let target_path = temp_dir.path().join("target.txt");
    std::fs::write(&target_path, "target content")?;
    
    // Create a symlink
    let link_path = temp_dir.path().join("link.txt");
    std::os::unix::fs::symlink(&target_path, &link_path)?;

    // Create archive with follow_links = false (default)
    let config = ArchiveConfig {
        root_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let archiver = CodeArchiver::new(config)?;
    let entries = archiver.create_archive()?;
    
    // Should include both the target file and the symlink
    assert_eq!(entries.len(), 2);
    
    // Sort entries for consistent ordering
    let mut entries = entries;
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    
    // Verify both files are included
    assert_eq!(entries[0].path, "link.txt");
    assert_eq!(entries[1].path, "target.txt");

    Ok(())
}

#[test]
fn test_archive_handles_permission_denied() -> Result<(), Box<dyn std::error::Error>> {
    // Skip on Windows where file permissions work differently
    if cfg!(windows) {
        return Ok(());
    }

    use std::os::unix::fs::PermissionsExt;
    
    let temp_dir = tempdir()?;
    
    // Create a readable file
    let readable_file = temp_dir.path().join("readable.txt");
    std::fs::write(&readable_file, "readable content")?;
    
    // Create a directory with restricted permissions
    let restricted_dir = temp_dir.path().join("restricted");
    std::fs::create_dir(&restricted_dir)?;
    std::fs::set_permissions(&restricted_dir, std::fs::Permissions::from_mode(0o000))?;
    
    // Should still be able to archive the readable file
    let config = ArchiveConfig {
        root_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let archiver = CodeArchiver::new(config)?;
    let entries = archiver.create_archive()?;
    
    // Should only include the readable file
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].path, "readable.txt");
    
    // Clean up (restore permissions so temp dir can be deleted)
    std::fs::set_permissions(restricted_dir, std::fs::Permissions::from_mode(0o755))?;
    
    Ok(())
}
