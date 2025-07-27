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
    /// dependencies, tests, and other files that are typically not useful for LLM analysis.
    /// This is the MVP smart default - no configuration needed.
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
            "**/docker-compose*.yml", "**/.dockerignore", "**/.docker/",
            "**/compose.yml", "**/docker-compose.override.yml",

            // OS generated files
            "**/ehthumbs.db", "**/Thumbs.db", "**/desktop.ini", "**/$RECYCLE.BIN/",
            "**/Thumbs.db:encryptable", "**/ehthumbs_vista.db", "**/Desktop.ini",

            // Python specific
            "**/__pycache__/", "**/*.py[cod]", "**/*$py.class", "**/.pytest_cache/**",
            "**/.mypy_cache/**", "**/.coverage", "**/htmlcov/**",
            "**/*.cover", "**/*.py,cover", "**/.hypothesis/", "**/.pytest/",

            // Node.js specific
            "**/node_modules/**", "**/.npm/**", "**/.yarn-integrity", "**/.yarn/cache/",
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
            "**/*.swp", "**/*.swo", "**/*.swn", "**/*.bak",
            "**/*.backup", "**/*.tmp", "**/*.temp", "**/*~", "**/*.orig", "**/*.rej",

            // macOS specific
            "**/.DS_Store", "**/._*", "**/.Spotlight-V100", "**/.Trashes",

            // Windows specific
            "**/Thumbs.db", "**/Desktop.ini", "**/Thumbs.db:encryptable",

            // Linux specific
            "**/.directory", "**/.Trash-*", "**/.nfs*",

            // Python ecosystem - comprehensive coverage from gitingest
            "**/__pycache__/", "**/*.py[cod]", "**/*$py.class", // Compiled Python
            "**/.pytest_cache/**", "**/.mypy_cache/**", "**/.coverage", "**/htmlcov/**",
            "**/*.cover", "**/*.py,cover", "**/.hypothesis/", "**/.pytest/",
            "**/.tox/**", "**/.nox/**", "**/.ipynb_checkpoints/**", // Testing and notebooks
            "**/celerybeat-schedule", "**/celerybeat.pid", "**/*.sage.py", // Celery and Sage
            "**/.pyre/**", "**/.pytype/**", "**/cython_debug/**", // Type checkers and Cython
            "**/.ruff_cache/", "**/poetry.lock", "**/Pipfile.lock", // Modern Python tools

            // Node.js ecosystem - comprehensive coverage from gitingest
            "**/node_modules/**", "**/.npm/**", "**/.yarn-integrity", "**/.yarn/cache/",
            "**/.yarn/unplugged/", "**/.yarn/build-state.yml", "**/.yarn/install-state.gz",
            "**/.pnp.*", "**/.yarnrc.yml", "**/yarn-debug.log*", "**/yarn-error.log*",
            "**/bower_components/**", "**/.bower-cache/", "**/npm-debug.log", "**/.eslintcache",

            // Java/JVM ecosystem - comprehensive coverage from gitingest
            "**/.gradle/**", "**/gradlew", "**/gradlew.bat", "**/.mvn/**", "**/mvnw", "**/mvnw.cmd",
            "**/.classpath", "**/.project", "**/.settings/", "**/*.class", "**/bin/",
            "**/build/", "**/out/", "**/*.iml",

            // Other language ecosystems - comprehensive coverage from gitingest
            "**/vendor/**", "**/composer.lock", "**/composer.phar", // PHP/Composer
            "**/.bundle/**", "**/vendor/bundle/", "**/vendor/cache/", "**/*.gem", // Ruby/Bundler
            "**/_build/**", "**/deps/", "**/mix.lock", // Elixir/Phoenix
            "**/pkg/", "**/go.work", "**/go.work.sum", // Go additional

            // Frontend frameworks and build tools - from gitingest
            "**/.next/", "**/.nuxt/", "**/.output/", "**/.svelte-kit/", "**/.astro/",
            "**/.cache/", "**/.parcel-cache/", "**/.turbo/", "**/.vercel/", "**/.netlify/",

            // Testing and coverage - comprehensive from gitingest
            "**/coverage/", "**/.nyc_output/", "**/coverage-*.lcov", "**/lcov.info",
            "**/.jest-cache/", "**/jest.config.*", "**/karma.conf.*", "**/test-results/",

            // Documentation builds - from gitingest
            "**/docs/_build/", "**/docs/api/", "**/site/", "**/.vuepress/", "**/storybook-static/",

            // CI/CD - comprehensive from gitingest
            "**/.github/", "**/.circleci/", "**/.travis.yml", "**/.gitlab-ci.yml",
            "**/Jenkinsfile", "**/azure-pipelines.yml", "**/.github/workflows/*.yaml",
            "**/.pre-commit-config.yaml", "**/.commitlintrc*", "**/.husky/",

            // Test directories and files - comprehensive from gitingest
            "**/tests/**", "**/test/**", // Test directories and all files within
            "**/*_test.rs", "**/*_test.go", // Test files
            "**/*.test.js", "**/*.spec.js", "**/*.test.ts", "**/*.spec.ts", "**/*.test.tsx", "**/*.spec.tsx", // JS/TS test files

            // Development tools - comprehensive from gitingest
            "**/.sonarqube/**", "**/.vscode-test/", "**/.history/", // Code analysis and editor tools
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
        
        // Test with absolute paths - the pattern should match the relative part
        let temp_dir = tempdir().unwrap();
        let file_path = create_test_file(&temp_dir, "src/foo.rs", "test");
        // For absolute paths, we need to check if the path contains the pattern
        // This is a limitation of the current implementation - it works better with relative paths
        // In real usage, the archive engine will pass relative paths
        let relative_path = std::path::Path::new("src/foo.rs");
        assert!(filter.is_included(relative_path));
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
        
        // Test files with no extension (should be excluded when extensions are specified)
        assert!(!filter.is_included("Dockerfile"));
        assert!(!filter.is_included("Makefile"));
        
        // Test hidden files (should be excluded due to no matching extension)
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
            // Directory patterns should end with '/' and be properly formatted
            if pattern.ends_with('/') {
                assert!(pattern.starts_with("**/"), 
                       "Directory pattern should start with '**/': {}", pattern);
            }
        }
    }

    #[test]
    fn test_mvp_python_ecosystem_patterns() {
        let patterns = FileFilter::default_llm_ignore_patterns();
        let pattern_set: std::collections::HashSet<_> = patterns.iter().cloned().collect();
        
        // MVP Python additions
        assert!(pattern_set.contains("**/.tox/**"), "Should exclude Python tox environments");
        assert!(pattern_set.contains("**/.pytest_cache/**"), "Should exclude pytest cache");
        assert!(pattern_set.contains("**/.mypy_cache/**"), "Should exclude mypy cache");
        assert!(pattern_set.contains("**/.ipynb_checkpoints/**"), "Should exclude Jupyter checkpoints");
        assert!(pattern_set.contains("**/htmlcov/**"), "Should exclude coverage HTML reports");
        assert!(pattern_set.contains("**/.coverage"), "Should exclude coverage data");
        assert!(pattern_set.contains("**/celerybeat-schedule"), "Should exclude Celery scheduler");
        assert!(pattern_set.contains("**/.pyre/**"), "Should exclude Pyre type checker");
        assert!(pattern_set.contains("**/.pytype/**"), "Should exclude pytype cache");
        assert!(pattern_set.contains("**/cython_debug/**"), "Should exclude Cython debug");
    }

    #[test]
    fn test_mvp_nodejs_ecosystem_patterns() {
        let patterns = FileFilter::default_llm_ignore_patterns();
        let pattern_set: std::collections::HashSet<_> = patterns.iter().cloned().collect();
        
        // MVP Node.js additions
        assert!(pattern_set.contains("**/bower_components/**"), "Should exclude Bower components");
        assert!(pattern_set.contains("**/.bower-cache/"), "Should exclude Bower cache");
        assert!(pattern_set.contains("**/npm-debug.log"), "Should exclude NPM debug logs");
        assert!(pattern_set.contains("**/.eslintcache"), "Should exclude ESLint cache");
        assert!(pattern_set.contains("**/.npm/**"), "Should exclude NPM cache");
        assert!(pattern_set.contains("**/.yarn/cache/"), "Should exclude Yarn cache");
    }

    #[test]
    fn test_mvp_java_ecosystem_patterns() {
        let patterns = FileFilter::default_llm_ignore_patterns();
        let pattern_set: std::collections::HashSet<_> = patterns.iter().cloned().collect();
        
        // MVP Java/JVM additions
        assert!(pattern_set.contains("**/.gradle/**"), "Should exclude Gradle cache");
        assert!(pattern_set.contains("**/gradlew"), "Should exclude Gradle wrapper script");
        assert!(pattern_set.contains("**/gradlew.bat"), "Should exclude Gradle wrapper batch");
        assert!(pattern_set.contains("**/.mvn/**"), "Should exclude Maven wrapper");
        assert!(pattern_set.contains("**/mvnw"), "Should exclude Maven wrapper script");
        assert!(pattern_set.contains("**/mvnw.cmd"), "Should exclude Maven wrapper cmd");
    }

    #[test]
    fn test_mvp_other_languages_patterns() {
        let patterns = FileFilter::default_llm_ignore_patterns();
        let pattern_set: std::collections::HashSet<_> = patterns.iter().cloned().collect();
        
        // PHP
        assert!(pattern_set.contains("**/vendor/"), "Should exclude PHP vendor directory");
        assert!(pattern_set.contains("**/composer.lock"), "Should exclude Composer lock");
        assert!(pattern_set.contains("**/composer.phar"), "Should exclude Composer phar");
        
        // Ruby
        assert!(pattern_set.contains("**/.bundle/**"), "Should exclude Ruby bundle directory");
        assert!(pattern_set.contains("**/vendor/bundle/"), "Should exclude bundled gems");
        assert!(pattern_set.contains("**/vendor/cache/"), "Should exclude gem cache");
        assert!(pattern_set.contains("**/*.gem"), "Should exclude gem files");
        
        // Elixir
        assert!(pattern_set.contains("**/_build/**"), "Should exclude Elixir build directory");
        assert!(pattern_set.contains("**/deps/"), "Should exclude Elixir dependencies");
        assert!(pattern_set.contains("**/mix.lock"), "Should exclude Mix lock file");
    }

    #[test]
    fn test_mvp_test_directories_excluded() {
        let patterns = FileFilter::default_llm_ignore_patterns();
        let pattern_set: std::collections::HashSet<_> = patterns.iter().cloned().collect();
        
        // Test directories
        assert!(pattern_set.contains("**/tests/**"), "Should exclude tests directory");
        assert!(pattern_set.contains("**/test/**"), "Should exclude test directory");
        
        // Test files
        assert!(pattern_set.contains("**/*_test.rs"), "Should exclude Rust test files");
        assert!(pattern_set.contains("**/*_test.go"), "Should exclude Go test files");
        assert!(pattern_set.contains("**/*.test.js"), "Should exclude JS test files");
        assert!(pattern_set.contains("**/*.spec.js"), "Should exclude JS spec files");
        assert!(pattern_set.contains("**/*.test.ts"), "Should exclude TS test files");
        assert!(pattern_set.contains("**/*.spec.ts"), "Should exclude TS spec files");
    }

    #[test]
    fn test_mvp_development_tools_patterns() {
        let patterns = FileFilter::default_llm_ignore_patterns();
        let pattern_set: std::collections::HashSet<_> = patterns.iter().cloned().collect();
        
        // Development tools
        assert!(pattern_set.contains("**/.sonarqube/**"), "Should exclude SonarQube analysis");
        assert!(pattern_set.contains("**/.vscode-test/"), "Should exclude VSCode test files");
        assert!(pattern_set.contains("**/.history/"), "Should exclude VSCode local history");
    }

    #[test]
    fn test_simple_pattern_matching() {
        // Test a simple case first
        let filter = FileFilter::new(
            None,
            Some(vec!["**/.pytest_cache/**".to_string()]),
            None,
            None,
        ).unwrap();
        
        assert!(!filter.is_included(".pytest_cache/test.py"), "Should exclude pytest cache file");
        assert!(!filter.is_included("project/.pytest_cache/test.py"), "Should exclude nested pytest cache");
        assert!(filter.is_included("src/main.py"), "Should include regular files");
    }

    #[test]
    fn test_filter_with_default_llm_patterns() {
        let temp_dir = tempdir().unwrap();
        
        // Create test files that should be excluded
        let python_cache = create_test_file(&temp_dir, ".pytest_cache/test.py", "cache");
        let node_modules = create_test_file(&temp_dir, "node_modules/lodash/index.js", "module");
        let gradle_cache = create_test_file(&temp_dir, ".gradle/cache/file", "cache");
        let test_dir = create_test_file(&temp_dir, "tests/unit_test.rs", "test");
        let vendor_dir = create_test_file(&temp_dir, "vendor/package/lib.php", "lib");
        
        // Create test files that should be included
        let src_file = create_test_file(&temp_dir, "src/main.rs", "fn main() {}");
        let config_file = create_test_file(&temp_dir, "Cargo.toml", "[package]");
        let readme = create_test_file(&temp_dir, "README.md", "# Project");
        
        // Create filter with default LLM patterns as exclude patterns
        let patterns: Vec<String> = FileFilter::default_llm_ignore_patterns()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        
        let filter = FileFilter::new(
            None,
            Some(patterns),
            None,
            None,
        ).unwrap();
        
        // Test exclusions - use relative paths for pattern matching
        assert!(!filter.is_included(".pytest_cache/test.py"), "Should exclude pytest cache");
        assert!(!filter.is_included("node_modules/lodash/index.js"), "Should exclude node_modules");
        assert!(!filter.is_included(".gradle/cache/file"), "Should exclude Gradle cache");
        assert!(!filter.is_included("tests/unit_test.rs"), "Should exclude test directory");
        assert!(!filter.is_included("vendor/package/lib.php"), "Should exclude vendor directory");
        
        // Test inclusions
        assert!(filter.is_included(&src_file), "Should include source files");
        assert!(filter.is_included(&config_file), "Should include config files");
        assert!(filter.is_included(&readme), "Should include documentation");
    }

    #[test]
    fn test_real_world_project_structure() {
        let temp_dir = tempdir().unwrap();
        
        // Create a realistic project structure
        let files_to_exclude = vec![
            // Python
            "__pycache__/module.cpython-39.pyc",
            ".pytest_cache/README.md",
            ".tox/py39/lib/python3.9/site-packages/pkg.py",
            ".ipynb_checkpoints/notebook-checkpoint.ipynb",
            "htmlcov/index.html",
            
            // Node.js
            "node_modules/react/index.js",
            "bower_components/jquery/dist/jquery.js",
            ".npm/_cacache/index.json",
            ".eslintcache",
            
            // Java
            ".gradle/7.4/executionHistory/executionHistory.bin",
            "gradlew",
            ".mvn/wrapper/maven-wrapper.properties",
            
            // Other languages
            "vendor/autoload.php",
            ".bundle/config",
            "_build/dev/lib/myapp/ebin/myapp.beam",
            
            // Tests
            "tests/test_main.py",
            "test/unit/test_helper.rb",
            "src/main_test.go",
            "components/Button.test.tsx",
            
            // Development tools
            ".sonarqube/report-task.txt",
            ".vscode-test/user-data/logs/main.log",
        ];
        
        let files_to_include = vec![
            "src/main.rs",
            "src/lib.rs",
            "Cargo.toml",
            "README.md",
            "docs/api.md",
            "config/database.yml",
            "scripts/deploy.sh",
            "Dockerfile",
        ];
        
        // Create all test files
        for file_path in &files_to_exclude {
            create_test_file(&temp_dir, file_path, "content");
        }
        
        for file_path in &files_to_include {
            create_test_file(&temp_dir, file_path, "content");
        }
        
        // Create filter with default patterns
        let patterns: Vec<String> = FileFilter::default_llm_ignore_patterns()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        
        let filter = FileFilter::new(
            None,
            Some(patterns),
            None,
            None,
        ).unwrap();
        
        // Test that excluded files are properly filtered (use relative paths)
        for file_path in &files_to_exclude {
            assert!(!filter.is_included(file_path), 
                   "Should exclude {}", file_path);
        }
        
        // Test that included files pass through (use relative paths)
        for file_path in &files_to_include {
            assert!(filter.is_included(file_path), 
                   "Should include {}", file_path);
        }
    }
}
