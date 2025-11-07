//! Indexing system for efficient queries
//!
//! Provides multiple index types for different query patterns:
//! - Hash indices for equality lookups (O(1))
//! - B-tree indices for range queries (O(log n))
//! - Composite indices for multi-column queries

pub mod hash;
pub mod btree;
pub mod manager;

pub use hash::HashIndex;
pub use btree::BTreeIndex;
pub use manager::{IndexManager, IndexType, IndexConfig};

use crate::error::Result;
use crate::graph::{NodeId, PropertyValue};

/// Trait for index implementations
pub trait Index: Send + Sync {
    /// Insert a key-value pair into the index
    fn insert(&mut self, key: Vec<u8>, value: NodeId) -> Result<()>;
    
    /// Remove a key-value pair from the index
    fn remove(&mut self, key: &[u8], value: NodeId) -> Result<()>;
    
    /// Lookup values by exact key
    fn lookup(&self, key: &[u8]) -> Result<Vec<NodeId>>;
    
    /// Range query (for indices that support it)
    fn range(&self, start: &[u8], end: &[u8]) -> Result<Vec<NodeId>>;
    
    /// Get all keys in the index
    fn keys(&self) -> Result<Vec<Vec<u8>>>;
    
    /// Clear the index
    fn clear(&mut self) -> Result<()>;
    
    /// Get index size (number of entries)
    fn len(&self) -> usize;
    
    /// Check if index is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Convert a PropertyValue to bytes for indexing
pub fn property_to_bytes(value: &PropertyValue) -> Vec<u8> {
    match value {
        PropertyValue::String(s) => s.as_bytes().to_vec(),
        PropertyValue::Integer(i) => i.to_le_bytes().to_vec(),
        PropertyValue::Float(f) => f.to_le_bytes().to_vec(),
        PropertyValue::Boolean(b) => vec![if *b { 1 } else { 0 }],
        PropertyValue::Null => vec![0],
        PropertyValue::List(_) | PropertyValue::Map(_) => {
            // For complex types, use JSON serialization
            serde_json::to_vec(value).unwrap_or_default()
        }
    }
}

/// Convert bytes back to a string key (for display)
pub fn bytes_to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_to_bytes_string() {
        let value = PropertyValue::String("test".to_string());
        let bytes = property_to_bytes(&value);
        assert_eq!(bytes, b"test");
    }

    #[test]
    fn test_property_to_bytes_integer() {
        let value = PropertyValue::Integer(42);
        let bytes = property_to_bytes(&value);
        assert_eq!(bytes.len(), 8); // i64 is 8 bytes
    }

    #[test]
    fn test_property_to_bytes_boolean() {
        let value = PropertyValue::Boolean(true);
        let bytes = property_to_bytes(&value);
        assert_eq!(bytes, vec![1]);
    }
}

