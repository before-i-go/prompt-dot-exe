use common::{
    error::Result,
    fs::{self, metadata},
    path::{ExtensionExt, NameExt},
};
use std::path::Path;

#[test]
fn test_path_extension() {
    let path = Path::new("test.txt");
    assert_eq!(path.extension_str(), Some("txt"));
    assert!(path.has_extension("txt"));
    assert!(!path.has_extension("rs"));
}

#[test]
fn test_file_operations() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let file_path = temp_dir.path().join("test.txt");
    
    // Test writing and reading a file
    fs::write(&file_path, "test content")?;
    let content = fs::read_to_string(&file_path)?;
    assert_eq!(content, "test content");
    
    // Test file metadata
    let meta = metadata(&file_path)?;
    assert!(meta.is_file());
    assert!(!meta.is_dir());
    assert_eq!(meta.len(), 12);
    
    // Test path name utilities
    assert_eq!(file_path.file_name_str(), Some("test.txt"));
    assert_eq!(file_path.file_stem_str(), Some("test"));
    assert!(file_path.has_file_name("test.txt"));
    assert!(file_path.has_stem("test"));
    
    Ok(())
}
