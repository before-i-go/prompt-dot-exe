//! Directory tree generation functionality.
//!
//! This module provides utilities for generating ASCII-based directory tree representations
//! similar to the Unix `tree` command output.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Represents a node in the directory tree
#[derive(Debug, Clone)]
pub struct TreeNode {
    /// The name of the file or directory
    pub name: String,
    /// Whether this is a directory
    pub is_dir: bool,
    /// Child nodes (only for directories)
    pub children: BTreeMap<String, TreeNode>,
    /// The full path to this node
    pub path: PathBuf,
}

impl TreeNode {
    /// Create a new tree node
    pub fn new(name: String, is_dir: bool, path: PathBuf) -> Self {
        Self {
            name,
            is_dir,
            children: BTreeMap::new(),
            path,
        }
    }

    /// Add a child node to this directory
    pub fn add_child(&mut self, name: String, node: TreeNode) {
        if self.is_dir {
            self.children.insert(name, node);
        }
    }

    /// Get the total number of files in this tree
    pub fn count_files(&self) -> usize {
        let mut count = if !self.is_dir { 1 } else { 0 };
        for child in self.children.values() {
            count += child.count_files();
        }
        count
    }

    /// Get the total number of directories in this tree
    pub fn count_dirs(&self) -> usize {
        let mut count = if self.is_dir { 1 } else { 0 };
        for child in self.children.values() {
            count += child.count_dirs();
        }
        count
    }
}

/// Configuration for directory tree generation
#[derive(Debug, Clone)]
pub struct TreeConfig {
    /// Include hidden files and directories
    pub include_hidden: bool,
    /// Maximum depth to traverse
    pub max_depth: Option<usize>,
    /// Follow symbolic links
    pub follow_links: bool,
    /// File extensions to include (if specified, only these will be included)
    pub include_extensions: Option<Vec<String>>,
    /// Maximum file size to include
    pub max_file_size: Option<u64>,
}

impl Default for TreeConfig {
    fn default() -> Self {
        Self {
            include_hidden: false,
            max_depth: None,
            follow_links: false,
            include_extensions: None,
            max_file_size: None,
        }
    }
}

/// Generate a directory tree structure from the given path
pub fn generate_tree(root_path: &Path, config: &TreeConfig) -> Result<TreeNode, Box<dyn std::error::Error>> {
    let root_name = root_path
        .file_name()
        .unwrap_or_else(|| root_path.as_os_str())
        .to_string_lossy()
        .to_string();
    
    let mut root = TreeNode::new(root_name, true, root_path.to_path_buf());
    
    // Configure the walker
    let mut walker = WalkDir::new(root_path)
        .min_depth(1)
        .follow_links(config.follow_links);
    
    if let Some(max_depth) = config.max_depth {
        walker = walker.max_depth(max_depth);
    }
    
    // Collect all valid entries
    let entries: Vec<_> = walker
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| {
            // Skip hidden files if not included
            if !config.include_hidden {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with('.') {
                        return false;
                    }
                }
            }
            
            // Check file size limit for files
            if entry.file_type().is_file() {
                if let Some(max_size) = config.max_file_size {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.len() > max_size {
                            return false;
                        }
                    }
                }
                
                // Check file extensions if specified
                if let Some(extensions) = &config.include_extensions {
                    if let Some(ext) = entry.path().extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        if !extensions.iter().any(|e| e.eq_ignore_ascii_case(&ext_str)) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
            }
            
            true
        })
        .collect();
    
    // Build the tree structure
    for entry in entries {
        let relative_path = entry.path().strip_prefix(root_path)?;
        add_path_to_tree(&mut root, relative_path, entry.file_type().is_dir());
    }
    
    Ok(root)
}

/// Add a path to the tree structure
fn add_path_to_tree(root: &mut TreeNode, path: &Path, is_dir: bool) {
    let components: Vec<_> = path.components().collect();
    let root_path = root.path.clone(); // Clone the root path to avoid borrow issues
    let mut current = root;
    
    for (i, component) in components.iter().enumerate() {
        let name = component.as_os_str().to_string_lossy().to_string();
        let is_last = i == components.len() - 1;
        let node_is_dir = if is_last { is_dir } else { true };
        
        if !current.children.contains_key(&name) {
            let full_path = root_path.join(path.iter().take(i + 1).collect::<PathBuf>());
            let node = TreeNode::new(name.clone(), node_is_dir, full_path);
            current.children.insert(name.clone(), node);
        }
        
        current = current.children.get_mut(&name).unwrap();
    }
}

/// Format a directory tree as ASCII art
pub fn format_tree(tree: &TreeNode, show_root: bool) -> String {
    let mut output = String::new();
    
    if show_root {
        output.push_str(&format!("└── {}/\n", tree.name));
        format_tree_recursive(tree, &mut output, "    ", true);
    } else {
        format_tree_recursive(tree, &mut output, "", false);
    }
    
    output
}

/// Recursively format the tree structure
fn format_tree_recursive(node: &TreeNode, output: &mut String, prefix: &str, skip_root: bool) {
    let children: Vec<_> = node.children.values().collect();
    
    if !skip_root && !children.is_empty() {
        // Sort children: directories first, then files, both alphabetically
        let mut sorted_children = children;
        sorted_children.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
        
        for (i, child) in sorted_children.iter().enumerate() {
            let is_last = i == sorted_children.len() - 1;
            let connector = if is_last { "└── " } else { "├── " };
            let child_prefix = if is_last { "    " } else { "│   " };
            
            let display_name = if child.is_dir {
                format!("{}/", child.name)
            } else {
                child.name.clone()
            };
            
            output.push_str(&format!("{}{}{}\n", prefix, connector, display_name));
            
            if child.is_dir && !child.children.is_empty() {
                format_tree_recursive(child, output, &format!("{}{}", prefix, child_prefix), false);
            }
        }
    } else if skip_root {
        // For root node, just process children without showing the root
        let mut sorted_children = children;
        sorted_children.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
        
        for (i, child) in sorted_children.iter().enumerate() {
            let is_last = i == sorted_children.len() - 1;
            let connector = if is_last { "└── " } else { "├── " };
            let child_prefix = if is_last { "    " } else { "│   " };
            
            let display_name = if child.is_dir {
                format!("{}/", child.name)
            } else {
                child.name.clone()
            };
            
            output.push_str(&format!("{}{}\n", connector, display_name));
            
            if child.is_dir && !child.children.is_empty() {
                format_tree_recursive(child, output, child_prefix, false);
            }
        }
    }
}

/// Generate a compact directory structure summary
pub fn generate_structure_summary(tree: &TreeNode) -> String {
    let file_count = tree.count_files();
    let dir_count = tree.count_dirs() - 1; // Subtract 1 for the root directory
    
    let mut output = String::new();
    output.push_str(&format!("Directory structure:\n"));
    output.push_str(&format_tree(tree, true));
    output.push_str(&format!("\nSummary: {} directories, {} files\n", dir_count, file_count));
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_tree_node_creation() {
        let node = TreeNode::new("test".to_string(), true, PathBuf::from("/test"));
        assert_eq!(node.name, "test");
        assert!(node.is_dir);
        assert!(node.children.is_empty());
    }

    #[test]
    fn test_tree_generation() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let root_path = temp_dir.path();
        
        // Create test structure
        fs::create_dir(root_path.join("subdir"))?;
        fs::write(root_path.join("file1.txt"), "content1")?;
        fs::write(root_path.join("subdir").join("file2.txt"), "content2")?;
        
        let config = TreeConfig::default();
        let tree = generate_tree(root_path, &config)?;
        
        assert!(tree.is_dir);
        assert!(!tree.children.is_empty());
        
        Ok(())
    }

    #[test]
    fn test_tree_formatting() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let root_path = temp_dir.path();
        
        // Create test structure
        fs::create_dir(root_path.join("subdir"))?;
        fs::write(root_path.join("file1.txt"), "content1")?;
        fs::write(root_path.join("subdir").join("file2.txt"), "content2")?;
        
        let config = TreeConfig::default();
        let tree = generate_tree(root_path, &config)?;
        let formatted = format_tree(&tree, true);
        
        assert!(formatted.contains("└──"));
        assert!(formatted.contains("file1.txt"));
        assert!(formatted.contains("subdir/"));
        
        Ok(())
    }
}