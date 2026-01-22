//! Storage module for DeepGraph
//!
//! This module provides multiple storage backends:
//! - In-memory HashMap storage (Phase 1)
//! - Columnar Arrow storage (Phase 2)
//! - Persistent Parquet storage (Phase 2)

pub mod memory;
pub mod columnar;
pub mod schema;

pub use memory::MemoryStorage;
pub use columnar::ColumnarStorage;

use crate::error::Result;
use crate::graph::{Edge, EdgeId, Node, NodeId};

/// Trait for storage backends
pub trait StorageBackend: Send + Sync {
    /// Add a node to storage
    fn add_node(&self, node: Node) -> Result<NodeId>;
    
    /// Get a node by ID
    fn get_node(&self, id: NodeId) -> Result<Node>;
    
    /// Update a node
    fn update_node(&self, node: Node) -> Result<()>;
    
    /// Delete a node
    fn delete_node(&self, id: NodeId) -> Result<()>;
    
    /// Add an edge to storage
    fn add_edge(&self, edge: Edge) -> Result<EdgeId>;
    
    /// Get an edge by ID
    fn get_edge(&self, id: EdgeId) -> Result<Edge>;
    
    /// Update an edge
    fn update_edge(&self, edge: Edge) -> Result<()>;
    
    /// Delete an edge
    fn delete_edge(&self, id: EdgeId) -> Result<()>;
    
    /// Get all nodes with a specific label
    fn get_nodes_by_label(&self, label: &str) -> Vec<Node>;
    
    /// Get all nodes (for full scan - MATCH (n))
    fn get_all_nodes(&self) -> Vec<Node>;
    
    /// Get outgoing edges from a node
    fn get_outgoing_edges(&self, node_id: NodeId) -> Result<Vec<Edge>>;
    
    /// Get incoming edges to a node
    fn get_incoming_edges(&self, node_id: NodeId) -> Result<Vec<Edge>>;
    
    /// Get node count
    fn node_count(&self) -> usize;
    
    /// Get edge count
    fn edge_count(&self) -> usize;
}

/// Re-export the default storage type for backward compatibility
pub type GraphStorage = MemoryStorage;

// Implement StorageBackend for MemoryStorage
impl StorageBackend for MemoryStorage {
    fn add_node(&self, node: Node) -> Result<NodeId> {
        MemoryStorage::add_node(self, node)
    }
    
    fn get_node(&self, id: NodeId) -> Result<Node> {
        MemoryStorage::get_node(self, id)
    }
    
    fn update_node(&self, node: Node) -> Result<()> {
        MemoryStorage::update_node(self, node)
    }
    
    fn delete_node(&self, id: NodeId) -> Result<()> {
        MemoryStorage::delete_node(self, id)
    }
    
    fn add_edge(&self, edge: Edge) -> Result<EdgeId> {
        MemoryStorage::add_edge(self, edge)
    }
    
    fn get_edge(&self, id: EdgeId) -> Result<Edge> {
        MemoryStorage::get_edge(self, id)
    }
    
    fn update_edge(&self, edge: Edge) -> Result<()> {
        MemoryStorage::update_edge(self, edge)
    }
    
    fn delete_edge(&self, id: EdgeId) -> Result<()> {
        MemoryStorage::delete_edge(self, id)
    }
    
    fn get_nodes_by_label(&self, label: &str) -> Vec<Node> {
        MemoryStorage::get_nodes_by_label(self, label)
    }
    
    fn get_all_nodes(&self) -> Vec<Node> {
        MemoryStorage::get_all_nodes(self)
    }
    
    fn get_outgoing_edges(&self, node_id: NodeId) -> Result<Vec<Edge>> {
        MemoryStorage::get_outgoing_edges(self, node_id)
    }
    
    fn get_incoming_edges(&self, node_id: NodeId) -> Result<Vec<Edge>> {
        MemoryStorage::get_incoming_edges(self, node_id)
    }
    
    fn node_count(&self) -> usize {
        MemoryStorage::node_count(self)
    }
    
    fn edge_count(&self) -> usize {
        MemoryStorage::edge_count(self)
    }
}

