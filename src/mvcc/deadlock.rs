//! Deadlock detection for MVCC transactions
//!
//! Implements wait-for graph cycle detection

use crate::error::{DeepGraphError, Result};
use crate::mvcc::TransactionId;
use dashmap::DashMap;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

/// Resource ID (could be node, edge, or any lockable resource)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u64);

/// Wait-for graph for deadlock detection
pub struct DeadlockDetector {
    /// Maps txn -> set of txns it's waiting for
    wait_for: Arc<DashMap<TransactionId, HashSet<TransactionId>>>,
    /// Maps resource -> txn holding the lock
    lock_holders: Arc<DashMap<ResourceId, TransactionId>>,
}

impl DeadlockDetector {
    /// Create a new deadlock detector
    pub fn new() -> Self {
        Self {
            wait_for: Arc::new(DashMap::new()),
            lock_holders: Arc::new(DashMap::new()),
        }
    }
    
    /// Request a lock on a resource
    pub fn request_lock(
        &self,
        txn_id: TransactionId,
        resource_id: ResourceId,
    ) -> Result<()> {
        // Check if resource is already locked
        if let Some(holder) = self.lock_holders.get(&resource_id) {
            let holder_id = *holder;
            
            // If we're the holder, it's a re-entrant lock (OK)
            if holder_id == txn_id {
                return Ok(());
            }
            
            // Add wait-for edge
            self.wait_for
                .entry(txn_id)
                .or_insert_with(HashSet::new)
                .insert(holder_id);
            
            // Check for deadlock
            if self.has_cycle(txn_id)? {
                // Remove the wait-for edge we just added
                if let Some(mut entry) = self.wait_for.get_mut(&txn_id) {
                    entry.remove(&holder_id);
                }
                
                return Err(DeepGraphError::TransactionError(format!(
                    "Deadlock detected: transaction {:?} waiting for {:?}",
                    txn_id, holder_id
                )));
            }
            
            // Would need to wait (in real system, this would block)
            return Err(DeepGraphError::TransactionError(format!(
                "Resource locked by {:?}",
                holder_id
            )));
        }
        
        // Lock is available, grant it
        self.lock_holders.insert(resource_id, txn_id);
        Ok(())
    }
    
    /// Release a lock on a resource
    pub fn release_lock(&self, _txn_id: TransactionId, resource_id: ResourceId) {
        self.lock_holders.remove(&resource_id);
        
        // Remove wait-for edges for txns that were waiting on this resource
        // (In a full implementation, we'd wake up waiting transactions)
    }
    
    /// Release all locks held by a transaction
    pub fn release_all_locks(&self, txn_id: TransactionId) {
        // Remove all locks held by this transaction
        self.lock_holders.retain(|_, holder| *holder != txn_id);
        
        // Remove wait-for edges
        self.wait_for.remove(&txn_id);
    }
    
    /// Check if there's a cycle in the wait-for graph (deadlock)
    fn has_cycle(&self, start: TransactionId) -> Result<bool> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        self.dfs_cycle_check(start, &mut visited, &mut rec_stack)
    }
    
    /// DFS-based cycle detection
    fn dfs_cycle_check(
        &self,
        node: TransactionId,
        visited: &mut HashSet<TransactionId>,
        rec_stack: &mut HashSet<TransactionId>,
    ) -> Result<bool> {
        visited.insert(node);
        rec_stack.insert(node);
        
        // Get nodes this transaction is waiting for
        if let Some(wait_set) = self.wait_for.get(&node) {
            for &neighbor in wait_set.iter() {
                if !visited.contains(&neighbor) {
                    if self.dfs_cycle_check(neighbor, visited, rec_stack)? {
                        return Ok(true);
                    }
                } else if rec_stack.contains(&neighbor) {
                    // Found a back edge = cycle
                    return Ok(true);
                }
            }
        }
        
        rec_stack.remove(&node);
        Ok(false)
    }
    
    /// Get all transactions involved in a potential deadlock
    pub fn get_deadlocked_txns(&self, start: TransactionId) -> Vec<TransactionId> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        queue.push_back(start);
        visited.insert(start);
        
        while let Some(txn) = queue.pop_front() {
            result.push(txn);
            
            if let Some(wait_set) = self.wait_for.get(&txn) {
                for &neighbor in wait_set.iter() {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                    }
                }
            }
        }
        
        result
    }
    
    /// Get statistics about the wait-for graph
    pub fn stats(&self) -> DeadlockStats {
        DeadlockStats {
            waiting_transactions: self.wait_for.len(),
            locked_resources: self.lock_holders.len(),
        }
    }
}

impl Default for DeadlockDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about deadlock detection
#[derive(Debug, Clone)]
pub struct DeadlockStats {
    /// Number of transactions currently waiting
    pub waiting_transactions: usize,
    /// Number of resources currently locked
    pub locked_resources: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = DeadlockDetector::new();
        let stats = detector.stats();
        assert_eq!(stats.waiting_transactions, 0);
        assert_eq!(stats.locked_resources, 0);
    }

    #[test]
    fn test_grant_lock() {
        let detector = DeadlockDetector::new();
        let txn1 = TransactionId(1);
        let res1 = ResourceId(100);
        
        // First lock should succeed
        assert!(detector.request_lock(txn1, res1).is_ok());
        
        // Check stats
        let stats = detector.stats();
        assert_eq!(stats.locked_resources, 1);
    }

    #[test]
    fn test_lock_conflict() {
        let detector = DeadlockDetector::new();
        let txn1 = TransactionId(1);
        let txn2 = TransactionId(2);
        let res1 = ResourceId(100);
        
        // Txn1 gets lock
        detector.request_lock(txn1, res1).unwrap();
        
        // Txn2 tries to get same lock - should fail
        assert!(detector.request_lock(txn2, res1).is_err());
    }

    #[test]
    fn test_release_lock() {
        let detector = DeadlockDetector::new();
        let txn1 = TransactionId(1);
        let res1 = ResourceId(100);
        
        detector.request_lock(txn1, res1).unwrap();
        detector.release_lock(txn1, res1);
        
        // Now another txn should be able to get the lock
        let txn2 = TransactionId(2);
        assert!(detector.request_lock(txn2, res1).is_ok());
    }

    #[test]
    fn test_deadlock_detection() {
        let detector = DeadlockDetector::new();
        
        let txn1 = TransactionId(1);
        let txn2 = TransactionId(2);
        let res1 = ResourceId(100);
        let res2 = ResourceId(200);
        
        // Txn1 locks res1
        detector.request_lock(txn1, res1).unwrap();
        
        // Txn2 locks res2
        detector.request_lock(txn2, res2).unwrap();
        
        // Txn1 tries to lock res2 (held by txn2) - creates wait
        let result1 = detector.request_lock(txn1, res2);
        assert!(result1.is_err());
        
        // Txn2 tries to lock res1 (held by txn1) - should detect deadlock
        let result2 = detector.request_lock(txn2, res1);
        assert!(result2.is_err());
        
        // Check that it's specifically a deadlock
        if let Err(e) = result2 {
            assert!(e.to_string().contains("Deadlock detected"));
        }
    }

    #[test]
    fn test_release_all_locks() {
        let detector = DeadlockDetector::new();
        let txn1 = TransactionId(1);
        let res1 = ResourceId(100);
        let res2 = ResourceId(200);
        
        detector.request_lock(txn1, res1).unwrap();
        detector.request_lock(txn1, res2).unwrap();
        
        assert_eq!(detector.stats().locked_resources, 2);
        
        detector.release_all_locks(txn1);
        
        assert_eq!(detector.stats().locked_resources, 0);
    }

    #[test]
    fn test_reentrant_lock() {
        let detector = DeadlockDetector::new();
        let txn1 = TransactionId(1);
        let res1 = ResourceId(100);
        
        // First lock
        detector.request_lock(txn1, res1).unwrap();
        
        // Same txn requests same lock - should succeed (re-entrant)
        assert!(detector.request_lock(txn1, res1).is_ok());
    }
}

