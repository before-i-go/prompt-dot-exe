use archive_to_txt::config::Config;
use archive_to_txt::ArchiveEngine;
use assert_fs::prelude::*;

#[test]
fn test_parallel_processing() -> Result<(), Box<dyn std::error::Error>> {
    // Create test directory with multiple files
    let temp_dir = assert_fs::TempDir::new()?;
    for i in 0..10 {
        let file = temp_dir.child(format!("test_{}.txt", i));
        file.write_str(&format!("Content {}", i))?;
    }

    let output_file = temp_dir.path().join("archive.txt");
    let config = Config::default()
        .with_input(temp_dir.path().to_path_buf())
        .with_output(output_file.clone())
        .with_include_hidden(true)  // Ensure we include all files
        .with_parallel(true);

    // Create and run the archive engine
    let engine = ArchiveEngine::new(config);
    engine.run()?;

    // Verify all files were processed
    let content = std::fs::read_to_string(&output_file)?;
    for i in 0..10 {
        let filename = format!("test_{}.txt", i);
        let content_str = format!("Content {}", i);
        assert!(
            content.contains(&filename),
            "File {} not found in output",
            filename
        );
        assert!(
            content.contains(&content_str),
            "Content '{}' not found in output",
            content_str
        );
    }

    Ok(())
}
