//! B-tree index for range queries
//!
//! Efficient for queries like:
//! - WHERE n.age > 25 AND n.age < 50
//! - WHERE n.created_at >= '2024-01-01'

use crate::error::{DeepGraphError, Result};
use crate::graph::NodeId;
use crate::index::Index;
use sled::Db;
use std::path::Path;

/// Persistent B-tree index using Sled
///
/// Provides O(log n) operations with disk persistence.
/// Supports efficient range queries.
pub struct BTreeIndex {
    /// Sled database instance
    db: Db,
    /// Tree name for this index
    tree_name: String,
}

impl BTreeIndex {
    /// Create a new B-tree index with persistence
    pub fn new(path: &Path, tree_name: &str) -> Result<Self> {
        let db = sled::open(path)
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to open sled: {}", e)))?;
        
        Ok(Self {
            db,
            tree_name: tree_name.to_string(),
        })
    }
    
    /// Create a new in-memory B-tree index (for testing)
    pub fn new_temp() -> Result<Self> {
        let db = sled::Config::new()
            .temporary(true)
            .open()
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to open temp sled: {}", e)))?;
        
        Ok(Self {
            db,
            tree_name: "temp".to_string(),
        })
    }
    
    /// Get the tree for this index
    fn tree(&self) -> Result<sled::Tree> {
        self.db
            .open_tree(&self.tree_name)
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to open tree: {}", e)))
    }
    
    /// Encode a NodeId as bytes
    fn encode_node_id(node_id: &NodeId) -> Vec<u8> {
        node_id.as_uuid().as_bytes().to_vec()
    }
    
    /// Decode bytes to NodeId
    fn decode_node_id(bytes: &[u8]) -> Result<NodeId> {
        let uuid = uuid::Uuid::from_slice(bytes)
            .map_err(|e| DeepGraphError::InvalidNodeId(e.to_string()))?;
        Ok(NodeId::from_uuid(uuid))
    }
    
    /// Create a composite key (index_key + node_id for uniqueness)
    fn make_key(key: &[u8], node_id: &NodeId) -> Vec<u8> {
        let mut composite = key.to_vec();
        composite.extend_from_slice(&Self::encode_node_id(node_id));
        composite
    }
    
    /// Get statistics about the index
    pub fn stats(&self) -> Result<BTreeIndexStats> {
        let tree = self.tree()?;
        
        // Get approximate size from database
        let size_on_disk = self.db.size_on_disk()
            .map_err(|e| DeepGraphError::StorageError(e.to_string()))?;
        
        Ok(BTreeIndexStats {
            total_entries: tree.len(),
            size_on_disk,
        })
    }
    
    /// Flush data to disk
    pub fn flush(&self) -> Result<()> {
        self.db
            .flush()
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to flush: {}", e)))?;
        Ok(())
    }
}

impl Index for BTreeIndex {
    fn insert(&mut self, key: Vec<u8>, value: NodeId) -> Result<()> {
        let tree = self.tree()?;
        let composite_key = Self::make_key(&key, &value);
        
        tree.insert(composite_key, &[])
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to insert: {}", e)))?;
        
        Ok(())
    }
    
    fn remove(&mut self, key: &[u8], value: NodeId) -> Result<()> {
        let tree = self.tree()?;
        let composite_key = Self::make_key(key, &value);
        
        tree.remove(composite_key)
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to remove: {}", e)))?;
        
        Ok(())
    }
    
    fn lookup(&self, key: &[u8]) -> Result<Vec<NodeId>> {
        let tree = self.tree()?;
        let mut results = Vec::new();
        
        // Scan all keys with this prefix
        for item in tree.scan_prefix(key) {
            let (composite_key, _) = item
                .map_err(|e| DeepGraphError::StorageError(e.to_string()))?;
            
            // Extract the node ID from the composite key
            if composite_key.len() > key.len() {
                let node_id_bytes = &composite_key[key.len()..];
                if let Ok(node_id) = Self::decode_node_id(node_id_bytes) {
                    results.push(node_id);
                }
            }
        }
        
        Ok(results)
    }
    
    fn range(&self, start: &[u8], end: &[u8]) -> Result<Vec<NodeId>> {
        let tree = self.tree()?;
        let mut results = Vec::new();
        
        // Range scan from start to end
        for item in tree.range(start..end) {
            let (composite_key, _) = item
                .map_err(|e| DeepGraphError::StorageError(e.to_string()))?;
            
            // Find where the node ID starts in the composite key
            // We need to extract just the node ID portion
            if composite_key.len() >= 16 {
                let node_id_start = composite_key.len() - 16;
                let node_id_bytes = &composite_key[node_id_start..];
                if let Ok(node_id) = Self::decode_node_id(node_id_bytes) {
                    results.push(node_id);
                }
            }
        }
        
        Ok(results)
    }
    
    fn keys(&self) -> Result<Vec<Vec<u8>>> {
        let tree = self.tree()?;
        let mut keys = Vec::new();
        
        for item in tree.iter() {
            let (key, _) = item
                .map_err(|e| DeepGraphError::StorageError(e.to_string()))?;
            keys.push(key.to_vec());
        }
        
        Ok(keys)
    }
    
    fn clear(&mut self) -> Result<()> {
        let tree = self.tree()?;
        tree.clear()
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to clear: {}", e)))?;
        Ok(())
    }
    
    fn len(&self) -> usize {
        self.tree()
            .map(|t| t.len())
            .unwrap_or(0)
    }
}

/// Statistics about a B-tree index
#[derive(Debug, Clone)]
pub struct BTreeIndexStats {
    /// Number of entries in the index
    pub total_entries: usize,
    /// Size on disk in bytes
    pub size_on_disk: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_btree_index_insert_lookup() {
        let mut index = BTreeIndex::new_temp().unwrap();
        let node_id = NodeId::new();
        
        index.insert(b"key1".to_vec(), node_id).unwrap();
        
        let results = index.lookup(b"key1").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], node_id);
    }

    #[test]
    fn test_btree_index_range_query() {
        let mut index = BTreeIndex::new_temp().unwrap();
        
        // Insert values with integer keys
        let node1 = NodeId::new();
        let node2 = NodeId::new();
        let node3 = NodeId::new();
        
        index.insert(10i64.to_le_bytes().to_vec(), node1).unwrap();
        index.insert(20i64.to_le_bytes().to_vec(), node2).unwrap();
        index.insert(30i64.to_le_bytes().to_vec(), node3).unwrap();
        
        // Range query: 15 to 25
        let start = 15i64.to_le_bytes();
        let end = 25i64.to_le_bytes();
        let results = index.range(&start, &end).unwrap();
        
        // Should only return node2 (value 20)
        assert_eq!(results.len(), 1);
        assert!(results.contains(&node2));
    }

    #[test]
    fn test_btree_index_remove() {
        let mut index = BTreeIndex::new_temp().unwrap();
        let node_id = NodeId::new();
        
        index.insert(b"key1".to_vec(), node_id).unwrap();
        assert_eq!(index.len(), 1);
        
        index.remove(b"key1", node_id).unwrap();
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_btree_index_stats() {
        let mut index = BTreeIndex::new_temp().unwrap();
        
        index.insert(b"key1".to_vec(), NodeId::new()).unwrap();
        index.insert(b"key2".to_vec(), NodeId::new()).unwrap();
        
        let stats = index.stats().unwrap();
        assert_eq!(stats.total_entries, 2);
    }

    #[test]
    fn test_btree_index_clear() {
        let mut index = BTreeIndex::new_temp().unwrap();
        
        index.insert(b"key1".to_vec(), NodeId::new()).unwrap();
        assert_eq!(index.len(), 1);
        
        index.clear().unwrap();
        assert_eq!(index.len(), 0);
    }
}

