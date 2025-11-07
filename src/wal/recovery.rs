//! WAL recovery for crash recovery
//!
//! Replays log entries to restore database state

use crate::error::{DeepGraphError, Result};
use crate::storage::StorageBackend;
use crate::wal::{WALConfig, WALEntry, WALOperation};
use std::collections::HashSet;
use std::fs::{File, read_dir};
use std::io::{BufReader, Read};
use std::path::Path;

/// WAL recovery manager
pub struct WALRecovery {
    config: WALConfig,
}

impl WALRecovery {
    /// Create a new recovery manager
    pub fn new(config: WALConfig) -> Self {
        Self { config }
    }
    
    /// Recover database from WAL
    pub fn recover<S: StorageBackend>(&self, storage: &S) -> Result<u64> {
        // Find all WAL segments
        let segments = self.find_segments()?;
        
        if segments.is_empty() {
            return Ok(0);
        }
        
        // Track committed transactions
        let mut committed_txns = HashSet::new();
        
        // First pass: identify committed transactions
        for segment_path in &segments {
            let entries = self.read_segment(segment_path)?;
            for entry in entries {
                if matches!(entry.operation, WALOperation::CommitTxn) {
                    committed_txns.insert(entry.txn_id);
                }
            }
        }
        
        // Second pass: replay committed transactions
        let mut recovered = 0;
        for segment_path in &segments {
            let entries = self.read_segment(segment_path)?;
            for entry in entries {
                // Only replay operations from committed transactions
                if committed_txns.contains(&entry.txn_id) {
                    self.replay_entry(storage, &entry)?;
                    recovered += 1;
                }
            }
        }
        
        Ok(recovered)
    }
    
    /// Find all WAL segment files
    fn find_segments(&self) -> Result<Vec<String>> {
        let wal_path = Path::new(&self.config.wal_dir);
        
        if !wal_path.exists() {
            return Ok(vec![]);
        }
        
        let mut segments = Vec::new();
        for entry in read_dir(wal_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("log") {
                if let Some(path_str) = path.to_str() {
                    segments.push(path_str.to_string());
                }
            }
        }
        
        // Sort by segment number
        segments.sort();
        
        Ok(segments)
    }
    
    /// Read entries from a segment file
    fn read_segment(&self, path: &str) -> Result<Vec<WALEntry>> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut entries = Vec::new();
        
        loop {
            // Read length prefix
            let mut len_bytes = [0u8; 4];
            match reader.read_exact(&mut len_bytes) {
                Ok(_) => {
                    let len = u32::from_le_bytes(len_bytes) as usize;
                    
                    // Read entry data
                    let mut entry_bytes = vec![0u8; len];
                    reader.read_exact(&mut entry_bytes)?;
                    
                    // Deserialize
                    let entry: WALEntry = bincode::deserialize(&entry_bytes)
                        .map_err(|e| DeepGraphError::StorageError(format!("Deserialize error: {}", e)))?;
                    
                    entries.push(entry);
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }
        }
        
        Ok(entries)
    }
    
    /// Replay a single entry
    fn replay_entry<S: StorageBackend>(&self, storage: &S, entry: &WALEntry) -> Result<()> {
        match &entry.operation {
            WALOperation::InsertNode { node } => {
                storage.add_node(node.clone())?;
            }
            WALOperation::UpdateNode { node } => {
                storage.update_node(node.clone())?;
            }
            WALOperation::DeleteNode { id } => {
                storage.delete_node(*id)?;
            }
            WALOperation::InsertEdge { edge } => {
                storage.add_edge(edge.clone())?;
            }
            WALOperation::UpdateEdge { edge } => {
                storage.update_edge(edge.clone())?;
            }
            WALOperation::DeleteEdge { id } => {
                storage.delete_edge(*id)?;
            }
            _ => {
                // Skip control operations (BeginTxn, CommitTxn, etc.)
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Node;
    use crate::storage::MemoryStorage;
    use crate::wal::WAL;
    use tempfile::tempdir;

    #[test]
    fn test_recovery_creation() {
        let config = WALConfig::new();
        let _recovery = WALRecovery::new(config);
        // Recovery created successfully
    }

    #[test]
    fn test_empty_recovery() {
        let dir = tempdir().unwrap();
        let config = WALConfig::new().with_dir(dir.path().to_string_lossy().to_string());
        
        let recovery = WALRecovery::new(config);
        let storage = MemoryStorage::new();
        
        let recovered = recovery.recover(&storage).unwrap();
        assert_eq!(recovered, 0);
    }

    #[test]
    fn test_recovery_with_commit() {
        let dir = tempdir().unwrap();
        let config = WALConfig::new()
            .with_dir(dir.path().to_string_lossy().to_string())
            .with_sync(false);
        
        // Write some WAL entries
        let wal = WAL::new(config.clone()).unwrap();
        
        wal.append(1, WALOperation::BeginTxn).unwrap();
        let node = Node::new(vec!["Person".to_string()]);
        wal.append(1, WALOperation::InsertNode { node }).unwrap();
        wal.append(1, WALOperation::CommitTxn).unwrap();
        wal.flush().unwrap();
        
        // Drop WAL to close files
        drop(wal);
        
        // Recover to new storage
        let recovery = WALRecovery::new(config);
        let storage = MemoryStorage::new();
        
        let recovered = recovery.recover(&storage).unwrap();
        assert!(recovered > 0);
        assert_eq!(storage.node_count(), 1);
    }

    #[test]
    fn test_recovery_without_commit() {
        let dir = tempdir().unwrap();
        let config = WALConfig::new()
            .with_dir(dir.path().to_string_lossy().to_string())
            .with_sync(false);
        
        // Write uncommitted transaction
        let wal = WAL::new(config.clone()).unwrap();
        
        wal.append(1, WALOperation::BeginTxn).unwrap();
        let node = Node::new(vec!["Person".to_string()]);
        wal.append(1, WALOperation::InsertNode { node }).unwrap();
        // No commit!
        wal.flush().unwrap();
        
        drop(wal);
        
        // Recover - should not replay uncommitted transaction
        let recovery = WALRecovery::new(config);
        let storage = MemoryStorage::new();
        
        recovery.recover(&storage).unwrap();
        assert_eq!(storage.node_count(), 0); // Node not recovered
    }
}

