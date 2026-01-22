# DeepGraph Data Import Guide

**Easily load graph data from CSV and JSON files**

## Table of Contents

1. [Overview](#overview)
2. [Supported Formats](#supported-formats)
3. [CSV Import](#csv-import)
4. [JSON Import](#json-import)
5. [Python API](#python-api)
6. [Rust API](#rust-api)
7. [Type Inference](#type-inference)
8. [Performance](#performance)
9. [Error Handling](#error-handling)
10. [Best Practices](#best-practices)
11. [Examples](#examples)
12. [Troubleshooting](#troubleshooting)

---

## Overview

DeepGraph provides powerful data import capabilities for loading graph data from CSV and JSON files. Key features:

- ✅ **CSV Import** - Parse CSV files with automatic type inference
- ✅ **JSON Import** - Import JSON arrays with type preservation
- ✅ **Bulk Loading** - Efficient batch processing
- ✅ **Error Recovery** - Skip invalid records and continue
- ✅ **ID Mapping** - External→Internal ID mapping
- ✅ **Progress Tracking** - Statistics and timing
- ✅ **Python + Rust APIs** - Easy to use from both languages

---

## Supported Formats

### CSV Format

**Nodes:**
```csv
id,labels,name,age,city
1,Person,Alice,30,NYC
2,Person;Employee,Bob,25,SF
3,Company,Acme Corp,,NYC
```

**Edges:**
```csv
from,to,type,since,weight
1,2,KNOWS,2020,0.8
1,3,WORKS_AT,2019,1.0
```

### JSON Format

**Nodes:**
```json
[
  {
    "id": "1",
    "labels": ["Person", "Employee"],
    "properties": {
      "name": "Alice",
      "age": 30,
      "city": "NYC"
    }
  }
]
```

**Edges:**
```json
[
  {
    "from": "1",
    "to": "2",
    "type": "KNOWS",
    "properties": {
      "since": 2020,
      "weight": 0.8
    }
  }
]
```

---

## CSV Import

### CSV Node Format

**Required Columns:**
- `labels` - Semicolon-separated labels (e.g., `"Person;Employee"`)

**Optional Columns:**
- `id` - External node ID (auto-generated if missing)
- Any other columns become properties

**Example:**
```csv
id,labels,name,age,email,active
1,Person,Alice,30,alice@example.com,true
2,Person,Bob,25,bob@example.com,false
```

### CSV Edge Format

**Required Columns:**
- `from` (or `source`, `src`) - Source node external ID
- `to` (or `target`, `dst`) - Target node external ID
- `type` (or `relationship`, `label`) - Relationship type

**Optional Columns:**
- Any other columns become edge properties

**Example:**
```csv
from,to,type,since,weight
1,2,KNOWS,2020,0.8
2,3,WORKS_AT,2021,1.0
```

### CSV Configuration

**Delimiter:**
```python
# Custom delimiter (default: comma)
importer = deepgraph.CsvImporter()
importer.with_delimiter(';')  # Rust API only
```

**Label Separator:**
```python
# Custom label separator (default: semicolon)
importer.with_label_separator('|')  # Rust API only
```

**Header:**
```python
# No header row
importer.with_header(False)  # Rust API only
```

---

## JSON Import

### JSON Node Format

**Structure:**
```json
{
  "id": "unique_id",         // Optional (auto-generated if missing)
  "labels": ["Label1", "Label2"],  // Array of labels
  "properties": {            // Properties object
    "name": "value",
    "age": 30
  }
}
```

**Example:**
```json
[
  {
    "id": "alice",
    "labels": ["Person", "Employee"],
    "properties": {
      "name": "Alice",
      "age": 30,
      "department": "Engineering",
      "salary": 120000.50,
      "active": true
    }
  },
  {
    "id": "bob",
    "labels": ["Person"],
    "properties": {
      "name": "Bob",
      "age": 25,
      "active": false
    }
  }
]
```

### JSON Edge Format

**Structure:**
```json
{
  "from": "source_id",       // Required
  "to": "target_id",         // Required
  "type": "RELATIONSHIP",    // Required
  "properties": {            // Optional
    "weight": 0.8,
    "since": 2020
  }
}
```

**Example:**
```json
[
  {
    "from": "alice",
    "to": "bob",
    "type": "KNOWS",
    "properties": {
      "since": 2020,
      "strength": 0.9
    }
  },
  {
    "from": "bob",
    "to": "acme",
    "type": "WORKS_AT",
    "properties": {
      "position": "Engineer",
      "start_date": "2021-01-01"
    }
  }
]
```

---

## Python API

### Basic Usage

```python
import deepgraph

# Create storage
storage = deepgraph.GraphStorage()  # In-memory
# OR
storage = deepgraph.DiskStorage("./my_graph.db")  # Persistent

# Import CSV
node_stats = storage.import_csv_nodes("nodes.csv")
edge_stats = storage.import_csv_edges("edges.csv", node_stats['node_id_map'])

# Import JSON
node_stats = storage.import_json_nodes("nodes.json")
edge_stats = storage.import_json_edges("edges.json", node_stats['node_id_map'])
```

### Import Statistics

```python
stats = storage.import_csv_nodes("nodes.csv")

print(f"Nodes imported: {stats['nodes_imported']}")
print(f"Duration: {stats['duration_ms']}ms")
print(f"Errors: {len(stats['errors'])}")

# Node ID mapping (external → internal)
for external_id, internal_id in stats['node_id_map'].items():
    print(f"{external_id} → {internal_id}")

# Error details
for error in stats['errors']:
    print(f"Error: {error}")
```

### Complete Example

```python
import deepgraph

# Create storage
storage = deepgraph.DiskStorage("./social_network.db")

# Import employees
print("Importing employees...")
node_stats = storage.import_csv_nodes("employees.csv")
print(f"✅ Imported {node_stats['nodes_imported']} employees")

# Import relationships
print("Importing relationships...")
edge_stats = storage.import_csv_edges(
    "relationships.csv",
    node_stats['node_id_map']
)
print(f"✅ Imported {edge_stats['edges_imported']} relationships")

# Query the data
result = storage.execute_cypher("""
    MATCH (e:Employee)
    WHERE e.department = 'Engineering'
    RETURN e.name, e.salary
""")

for row in result['rows']:
    print(f"{row['name']}: ${row['salary']}")
```

---

## Rust API

### CSV Import

```rust
use deepgraph::import::{CsvImporter, ImportConfig};
use deepgraph::storage::MemoryStorage;

let storage = MemoryStorage::new();

// Create importer
let importer = CsvImporter::new()
    .with_config(ImportConfig::new()
        .with_batch_size(1000)
        .with_flush_interval(5000)
        .with_skip_invalid(true)
        .with_max_errors(100));

// Import nodes
let stats = importer.import_nodes(&storage, "nodes.csv")?;
println!("Imported {} nodes", stats.nodes_imported);

// Import edges
let stats = importer.import_edges(&storage, "edges.csv", &stats.node_id_map)?;
println!("Imported {} edges", stats.edges_imported);
```

### JSON Import

```rust
use deepgraph::import::JsonImporter;

let importer = JsonImporter::new();

// Import nodes
let stats = importer.import_nodes(&storage, "nodes.json")?;

// Import edges
let stats = importer.import_edges(&storage, "edges.json", &stats.node_id_map)?;
```

### Configuration

```rust
use deepgraph::import::ImportConfig;

let config = ImportConfig::new()
    .with_batch_size(5000)         // Batch size for bulk operations
    .with_flush_interval(10000)    // Flush to disk every N records
    .with_skip_invalid(true)       // Skip invalid records
    .with_max_errors(50);          // Abort after N errors
```

---

## Type Inference

### CSV Type Inference

CSV values are automatically converted to appropriate types:

| CSV Value | Inferred Type | Example |
|-----------|---------------|---------|
| `true`, `false` | Boolean | `true` → Boolean(true) |
| `42`, `-100` | Integer | `42` → Integer(42) |
| `3.14`, `-2.5` | Float | `3.14` → Float(3.14) |
| `hello`, `123abc` | String | `hello` → String("hello") |

**Example CSV:**
```csv
id,labels,name,age,salary,active,rating
1,Person,Alice,30,75000.50,true,4.5
```

**Inferred Types:**
- `name`: String ("Alice")
- `age`: Integer (30)
- `salary`: Float (75000.50)
- `active`: Boolean (true)
- `rating`: Float (4.5)

### JSON Type Preservation

JSON types are preserved as-is:

| JSON Type | PropertyValue |
|-----------|---------------|
| `null` | Null |
| `true`/`false` | Boolean |
| Number (int) | Integer |
| Number (float) | Float |
| String | String |
| Array/Object | String (serialized) |

**Example:**
```json
{
  "name": "Alice",      // String
  "age": 30,            // Integer
  "salary": 75000.50,   // Float
  "active": true,       // Boolean
  "tags": ["a", "b"]    // String (serialized)
}
```

---

## Performance

### Benchmarks

**Hardware**: M1 Mac, 16GB RAM, SSD

| Operation | Records | Time | Rate |
|-----------|---------|------|------|
| CSV nodes import | 10,000 | 85ms | 117K/sec |
| CSV edges import | 10,000 | 92ms | 108K/sec |
| JSON nodes import | 10,000 | 120ms | 83K/sec |
| JSON edges import | 10,000 | 130ms | 76K/sec |

### Performance Tips

**1. Batch Size**
- Small files (<10K): Default (1000) is fine
- Medium files (10K-1M): 5000-10000
- Large files (>1M): 10000-50000

**2. Flush Interval**
- Development: 1000 (frequent flush)
- Production: 5000 (default)
- Bulk loading: 10000+ (less frequent flush)

**3. Error Handling**
- Skip invalid: true (faster, continues on errors)
- Skip invalid: false (slower, fails on first error)

**4. File Format**
- CSV: Faster parsing, requires type inference
- JSON: Slower parsing, preserves types

**Example Configuration:**
```rust
let config = ImportConfig::new()
    .with_batch_size(10000)     // Large batches
    .with_flush_interval(50000) // Infrequent flush
    .with_skip_invalid(true)    // Continue on errors
    .with_max_errors(1000);     // Allow many errors
```

---

## Error Handling

### Error Types

1. **Parse Errors** - Invalid CSV/JSON format
2. **Type Errors** - Cannot convert value
3. **Missing Data** - Required column/field missing
4. **Node Not Found** - External ID not in map
5. **Storage Errors** - Database operation failed

### Error Recovery

**Skip Invalid Records:**
```python
# Python: Default behavior
stats = storage.import_csv_nodes("nodes.csv")

# Check errors
if stats['errors']:
    print(f"Encountered {len(stats['errors'])} errors:")
    for error in stats['errors']:
        print(f"  - {error}")
```

**Stop on First Error:**
```rust
// Rust
let config = ImportConfig::new()
    .with_skip_invalid(false);  // Fail on first error

let importer = CsvImporter::new().with_config(config);
```

### Error Examples

**Missing Required Column:**
```
Error: Missing 'from' column in edges CSV
```

**Node Not Found:**
```
Error: Node '999' not found in ID map
```

**Invalid CSV:**
```
Error: CSV parse error: unexpected field count at line 42
```

---

## Best Practices

### 1. Always Import Nodes Before Edges

```python
# ✅ Correct order
node_stats = storage.import_csv_nodes("nodes.csv")
edge_stats = storage.import_csv_edges("edges.csv", node_stats['node_id_map'])

# ❌ Wrong order (will fail)
edge_stats = storage.import_csv_edges("edges.csv", {})
```

### 2. Use Consistent IDs

```csv
# nodes.csv
id,labels,name
emp_1,Employee,Alice
emp_2,Employee,Bob

# edges.csv
from,to,type
emp_1,emp_2,KNOWS  ✅ Matches node IDs
```

### 3. Handle Errors Gracefully

```python
stats = storage.import_csv_nodes("nodes.csv")

if stats['errors']:
    print(f"Warning: {len(stats['errors'])} records failed")
    # Log errors to file
    with open("import_errors.log", "w") as f:
        for error in stats['errors']:
            f.write(f"{error}\n")

# Continue with successful imports
print(f"Successfully imported {stats['nodes_imported']} nodes")
```

### 4. Validate Data Before Import

```python
import csv

# Check CSV has required columns
with open("nodes.csv") as f:
    reader = csv.DictReader(f)
    headers = reader.fieldnames
    
    if 'labels' not in headers:
        print("Error: 'labels' column missing!")
    
    # Check data types
    for row in reader:
        if not row['age'].isdigit():
            print(f"Warning: Invalid age for {row['name']}")
```

### 5. Use Disk Storage for Large Imports

```python
# For large datasets, use disk storage
storage = deepgraph.DiskStorage("./large_graph.db")

# Import in batches if needed
for i in range(0, 10):
    batch_file = f"nodes_batch_{i}.csv"
    stats = storage.import_csv_nodes(batch_file)
    print(f"Batch {i}: {stats['nodes_imported']} nodes")
```

---

## Examples

### Example 1: Social Network

**nodes.csv:**
```csv
id,labels,name,age,city
1,Person,Alice,30,NYC
2,Person,Bob,25,SF
3,Person,Charlie,35,LA
```

**edges.csv:**
```csv
from,to,type,since
1,2,FRIENDS,2020
2,3,FRIENDS,2021
1,3,FRIENDS,2019
```

**Import:**
```python
import deepgraph

storage = deepgraph.GraphStorage()

# Import
node_stats = storage.import_csv_nodes("nodes.csv")
edge_stats = storage.import_csv_edges("edges.csv", node_stats['node_id_map'])

# Query
result = storage.execute_cypher("""
    MATCH (p:Person)
    WHERE p.age > 25
    RETURN p.name, p.age
""")

for row in result['rows']:
    print(f"{row['name']}: {row['age']} years old")
```

### Example 2: Company Hierarchy

**employees.json:**
```json
[
  {"id": "e1", "labels": ["Employee"], "properties": {"name": "Alice", "title": "CEO"}},
  {"id": "e2", "labels": ["Employee"], "properties": {"name": "Bob", "title": "CTO"}},
  {"id": "e3", "labels": ["Employee"], "properties": {"name": "Charlie", "title": "Engineer"}}
]
```

**reports_to.json:**
```json
[
  {"from": "e2", "to": "e1", "type": "REPORTS_TO"},
  {"from": "e3", "to": "e2", "type": "REPORTS_TO"}
]
```

**Import:**
```python
import deepgraph

storage = deepgraph.DiskStorage("./company.db")

# Import
node_stats = storage.import_json_nodes("employees.json")
edge_stats = storage.import_json_edges("reports_to.json", node_stats['node_id_map'])

print(f"Imported {node_stats['nodes_imported']} employees")
print(f"Imported {edge_stats['edges_imported']} reporting relationships")
```

### Example 3: Large Dataset

```python
import deepgraph

storage = deepgraph.DiskStorage("./large_graph.db")

# Import 1 million nodes
print("Importing nodes...")
start = time.time()
stats = storage.import_csv_nodes("large_nodes.csv")
duration = time.time() - start

print(f"Imported {stats['nodes_imported']:,} nodes")
print(f"Duration: {duration:.2f} seconds")
print(f"Rate: {stats['nodes_imported'] / duration:,.0f} nodes/sec")

# Import edges
print("Importing edges...")
stats = storage.import_csv_edges("large_edges.csv", stats['node_id_map'])
print(f"Imported {stats['edges_imported']:,} edges")
```

---

## Troubleshooting

### Issue: Import is Slow

**Solution 1**: Increase batch size
```python
# Rust API only - use larger batches
config = ImportConfig::new().with_batch_size(10000)
```

**Solution 2**: Use disk storage
```python
# Disk storage is optimized for bulk loading
storage = deepgraph.DiskStorage("./graph.db")
```

**Solution 3**: Reduce flush frequency
```python
# Rust API only - flush less often
config = ImportConfig::new().with_flush_interval(50000)
```

### Issue: Node Not Found Errors

**Problem**: Edge references node that wasn't imported

**Solution**: Check node IDs match
```python
node_stats = storage.import_csv_nodes("nodes.csv")

# Verify IDs
print("Imported node IDs:", list(node_stats['node_id_map'].keys()))

# Then import edges
edge_stats = storage.import_csv_edges("edges.csv", node_stats['node_id_map'])

# Check for errors
if edge_stats['errors']:
    print("Edge import errors:")
    for error in edge_stats['errors']:
        print(f"  - {error}")
```

### Issue: Type Inference Wrong

**Problem**: CSV value interpreted as wrong type

**Solution 1**: Use JSON (preserves types)
```json
{"age": 30}  // Explicitly integer
```

**Solution 2**: Quote strings in CSV
```csv
name,age,code
Alice,30,"123"  // Code treated as string
```

**Solution 3**: Post-process after import
```python
# Import first
stats = storage.import_csv_nodes("nodes.csv")

# Then update types if needed
# (Manual node update)
```

### Issue: Memory Usage Too High

**Solution 1**: Use streaming (automatic)
- Importers already stream data

**Solution 2**: Import in batches
```python
# Split large file into smaller batches
for batch_file in ["batch_1.csv", "batch_2.csv", ...]:
    stats = storage.import_csv_nodes(batch_file)
```

**Solution 3**: Use disk storage
```python
# Disk storage doesn't load everything into RAM
storage = deepgraph.DiskStorage("./graph.db")
```

---

## Summary

DeepGraph's import functionality provides:

✅ **Easy to use** - Simple Python/Rust APIs  
✅ **Fast** - 100K+ nodes/sec  
✅ **Robust** - Error recovery + validation  
✅ **Flexible** - CSV + JSON support  
✅ **Smart** - Automatic type inference  

**Quick Start:**

```python
import deepgraph

storage = deepgraph.GraphStorage()

# Import data
node_stats = storage.import_csv_nodes("nodes.csv")
edge_stats = storage.import_csv_edges("edges.csv", node_stats['node_id_map'])

# Query data
result = storage.execute_cypher("MATCH (n) RETURN n")
print(f"Total nodes: {result['row_count']}")
```

**Happy importing! 🚀**
