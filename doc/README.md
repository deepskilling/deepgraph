# DeepGraph Documentation

Welcome to the DeepGraph documentation! This guide will help you navigate all available documentation based on your needs.

---

## 📚 Documentation Index

### Getting Started

1. **[Getting Started Guide](GETTING_STARTED.md)**
   - Quick introduction to DeepGraph
   - Basic concepts
   - Installation instructions
   - First steps

2. **[Python Quick Start](PYTHON_QUICKSTART.md)** ⭐ **Recommended for Beginners**
   - Comprehensive Python API guide
   - End-user, developer, and SRE sections
   - Code examples for all features
   - 1,700+ lines of tutorials

---

### Core Features

3. **[Cypher Query Guide](CYPHER_GUIDE.md)** 🔍
   - Complete Cypher syntax reference
   - Query examples (MATCH, WHERE, RETURN)
   - Best practices and patterns
   - Performance tips
   - 800+ lines, 30+ examples

4. **[Disk Storage Guide](DISK_STORAGE_GUIDE.md)** 💾
   - Persistent storage with Sled
   - Configuration options
   - Performance tuning
   - Backup and recovery
   - Python and Rust APIs

5. **[Data Import Guide](IMPORT_GUIDE.md)** 📥
   - CSV import (automatic type inference)
   - JSON import (type preservation)
   - Bulk loading strategies
   - Error handling
   - Performance optimization

6. **[CLI/REPL Guide](CLI_GUIDE.md)** 💻
   - Interactive REPL usage
   - Non-interactive mode
   - Output formats (table, JSON, CSV)
   - Meta commands
   - Data import from CLI

---

### Development

7. **[Contributing Guide](CONTRIBUTING.md)**
   - How to contribute
   - Code style guidelines
   - Pull request process
   - Testing requirements

---

### Reference

8. **[Kuzu Comparison](KUZU_COMPARISON.md)**
   - Feature comparison with Kuzu
   - Unique DeepGraph features
   - Roadmap alignment

9. **[Phase 4 Completion Summary](PHASE4_COMPLETE.md)**
   - Phase 4 milestone achievements
   - Code statistics
   - Test coverage
   - Architecture overview

---

## 🎯 Quick Navigation by Role

### For End Users (Python)

**Start Here:**
1. [Python Quick Start](PYTHON_QUICKSTART.md) - Comprehensive tutorial
2. [Cypher Query Guide](CYPHER_GUIDE.md) - Learn to write queries
3. [Import Guide](IMPORT_GUIDE.md) - Load your data
4. [CLI Guide](CLI_GUIDE.md) - Interactive queries

**Core Workflow:**
```python
import deepgraph

# Create storage
storage = deepgraph.DiskStorage("./mydb.db")

# Import data
storage.import_csv_nodes("nodes.csv")
storage.import_csv_edges("edges.csv")

# Run queries
result = storage.execute_cypher("MATCH (n:Person) WHERE n.age > 25 RETURN n;")
print(result)
```

---

### For CLI Users

**Start Here:**
1. [CLI Guide](CLI_GUIDE.md) - Complete CLI reference
2. [Cypher Query Guide](CYPHER_GUIDE.md) - Query syntax

**Quick Commands:**
```bash
# Interactive REPL
./target/release/deepgraph-cli --database ./mydb.db

# Single query
./target/release/deepgraph-cli -d ./mydb.db -q "MATCH (n) RETURN n"

# Import data
./target/release/deepgraph-cli --database ./mydb.db \
    --import-csv-nodes nodes.csv \
    --import-csv-edges edges.csv
```

---

### For Developers (Rust)

**Start Here:**
1. [Getting Started](GETTING_STARTED.md) - Basic setup
2. [Contributing Guide](CONTRIBUTING.md) - Development workflow

**Core APIs:**
```rust
use deepgraph::{DiskStorage, StorageBackend};

// Create storage
let storage = DiskStorage::new("./mydb.db")?;

// Add nodes
let mut node = Node::new(vec!["Person".to_string()]);
node.set_property("name".to_string(), "Alice".into());
let id = storage.add_node(node)?;

// Query
let parser = CypherParser::new();
let ast = parser.parse("MATCH (n:Person) RETURN n")?;
```

---

### For SREs/Operators

**Start Here:**
1. [Disk Storage Guide](DISK_STORAGE_GUIDE.md) - Configuration
2. [Python Quick Start (SRE Section)](PYTHON_QUICKSTART.md#for-sres-and-operators) - Operations

**Key Topics:**
- Configuration (`config.toml`)
- Backup and recovery
- Performance tuning
- Monitoring
- Troubleshooting

---

## 📖 Documentation by Feature

### Phase 1-3: Core Database

| Feature | Description | Documentation |
|---------|-------------|---------------|
| **Storage** | In-memory and disk-based storage | [Disk Storage Guide](DISK_STORAGE_GUIDE.md) |
| **Transactions** | ACID guarantees, MVCC | [Python Quick Start](PYTHON_QUICKSTART.md) |
| **Indexing** | Hash and B-tree indices | [Python Quick Start](PYTHON_QUICKSTART.md) |
| **WAL & Recovery** | Write-ahead log, crash recovery | [Disk Storage Guide](DISK_STORAGE_GUIDE.md) |
| **Algorithms** | BFS, DFS, Dijkstra, PageRank, etc. | [Python Quick Start](PYTHON_QUICKSTART.md) |

### Phase 4: Critical Features ✅

| Feature | Description | Documentation |
|---------|-------------|---------------|
| **Cypher Execution** | Query parser, planner, executor | [Cypher Guide](CYPHER_GUIDE.md) |
| **Disk Storage** | Persistent, ACID-compliant storage | [Disk Storage Guide](DISK_STORAGE_GUIDE.md) |
| **CSV/JSON Import** | Data loading capabilities | [Import Guide](IMPORT_GUIDE.md) |
| **REPL/CLI** | Interactive command-line interface | [CLI Guide](CLI_GUIDE.md) |

---

## 🔍 Common Tasks

### How do I...

**...create a graph database?**
- Python: [Python Quick Start → Getting Started](PYTHON_QUICKSTART.md#getting-started)
- CLI: [CLI Guide → Quick Start](CLI_GUIDE.md#quick-start)

**...add nodes and edges?**
- Python: [Python Quick Start → Core Operations](PYTHON_QUICKSTART.md#core-operations)
- CLI: [Import Guide → CSV Import](IMPORT_GUIDE.md#csv-import)

**...run queries?**
- [Cypher Query Guide](CYPHER_GUIDE.md)

**...import data from CSV/JSON?**
- [Import Guide](IMPORT_GUIDE.md)

**...make data persistent?**
- [Disk Storage Guide](DISK_STORAGE_GUIDE.md)

**...use the interactive CLI?**
- [CLI Guide](CLI_GUIDE.md)

**...run graph algorithms?**
- [Python Quick Start → Graph Algorithms](PYTHON_QUICKSTART.md#graph-algorithms)

**...handle transactions?**
- [Python Quick Start → Transactions](PYTHON_QUICKSTART.md#transactions)

**...configure DeepGraph?**
- [Disk Storage Guide → Configuration](DISK_STORAGE_GUIDE.md#configuration)

**...optimize performance?**
- [Cypher Guide → Performance](CYPHER_GUIDE.md#performance)
- [Disk Storage Guide → Performance](DISK_STORAGE_GUIDE.md#performance)

---

## 📊 Documentation Statistics

| Document | Lines | Topics | Examples |
|----------|-------|--------|----------|
| Python Quick Start | 1,742 | 12 | 50+ |
| Cypher Guide | 800+ | 10 | 30+ |
| Disk Storage Guide | 580+ | 11 | 10+ |
| Import Guide | 700+ | 12 | 15+ |
| CLI Guide | 700+ | 10 | 20+ |
| **Total** | **4,500+** | **55** | **125+** |

---

## 🚀 Getting Help

### Resources

- **GitHub Issues**: https://github.com/deepskilling/deepgraph/issues
- **Documentation**: https://github.com/deepskilling/deepgraph/tree/main/doc
- **Examples**: `examples/python/` and `examples/rust/`

### Common Issues

See the troubleshooting sections in:
- [Cypher Guide → Troubleshooting](CYPHER_GUIDE.md#troubleshooting)
- [Disk Storage Guide → Troubleshooting](DISK_STORAGE_GUIDE.md#troubleshooting)
- [Import Guide → Troubleshooting](IMPORT_GUIDE.md#troubleshooting)
- [CLI Guide → Troubleshooting](CLI_GUIDE.md#troubleshooting)

---

## 📝 Documentation Updates

This documentation is actively maintained. For the latest updates:

- Check the [CHANGELOG](../CHANGELOG.md)
- Review recent commits
- See [PHASE4_COMPLETE.md](PHASE4_COMPLETE.md) for Phase 4 summary

---

## 🎓 Learning Path

### Beginner Path
1. [Getting Started](GETTING_STARTED.md) - 10 min
2. [Python Quick Start](PYTHON_QUICKSTART.md) - 1 hour
3. [Cypher Guide](CYPHER_GUIDE.md) - 30 min
4. [CLI Guide](CLI_GUIDE.md) - 20 min

### Intermediate Path
1. [Import Guide](IMPORT_GUIDE.md) - Load real data
2. [Disk Storage Guide](DISK_STORAGE_GUIDE.md) - Configure persistence
3. [Cypher Guide → Advanced](CYPHER_GUIDE.md) - Complex queries

### Advanced Path
1. [Contributing Guide](CONTRIBUTING.md) - Contribute code
2. [Phase 4 Complete](PHASE4_COMPLETE.md) - Architecture deep dive
3. Review source code in `src/`

---

## 🔗 Quick Links

### GitHub
- [Main Repository](https://github.com/deepskilling/deepgraph)
- [Issues](https://github.com/deepskilling/deepgraph/issues)
- [Releases](https://github.com/deepskilling/deepgraph/releases)

### Documentation Files
- [../README.md](../README.md) - Project overview and roadmap
- [../CHANGELOG.md](../CHANGELOG.md) - Version history
- [../CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md) - Community guidelines
- [../SECURITY.md](../SECURITY.md) - Security policy

---

**Happy graphing with DeepGraph! 🚀**
