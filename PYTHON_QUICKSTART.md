# DeepGraph Python Quick Start Guide

**Choose your role to get started quickly:**

- 👤 [**End Users**](#end-users-quick-start) - Build applications with DeepGraph
- 👨‍💻 [**Developers**](#developers-quick-start) - Contribute to DeepGraph
- 🔧 [**SRE/DevOps**](#sredevops-quick-start) - Deploy and operate DeepGraph

---

# End Users Quick Start

> **Goal**: Start building graph applications with DeepGraph in Python

## Installation

### Prerequisites

```bash
# Check Python version (3.8+ required)
python --version

# Check Rust (1.75+ required)
rustc --version
```

### Install DeepGraph

```bash
# Clone the repository
git clone https://github.com/deepskilling/deepgraph.git
cd deepgraph

# Build and install Python bindings
pip install maturin
maturin develop --release --features python
```

### Verify Installation

```python
import deepgraph

print(f"DeepGraph v{deepgraph.__version__}")
# Output: DeepGraph v0.1.0
```

---

## 5-Minute Tutorial

### Step 1: Create a Graph

```python
import deepgraph

# Create a new graph database
storage = deepgraph.GraphStorage()

# Add nodes with labels and properties
alice = storage.add_node(
    labels=["Person", "Engineer"],
    properties={
        "name": "Alice",
        "age": 30,
        "email": "alice@example.com"
    }
)

bob = storage.add_node(
    labels=["Person", "Manager"],
    properties={
        "name": "Bob",
        "age": 35,
        "email": "bob@example.com"
    }
)

acme = storage.add_node(
    labels=["Company"],
    properties={
        "name": "Acme Corp",
        "founded": 2010
    }
)

print(f"Created {storage.node_count()} nodes")
```

### Step 2: Create Relationships

```python
# Add edges between nodes
knows = storage.add_edge(
    from_node=alice,
    to_node=bob,
    relationship_type="KNOWS",
    properties={"since": 2015, "strength": "strong"}
)

works_at_1 = storage.add_edge(
    from_node=alice,
    to_node=acme,
    relationship_type="WORKS_AT",
    properties={"role": "Senior Engineer", "start_date": "2018-01-01"}
)

works_at_2 = storage.add_edge(
    from_node=bob,
    to_node=acme,
    relationship_type="WORKS_AT",
    properties={"role": "Engineering Manager", "start_date": "2016-06-15"}
)

print(f"Created {storage.edge_count()} relationships")
```

### Step 3: Query the Graph

```python
# Find all Person nodes
persons = storage.find_nodes_by_label("Person")
print(f"Found {len(persons)} persons")

# Find nodes by property
engineers = storage.find_nodes_by_property("role", "Senior Engineer")

# Get relationships
alice_relationships = storage.get_outgoing_edges(alice)
print(f"Alice has {len(alice_relationships)} relationships")

# Traverse the graph
for edge in alice_relationships:
    edge_data = storage.get_edge(edge)
    print(f"Alice -> {edge_data}")
```

### Step 4: Use Transactions

```python
# Create transaction manager
tx_mgr = deepgraph.TransactionManager()

# Begin transaction
txn_id = tx_mgr.begin_transaction()

try:
    # Make changes within transaction
    charlie = storage.add_node(
        labels=["Person"],
        properties={"name": "Charlie"}
    )
    
    edge = storage.add_edge(alice, charlie, "KNOWS", {})
    
    # Commit if successful
    tx_mgr.commit_transaction(txn_id)
    print("Transaction committed")
    
except Exception as e:
    # Rollback on error
    tx_mgr.abort_transaction(txn_id)
    print(f"Transaction aborted: {e}")
```

### Step 5: Create Indices for Performance

```python
# Create index manager
idx_mgr = deepgraph.IndexManager()

# Create hash index for fast lookups
idx_mgr.create_hash_index(
    name="person_idx",
    label="Person"
)

# Create B-tree index for range queries
idx_mgr.create_btree_index(
    name="age_idx",
    property="age"
)

print("Indices created for better query performance")
```

---

## Common Use Cases

### Use Case 1: Social Network

```python
import deepgraph

storage = deepgraph.GraphStorage()

# Create users
users = {}
for name in ["Alice", "Bob", "Charlie", "Diana"]:
    user_id = storage.add_node(
        ["User"],
        {"name": name, "joined": "2024-01-01"}
    )
    users[name] = user_id

# Create friendships
friendships = [
    ("Alice", "Bob"),
    ("Bob", "Charlie"),
    ("Charlie", "Diana"),
    ("Alice", "Diana")
]

for user1, user2 in friendships:
    storage.add_edge(
        users[user1],
        users[user2],
        "FRIENDS_WITH",
        {"since": "2024-01-15"}
    )

# Find Alice's friends
alice_edges = storage.get_outgoing_edges(users["Alice"])
print(f"Alice has {len(alice_edges)} friends")

# Find all friendships
all_friendships = storage.find_edges_by_type("FRIENDS_WITH")
print(f"Total friendships: {len(all_friendships)}")
```

### Use Case 2: Product Recommendations

```python
import deepgraph

storage = deepgraph.GraphStorage()

# Create products
laptop = storage.add_node(["Product"], {
    "name": "Laptop Pro",
    "price": 1299.99,
    "category": "Electronics"
})

mouse = storage.add_node(["Product"], {
    "name": "Wireless Mouse",
    "price": 29.99,
    "category": "Electronics"
})

# Create customers
customer = storage.add_node(["Customer"], {
    "name": "John Doe",
    "email": "john@example.com"
})

# Track purchases
storage.add_edge(
    customer,
    laptop,
    "PURCHASED",
    {"date": "2024-01-20", "quantity": 1}
)

# Track views (for recommendations)
storage.add_edge(
    customer,
    mouse,
    "VIEWED",
    {"date": "2024-01-21", "duration_seconds": 45}
)

# Find what customer purchased
purchases = storage.get_outgoing_edges(customer)
print(f"Customer made {len(purchases)} interactions")
```

### Use Case 3: Knowledge Graph

```python
import deepgraph

storage = deepgraph.GraphStorage()

# Create concepts
python = storage.add_node(["Language"], {
    "name": "Python",
    "paradigm": "Multi-paradigm",
    "year": 1991
})

rust = storage.add_node(["Language"], {
    "name": "Rust",
    "paradigm": "Multi-paradigm",
    "year": 2010
})

deepgraph_concept = storage.add_node(["Library"], {
    "name": "DeepGraph",
    "type": "Graph Database"
})

# Create relationships
storage.add_edge(
    python,
    deepgraph_concept,
    "HAS_BINDING",
    {"version": "0.1.0"}
)

storage.add_edge(
    rust,
    deepgraph_concept,
    "IMPLEMENTED_IN",
    {"version": "0.1.0"}
)

# Query the knowledge
languages = storage.find_nodes_by_label("Language")
print(f"Found {len(languages)} programming languages")
```

---

## Graph Algorithms

DeepGraph includes 8 built-in graph algorithms:

### Shortest Path (Dijkstra)

```python
import deepgraph

storage = deepgraph.GraphStorage()

# Create a graph
a = storage.add_node(["City"], {"name": "A"})
b = storage.add_node(["City"], {"name": "B"})
c = storage.add_node(["City"], {"name": "C"})

# Add weighted edges (distances)
storage.add_edge(a, b, "ROAD", {"weight": 10})
storage.add_edge(b, c, "ROAD", {"weight": 20})
storage.add_edge(a, c, "ROAD", {"weight": 35})

# Find shortest path
result = deepgraph.dijkstra(
    storage,
    source=a,
    weight_property="weight"
)

# Get distance to C
distance = result.distance_to(c)
print(f"Shortest distance from A to C: {distance}")

# Get the path
path = result.path_to(c)
print(f"Path: {path}")
```

### PageRank (Centrality)

```python
import deepgraph

storage = deepgraph.GraphStorage()

# Create a citation network
papers = []
for i in range(5):
    paper = storage.add_node(["Paper"], {"title": f"Paper {i}"})
    papers.append(paper)

# Create citations
storage.add_edge(papers[0], papers[1], "CITES", {})
storage.add_edge(papers[0], papers[2], "CITES", {})
storage.add_edge(papers[1], papers[2], "CITES", {})
storage.add_edge(papers[3], papers[1], "CITES", {})

# Calculate PageRank
result = deepgraph.pagerank(
    storage,
    damping=0.85,
    max_iterations=100,
    tolerance=1e-4
)

# Get most important papers
print("PageRank scores:")
for node_id, score in result.scores.items():
    print(f"  {node_id}: {score:.4f}")
```

### Community Detection (Louvain)

```python
import deepgraph

storage = deepgraph.GraphStorage()

# Create a social network
users = [storage.add_node(["User"], {"id": i}) for i in range(10)]

# Create connections (communities)
# Group 1: 0-3
for i in range(4):
    for j in range(i+1, 4):
        storage.add_edge(users[i], users[j], "FRIENDS", {})

# Group 2: 4-7
for i in range(4, 8):
    for j in range(i+1, 8):
        storage.add_edge(users[i], users[j], "FRIENDS", {})

# Few connections between groups
storage.add_edge(users[3], users[4], "FRIENDS", {})

# Detect communities
result = deepgraph.louvain(
    storage,
    max_iterations=100,
    min_improvement=1e-4
)

print(f"Found {result.num_communities} communities")
print(f"Modularity score: {result.modularity:.4f}")

# Show community assignments
for node_id, community in result.communities.items():
    print(f"Node {node_id} -> Community {community}")
```

### More Algorithms

```python
# Breadth-First Search
bfs_result = deepgraph.bfs(storage, start_node=node_id, max_depth=3)

# Depth-First Search
dfs_result = deepgraph.dfs(storage, start_node=node_id)

# Connected Components
components = deepgraph.connected_components(storage)

# Triangle Counting
triangles = deepgraph.triangle_count(storage)

# Node2Vec (Graph Embeddings)
embeddings = deepgraph.node2vec(storage, {
    'walk_length': 80,
    'num_walks': 10,
    'p': 1.0,
    'q': 1.0
})
```

---

## Best Practices

### 1. Always Check for None

```python
# ✅ Good - check before using
node = storage.get_node(node_id)
if node is not None:
    process_node(node)
else:
    handle_missing_node()

# ❌ Bad - assume node exists
node = storage.get_node(node_id)
print(node.name)  # May fail if node is None
```

### 2. Use Transactions for Consistency

```python
# ✅ Good - use transactions
tx_mgr = deepgraph.TransactionManager()
txn = tx_mgr.begin_transaction()

try:
    storage.add_node(["User"], {"name": "Alice"})
    storage.add_node(["User"], {"name": "Bob"})
    tx_mgr.commit_transaction(txn)
except:
    tx_mgr.abort_transaction(txn)

# ❌ Bad - no transaction protection
storage.add_node(["User"], {"name": "Alice"})
storage.add_node(["User"], {"name": "Bob"})  # If this fails, Alice is still added
```

### 3. Create Indices for Large Graphs

```python
# ✅ Good - create indices before large queries
idx_mgr = deepgraph.IndexManager()
idx_mgr.create_hash_index("user_idx", "User")

# Now queries are fast
users = storage.find_nodes_by_label("User")  # O(1) with index

# ❌ Bad - no index for large graph
# queries = storage.find_nodes_by_label("User")  # O(n) without index
```

### 4. Use WAL for Durability

```python
# ✅ Good - use WAL for important data
wal = deepgraph.WAL("./data/wal")

# Make changes
storage.add_node(["Important"], {"data": "critical"})

# Flush to disk
wal.flush()

# If crash happens, can recover:
# recovery = deepgraph.WALRecovery("./data/wal")
# recovery.recover(storage)
```

---

## Troubleshooting

### Issue: Module not found

```bash
# Error: ModuleNotFoundError: No module named 'deepgraph'

# Solution: Rebuild and install
cd deepgraph
maturin develop --release --features python
```

### Issue: Rust compiler not found

```bash
# Error: error: could not find `rustc` in PATH

# Solution: Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Issue: Node returns None

```python
# This is NORMAL behavior (not an error)
node = storage.get_node("non-existent-id")
# Returns: None

# Always check:
if node is None:
    print("Node not found")
```

---

## Next Steps

- 📖 Read the [Python API Reference](pythonAPI.md)
- 🧪 Run [Example Scripts](examples/python/)
- 📚 Check [Architecture Documentation](ARCHITECTURE.md)
- 🐛 Report issues on [GitHub](https://github.com/deepskilling/deepgraph/issues)

---

# Developers Quick Start

> **Goal**: Set up development environment and contribute to DeepGraph

## Setup Development Environment

### 1. Clone Repository

```bash
git clone https://github.com/deepskilling/deepgraph.git
cd deepgraph
```

### 2. Install Development Tools

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Python development dependencies
pip install maturin pytest pytest-cov

# Install additional Rust tools
cargo install cargo-watch cargo-tarpaulin
```

### 3. Build in Development Mode

```bash
# Build Rust library
cargo build

# Build Python bindings in development mode (fast iteration)
maturin develop --features python

# Or with hot reload:
cargo watch -x 'build'
```

### 4. Run Tests

```bash
# Run Rust tests
cargo test --lib --verbose

# Run Python tests
cd PyRustTest
python test_1_core_operations.py
python test_2_transactions.py
python test_3_indexing.py
python test_4_durability.py
python test_5_query_language.py

# Or use pytest
pytest PyRustTest/ -v
```

---

## Development Workflow

### Making Changes

```bash
# 1. Create feature branch
git checkout -b feature/my-new-feature

# 2. Make changes to Rust code
vim src/storage/memory.rs

# 3. Rebuild Python bindings
maturin develop --features python

# 4. Test your changes
python -c "import deepgraph; storage = deepgraph.GraphStorage()"

# 5. Run full test suite
cargo test
pytest PyRustTest/

# 6. Check formatting and lints
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

### Adding New Python Bindings

1. **Add Rust implementation** (`src/my_feature.rs`):

```rust
// src/my_feature.rs
pub struct MyFeature {
    data: String,
}

impl MyFeature {
    pub fn new(data: String) -> Self {
        Self { data }
    }
    
    pub fn process(&self) -> String {
        format!("Processed: {}", self.data)
    }
}
```

2. **Expose to lib.rs**:

```rust
// src/lib.rs
pub mod my_feature;
pub use my_feature::MyFeature;
```

3. **Add Python wrapper** (`src/python.rs`):

```rust
// src/python.rs
use pyo3::prelude::*;
use crate::my_feature::MyFeature;

#[pyclass]
pub struct PyMyFeature {
    inner: MyFeature,
}

#[pymethods]
impl PyMyFeature {
    #[new]
    fn new(data: String) -> Self {
        PyMyFeature {
            inner: MyFeature::new(data),
        }
    }
    
    fn process(&self) -> PyResult<String> {
        Ok(self.inner.process())
    }
}

// Add to module (in pymodule function)
#[pymodule]
fn deepgraph(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // ... existing classes ...
    m.add_class::<PyMyFeature>()?;
    Ok(())
}
```

4. **Export in Python package** (`python/deepgraph/__init__.py`):

```python
from .deepgraph import PyMyFeature

# Alias
MyFeature = PyMyFeature

__all__ = [
    # ... existing exports ...
    'MyFeature',
    'PyMyFeature',
]
```

5. **Add tests** (`PyRustTest/test_my_feature.py`):

```python
def test_my_feature_basic():
    import deepgraph
    
    feature = deepgraph.MyFeature("test data")
    result = feature.process()
    
    assert result == "Processed: test data"
```

6. **Rebuild and test**:

```bash
maturin develop --features python
python PyRustTest/test_my_feature.py
```

---

## Project Structure

```
deepgraph/
├── src/                          # Rust source code
│   ├── lib.rs                    # Main library entry
│   ├── graph.rs                  # Core graph structures
│   ├── storage/                  # Storage backends
│   │   ├── memory.rs             # In-memory storage
│   │   └── columnar.rs           # Columnar storage
│   ├── algorithms/               # Graph algorithms
│   │   ├── traversal.rs          # BFS, DFS
│   │   ├── shortest_path.rs      # Dijkstra
│   │   └── ...
│   ├── python.rs                 # Python bindings (PyO3)
│   └── ...
├── python/                       # Python package
│   └── deepgraph/
│       └── __init__.py           # Python entry point
├── examples/                     # Example code
│   ├── rust/                     # Rust examples
│   └── python/                   # Python examples
├── PyRustTest/                   # Python test suite
│   ├── test_1_core_operations.py
│   ├── test_2_transactions.py
│   └── ...
├── Cargo.toml                    # Rust dependencies
├── pyproject.toml                # Python package config
└── README.md                     # Main documentation
```

---

## Code Style Guidelines

### Rust Code

```rust
// ✅ Good - follow Rust conventions
pub struct GraphStorage {
    nodes: Arc<DashMap<NodeId, Node>>,
}

impl GraphStorage {
    /// Create a new graph storage
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(DashMap::new()),
        }
    }
    
    /// Add a node to the graph
    pub fn add_node(&self, node: Node) -> Result<NodeId> {
        let id = node.id();
        self.nodes.insert(id, node);
        Ok(id)
    }
}

// ❌ Bad - inconsistent naming
pub struct graphStorage {  // Should be PascalCase
    Nodes: DashMap<NodeId, Node>,  // Should be snake_case
}
```

### Python Bindings

```rust
// ✅ Good - clear Python interface
#[pyclass]
pub struct PyGraphStorage {
    storage: Arc<RwLock<GraphStorage>>,
}

#[pymethods]
impl PyGraphStorage {
    /// Create a new graph storage
    #[new]
    fn new() -> Self {
        PyGraphStorage {
            storage: Arc::new(RwLock::new(GraphStorage::new())),
        }
    }
    
    /// Add a node with labels and properties
    /// 
    /// Args:
    ///     labels: List of string labels
    ///     properties: Dictionary of properties
    ///     
    /// Returns:
    ///     Node ID as string
    fn add_node(&self, labels: Vec<String>, properties: HashMap<String, PyObject>) -> PyResult<String> {
        // Implementation
    }
}
```

---

## Testing Guidelines

### Write Comprehensive Tests

```python
def test_feature_happy_path():
    """Test normal, expected behavior"""
    storage = deepgraph.GraphStorage()
    node = storage.add_node(["Test"], {"name": "test"})
    assert node is not None

def test_feature_edge_case_empty():
    """Test with empty input"""
    storage = deepgraph.GraphStorage()
    node = storage.add_node([], {})
    assert node is not None

def test_feature_error_handling():
    """Test error conditions"""
    storage = deepgraph.GraphStorage()
    try:
        storage.delete_node("invalid-id")
        assert False, "Should raise exception"
    except RuntimeError:
        pass  # Expected

def test_feature_stress():
    """Test with large dataset"""
    storage = deepgraph.GraphStorage()
    for i in range(1000):
        storage.add_node(["Test"], {"id": i})
    assert storage.node_count() == 1000
```

---

## Contributing

### Before Submitting PR

```bash
# 1. Update from main
git pull origin main

# 2. Run all tests
cargo test --lib
pytest PyRustTest/ -v

# 3. Check formatting
cargo fmt --all

# 4. Run linter
cargo clippy --all-targets --all-features -- -D warnings

# 5. Update documentation if needed
# Edit relevant .md files

# 6. Commit with clear message
git add .
git commit -m "feat: Add awesome new feature

- Implemented X
- Added tests for Y
- Updated documentation"

# 7. Push and create PR
git push origin feature/my-new-feature
```

### Commit Message Convention

```
<type>: <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `test`: Adding tests
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `chore`: Maintenance

**Example**:
```
feat: Add BFS traversal algorithm

Implemented breadth-first search with:
- Support for max depth limiting
- Distance tracking
- Parent node tracking

Closes #123
```

---

## Debugging

### Debug Python Bindings

```python
# Add debug prints in Rust code
println!("Debug: node_id = {:?}", node_id);

# Rebuild with debug symbols
maturin develop --features python

# Run with environment variables
RUST_BACKTRACE=1 python test_script.py
```

### Use Rust Debugger

```bash
# Build with debug symbols
cargo build

# Run with lldb/gdb
rust-lldb target/debug/deepgraph
```

---

# SRE/DevOps Quick Start

> **Goal**: Deploy, monitor, and operate DeepGraph in production

## Production Deployment

### Installation on Production Server

```bash
#!/bin/bash
# deploy_deepgraph.sh

set -e

# 1. Install system dependencies
sudo apt-get update
sudo apt-get install -y build-essential python3 python3-pip

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# 3. Clone repository
git clone https://github.com/deepskilling/deepgraph.git
cd deepgraph

# 4. Build release version
pip3 install maturin
maturin build --release --features python

# 5. Install wheel
pip3 install target/wheels/deepgraph-*.whl

# 6. Verify installation
python3 -c "import deepgraph; print(f'Installed: {deepgraph.__version__}')"

echo "✅ DeepGraph installed successfully"
```

### Docker Deployment

Create `Dockerfile`:

```dockerfile
# Dockerfile
FROM rust:1.75 as builder

# Install Python
RUN apt-get update && apt-get install -y python3 python3-pip

# Install maturin
RUN pip3 install maturin

# Copy source code
WORKDIR /app
COPY . .

# Build release version
RUN maturin build --release --features python

# Production image
FROM python:3.11-slim

# Copy built wheel
COPY --from=builder /app/target/wheels/*.whl /tmp/

# Install DeepGraph
RUN pip install /tmp/*.whl && rm /tmp/*.whl

# Create data directories
RUN mkdir -p /data/wal /data/indices /logs

# Set working directory
WORKDIR /app

# Healthcheck
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD python3 -c "import deepgraph; deepgraph.GraphStorage()" || exit 1

# Default command
CMD ["python3"]
```

Build and run:

```bash
# Build Docker image
docker build -t deepgraph:latest .

# Run container
docker run -d \
    --name deepgraph \
    -v $(pwd)/data:/data \
    -v $(pwd)/logs:/logs \
    deepgraph:latest \
    python3 your_app.py
```

### Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  deepgraph:
    build: .
    image: deepgraph:latest
    container_name: deepgraph-app
    restart: unless-stopped
    volumes:
      - ./data:/data
      - ./logs:/logs
      - ./app:/app
    environment:
      - DEEPGRAPH_STORAGE_DATA_DIR=/data
      - DEEPGRAPH_WAL_WAL_DIR=/data/wal
      - DEEPGRAPH_INDEX_INDEX_DIR=/data/indices
      - DEEPGRAPH_LOGGING_LEVEL=info
      - DEEPGRAPH_LOGGING_LOG_TO_FILE=true
      - DEEPGRAPH_LOGGING_LOG_DIR=/logs
    command: python3 /app/main.py
    healthcheck:
      test: ["CMD", "python3", "-c", "import deepgraph"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
```

Run with Docker Compose:

```bash
docker-compose up -d
docker-compose logs -f
```

---

## Configuration Management

### Production Configuration

Create `config.toml`:

```toml
# config.toml - Production Configuration

[storage]
data_dir = "/data"
cache_size_mb = 4096          # 4GB cache for production
enable_cache = true

[wal]
enabled = true
wal_dir = "/data/wal"
segment_size_mb = 128         # Larger segments for production
sync_on_write = true          # Durability over performance
checkpoint_threshold = 5000   # Checkpoint every 5000 operations

[index]
index_dir = "/data/indices"
auto_index = true
default_index_type = "BTree"  # Better for range queries

[algorithm]
pagerank_damping = 0.85
pagerank_max_iterations = 100
pagerank_tolerance = 0.0001
node2vec_walk_length = 80
node2vec_num_walks = 10

[logging]
level = "info"                # info for production, debug for troubleshooting
log_to_file = true
log_dir = "/logs"
```

### Environment Variables

Create `.env.production`:

```bash
# Storage
DEEPGRAPH_STORAGE_DATA_DIR=/data
DEEPGRAPH_STORAGE_CACHE_SIZE_MB=4096

# WAL
DEEPGRAPH_WAL_ENABLED=true
DEEPGRAPH_WAL_WAL_DIR=/data/wal
DEEPGRAPH_WAL_SYNC_ON_WRITE=true

# Logging
DEEPGRAPH_LOGGING_LEVEL=info
DEEPGRAPH_LOGGING_LOG_TO_FILE=true
DEEPGRAPH_LOGGING_LOG_DIR=/logs

# Performance
DEEPGRAPH_STORAGE_ENABLE_CACHE=true
```

Load in application:

```python
import os
from dotenv import load_dotenv
import deepgraph

# Load environment variables
load_dotenv('.env.production')

# Create storage with environment configuration
storage = deepgraph.GraphStorage()
wal = deepgraph.WAL(os.getenv('DEEPGRAPH_WAL_WAL_DIR', '/data/wal'))
```

---

## Monitoring & Observability

### Application Metrics

```python
# metrics.py
import time
import deepgraph
from prometheus_client import Counter, Histogram, Gauge, start_http_server

# Define metrics
node_operations = Counter('deepgraph_node_operations_total', 'Total node operations', ['operation'])
edge_operations = Counter('deepgraph_edge_operations_total', 'Total edge operations', ['operation'])
operation_duration = Histogram('deepgraph_operation_duration_seconds', 'Operation duration', ['operation'])
graph_size = Gauge('deepgraph_graph_size', 'Graph size', ['type'])

class MonitoredGraphStorage:
    def __init__(self):
        self.storage = deepgraph.GraphStorage()
    
    def add_node(self, labels, properties):
        start = time.time()
        try:
            result = self.storage.add_node(labels, properties)
            node_operations.labels(operation='add').inc()
            return result
        finally:
            duration = time.time() - start
            operation_duration.labels(operation='add_node').observe(duration)
            graph_size.labels(type='nodes').set(self.storage.node_count())
    
    def add_edge(self, from_node, to_node, rel_type, properties):
        start = time.time()
        try:
            result = self.storage.add_edge(from_node, to_node, rel_type, properties)
            edge_operations.labels(operation='add').inc()
            return result
        finally:
            duration = time.time() - start
            operation_duration.labels(operation='add_edge').observe(duration)
            graph_size.labels(type='edges').set(self.storage.edge_count())

# Start metrics server
if __name__ == '__main__':
    start_http_server(8000)
    print("Metrics available at http://localhost:8000")
```

### Logging Configuration

```python
# logging_config.py
import logging
import logging.handlers
import os

def setup_logging():
    """Configure production logging"""
    
    # Create logs directory
    log_dir = os.getenv('DEEPGRAPH_LOGGING_LOG_DIR', '/logs')
    os.makedirs(log_dir, exist_ok=True)
    
    # Configure root logger
    logger = logging.getLogger('deepgraph')
    logger.setLevel(logging.INFO)
    
    # File handler with rotation
    file_handler = logging.handlers.RotatingFileHandler(
        filename=f'{log_dir}/deepgraph.log',
        maxBytes=100 * 1024 * 1024,  # 100MB
        backupCount=10
    )
    file_handler.setLevel(logging.INFO)
    
    # Console handler
    console_handler = logging.StreamHandler()
    console_handler.setLevel(logging.WARNING)
    
    # Formatter
    formatter = logging.Formatter(
        '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )
    file_handler.setFormatter(formatter)
    console_handler.setFormatter(formatter)
    
    # Add handlers
    logger.addHandler(file_handler)
    logger.addHandler(console_handler)
    
    return logger

# Usage
logger = setup_logging()
logger.info("DeepGraph application started")
```

### Health Checks

```python
# healthcheck.py
import deepgraph
import sys

def health_check():
    """Production health check"""
    try:
        # Test basic operations
        storage = deepgraph.GraphStorage()
        
        # Test write
        node_id = storage.add_node(["HealthCheck"], {"timestamp": "now"})
        
        # Test read
        node = storage.get_node(node_id)
        if node is None:
            raise Exception("Failed to read node")
        
        # Test delete
        storage.delete_node(node_id)
        
        # Success
        print("✅ Health check passed")
        return 0
        
    except Exception as e:
        print(f"❌ Health check failed: {e}")
        return 1

if __name__ == '__main__':
    sys.exit(health_check())
```

Run health check:

```bash
# Kubernetes liveness probe
python3 healthcheck.py

# Or with curl (if running HTTP service)
curl -f http://localhost:8000/health || exit 1
```

---

## Backup & Recovery

### Backup Strategy

```bash
#!/bin/bash
# backup.sh - Backup DeepGraph data

set -e

BACKUP_DIR="/backups/deepgraph"
DATA_DIR="/data"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_NAME="deepgraph_backup_${TIMESTAMP}"

echo "Starting backup: ${BACKUP_NAME}"

# Create backup directory
mkdir -p "${BACKUP_DIR}"

# Backup data directory
tar -czf "${BACKUP_DIR}/${BACKUP_NAME}.tar.gz" \
    -C "${DATA_DIR}" \
    --exclude='*.tmp' \
    .

# Backup configuration
cp config.toml "${BACKUP_DIR}/${BACKUP_NAME}_config.toml"

# Calculate checksum
sha256sum "${BACKUP_DIR}/${BACKUP_NAME}.tar.gz" > "${BACKUP_DIR}/${BACKUP_NAME}.sha256"

# Keep only last 30 days of backups
find "${BACKUP_DIR}" -name "deepgraph_backup_*.tar.gz" -mtime +30 -delete

echo "✅ Backup complete: ${BACKUP_NAME}.tar.gz"
```

### Recovery Procedure

```bash
#!/bin/bash
# recover.sh - Recover from backup

set -e

if [ $# -ne 1 ]; then
    echo "Usage: $0 <backup_file>"
    exit 1
fi

BACKUP_FILE=$1
DATA_DIR="/data"
RECOVERY_DIR="/data_recovery"

echo "Starting recovery from: ${BACKUP_FILE}"

# Verify checksum
sha256sum -c "${BACKUP_FILE}.sha256"

# Stop application
echo "Stopping application..."
# systemctl stop deepgraph-app  # Adjust for your setup

# Backup current data (just in case)
if [ -d "${DATA_DIR}" ]; then
    mv "${DATA_DIR}" "${DATA_DIR}.pre_recovery.$(date +%s)"
fi

# Extract backup
mkdir -p "${RECOVERY_DIR}"
tar -xzf "${BACKUP_FILE}" -C "${RECOVERY_DIR}"

# Move to production location
mv "${RECOVERY_DIR}" "${DATA_DIR}"

# Start application
echo "Starting application..."
# systemctl start deepgraph-app

echo "✅ Recovery complete"
```

### WAL Recovery

```python
# wal_recovery.py
import deepgraph
import os

def recover_from_wal(wal_dir="/data/wal"):
    """Recover database from WAL"""
    
    print(f"Starting WAL recovery from: {wal_dir}")
    
    # Create new storage
    storage = deepgraph.GraphStorage()
    
    # Create recovery object
    recovery = deepgraph.WALRecovery(wal_dir)
    
    # Perform recovery
    recovered_ops = recovery.recover(storage)
    
    print(f"✅ Recovered {recovered_ops} operations")
    print(f"   Nodes: {storage.node_count()}")
    print(f"   Edges: {storage.edge_count()}")
    
    return storage

if __name__ == '__main__':
    storage = recover_from_wal()
```

---

## Performance Tuning

### Configuration for Different Workloads

#### High-Throughput Writes

```toml
[storage]
cache_size_mb = 8192           # Large cache

[wal]
sync_on_write = false          # Async for speed
segment_size_mb = 256          # Large segments
checkpoint_threshold = 10000   # Less frequent checkpoints
```

#### Read-Heavy Workload

```toml
[storage]
cache_size_mb = 16384          # Very large cache
enable_cache = true

[index]
auto_index = true              # Auto-create indices
default_index_type = "Hash"    # Fast O(1) lookups
```

#### Balanced (Default)

```toml
[storage]
cache_size_mb = 4096

[wal]
sync_on_write = true
checkpoint_threshold = 5000
```

### Monitoring Performance

```bash
# Check graph size
python3 -c "
import deepgraph
storage = deepgraph.GraphStorage()
print(f'Nodes: {storage.node_count()}')
print(f'Edges: {storage.edge_count()}')
"

# Monitor disk usage
du -sh /data/*

# Monitor memory usage
ps aux | grep python | grep deepgraph
```

---

## Kubernetes Deployment

### Deployment YAML

```yaml
# k8s-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: deepgraph
  labels:
    app: deepgraph
spec:
  replicas: 3
  selector:
    matchLabels:
      app: deepgraph
  template:
    metadata:
      labels:
        app: deepgraph
    spec:
      containers:
      - name: deepgraph
        image: deepgraph:latest
        ports:
        - containerPort: 8000
          name: metrics
        env:
        - name: DEEPGRAPH_STORAGE_DATA_DIR
          value: "/data"
        - name: DEEPGRAPH_LOGGING_LEVEL
          value: "info"
        volumeMounts:
        - name: data
          mountPath: /data
        - name: config
          mountPath: /config
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
        livenessProbe:
          exec:
            command:
            - python3
            - /app/healthcheck.py
          initialDelaySeconds: 30
          periodSeconds: 30
        readinessProbe:
          exec:
            command:
            - python3
            - /app/healthcheck.py
          initialDelaySeconds: 5
          periodSeconds: 10
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: deepgraph-data
      - name: config
        configMap:
          name: deepgraph-config
---
apiVersion: v1
kind: Service
metadata:
  name: deepgraph
spec:
  selector:
    app: deepgraph
  ports:
  - port: 8000
    targetPort: 8000
    name: metrics
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: deepgraph-data
spec:
  accessModes:
  - ReadWriteOnce
  resources:
    requests:
      storage: 100Gi
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: deepgraph-config
data:
  config.toml: |
    [storage]
    data_dir = "/data"
    cache_size_mb = 2048
    
    [wal]
    enabled = true
    wal_dir = "/data/wal"
    
    [logging]
    level = "info"
    log_to_file = true
```

Deploy to Kubernetes:

```bash
kubectl apply -f k8s-deployment.yaml
kubectl get pods -l app=deepgraph
kubectl logs -f deployment/deepgraph
```

---

## Troubleshooting Common Issues

### Issue: High Memory Usage

```bash
# Check current usage
python3 -c "
import deepgraph
storage = deepgraph.GraphStorage()
print(f'Graph size: {storage.node_count()} nodes, {storage.edge_count()} edges')
"

# Solution: Reduce cache size in config.toml
[storage]
cache_size_mb = 1024  # Reduce from default
```

### Issue: Slow Queries

```python
# Check if indices exist
idx_mgr = deepgraph.IndexManager()

# Create indices for frequently queried labels/properties
idx_mgr.create_hash_index("user_idx", "User")
idx_mgr.create_btree_index("age_idx", "age")
```

### Issue: Disk Space

```bash
# Check WAL size
du -sh /data/wal/

# Cleanup old WAL segments (after backup)
# Be careful! Only delete if you have backups
find /data/wal -name "*.log" -mtime +7 -delete
```

---

## Security Best Practices

### 1. File Permissions

```bash
# Secure data directory
chmod 750 /data
chown app:app /data

# Secure configuration
chmod 640 /config/config.toml
chown app:app /config/config.toml
```

### 2. Network Isolation

```yaml
# Docker network isolation
docker network create deepgraph-net

docker run -d \
  --name deepgraph \
  --network deepgraph-net \
  deepgraph:latest
```

### 3. Resource Limits

```python
# Set resource limits in application
import resource

# Limit memory
resource.setrlimit(resource.RLIMIT_AS, (4 * 1024 * 1024 * 1024, -1))  # 4GB

# Limit file descriptors
resource.setrlimit(resource.RLIMIT_NOFILE, (65536, 65536))
```

---

## Support & Resources

- 📖 [Full Documentation](README.md)
- 🐛 [Report Issues](https://github.com/deepskilling/deepgraph/issues)
- 💬 [Discussions](https://github.com/deepskilling/deepgraph/discussions)
- 📧 [Email Support](mailto:support@deepskilling.com)

---

**DeepGraph** - Production-Ready Graph Database  
© 2025 DeepSkilling. Licensed under MIT.

