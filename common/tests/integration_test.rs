use common::{
    error::Result,
    fs::{create_dir_all, metadata, read_to_string, write},
    path::{ExtensionExt, NameExt, PathExt},
};
use tempfile::tempdir;

#[test]
fn test_integration() -> Result<()> {
    // Create a temporary directory for testing
    let temp_dir = tempdir()?;
    let test_dir = temp_dir.path();
    
    // Test 1: Create a directory
    let test_subdir = test_dir.join("test_dir");
    create_dir_all(&test_subdir)?;
    assert!(test_subdir.is_dir());
    
    // Test 2: Create a file
    let test_file = test_subdir.join("test.txt");
    let test_content = "Hello, world!";
    write(&test_file, test_content)?;
    
    // Test 3: Read the file back
    let content = read_to_string(&test_file)?;
    assert_eq!(content, test_content);
    
    // Test 4: Check file metadata
    let meta = metadata(&test_file)?;
    assert!(meta.is_file());
    assert!(!meta.is_dir());
    assert_eq!(meta.len() as usize, test_content.len());
    
    // Test 5: Path extensions
    assert_eq!(test_file.extension_str(), Some("txt"));
    assert!(test_file.has_extension("txt"));
    assert!(!test_file.has_extension("rs"));
    
    // Test 6: File name and stem
    assert_eq!(test_file.file_name_str(), Some("test.txt"));
    assert_eq!(test_file.file_stem_str(), Some("test"));
    
    // Test 7: Parent directory
    assert_eq!(test_file.parent_str(), Some(test_subdir.to_str().unwrap()));
    
    // Test 8: Path manipulation
    let new_path = test_file.with_extension("md");
    assert_eq!(new_path.extension_str(), Some("md"));
    
    // Test 9: Absolute path
    let abs_path = test_file.absolute_path()?;
    assert!(abs_path.is_absolute());
    
    // Test 10: Error handling
    let non_existent = test_dir.join("nonexistent");
    let result = read_to_string(&non_existent);
    assert!(result.is_err());
    
    Ok(())
}
