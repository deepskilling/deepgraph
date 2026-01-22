//! Snapshot management for point-in-time backups
//!
//! Provides functionality to create, manage, and restore from snapshots.

use crate::error::{DeepGraphError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Metadata for a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Unique identifier for the snapshot
    pub id: String,
    /// Timestamp when snapshot was created
    pub timestamp: i64,
    /// Path to the snapshot directory
    pub path: PathBuf,
    /// Number of nodes in the snapshot
    pub node_count: usize,
    /// Number of edges in the snapshot
    pub edge_count: usize,
    /// Optional description
    pub description: Option<String>,
}

impl Snapshot {
    /// Create a new snapshot metadata
    pub fn new(
        id: String,
        path: PathBuf,
        node_count: usize,
        edge_count: usize,
    ) -> Self {
        Self {
            id,
            timestamp: chrono::Utc::now().timestamp(),
            path,
            node_count,
            edge_count,
            description: None,
        }
    }
    
    /// Create a snapshot with a description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    
    /// Get the path to the nodes file
    pub fn nodes_file(&self) -> PathBuf {
        self.path.join("nodes.parquet")
    }
    
    /// Get the path to the edges file
    pub fn edges_file(&self) -> PathBuf {
        self.path.join("edges.parquet")
    }
    
    /// Get the path to the metadata file
    pub fn metadata_file(&self) -> PathBuf {
        self.path.join("metadata.json")
    }
    
    /// Save snapshot metadata to disk
    pub fn save_metadata(&self) -> Result<()> {
        let metadata_path = self.metadata_file();
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| DeepGraphError::SerializationError(e.to_string()))?;
        
        fs::write(&metadata_path, json)
            .map_err(|e| DeepGraphError::IoError(e))?;
        
        Ok(())
    }
    
    /// Load snapshot metadata from disk
    pub fn load_metadata(path: &Path) -> Result<Self> {
        let metadata_path = path.join("metadata.json");
        let json = fs::read_to_string(&metadata_path)
            .map_err(|e| DeepGraphError::IoError(e))?;
        
        let snapshot: Self = serde_json::from_str(&json)
            .map_err(|e| DeepGraphError::SerializationError(e.to_string()))?;
        
        Ok(snapshot)
    }
}

/// Snapshot manager for coordinating snapshots
pub struct SnapshotManager {
    /// Base directory for snapshots
    base_dir: PathBuf,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub fn new(base_dir: PathBuf) -> Result<Self> {
        // Create the base directory if it doesn't exist
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir)
                .map_err(|e| DeepGraphError::IoError(e))?;
        }
        
        Ok(Self { base_dir })
    }
    
    /// Create a new snapshot directory
    pub fn create_snapshot_dir(&self, snapshot_id: &str) -> Result<PathBuf> {
        let snapshot_dir = self.base_dir.join(snapshot_id);
        
        fs::create_dir_all(&snapshot_dir)
            .map_err(|e| DeepGraphError::IoError(e))?;
        
        Ok(snapshot_dir)
    }
    
    /// List all snapshots
    pub fn list_snapshots(&self) -> Result<Vec<Snapshot>> {
        let mut snapshots = Vec::new();
        
        if !self.base_dir.exists() {
            return Ok(snapshots);
        }
        
        for entry in fs::read_dir(&self.base_dir)
            .map_err(|e| DeepGraphError::IoError(e))? 
        {
            let entry = entry.map_err(|e| DeepGraphError::IoError(e))?;
            let path = entry.path();
            
            if path.is_dir() {
                if let Ok(snapshot) = Snapshot::load_metadata(&path) {
                    snapshots.push(snapshot);
                }
            }
        }
        
        // Sort by timestamp (newest first)
        snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(snapshots)
    }
    
    /// Get a specific snapshot by ID
    pub fn get_snapshot(&self, snapshot_id: &str) -> Result<Snapshot> {
        let snapshot_dir = self.base_dir.join(snapshot_id);
        
        if !snapshot_dir.exists() {
            return Err(DeepGraphError::StorageError(
                format!("Snapshot {} not found", snapshot_id)
            ));
        }
        
        Snapshot::load_metadata(&snapshot_dir)
    }
    
    /// Delete a snapshot
    pub fn delete_snapshot(&self, snapshot_id: &str) -> Result<()> {
        let snapshot_dir = self.base_dir.join(snapshot_id);
        
        if snapshot_dir.exists() {
            fs::remove_dir_all(&snapshot_dir)
                .map_err(|e| DeepGraphError::IoError(e))?;
        }
        
        Ok(())
    }
    
    /// Clean up old snapshots, keeping only the most recent N
    pub fn cleanup_old_snapshots(&self, keep_count: usize) -> Result<usize> {
        let mut snapshots = self.list_snapshots()?;
        
        if snapshots.len() <= keep_count {
            return Ok(0);
        }
        
        let to_delete = snapshots.len() - keep_count;
        let old_snapshots: Vec<_> = snapshots.drain(keep_count..).collect();
        
        for snapshot in &old_snapshots {
            self.delete_snapshot(&snapshot.id)?;
        }
        
        Ok(to_delete)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_snapshot_creation() {
        let temp_dir = TempDir::new().unwrap();
        let snapshot = Snapshot::new(
            "test-snapshot".to_string(),
            temp_dir.path().to_path_buf(),
            100,
            200,
        );
        
        assert_eq!(snapshot.id, "test-snapshot");
        assert_eq!(snapshot.node_count, 100);
        assert_eq!(snapshot.edge_count, 200);
    }
    
    #[test]
    fn test_snapshot_metadata_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let snapshot = Snapshot::new(
            "test-snapshot".to_string(),
            temp_dir.path().to_path_buf(),
            100,
            200,
        ).with_description("Test snapshot".to_string());
        
        snapshot.save_metadata().unwrap();
        
        let loaded = Snapshot::load_metadata(temp_dir.path()).unwrap();
        assert_eq!(loaded.id, snapshot.id);
        assert_eq!(loaded.node_count, snapshot.node_count);
        assert_eq!(loaded.description, snapshot.description);
    }
    
    #[test]
    fn test_snapshot_manager() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SnapshotManager::new(temp_dir.path().to_path_buf()).unwrap();
        
        // Create a snapshot
        let snapshot_dir = manager.create_snapshot_dir("snap-1").unwrap();
        let snapshot = Snapshot::new(
            "snap-1".to_string(),
            snapshot_dir,
            50,
            75,
        );
        snapshot.save_metadata().unwrap();
        
        // List snapshots
        let snapshots = manager.list_snapshots().unwrap();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].id, "snap-1");
        
        // Get specific snapshot
        let retrieved = manager.get_snapshot("snap-1").unwrap();
        assert_eq!(retrieved.node_count, 50);
        
        // Delete snapshot
        manager.delete_snapshot("snap-1").unwrap();
        let snapshots = manager.list_snapshots().unwrap();
        assert_eq!(snapshots.len(), 0);
    }
    
    #[test]
    fn test_snapshot_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SnapshotManager::new(temp_dir.path().to_path_buf()).unwrap();
        
        // Create multiple snapshots
        for i in 1..=5 {
            let id = format!("snap-{}", i);
            let snapshot_dir = manager.create_snapshot_dir(&id).unwrap();
            let snapshot = Snapshot::new(id, snapshot_dir, i * 10, i * 20);
            snapshot.save_metadata().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure different timestamps
        }
        
        let snapshots = manager.list_snapshots().unwrap();
        assert_eq!(snapshots.len(), 5);
        
        // Keep only 2 most recent
        let deleted = manager.cleanup_old_snapshots(2).unwrap();
        assert_eq!(deleted, 3);
        
        let snapshots = manager.list_snapshots().unwrap();
        assert_eq!(snapshots.len(), 2);
    }
}

