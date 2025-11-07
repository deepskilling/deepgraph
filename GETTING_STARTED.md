# Getting Started with DeepGraph

DeepGraph is a high-performance graph database written in Rust, designed for modern graph analytics workloads.

## Installation

### Prerequisites
- Rust 1.70 or higher
- Cargo (comes with Rust)

### Building from Source

```bash
# Clone the repository (or navigate to the project directory)
cd deepgraph

# Build the project
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

## Quick Start

### Running the Demo

The quickest way to see DeepGraph in action is to run the CLI demo:

```bash
cargo run --bin deepgraph-cli
```

This will create a sample graph with people and organizations, demonstrating basic operations.

### Using DeepGraph as a Library

Add DeepGraph to your `Cargo.toml`:

```toml
[dependencies]
deepgraph = { path = "../path/to/deepgraph" }
```

## Basic Usage

### Creating a Graph

```rust
use deepgraph::{GraphStorage, Node, Edge};
use std::sync::Arc;

// Create a new graph storage
let storage = Arc::new(GraphStorage::new());

// Create nodes
let mut alice = Node::new(vec!["Person".to_string()]);
alice.set_property("name".to_string(), "Alice".into());
alice.set_property("age".to_string(), 30i64.into());

let mut bob = Node::new(vec!["Person".to_string()]);
bob.set_property("name".to_string(), "Bob".into());
bob.set_property("age".to_string(), 35i64.into());

// Add nodes to storage
let alice_id = storage.add_node(alice).unwrap();
let bob_id = storage.add_node(bob).unwrap();

// Create a relationship
let mut edge = Edge::new(alice_id, bob_id, "KNOWS".to_string());
edge.set_property("since".to_string(), 2015i64.into());

// Add edge to storage
storage.add_edge(edge).unwrap();
```

### Querying the Graph

```rust
// Query by label
let people = storage.get_nodes_by_label("Person");
println!("Found {} people", people.len());

// Query by property
use deepgraph::PropertyValue;
let age_30 = storage.get_nodes_by_property("age", &PropertyValue::Integer(30));

// Get relationships
let alice_edges = storage.get_outgoing_edges(alice_id).unwrap();
println!("Alice has {} relationships", alice_edges.len());
```

### Using Transactions

```rust
use deepgraph::Transaction;

// Begin a transaction
let mut tx = Transaction::begin(storage.clone());

// Perform operations
let node = Node::new(vec!["Person".to_string()]);
let id = tx.add_node(node).unwrap();

// Commit the transaction
tx.commit().unwrap();
```

## Core Concepts

### Nodes
Nodes represent entities in your graph. Each node can have:
- One or more labels (e.g., "Person", "Organization")
- Properties (key-value pairs)
- A unique ID

### Edges
Edges represent relationships between nodes. Each edge has:
- A source node (from)
- A target node (to)
- A relationship type (e.g., "KNOWS", "WORKS_AT")
- Properties (key-value pairs)
- A unique ID

### Properties
Properties are key-value pairs that can be attached to both nodes and edges. Supported types:
- String
- Integer (i64)
- Float (f64)
- Boolean
- Null
- List (array of property values)
- Map (nested key-value pairs)

### Storage
The storage engine manages the graph data in memory. It provides:
- Fast lookups by ID
- Efficient traversal (outgoing/incoming edges)
- Query capabilities (by label, by property)
- Automatic cascade deletion

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test storage

# Run integration tests only
cargo test --test integration_tests

# Run with output
cargo test -- --nocapture
```

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench node_creation

# Quick test mode (doesn't take long)
cargo bench -- --test
```

## Project Structure

```
deepgraph/
├── src/
│   ├── lib.rs              # Library entry point
│   ├── graph.rs            # Core graph data structures
│   ├── storage.rs          # In-memory storage engine
│   ├── parser.rs           # Cypher parser (placeholder)
│   ├── transaction.rs      # Transaction management
│   ├── error.rs            # Error types
│   └── bin/
│       └── cli.rs          # CLI demo application
├── tests/
│   └── integration_tests.rs # Integration tests
├── benches/
│   └── graph_ops.rs        # Performance benchmarks
├── Cargo.toml              # Project configuration
└── README.md               # Project overview
```

## Performance

Phase 1 focuses on correctness and API design. Performance optimizations will come in later phases with:
- Columnar storage with Apache Arrow
- Proper indexing (B-tree, hash indices)
- Query planning and optimization
- Parallelization

Current benchmarks (on a typical development machine):
- Node creation: ~100ns per node
- Node insertion: ~500ns per node
- Node lookup: ~50ns
- Edge creation: ~1µs per edge
- Query by label (1000 nodes): ~100µs
- Graph traversal: O(n) with efficient indexing

## What's Next?

Phase 1 provides the foundation. Here's what's coming:

### Phase 2: Core Features
- Columnar storage with Arrow
- Query planning and optimization
- Proper indexing
- Full ACID transactions
- Enhanced CLI interface

### Phase 3: Advanced Features
- Full-text search
- Vector indices for similarity search
- Graph algorithms library
- Language bindings (Python, Node.js)

### Phase 4: Production Ready
- WebAssembly support
- Extension system
- Comprehensive documentation
- Production deployment guides

## Getting Help

For questions, issues, or contributions:
1. Check the documentation in the source code
2. Review existing tests for usage examples
3. Run the demo application to see features in action

## License

DeepGraph is licensed under the MIT License. See LICENSE file for details.

