# Design Document

## Overview

This design transforms the monolithic `UniversalCompressor` into a scalable, observable pipeline that leverages database persistence, parallel processing, and real-time progress tracking. The solution builds on the existing parallel processing infrastructure (rayon, dashmap, crossbeam) while adding SQLite for persistence and a progress reporting system.

## Architecture

### High-Level Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   CLI Layer     │───▶│  Pipeline        │───▶│  Progress       │
│                 │    │  Orchestrator    │    │  Reporter       │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │
                                ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   SQLite        │◀───│  Pattern         │───▶│  Parallel       │
│   Database      │    │  Analyzer        │    │  File           │
│                 │    │                  │    │  Processor      │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

### Core Components

1. **PipelineOrchestrator**: Coordinates the entire compression workflow
2. **PatternDatabase**: SQLite-based persistence for patterns and state
3. **ParallelAnalyzer**: Rayon-based parallel pattern analysis
4. **ProgressReporter**: Real-time progress tracking and display
5. **FileProcessor**: Concurrent file reading and processing

## Components and Interfaces

### 1. PatternDatabase

```rust
pub struct PatternDatabase {
    connection_pool: Arc<Mutex<rusqlite::Connection>>,
    cache: Arc<DashMap<String, PatternEntry>>,
}

pub struct PatternEntry {
    pub pattern: String,
    pub frequency: usize,
    pub first_seen: chrono::DateTime<chrono::Utc>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl PatternDatabase {
    pub fn new(db_path: &Path) -> Result<Self, CompressionError>;
    pub fn store_patterns(&self, patterns: Vec<PatternEntry>) -> Result<(), CompressionError>;
    pub fn get_frequent_patterns(&self, min_frequency: usize) -> Result<Vec<PatternEntry>, CompressionError>;
    pub fn save_checkpoint(&self, state: &PipelineState) -> Result<(), CompressionError>;
    pub fn load_checkpoint(&self) -> Result<Option<PipelineState>, CompressionError>;
}
```

### 2. PipelineOrchestrator

```rust
pub struct PipelineOrchestrator {
    config: CompressionConfig,
    database: Arc<PatternDatabase>,
    progress_reporter: Arc<ProgressReporter>,
    file_processor: ParallelFileProcessor,
}

#[derive(Debug, Clone)]
pub struct PipelineState {
    pub phase: PipelinePhase,
    pub files_processed: usize,
    pub total_files: usize,
    pub patterns_found: usize,
    pub current_file: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum PipelinePhase {
    Initializing,
    ScanningFiles,
    AnalyzingPatterns,
    BuildingDictionary,
    Compressing,
    Completed,
}

impl PipelineOrchestrator {
    pub fn new(config: CompressionConfig) -> Result<Self, CompressionError>;
    pub fn run(&mut self) -> Result<CompressionResult, CompressionError>;
    pub fn resume_from_checkpoint(&mut self) -> Result<CompressionResult, CompressionError>;
}
```

### 3. ParallelAnalyzer

```rust
pub struct ParallelAnalyzer {
    min_pattern_length: usize,
    min_frequency: usize,
    database: Arc<PatternDatabase>,
    progress_reporter: Arc<ProgressReporter>,
}

impl ParallelAnalyzer {
    pub fn analyze_files(&self, files: Vec<PathBuf>) -> Result<Vec<PatternEntry>, CompressionError> {
        // Use rayon to process files in parallel
        files.par_iter()
            .map(|file| self.analyze_single_file(file))
            .try_reduce(|| Vec::new(), |mut acc, patterns| {
                acc.extend(patterns);
                Ok(acc)
            })
    }
    
    fn analyze_single_file(&self, file: &Path) -> Result<Vec<PatternEntry>, CompressionError>;
    fn merge_pattern_frequencies(&self, patterns: Vec<PatternEntry>) -> Vec<PatternEntry>;
}
```

### 4. ProgressReporter

```rust
pub struct ProgressReporter {
    sender: crossbeam_channel::Sender<ProgressUpdate>,
    receiver: crossbeam_channel::Receiver<ProgressUpdate>,
    current_state: Arc<Mutex<PipelineState>>,
}

#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub phase: PipelinePhase,
    pub current_file: Option<PathBuf>,
    pub files_processed: usize,
    pub total_files: usize,
    pub patterns_found: usize,
    pub bytes_processed: usize,
    pub estimated_remaining: Option<Duration>,
}

impl ProgressReporter {
    pub fn new() -> Self;
    pub fn start_display_thread(&self);
    pub fn update(&self, update: ProgressUpdate);
    pub fn finish(&self, final_stats: CompressionStatistics);
}
```

### 5. ParallelFileProcessor

```rust
pub struct ParallelFileProcessor {
    chunk_size: usize,
    memory_map_threshold: usize,
    progress_reporter: Arc<ProgressReporter>,
}

impl ParallelFileProcessor {
    pub fn process_files(&self, files: Vec<PathBuf>) -> Result<Vec<FileContent>, CompressionError> {
        files.par_chunks(self.chunk_size)
            .map(|chunk| self.process_chunk(chunk))
            .try_reduce(|| Vec::new(), |mut acc, chunk_results| {
                acc.extend(chunk_results);
                Ok(acc)
            })
    }
    
    fn process_chunk(&self, files: &[PathBuf]) -> Result<Vec<FileContent>, CompressionError>;
    fn read_file_content(&self, file: &Path) -> Result<FileContent, CompressionError>;
}
```

## Data Models

### Database Schema

```sql
-- Patterns table for storing discovered patterns
CREATE TABLE patterns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern TEXT NOT NULL UNIQUE,
    frequency INTEGER NOT NULL,
    first_seen DATETIME NOT NULL,
    last_updated DATETIME NOT NULL,
    file_count INTEGER DEFAULT 1
);

-- Checkpoints table for resumable operations
CREATE TABLE checkpoints (
    id INTEGER PRIMARY KEY,
    phase TEXT NOT NULL,
    files_processed INTEGER NOT NULL,
    total_files INTEGER NOT NULL,
    patterns_found INTEGER NOT NULL,
    current_file TEXT,
    created_at DATETIME NOT NULL
);

-- Files table for tracking processed files
CREATE TABLE processed_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path TEXT NOT NULL UNIQUE,
    file_size INTEGER NOT NULL,
    patterns_found INTEGER NOT NULL,
    processed_at DATETIME NOT NULL
);

-- Indexes for performance
CREATE INDEX idx_patterns_frequency ON patterns(frequency DESC);
CREATE INDEX idx_patterns_updated ON patterns(last_updated);
CREATE INDEX idx_checkpoints_created ON checkpoints(created_at DESC);
```

### Configuration Extensions

```rust
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub database_path: PathBuf,
    pub checkpoint_interval: Duration,
    pub progress_update_interval: Duration,
    pub batch_size: usize,
    pub memory_limit_mb: usize,
    pub enable_resume: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from("compression_patterns.db"),
            checkpoint_interval: Duration::from_secs(30),
            progress_update_interval: Duration::from_millis(100),
            batch_size: 1000,
            memory_limit_mb: 1024,
            enable_resume: true,
        }
    }
}
```

## Error Handling

### Enhanced Error Types

```rust
#[derive(thiserror::Error, Debug)]
pub enum PipelineError {
    #[error("Database operation failed: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Checkpoint save failed: {message}")]
    CheckpointSave { message: String },
    
    #[error("Resume operation failed: {message}")]
    ResumeFailed { message: String },
    
    #[error("Parallel processing error: {message}")]
    ParallelProcessing { message: String },
    
    #[error("Memory limit exceeded: {current_mb}MB > {limit_mb}MB")]
    MemoryLimitExceeded { current_mb: usize, limit_mb: usize },
}
```

## Testing Strategy

### Unit Testing Approach

1. **Database Layer**: Test pattern storage, retrieval, and checkpoint functionality with in-memory SQLite
2. **Parallel Processing**: Test rayon-based processing with small file sets and mock data
3. **Progress Reporting**: Test progress updates and display formatting
4. **Error Handling**: Test recovery from various failure scenarios
5. **Integration**: Test full pipeline with temporary directories and sample codebases

### Performance Testing

1. **Scalability**: Test with varying numbers of files (100, 1K, 10K, 100K files)
2. **Memory Usage**: Monitor memory consumption during large file processing
3. **CPU Utilization**: Verify parallel processing utilizes all available cores
4. **Database Performance**: Test pattern storage and retrieval with large datasets

### Test Data Strategy

```rust
#[cfg(test)]
mod test_utils {
    pub fn create_test_codebase(file_count: usize, avg_file_size: usize) -> TempDir;
    pub fn create_pattern_heavy_files(pattern_density: f64) -> Vec<PathBuf>;
    pub fn measure_compression_performance(files: &[PathBuf]) -> PerformanceMetrics;
}
```

## Implementation Phases

### Phase 1: Database Foundation
- Implement `PatternDatabase` with SQLite backend
- Add checkpoint save/restore functionality
- Create database schema and migrations

### Phase 2: Progress Reporting
- Implement `ProgressReporter` with crossbeam channels
- Add CLI progress bar and status display
- Integrate timing and ETA calculations

### Phase 3: Parallel Pipeline
- Refactor existing analyzer to use `ParallelAnalyzer`
- Implement `ParallelFileProcessor` with rayon
- Add memory management and batching

### Phase 4: Integration and Optimization
- Integrate all components in `PipelineOrchestrator`
- Add resume functionality
- Performance tuning and optimization

This design maintains the existing codebase's strengths while addressing the core issues of observability, persistence, and scalability. The modular approach allows for incremental implementation and testing.