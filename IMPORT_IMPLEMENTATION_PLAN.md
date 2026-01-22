# CSV/JSON Import Implementation Plan

## Goal
Implement data import functionality to easily load graph data from CSV and JSON files into DeepGraph.

## Current State
- ✅ Parquet export exists (snapshot.rs)
- ❌ No CSV import
- ❌ No JSON import
- ❌ No bulk loading API

## Target State
- ✅ CSV import for nodes and edges
- ✅ JSON import for nodes and edges
- ✅ Schema inference
- ✅ Bulk loading optimizations
- ✅ Python + Rust APIs
- ✅ Comprehensive error handling

---

## Data Formats

### CSV Format for Nodes

```csv
id,labels,name,age,city
1,"Person;Employee",Alice,30,NYC
2,"Person;Employee",Bob,25,SF
3,"Company",Acme Corp,,NYC
```

**Format**:
- `id`: Unique identifier (optional, can auto-generate)
- `labels`: Semicolon-separated labels
- Other columns: Properties (auto-inferred types)

### CSV Format for Edges

```csv
from,to,type,since,weight
1,2,KNOWS,2020,0.8
1,3,WORKS_AT,2019,1.0
2,3,WORKS_AT,2021,1.0
```

**Format**:
- `from`: Source node ID
- `to`: Target node ID
- `type`: Relationship type
- Other columns: Edge properties

### JSON Format for Nodes

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
  },
  {
    "id": "2",
    "labels": ["Person"],
    "properties": {
      "name": "Bob",
      "age": 25
    }
  }
]
```

### JSON Format for Edges

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
  },
  {
    "from": "1",
    "to": "3",
    "type": "WORKS_AT",
    "properties": {
      "since": 2019
    }
  }
]
```

---

## Implementation Tasks

### Task 1: Design API ✅ (This Doc)

**File**: `IMPORT_IMPLEMENTATION_PLAN.md`

### Task 2: Implement CSV Parser for Nodes

**File**: `src/import/csv.rs` (NEW)

**Struct**:
```rust
pub struct CsvImporter {
    delimiter: u8,
    has_header: bool,
    label_separator: char,
}

impl CsvImporter {
    pub fn new() -> Self { ... }
    
    pub fn import_nodes<S: StorageBackend>(
        &self,
        storage: &S,
        path: impl AsRef<Path>,
    ) -> Result<ImportStats> { ... }
}
```

**Features**:
- Parse CSV with configurable delimiter
- Infer property types (String, Integer, Float, Boolean)
- Handle missing values
- Support multiple labels (semicolon-separated)
- Progress reporting

### Task 3: Implement CSV Parser for Edges

**File**: `src/import/csv.rs` (continuation)

**Method**:
```rust
impl CsvImporter {
    pub fn import_edges<S: StorageBackend>(
        &self,
        storage: &S,
        path: impl AsRef<Path>,
        node_id_map: &HashMap<String, NodeId>,
    ) -> Result<ImportStats> { ... }
}
```

**Features**:
- Parse edges with from/to/type
- Map external IDs to internal NodeIds
- Handle edge properties
- Validate node existence

### Task 4: Implement JSON Parser

**File**: `src/import/json.rs` (NEW)

**Struct**:
```rust
pub struct JsonImporter;

impl JsonImporter {
    pub fn import_nodes<S: StorageBackend>(
        storage: &S,
        path: impl AsRef<Path>,
    ) -> Result<ImportStats> { ... }
    
    pub fn import_edges<S: StorageBackend>(
        storage: &S,
        path: impl AsRef<Path>,
        node_id_map: &HashMap<String, NodeId>,
    ) -> Result<ImportStats> { ... }
}
```

**Features**:
- Parse JSON arrays
- Support nested properties
- Type preservation (int, float, string, bool, null)
- Streaming for large files

### Task 5: Bulk Import Optimizations

**Optimizations**:
1. Batch operations (reduce flush frequency)
2. Parallel parsing (use rayon)
3. Memory-mapped I/O for large files
4. Progress callbacks

**File**: `src/import/mod.rs`

```rust
pub struct ImportConfig {
    pub batch_size: usize,
    pub parallel: bool,
    pub flush_interval: usize,
}

pub struct ImportStats {
    pub nodes_imported: usize,
    pub edges_imported: usize,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}
```

### Task 6: Python Bindings

**File**: `src/python.rs` (add methods)

```python
# PyGraphStorage methods
storage.import_csv_nodes("nodes.csv")
storage.import_csv_edges("edges.csv", node_id_map)
storage.import_json_nodes("nodes.json")
storage.import_json_edges("edges.json", node_id_map)

# Or unified API
stats = storage.import_from_csv("nodes.csv", "edges.csv")
stats = storage.import_from_json("nodes.json", "edges.json")
```

### Task 7: Tests

**File**: `tests/test_import.rs` (NEW)

**Tests**:
- CSV nodes import
- CSV edges import
- JSON nodes import
- JSON edges import
- Type inference
- Error handling
- Large file handling
- Progress reporting

### Task 8: Documentation

**File**: `doc/IMPORT_GUIDE.md` (NEW)

**Sections**:
- Supported formats
- CSV format specification
- JSON format specification
- Import API (Python + Rust)
- Performance tips
- Error handling
- Examples

---

## Type Inference

### CSV Type Inference

```rust
fn infer_type(value: &str) -> PropertyValue {
    // Try bool
    if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false") {
        return PropertyValue::Boolean(value.parse().unwrap());
    }
    
    // Try integer
    if let Ok(int) = value.parse::<i64>() {
        return PropertyValue::Integer(int);
    }
    
    // Try float
    if let Ok(float) = value.parse::<f64>() {
        return PropertyValue::Float(float);
    }
    
    // Default to string
    PropertyValue::String(value.to_string())
}
```

### JSON Type Preservation

JSON types map directly:
- `number` (int) → `PropertyValue::Integer`
- `number` (float) → `PropertyValue::Float`
- `string` → `PropertyValue::String`
- `boolean` → `PropertyValue::Boolean`
- `null` → `PropertyValue::Null`

---

## Dependencies

### Existing:
✅ `serde = "1.0"` - JSON serialization  
✅ `serde_json = "1.0"` - JSON parsing  

### New:
- `csv = "1.3"` - CSV parsing
- `rayon = "1.8"` (optional) - Parallel processing

---

## Error Handling

```rust
#[derive(Error, Debug)]
pub enum ImportError {
    #[error("Invalid CSV format: {0}")]
    CsvFormatError(String),
    
    #[error("Invalid JSON format: {0}")]
    JsonFormatError(String),
    
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    
    #[error("Type inference failed: {0}")]
    TypeInferenceError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

---

## Usage Examples

### Python - CSV Import

```python
import deepgraph

storage = deepgraph.GraphStorage()

# Import nodes
stats = storage.import_csv_nodes("employees.csv")
print(f"Imported {stats['nodes_imported']} nodes")

# Import edges (with node ID mapping)
stats = storage.import_csv_edges("relationships.csv")
print(f"Imported {stats['edges_imported']} edges")
```

### Python - JSON Import

```python
import deepgraph

storage = deepgraph.DiskStorage("./data/graph.db")

# Import from JSON
stats = storage.import_json_nodes("nodes.json")
stats = storage.import_json_edges("edges.json")

print(f"Total: {stats['nodes_imported']} nodes, {stats['edges_imported']} edges")
```

### Rust - CSV Import

```rust
use deepgraph::import::{CsvImporter, ImportConfig};

let storage = DiskStorage::new("./data/graph.db")?;
let importer = CsvImporter::new();

// Import nodes
let stats = importer.import_nodes(&storage, "nodes.csv")?;
println!("Imported {} nodes", stats.nodes_imported);

// Import edges
let stats = importer.import_edges(&storage, "edges.csv", &node_map)?;
println!("Imported {} edges", stats.edges_imported);
```

---

## Performance Considerations

### Batch Size
- **Small datasets** (<10K records): Batch size 100-500
- **Medium datasets** (10K-1M): Batch size 1000-5000
- **Large datasets** (>1M): Batch size 10000+

### Memory Usage
- **Streaming**: Process one record at a time (low memory)
- **Batch**: Load batch into memory (faster, more memory)
- **Memory-mapped**: Map entire file (fastest, requires RAM)

### Flush Strategy
- **Frequent flush** (every 100 records): Durable, slower
- **Moderate flush** (every 1000 records): Balanced
- **Infrequent flush** (every 10000 records): Fast, less durable

---

## Timeline Estimate

| Task | Effort | Dependencies |
|------|--------|--------------|
| 1. Design (this doc) | 0.5h | None |
| 2. CSV nodes parser | 1.5h | Task 1 |
| 3. CSV edges parser | 1h | Task 2 |
| 4. JSON parser | 1.5h | Task 1 |
| 5. Bulk optimizations | 1h | Task 2-4 |
| 6. Python bindings | 1h | Task 2-5 |
| 7. Tests | 1.5h | Task 2-6 |
| 8. Documentation | 1h | Task 2-7 |
| **Total** | **~9h** | |

---

## Success Criteria

1. ✅ Import CSV nodes with type inference
2. ✅ Import CSV edges with ID mapping
3. ✅ Import JSON nodes/edges
4. ✅ Handle errors gracefully
5. ✅ Performance: >10K nodes/sec
6. ✅ Python + Rust APIs
7. ✅ All tests passing
8. ✅ Documentation complete

---

**Status**: ✅ Plan Complete - Ready for Implementation  
**Date**: 2026-01-22  
**Next**: Implement CSV parser for nodes
