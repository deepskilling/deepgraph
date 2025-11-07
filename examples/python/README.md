# DeepGraph Python Examples

This directory contains Python examples demonstrating how to use DeepGraph from Python.

## Prerequisites

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install Maturin** (Python build tool for Rust):
   ```bash
   pip install maturin
   ```

## Building the Python Package

From the project root directory:

```bash
# Development build
maturin develop --features python

# Release build
maturin build --release --features python
```

## Installing

```bash
# Install from local build
pip install .

# Or install in development mode
maturin develop --features python
```

## Running Examples

### Basic Usage
```bash
python examples/python/basic_usage.py
```

This example demonstrates:
- Creating a graph storage
- Adding nodes with labels and properties
- Creating edges between nodes
- Querying nodes and edges
- Finding nodes by label
- Using transactions

### Social Network
```bash
python examples/python/social_network.py
```

This example builds a complete social network with:
- User profiles
- Friendships
- Posts and likes
- Groups and memberships
- Statistics and queries

## Example Code

### Quick Start

```python
import deepgraph

# Create a graph
storage = deepgraph.GraphStorage()

# Add a node
node_id = storage.add_node(
    labels=["Person"],
    properties={
        "name": "Alice",
        "age": 30
    }
)

# Add an edge
edge_id = storage.add_edge(
    from_id=node_id,
    to_id=other_node_id,
    label="KNOWS",
    properties={"since": 2020}
)

# Query the graph
node = storage.get_node(node_id)
people = storage.find_nodes_by_label("Person")
```

### Using Transactions

```python
import deepgraph

storage = deepgraph.GraphStorage()
txn_manager = deepgraph.TransactionManager()

# Begin transaction
txn_id = txn_manager.begin_transaction()

# Perform operations
node_id = storage.add_node(["Person"], {"name": "Bob"})

# Commit or abort
txn_manager.commit_transaction(txn_id)
# or
# txn_manager.abort_transaction(txn_id)
```

## API Documentation

See the main [API documentation](../../doc/API.md) for detailed information about the Python API.

## Performance

The Python bindings use PyO3 to provide zero-cost abstractions over the Rust implementation, giving you:

- 🚀 Native Rust performance
- 🔒 Memory safety guarantees
- ⚡ Lock-free concurrent reads
- 💾 Full ACID compliance

## Troubleshooting

### Import Error

If you get an import error, make sure you've built and installed the package:
```bash
maturin develop --features python
```

### Build Errors

If you encounter build errors, ensure you have:
- Latest Rust toolchain: `rustup update`
- Python 3.8 or later
- Maturin installed: `pip install maturin`

## Contributing

Found a bug or have a suggestion? Please open an issue at:
https://github.com/deepskilling/deepgraph/issues

