//! Integration tests for DeepGraph

use deepgraph::{GraphStorage, Node, Edge, PropertyValue, Transaction};
use std::sync::Arc;

#[test]
fn test_basic_graph_operations() {
    let storage = GraphStorage::new();
    
    // Create nodes
    let mut alice = Node::new(vec!["Person".to_string()]);
    alice.set_property("name".to_string(), "Alice".into());
    alice.set_property("age".to_string(), 30i64.into());
    
    let mut bob = Node::new(vec!["Person".to_string()]);
    bob.set_property("name".to_string(), "Bob".into());
    bob.set_property("age".to_string(), 35i64.into());
    
    let alice_id = storage.add_node(alice).unwrap();
    let bob_id = storage.add_node(bob).unwrap();
    
    // Create edge
    let mut edge = Edge::new(alice_id, bob_id, "KNOWS".to_string());
    edge.set_property("since".to_string(), 2015i64.into());
    let edge_id = storage.add_edge(edge).unwrap();
    
    // Verify
    assert_eq!(storage.node_count(), 2);
    assert_eq!(storage.edge_count(), 1);
    
    let retrieved_edge = storage.get_edge(edge_id).unwrap();
    assert_eq!(retrieved_edge.relationship_type(), "KNOWS");
    assert_eq!(
        retrieved_edge.get_property("since").unwrap().as_integer(),
        Some(2015)
    );
}

#[test]
fn test_complex_graph_structure() {
    let storage = GraphStorage::new();
    
    // Create a social network
    let mut people = Vec::new();
    for i in 0..10 {
        let mut person = Node::new(vec!["Person".to_string()]);
        person.set_property("id".to_string(), (i as i64).into());
        person.set_property("name".to_string(), format!("Person{}", i).into());
        let id = storage.add_node(person).unwrap();
        people.push(id);
    }
    
    // Create friendships (each person knows the next two)
    for i in 0..8 {
        storage.add_edge(Edge::new(
            people[i],
            people[i + 1],
            "KNOWS".to_string()
        )).unwrap();
        storage.add_edge(Edge::new(
            people[i],
            people[i + 2],
            "KNOWS".to_string()
        )).unwrap();
    }
    
    assert_eq!(storage.node_count(), 10);
    assert_eq!(storage.edge_count(), 16);
    
    // Test outgoing edges
    let outgoing = storage.get_outgoing_edges(people[0]).unwrap();
    assert_eq!(outgoing.len(), 2);
    
    // Test incoming edges
    let incoming = storage.get_incoming_edges(people[2]).unwrap();
    assert!(incoming.len() >= 2);
}

#[test]
fn test_multi_label_nodes() {
    let storage = GraphStorage::new();
    
    let mut node = Node::new(vec!["Person".to_string(), "Employee".to_string()]);
    node.set_property("name".to_string(), "Alice".into());
    storage.add_node(node).unwrap();
    
    let people = storage.get_nodes_by_label("Person");
    assert_eq!(people.len(), 1);
    
    let employees = storage.get_nodes_by_label("Employee");
    assert_eq!(employees.len(), 1);
}

#[test]
fn test_property_types() {
    let storage = GraphStorage::new();
    
    let mut node = Node::new(vec!["Test".to_string()]);
    node.set_property("string".to_string(), "value".into());
    node.set_property("integer".to_string(), 42i64.into());
    node.set_property("float".to_string(), 3.14f64.into());
    node.set_property("boolean".to_string(), true.into());
    node.set_property("null".to_string(), PropertyValue::Null);
    
    let id = storage.add_node(node).unwrap();
    let retrieved = storage.get_node(id).unwrap();
    
    assert_eq!(retrieved.get_property("string").unwrap().as_string(), Some("value"));
    assert_eq!(retrieved.get_property("integer").unwrap().as_integer(), Some(42));
    assert_eq!(retrieved.get_property("float").unwrap().as_float(), Some(3.14));
    assert_eq!(retrieved.get_property("boolean").unwrap().as_boolean(), Some(true));
    assert!(retrieved.get_property("null").unwrap().is_null());
}

#[test]
fn test_node_deletion_cascades() {
    let storage = GraphStorage::new();
    
    let node1 = Node::new(vec!["Node".to_string()]);
    let node2 = Node::new(vec!["Node".to_string()]);
    let node3 = Node::new(vec!["Node".to_string()]);
    
    let id1 = storage.add_node(node1).unwrap();
    let id2 = storage.add_node(node2).unwrap();
    let id3 = storage.add_node(node3).unwrap();
    
    storage.add_edge(Edge::new(id1, id2, "RELATES".to_string())).unwrap();
    storage.add_edge(Edge::new(id1, id3, "RELATES".to_string())).unwrap();
    storage.add_edge(Edge::new(id2, id3, "RELATES".to_string())).unwrap();
    
    assert_eq!(storage.edge_count(), 3);
    
    // Delete node1 should remove its edges
    storage.delete_node(id1).unwrap();
    
    assert_eq!(storage.node_count(), 2);
    assert_eq!(storage.edge_count(), 1); // Only id2->id3 edge remains
}

#[test]
fn test_update_operations() {
    let storage = GraphStorage::new();
    
    let mut node = Node::new(vec!["Person".to_string()]);
    node.set_property("name".to_string(), "Alice".into());
    node.set_property("age".to_string(), 30i64.into());
    
    let id = storage.add_node(node).unwrap();
    
    // Update the node
    let mut updated = storage.get_node(id).unwrap();
    updated.set_property("age".to_string(), 31i64.into());
    updated.set_property("city".to_string(), "New York".into());
    
    storage.update_node(updated).unwrap();
    
    // Verify update
    let retrieved = storage.get_node(id).unwrap();
    assert_eq!(retrieved.get_property("age").unwrap().as_integer(), Some(31));
    assert_eq!(retrieved.get_property("city").unwrap().as_string(), Some("New York"));
}

#[test]
fn test_transaction_operations() {
    let storage = Arc::new(GraphStorage::new());
    let mut tx = Transaction::begin(storage);
    
    let mut node = Node::new(vec!["Person".to_string()]);
    node.set_property("name".to_string(), "Alice".into());
    
    let id = tx.add_node(node).unwrap();
    let retrieved = tx.get_node(id).unwrap();
    
    assert_eq!(retrieved.get_property("name").unwrap().as_string(), Some("Alice"));
    
    tx.commit().unwrap();
}

#[test]
fn test_large_graph_performance() {
    let storage = GraphStorage::new();
    
    // Create 1000 nodes
    let mut node_ids = Vec::new();
    for i in 0..1000 {
        let mut node = Node::new(vec!["Node".to_string()]);
        node.set_property("id".to_string(), (i as i64).into());
        let id = storage.add_node(node).unwrap();
        node_ids.push(id);
    }
    
    assert_eq!(storage.node_count(), 1000);
    
    // Create 2000 random edges
    for i in 0..2000 {
        let from = node_ids[i % 1000];
        let to = node_ids[(i * 7) % 1000];
        if from != to {
            storage.add_edge(Edge::new(from, to, "RELATES".to_string())).ok();
        }
    }
    
    // Query performance
    let nodes = storage.get_nodes_by_label("Node");
    assert_eq!(nodes.len(), 1000);
}

#[test]
fn test_bidirectional_edges() {
    let storage = GraphStorage::new();
    
    let node1 = Node::new(vec!["Node".to_string()]);
    let node2 = Node::new(vec!["Node".to_string()]);
    
    let id1 = storage.add_node(node1).unwrap();
    let id2 = storage.add_node(node2).unwrap();
    
    storage.add_edge(Edge::new(id1, id2, "KNOWS".to_string())).unwrap();
    storage.add_edge(Edge::new(id2, id1, "KNOWS".to_string())).unwrap();
    
    let outgoing1 = storage.get_outgoing_edges(id1).unwrap();
    let incoming1 = storage.get_incoming_edges(id1).unwrap();
    
    assert_eq!(outgoing1.len(), 1);
    assert_eq!(incoming1.len(), 1);
}

#[test]
fn test_self_referential_edge() {
    let storage = GraphStorage::new();
    
    let node = Node::new(vec!["Node".to_string()]);
    let id = storage.add_node(node).unwrap();
    
    storage.add_edge(Edge::new(id, id, "SELF".to_string())).unwrap();
    
    let outgoing = storage.get_outgoing_edges(id).unwrap();
    let incoming = storage.get_incoming_edges(id).unwrap();
    
    assert_eq!(outgoing.len(), 1);
    assert_eq!(incoming.len(), 1);
    assert_eq!(outgoing[0].from(), outgoing[0].to());
}

#[test]
fn test_property_list_and_map() {
    let storage = GraphStorage::new();
    
    let mut node = Node::new(vec!["Test".to_string()]);
    
    // List property
    let list = PropertyValue::List(vec![
        PropertyValue::Integer(1),
        PropertyValue::Integer(2),
        PropertyValue::Integer(3),
    ]);
    node.set_property("numbers".to_string(), list);
    
    // Map property
    let mut map = std::collections::HashMap::new();
    map.insert("city".to_string(), PropertyValue::String("NYC".to_string()));
    map.insert("zip".to_string(), PropertyValue::Integer(10001));
    node.set_property("address".to_string(), PropertyValue::Map(map));
    
    let id = storage.add_node(node).unwrap();
    let retrieved = storage.get_node(id).unwrap();
    
    assert!(matches!(
        retrieved.get_property("numbers"),
        Some(PropertyValue::List(_))
    ));
    assert!(matches!(
        retrieved.get_property("address"),
        Some(PropertyValue::Map(_))
    ));
}

