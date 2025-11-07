//! In-memory storage engine for the graph database
//!
//! This module provides a simple in-memory storage implementation using hash maps
//! for quick lookups. It supports basic CRUD operations for nodes and edges.

use crate::error::{DeepGraphError, Result};
use crate::graph::{Edge, EdgeId, Node, NodeId, PropertyValue};
use dashmap::DashMap;
use std::sync::Arc;

/// In-memory graph storage engine
///
/// Uses concurrent hash maps (DashMap) for thread-safe operations.
/// Maintains indices for efficient lookups:
/// - Nodes by ID
/// - Edges by ID
/// - Outgoing edges by source node
/// - Incoming edges by target node
#[derive(Debug, Clone)]
pub struct MemoryStorage {
    /// Store nodes by ID
    nodes: Arc<DashMap<NodeId, Node>>,
    /// Store edges by ID
    edges: Arc<DashMap<EdgeId, Edge>>,
    /// Index: source node -> outgoing edges
    outgoing_edges: Arc<DashMap<NodeId, Vec<EdgeId>>>,
    /// Index: target node -> incoming edges
    incoming_edges: Arc<DashMap<NodeId, Vec<EdgeId>>>,
}

impl MemoryStorage {
    /// Create a new empty graph storage
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(DashMap::new()),
            edges: Arc::new(DashMap::new()),
            outgoing_edges: Arc::new(DashMap::new()),
            incoming_edges: Arc::new(DashMap::new()),
        }
    }

    /// Get the number of nodes in the graph
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of edges in the graph
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Add a node to the storage
    pub fn add_node(&self, node: Node) -> Result<NodeId> {
        let id = node.id();
        self.nodes.insert(id, node);
        Ok(id)
    }

    /// Get a node by ID
    pub fn get_node(&self, id: NodeId) -> Result<Node> {
        self.nodes
            .get(&id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| DeepGraphError::NodeNotFound(id.to_string()))
    }

    /// Update a node
    pub fn update_node(&self, node: Node) -> Result<()> {
        let id = node.id();
        if self.nodes.contains_key(&id) {
            self.nodes.insert(id, node);
            Ok(())
        } else {
            Err(DeepGraphError::NodeNotFound(id.to_string()))
        }
    }

    /// Delete a node and all connected edges
    pub fn delete_node(&self, id: NodeId) -> Result<()> {
        // Remove the node
        self.nodes
            .remove(&id)
            .ok_or_else(|| DeepGraphError::NodeNotFound(id.to_string()))?;

        // Remove all outgoing edges
        if let Some((_, edge_ids)) = self.outgoing_edges.remove(&id) {
            for edge_id in edge_ids {
                self.edges.remove(&edge_id);
            }
        }

        // Remove all incoming edges
        if let Some((_, edge_ids)) = self.incoming_edges.remove(&id) {
            for edge_id in edge_ids {
                self.edges.remove(&edge_id);
            }
        }

        Ok(())
    }

    /// Get all nodes with a specific label
    pub fn get_nodes_by_label(&self, label: &str) -> Vec<Node> {
        self.nodes
            .iter()
            .filter(|entry| entry.value().has_label(label))
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get all nodes with a specific property
    pub fn get_nodes_by_property(&self, key: &str, value: &PropertyValue) -> Vec<Node> {
        self.nodes
            .iter()
            .filter(|entry| {
                entry
                    .value()
                    .get_property(key)
                    .map(|v| v == value)
                    .unwrap_or(false)
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Add an edge to the storage
    pub fn add_edge(&self, edge: Edge) -> Result<EdgeId> {
        let id = edge.id();
        let from = edge.from();
        let to = edge.to();

        // Verify that both nodes exist
        if !self.nodes.contains_key(&from) {
            return Err(DeepGraphError::NodeNotFound(from.to_string()));
        }
        if !self.nodes.contains_key(&to) {
            return Err(DeepGraphError::NodeNotFound(to.to_string()));
        }

        // Add edge to storage
        self.edges.insert(id, edge);

        // Update outgoing edges index
        self.outgoing_edges
            .entry(from)
            .or_insert_with(Vec::new)
            .push(id);

        // Update incoming edges index
        self.incoming_edges
            .entry(to)
            .or_insert_with(Vec::new)
            .push(id);

        Ok(id)
    }

    /// Get an edge by ID
    pub fn get_edge(&self, id: EdgeId) -> Result<Edge> {
        self.edges
            .get(&id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| DeepGraphError::EdgeNotFound(id.to_string()))
    }

    /// Update an edge
    pub fn update_edge(&self, edge: Edge) -> Result<()> {
        let id = edge.id();
        if self.edges.contains_key(&id) {
            self.edges.insert(id, edge);
            Ok(())
        } else {
            Err(DeepGraphError::EdgeNotFound(id.to_string()))
        }
    }

    /// Delete an edge
    pub fn delete_edge(&self, id: EdgeId) -> Result<()> {
        let edge = self
            .edges
            .remove(&id)
            .ok_or_else(|| DeepGraphError::EdgeNotFound(id.to_string()))?;

        let from = edge.1.from();
        let to = edge.1.to();

        // Remove from outgoing edges index
        if let Some(mut edges) = self.outgoing_edges.get_mut(&from) {
            edges.retain(|&eid| eid != id);
        }

        // Remove from incoming edges index
        if let Some(mut edges) = self.incoming_edges.get_mut(&to) {
            edges.retain(|&eid| eid != id);
        }

        Ok(())
    }

    /// Get all outgoing edges from a node
    pub fn get_outgoing_edges(&self, node_id: NodeId) -> Result<Vec<Edge>> {
        if !self.nodes.contains_key(&node_id) {
            return Err(DeepGraphError::NodeNotFound(node_id.to_string()));
        }

        let edge_ids = self
            .outgoing_edges
            .get(&node_id)
            .map(|entry| entry.value().clone())
            .unwrap_or_default();

        Ok(edge_ids
            .into_iter()
            .filter_map(|id| self.edges.get(&id).map(|e| e.value().clone()))
            .collect())
    }

    /// Get all incoming edges to a node
    pub fn get_incoming_edges(&self, node_id: NodeId) -> Result<Vec<Edge>> {
        if !self.nodes.contains_key(&node_id) {
            return Err(DeepGraphError::NodeNotFound(node_id.to_string()));
        }

        let edge_ids = self
            .incoming_edges
            .get(&node_id)
            .map(|entry| entry.value().clone())
            .unwrap_or_default();

        Ok(edge_ids
            .into_iter()
            .filter_map(|id| self.edges.get(&id).map(|e| e.value().clone()))
            .collect())
    }

    /// Get all edges of a specific type
    pub fn get_edges_by_type(&self, relationship_type: &str) -> Vec<Edge> {
        self.edges
            .iter()
            .filter(|entry| entry.value().relationship_type() == relationship_type)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get all nodes in the graph
    pub fn get_all_nodes(&self) -> Vec<Node> {
        self.nodes
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get all edges in the graph
    pub fn get_all_edges(&self) -> Vec<Edge> {
        self.edges
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Clear all data from storage
    pub fn clear(&self) {
        self.nodes.clear();
        self.edges.clear();
        self.outgoing_edges.clear();
        self.incoming_edges.clear();
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::PropertyValue;

    #[test]
    fn test_add_and_get_node() {
        let storage = MemoryStorage::new();
        let mut node = Node::new(vec!["Person".to_string()]);
        node.set_property("name".to_string(), "Alice".into());

        let id = storage.add_node(node.clone()).unwrap();
        let retrieved = storage.get_node(id).unwrap();

        assert_eq!(retrieved.id(), id);
        assert_eq!(retrieved.get_property("name").unwrap().as_string(), Some("Alice"));
    }

    #[test]
    fn test_add_and_get_edge() {
        let storage = MemoryStorage::new();

        let node1 = Node::new(vec!["Person".to_string()]);
        let node2 = Node::new(vec!["Person".to_string()]);

        let id1 = storage.add_node(node1.clone()).unwrap();
        let id2 = storage.add_node(node2.clone()).unwrap();

        let mut edge = Edge::new(id1, id2, "KNOWS".to_string());
        edge.set_property("since".to_string(), 2020i64.into());

        let edge_id = storage.add_edge(edge.clone()).unwrap();
        let retrieved = storage.get_edge(edge_id).unwrap();

        assert_eq!(retrieved.from(), id1);
        assert_eq!(retrieved.to(), id2);
        assert_eq!(retrieved.relationship_type(), "KNOWS");
    }

    #[test]
    fn test_delete_node_cascades_edges() {
        let storage = MemoryStorage::new();

        let node1 = Node::new(vec!["Person".to_string()]);
        let node2 = Node::new(vec!["Person".to_string()]);

        let id1 = storage.add_node(node1).unwrap();
        let id2 = storage.add_node(node2).unwrap();

        let edge = Edge::new(id1, id2, "KNOWS".to_string());
        let _edge_id = storage.add_edge(edge).unwrap();

        assert_eq!(storage.node_count(), 2);
        assert_eq!(storage.edge_count(), 1);

        storage.delete_node(id1).unwrap();

        assert_eq!(storage.node_count(), 1);
        assert_eq!(storage.edge_count(), 0);
    }

    #[test]
    fn test_get_outgoing_and_incoming_edges() {
        let storage = MemoryStorage::new();

        let node1 = Node::new(vec!["Person".to_string()]);
        let node2 = Node::new(vec!["Person".to_string()]);
        let node3 = Node::new(vec!["Person".to_string()]);

        let id1 = storage.add_node(node1).unwrap();
        let id2 = storage.add_node(node2).unwrap();
        let id3 = storage.add_node(node3).unwrap();

        storage.add_edge(Edge::new(id1, id2, "KNOWS".to_string())).unwrap();
        storage.add_edge(Edge::new(id1, id3, "KNOWS".to_string())).unwrap();
        storage.add_edge(Edge::new(id2, id1, "KNOWS".to_string())).unwrap();

        let outgoing = storage.get_outgoing_edges(id1).unwrap();
        assert_eq!(outgoing.len(), 2);

        let incoming = storage.get_incoming_edges(id1).unwrap();
        assert_eq!(incoming.len(), 1);
    }

    #[test]
    fn test_query_by_label() {
        let storage = MemoryStorage::new();

        let node1 = Node::new(vec!["Person".to_string()]);
        let node2 = Node::new(vec!["Organization".to_string()]);
        let node3 = Node::new(vec!["Person".to_string()]);

        storage.add_node(node1).unwrap();
        storage.add_node(node2).unwrap();
        storage.add_node(node3).unwrap();

        let people = storage.get_nodes_by_label("Person");
        assert_eq!(people.len(), 2);

        let orgs = storage.get_nodes_by_label("Organization");
        assert_eq!(orgs.len(), 1);
    }

    #[test]
    fn test_query_by_property() {
        let storage = MemoryStorage::new();

        let mut node1 = Node::new(vec!["Person".to_string()]);
        node1.set_property("age".to_string(), 30i64.into());

        let mut node2 = Node::new(vec!["Person".to_string()]);
        node2.set_property("age".to_string(), 25i64.into());

        let mut node3 = Node::new(vec!["Person".to_string()]);
        node3.set_property("age".to_string(), 30i64.into());

        storage.add_node(node1).unwrap();
        storage.add_node(node2).unwrap();
        storage.add_node(node3).unwrap();

        let age_30 = storage.get_nodes_by_property("age", &PropertyValue::Integer(30));
        assert_eq!(age_30.len(), 2);
    }
}

