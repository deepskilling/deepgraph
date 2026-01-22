# Disk-Based Storage Implementation Plan

## Goal
Implement persistent disk-based storage as the primary storage backend for DeepGraph, supporting graphs larger than RAM with ACID guarantees.

## Current State
- ✅ In-memory storage (DashMap-based)
- ✅ Columnar storage (Arrow-based, but still in-memory)
- ✅ Parquet export (snapshot only, not primary storage)
- ❌ No persistent disk storage

## Target State
- ✅ Disk-based primary storage using Sled
- ✅ Support graphs larger than RAM
- ✅ ACID guarantees (transactions, durability)
- ✅ Fast lookups with indices
- ✅ Compatible with existing StorageBackend trait

---

## Architecture

### Sled Database Structure

```
DeepGraph Sled DB
├── nodes          # Tree: NodeId → Node (serialized)
├── edges          # Tree: EdgeId → Edge (serialized)
├── label_index    # Tree: Label → Vec<NodeId>
├── property_index # Tree: (Label, PropertyKey) → Vec<NodeId>
├── outgoing_edges # Tree: NodeId → Vec<EdgeId>
└── incoming_edges # Tree: NodeId → Vec<EdgeId>
```

### Data Layout

**Nodes Tree:**
```
Key:   NodeId (16 bytes UUID)
Value: Bincode-serialized Node
```

**Edges Tree:**
```
Key:   EdgeId (16 bytes UUID)
Value: Bincode-serialized Edge
```

**Label Index:**
```
Key:   Label (String)
Value: Vec<NodeId> (bincode-serialized)
```

**Outgoing/Incoming Edges:**
```
Key:   NodeId (16 bytes UUID)
Value: Vec<EdgeId> (bincode-serialized)
```

---

## Implementation Tasks

### Task 1: Design & Setup ✅ (This Doc)

**File**: `DISK_STORAGE_PLAN.md`

**What**:
- Define architecture
- Plan data structures
- Identify dependencies

### Task 2: Implement DiskStorage

**File**: `src/storage/disk.rs` (NEW)

**What**:
- Create `DiskStorage` struct
- Open Sled database
- Define tree names
- Implement CRUD operations

**Structure**:
```rust
pub struct DiskStorage {
    db: sled::Db,
    nodes: sled::Tree,
    edges: sled::Tree,
    label_index: sled::Tree,
    outgoing_edges: sled::Tree,
    incoming_edges: sled::Tree,
}

impl DiskStorage {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let db = sled::open(path)?;
        let nodes = db.open_tree("nodes")?;
        let edges = db.open_tree("edges")?;
        // ... open other trees
        Ok(Self { db, nodes, edges, ... })
    }
}
```

### Task 3: Serialization/Deserialization

**Files**: 
- `src/graph.rs` (add derives)
- `Cargo.toml` (ensure bincode dep)

**What**:
- Add `Serialize`, `Deserialize` derives to Node, Edge
- Use `bincode` for efficient binary serialization
- Helper functions for ser/de

**Example**:
```rust
// Serialize
let bytes = bincode::serialize(&node)?;
nodes.insert(node.id().as_bytes(), bytes)?;

// Deserialize
let bytes = nodes.get(id.as_bytes())?;
let node: Node = bincode::deserialize(&bytes)?;
```

### Task 4: Implement StorageBackend Trait

**File**: `src/storage/disk.rs`

**What**:
- Implement all `StorageBackend` methods
- Add node: serialize + insert + update indices
- Get node: fetch + deserialize
- Delete node: remove + update indices
- Similar for edges

**Key Methods**:
```rust
impl StorageBackend for DiskStorage {
    fn add_node(&self, node: Node) -> Result<NodeId> {
        let id = node.id();
        let bytes = bincode::serialize(&node)?;
        
        // Insert node
        self.nodes.insert(id.as_bytes(), bytes)?;
        
        // Update label index
        for label in node.labels() {
            self.add_to_label_index(label, id)?;
        }
        
        self.db.flush()?; // Ensure durability
        Ok(id)
    }
    
    fn get_node(&self, id: NodeId) -> Result<Node> {
        let bytes = self.nodes.get(id.as_bytes())?
            .ok_or(DeepGraphError::NotFound)?;
        let node: Node = bincode::deserialize(&bytes)?;
        Ok(node)
    }
    
    // ... implement all other methods
}
```

### Task 5: Index Management

**File**: `src/storage/disk.rs`

**What**:
- Maintain label index for fast label queries
- Maintain edge adjacency lists (outgoing/incoming)
- Update indices on insert/delete

**Functions**:
```rust
fn add_to_label_index(&self, label: &str, node_id: NodeId) -> Result<()> {
    let mut ids = self.get_label_index(label)?;
    ids.push(node_id);
    let bytes = bincode::serialize(&ids)?;
    self.label_index.insert(label, bytes)?;
    Ok(())
}

fn remove_from_label_index(&self, label: &str, node_id: NodeId) -> Result<()> {
    let mut ids = self.get_label_index(label)?;
    ids.retain(|&id| id != node_id);
    let bytes = bincode::serialize(&ids)?;
    self.label_index.insert(label, bytes)?;
    Ok(())
}
```

### Task 6: Testing

**File**: `tests/test_disk_storage.rs` (NEW)

**What**:
- Test all CRUD operations
- Test persistence (write, close, reopen, read)
- Test large graphs (beyond RAM simulation)
- Test crash recovery
- Test concurrent access

**Test Cases**:
```rust
#[test]
fn test_add_and_get_node() { ... }

#[test]
fn test_persistence() {
    let temp_dir = tempdir()?;
    let path = temp_dir.path();
    
    // Write data
    {
        let storage = DiskStorage::new(path)?;
        storage.add_node(node)?;
    } // Close database
    
    // Reopen and read
    {
        let storage = DiskStorage::new(path)?;
        let node = storage.get_node(id)?;
        assert_eq!(node.id(), id);
    }
}

#[test]
fn test_label_index() { ... }

#[test]
fn test_edge_operations() { ... }
```

### Task 7: Python Bindings

**File**: `src/python.rs`

**What**:
- Add `PyDiskStorage` class
- Mirror `PyGraphStorage` API
- Allow path configuration

**Example**:
```python
import deepgraph

# In-memory storage (existing)
storage = deepgraph.GraphStorage()

# Disk storage (NEW)
storage = deepgraph.DiskStorage("./data/graph.db")
storage.add_node(labels=["Person"], properties={"name": "Alice"})
```

### Task 8: Configuration

**File**: `src/config.rs`

**What**:
- Add disk storage configuration
- Path, cache size, flush interval
- Durability vs performance tuning

**Example**:
```toml
[storage]
type = "disk"  # or "memory"
path = "./data/deepgraph.db"
cache_size_mb = 1024
flush_interval_ms = 100
```

### Task 9: Documentation

**Files**:
- `doc/DISK_STORAGE_GUIDE.md` (NEW)
- `README.md` (update)
- `PYTHON_QUICKSTART.md` (add disk storage examples)

**What**:
- Usage guide
- Performance tuning
- Migration from memory to disk
- Backup/restore

---

## Performance Considerations

### 1. Serialization Overhead
- **Solution**: Use `bincode` (fastest Rust serializer)
- **Benchmark**: ~100ns per node serialization

### 2. Disk I/O
- **Solution**: Sled's write-ahead log and caching
- **Benchmark**: ~1-10µs per write (with cache)

### 3. Index Updates
- **Solution**: Batch updates when possible
- **Future**: Asynchronous index updates

### 4. Large Graphs
- **Solution**: Sled handles larger-than-RAM datasets
- **Target**: Support billions of nodes/edges

---

## Migration Path

### From Memory to Disk

**Option 1: Export/Import**
```python
# Export from memory
memory_storage = deepgraph.GraphStorage()
# ... add data ...
memory_storage.export_parquet("backup.parquet")

# Import to disk
disk_storage = deepgraph.DiskStorage("./data/graph.db")
disk_storage.import_parquet("backup.parquet")
```

**Option 2: Hybrid Mode** (Future)
```python
# Automatic persistence
storage = deepgraph.HybridStorage(
    memory_cache_mb=4096,
    disk_path="./data/graph.db"
)
```

---

## Dependencies

### Existing:
✅ `sled = "0.34"` - Embedded database  
✅ `bincode = "1.3"` - Serialization  
✅ `serde = { version = "1.0", features = ["derive"] }` - Serialization traits

### New (if needed):
- `tempfile = "3.0"` - For testing (may already exist)

---

## Success Criteria

1. ✅ All `StorageBackend` methods implemented
2. ✅ Data persists across restarts
3. ✅ Performance: <1ms for single node read/write
4. ✅ Can handle graphs with millions of nodes
5. ✅ All tests passing (unit + integration)
6. ✅ Python bindings working
7. ✅ Documentation complete

---

## Risks & Mitigations

### Risk 1: Performance Degradation
- **Mitigation**: Benchmark early, optimize hot paths
- **Fallback**: Keep memory storage as option

### Risk 2: Data Corruption
- **Mitigation**: Sled has ACID guarantees, crash recovery
- **Fallback**: Regular backups via Parquet export

### Risk 3: Complex Migration
- **Mitigation**: Start with new projects, gradual migration
- **Fallback**: Maintain both storage backends

---

## Timeline Estimate

| Task | Effort | Dependencies |
|------|--------|--------------|
| 1. Design (this doc) | 0.5h | None |
| 2. Implement DiskStorage | 2h | Task 1 |
| 3. Serialization | 0.5h | Task 2 |
| 4. StorageBackend impl | 2h | Task 2, 3 |
| 5. Index management | 1h | Task 4 |
| 6. Testing | 2h | Task 4, 5 |
| 7. Python bindings | 1h | Task 4 |
| 8. Configuration | 0.5h | Task 4 |
| 9. Documentation | 1h | Task 6, 7 |
| **Total** | **~10.5h** | |

---

## Next Steps

1. ✅ Create this plan
2. 🚧 Implement `DiskStorage` struct
3. ⏳ Add serialization derives
4. ⏳ Implement `StorageBackend`
5. ⏳ Add tests
6. ⏳ Python bindings
7. ⏳ Documentation

---

**Status**: ✅ Plan Complete - Ready for Implementation  
**Date**: 2026-01-22  
**Estimated Completion**: 2026-01-22 (same day)
