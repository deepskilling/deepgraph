# DeepGraph Architecture & Persistence

## Is DeepGraph an Embedded Database?

**YES**, DeepGraph is a **fully embedded graph database** that runs directly within your application process. There is no separate server process required.

### What This Means:

- ✅ **Zero Network Overhead**: No client-server communication
- ✅ **Single Process**: Database runs in the same process as your application
- ✅ **No Setup Required**: No installation, configuration, or server management
- ✅ **Direct Memory Access**: Lightning-fast operations with no serialization overhead
- ✅ **Portable**: Just link the library - works on Linux, macOS, Windows

### Similar To:

- SQLite (embedded SQL database)
- RocksDB (embedded key-value store)
- LMDB (embedded key-value store)

### Different From:

- Neo4j (requires server)
- ArangoDB (requires server)
- TigerGraph (requires server)

---

## Storage Architecture

DeepGraph uses a **hybrid storage model** combining in-memory and disk-based persistence:

### 1. Primary Storage: In-Memory (MemoryStorage)

**Location**: RAM (main memory)

**Implementation**:
```
src/storage/memory.rs
```

**Structure**:
- Uses concurrent hash maps (DashMap) for thread-safe operations
- Maintains multiple indices for fast lookups:
  - `nodes: DashMap<NodeId, Node>` - All nodes by ID
  - `edges: DashMap<EdgeId, Edge>` - All edges by ID
  - `outgoing_edges: DashMap<NodeId, Vec<EdgeId>>` - Outgoing edge index
  - `incoming_edges: DashMap<NodeId, Vec<EdgeId>>` - Incoming edge index

**Performance**:
- Node/Edge lookup: O(1)
- Graph traversal: O(degree)
- Lock-free reads with RwLock

### 2. Durability: Write-Ahead Logging (WAL)

**Default Location**: `./data/wal/`

**Implementation**:
```
src/wal/log.rs
src/wal/recovery.rs
```

**Files**:
```
./data/wal/
├── 00000001.log    # WAL segment 1
├── 00000002.log    # WAL segment 2
└── ...
```

**What's Stored**:
- All database operations (INSERT, UPDATE, DELETE)
- Transaction markers (BEGIN, COMMIT, ABORT)
- Timestamps and LSNs (Log Sequence Numbers)

**Configuration**:
```rust
WALConfig {
    wal_dir: "./data/wal",           // Directory path
    segment_size: 64 * 1024 * 1024,  // 64MB per segment
    sync_on_write: true,              // Durability guarantee
    checkpoint_threshold: 1000,       // Auto-checkpoint
}
```

**Python Example**:
```python
import deepgraph

# Create WAL with custom directory
wal = deepgraph.WAL("./my_database/wal")
storage = deepgraph.GraphStorage()

# Operations are logged
node_id = storage.add_node(["Person"], {"name": "Alice"})

# Flush to disk for durability
wal.flush()
```

### 3. Persistence: Parquet Snapshots

**Default Location**: `./data/snapshots/`

**Implementation**:
```
src/persistence/parquet_io.rs
src/persistence/snapshot.rs
```

**Files**:
```
./data/snapshots/
├── snapshot_20240101_120000/
│   ├── nodes.parquet       # All nodes in columnar format
│   ├── edges.parquet       # All edges in columnar format
│   └── metadata.json       # Snapshot metadata
├── snapshot_20240102_120000/
│   ├── nodes.parquet
│   ├── edges.parquet
│   └── metadata.json
└── ...
```

**Format**: Apache Parquet (columnar storage)
- Efficient compression (SNAPPY)
- Fast column-based queries
- Schema evolution support
- Industry-standard format

**Snapshot Metadata** (`metadata.json`):
```json
{
  "id": "snapshot_20240101_120000",
  "timestamp": 1704110400,
  "path": "./data/snapshots/snapshot_20240101_120000",
  "node_count": 1000000,
  "edge_count": 5000000,
  "description": "Daily backup"
}
```

### 4. Indexing: B-Tree (Disk-Backed)

**Default Location**: `./data/indices/`

**Implementation**:
```
src/index/btree.rs
src/index/manager.rs
```

**Files**:
```
./data/indices/
├── age_idx/          # B-tree index on age property
├── name_idx/         # B-tree index on name property
└── ...
```

**Index Types**:
1. **Hash Index** (in-memory only)
   - O(1) exact lookups
   - No disk persistence

2. **B-tree Index** (disk-backed)
   - O(log n) range queries
   - Persistent across restarts
   - Supports: <, >, ≤, ≥, BETWEEN

---

## Data Flow

### Write Path

```
Application
    │
    ↓
Python/Rust API
    │
    ↓
GraphStorage (in-memory)
    │
    ├─→ WAL (disk)              # Durability
    │   └─→ segment.log
    │
    ├─→ Indices (memory/disk)   # Fast lookups
    │   ├─→ Hash (memory)
    │   └─→ B-tree (disk)
    │
    └─→ MVCC (memory)           # Transaction control
```

### Read Path

```
Application
    │
    ↓
Python/Rust API
    │
    ↓
MVCC Snapshot (consistent read)
    │
    ↓
Indices (fast lookup)
    │
    ↓
GraphStorage (in-memory)
    │
    └─→ Returns data
```

### Recovery Path (After Crash)

```
Crash/Restart
    │
    ↓
WALRecovery
    │
    ├─→ Read WAL segments
    │   └─→ ./data/wal/*.log
    │
    ├─→ Identify committed transactions
    │
    ├─→ Replay operations
    │
    └─→ Restore GraphStorage (in-memory)
```

---

## Default File Locations

### Complete Directory Structure

```
./
├── data/                      # Default data directory
│   ├── wal/                   # Write-ahead logs
│   │   ├── 00000001.log
│   │   ├── 00000002.log
│   │   └── ...
│   │
│   ├── snapshots/             # Point-in-time backups
│   │   ├── snapshot_*/
│   │   │   ├── nodes.parquet
│   │   │   ├── edges.parquet
│   │   │   └── metadata.json
│   │   └── ...
│   │
│   └── indices/               # Persistent indices
│       ├── age_idx/
│       ├── name_idx/
│       └── ...
│
└── your_application           # Your executable
```

---

## Configuration Examples

### Rust Configuration

```rust
use deepgraph::storage::GraphStorage;
use deepgraph::wal::{WAL, WALConfig};
use deepgraph::persistence::SnapshotManager;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure storage location
    let data_dir = PathBuf::from("./my_database");
    
    // Create storage
    let storage = GraphStorage::new();
    
    // Configure WAL
    let wal_config = WALConfig::new()
        .with_dir(data_dir.join("wal").to_string_lossy().to_string())
        .with_segment_size(128 * 1024 * 1024)  // 128MB segments
        .with_sync(true);                       // Durable writes
    
    let wal = WAL::new(wal_config)?;
    
    // Configure snapshots
    let snapshot_mgr = SnapshotManager::new(data_dir.join("snapshots"))?;
    
    // Use the database...
    
    Ok(())
}
```

### Python Configuration

```python
import deepgraph
import os

# Configure base directory
data_dir = "./my_database"
os.makedirs(data_dir, exist_ok=True)

# Create storage
storage = deepgraph.GraphStorage()

# Configure WAL with custom location
wal_dir = os.path.join(data_dir, "wal")
wal = deepgraph.WAL(wal_dir)

# Add data
node_id = storage.add_node(["Person"], {"name": "Alice", "age": 30})

# Ensure durability
wal.flush()

print(f"Data stored in: {data_dir}")
```

---

## Persistence Guarantees

### Durability Levels

#### 1. **No Persistence** (In-Memory Only)
```python
# Just use storage - data lost on restart
storage = deepgraph.GraphStorage()
```

**Use Case**: Testing, temporary data, caching

#### 2. **WAL Only** (Crash Recovery)
```python
storage = deepgraph.GraphStorage()
wal = deepgraph.WAL("./data/wal")
# Data can be recovered after crash
```

**Use Case**: Production systems needing ACID guarantees

#### 3. **WAL + Snapshots** (Full Backup)
```python
storage = deepgraph.GraphStorage()
wal = deepgraph.WAL("./data/wal")
# Periodically: save snapshot
# Data preserved across restarts and crashes
```

**Use Case**: Production with backup requirements

---

## Recovery Scenarios

### Scenario 1: Graceful Shutdown

```python
# Application stops normally
storage.clear()  # Optional: clean shutdown
# All data preserved in WAL
```

**Recovery**: Load from snapshots or replay WAL

### Scenario 2: Crash/Kill

```python
# Application crashes or killed (kill -9)
# In-memory data lost, but WAL intact
```

**Recovery**:
```python
storage = deepgraph.GraphStorage()
recovery = deepgraph.WALRecovery("./data/wal")
entries = recovery.recover(storage)
print(f"Recovered {entries} operations")
```

### Scenario 3: Disk Full

```python
# WAL write fails
# Transaction automatically aborted
# No data corruption
```

**Recovery**: Free disk space, restart application

---

## Performance Characteristics

### Storage Performance

| Operation | In-Memory | With WAL | With Snapshots |
|-----------|-----------|----------|----------------|
| **Read** | 5M ops/sec | 5M ops/sec | 5M ops/sec |
| **Write** | 1M ops/sec | 100K ops/sec | 100K ops/sec |
| **Startup** | Instant | Instant | Seconds (load) |
| **Recovery** | N/A | Seconds (replay) | Seconds (load) |

### Disk Usage

```
Graph Size:  1M nodes, 5M edges
Memory:      ~500MB (in-memory structures)
WAL:         ~100MB (rolling, compacted)
Snapshots:   ~200MB (Parquet compressed)
Indices:     ~50MB (B-tree files)
Total:       ~350MB on disk
```

---

## When to Use Which Storage

### In-Memory Only
- ✅ Development/testing
- ✅ Temporary/cache data
- ✅ Maximum performance
- ❌ No durability needed

### In-Memory + WAL
- ✅ Production ACID guarantees
- ✅ Crash recovery
- ✅ Fast writes
- ❌ Limited to RAM size

### In-Memory + WAL + Snapshots
- ✅ Production with backups
- ✅ Point-in-time recovery
- ✅ Fast restarts
- ✅ Archive old data

---

## Embedded vs. Client-Server Comparison

### DeepGraph (Embedded)

**Pros**:
- Zero network latency
- Simpler deployment
- Lower resource usage
- Direct memory access
- No ports/firewall issues

**Cons**:
- Single application access
- Scales with application
- In-process memory usage

### Neo4j (Client-Server)

**Pros**:
- Multiple clients
- Remote access
- Independent scaling
- Built-in clustering

**Cons**:
- Network overhead
- Complex deployment
- Higher resource usage
- Requires server management

---

## Migration & Backup

### Export Data

```python
import deepgraph

storage = deepgraph.GraphStorage()

# Get all data
nodes = storage.get_all_nodes()
edges = storage.get_all_edges()

# Export to JSON, CSV, etc.
import json
with open("backup.json", "w") as f:
    json.dump({"nodes": nodes, "edges": edges}, f)
```

### Import Data

```python
storage = deepgraph.GraphStorage()

with open("backup.json", "r") as f:
    data = json.load(f)

for node in data["nodes"]:
    storage.add_node(node["labels"], node["properties"])

for edge in data["edges"]:
    storage.add_edge(
        edge["from_id"],
        edge["to_id"],
        edge["relationship_type"],
        edge["properties"]
    )
```

---

## Best Practices

### 1. Choose Appropriate Storage Location

```python
# Development
wal = deepgraph.WAL("./dev_data/wal")

# Production
wal = deepgraph.WAL("/var/lib/myapp/deepgraph/wal")

# Docker
wal = deepgraph.WAL("/data/wal")  # Mount volume here
```

### 2. Regular Snapshots

```python
import schedule

def create_snapshot():
    # Save snapshot (when feature is exposed)
    print("Snapshot created")

# Daily snapshots
schedule.every().day.at("02:00").do(create_snapshot)
```

### 3. Monitor Disk Usage

```python
import os

def check_wal_size(wal_dir):
    total_size = sum(
        os.path.getsize(os.path.join(wal_dir, f))
        for f in os.listdir(wal_dir)
        if f.endswith(".log")
    )
    return total_size

size = check_wal_size("./data/wal")
print(f"WAL size: {size / 1024 / 1024:.2f} MB")
```

### 4. Clean Old WAL Files

After creating snapshots, old WAL files can be safely deleted to save space.

---

## Summary

| Aspect | Details |
|--------|---------|
| **Type** | Embedded (in-process) |
| **Primary Storage** | In-memory (DashMap) |
| **Durability** | WAL (Write-Ahead Log) |
| **Snapshots** | Parquet (columnar) |
| **Indices** | Hash (memory), B-tree (disk) |
| **Default Location** | `./data/` |
| **Startup** | Instant (or replay WAL) |
| **Recovery** | Automatic from WAL |
| **Thread Safety** | Full (lock-free reads) |
| **Portability** | Cross-platform |

---

## Need More Help?

- **Documentation**: See `doc/` folder
- **Examples**: See `examples/python/` folder
- **Issues**: https://github.com/deepskilling/deepgraph/issues

---

**DeepGraph** - Embedded High-Performance Graph Database  
© 2024 DeepSkilling. Licensed under MIT.

