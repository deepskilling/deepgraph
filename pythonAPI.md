# DeepGraph Python API Reference

Complete reference for DeepGraph's Python bindings - 100% API Coverage

## Table of Contents

1. [Installation](#installation)
2. [Core Storage](#core-storage)
3. [Transaction Management](#transaction-management)
4. [Index Management](#index-management)
5. [Write-Ahead Logging (WAL)](#write-ahead-logging-wal)
6. [Query System](#query-system)
7. [MVCC & Concurrency](#mvcc--concurrency)
8. [Deadlock Detection](#deadlock-detection)
9. [Module Metadata](#module-metadata)
10. [Complete Examples](#complete-examples)

---

## Installation

### Prerequisites

```bash
# Ensure Rust and Python are installed
rustc --version  # >= 1.70
python --version # >= 3.8
```

### Install from Source

```bash
# Install maturin
pip install maturin

# Build and install
maturin develop --release --features python
```

### Usage

```python
import deepgraph

# Access classes
storage = deepgraph.GraphStorage()
tx_manager = deepgraph.TransactionManager()
```

---

## Core Storage

### `GraphStorage`

High-performance graph storage with CRUD operations.

#### Constructor

```python
storage = deepgraph.GraphStorage()
```

#### Methods

##### `add_node(labels: List[str], properties: Dict[str, Any]) -> str`

Create a new node in the graph.

**Parameters:**
- `labels` (List[str]): List of labels for the node (e.g., `["Person", "Employee"]`)
- `properties` (Dict[str, Any]): Node properties as key-value pairs

**Returns:**
- `str`: UUID of the created node

**Example:**

```python
node_id = storage.add_node(
    ["Person", "Employee"],
    {
        "name": "Alice",
        "age": 30,
        "email": "alice@example.com"
    }
)
print(f"Created node: {node_id}")
```

---

##### `add_edge(from_id: str, to_id: str, relationship_type: str, properties: Dict[str, Any]) -> str`

Create a new edge between two nodes.

**Parameters:**
- `from_id` (str): Source node UUID
- `to_id` (str): Target node UUID
- `relationship_type` (str): Type of relationship (e.g., `"KNOWS"`, `"MANAGES"`)
- `properties` (Dict[str, Any]): Edge properties

**Returns:**
- `str`: UUID of the created edge

**Example:**

```python
edge_id = storage.add_edge(
    node1_id,
    node2_id,
    "KNOWS",
    {"since": 2020, "strength": 0.8}
)
```

---

##### `get_node(node_id: str) -> Dict[str, Any]`

Retrieve a node by its ID.

**Parameters:**
- `node_id` (str): Node UUID

**Returns:**
- `Dict[str, Any]`: Node data with `id`, `labels`, and `properties`

**Example:**

```python
node = storage.get_node(node_id)
print(f"Name: {node['properties']['name']}")
print(f"Labels: {node['labels']}")
```

---

##### `get_edge(edge_id: str) -> Dict[str, Any]`

Retrieve an edge by its ID.

**Parameters:**
- `edge_id` (str): Edge UUID

**Returns:**
- `Dict[str, Any]`: Edge data with `id`, `from_id`, `to_id`, `relationship_type`, and `properties`

**Example:**

```python
edge = storage.get_edge(edge_id)
print(f"From: {edge['from_id']} -> To: {edge['to_id']}")
print(f"Type: {edge['relationship_type']}")
```

---

##### `update_node(node_id: str, properties: Dict[str, Any]) -> None`

Update a node's properties.

**Parameters:**
- `node_id` (str): Node UUID
- `properties` (Dict[str, Any]): Properties to update/add

**Example:**

```python
storage.update_node(node_id, {"age": 31, "city": "NYC"})
```

---

##### `update_edge(edge_id: str, properties: Dict[str, Any]) -> None`

Update an edge's properties.

**Parameters:**
- `edge_id` (str): Edge UUID
- `properties` (Dict[str, Any]): Properties to update/add

**Example:**

```python
storage.update_edge(edge_id, {"strength": 0.9})
```

---

##### `delete_node(node_id: str) -> None`

Delete a node from the graph.

**Parameters:**
- `node_id` (str): Node UUID to delete

**Example:**

```python
storage.delete_node(node_id)
```

---

##### `delete_edge(edge_id: str) -> None`

Delete an edge from the graph.

**Parameters:**
- `edge_id` (str): Edge UUID to delete

**Example:**

```python
storage.delete_edge(edge_id)
```

---

##### `get_outgoing_edges(node_id: str) -> List[Dict[str, Any]]`

Get all edges originating from a node.

**Parameters:**
- `node_id` (str): Source node UUID

**Returns:**
- `List[Dict[str, Any]]`: List of outgoing edges

**Example:**

```python
outgoing = storage.get_outgoing_edges(node_id)
for edge in outgoing:
    print(f"-> {edge['to_id']} ({edge['relationship_type']})")
```

---

##### `get_incoming_edges(node_id: str) -> List[Dict[str, Any]]`

Get all edges pointing to a node.

**Parameters:**
- `node_id` (str): Target node UUID

**Returns:**
- `List[Dict[str, Any]]`: List of incoming edges

**Example:**

```python
incoming = storage.get_incoming_edges(node_id)
for edge in incoming:
    print(f"{edge['from_id']} -> ({edge['relationship_type']})")
```

---

##### `find_nodes_by_label(label: str) -> List[Dict[str, Any]]`

Find all nodes with a specific label.

**Parameters:**
- `label` (str): Label to search for

**Returns:**
- `List[Dict[str, Any]]`: List of matching nodes

**Example:**

```python
persons = storage.find_nodes_by_label("Person")
print(f"Found {len(persons)} Person nodes")
```

---

##### `find_nodes_by_property(key: str, value: Any) -> List[Dict[str, Any]]`

Find nodes with a specific property value.

**Parameters:**
- `key` (str): Property key
- `value` (Any): Property value to match

**Returns:**
- `List[Dict[str, Any]]`: List of matching nodes

**Example:**

```python
alice_nodes = storage.find_nodes_by_property("name", "Alice")
```

---

##### `find_edges_by_type(relationship_type: str) -> List[Dict[str, Any]]`

Find all edges of a specific type.

**Parameters:**
- `relationship_type` (str): Relationship type to search for

**Returns:**
- `List[Dict[str, Any]]`: List of matching edges

**Example:**

```python
knows_edges = storage.find_edges_by_type("KNOWS")
print(f"Found {len(knows_edges)} KNOWS relationships")
```

---

##### `get_all_nodes() -> List[Dict[str, Any]]`

Retrieve all nodes in the graph.

**Returns:**
- `List[Dict[str, Any]]`: List of all nodes

**Example:**

```python
all_nodes = storage.get_all_nodes()
print(f"Total nodes: {len(all_nodes)}")
```

---

##### `get_all_edges() -> List[Dict[str, Any]]`

Retrieve all edges in the graph.

**Returns:**
- `List[Dict[str, Any]]`: List of all edges

**Example:**

```python
all_edges = storage.get_all_edges()
print(f"Total edges: {len(all_edges)}")
```

---

##### `clear() -> None`

Clear all data from the graph.

**Example:**

```python
storage.clear()
```

---

## Transaction Management

### `TransactionManager`

MVCC-based transaction management with ACID guarantees.

#### Constructor

```python
tx_manager = deepgraph.TransactionManager()
```

#### Methods

##### `begin_transaction() -> int`

Start a new transaction.

**Returns:**
- `int`: Transaction ID

**Example:**

```python
txn_id = tx_manager.begin_transaction()
print(f"Started transaction: {txn_id}")
```

---

##### `commit_transaction(txn_id: int) -> None`

Commit a transaction.

**Parameters:**
- `txn_id` (int): Transaction ID to commit

**Example:**

```python
tx_manager.commit_transaction(txn_id)
print("Transaction committed")
```

---

##### `abort_transaction(txn_id: int) -> None`

Abort a transaction and rollback changes.

**Parameters:**
- `txn_id` (int): Transaction ID to abort

**Example:**

```python
try:
    txn_id = tx_manager.begin_transaction()
    # ... operations ...
    tx_manager.commit_transaction(txn_id)
except Exception as e:
    tx_manager.abort_transaction(txn_id)
    print(f"Transaction aborted: {e}")
```

---

## Index Management

### `IndexManager`

Create and manage indexes for fast lookups.

#### Constructor

```python
idx_manager = deepgraph.IndexManager()
```

#### Methods

##### `create_hash_index(index_name: str, label: str) -> None`

Create a hash index for O(1) lookups.

**Parameters:**
- `index_name` (str): Name for the index
- `label` (str): Label to index

**Example:**

```python
idx_manager.create_hash_index("person_name_idx", "Person")
```

---

##### `create_btree_index(index_name: str, property_key: str) -> None`

Create a B-tree index for range queries.

**Parameters:**
- `index_name` (str): Name for the index
- `property_key` (str): Property to index

**Example:**

```python
idx_manager.create_btree_index("age_idx", "age")
```

---

##### `drop_index(index_name: str) -> None`

Remove an index.

**Parameters:**
- `index_name` (str): Name of index to drop

**Example:**

```python
idx_manager.drop_index("person_name_idx")
```

---

## Write-Ahead Logging (WAL)

### `WAL`

Write-ahead logging for durability and crash recovery.

#### Constructor

```python
wal = deepgraph.WAL(wal_dir: str)
```

**Parameters:**
- `wal_dir` (str): Directory path for WAL storage

**Example:**

```python
wal = deepgraph.WAL("./data/wal")
```

#### Methods

##### `flush() -> None`

Flush WAL buffer to disk.

**Example:**

```python
wal.flush()
print("WAL flushed to disk")
```

---

### `WALRecovery`

Recover database state from WAL after crash.

#### Constructor

```python
recovery = deepgraph.WALRecovery(wal_dir: str)
```

**Parameters:**
- `wal_dir` (str): Directory path for WAL storage

**Example:**

```python
recovery = deepgraph.WALRecovery("./data/wal")
```

#### Methods

##### `recover(storage: GraphStorage) -> int`

Recover database from WAL.

**Parameters:**
- `storage` (GraphStorage): Storage instance to recover into

**Returns:**
- `int`: Number of entries recovered

**Example:**

```python
storage = deepgraph.GraphStorage()
recovered = recovery.recover(storage)
print(f"Recovered {recovered} entries")
```

---

## Query System

### `CypherParser`

Parse Cypher query language.

#### Constructor

```python
parser = deepgraph.CypherParser()
```

#### Methods

##### `parse(query: str) -> str`

Parse a Cypher query into AST.

**Parameters:**
- `query` (str): Cypher query string

**Returns:**
- `str`: Parsed query representation

**Example:**

```python
ast = parser.parse("MATCH (n:Person) RETURN n")
print(ast)
```

---

##### `validate(query: str) -> None`

Validate query syntax.

**Parameters:**
- `query` (str): Cypher query string

**Raises:**
- `RuntimeError`: If query syntax is invalid

**Example:**

```python
try:
    parser.validate("MATCH (n:Person) RETURN n")
    print("Query is valid")
except RuntimeError as e:
    print(f"Invalid query: {e}")
```

---

### `QueryPlanner`

Create and optimize query execution plans.

#### Constructor

```python
planner = deepgraph.QueryPlanner()
```

#### Methods

##### `create_logical_plan(query_str: str) -> str`

Create a logical plan from parsed query.

**Parameters:**
- `query_str` (str): Parsed query string

**Returns:**
- `str`: Logical plan representation

**Example:**

```python
plan = planner.create_logical_plan("MATCH (n) RETURN n")
print(plan)
```

---

##### `optimize(plan_str: str) -> str`

Optimize a logical plan.

**Parameters:**
- `plan_str` (str): Logical plan string

**Returns:**
- `str`: Optimized plan

**Example:**

```python
optimized = planner.optimize(plan)
print(optimized)
```

---

### `QueryExecutor`

Execute optimized query plans.

#### Constructor

```python
executor = deepgraph.QueryExecutor(storage: GraphStorage)
```

**Parameters:**
- `storage` (GraphStorage): Storage instance to query

**Example:**

```python
storage = deepgraph.GraphStorage()
executor = deepgraph.QueryExecutor(storage)
```

#### Methods

##### `execute(query: str) -> Dict[str, Any]`

Execute a Cypher query.

**Parameters:**
- `query` (str): Cypher query string

**Returns:**
- `Dict[str, Any]`: Query result with columns and rows

**Example:**

```python
result = executor.execute("MATCH (n:Person) RETURN n")
print(f"Rows: {result['row_count']}")
```

---

## MVCC & Concurrency

### `Snapshot`

MVCC snapshot for consistent reads.

#### Constructor

```python
snapshot = deepgraph.Snapshot()
```

#### Static Methods

##### `current_timestamp() -> int`

Get current database timestamp.

**Returns:**
- `int`: Current timestamp

**Example:**

```python
ts = deepgraph.Snapshot.current_timestamp()
print(f"Current timestamp: {ts}")
```

#### Methods

##### `get_timestamp() -> int`

Get snapshot's timestamp.

**Returns:**
- `int`: Snapshot timestamp

**Example:**

```python
snapshot = deepgraph.Snapshot()
ts = snapshot.get_timestamp()
print(f"Snapshot at: {ts}")
```

---

## Deadlock Detection

### `DeadlockDetector`

Wait-for graph based deadlock detection.

#### Constructor

```python
detector = deepgraph.DeadlockDetector()
```

#### Methods

##### `request_lock(txn_id: int, resource_id: int) -> None`

Request a lock on a resource.

**Parameters:**
- `txn_id` (int): Transaction ID
- `resource_id` (int): Resource ID

**Raises:**
- `RuntimeError`: If deadlock detected or resource locked

**Example:**

```python
try:
    detector.request_lock(txn_id=1, resource_id=100)
    print("Lock acquired")
except RuntimeError as e:
    print(f"Lock failed: {e}")
```

---

##### `release_lock(txn_id: int, resource_id: int) -> None`

Release a lock on a resource.

**Parameters:**
- `txn_id` (int): Transaction ID
- `resource_id` (int): Resource ID

**Example:**

```python
detector.release_lock(txn_id=1, resource_id=100)
```

---

##### `release_all_locks(txn_id: int) -> None`

Release all locks held by a transaction.

**Parameters:**
- `txn_id` (int): Transaction ID

**Example:**

```python
detector.release_all_locks(txn_id=1)
```

---

##### `get_deadlocked_txns(start_txn_id: int) -> List[int]`

Get all transactions involved in a deadlock.

**Parameters:**
- `start_txn_id` (int): Starting transaction ID

**Returns:**
- `List[int]`: List of transaction IDs

**Example:**

```python
deadlocked = detector.get_deadlocked_txns(1)
print(f"Deadlocked transactions: {deadlocked}")
```

---

##### `stats() -> Dict[str, int]`

Get deadlock detector statistics.

**Returns:**
- `Dict[str, int]`: Statistics with `waiting_transactions` and `locked_resources`

**Example:**

```python
stats = detector.stats()
print(f"Waiting: {stats['waiting_transactions']}")
print(f"Locked: {stats['locked_resources']}")
```

---

## Module Metadata

### Version Information

```python
import deepgraph

print(f"Version: {deepgraph.__version__}")
print(f"Author: {deepgraph.__author__}")
```

---

## Complete Examples

### Example 1: Basic Graph Operations

```python
import deepgraph

# Create storage
storage = deepgraph.GraphStorage()

# Add nodes
alice = storage.add_node(["Person"], {"name": "Alice", "age": 30})
bob = storage.add_node(["Person"], {"name": "Bob", "age": 25})
charlie = storage.add_node(["Person"], {"name": "Charlie", "age": 35})

# Add relationships
edge1 = storage.add_edge(alice, bob, "KNOWS", {"since": 2020})
edge2 = storage.add_edge(bob, charlie, "WORKS_WITH", {"since": 2019})

# Query
persons = storage.find_nodes_by_label("Person")
print(f"Total persons: {len(persons)}")

# Traverse
friends = storage.get_outgoing_edges(alice)
for edge in friends:
    friend = storage.get_node(edge['to_id'])
    print(f"Alice knows {friend['properties']['name']}")
```

---

### Example 2: Transactions

```python
import deepgraph

storage = deepgraph.GraphStorage()
tx_manager = deepgraph.TransactionManager()

# Begin transaction
txn_id = tx_manager.begin_transaction()

try:
    # Perform operations
    node_id = storage.add_node(["User"], {"username": "alice"})
    storage.update_node(node_id, {"verified": True})
    
    # Commit
    tx_manager.commit_transaction(txn_id)
    print("Transaction committed")
except Exception as e:
    # Rollback on error
    tx_manager.abort_transaction(txn_id)
    print(f"Transaction aborted: {e}")
```

---

### Example 3: Indexing

```python
import deepgraph

storage = deepgraph.GraphStorage()
idx_manager = deepgraph.IndexManager()

# Create indexes
idx_manager.create_hash_index("person_idx", "Person")
idx_manager.create_btree_index("age_idx", "age")

# Add data
for i in range(1000):
    storage.add_node(["Person"], {"name": f"Person{i}", "age": 20 + i % 50})

# Fast lookups (using indexes behind the scenes)
persons = storage.find_nodes_by_label("Person")
print(f"Found {len(persons)} persons")
```

---

### Example 4: WAL and Recovery

```python
import deepgraph
import tempfile

# Create WAL
with tempfile.TemporaryDirectory() as tmpdir:
    wal = deepgraph.WAL(tmpdir)
    storage = deepgraph.GraphStorage()
    
    # Add data
    node_id = storage.add_node(["Person"], {"name": "Alice"})
    
    # Flush to disk
    wal.flush()
    
    # Simulate crash and recovery
    recovery = deepgraph.WALRecovery(tmpdir)
    new_storage = deepgraph.GraphStorage()
    recovered = recovery.recover(new_storage)
    
    print(f"Recovered {recovered} entries")
```

---

### Example 5: Deadlock Detection

```python
import deepgraph

detector = deepgraph.DeadlockDetector()

# Transaction 1 locks resource 100
detector.request_lock(txn_id=1, resource_id=100)

# Transaction 2 locks resource 200
detector.request_lock(txn_id=2, resource_id=200)

# Transaction 1 tries to lock resource 200 (held by txn 2)
try:
    detector.request_lock(txn_id=1, resource_id=200)
except RuntimeError as e:
    print(f"Transaction 1 waiting: {e}")

# Transaction 2 tries to lock resource 100 (held by txn 1)
# This creates a deadlock!
try:
    detector.request_lock(txn_id=2, resource_id=100)
except RuntimeError as e:
    print(f"Deadlock detected: {e}")

# Check stats
stats = detector.stats()
print(f"Statistics: {stats}")
```

---

### Example 6: Social Network

```python
import deepgraph

storage = deepgraph.GraphStorage()

# Create users
users = {}
for name in ["Alice", "Bob", "Charlie", "David"]:
    user_id = storage.add_node(
        ["User"],
        {"name": name, "joined": "2024"}
    )
    users[name] = user_id

# Create friendships
storage.add_edge(users["Alice"], users["Bob"], "FRIEND", {"since": 2020})
storage.add_edge(users["Alice"], users["Charlie"], "FRIEND", {"since": 2021})
storage.add_edge(users["Bob"], users["David"], "FRIEND", {"since": 2022})

# Find Alice's friends
alice_edges = storage.get_outgoing_edges(users["Alice"])
print(f"Alice has {len(alice_edges)} friends")

for edge in alice_edges:
    friend = storage.get_node(edge['to_id'])
    print(f"  - {friend['properties']['name']}")

# Find who is friends with Bob
bob_incoming = storage.get_incoming_edges(users["Bob"])
print(f"\nFriends with Bob: {len(bob_incoming)}")
for edge in bob_incoming:
    friend = storage.get_node(edge['from_id'])
    print(f"  - {friend['properties']['name']}")
```

---

## API Coverage Summary

| Category | Methods | Status |
|----------|---------|--------|
| **Core Storage** | 20 | ✅ Complete |
| **Transaction Manager** | 3 | ✅ Complete |
| **Index Manager** | 3 | ✅ Complete |
| **WAL & Recovery** | 3 | ✅ Complete |
| **Query System** | 5 | ✅ Complete |
| **MVCC Snapshot** | 2 | ✅ Complete |
| **Deadlock Detector** | 5 | ✅ Complete |
| **Metadata** | 2 | ✅ Complete |
| **TOTAL** | **43** | **100% Coverage** |

---

## Performance Tips

1. **Use Indexes**: Create indexes for frequently queried labels and properties
2. **Batch Operations**: Group multiple operations in transactions
3. **WAL Configuration**: Adjust WAL sync settings based on durability vs. performance needs
4. **Connection Pooling**: Reuse storage and manager instances
5. **Property Types**: Use appropriate types (int, float, string) for better performance

---

## Error Handling

All methods may raise `RuntimeError` on failure. Always use try-except:

```python
try:
    node = storage.get_node(node_id)
except RuntimeError as e:
    print(f"Error: {e}")
```

---

## Thread Safety

- **GraphStorage**: Thread-safe with internal RwLock
- **TransactionManager**: Thread-safe using concurrent data structures
- **IndexManager**: Thread-safe with RwLock
- **DeadlockDetector**: Thread-safe using DashMap

---

## Version Compatibility

- **Python**: 3.8+
- **Rust**: 1.70+
- **PyO3**: 0.21+

---

## Additional Resources

- [GitHub Repository](https://github.com/deepskilling/deepgraph)
- [Examples](./examples/python/)
- [Rust Documentation](./doc/)

---

**DeepGraph** - High-Performance Graph Database
© 2024 DeepSkilling. Licensed under MIT.

