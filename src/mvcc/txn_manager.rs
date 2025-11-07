//! Transaction manager for MVCC
//!
//! Manages transaction lifecycle and isolation

use crate::error::{DeepGraphError, Result};
use crate::mvcc::{current_timestamp, next_txn_id, Snapshot, Timestamp};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;

/// Transaction ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TransactionId(pub u64);

/// Transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    /// Transaction is active
    Active,
    /// Transaction committed
    Committed,
    /// Transaction aborted
    Aborted,
}

/// Transaction metadata
#[derive(Debug, Clone)]
struct TransactionInfo {
    /// Start timestamp
    start_ts: Timestamp,
    /// Commit timestamp (if committed)
    commit_ts: Option<Timestamp>,
    /// Status
    status: TransactionStatus,
}

/// Transaction manager
pub struct TransactionManager {
    /// Active transactions
    active_txns: Arc<DashMap<TransactionId, TransactionInfo>>,
}

impl TransactionManager {
    /// Create a new transaction manager
    pub fn new() -> Self {
        Self {
            active_txns: Arc::new(DashMap::new()),
        }
    }
    
    /// Begin a new transaction
    pub fn begin_transaction(&self) -> Result<(TransactionId, Snapshot)> {
        let txn_id = next_txn_id();
        let timestamp = current_timestamp();
        
        // Get currently active transaction IDs
        let active_txn_ids: HashSet<TransactionId> = self
            .active_txns
            .iter()
            .map(|entry| *entry.key())
            .collect();
        
        // Create snapshot
        let snapshot = Snapshot::new(timestamp, active_txn_ids.clone());
        
        // Register transaction
        let info = TransactionInfo {
            start_ts: timestamp,
            commit_ts: None,
            status: TransactionStatus::Active,
        };
        
        self.active_txns.insert(txn_id, info);
        
        Ok((txn_id, snapshot))
    }
    
    /// Commit a transaction
    pub fn commit_transaction(&self, txn_id: TransactionId) -> Result<Timestamp> {
        let commit_ts = current_timestamp();
        
        // Update transaction status
        if let Some(mut info) = self.active_txns.get_mut(&txn_id) {
            info.status = TransactionStatus::Committed;
            info.commit_ts = Some(commit_ts);
        } else {
            return Err(DeepGraphError::TransactionError(
                "Transaction not found".to_string(),
            ));
        }
        
        // Remove from active set
        self.active_txns.remove(&txn_id);
        
        Ok(commit_ts)
    }
    
    /// Abort a transaction
    pub fn abort_transaction(&self, txn_id: TransactionId) -> Result<()> {
        // Update status
        if let Some(mut info) = self.active_txns.get_mut(&txn_id) {
            info.status = TransactionStatus::Aborted;
        } else {
            return Err(DeepGraphError::TransactionError(
                "Transaction not found".to_string(),
            ));
        }
        
        // Remove from active set
        self.active_txns.remove(&txn_id);
        
        Ok(())
    }
    
    /// Check if transaction is active
    pub fn is_active(&self, txn_id: TransactionId) -> bool {
        self.active_txns.contains_key(&txn_id)
    }
    
    /// Get number of active transactions
    pub fn active_count(&self) -> usize {
        self.active_txns.len()
    }
    
    /// Get oldest active timestamp (for garbage collection)
    pub fn oldest_active_timestamp(&self) -> Option<Timestamp> {
        self.active_txns
            .iter()
            .map(|entry| entry.value().start_ts)
            .min()
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_txn_manager_creation() {
        let manager = TransactionManager::new();
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_begin_transaction() {
        let manager = TransactionManager::new();
        
        let (txn_id, _snapshot) = manager.begin_transaction().unwrap();
        
        assert!(manager.is_active(txn_id));
        assert_eq!(manager.active_count(), 1);
    }

    #[test]
    fn test_commit_transaction() {
        let manager = TransactionManager::new();
        
        let (txn_id, _snapshot) = manager.begin_transaction().unwrap();
        assert!(manager.is_active(txn_id));
        
        let commit_ts = manager.commit_transaction(txn_id).unwrap();
        assert!(commit_ts > 0);
        assert!(!manager.is_active(txn_id));
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_abort_transaction() {
        let manager = TransactionManager::new();
        
        let (txn_id, _snapshot) = manager.begin_transaction().unwrap();
        assert!(manager.is_active(txn_id));
        
        manager.abort_transaction(txn_id).unwrap();
        assert!(!manager.is_active(txn_id));
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_multiple_transactions() {
        let manager = TransactionManager::new();
        
        let (txn1, _) = manager.begin_transaction().unwrap();
        let (txn2, _) = manager.begin_transaction().unwrap();
        let (txn3, _) = manager.begin_transaction().unwrap();
        
        assert_eq!(manager.active_count(), 3);
        
        manager.commit_transaction(txn1).unwrap();
        assert_eq!(manager.active_count(), 2);
        
        manager.abort_transaction(txn2).unwrap();
        assert_eq!(manager.active_count(), 1);
        
        manager.commit_transaction(txn3).unwrap();
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_oldest_active_timestamp() {
        let manager = TransactionManager::new();
        
        // No active transactions
        assert!(manager.oldest_active_timestamp().is_none());
        
        // Start some transactions
        let (txn1, snapshot1) = manager.begin_transaction().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let (_txn2, _snapshot2) = manager.begin_transaction().unwrap();
        
        // Oldest should be first transaction
        let oldest = manager.oldest_active_timestamp().unwrap();
        assert_eq!(oldest, snapshot1.timestamp);
        
        // Commit first, oldest should update
        manager.commit_transaction(txn1).unwrap();
        let new_oldest = manager.oldest_active_timestamp().unwrap();
        assert!(new_oldest > oldest);
    }
}

