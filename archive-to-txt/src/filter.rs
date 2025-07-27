//! File filtering functionality for the archive-to-txt library.
//!
//! This module provides functionality to filter files based on patterns, extensions,
//! and other criteria.

use std::path::{Path, PathBuf};
use std::collections::HashSet;

use globset::{Glob, GlobSet, GlobSetBuilder};
use log::debug;

/// A filter for including/excluding files based on patterns and extensions.
#[derive(Debug, Clone)]
pub struct FileFilter {
    include_patterns: Option<GlobSet>,
    exclude_patterns: Option<GlobSet>,
    allowed_extensions: Option<HashSet<String>>,
    max_file_size: Option<u64>,
}

impl FileFilter {
    /// Create a new FileFilter with the given configuration.
    pub fn new(
        include_patterns: Option<Vec<String>>,
        exclude_patterns: Option<Vec<String>>,
        allowed_extensions: Option<HashSet<String>>,
        max_file_size: Option<u64>,
    ) -> Result<Self, globset::Error> {
        let include_patterns = if let Some(patterns) = include_patterns {
            let mut builder = GlobSetBuilder::new();
            for pattern in patterns {
                builder.add(Glob::new(&pattern)?);
            }
            Some(builder.build()?)
        } else {
            None
        };

        let exclude_patterns = if let Some(patterns) = exclude_patterns {
            let mut builder = GlobSetBuilder::new();
            for pattern in patterns {
                builder.add(Glob::new(&pattern)?);
            }
            Some(builder.build()?)
        } else {
            None
        };

        Ok(Self {
            include_patterns,
            exclude_patterns,
            allowed_extensions,
            max_file_size,
        })
    }

    /// Check if a file should be included based on the filter criteria.
    pub fn is_included<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();
        
        // Check against include patterns if any are specified
        if let Some(include_patterns) = &self.include_patterns {
            if !include_patterns.is_match(path) {
                debug!("Excluding {}: does not match include patterns", path.display());
                return false;
            }
        }

        // Check against exclude patterns if any are specified
        if let Some(exclude_patterns) = &self.exclude_patterns {
            if exclude_patterns.is_match(path) {
                debug!("Excluding {}: matches exclude patterns", path.display());
                return false;
            }
        }

        // Check file extension if allowed extensions are specified
        if let Some(allowed_exts) = &self.allowed_extensions {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if !allowed_exts.contains(ext) {
                    debug!("Excluding {}: extension '{}' not allowed", path.display(), ext);
                    return false;
                }
            } else if !allowed_exts.is_empty() {
                debug!("Excluding {}: no file extension", path.display());
                return false;
            }
        }

        // Check file size if max size is specified
        if let Some(max_size) = self.max_file_size {
            if let Ok(metadata) = std::fs::metadata(path) {
                if metadata.len() > max_size {
                    debug!(
                        "Excluding {}: file size {} exceeds maximum {} bytes",
                        path.display(),
                        metadata.len(),
                        max_size
                    );
                    return false;
                }
            }
        }

        true
    }

    /// Get the default LLM ignore patterns.
    /// 
    /// These patterns are designed to exclude common build artifacts, cache directories,
    /// and other files that are typically not useful for LLM training.
    pub fn default_llm_ignore_patterns() -> Vec<&'static str> {
        vec![
            // Version control
            "**/.git/", "**/.svn/", "**/.hg/", "**/.gitignore", "**/.gitmodules", "**/.gitattributes",
            
            // Build artifacts
            "**/target/", "**/build/", "**/dist/", "**/node_modules/", "**/__pycache__/",
            "**/*.pyc", "**/*.pyo", "**/*.pyd", "**/*.so", "**/*.dll", "**/*.dylib",
            "**/*.a", "**/*.lib", "**/*.o", "**/*.obj", "**/*.class", "**/*.jar", "**/*.war",
            
            // Package managers and dependencies
            "**/package-lock.json", "**/yarn.lock", "**/Cargo.lock", "**/Gemfile.lock",
            "**/Pipfile.lock", "**/poetry.lock", "**/yarn-error.log", "**/requirements*.txt",
            "**/requirements/*.txt", "**/constraints.txt", "**/setup.cfg", "**/setup.py",
            
            // Environment and configuration
            "**/.env", "**/.env.*", "**/.venv/", "**/venv/", "**/env/", "**/ENV/",
            "**/env.bak/", "**/venv.bak/", "**/.python-version", "**/.ruby-version",
            "**/.node-version", "**/.nvmrc", "**/.editorconfig", "**/.prettierrc",
            "**/.eslintrc*", "**/.babelrc*", "**/tsconfig.json", "**/jsconfig.json",
            
            // IDE and editor files
            "**/.idea/", "**/.vscode/", "**/*.swp", "**/*.swo", "**/*.swn",
            "**/.DS_Store", "**/Thumbs.db", "**/.vs/", "**/*.sublime-*", "**/.history/",
            "**/.vscode-test/", "**/.vscode/extensions.json", "**/.vscode/settings.json",
            
            // Logs and databases
            "**/*.log", "**/*.sqlite", "**/*.db", "**/*.sql", "**/*.sqlite3",
            "**/*.sqlite-journal", "**/*.sqlite3-journal", "**/*.db-journal",
            "**/logs/", "**/log/", "**/var/log/",
            
            // Archives and binaries
            "**/*.zip", "**/*.tar.gz", "**/*.tgz", "**/*.7z", "**/*.rar", "**/*.tar",
            "**/*.exe", "**/*.dmg", "**/*.pkg", "**/*.app", "**/*.msi", "**/*.deb",
            "**/*.rpm", "**/*.snap",
            
            // Media and binary files
            "**/*.png", "**/*.jpg", "**/*.jpeg", "**/*.gif", "**/*.bmp", "**/*.tiff",
            "**/*.ico", "**/*.svg", "**/*.mp3", "**/*.wav", "**/*.mp4", "**/*.avi",
            "**/*.mov", "**/*.wmv", "**/*.flv", "**/*.mkv", "**/*.webp", "**/*.webm",
            "**/*.woff", "**/*.woff2", "**/*.ttf", "**/*.eot", "**/*.otf",
            
            // Documents
            "**/*.pdf", "**/*.doc", "**/*.docx", "**/*.xls", "**/*.xlsx", "**/*.ppt",
            "**/*.pptx", "**/*.odt", "**/*.ods", "**/*.odp", "**/*.epub", "**/*.mobi",
            
            // Virtual machines and containers
            "**/.vagrant/", "**/*.vagrant/", "**/*.vbox", "**/*.vbox-prev", "**/Vagrantfile",
            "**/Dockerfile", "**/docker-compose*.yml", "**/.dockerignore", "**/.docker/",
            "**/compose.yml", "**/docker-compose.override.yml",
            
            // OS generated files
            "**/ehthumbs.db", "**/Thumbs.db", "**/desktop.ini", "**/$RECYCLE.BIN/",
            "**/Thumbs.db:encryptable", "**/ehthumbs_vista.db", "**/Desktop.ini",
            
            // Python specific
            "**/__pycache__/", "**/*.py[cod]", "**/*$py.class", "**/.pytest_cache/",
            "**/.mypy_cache/", "**/.pytest_cache/", "**/.coverage", "**/htmlcov/",
            "**/*.cover", "**/*.py,cover", "**/.hypothesis/", "**/.pytest/",
            
            // Node.js specific
            "**/node_modules/", "**/.npm/", "**/.yarn-integrity", "**/.yarn/cache/",
            "**/.yarn/unplugged/", "**/.yarn/build-state.yml", "**/.yarn/install-state.gz",
            "**/.pnp.*", "**/.yarnrc.yml", "**/yarn-debug.log*", "**/yarn-error.log*",
            
            // Rust specific
            "**/target/", "**/Cargo.lock", "**/*.rs.bk", "**/Cargo.toml.orig",
            
            // Java specific
            "**/.classpath", "**/.project", "**/.settings/", "**/*.class",
            "**/bin/", "**/build/", "**/out/", "**/*.iml",
            
            // Go specific
            "**/bin/", "**/pkg/", "**/vendor/", "**/go.work", "**/go.work.sum",
            
            // Web and frontend
            "**/dist/", "**/build/", "**/.next/", "**/out/", "**/.nuxt/", "**/.output/",
            "**/.svelte-kit/", "**/.astro/", "**/.cache/", "**/.parcel-cache/",
            "**/.turbo/", "**/.vercel/", "**/.netlify/", 
            
            // Testing and coverage
            "**/coverage/", "**/.nyc_output/", "**/coverage-*.lcov", "**/lcov.info",
            "**/.jest-cache/", "**/jest.config.*", "**/karma.conf.*", "**/test-results/",
            
            // Documentation
            "**/docs/_build/", "**/docs/api/", "**/site/", "**/.vuepress/", "**/storybook-static/",
            
            // Development tools
            "**/.github/", "**/.circleci/", "**/.travis.yml", "**/.gitlab-ci.yml",
            "**/Jenkinsfile", "**/azure-pipelines.yml", "**/.github/workflows/*.yaml",
            "**/.pre-commit-config.yaml", "**/.commitlintrc*", "**/.husky/",
            
            // Temporary files
            "**/*.swp", "**/*.swo", "**/*.swn", "**/*.swo", "**/*.swn", "**/*.bak",
            "**/*.backup", "**/*.tmp", "**/*.temp", "**/*~", "**/*.orig", "**/*.rej",
            
            // macOS specific
            "**/.DS_Store", "**/._*", "**/.Spotlight-V100", "**/.Trashes", "**/ehthumbs.db",
            
            // Windows specific
            "**/Thumbs.db", "**/Desktop.ini", "**/Thumbs.db:encryptable",
            
            // Linux specific
            "**/.directory", "**/.Trash-*", "**/.nfs*",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::{tempdir, TempDir};

    // Helper function to create a test file with content
    fn create_test_file(dir: &TempDir, path: &str, content: &str) -> std::path::PathBuf {
        let file_path = dir.path().join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&file_path, content).unwrap();
        file_path
    }

    #[test]
    fn test_include_patterns() {
        let filter = FileFilter::new(
            Some(vec!["src/**/*.rs".to_string(), "tests/**/*.rs".to_string()]),
            None,
            None,
            None,
        ).unwrap();

        // Test basic inclusion
        assert!(filter.is_included("src/main.rs"));
        assert!(filter.is_included("src/lib.rs"));
        assert!(filter.is_included("tests/test.rs"));
        
        // Test non-matching files
        assert!(!filter.is_included("src/main.py"));
        assert!(!filter.is_included("docs/README.md"));
        
        // Test with absolute paths
        let temp_dir = tempdir().unwrap();
        let file_path = create_test_file(&temp_dir, "src/foo.rs", "test");
        assert!(filter.is_included(&file_path));
    }

    #[test]
    fn test_exclude_patterns() {
        let filter = FileFilter::new(
            None,
            Some(vec![
                "**/target/**".to_string(),
                "**/*.log".to_string(),
                "**/node_modules/**".to_string()
            ]),
            None,
            None,
        ).unwrap();

        // Test basic exclusion
        assert!(filter.is_included("src/main.rs"));
        assert!(!filter.is_included("target/debug/foo"));
        assert!(!filter.is_included("src/target/debug/foo"));
        
        // Test multiple exclude patterns
        assert!(!filter.is_included("app.log"));
        assert!(!filter.is_included("logs/application.log"));
        assert!(!filter.is_included("node_modules/lodash/index.js"));
        
        // Test with absolute paths
        let temp_dir = tempdir().unwrap();
        let excluded_file = create_test_file(&temp_dir, "node_modules/foo/index.js", "test");
        assert!(!filter.is_included(&excluded_file));
    }

    #[test]
    fn test_extensions() {
        let mut extensions = HashSet::new();
        extensions.insert("rs".to_string());
        extensions.insert("toml".to_string());
        extensions.insert("".to_string()); // Test empty extension

        let filter = FileFilter::new(
            None,
            None,
            Some(extensions),
            None,
        ).unwrap();

        // Test allowed extensions
        assert!(filter.is_included("Cargo.toml"));
        assert!(filter.is_included("src/main.rs"));
        
        // Test disallowed extensions
        assert!(!filter.is_included("README.md"));
        assert!(!filter.is_included("script.sh"));
        
        // Test files with no extension
        assert!(filter.is_included("Dockerfile"));
        assert!(filter.is_included("Makefile"));
        
        // Test hidden files
        assert!(!filter.is_included(".gitignore"));
        assert!(!filter.is_included(".env"));
    }

    #[test]
    fn test_max_file_size() {
        let temp_dir = tempdir().unwrap();
        let small_file = create_test_file(&temp_dir, "small.txt", "small"); // 5 bytes
        let large_file = create_test_file(&temp_dir, "large.txt", "this is a large file content that exceeds the limit"); // > 20 bytes

        // Test with small max size
        let filter = FileFilter::new(
            None,
            None,
            None,
            Some(10), // 10 bytes max
        ).unwrap();

        assert!(filter.is_included(&small_file));
        assert!(!filter.is_included(&large_file));

        // Test with larger max size
        let filter = FileFilter::new(
            None,
            None,
            None,
            Some(100), // 100 bytes max
        ).unwrap();

        assert!(filter.is_included(&small_file));
        assert!(filter.is_included(&large_file));
        
        // Test with no size limit
        let filter = FileFilter::new(
            None,
            None,
            None,
            None, // No size limit
        ).unwrap();
        
        assert!(filter.is_included(&small_file));
        assert!(filter.is_included(&large_file));
    }
    
    #[test]
    fn test_combined_filters() {
        let temp_dir = tempdir().unwrap();
        
        // Create test files
        let src_file = create_test_file(&temp_dir, "src/main.rs", "fn main() {}");
        let test_file = create_test_file(&temp_dir, "tests/test.rs", "#[test] fn test() {}");
        let large_test_file = create_test_file(
            &temp_dir, 
            "tests/large_test.rs", 
            &"x".repeat(1024) // 1KB file
        );
        let config_file = create_test_file(&temp_dir, "config.toml", "key = 'value'");
        
        // Create a filter with multiple conditions
        let mut extensions = HashSet::new();
        extensions.insert("rs".to_string());
        extensions.insert("toml".to_string());
        
        let filter = FileFilter::new(
            Some(vec!["**/*.rs".to_string(), "**/*.toml".to_string()]), // Only .rs and .toml files
            Some(vec!["**/large_*".to_string()]), // Exclude large files
            Some(extensions),
            Some(1000), // Max 1000 bytes
        ).unwrap();
        
        // Test inclusion
        assert!(filter.is_included(&src_file));
        assert!(filter.is_included(&test_file));
        assert!(filter.is_included(&config_file));
        
        // Test exclusions
        assert!(!filter.is_included(&large_test_file)); // Excluded by size pattern
        assert!(!filter.is_included("src/main.py")); // Wrong extension
        assert!(!filter.is_included("README.md")); // Wrong extension
    }
    
    #[test]
    fn test_default_llm_ignore_patterns() {
        let patterns = FileFilter::default_llm_ignore_patterns();
        assert!(!patterns.is_empty());
        
        // Check some common patterns are included
        let pattern_set: std::collections::HashSet<_> = patterns.iter().cloned().collect();
        
        // Version control
        assert!(pattern_set.contains("**/.git/"));
        assert!(pattern_set.contains("**/.gitignore"));
        
        // Build artifacts
        assert!(pattern_set.contains("**/target/"));
        assert!(pattern_set.contains("**/node_modules/"));
        
        // Package managers
        assert!(pattern_set.contains("**/package-lock.json"));
        assert!(pattern_set.contains("**/yarn.lock"));
        assert!(pattern_set.contains("**/Cargo.lock"));
        
        // Environment
        assert!(pattern_set.contains("**/.env"));
        assert!(pattern_set.contains("**/.venv/"));
        
        // Test that patterns are properly formatted
        for pattern in patterns {
            assert!(!pattern.is_empty(), "Empty pattern found");
            assert!(!pattern.ends_with('/') || pattern.ends_with("**/"), 
                   "Directory pattern should end with '**/': {}", pattern);
        }
    }
}
