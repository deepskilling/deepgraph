//! Hash index for O(1) equality lookups
//!
//! Efficient for queries like:
//! - MATCH (n:Person {name: "Alice"})
//! - WHERE n.age = 30

use crate::error::Result;
use crate::graph::NodeId;
use crate::index::Index;
use dashmap::DashMap;
use std::sync::Arc;

/// In-memory hash index using DashMap
///
/// Provides O(1) lookup time for exact matches.
/// Thread-safe and lock-free for reads.
pub struct HashIndex {
    /// Map from key to list of node IDs
    data: Arc<DashMap<Vec<u8>, Vec<NodeId>>>,
}

impl HashIndex {
    /// Create a new hash index
    pub fn new() -> Self {
        Self {
            data: Arc::new(DashMap::new()),
        }
    }
    
    /// Get statistics about the index
    pub fn stats(&self) -> HashIndexStats {
        let total_keys = self.data.len();
        let mut total_values = 0;
        let mut max_values = 0;
        
        for entry in self.data.iter() {
            let count = entry.value().len();
            total_values += count;
            max_values = max_values.max(count);
        }
        
        HashIndexStats {
            total_keys,
            total_values,
            max_values_per_key: max_values,
            avg_values_per_key: if total_keys > 0 {
                total_values as f64 / total_keys as f64
            } else {
                0.0
            },
        }
    }
}

impl Default for HashIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl Index for HashIndex {
    fn insert(&mut self, key: Vec<u8>, value: NodeId) -> Result<()> {
        self.data
            .entry(key)
            .or_insert_with(Vec::new)
            .push(value);
        Ok(())
    }
    
    fn remove(&mut self, key: &[u8], value: NodeId) -> Result<()> {
        if let Some(mut entry) = self.data.get_mut(key) {
            entry.retain(|&id| id != value);
            if entry.is_empty() {
                drop(entry);
                self.data.remove(key);
            }
        }
        Ok(())
    }
    
    fn lookup(&self, key: &[u8]) -> Result<Vec<NodeId>> {
        Ok(self.data
            .get(key)
            .map(|entry| entry.value().clone())
            .unwrap_or_default())
    }
    
    fn range(&self, _start: &[u8], _end: &[u8]) -> Result<Vec<NodeId>> {
        // Hash indices don't support range queries efficiently
        Ok(Vec::new())
    }
    
    fn keys(&self) -> Result<Vec<Vec<u8>>> {
        Ok(self.data
            .iter()
            .map(|entry| entry.key().clone())
            .collect())
    }
    
    fn clear(&mut self) -> Result<()> {
        self.data.clear();
        Ok(())
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }
}

/// Statistics about a hash index
#[derive(Debug, Clone)]
pub struct HashIndexStats {
    /// Number of unique keys
    pub total_keys: usize,
    /// Total number of values across all keys
    pub total_values: usize,
    /// Maximum values for any single key
    pub max_values_per_key: usize,
    /// Average values per key
    pub avg_values_per_key: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_index_insert_lookup() {
        let mut index = HashIndex::new();
        let node_id = NodeId::new();
        
        index.insert(b"key1".to_vec(), node_id).unwrap();
        
        let results = index.lookup(b"key1").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], node_id);
    }

    #[test]
    fn test_hash_index_multiple_values() {
        let mut index = HashIndex::new();
        let node1 = NodeId::new();
        let node2 = NodeId::new();
        
        index.insert(b"key1".to_vec(), node1).unwrap();
        index.insert(b"key1".to_vec(), node2).unwrap();
        
        let results = index.lookup(b"key1").unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.contains(&node1));
        assert!(results.contains(&node2));
    }

    #[test]
    fn test_hash_index_remove() {
        let mut index = HashIndex::new();
        let node_id = NodeId::new();
        
        index.insert(b"key1".to_vec(), node_id).unwrap();
        index.remove(b"key1", node_id).unwrap();
        
        let results = index.lookup(b"key1").unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_hash_index_stats() {
        let mut index = HashIndex::new();
        let node1 = NodeId::new();
        let node2 = NodeId::new();
        
        index.insert(b"key1".to_vec(), node1).unwrap();
        index.insert(b"key1".to_vec(), node2).unwrap();
        index.insert(b"key2".to_vec(), node1).unwrap();
        
        let stats = index.stats();
        assert_eq!(stats.total_keys, 2);
        assert_eq!(stats.total_values, 3);
        assert_eq!(stats.max_values_per_key, 2);
    }

    #[test]
    fn test_hash_index_clear() {
        let mut index = HashIndex::new();
        index.insert(b"key1".to_vec(), NodeId::new()).unwrap();
        
        assert_eq!(index.len(), 1);
        index.clear().unwrap();
        assert_eq!(index.len(), 0);
    }
}

