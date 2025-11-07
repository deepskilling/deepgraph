//! Write-Ahead Logging (WAL) for durability
//!
//! Implements ACID guarantees through write-ahead logging

pub mod log;
pub mod recovery;

pub use log::{WAL, WALEntry, WALOperation};
pub use recovery::WALRecovery;

/// WAL configuration
#[derive(Debug, Clone)]
pub struct WALConfig {
    /// Path to WAL directory
    pub wal_dir: String,
    /// Segment size in bytes (default: 64MB)
    pub segment_size: usize,
    /// Sync mode (default: true for durability)
    pub sync_on_write: bool,
    /// Auto-checkpoint after N entries (default: 1000)
    pub checkpoint_threshold: usize,
}

impl Default for WALConfig {
    fn default() -> Self {
        Self {
            wal_dir: "./data/wal".to_string(),
            segment_size: 64 * 1024 * 1024, // 64MB
            sync_on_write: true,
            checkpoint_threshold: 1000,
        }
    }
}

impl WALConfig {
    /// Create a new WAL config with defaults
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the WAL directory
    pub fn with_dir(mut self, dir: impl Into<String>) -> Self {
        self.wal_dir = dir.into();
        self
    }
    
    /// Set segment size
    pub fn with_segment_size(mut self, size: usize) -> Self {
        self.segment_size = size;
        self
    }
    
    /// Set sync mode
    pub fn with_sync(mut self, sync: bool) -> Self {
        self.sync_on_write = sync;
        self
    }
}

