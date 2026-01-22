//! Disk-based persistent storage using Sled
//!
//! This module provides a disk-first storage backend that can handle
//! graphs larger than RAM with ACID guarantees and crash recovery.

use crate::error::{DeepGraphError, Result};
use crate::graph::{Edge, EdgeId, Node, NodeId, PropertyValue};
use log::{debug, info, warn};
use sled::{Db, Tree};
use std::path::Path;

/// Disk-based storage using Sled embedded database
///
/// Features:
/// - Persistent storage (survives restarts)
/// - ACID guarantees
/// - Crash recovery
/// - Supports graphs larger than RAM
/// - Fast lookups with indices
///
/// # Example
///
/// ```rust
/// use deepgraph::storage::DiskStorage;
///
/// let storage = DiskStorage::new("./data/graph.db")?;
/// let node = Node::new(vec!["Person".to_string()]);
/// let id = storage.add_node(node)?;
/// ```
pub struct DiskStorage {
    /// Sled database instance
    db: Db,
    /// Tree for storing nodes (NodeId → Node)
    nodes: Tree,
    /// Tree for storing edges (EdgeId → Edge)
    edges: Tree,
    /// Tree for label index (Label → Vec<NodeId>)
    label_index: Tree,
    /// Tree for outgoing edges (NodeId → Vec<EdgeId>)
    outgoing_edges: Tree,
    /// Tree for incoming edges (NodeId → Vec<EdgeId>)
    incoming_edges: Tree,
    /// Tree for property index (PropertyKey → Vec<NodeId>)
    #[allow(dead_code)] // Will be used for property queries in the future
    property_index: Tree,
    /// Tree for edge type index (EdgeType → Vec<EdgeId>)
    edge_type_index: Tree,
}

impl DiskStorage {
    /// Create or open a disk-based storage at the given path
    ///
    /// # Arguments
    ///
    /// * `path` - Directory path for the database
    ///
    /// # Example
    ///
    /// ```rust
    /// let storage = DiskStorage::new("./data/my_graph")?;
    /// ```
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        info!("Opening disk storage at {:?}", path.as_ref());
        
        let db = sled::open(path.as_ref())
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to open Sled database: {}", e)))?;
        
        // Open all trees
        let nodes = db.open_tree("nodes")
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to open nodes tree: {}", e)))?;
        
        let edges = db.open_tree("edges")
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to open edges tree: {}", e)))?;
        
        let label_index = db.open_tree("label_index")
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to open label_index tree: {}", e)))?;
        
        let outgoing_edges = db.open_tree("outgoing_edges")
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to open outgoing_edges tree: {}", e)))?;
        
        let incoming_edges = db.open_tree("incoming_edges")
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to open incoming_edges tree: {}", e)))?;
        
        let property_index = db.open_tree("property_index")
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to open property_index tree: {}", e)))?;
        
        let edge_type_index = db.open_tree("edge_type_index")
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to open edge_type_index tree: {}", e)))?;
        
        info!("Disk storage opened successfully");
        info!("  Nodes: {}", nodes.len());
        info!("  Edges: {}", edges.len());
        
        Ok(Self {
            db,
            nodes,
            edges,
            label_index,
            outgoing_edges,
            incoming_edges,
            property_index,
            edge_type_index,
        })
    }
    
    /// Flush all pending writes to disk
    ///
    /// Ensures all data is persisted. Called automatically on important operations,
    /// but can be called manually for explicit durability.
    pub fn flush(&self) -> Result<()> {
        debug!("Flushing disk storage");
        self.db.flush()
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to flush: {}", e)))?;
        Ok(())
    }
    
    /// Get database statistics
    pub fn stats(&self) -> DiskStorageStats {
        DiskStorageStats {
            node_count: self.nodes.len(),
            edge_count: self.edges.len(),
            size_on_disk_bytes: self.db.size_on_disk().unwrap_or(0),
        }
    }
    
    // --- Helper methods for serialization ---
    
    /// Serialize a node to bytes
    fn serialize_node(&self, node: &Node) -> Result<Vec<u8>> {
        bincode::serialize(node)
            .map_err(|e| DeepGraphError::SerializationError(format!("Failed to serialize node: {}", e)))
    }
    
    /// Deserialize a node from bytes
    fn deserialize_node(&self, bytes: &[u8]) -> Result<Node> {
        bincode::deserialize(bytes)
            .map_err(|e| DeepGraphError::SerializationError(format!("Failed to deserialize node: {}", e)))
    }
    
    /// Serialize an edge to bytes
    fn serialize_edge(&self, edge: &Edge) -> Result<Vec<u8>> {
        bincode::serialize(edge)
            .map_err(|e| DeepGraphError::SerializationError(format!("Failed to serialize edge: {}", e)))
    }
    
    /// Deserialize an edge from bytes
    fn deserialize_edge(&self, bytes: &[u8]) -> Result<Edge> {
        bincode::deserialize(bytes)
            .map_err(|e| DeepGraphError::SerializationError(format!("Failed to deserialize edge: {}", e)))
    }
    
    /// Serialize a vector of NodeIds
    fn serialize_node_ids(&self, ids: &[NodeId]) -> Result<Vec<u8>> {
        bincode::serialize(ids)
            .map_err(|e| DeepGraphError::SerializationError(format!("Failed to serialize node IDs: {}", e)))
    }
    
    /// Deserialize a vector of NodeIds
    fn deserialize_node_ids(&self, bytes: &[u8]) -> Result<Vec<NodeId>> {
        bincode::deserialize(bytes)
            .map_err(|e| DeepGraphError::SerializationError(format!("Failed to deserialize node IDs: {}", e)))
    }
    
    /// Serialize a vector of EdgeIds
    fn serialize_edge_ids(&self, ids: &[EdgeId]) -> Result<Vec<u8>> {
        bincode::serialize(ids)
            .map_err(|e| DeepGraphError::SerializationError(format!("Failed to serialize edge IDs: {}", e)))
    }
    
    /// Deserialize a vector of EdgeIds
    fn deserialize_edge_ids(&self, bytes: &[u8]) -> Result<Vec<EdgeId>> {
        bincode::deserialize(bytes)
            .map_err(|e| DeepGraphError::SerializationError(format!("Failed to deserialize edge IDs: {}", e)))
    }
    
    // --- Helper methods for index management ---
    
    /// Add a node to the label index
    fn add_to_label_index(&self, label: &str, node_id: NodeId) -> Result<()> {
        let mut ids = self.get_nodes_for_label(label)?;
        if !ids.contains(&node_id) {
            ids.push(node_id);
            let bytes = self.serialize_node_ids(&ids)?;
            self.label_index.insert(label.as_bytes(), bytes)
                .map_err(|e| DeepGraphError::StorageError(format!("Failed to update label index: {}", e)))?;
        }
        Ok(())
    }
    
    /// Remove a node from the label index
    fn remove_from_label_index(&self, label: &str, node_id: NodeId) -> Result<()> {
        let mut ids = self.get_nodes_for_label(label)?;
        ids.retain(|&id| id != node_id);
        let bytes = self.serialize_node_ids(&ids)?;
        self.label_index.insert(label.as_bytes(), bytes)
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to update label index: {}", e)))?;
        Ok(())
    }
    
    /// Get all nodes with a specific label
    fn get_nodes_for_label(&self, label: &str) -> Result<Vec<NodeId>> {
        match self.label_index.get(label.as_bytes())
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to read label index: {}", e)))? {
            Some(bytes) => self.deserialize_node_ids(&bytes),
            None => Ok(Vec::new()),
        }
    }
    
    /// Add an edge to outgoing edges index
    fn add_to_outgoing_edges(&self, node_id: NodeId, edge_id: EdgeId) -> Result<()> {
        let mut edges = self.get_outgoing_edge_ids(node_id)?;
        if !edges.contains(&edge_id) {
            edges.push(edge_id);
            let bytes = self.serialize_edge_ids(&edges)?;
            self.outgoing_edges.insert(node_id.as_bytes(), bytes)
                .map_err(|e| DeepGraphError::StorageError(format!("Failed to update outgoing edges: {}", e)))?;
        }
        Ok(())
    }
    
    /// Add an edge to incoming edges index
    fn add_to_incoming_edges(&self, node_id: NodeId, edge_id: EdgeId) -> Result<()> {
        let mut edges = self.get_incoming_edge_ids(node_id)?;
        if !edges.contains(&edge_id) {
            edges.push(edge_id);
            let bytes = self.serialize_edge_ids(&edges)?;
            self.incoming_edges.insert(node_id.as_bytes(), bytes)
                .map_err(|e| DeepGraphError::StorageError(format!("Failed to update incoming edges: {}", e)))?;
        }
        Ok(())
    }
    
    /// Get outgoing edge IDs for a node
    fn get_outgoing_edge_ids(&self, node_id: NodeId) -> Result<Vec<EdgeId>> {
        match self.outgoing_edges.get(node_id.as_bytes())
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to read outgoing edges: {}", e)))? {
            Some(bytes) => self.deserialize_edge_ids(&bytes),
            None => Ok(Vec::new()),
        }
    }
    
    /// Get incoming edge IDs for a node
    fn get_incoming_edge_ids(&self, node_id: NodeId) -> Result<Vec<EdgeId>> {
        match self.incoming_edges.get(node_id.as_bytes())
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to read incoming edges: {}", e)))? {
            Some(bytes) => self.deserialize_edge_ids(&bytes),
            None => Ok(Vec::new()),
        }
    }
    
    /// Remove an edge from outgoing edges index
    fn remove_from_outgoing_edges(&self, node_id: NodeId, edge_id: EdgeId) -> Result<()> {
        let mut edges = self.get_outgoing_edge_ids(node_id)?;
        edges.retain(|&id| id != edge_id);
        let bytes = self.serialize_edge_ids(&edges)?;
        self.outgoing_edges.insert(node_id.as_bytes(), bytes)
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to update outgoing edges: {}", e)))?;
        Ok(())
    }
    
    /// Remove an edge from incoming edges index
    fn remove_from_incoming_edges(&self, node_id: NodeId, edge_id: EdgeId) -> Result<()> {
        let mut edges = self.get_incoming_edge_ids(node_id)?;
        edges.retain(|&id| id != edge_id);
        let bytes = self.serialize_edge_ids(&edges)?;
        self.incoming_edges.insert(node_id.as_bytes(), bytes)
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to update incoming edges: {}", e)))?;
        Ok(())
    }
    
    /// Add an edge to the edge type index
    fn add_to_edge_type_index(&self, edge_type: &str, edge_id: EdgeId) -> Result<()> {
        let mut ids = self.get_edges_for_type(edge_type)?;
        if !ids.contains(&edge_id) {
            ids.push(edge_id);
            let bytes = self.serialize_edge_ids(&ids)?;
            self.edge_type_index.insert(edge_type.as_bytes(), bytes)
                .map_err(|e| DeepGraphError::StorageError(format!("Failed to update edge type index: {}", e)))?;
        }
        Ok(())
    }
    
    /// Get all edges of a specific type
    fn get_edges_for_type(&self, edge_type: &str) -> Result<Vec<EdgeId>> {
        match self.edge_type_index.get(edge_type.as_bytes())
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to read edge type index: {}", e)))? {
            Some(bytes) => self.deserialize_edge_ids(&bytes),
            None => Ok(Vec::new()),
        }
    }
}

/// Statistics about disk storage
#[derive(Debug, Clone)]
pub struct DiskStorageStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub size_on_disk_bytes: u64,
}

// --- Implement StorageBackend trait ---

use crate::storage::StorageBackend;

impl StorageBackend for DiskStorage {
    fn add_node(&self, node: Node) -> Result<NodeId> {
        let id = node.id();
        debug!("Adding node {} to disk storage", id);
        
        // Serialize and store node
        let bytes = self.serialize_node(&node)?;
        self.nodes.insert(id.as_bytes(), bytes)
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to insert node: {}", e)))?;
        
        // Update label indices
        for label in node.labels() {
            self.add_to_label_index(label, id)?;
        }
        
        // Flush to ensure durability
        self.flush()?;
        
        debug!("Node {} added successfully", id);
        Ok(id)
    }
    
    fn get_node(&self, id: NodeId) -> Result<Node> {
        debug!("Getting node {} from disk storage", id);
        
        match self.nodes.get(id.as_bytes())
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to get node: {}", e)))? {
            Some(bytes) => {
                let node = self.deserialize_node(&bytes)?;
                Ok(node)
            }
            None => {
                warn!("Node {} not found", id);
                Err(DeepGraphError::NotFound(format!("Node {} not found", id)))
            }
        }
    }
    
    fn update_node(&self, node: Node) -> Result<()> {
        let id = node.id();
        debug!("Updating node {} in disk storage", id);
        
        // Check if node exists
        if !self.nodes.contains_key(id.as_bytes())
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to check node existence: {}", e)))? {
            return Err(DeepGraphError::NotFound(format!("Node {} not found", id)));
        }
        
        // Get old node to update indices
        let old_node = self.get_node(id)?;
        
        // Remove old labels from index
        for label in old_node.labels() {
            self.remove_from_label_index(label, id)?;
        }
        
        // Add new labels to index
        for label in node.labels() {
            self.add_to_label_index(label, id)?;
        }
        
        // Update node
        let bytes = self.serialize_node(&node)?;
        self.nodes.insert(id.as_bytes(), bytes)
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to update node: {}", e)))?;
        
        self.flush()?;
        
        debug!("Node {} updated successfully", id);
        Ok(())
    }
    
    fn delete_node(&self, id: NodeId) -> Result<()> {
        debug!("Deleting node {} from disk storage", id);
        
        // Get node to update indices
        let node = self.get_node(id)?;
        
        // Remove from label indices
        for label in node.labels() {
            self.remove_from_label_index(label, id)?;
        }
        
        // Remove all edges connected to this node
        let outgoing = self.get_outgoing_edge_ids(id)?;
        for edge_id in outgoing {
            self.delete_edge(edge_id)?;
        }
        
        let incoming = self.get_incoming_edge_ids(id)?;
        for edge_id in incoming {
            self.delete_edge(edge_id)?;
        }
        
        // Remove node
        self.nodes.remove(id.as_bytes())
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to delete node: {}", e)))?;
        
        self.flush()?;
        
        debug!("Node {} deleted successfully", id);
        Ok(())
    }
    
    fn add_edge(&self, edge: Edge) -> Result<EdgeId> {
        let id = edge.id();
        debug!("Adding edge {} to disk storage", id);
        
        // Verify nodes exist
        let _from = self.get_node(edge.from())?;
        let _to = self.get_node(edge.to())?;
        
        // Serialize and store edge
        let bytes = self.serialize_edge(&edge)?;
        self.edges.insert(id.as_bytes(), bytes)
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to insert edge: {}", e)))?;
        
        // Update adjacency indices
        self.add_to_outgoing_edges(edge.from(), id)?;
        self.add_to_incoming_edges(edge.to(), id)?;
        
        // Update edge type index
        self.add_to_edge_type_index(edge.relationship_type(), id)?;
        
        self.flush()?;
        
        debug!("Edge {} added successfully", id);
        Ok(id)
    }
    
    fn get_edge(&self, id: EdgeId) -> Result<Edge> {
        debug!("Getting edge {} from disk storage", id);
        
        match self.edges.get(id.as_bytes())
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to get edge: {}", e)))? {
            Some(bytes) => {
                let edge = self.deserialize_edge(&bytes)?;
                Ok(edge)
            }
            None => {
                warn!("Edge {} not found", id);
                Err(DeepGraphError::NotFound(format!("Edge {} not found", id)))
            }
        }
    }
    
    fn update_edge(&self, edge: Edge) -> Result<()> {
        let id = edge.id();
        debug!("Updating edge {} in disk storage", id);
        
        // Check if edge exists
        if !self.edges.contains_key(id.as_bytes())
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to check edge existence: {}", e)))? {
            return Err(DeepGraphError::NotFound(format!("Edge {} not found", id)));
        }
        
        // Update edge
        let bytes = self.serialize_edge(&edge)?;
        self.edges.insert(id.as_bytes(), bytes)
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to update edge: {}", e)))?;
        
        self.flush()?;
        
        debug!("Edge {} updated successfully", id);
        Ok(())
    }
    
    fn delete_edge(&self, id: EdgeId) -> Result<()> {
        debug!("Deleting edge {} from disk storage", id);
        
        // Get edge to update indices
        let edge = self.get_edge(id)?;
        
        // Remove from adjacency indices
        self.remove_from_outgoing_edges(edge.from(), id)?;
        self.remove_from_incoming_edges(edge.to(), id)?;
        
        // Remove edge
        self.edges.remove(id.as_bytes())
            .map_err(|e| DeepGraphError::StorageError(format!("Failed to delete edge: {}", e)))?;
        
        self.flush()?;
        
        debug!("Edge {} deleted successfully", id);
        Ok(())
    }
    
    fn get_nodes_by_label(&self, label: &str) -> Vec<Node> {
        debug!("Getting nodes by label: {}", label);
        
        match self.get_nodes_for_label(label) {
            Ok(ids) => {
                ids.into_iter()
                    .filter_map(|id| self.get_node(id).ok())
                    .collect()
            }
            Err(e) => {
                warn!("Failed to get nodes by label: {}", e);
                Vec::new()
            }
        }
    }
    
    fn get_all_nodes(&self) -> Vec<Node> {
        debug!("Getting all nodes from disk storage");
        
        self.nodes
            .iter()
            .filter_map(|result| {
                match result {
                    Ok((_key, value)) => {
                        self.deserialize_node(&value).ok()
                    }
                    Err(e) => {
                        warn!("Failed to iterate node: {}", e);
                        None
                    }
                }
            })
            .collect()
    }
    
    fn get_outgoing_edges(&self, node_id: NodeId) -> Result<Vec<Edge>> {
        debug!("Getting outgoing edges for node {}", node_id);
        
        let edge_ids = self.get_outgoing_edge_ids(node_id)?;
        let edges: Vec<Edge> = edge_ids
            .into_iter()
            .filter_map(|id| self.get_edge(id).ok())
            .collect();
        
        Ok(edges)
    }
    
    fn get_incoming_edges(&self, node_id: NodeId) -> Result<Vec<Edge>> {
        debug!("Getting incoming edges for node {}", node_id);
        
        let edge_ids = self.get_incoming_edge_ids(node_id)?;
        let edges: Vec<Edge> = edge_ids
            .into_iter()
            .filter_map(|id| self.get_edge(id).ok())
            .collect();
        
        Ok(edges)
    }
    
    fn node_count(&self) -> usize {
        self.nodes.len()
    }
    
    fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

// Additional helper methods specific to DiskStorage

impl DiskStorage {
    /// Get all nodes with a specific property value
    pub fn get_nodes_by_property(&self, key: &str, value: &PropertyValue) -> Vec<Node> {
        self.get_all_nodes()
            .into_iter()
            .filter(|node| node.get_property(key) == Some(value))
            .collect()
    }
    
    /// Get all edges of a specific type
    pub fn get_edges_by_type(&self, edge_type: &str) -> Vec<Edge> {
        match self.get_edges_for_type(edge_type) {
            Ok(ids) => {
                ids.into_iter()
                    .filter_map(|id| self.get_edge(id).ok())
                    .collect()
            }
            Err(e) => {
                warn!("Failed to get edges by type: {}", e);
                Vec::new()
            }
        }
    }
    
    /// Compact the database to reclaim space
    pub fn compact(&self) -> Result<()> {
        info!("Compacting disk storage");
        // Sled doesn't have explicit compaction, but flush helps
        self.flush()?;
        Ok(())
    }
    
    /// Export all data to a temporary snapshot (for backup)
    pub fn create_snapshot(&self) -> Result<Vec<u8>> {
        info!("Creating snapshot of disk storage");
        let mut snapshot = Vec::new();
        
        // Serialize node count
        let node_count = self.node_count();
        snapshot.extend_from_slice(&node_count.to_le_bytes());
        
        // Serialize all nodes
        for node in self.get_all_nodes() {
            let bytes = self.serialize_node(&node)?;
            snapshot.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
            snapshot.extend_from_slice(&bytes);
        }
        
        // Serialize edge count
        let edge_count = self.edge_count();
        snapshot.extend_from_slice(&edge_count.to_le_bytes());
        
        // Serialize all edges
        for result in self.edges.iter() {
            if let Ok((_key, value)) = result {
                snapshot.extend_from_slice(&(value.len() as u64).to_le_bytes());
                snapshot.extend_from_slice(&value);
            }
        }
        
        info!("Snapshot created: {} bytes", snapshot.len());
        Ok(snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn create_test_storage() -> (DiskStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = DiskStorage::new(temp_dir.path()).unwrap();
        (storage, temp_dir)
    }
    
    #[test]
    fn test_create_storage() {
        let (_storage, _temp_dir) = create_test_storage();
        // Storage created successfully
    }
    
    #[test]
    fn test_stats() {
        let (storage, _temp_dir) = create_test_storage();
        let stats = storage.stats();
        assert_eq!(stats.node_count, 0);
        assert_eq!(stats.edge_count, 0);
    }
    
    #[test]
    fn test_flush() {
        let (storage, _temp_dir) = create_test_storage();
        storage.flush().unwrap();
    }
    
    #[test]
    fn test_add_and_get_node() {
        let (storage, _temp_dir) = create_test_storage();
        
        let mut node = Node::new(vec!["Person".to_string()]);
        node.set_property("name".to_string(), PropertyValue::String("Alice".to_string()));
        node.set_property("age".to_string(), PropertyValue::Integer(30));
        
        let id = storage.add_node(node.clone()).unwrap();
        let retrieved = storage.get_node(id).unwrap();
        
        assert_eq!(retrieved.id(), id);
        assert_eq!(retrieved.labels(), node.labels());
        assert_eq!(retrieved.get_property("name"), node.get_property("name"));
        assert_eq!(retrieved.get_property("age"), node.get_property("age"));
    }
    
    #[test]
    fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();
        
        let node_id;
        
        // Create storage and add node
        {
            let storage = DiskStorage::new(&path).unwrap();
            let mut node = Node::new(vec!["Person".to_string()]);
            node.set_property("name".to_string(), PropertyValue::String("Bob".to_string()));
            node_id = storage.add_node(node).unwrap();
        } // Storage dropped, database closed
        
        // Reopen storage and verify node exists
        {
            let storage = DiskStorage::new(&path).unwrap();
            let node = storage.get_node(node_id).unwrap();
            assert_eq!(node.id(), node_id);
            assert_eq!(node.get_property("name"), Some(&PropertyValue::String("Bob".to_string())));
        }
    }
    
    #[test]
    fn test_label_index() {
        let (storage, _temp_dir) = create_test_storage();
        
        let mut node1 = Node::new(vec!["Person".to_string()]);
        node1.set_property("name".to_string(), PropertyValue::String("Alice".to_string()));
        storage.add_node(node1).unwrap();
        
        let mut node2 = Node::new(vec!["Person".to_string()]);
        node2.set_property("name".to_string(), PropertyValue::String("Bob".to_string()));
        storage.add_node(node2).unwrap();
        
        let mut node3 = Node::new(vec!["Company".to_string()]);
        node3.set_property("name".to_string(), PropertyValue::String("Acme".to_string()));
        storage.add_node(node3).unwrap();
        
        let people = storage.get_nodes_by_label("Person");
        assert_eq!(people.len(), 2);
        
        let companies = storage.get_nodes_by_label("Company");
        assert_eq!(companies.len(), 1);
    }
    
    #[test]
    fn test_edge_operations() {
        let (storage, _temp_dir) = create_test_storage();
        
        let node1 = Node::new(vec!["Person".to_string()]);
        let id1 = storage.add_node(node1).unwrap();
        
        let node2 = Node::new(vec!["Person".to_string()]);
        let id2 = storage.add_node(node2).unwrap();
        
        let mut edge = Edge::new(id1, id2, "KNOWS".to_string());
        edge.set_property("since".to_string(), PropertyValue::Integer(2020));
        
        let edge_id = storage.add_edge(edge.clone()).unwrap();
        let retrieved = storage.get_edge(edge_id).unwrap();
        
        assert_eq!(retrieved.id(), edge_id);
        assert_eq!(retrieved.from(), id1);
        assert_eq!(retrieved.to(), id2);
        assert_eq!(retrieved.relationship_type(), "KNOWS");
    }
    
    #[test]
    fn test_node_count_and_edge_count() {
        let (storage, _temp_dir) = create_test_storage();
        
        assert_eq!(storage.node_count(), 0);
        assert_eq!(storage.edge_count(), 0);
        
        let node1 = Node::new(vec!["Person".to_string()]);
        let id1 = storage.add_node(node1).unwrap();
        
        let node2 = Node::new(vec!["Person".to_string()]);
        let id2 = storage.add_node(node2).unwrap();
        
        assert_eq!(storage.node_count(), 2);
        
        let edge = Edge::new(id1, id2, "KNOWS".to_string());
        storage.add_edge(edge).unwrap();
        
        assert_eq!(storage.edge_count(), 1);
    }
}
