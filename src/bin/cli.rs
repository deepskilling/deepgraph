//! Command-line interface for DeepGraph (basic placeholder)

use deepgraph::{GraphStorage, Node, Edge, PropertyValue};
use std::sync::Arc;

fn main() {
    env_logger::init();

    println!("DeepGraph - High-Performance Graph Database");
    println!("============================================\n");

    // Create a new graph storage
    let storage = Arc::new(GraphStorage::new());

    // Demo: Create some nodes
    println!("Creating nodes...");
    let mut alice = Node::new(vec!["Person".to_string()]);
    alice.set_property("name".to_string(), "Alice".into());
    alice.set_property("age".to_string(), 30i64.into());

    let mut bob = Node::new(vec!["Person".to_string()]);
    bob.set_property("name".to_string(), "Bob".into());
    bob.set_property("age".to_string(), 35i64.into());

    let mut company = Node::new(vec!["Organization".to_string()]);
    company.set_property("name".to_string(), "TechCorp".into());

    let alice_id = storage.add_node(alice).expect("Failed to add Alice");
    let bob_id = storage.add_node(bob).expect("Failed to add Bob");
    let company_id = storage.add_node(company).expect("Failed to add company");

    println!("✓ Created 3 nodes");

    // Demo: Create edges
    println!("\nCreating relationships...");
    let mut knows_edge = Edge::new(alice_id, bob_id, "KNOWS".to_string());
    knows_edge.set_property("since".to_string(), 2015i64.into());

    let works_at_edge1 = Edge::new(alice_id, company_id, "WORKS_AT".to_string());
    let works_at_edge2 = Edge::new(bob_id, company_id, "WORKS_AT".to_string());

    storage.add_edge(knows_edge).expect("Failed to add KNOWS edge");
    storage.add_edge(works_at_edge1).expect("Failed to add WORKS_AT edge");
    storage.add_edge(works_at_edge2).expect("Failed to add WORKS_AT edge");

    println!("✓ Created 3 relationships");

    // Demo: Query the graph
    println!("\n=== Graph Statistics ===");
    println!("Total nodes: {}", storage.node_count());
    println!("Total edges: {}", storage.edge_count());

    println!("\n=== Nodes by Label ===");
    let people = storage.get_nodes_by_label("Person");
    println!("People: {}", people.len());
    for person in people {
        let name = person.get_property("name")
            .and_then(|v| v.as_string())
            .unwrap_or("Unknown");
        let age = person.get_property("age")
            .and_then(|v| v.as_integer())
            .unwrap_or(0);
        println!("  - {} (age: {})", name, age);
    }

    let orgs = storage.get_nodes_by_label("Organization");
    println!("Organizations: {}", orgs.len());
    for org in orgs {
        let name = org.get_property("name")
            .and_then(|v| v.as_string())
            .unwrap_or("Unknown");
        println!("  - {}", name);
    }

    println!("\n=== Relationships ===");
    let alice_outgoing = storage.get_outgoing_edges(alice_id)
        .expect("Failed to get Alice's edges");
    println!("Alice's relationships: {}", alice_outgoing.len());
    for edge in alice_outgoing {
        println!("  - {} to {}", edge.relationship_type(), edge.to());
    }

    println!("\n=== Query by Property ===");
    let age_30 = storage.get_nodes_by_property("age", &PropertyValue::Integer(30));
    println!("People aged 30: {}", age_30.len());

    println!("\nPhase 1 Demo Complete! ✓");
    println!("\nNote: This is a Phase 1 implementation with basic features.");
    println!("Full Cypher query support and advanced features coming in later phases.");
}

