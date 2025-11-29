//! Write-Ahead Log implementation
//!
//! Logs all mutations before applying them to storage

use crate::error::{DeepGraphError, Result};
use crate::graph::{Edge, EdgeId, Node, NodeId};
use crate::wal::WALConfig;
use log::{debug, info, trace};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use parking_lot::RwLock;

/// Log Sequence Number - monotonically increasing
pub type LSN = u64;

/// Write-Ahead Log
pub struct WAL {
    /// Configuration
    config: WALConfig,
    /// Current segment file
    current_segment: Arc<RwLock<Option<BufWriter<File>>>>,
    /// Current LSN
    current_lsn: Arc<AtomicU64>,
    /// Segment number
    segment_number: Arc<AtomicU64>,
    /// Entries written in current segment
    entries_in_segment: Arc<AtomicU64>,
}

/// WAL entry representing a single operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WALEntry {
    /// Log sequence number
    pub lsn: LSN,
    /// Transaction ID
    pub txn_id: u64,
    /// The operation
    pub operation: WALOperation,
    /// Timestamp
    pub timestamp: u64,
}

/// Operations that can be logged
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WALOperation {
    /// Begin transaction
    BeginTxn,
    
    /// Commit transaction
    CommitTxn,
    
    /// Abort transaction
    AbortTxn,
    
    /// Add node
    InsertNode { node: Node },
    
    /// Update node
    UpdateNode { node: Node },
    
    /// Delete node
    DeleteNode { id: NodeId },
    
    /// Add edge
    InsertEdge { edge: Edge },
    
    /// Update edge
    UpdateEdge { edge: Edge },
    
    /// Delete edge
    DeleteEdge { id: EdgeId },
    
    /// Checkpoint marker
    Checkpoint,
}

impl WAL {
    /// Create a new WAL
    pub fn new(config: WALConfig) -> Result<Self> {
        info!("Initializing WAL at directory: {}", config.wal_dir);
        info!("WAL configuration: segment_size={}MB, sync_on_write={}, checkpoint_threshold={}", 
              config.segment_size / (1024 * 1024), 
              config.sync_on_write,
              config.checkpoint_threshold);
        
        // Create WAL directory
        std::fs::create_dir_all(&config.wal_dir)?;
        
        let wal = Self {
            config,
            current_segment: Arc::new(RwLock::new(None)),
            current_lsn: Arc::new(AtomicU64::new(0)),
            segment_number: Arc::new(AtomicU64::new(0)),
            entries_in_segment: Arc::new(AtomicU64::new(0)),
        };
        
        // Open first segment
        wal.rotate_segment()?;
        
        info!("WAL initialized successfully");
        Ok(wal)
    }
    
    /// Append an entry to the log
    pub fn append(&self, txn_id: u64, operation: WALOperation) -> Result<LSN> {
        // Get next LSN
        let lsn = self.current_lsn.fetch_add(1, Ordering::SeqCst);
        
        debug!("WAL append: LSN={}, txn_id={}, op={:?}", lsn, txn_id, operation);
        
        // Create entry
        let entry = WALEntry {
            lsn,
            txn_id,
            operation,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        // Serialize entry
        let serialized = bincode::serialize(&entry)
            .map_err(|e| DeepGraphError::StorageError(format!("WAL serialize error: {}", e)))?;
        
        trace!("WAL entry serialized: {} bytes", serialized.len());
        
        // Write length prefix + data
        let mut segment = self.current_segment.write();
        if let Some(ref mut writer) = *segment {
            let len = serialized.len() as u32;
            writer.write_all(&len.to_le_bytes())?;
            writer.write_all(&serialized)?;
            
            // Sync if configured
            if self.config.sync_on_write {
                writer.flush()?;
                trace!("WAL entry synced to disk at LSN {}", lsn);
            }
        }
        
        // Increment entries counter
        let entries = self.entries_in_segment.fetch_add(1, Ordering::SeqCst);
        
        // Check if we need to rotate segment
        if entries > 0 && entries % self.config.checkpoint_threshold as u64 == 0 {
            drop(segment); // Release lock before rotating
            self.rotate_segment()?;
        }
        
        Ok(lsn)
    }
    
    /// Rotate to a new segment file
    fn rotate_segment(&self) -> Result<()> {
        let segment_num = self.segment_number.fetch_add(1, Ordering::SeqCst);
        let segment_path = self.segment_path(segment_num);
        
        info!("Rotating WAL to new segment: {:?} (segment #{})", segment_path, segment_num);
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&segment_path)?;
        
        let writer = BufWriter::new(file);
        
        let mut current = self.current_segment.write();
        *current = Some(writer);
        
        self.entries_in_segment.store(0, Ordering::SeqCst);
        
        info!("WAL segment rotation complete");
        Ok(())
    }
    
    /// Get path for a segment
    fn segment_path(&self, segment: u64) -> PathBuf {
        Path::new(&self.config.wal_dir).join(format!("wal-{:08}.log", segment))
    }
    
    /// Force sync to disk
    pub fn flush(&self) -> Result<()> {
        debug!("Flushing WAL to disk");
        let mut segment = self.current_segment.write();
        if let Some(ref mut writer) = *segment {
            writer.flush()?;
        }
        trace!("WAL flushed successfully");
        Ok(())
    }
    
    /// Get current LSN
    pub fn current_lsn(&self) -> LSN {
        self.current_lsn.load(Ordering::SeqCst)
    }
    
    /// Write checkpoint marker
    pub fn checkpoint(&self) -> Result<LSN> {
        info!("Writing WAL checkpoint");
        let lsn = self.append(0, WALOperation::Checkpoint)?;
        info!("WAL checkpoint written at LSN {}", lsn);
        Ok(lsn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_wal_creation() {
        let dir = tempdir().unwrap();
        let config = WALConfig::new().with_dir(dir.path().to_string_lossy().to_string());
        
        let wal = WAL::new(config).unwrap();
        assert_eq!(wal.current_lsn(), 0);
    }

    #[test]
    fn test_append_entry() {
        let dir = tempdir().unwrap();
        let config = WALConfig::new()
            .with_dir(dir.path().to_string_lossy().to_string())
            .with_sync(false); // Faster for tests
        
        let wal = WAL::new(config).unwrap();
        
        let node = Node::new(vec!["Test".to_string()]);
        let lsn = wal.append(1, WALOperation::InsertNode { node }).unwrap();
        
        assert_eq!(lsn, 0);
        assert_eq!(wal.current_lsn(), 1);
    }

    #[test]
    fn test_transaction_flow() {
        let dir = tempdir().unwrap();
        let config = WALConfig::new()
            .with_dir(dir.path().to_string_lossy().to_string())
            .with_sync(false);
        
        let wal = WAL::new(config).unwrap();
        
        // Begin transaction
        let lsn1 = wal.append(1, WALOperation::BeginTxn).unwrap();
        
        // Insert node
        let node = Node::new(vec!["Test".to_string()]);
        let lsn2 = wal.append(1, WALOperation::InsertNode { node }).unwrap();
        
        // Commit
        let lsn3 = wal.append(1, WALOperation::CommitTxn).unwrap();
        
        assert_eq!(lsn1, 0);
        assert_eq!(lsn2, 1);
        assert_eq!(lsn3, 2);
    }

    #[test]
    fn test_checkpoint() {
        let dir = tempdir().unwrap();
        let config = WALConfig::new()
            .with_dir(dir.path().to_string_lossy().to_string())
            .with_sync(false);
        
        let wal = WAL::new(config).unwrap();
        
        let lsn = wal.checkpoint().unwrap();
        assert_eq!(lsn, 0);
    }

    #[test]
    fn test_flush() {
        let dir = tempdir().unwrap();
        let config = WALConfig::new()
            .with_dir(dir.path().to_string_lossy().to_string())
            .with_sync(false);
        
        let wal = WAL::new(config).unwrap();
        
        let node = Node::new(vec!["Test".to_string()]);
        wal.append(1, WALOperation::InsertNode { node }).unwrap();
        
        assert!(wal.flush().is_ok());
    }
}

