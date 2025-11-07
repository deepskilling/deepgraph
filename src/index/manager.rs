//! Index manager for coordinating multiple indices
//!
//! Manages all indices and provides query optimization hints

use crate::error::{DeepGraphError, Result};
use crate::graph::{NodeId, PropertyValue};
use crate::index::{property_to_bytes, BTreeIndex, HashIndex, Index};
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::RwLock;

/// Type of index
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndexType {
    /// Hash index for equality lookups
    Hash,
    /// B-tree index for range queries
    BTree,
}

/// Configuration for creating an index
#[derive(Debug, Clone)]
pub struct IndexConfig {
    /// Name of the index
    pub name: String,
    /// Type of index
    pub index_type: IndexType,
    /// Property key to index (None for labels)
    pub property_key: Option<String>,
    /// Whether this is a label index
    pub is_label_index: bool,
}

impl IndexConfig {
    /// Create a label index configuration
    pub fn label_index(name: String, index_type: IndexType) -> Self {
        Self {
            name,
            index_type,
            property_key: None,
            is_label_index: true,
        }
    }
    
    /// Create a property index configuration
    pub fn property_index(name: String, index_type: IndexType, property_key: String) -> Self {
        Self {
            name,
            index_type,
            property_key: Some(property_key),
            is_label_index: false,
        }
    }
}

/// Wrapper for different index types
enum IndexImpl {
    Hash(RwLock<HashIndex>),
    BTree(RwLock<BTreeIndex>),
}

/// Index manager
pub struct IndexManager {
    /// All indices by name
    indices: DashMap<String, IndexImpl>,
    /// Label indices (label -> index name)
    label_indices: DashMap<String, String>,
    /// Property indices (property key -> index name)
    property_indices: DashMap<String, String>,
    /// Base directory for persistent indices
    base_dir: Option<PathBuf>,
}

impl IndexManager {
    /// Create a new index manager
    pub fn new() -> Self {
        Self {
            indices: DashMap::new(),
            label_indices: DashMap::new(),
            property_indices: DashMap::new(),
            base_dir: None,
        }
    }
    
    /// Create an index manager with persistence
    pub fn with_persistence(base_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&base_dir)
            .map_err(|e| DeepGraphError::IoError(e))?;
        
        Ok(Self {
            indices: DashMap::new(),
            label_indices: DashMap::new(),
            property_indices: DashMap::new(),
            base_dir: Some(base_dir),
        })
    }
    
    /// Create an index
    pub fn create_index(&self, config: IndexConfig) -> Result<()> {
        let index_impl = match config.index_type {
            IndexType::Hash => {
                IndexImpl::Hash(RwLock::new(HashIndex::new()))
            }
            IndexType::BTree => {
                let btree = if let Some(base_dir) = &self.base_dir {
                    let index_path = base_dir.join(&config.name);
                    BTreeIndex::new(&index_path, &config.name)?
                } else {
                    BTreeIndex::new_temp()?
                };
                IndexImpl::BTree(RwLock::new(btree))
            }
        };
        
        // Register the index
        self.indices.insert(config.name.clone(), index_impl);
        
        // Track label or property index
        if config.is_label_index {
            self.label_indices.insert(config.name.clone(), config.name.clone());
        } else if let Some(prop_key) = config.property_key {
            self.property_indices.insert(prop_key, config.name);
        }
        
        Ok(())
    }
    
    /// Drop an index
    pub fn drop_index(&self, name: &str) -> Result<()> {
        self.indices
            .remove(name)
            .ok_or_else(|| DeepGraphError::StorageError(format!("Index {} not found", name)))?;
        
        // Remove from tracking maps
        self.label_indices.retain(|_, v| v != name);
        self.property_indices.retain(|_, v| v != name);
        
        Ok(())
    }
    
    /// Insert into label index
    pub fn insert_label(&self, label: &str, node_id: NodeId) -> Result<()> {
        if let Some(index_name) = self.label_indices.get(label) {
            if let Some(index_entry) = self.indices.get(index_name.value()) {
                match index_entry.value() {
                    IndexImpl::Hash(index) => {
                        index.write().unwrap().insert(label.as_bytes().to_vec(), node_id)?;
                    }
                    IndexImpl::BTree(index) => {
                        index.write().unwrap().insert(label.as_bytes().to_vec(), node_id)?;
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Insert into property index
    pub fn insert_property(&self, key: &str, value: &PropertyValue, node_id: NodeId) -> Result<()> {
        if let Some(index_name) = self.property_indices.get(key) {
            if let Some(index_entry) = self.indices.get(index_name.value()) {
                let bytes = property_to_bytes(value);
                
                match index_entry.value() {
                    IndexImpl::Hash(index) => {
                        index.write().unwrap().insert(bytes, node_id)?;
                    }
                    IndexImpl::BTree(index) => {
                        index.write().unwrap().insert(bytes, node_id)?;
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Lookup by label
    pub fn lookup_label(&self, label: &str) -> Result<Vec<NodeId>> {
        if let Some(index_name) = self.label_indices.get(label) {
            if let Some(index_entry) = self.indices.get(index_name.value()) {
                return match index_entry.value() {
                    IndexImpl::Hash(index) => {
                        index.read().unwrap().lookup(label.as_bytes())
                    }
                    IndexImpl::BTree(index) => {
                        index.read().unwrap().lookup(label.as_bytes())
                    }
                };
            }
        }
        Ok(Vec::new())
    }
    
    /// Lookup by property value
    pub fn lookup_property(&self, key: &str, value: &PropertyValue) -> Result<Vec<NodeId>> {
        if let Some(index_name) = self.property_indices.get(key) {
            if let Some(index_entry) = self.indices.get(index_name.value()) {
                let bytes = property_to_bytes(value);
                
                return match index_entry.value() {
                    IndexImpl::Hash(index) => {
                        index.read().unwrap().lookup(&bytes)
                    }
                    IndexImpl::BTree(index) => {
                        index.read().unwrap().lookup(&bytes)
                    }
                };
            }
        }
        Ok(Vec::new())
    }
    
    /// Range query on a property (only works with B-tree indices)
    pub fn range_property(
        &self,
        key: &str,
        start: &PropertyValue,
        end: &PropertyValue,
    ) -> Result<Vec<NodeId>> {
        if let Some(index_name) = self.property_indices.get(key) {
            if let Some(index_entry) = self.indices.get(index_name.value()) {
                match index_entry.value() {
                    IndexImpl::BTree(index) => {
                        let start_bytes = property_to_bytes(start);
                        let end_bytes = property_to_bytes(end);
                        return index.read().unwrap().range(&start_bytes, &end_bytes);
                    }
                    IndexImpl::Hash(_) => {
                        return Err(DeepGraphError::StorageError(
                            "Range queries not supported on hash indices".to_string()
                        ));
                    }
                }
            }
        }
        Ok(Vec::new())
    }
    
    /// Check if an index exists for a label
    pub fn has_label_index(&self, label: &str) -> bool {
        self.label_indices.contains_key(label)
    }
    
    /// Check if an index exists for a property
    pub fn has_property_index(&self, key: &str) -> bool {
        self.property_indices.contains_key(key)
    }
    
    /// Get all index names
    pub fn list_indices(&self) -> Vec<String> {
        self.indices
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
    
    /// Get index count
    pub fn index_count(&self) -> usize {
        self.indices.len()
    }
}

impl Default for IndexManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_label_index() {
        let manager = IndexManager::new();
        let config = IndexConfig::label_index("person_label".to_string(), IndexType::Hash);
        
        manager.create_index(config).unwrap();
        assert_eq!(manager.index_count(), 1);
    }

    #[test]
    fn test_create_property_index() {
        let manager = IndexManager::new();
        let config = IndexConfig::property_index(
            "age_index".to_string(),
            IndexType::BTree,
            "age".to_string(),
        );
        
        manager.create_index(config).unwrap();
        assert_eq!(manager.index_count(), 1);
        assert!(manager.has_property_index("age"));
    }

    #[test]
    fn test_label_insert_and_lookup() {
        let manager = IndexManager::new();
        
        // Create label index
        let config = IndexConfig::label_index("person".to_string(), IndexType::Hash);
        manager.create_index(config).unwrap();
        
        // Insert
        let node_id = NodeId::new();
        manager.insert_label("person", node_id).unwrap();
        
        // Lookup
        let results = manager.lookup_label("person").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], node_id);
    }

    #[test]
    fn test_property_insert_and_lookup() {
        let manager = IndexManager::new();
        
        // Create property index
        let config = IndexConfig::property_index(
            "age".to_string(),
            IndexType::Hash,
            "age".to_string(),
        );
        manager.create_index(config).unwrap();
        
        // Insert
        let node_id = NodeId::new();
        let value = PropertyValue::Integer(30);
        manager.insert_property("age", &value, node_id).unwrap();
        
        // Lookup
        let results = manager.lookup_property("age", &value).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], node_id);
    }

    #[test]
    fn test_range_query() {
        let manager = IndexManager::new();
        
        // Create B-tree index for range queries
        let config = IndexConfig::property_index(
            "age".to_string(),
            IndexType::BTree,
            "age".to_string(),
        );
        manager.create_index(config).unwrap();
        
        // Insert multiple values
        let node1 = NodeId::new();
        let node2 = NodeId::new();
        let node3 = NodeId::new();
        
        manager.insert_property("age", &PropertyValue::Integer(25), node1).unwrap();
        manager.insert_property("age", &PropertyValue::Integer(30), node2).unwrap();
        manager.insert_property("age", &PropertyValue::Integer(35), node3).unwrap();
        
        // Range query: 28 to 32
        let results = manager.range_property(
            "age",
            &PropertyValue::Integer(28),
            &PropertyValue::Integer(32),
        ).unwrap();
        
        // Should only return node2 (age 30)
        assert_eq!(results.len(), 1);
        assert!(results.contains(&node2));
    }

    #[test]
    fn test_drop_index() {
        let manager = IndexManager::new();
        
        let config = IndexConfig::label_index("test".to_string(), IndexType::Hash);
        manager.create_index(config).unwrap();
        assert_eq!(manager.index_count(), 1);
        
        manager.drop_index("test").unwrap();
        assert_eq!(manager.index_count(), 0);
    }
}

