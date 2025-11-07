# DeepGraph Storage - Quick Reference

## Is This an Embedded Database?

**YES!** DeepGraph is a fully embedded graph database, similar to SQLite.

- ✅ Runs directly in your application (no separate server)
- ✅ No network overhead - direct memory access
- ✅ Zero setup - just link the library
- ✅ Single process - database lives with your app

---

## Where Is Data Stored?

### Default Locations

```
./data/
├── wal/                    # Write-ahead logs (durability)
│   ├── 00000001.log
│   ├── 00000002.log
│   └── ...
│
├── snapshots/              # Point-in-time backups
│   └── snapshot_*/
│       ├── nodes.parquet
│       ├── edges.parquet
│       └── metadata.json
│
└── indices/                # Persistent B-tree indices
    ├── age_idx/
    └── name_idx/
```

---

## Storage Layers

### 1. **Primary: In-Memory (RAM)**
- All graph data lives in memory for ultra-fast access
- Uses concurrent hash maps (DashMap)
- 5M reads/sec, 1M writes/sec

### 2. **Durability: Write-Ahead Log (Disk)**
- **Location**: `./data/wal/` (configurable)
- **Format**: Binary log files (`.log`)
- **Purpose**: Crash recovery, ACID guarantees
- **Size**: ~100MB (rolling segments)

### 3. **Backup: Parquet Snapshots (Disk)**
- **Location**: `./data/snapshots/` (configurable)
- **Format**: Apache Parquet (columnar)
- **Purpose**: Point-in-time backups, fast restarts
- **Size**: ~200MB compressed (for 1M nodes)

### 4. **Indices: B-Tree (Disk)**
- **Location**: `./data/indices/` (configurable)
- **Format**: B-tree files
- **Purpose**: Fast range queries
- **Size**: ~50MB per index

---

## Configuration Examples

### Python - Default Location

```python
import deepgraph

# In-memory only (no persistence)
storage = deepgraph.GraphStorage()
```

### Python - With WAL (Crash Recovery)

```python
import deepgraph

# Configure WAL location
wal = deepgraph.WAL("./my_database/wal")

# Create storage
storage = deepgraph.GraphStorage()

# Operations are logged to disk
node_id = storage.add_node(["Person"], {"name": "Alice"})

# Flush to ensure durability
wal.flush()
```

### Python - Custom Location

```python
import deepgraph
import os

# Production setup
data_dir = "/var/lib/myapp/deepgraph"
os.makedirs(data_dir, exist_ok=True)

# Configure components
wal = deepgraph.WAL(f"{data_dir}/wal")
storage = deepgraph.GraphStorage()

# After crash - recover
recovery = deepgraph.WALRecovery(f"{data_dir}/wal")
recovered = recovery.recover(storage)
print(f"Recovered {recovered} operations")
```

### Rust - Full Configuration

```rust
use deepgraph::wal::{WAL, WALConfig};

let wal_config = WALConfig::new()
    .with_dir("./production_data/wal")
    .with_segment_size(128 * 1024 * 1024)  // 128MB
    .with_sync(true);                       // Durable

let wal = WAL::new(wal_config)?;
```

---

## Data Persistence Modes

| Mode | Setup | Use Case | Data Loss Risk |
|------|-------|----------|----------------|
| **In-Memory Only** | `storage = GraphStorage()` | Testing, temp data | High (all data) |
| **In-Memory + WAL** | `storage + WAL()` | Production ACID | Low (since last flush) |
| **Full Persistence** | `storage + WAL + Snapshots` | Production backups | None |

---

## Recovery After Crash

### Automatic Recovery

```python
import deepgraph

# Application restarts after crash
storage = deepgraph.GraphStorage()

# Recover from WAL
recovery = deepgraph.WALRecovery("./data/wal")
entries_recovered = recovery.recover(storage)

print(f"✅ Recovered {entries_recovered} operations")

# Continue normal operations
node = storage.add_node(["Person"], {"name": "Bob"})
```

---

## File Sizes (Typical)

For a graph with **1 million nodes** and **5 million edges**:

| Component | Size | Location |
|-----------|------|----------|
| **RAM** | ~500 MB | Memory |
| **WAL** | ~100 MB | `./data/wal/` |
| **Snapshots** | ~200 MB | `./data/snapshots/` |
| **Indices** | ~50 MB | `./data/indices/` |
| **Total Disk** | ~350 MB | `./data/` |

---

## Production Setup

### Recommended Directory Structure

```
/var/lib/myapp/
└── deepgraph/
    ├── wal/            # Write-ahead logs
    ├── snapshots/      # Backups (daily)
    └── indices/        # Persistent indices
```

### Docker Setup

```yaml
# docker-compose.yml
services:
  myapp:
    image: myapp:latest
    volumes:
      - deepgraph_data:/data
    environment:
      - DEEPGRAPH_WAL_DIR=/data/wal
      - DEEPGRAPH_SNAPSHOT_DIR=/data/snapshots

volumes:
  deepgraph_data:
```

### Environment Configuration

```python
import os
import deepgraph

# Read from environment
wal_dir = os.getenv("DEEPGRAPH_WAL_DIR", "./data/wal")
wal = deepgraph.WAL(wal_dir)
```

---

## Checking Storage

### Check WAL Size

```python
import os

def get_wal_size(wal_dir="./data/wal"):
    if not os.path.exists(wal_dir):
        return 0
    
    total = sum(
        os.path.getsize(os.path.join(wal_dir, f))
        for f in os.listdir(wal_dir)
        if f.endswith(".log")
    )
    return total

size_mb = get_wal_size() / 1024 / 1024
print(f"WAL size: {size_mb:.2f} MB")
```

### List Data Files

```bash
# View all DeepGraph data files
tree ./data/

# Output:
# ./data/
# ├── wal
# │   ├── 00000001.log
# │   └── 00000002.log
# ├── snapshots
# │   └── snapshot_20240101_120000
# │       ├── nodes.parquet
# │       ├── edges.parquet
# │       └── metadata.json
# └── indices
#     └── age_idx
```

---

## Migration Between Storage Locations

### Move Data to New Location

```python
import deepgraph
import shutil

# Old location
old_wal = "./old_data/wal"
old_storage = deepgraph.GraphStorage()
old_recovery = deepgraph.WALRecovery(old_wal)
old_recovery.recover(old_storage)

# Get all data
nodes = old_storage.get_all_nodes()
edges = old_storage.get_all_edges()

# New location
new_wal = "./new_data/wal"
new_storage = deepgraph.GraphStorage()
new_wal_instance = deepgraph.WAL(new_wal)

# Restore data
for node in nodes:
    new_storage.add_node(node["labels"], node["properties"])

for edge in edges:
    new_storage.add_edge(
        edge["from_id"],
        edge["to_id"],
        edge["relationship_type"],
        edge["properties"]
    )

new_wal_instance.flush()
print("✅ Migration complete")
```

---

## Common Questions

### Q: Do I need to manually save data?

**A**: If using WAL, data is automatically logged. Call `wal.flush()` to ensure durability.

### Q: What happens on crash?

**A**: Use `WALRecovery` to replay operations from WAL. All committed transactions are recovered.

### Q: Can multiple processes access the same database?

**A**: No, it's embedded (single process). Use separate database directories for each process.

### Q: How do I backup?

**A**: Copy the entire `./data/` directory. Or use snapshots for point-in-time backups.

### Q: Can I change the storage location?

**A**: Yes! Pass custom paths when creating WAL, indices, etc.

---

## Performance Tips

1. **Use WAL for Production**
   ```python
   wal = deepgraph.WAL("./data/wal")
   wal.flush()  # Call periodically
   ```

2. **Batch Operations**
   ```python
   tx = tx_manager.begin_transaction()
   # ... multiple operations ...
   tx_manager.commit_transaction(tx)
   ```

3. **Create Indices for Queries**
   ```python
   idx = deepgraph.IndexManager()
   idx.create_hash_index("person_idx", "Person")
   ```

4. **Monitor Disk Usage**
   ```python
   # Clean old WAL files after snapshots
   # Archive old snapshots
   ```

---

## Summary

| Question | Answer |
|----------|--------|
| **Is it embedded?** | ✅ YES - runs in your process |
| **Where is data?** | RAM (primary), `./data/` (disk) |
| **Default WAL location** | `./data/wal/` |
| **Default snapshots** | `./data/snapshots/` |
| **Can I change paths?** | ✅ YES - fully configurable |
| **Need server?** | ❌ NO - embedded database |
| **Crash recovery?** | ✅ YES - via WAL replay |
| **Multiple processes?** | ❌ NO - single process only |

---

**For detailed architecture, see [ARCHITECTURE.md](ARCHITECTURE.md)**

**DeepGraph** - Embedded High-Performance Graph Database  
© 2024 DeepSkilling

