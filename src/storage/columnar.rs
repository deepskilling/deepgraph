//! Columnar storage implementation using Apache Arrow
//!
//! This provides high-performance storage with efficient memory layout
//! and support for analytical queries.

use crate::error::{DeepGraphError, Result};
use crate::graph::{Edge, EdgeId, Node, NodeId, PropertyValue};
use crate::storage::schema::{edge_schema, node_schema};
use crate::storage::StorageBackend;

use arrow::array::{
    Array, ArrayRef, FixedSizeBinaryArray, FixedSizeBinaryBuilder,
    ListArray, RecordBatch, StringBuilder, StringArray,
};
use arrow::buffer::OffsetBuffer;
use arrow::datatypes::Schema;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Columnar storage using Apache Arrow
///
/// Stores nodes and edges in columnar format for efficient querying
/// and analytical workloads. Provides better compression and cache locality
/// compared to row-based storage.
pub struct ColumnarStorage {
    /// Node batches (columnar format)
    node_batches: RwLock<Vec<RecordBatch>>,
    /// Edge batches (columnar format)
    #[allow(dead_code)] // TODO: Implement edge serialization in Phase 2
    edge_batches: RwLock<Vec<RecordBatch>>,
    /// In-memory index for fast lookups (node_id -> batch_index, row_index)
    node_index: DashMap<NodeId, (usize, usize)>,
    /// In-memory index for fast lookups (edge_id -> batch_index, row_index)
    edge_index: DashMap<EdgeId, (usize, usize)>,
    /// Outgoing edges index
    outgoing_edges: DashMap<NodeId, Vec<EdgeId>>,
    /// Incoming edges index
    incoming_edges: DashMap<NodeId, Vec<EdgeId>>,
    /// Node schema
    node_schema: Arc<Schema>,
    /// Edge schema
    #[allow(dead_code)] // TODO: Implement edge serialization in Phase 2
    edge_schema: Arc<Schema>,
}

impl ColumnarStorage {
    /// Create a new columnar storage
    pub fn new() -> Self {
        Self {
            node_batches: RwLock::new(Vec::new()),
            edge_batches: RwLock::new(Vec::new()),
            node_index: DashMap::new(),
            edge_index: DashMap::new(),
            outgoing_edges: DashMap::new(),
            incoming_edges: DashMap::new(),
            node_schema: node_schema(),
            edge_schema: edge_schema(),
        }
    }

    /// Serialize a node to Arrow format and add to batch
    fn serialize_node(&self, node: &Node) -> Result<()> {
        let id = node.id();
        let id_bytes = id.as_uuid().as_bytes().to_vec();
        
        // Build Arrow arrays
        let mut id_builder = FixedSizeBinaryBuilder::new(16);
        id_builder.append_value(&id_bytes).map_err(|e| {
            DeepGraphError::StorageError(format!("Failed to append ID: {}", e))
        })?;
        
        // Build labels list with correct schema
        let mut string_builder = StringBuilder::new();
        for label in node.labels() {
            string_builder.append_value(label);
        }
        let string_array = string_builder.finish();
        
        let field = arrow::datatypes::Field::new("item", arrow::datatypes::DataType::Utf8, false);
        let labels_array = ListArray::new(
            Arc::new(field),
            OffsetBuffer::from_lengths(vec![node.labels().len()]),
            Arc::new(string_array),
            None,
        );
        
        let mut props_builder = StringBuilder::new();
        let props_json = serde_json::to_string(node.properties())
            .map_err(|e| DeepGraphError::SerializationError(e.to_string()))?;
        props_builder.append_value(&props_json);
        
        let mut created_builder = arrow::array::Int64Builder::new();
        created_builder.append_value(chrono::Utc::now().timestamp());
        
        let mut updated_builder = arrow::array::Int64Builder::new();
        updated_builder.append_value(chrono::Utc::now().timestamp());
        
        // Create record batch
        let batch = RecordBatch::try_new(
            self.node_schema.clone(),
            vec![
                Arc::new(id_builder.finish()) as ArrayRef,
                Arc::new(labels_array) as ArrayRef,
                Arc::new(props_builder.finish()) as ArrayRef,
                Arc::new(created_builder.finish()) as ArrayRef,
                Arc::new(updated_builder.finish()) as ArrayRef,
            ],
        ).map_err(|e| DeepGraphError::StorageError(format!("Failed to create batch: {}", e)))?;
        
        // Add to batches
        let mut batches = self.node_batches.write();
        let batch_idx = batches.len();
        let row_idx = 0;
        batches.push(batch);
        
        // Update index
        self.node_index.insert(id, (batch_idx, row_idx));
        
        Ok(())
    }

    /// Deserialize a node from Arrow format
    fn deserialize_node(&self, batch_idx: usize, row_idx: usize) -> Result<Node> {
        let batches = self.node_batches.read();
        let batch = batches.get(batch_idx)
            .ok_or_else(|| DeepGraphError::StorageError("Batch not found".to_string()))?;
        
        // Extract ID
        let id_array = batch.column(0)
            .as_any()
            .downcast_ref::<FixedSizeBinaryArray>()
            .ok_or_else(|| DeepGraphError::StorageError("Invalid ID column".to_string()))?;
        let id_bytes = id_array.value(row_idx);
        let id = NodeId::from_uuid(uuid::Uuid::from_slice(id_bytes)
            .map_err(|e| DeepGraphError::InvalidNodeId(e.to_string()))?);
        
        // Extract labels
        let labels_array = batch.column(1)
            .as_any()
            .downcast_ref::<ListArray>()
            .ok_or_else(|| DeepGraphError::StorageError("Invalid labels column".to_string()))?;
        let labels_values = labels_array.value(row_idx);
        let labels_str = labels_values
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| DeepGraphError::StorageError("Invalid labels data".to_string()))?;
        
        let mut labels = Vec::new();
        for i in 0..labels_str.len() {
            labels.push(labels_str.value(i).to_string());
        }
        
        // Extract properties
        let props_array = batch.column(2)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| DeepGraphError::StorageError("Invalid properties column".to_string()))?;
        let props_json = props_array.value(row_idx);
        let properties: HashMap<String, PropertyValue> = serde_json::from_str(props_json)
            .map_err(|e| DeepGraphError::SerializationError(e.to_string()))?;
        
        // Reconstruct node
        let mut node = Node::with_id(id, labels);
        for (key, value) in properties {
            node.set_property(key, value);
        }
        
        Ok(node)
    }
}

impl Default for ColumnarStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageBackend for ColumnarStorage {
    fn add_node(&self, node: Node) -> Result<NodeId> {
        let id = node.id();
        self.serialize_node(&node)?;
        Ok(id)
    }
    
    fn get_node(&self, id: NodeId) -> Result<Node> {
        let (batch_idx, row_idx) = self.node_index
            .get(&id)
            .map(|entry| *entry.value())
            .ok_or_else(|| DeepGraphError::NodeNotFound(id.to_string()))?;
        
        self.deserialize_node(batch_idx, row_idx)
    }
    
    fn update_node(&self, node: Node) -> Result<()> {
        let id = node.id();
        if !self.node_index.contains_key(&id) {
            return Err(DeepGraphError::NodeNotFound(id.to_string()));
        }
        
        // For now, simple implementation: mark old as deleted and add new
        // TODO: Implement in-place update or better versioning
        self.serialize_node(&node)?;
        Ok(())
    }
    
    fn delete_node(&self, id: NodeId) -> Result<()> {
        self.node_index
            .remove(&id)
            .ok_or_else(|| DeepGraphError::NodeNotFound(id.to_string()))?;
        
        // Remove associated edges
        if let Some((_, edge_ids)) = self.outgoing_edges.remove(&id) {
            for edge_id in edge_ids {
                self.edge_index.remove(&edge_id);
            }
        }
        
        if let Some((_, edge_ids)) = self.incoming_edges.remove(&id) {
            for edge_id in edge_ids {
                self.edge_index.remove(&edge_id);
            }
        }
        
        Ok(())
    }
    
    fn add_edge(&self, edge: Edge) -> Result<EdgeId> {
        let id = edge.id();
        let from = edge.from();
        let to = edge.to();
        
        // Verify nodes exist
        if !self.node_index.contains_key(&from) {
            return Err(DeepGraphError::NodeNotFound(from.to_string()));
        }
        if !self.node_index.contains_key(&to) {
            return Err(DeepGraphError::NodeNotFound(to.to_string()));
        }
        
        // TODO: Serialize edge to Arrow format (similar to nodes)
        // For now, use placeholder
        
        // Update indices
        self.outgoing_edges
            .entry(from)
            .or_insert_with(Vec::new)
            .push(id);
        
        self.incoming_edges
            .entry(to)
            .or_insert_with(Vec::new)
            .push(id);
        
        Ok(id)
    }
    
    fn get_edge(&self, id: EdgeId) -> Result<Edge> {
        // TODO: Implement edge deserialization
        Err(DeepGraphError::EdgeNotFound(id.to_string()))
    }
    
    fn update_edge(&self, _edge: Edge) -> Result<()> {
        // TODO: Implement edge update
        Ok(())
    }
    
    fn delete_edge(&self, id: EdgeId) -> Result<()> {
        self.edge_index
            .remove(&id)
            .ok_or_else(|| DeepGraphError::EdgeNotFound(id.to_string()))?;
        Ok(())
    }
    
    fn get_nodes_by_label(&self, label: &str) -> Vec<Node> {
        // TODO: Use label index for efficiency
        let mut nodes = Vec::new();
        for entry in self.node_index.iter() {
            let (batch_idx, row_idx) = *entry.value();
            if let Ok(node) = self.deserialize_node(batch_idx, row_idx) {
                if node.has_label(label) {
                    nodes.push(node);
                }
            }
        }
        nodes
    }
    
    fn get_all_nodes(&self) -> Vec<Node> {
        // Get all nodes (full scan)
        let mut nodes = Vec::new();
        for entry in self.node_index.iter() {
            let (batch_idx, row_idx) = *entry.value();
            if let Ok(node) = self.deserialize_node(batch_idx, row_idx) {
                nodes.push(node);
            }
        }
        nodes
    }
    
    fn get_outgoing_edges(&self, node_id: NodeId) -> Result<Vec<Edge>> {
        if !self.node_index.contains_key(&node_id) {
            return Err(DeepGraphError::NodeNotFound(node_id.to_string()));
        }
        
        // TODO: Deserialize edges
        Ok(Vec::new())
    }
    
    fn get_incoming_edges(&self, node_id: NodeId) -> Result<Vec<Edge>> {
        if !self.node_index.contains_key(&node_id) {
            return Err(DeepGraphError::NodeNotFound(node_id.to_string()));
        }
        
        // TODO: Deserialize edges
        Ok(Vec::new())
    }
    
    fn node_count(&self) -> usize {
        self.node_index.len()
    }
    
    fn edge_count(&self) -> usize {
        self.edge_index.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_columnar_storage_creation() {
        let storage = ColumnarStorage::new();
        assert_eq!(storage.node_count(), 0);
        assert_eq!(storage.edge_count(), 0);
    }

    #[test]
    fn test_add_and_get_node() {
        let storage = ColumnarStorage::new();
        let mut node = Node::new(vec!["Person".to_string()]);
        node.set_property("name".to_string(), "Alice".into());
        
        let id = storage.add_node(node.clone()).unwrap();
        let retrieved = storage.get_node(id).unwrap();
        
        assert_eq!(retrieved.id(), id);
        assert_eq!(retrieved.labels(), node.labels());
    }
}

