//! Snapshot isolation implementation
//!
//! Provides consistent read views for transactions

use crate::mvcc::{Timestamp, TransactionId};
use std::collections::HashSet;

/// A snapshot representing a consistent view of the database
#[derive(Debug, Clone)]
pub struct Snapshot {
    /// Snapshot timestamp
    pub timestamp: Timestamp,
    /// Active transactions at snapshot time
    pub active_txns: HashSet<TransactionId>,
}

impl Snapshot {
    /// Create a new snapshot
    pub fn new(timestamp: Timestamp, active_txns: HashSet<TransactionId>) -> Self {
        Self {
            timestamp,
            active_txns,
        }
    }
    
    /// Check if a transaction is visible in this snapshot
    pub fn is_txn_visible(&self, txn_id: TransactionId) -> bool {
        // Transaction must have started before snapshot
        if txn_id.0 >= self.timestamp {
            return false;
        }
        
        // Transaction must not be in active set (i.e., must be committed)
        !self.active_txns.contains(&txn_id)
    }
    
    /// Check if a version with given xmin/xmax is visible
    pub fn is_version_visible(&self, xmin: TransactionId, xmax: Option<TransactionId>) -> bool {
        // Creator transaction must be visible
        if !self.is_txn_visible(xmin) {
            return false;
        }
        
        // If deleted, deleter must not be visible
        if let Some(xmax) = xmax {
            if self.is_txn_visible(xmax) {
                return false;
            }
        }
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let mut active = HashSet::new();
        active.insert(TransactionId(5));
        active.insert(TransactionId(7));
        
        let snapshot = Snapshot::new(10, active);
        
        assert_eq!(snapshot.timestamp, 10);
        assert_eq!(snapshot.active_txns.len(), 2);
    }

    #[test]
    fn test_txn_visibility() {
        let mut active = HashSet::new();
        active.insert(TransactionId(5));
        
        let snapshot = Snapshot::new(10, active);
        
        // Committed before snapshot
        assert!(snapshot.is_txn_visible(TransactionId(3)));
        
        // Active at snapshot time
        assert!(!snapshot.is_txn_visible(TransactionId(5)));
        
        // Started after snapshot
        assert!(!snapshot.is_txn_visible(TransactionId(15)));
    }

    #[test]
    fn test_version_visibility() {
        let snapshot = Snapshot::new(10, HashSet::new());
        
        // Version created before snapshot, not deleted
        assert!(snapshot.is_version_visible(TransactionId(5), None));
        
        // Version created before snapshot, deleted after
        assert!(snapshot.is_version_visible(TransactionId(5), Some(TransactionId(15))));
        
        // Version created after snapshot
        assert!(!snapshot.is_version_visible(TransactionId(15), None));
        
        // Version created and deleted before snapshot
        assert!(!snapshot.is_version_visible(TransactionId(3), Some(TransactionId(7))));
    }
}

