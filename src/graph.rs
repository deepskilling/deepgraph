//! Core graph data structures
//!
//! This module provides the fundamental building blocks for the graph database:
//! - `Node`: Represents a vertex in the graph with properties and labels
//! - `Edge`: Represents a relationship between two nodes with properties
//! - `Property`: Key-value pairs attached to nodes and edges

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeId(Uuid);

impl NodeId {
    /// Create a new unique node ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a node ID from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for edges
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(Uuid);

impl EdgeId {
    /// Create a new unique edge ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create an edge ID from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for EdgeId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for EdgeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Supported property value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PropertyValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
    List(Vec<PropertyValue>),
    Map(HashMap<String, PropertyValue>),
}

impl PropertyValue {
    /// Check if the value is null
    pub fn is_null(&self) -> bool {
        matches!(self, PropertyValue::Null)
    }

    /// Try to get as string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            PropertyValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get as integer
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            PropertyValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Try to get as float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            PropertyValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Try to get as boolean
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            PropertyValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

impl From<String> for PropertyValue {
    fn from(s: String) -> Self {
        PropertyValue::String(s)
    }
}

impl From<&str> for PropertyValue {
    fn from(s: &str) -> Self {
        PropertyValue::String(s.to_string())
    }
}

impl From<i64> for PropertyValue {
    fn from(i: i64) -> Self {
        PropertyValue::Integer(i)
    }
}

impl From<f64> for PropertyValue {
    fn from(f: f64) -> Self {
        PropertyValue::Float(f)
    }
}

impl From<bool> for PropertyValue {
    fn from(b: bool) -> Self {
        PropertyValue::Boolean(b)
    }
}

/// A key-value property
pub type Property = (String, PropertyValue);

/// A node (vertex) in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier
    id: NodeId,
    /// Labels categorizing the node (e.g., "Person", "Organization")
    labels: Vec<String>,
    /// Key-value properties
    properties: HashMap<String, PropertyValue>,
}

impl Node {
    /// Create a new node with the given labels
    pub fn new(labels: Vec<String>) -> Self {
        Self {
            id: NodeId::new(),
            labels,
            properties: HashMap::new(),
        }
    }

    /// Create a new node with a specific ID (useful for deserialization)
    pub fn with_id(id: NodeId, labels: Vec<String>) -> Self {
        Self {
            id,
            labels,
            properties: HashMap::new(),
        }
    }

    /// Get the node's ID
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// Get the node's labels
    pub fn labels(&self) -> &[String] {
        &self.labels
    }

    /// Add a label to the node
    pub fn add_label(&mut self, label: String) {
        if !self.labels.contains(&label) {
            self.labels.push(label);
        }
    }

    /// Remove a label from the node
    pub fn remove_label(&mut self, label: &str) -> bool {
        if let Some(pos) = self.labels.iter().position(|l| l == label) {
            self.labels.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if the node has a specific label
    pub fn has_label(&self, label: &str) -> bool {
        self.labels.iter().any(|l| l == label)
    }

    /// Get all properties
    pub fn properties(&self) -> &HashMap<String, PropertyValue> {
        &self.properties
    }

    /// Get a mutable reference to all properties
    pub fn properties_mut(&mut self) -> &mut HashMap<String, PropertyValue> {
        &mut self.properties
    }

    /// Get a property value by key
    pub fn get_property(&self, key: &str) -> Option<&PropertyValue> {
        self.properties.get(key)
    }

    /// Set a property
    pub fn set_property(&mut self, key: String, value: PropertyValue) {
        self.properties.insert(key, value);
    }

    /// Remove a property
    pub fn remove_property(&mut self, key: &str) -> Option<PropertyValue> {
        self.properties.remove(key)
    }

    /// Check if the node has a specific property
    pub fn has_property(&self, key: &str) -> bool {
        self.properties.contains_key(key)
    }
}

/// An edge (relationship) in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Unique identifier
    id: EdgeId,
    /// Source node ID
    from: NodeId,
    /// Target node ID
    to: NodeId,
    /// Relationship type (e.g., "KNOWS", "WORKS_AT")
    relationship_type: String,
    /// Key-value properties
    properties: HashMap<String, PropertyValue>,
}

impl Edge {
    /// Create a new edge
    pub fn new(from: NodeId, to: NodeId, relationship_type: String) -> Self {
        Self {
            id: EdgeId::new(),
            from,
            to,
            relationship_type,
            properties: HashMap::new(),
        }
    }

    /// Create a new edge with a specific ID (useful for deserialization)
    pub fn with_id(id: EdgeId, from: NodeId, to: NodeId, relationship_type: String) -> Self {
        Self {
            id,
            from,
            to,
            relationship_type,
            properties: HashMap::new(),
        }
    }

    /// Get the edge's ID
    pub fn id(&self) -> EdgeId {
        self.id
    }

    /// Get the source node ID
    pub fn from(&self) -> NodeId {
        self.from
    }

    /// Get the target node ID
    pub fn to(&self) -> NodeId {
        self.to
    }

    /// Get the relationship type
    pub fn relationship_type(&self) -> &str {
        &self.relationship_type
    }

    /// Get all properties
    pub fn properties(&self) -> &HashMap<String, PropertyValue> {
        &self.properties
    }

    /// Get a mutable reference to all properties
    pub fn properties_mut(&mut self) -> &mut HashMap<String, PropertyValue> {
        &mut self.properties
    }

    /// Get a property value by key
    pub fn get_property(&self, key: &str) -> Option<&PropertyValue> {
        self.properties.get(key)
    }

    /// Set a property
    pub fn set_property(&mut self, key: String, value: PropertyValue) {
        self.properties.insert(key, value);
    }

    /// Remove a property
    pub fn remove_property(&mut self, key: &str) -> Option<PropertyValue> {
        self.properties.remove(key)
    }

    /// Check if the edge has a specific property
    pub fn has_property(&self, key: &str) -> bool {
        self.properties.contains_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let mut node = Node::new(vec!["Person".to_string()]);
        assert_eq!(node.labels().len(), 1);
        assert!(node.has_label("Person"));

        node.set_property("name".to_string(), "Alice".into());
        node.set_property("age".to_string(), 30i64.into());

        assert_eq!(node.get_property("name").unwrap().as_string(), Some("Alice"));
        assert_eq!(node.get_property("age").unwrap().as_integer(), Some(30));
    }

    #[test]
    fn test_edge_creation() {
        let node1 = Node::new(vec!["Person".to_string()]);
        let node2 = Node::new(vec!["Person".to_string()]);

        let mut edge = Edge::new(node1.id(), node2.id(), "KNOWS".to_string());
        edge.set_property("since".to_string(), 2020i64.into());

        assert_eq!(edge.relationship_type(), "KNOWS");
        assert_eq!(edge.get_property("since").unwrap().as_integer(), Some(2020));
    }

    #[test]
    fn test_property_values() {
        let string_val = PropertyValue::from("test");
        assert_eq!(string_val.as_string(), Some("test"));

        let int_val = PropertyValue::from(42i64);
        assert_eq!(int_val.as_integer(), Some(42));

        let float_val = PropertyValue::from(3.14f64);
        assert_eq!(float_val.as_float(), Some(3.14));

        let bool_val = PropertyValue::from(true);
        assert_eq!(bool_val.as_boolean(), Some(true));

        let null_val = PropertyValue::Null;
        assert!(null_val.is_null());
    }

    #[test]
    fn test_node_labels() {
        let mut node = Node::new(vec!["Person".to_string()]);
        
        node.add_label("Employee".to_string());
        assert_eq!(node.labels().len(), 2);
        assert!(node.has_label("Person"));
        assert!(node.has_label("Employee"));

        node.remove_label("Person");
        assert_eq!(node.labels().len(), 1);
        assert!(!node.has_label("Person"));
        assert!(node.has_label("Employee"));
    }
}

