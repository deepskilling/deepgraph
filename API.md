# DeepGraph API Reference

## Table of Contents
- [Core Data Types](#core-data-types)
- [Storage API](#storage-api)
- [Transaction API](#transaction-api)
- [Parser API](#parser-api)
- [Error Handling](#error-handling)

## Core Data Types

### Node

Represents a vertex in the graph.

```rust
pub struct Node {
    id: NodeId,
    labels: Vec<String>,
    properties: HashMap<String, PropertyValue>,
}
```

#### Methods

##### `Node::new(labels: Vec<String>) -> Self`
Creates a new node with the given labels.

```rust
let node = Node::new(vec!["Person".to_string()]);
```

##### `node.id() -> NodeId`
Returns the unique identifier of the node.

##### `node.labels() -> &[String]`
Returns a slice of the node's labels.

##### `node.add_label(label: String)`
Adds a label to the node (if not already present).

```rust
node.add_label("Employee".to_string());
```

##### `node.remove_label(label: &str) -> bool`
Removes a label from the node. Returns `true` if the label was present.

##### `node.has_label(label: &str) -> bool`
Checks if the node has a specific label.

##### `node.set_property(key: String, value: PropertyValue)`
Sets a property on the node.

```rust
node.set_property("name".to_string(), "Alice".into());
node.set_property("age".to_string(), 30i64.into());
```

##### `node.get_property(key: &str) -> Option<&PropertyValue>`
Gets a property value by key.

##### `node.remove_property(key: &str) -> Option<PropertyValue>`
Removes a property from the node and returns its value.

##### `node.has_property(key: &str) -> bool`
Checks if the node has a specific property.

### Edge

Represents a relationship between two nodes.

```rust
pub struct Edge {
    id: EdgeId,
    from: NodeId,
    to: NodeId,
    relationship_type: String,
    properties: HashMap<String, PropertyValue>,
}
```

#### Methods

##### `Edge::new(from: NodeId, to: NodeId, relationship_type: String) -> Self`
Creates a new edge between two nodes.

```rust
let edge = Edge::new(node1_id, node2_id, "KNOWS".to_string());
```

##### `edge.id() -> EdgeId`
Returns the unique identifier of the edge.

##### `edge.from() -> NodeId`
Returns the source node ID.

##### `edge.to() -> NodeId`
Returns the target node ID.

##### `edge.relationship_type() -> &str`
Returns the relationship type.

##### `edge.set_property(key: String, value: PropertyValue)`
Sets a property on the edge.

```rust
edge.set_property("since".to_string(), 2015i64.into());
```

##### `edge.get_property(key: &str) -> Option<&PropertyValue>`
Gets a property value by key.

##### `edge.remove_property(key: &str) -> Option<PropertyValue>`
Removes a property from the edge.

##### `edge.has_property(key: &str) -> bool`
Checks if the edge has a specific property.

### PropertyValue

Represents a property value with multiple supported types.

```rust
pub enum PropertyValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
    List(Vec<PropertyValue>),
    Map(HashMap<String, PropertyValue>),
}
```

#### Conversions

PropertyValue implements `From` for common types:

```rust
let string_val = PropertyValue::from("text");
let int_val = PropertyValue::from(42i64);
let float_val = PropertyValue::from(3.14f64);
let bool_val = PropertyValue::from(true);
```

#### Methods

##### `value.is_null() -> bool`
Checks if the value is null.

##### `value.as_string() -> Option<&str>`
Attempts to get the value as a string.

##### `value.as_integer() -> Option<i64>`
Attempts to get the value as an integer.

##### `value.as_float() -> Option<f64>`
Attempts to get the value as a float.

##### `value.as_boolean() -> Option<bool>`
Attempts to get the value as a boolean.

## Storage API

### GraphStorage

The in-memory storage engine for the graph database.

```rust
pub struct GraphStorage {
    // Internal fields
}
```

#### Methods

##### `GraphStorage::new() -> Self`
Creates a new empty graph storage.

```rust
let storage = GraphStorage::new();
```

##### `storage.node_count() -> usize`
Returns the number of nodes in the graph.

##### `storage.edge_count() -> usize`
Returns the number of edges in the graph.

### Node Operations

##### `storage.add_node(node: Node) -> Result<NodeId>`
Adds a node to the storage.

```rust
let node = Node::new(vec!["Person".to_string()]);
let id = storage.add_node(node)?;
```

##### `storage.get_node(id: NodeId) -> Result<Node>`
Retrieves a node by ID.

```rust
let node = storage.get_node(id)?;
```

##### `storage.update_node(node: Node) -> Result<()>`
Updates an existing node.

```rust
let mut node = storage.get_node(id)?;
node.set_property("age".to_string(), 31i64.into());
storage.update_node(node)?;
```

##### `storage.delete_node(id: NodeId) -> Result<()>`
Deletes a node and all its connected edges.

```rust
storage.delete_node(id)?;
```

##### `storage.get_nodes_by_label(label: &str) -> Vec<Node>`
Gets all nodes with a specific label.

```rust
let people = storage.get_nodes_by_label("Person");
```

##### `storage.get_nodes_by_property(key: &str, value: &PropertyValue) -> Vec<Node>`
Gets all nodes with a specific property value.

```rust
let age_30 = storage.get_nodes_by_property("age", &PropertyValue::Integer(30));
```

##### `storage.get_all_nodes() -> Vec<Node>`
Gets all nodes in the graph.

### Edge Operations

##### `storage.add_edge(edge: Edge) -> Result<EdgeId>`
Adds an edge to the storage. Both nodes must exist.

```rust
let edge = Edge::new(from_id, to_id, "KNOWS".to_string());
let edge_id = storage.add_edge(edge)?;
```

##### `storage.get_edge(id: EdgeId) -> Result<Edge>`
Retrieves an edge by ID.

```rust
let edge = storage.get_edge(edge_id)?;
```

##### `storage.update_edge(edge: Edge) -> Result<()>`
Updates an existing edge.

```rust
let mut edge = storage.get_edge(edge_id)?;
edge.set_property("weight".to_string(), 1.5f64.into());
storage.update_edge(edge)?;
```

##### `storage.delete_edge(id: EdgeId) -> Result<()>`
Deletes an edge.

```rust
storage.delete_edge(edge_id)?;
```

##### `storage.get_outgoing_edges(node_id: NodeId) -> Result<Vec<Edge>>`
Gets all outgoing edges from a node.

```rust
let outgoing = storage.get_outgoing_edges(node_id)?;
```

##### `storage.get_incoming_edges(node_id: NodeId) -> Result<Vec<Edge>>`
Gets all incoming edges to a node.

```rust
let incoming = storage.get_incoming_edges(node_id)?;
```

##### `storage.get_edges_by_type(relationship_type: &str) -> Vec<Edge>`
Gets all edges of a specific type.

```rust
let knows_edges = storage.get_edges_by_type("KNOWS");
```

##### `storage.get_all_edges() -> Vec<Edge>`
Gets all edges in the graph.

##### `storage.clear()`
Clears all data from storage.

## Transaction API

### Transaction

Represents a database transaction (placeholder in Phase 1).

```rust
pub struct Transaction {
    // Internal fields
}
```

#### Methods

##### `Transaction::begin(storage: Arc<GraphStorage>) -> Self`
Begins a new transaction.

```rust
use std::sync::Arc;
let storage = Arc::new(GraphStorage::new());
let tx = Transaction::begin(storage);
```

##### `Transaction::begin_with_isolation(storage: Arc<GraphStorage>, level: IsolationLevel) -> Self`
Begins a transaction with a specific isolation level.

```rust
let tx = Transaction::begin_with_isolation(storage, IsolationLevel::Serializable);
```

##### `tx.id() -> TransactionId`
Returns the transaction ID.

##### `tx.state() -> TransactionState`
Returns the current transaction state.

##### `tx.is_active() -> bool`
Checks if the transaction is active.

##### Transaction Operations

All storage operations are available on transactions:
- `tx.add_node(node: Node) -> Result<NodeId>`
- `tx.get_node(id: NodeId) -> Result<Node>`
- `tx.update_node(node: Node) -> Result<()>`
- `tx.delete_node(id: NodeId) -> Result<()>`
- `tx.add_edge(edge: Edge) -> Result<EdgeId>`
- `tx.get_edge(id: EdgeId) -> Result<Edge>`
- `tx.update_edge(edge: Edge) -> Result<()>`
- `tx.delete_edge(id: EdgeId) -> Result<()>`

##### `tx.commit(self) -> Result<()>`
Commits the transaction.

```rust
tx.commit()?;
```

##### `tx.rollback(self) -> Result<()>`
Rolls back the transaction.

```rust
tx.rollback()?;
```

### TransactionManager

Manages multiple transactions.

```rust
pub struct TransactionManager {
    // Internal fields
}
```

#### Methods

##### `TransactionManager::new(storage: Arc<GraphStorage>) -> Self`
Creates a new transaction manager.

##### `manager.begin_transaction() -> Transaction`
Begins a new transaction.

##### `manager.begin_transaction_with_isolation(level: IsolationLevel) -> Transaction`
Begins a transaction with a specific isolation level.

### IsolationLevel

```rust
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}
```

### TransactionState

```rust
pub enum TransactionState {
    Active,
    Committing,
    Committed,
    RollingBack,
    RolledBack,
    Aborted,
}
```

## Parser API

### CypherParser

Basic Cypher query parser (placeholder in Phase 1).

```rust
pub struct CypherParser;
```

#### Methods

##### `CypherParser::new() -> Self`
Creates a new parser.

##### `parser.parse(query: &str) -> Result<CypherQuery>`
Parses a Cypher query string.

```rust
let parser = CypherParser::new();
let query = parser.parse("MATCH (n:Person) RETURN n")?;
```

##### `parser.validate(query: &CypherQuery) -> Result<()>`
Validates a parsed query.

### CypherQuery

```rust
pub struct CypherQuery {
    pub raw_query: String,
    pub query_type: QueryType,
}
```

### QueryType

```rust
pub enum QueryType {
    Match,
    Create,
    Merge,
    Delete,
    Set,
    Unknown,
}
```

## Error Handling

### DeepGraphError

```rust
pub enum DeepGraphError {
    NodeNotFound(String),
    EdgeNotFound(String),
    PropertyNotFound(String),
    InvalidNodeId(String),
    InvalidEdgeId(String),
    StorageError(String),
    TransactionError(String),
    ParserError(String),
    InvalidPropertyType { expected: String, actual: String },
    IoError(std::io::Error),
    SerializationError(serde_json::Error),
    Unknown(String),
}
```

### Result Type

```rust
pub type Result<T> = std::result::Result<T, DeepGraphError>;
```

All fallible operations return `Result<T>`.

## Usage Examples

### Complete Example

```rust
use deepgraph::{GraphStorage, Node, Edge, Transaction};
use std::sync::Arc;

fn main() -> deepgraph::Result<()> {
    // Create storage
    let storage = Arc::new(GraphStorage::new());
    
    // Create nodes
    let mut alice = Node::new(vec!["Person".to_string()]);
    alice.set_property("name".to_string(), "Alice".into());
    alice.set_property("age".to_string(), 30i64.into());
    
    let mut bob = Node::new(vec!["Person".to_string()]);
    bob.set_property("name".to_string(), "Bob".into());
    
    let alice_id = storage.add_node(alice)?;
    let bob_id = storage.add_node(bob)?;
    
    // Create relationship
    let mut edge = Edge::new(alice_id, bob_id, "KNOWS".to_string());
    edge.set_property("since".to_string(), 2015i64.into());
    storage.add_edge(edge)?;
    
    // Query
    let people = storage.get_nodes_by_label("Person");
    println!("Found {} people", people.len());
    
    let alice_friends = storage.get_outgoing_edges(alice_id)?;
    println!("Alice knows {} people", alice_friends.len());
    
    // Transaction example
    let mut tx = Transaction::begin(storage);
    let new_node = Node::new(vec!["Person".to_string()]);
    let new_id = tx.add_node(new_node)?;
    tx.commit()?;
    
    Ok(())
}
```

## Thread Safety

- `GraphStorage` is thread-safe and can be shared across threads using `Arc<GraphStorage>`
- All storage operations use concurrent data structures (DashMap)
- Transactions are currently not fully isolated (Phase 1 limitation)

## Performance Characteristics

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| add_node | O(1) | Hash map insertion |
| get_node | O(1) | Hash map lookup |
| delete_node | O(d) | d = degree of node |
| add_edge | O(1) | Hash map insertion |
| get_edge | O(1) | Hash map lookup |
| get_outgoing_edges | O(k) | k = number of outgoing edges |
| get_incoming_edges | O(k) | k = number of incoming edges |
| get_nodes_by_label | O(n) | n = total nodes (will be O(k) in Phase 2 with indexing) |
| get_nodes_by_property | O(n) | n = total nodes (will be O(k) in Phase 2 with indexing) |

## Limitations (Phase 1)

- In-memory only (no persistence)
- No query optimization
- Transactions are placeholders (no true ACID guarantees)
- No indexing (full scans for label/property queries)
- Parser only recognizes query types, doesn't parse structure

These limitations will be addressed in future phases.

