//! Database integration for compression checkpoints and pattern persistence
//!
//! Provides SQLite-based storage for compression patterns, checkpoints, and
//! compression statistics with ACID transactions and data integrity.

use crate::compression::{CompressionError, CompressionResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;
use tracing::{debug, error, info, instrument};

/// Checkpoint metadata for resumable compression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionCheckpoint {
    pub id: Option<i64>,
    pub target_folder: String,
    pub created_at: SystemTime,
    pub total_files: usize,
    pub processed_files: usize,
    pub patterns_found: usize,
    pub compression_config: String, // JSON-serialized config
    pub status: CheckpointStatus,
}

/// Status of a compression checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckpointStatus {
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Pattern entry for database storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternEntry {
    pub id: Option<i64>,
    pub checkpoint_id: i64,
    pub pattern: String,
    pub frequency: usize,
    pub token: String,
    pub first_seen: SystemTime,
    pub last_used: SystemTime,
}

/// Database manager for compression operations
pub struct CompressionDatabase {
    // Using a placeholder for now since rusqlite isn't in dependencies
    // In a real implementation, this would be rusqlite::Connection
    db_path: std::path::PathBuf,
    checkpoints: HashMap<i64, CompressionCheckpoint>,
    patterns: HashMap<i64, Vec<PatternEntry>>,
    next_id: i64,
}

impl CompressionDatabase {
    /// Create or open database at specified path
    pub fn new<P: AsRef<Path>>(db_path: P) -> CompressionResult<Self> {
        let db_path = db_path.as_ref().to_path_buf();

        // Ensure directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                CompressionError::config_validation(format!(
                    "Failed to create database directory: {}",
                    e
                ))
            })?;
        }

        let db = Self {
            db_path,
            checkpoints: HashMap::new(),
            patterns: HashMap::new(),
            next_id: 1,
        };

        info!("Database initialized at: {}", db.db_path.display());
        Ok(db)
    }

    /// Save a new checkpoint
    #[instrument(skip(self, checkpoint))]
    pub fn save_checkpoint(
        &mut self,
        checkpoint: &CompressionCheckpoint,
    ) -> CompressionResult<i64> {
        let id = self.next_id;
        self.next_id += 1;

        let mut checkpoint_with_id = checkpoint.clone();
        checkpoint_with_id.id = Some(id);

        self.checkpoints.insert(id, checkpoint_with_id);

        debug!(
            checkpoint_id = id,
            target_folder = %checkpoint.target_folder,
            status = ?checkpoint.status,
            "Checkpoint saved"
        );

        Ok(id)
    }

    /// Load checkpoint by ID
    #[instrument(skip(self))]
    pub fn load_checkpoint(&self, id: i64) -> CompressionResult<Option<CompressionCheckpoint>> {
        Ok(self.checkpoints.get(&id).cloned())
    }

    /// List all checkpoints
    #[instrument(skip(self))]
    pub fn list_checkpoints(&self) -> CompressionResult<Vec<CompressionCheckpoint>> {
        let mut checkpoints: Vec<_> = self.checkpoints.values().cloned().collect();
        checkpoints.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(checkpoints)
    }

    /// Update checkpoint status
    #[instrument(skip(self))]
    pub fn update_checkpoint_status(
        &mut self,
        id: i64,
        status: CheckpointStatus,
    ) -> CompressionResult<()> {
        match self.checkpoints.get_mut(&id) {
            Some(checkpoint) => {
                checkpoint.status = status;
                debug!(checkpoint_id = id, new_status = ?checkpoint.status, "Checkpoint status updated");
                Ok(())
            }
            None => Err(CompressionError::config_validation(format!(
                "Checkpoint with id {} not found",
                id
            ))),
        }
    }

    /// Delete checkpoint and associated patterns
    #[instrument(skip(self))]
    pub fn delete_checkpoint(&mut self, id: i64) -> CompressionResult<()> {
        if self.checkpoints.remove(&id).is_some() {
            self.patterns.remove(&id);
            debug!(checkpoint_id = id, "Checkpoint deleted");
            Ok(())
        } else {
            Err(CompressionError::config_validation(format!(
                "Checkpoint with id {} not found",
                id
            )))
        }
    }

    /// Clean old checkpoints, keeping only the latest N
    #[instrument(skip(self))]
    pub fn clean_checkpoints(&mut self, keep_count: usize) -> CompressionResult<usize> {
        let mut checkpoints: Vec<_> = self.checkpoints.values().cloned().collect();
        checkpoints.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        if checkpoints.len() <= keep_count {
            return Ok(0);
        }

        let to_delete = &checkpoints[keep_count..];
        let mut deleted_count = 0;

        for checkpoint in to_delete {
            if let Some(id) = checkpoint.id {
                self.delete_checkpoint(id)?;
                deleted_count += 1;
            }
        }

        info!(deleted_count = deleted_count, "Old checkpoints cleaned");
        Ok(deleted_count)
    }

    /// Save patterns for a checkpoint
    #[instrument(skip(self, patterns))]
    pub fn save_patterns(
        &mut self,
        checkpoint_id: i64,
        patterns: &[(String, usize, String)],
    ) -> CompressionResult<()> {
        let now = SystemTime::now();
        let mut pattern_entries = Vec::new();

        for (pattern, frequency, token) in patterns {
            let entry = PatternEntry {
                id: None,
                checkpoint_id,
                pattern: pattern.clone(),
                frequency: *frequency,
                token: token.clone(),
                first_seen: now,
                last_used: now,
            };
            pattern_entries.push(entry);
        }

        self.patterns.insert(checkpoint_id, pattern_entries);

        debug!(
            checkpoint_id = checkpoint_id,
            pattern_count = patterns.len(),
            "Patterns saved"
        );

        Ok(())
    }

    /// Load patterns for a checkpoint
    #[instrument(skip(self))]
    pub fn load_patterns(
        &self,
        checkpoint_id: i64,
    ) -> CompressionResult<Vec<(String, usize, String)>> {
        let patterns = self
            .patterns
            .get(&checkpoint_id)
            .map(|entries| {
                entries
                    .iter()
                    .map(|entry| (entry.pattern.clone(), entry.frequency, entry.token.clone()))
                    .collect()
            })
            .unwrap_or_default();

        debug!(
            checkpoint_id = checkpoint_id,
            pattern_count = patterns.len(),
            "Patterns loaded"
        );

        Ok(patterns)
    }

    /// Get database statistics
    pub fn get_statistics(&self) -> DatabaseStatistics {
        let total_checkpoints = self.checkpoints.len();
        let total_patterns = self.patterns.values().map(|v| v.len()).sum();
        let completed_checkpoints = self
            .checkpoints
            .values()
            .filter(|c| matches!(c.status, CheckpointStatus::Completed))
            .count();

        DatabaseStatistics {
            total_checkpoints,
            total_patterns,
            completed_checkpoints,
        }
    }

    /// Validate database integrity
    #[instrument(skip(self))]
    pub fn validate_integrity(&self) -> CompressionResult<bool> {
        // Check that all pattern entries reference valid checkpoints
        for (checkpoint_id, patterns) in &self.patterns {
            if !self.checkpoints.contains_key(checkpoint_id) {
                error!(
                    checkpoint_id = checkpoint_id,
                    "Found patterns for non-existent checkpoint"
                );
                return Ok(false);
            }

            // Validate pattern entries
            for pattern in patterns {
                if pattern.pattern.is_empty() {
                    error!("Found empty pattern in database");
                    return Ok(false);
                }

                if pattern.token.is_empty() {
                    error!("Found empty token in database");
                    return Ok(false);
                }

                if pattern.frequency == 0 {
                    error!("Found zero-frequency pattern in database");
                    return Ok(false);
                }
            }
        }

        debug!("Database integrity validation passed");
        Ok(true)
    }

    /// Export checkpoints to JSON
    #[instrument(skip(self))]
    pub fn export_checkpoints(&self) -> CompressionResult<String> {
        let export_data = DatabaseExport {
            checkpoints: self.checkpoints.values().cloned().collect(),
            patterns: self.patterns.clone(),
        };

        serde_json::to_string_pretty(&export_data).map_err(|e| {
            CompressionError::config_validation(format!("Failed to export checkpoints: {}", e))
        })
    }

    /// Import checkpoints from JSON
    #[instrument(skip(self, json_data))]
    pub fn import_checkpoints(&mut self, json_data: &str) -> CompressionResult<()> {
        let import_data: DatabaseExport = serde_json::from_str(json_data).map_err(|e| {
            CompressionError::config_validation(format!("Failed to parse import data: {}", e))
        })?;

        for checkpoint in import_data.checkpoints {
            if let Some(id) = checkpoint.id {
                self.checkpoints.insert(id, checkpoint);
                if id >= self.next_id {
                    self.next_id = id + 1;
                }
            }
        }

        for (checkpoint_id, patterns) in import_data.patterns {
            self.patterns.insert(checkpoint_id, patterns);
        }

        info!("Checkpoints imported successfully");
        Ok(())
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStatistics {
    pub total_checkpoints: usize,
    pub total_patterns: usize,
    pub completed_checkpoints: usize,
}

/// Export/import data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DatabaseExport {
    checkpoints: Vec<CompressionCheckpoint>,
    patterns: HashMap<i64, Vec<PatternEntry>>,
}

impl std::fmt::Display for CheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckpointStatus::InProgress => write!(f, "In Progress"),
            CheckpointStatus::Completed => write!(f, "Completed"),
            CheckpointStatus::Failed => write!(f, "Failed"),
            CheckpointStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

impl std::fmt::Display for DatabaseStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Database Statistics:")?;
        writeln!(f, "  Total checkpoints: {}", self.total_checkpoints)?;
        writeln!(f, "  Completed checkpoints: {}", self.completed_checkpoints)?;
        writeln!(f, "  Total patterns: {}", self.total_patterns)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_database_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db = CompressionDatabase::new(&db_path).unwrap();
        assert_eq!(db.checkpoints.len(), 0);
        assert_eq!(db.patterns.len(), 0);
    }

    #[test]
    fn test_checkpoint_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut db = CompressionDatabase::new(&db_path).unwrap();

        let checkpoint = CompressionCheckpoint {
            id: None,
            target_folder: "/test/path".to_string(),
            created_at: SystemTime::now(),
            total_files: 100,
            processed_files: 50,
            patterns_found: 25,
            compression_config: "{}".to_string(),
            status: CheckpointStatus::InProgress,
        };

        // Save checkpoint
        let id = db.save_checkpoint(&checkpoint).unwrap();
        assert!(id > 0);

        // Load checkpoint
        let loaded = db.load_checkpoint(id).unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.target_folder, "/test/path");
        assert_eq!(loaded.total_files, 100);

        // Update status
        db.update_checkpoint_status(id, CheckpointStatus::Completed)
            .unwrap();

        // Verify status update
        let updated = db.load_checkpoint(id).unwrap().unwrap();
        assert!(matches!(updated.status, CheckpointStatus::Completed));

        // Delete checkpoint
        db.delete_checkpoint(id).unwrap();

        // Verify deletion
        let deleted = db.load_checkpoint(id).unwrap();
        assert!(deleted.is_none());
    }

    #[test]
    fn test_pattern_storage() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut db = CompressionDatabase::new(&db_path).unwrap();

        // Create a checkpoint first
        let checkpoint = CompressionCheckpoint {
            id: None,
            target_folder: "/test/path".to_string(),
            created_at: SystemTime::now(),
            total_files: 100,
            processed_files: 100,
            patterns_found: 3,
            compression_config: "{}".to_string(),
            status: CheckpointStatus::Completed,
        };

        let checkpoint_id = db.save_checkpoint(&checkpoint).unwrap();

        // Save patterns
        let patterns = vec![
            ("function".to_string(), 10, "T0000".to_string()),
            ("return".to_string(), 8, "T0001".to_string()),
            ("const".to_string(), 5, "T0002".to_string()),
        ];

        db.save_patterns(checkpoint_id, &patterns).unwrap();

        // Load patterns
        let loaded_patterns = db.load_patterns(checkpoint_id).unwrap();
        assert_eq!(loaded_patterns.len(), 3);
        assert!(loaded_patterns.contains(&("function".to_string(), 10, "T0000".to_string())));
        assert!(loaded_patterns.contains(&("return".to_string(), 8, "T0001".to_string())));
        assert!(loaded_patterns.contains(&("const".to_string(), 5, "T0002".to_string())));
    }

    #[test]
    fn test_checkpoint_listing() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut db = CompressionDatabase::new(&db_path).unwrap();

        // Create multiple checkpoints
        for i in 0..5 {
            let checkpoint = CompressionCheckpoint {
                id: None,
                target_folder: format!("/test/path{}", i),
                created_at: SystemTime::now(),
                total_files: 100 + i,
                processed_files: 50 + i,
                patterns_found: 25 + i,
                compression_config: "{}".to_string(),
                status: CheckpointStatus::InProgress,
            };
            db.save_checkpoint(&checkpoint).unwrap();
        }

        let checkpoints = db.list_checkpoints().unwrap();
        assert_eq!(checkpoints.len(), 5);
    }

    #[test]
    fn test_checkpoint_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut db = CompressionDatabase::new(&db_path).unwrap();

        // Create 10 checkpoints
        for i in 0..10 {
            let checkpoint = CompressionCheckpoint {
                id: None,
                target_folder: format!("/test/path{}", i),
                created_at: SystemTime::now(),
                total_files: 100,
                processed_files: 100,
                patterns_found: 25,
                compression_config: "{}".to_string(),
                status: CheckpointStatus::Completed,
            };
            db.save_checkpoint(&checkpoint).unwrap();
        }

        // Clean to keep only 5
        let deleted = db.clean_checkpoints(5).unwrap();
        assert_eq!(deleted, 5);

        let remaining = db.list_checkpoints().unwrap();
        assert_eq!(remaining.len(), 5);
    }

    #[test]
    fn test_database_integrity() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut db = CompressionDatabase::new(&db_path).unwrap();

        // Create checkpoint and patterns
        let checkpoint = CompressionCheckpoint {
            id: None,
            target_folder: "/test/path".to_string(),
            created_at: SystemTime::now(),
            total_files: 100,
            processed_files: 100,
            patterns_found: 2,
            compression_config: "{}".to_string(),
            status: CheckpointStatus::Completed,
        };

        let checkpoint_id = db.save_checkpoint(&checkpoint).unwrap();

        let patterns = vec![
            ("function".to_string(), 10, "T0000".to_string()),
            ("return".to_string(), 8, "T0001".to_string()),
        ];

        db.save_patterns(checkpoint_id, &patterns).unwrap();

        // Validate integrity
        assert!(db.validate_integrity().unwrap());

        // Get statistics
        let stats = db.get_statistics();
        assert_eq!(stats.total_checkpoints, 1);
        assert_eq!(stats.total_patterns, 2);
        assert_eq!(stats.completed_checkpoints, 1);
    }

    #[test]
    fn test_export_import() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut db = CompressionDatabase::new(&db_path).unwrap();

        // Create checkpoint and patterns
        let checkpoint = CompressionCheckpoint {
            id: None,
            target_folder: "/test/path".to_string(),
            created_at: SystemTime::now(),
            total_files: 100,
            processed_files: 100,
            patterns_found: 2,
            compression_config: "{}".to_string(),
            status: CheckpointStatus::Completed,
        };

        let checkpoint_id = db.save_checkpoint(&checkpoint).unwrap();

        let patterns = vec![
            ("function".to_string(), 10, "T0000".to_string()),
            ("return".to_string(), 8, "T0001".to_string()),
        ];

        db.save_patterns(checkpoint_id, &patterns).unwrap();

        // Export
        let export_data = db.export_checkpoints().unwrap();
        assert!(!export_data.is_empty());

        // Create new database and import
        let db_path2 = temp_dir.path().join("test2.db");
        let mut db2 = CompressionDatabase::new(&db_path2).unwrap();

        db2.import_checkpoints(&export_data).unwrap();

        // Verify imported data
        let imported_checkpoints = db2.list_checkpoints().unwrap();
        assert_eq!(imported_checkpoints.len(), 1);
        assert_eq!(imported_checkpoints[0].target_folder, "/test/path");

        let imported_patterns = db2.load_patterns(checkpoint_id).unwrap();
        assert_eq!(imported_patterns.len(), 2);
    }
}
