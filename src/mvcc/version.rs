//! Version management for MVCC
//!
//! Tracks multiple versions of data items

use crate::mvcc::{Timestamp, TransactionId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

/// A versioned data item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version<T> {
    /// The data
    pub data: T,
    /// Transaction that created this version
    pub xmin: TransactionId,
    /// Transaction that deleted/updated this version (None if current)
    pub xmax: Option<TransactionId>,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Deletion/update timestamp
    pub deleted_at: Option<Timestamp>,
}

impl<T> Version<T> {
    /// Create a new version
    pub fn new(data: T, txn_id: TransactionId, timestamp: Timestamp) -> Self {
        Self {
            data,
            xmin: txn_id,
            xmax: None,
            created_at: timestamp,
            deleted_at: None,
        }
    }
    
    /// Check if version is visible to a given snapshot timestamp
    pub fn is_visible(&self, snapshot_ts: Timestamp) -> bool {
        // Version must be created before snapshot
        if self.created_at > snapshot_ts {
            return false;
        }
        
        // Version must not be deleted before snapshot
        if let Some(deleted_at) = self.deleted_at {
            if deleted_at <= snapshot_ts {
                return false;
            }
        }
        
        true
    }
    
    /// Mark this version as deleted by a transaction
    pub fn mark_deleted(&mut self, txn_id: TransactionId, timestamp: Timestamp) {
        self.xmax = Some(txn_id);
        self.deleted_at = Some(timestamp);
    }
    
    /// Check if version is active (not deleted)
    pub fn is_active(&self) -> bool {
        self.xmax.is_none()
    }
}

/// Chain of versions for a data item
#[derive(Debug)]
pub struct VersionChain<T> {
    /// All versions, newest first
    versions: Arc<RwLock<Vec<Version<T>>>>,
}

impl<T: Clone> VersionChain<T> {
    /// Create a new version chain
    pub fn new() -> Self {
        Self {
            versions: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Add a new version
    pub fn add_version(&self, version: Version<T>) {
        let mut versions = self.versions.write();
        versions.insert(0, version); // Add at front (newest)
    }
    
    /// Get the version visible at a given snapshot timestamp
    pub fn get_visible_version(&self, snapshot_ts: Timestamp) -> Option<T> {
        let versions = self.versions.read();
        
        // Find first visible version
        for version in versions.iter() {
            if version.is_visible(snapshot_ts) {
                return Some(version.data.clone());
            }
        }
        
        None
    }
    
    /// Get the latest active version
    pub fn get_latest_active(&self) -> Option<T> {
        let versions = self.versions.read();
        
        for version in versions.iter() {
            if version.is_active() {
                return Some(version.data.clone());
            }
        }
        
        None
    }
    
    /// Mark latest version as deleted
    pub fn mark_latest_deleted(&self, txn_id: TransactionId, timestamp: Timestamp) {
        let mut versions = self.versions.write();
        
        if let Some(version) = versions.first_mut() {
            version.mark_deleted(txn_id, timestamp);
        }
    }
    
    /// Garbage collect old versions
    pub fn gc(&self, min_snapshot_ts: Timestamp) {
        let mut versions = self.versions.write();
        
        // Keep only versions that might be visible to active snapshots
        versions.retain(|v| {
            v.is_active() || v.deleted_at.map_or(true, |ts| ts >= min_snapshot_ts)
        });
    }
    
    /// Get version count
    pub fn version_count(&self) -> usize {
        self.versions.read().len()
    }
}

impl<T: Clone> Default for VersionChain<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_creation() {
        let version = Version::new("data".to_string(), TransactionId(1), 100);
        
        assert_eq!(version.data, "data");
        assert_eq!(version.xmin.0, 1);
        assert!(version.xmax.is_none());
        assert_eq!(version.created_at, 100);
    }

    #[test]
    fn test_version_visibility() {
        let version = Version::new("data".to_string(), TransactionId(1), 100);
        
        // Visible at timestamp 100 and later
        assert!(!version.is_visible(99));
        assert!(version.is_visible(100));
        assert!(version.is_visible(101));
    }

    #[test]
    fn test_version_deletion() {
        let mut version = Version::new("data".to_string(), TransactionId(1), 100);
        
        version.mark_deleted(TransactionId(2), 200);
        
        // Not visible after deletion
        assert!(version.is_visible(150));
        assert!(!version.is_visible(200));
        assert!(!version.is_visible(250));
    }

    #[test]
    fn test_version_chain() {
        let chain: VersionChain<String> = VersionChain::new();
        
        chain.add_version(Version::new("v1".to_string(), TransactionId(1), 100));
        chain.add_version(Version::new("v2".to_string(), TransactionId(2), 200));
        
        assert_eq!(chain.version_count(), 2);
        
        // At timestamp 150, should see v1
        assert_eq!(chain.get_visible_version(150), Some("v1".to_string()));
        
        // At timestamp 250, should see v2
        assert_eq!(chain.get_visible_version(250), Some("v2".to_string()));
    }

    #[test]
    fn test_version_chain_deletion() {
        let chain: VersionChain<String> = VersionChain::new();
        
        chain.add_version(Version::new("v1".to_string(), TransactionId(1), 100));
        chain.mark_latest_deleted(TransactionId(2), 200);
        
        // Visible before deletion
        assert_eq!(chain.get_visible_version(150), Some("v1".to_string()));
        
        // Not visible after deletion
        assert_eq!(chain.get_visible_version(200), None);
    }

    #[test]
    fn test_garbage_collection() {
        let chain: VersionChain<String> = VersionChain::new();
        
        let mut v1 = Version::new("v1".to_string(), TransactionId(1), 100);
        v1.mark_deleted(TransactionId(2), 150);
        chain.add_version(v1);
        
        chain.add_version(Version::new("v2".to_string(), TransactionId(2), 200));
        
        assert_eq!(chain.version_count(), 2);
        
        // GC with min snapshot at 140 should keep v1 (deleted at 150, so still visible)
        chain.gc(140);
        assert_eq!(chain.version_count(), 2);
        
        // GC with min snapshot at 180 should remove v1 (deleted before 180)
        chain.gc(180);
        assert_eq!(chain.version_count(), 1);
    }
}

