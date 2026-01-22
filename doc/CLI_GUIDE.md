# DeepGraph CLI Guide

## Table of Contents
1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [Interactive REPL](#interactive-repl)
5. [Non-Interactive Mode](#non-interactive-mode)
6. [Data Import](#data-import)
7. [Output Formats](#output-formats)
8. [Meta Commands](#meta-commands)
9. [Examples](#examples)
10. [Troubleshooting](#troubleshooting)

---

## Introduction

DeepGraph CLI (`deepgraph-cli`) is an interactive command-line interface for working with DeepGraph databases. It provides:

- **Interactive REPL**: Execute queries interactively with history support
- **Non-Interactive Mode**: Run single queries or scripts
- **Data Import**: Load CSV/JSON data
- **Multiple Output Formats**: Table, JSON, CSV
- **Meta Commands**: Database statistics, help, etc.

---

## Installation

### Build from Source

```bash
# Clone the repository
git clone https://github.com/deepskilling/deepgraph.git
cd deepgraph

# Build the CLI
cargo build --release --bin deepgraph-cli

# The binary will be at:
# ./target/release/deepgraph-cli
```

### Add to PATH (Optional)

```bash
# Copy to a directory in your PATH
sudo cp target/release/deepgraph-cli /usr/local/bin/

# Now you can run it from anywhere
deepgraph-cli --help
```

---

## Quick Start

### Start Interactive REPL

```bash
# In-memory database (data not persisted)
./target/release/deepgraph-cli

# Persistent database (data saved to disk)
./target/release/deepgraph-cli --database ./mydb.db
```

### Execute Single Query

```bash
# Run a query and exit
./target/release/deepgraph-cli -d ./mydb.db -q "MATCH (n:Person) RETURN n.name"
```

### Import Data

```bash
# Import CSV data
./target/release/deepgraph-cli \
    --database ./mydb.db \
    --import-csv-nodes nodes.csv \
    --import-csv-edges edges.csv
```

---

## Interactive REPL

### Starting the REPL

```bash
$ ./target/release/deepgraph-cli --database ./mydb.db

DeepGraph REPL v0.1.0
Type :help for help, :exit to quit

✅ Opened database: ./mydb.db
deepgraph>
```

### REPL Features

#### Query Execution

Simply type a Cypher query and press Enter:

```cypher
deepgraph> MATCH (n:Person) RETURN n.name, n.age;
┌──────────┬─────┐
│ name     │ age │
├──────────┼─────┤
│ Alice    │ 30  │
│ Bob      │ 25  │
└──────────┴─────┘
2 row(s) (1ms)
```

#### Query History

- **Up Arrow**: Previous command
- **Down Arrow**: Next command
- **Ctrl+R**: Search history (rustyline feature)

#### Multi-Line Queries

Queries can span multiple lines (press Enter to execute):

```cypher
deepgraph> MATCH (n:Person)
... WHERE n.age > 25
... RETURN n.name, n.age;
```

#### Exiting the REPL

- Type `:exit` or `:quit`
- Press `Ctrl+D` (EOF)
- Press `Ctrl+C` twice

---

## Non-Interactive Mode

### Execute Single Query

```bash
# Basic query
deepgraph-cli -d ./mydb.db -q "MATCH (n) RETURN count(n)"

# With output format
deepgraph-cli -d ./mydb.db -q "MATCH (n) RETURN n" --output json
```

### Execute Queries from File

```bash
# Create a query file
cat > queries.cypher << EOF
MATCH (n:Person) WHERE n.age > 25 RETURN n;
MATCH (n)-[r:KNOWS]->(m) RETURN n.name, m.name;
EOF

# Execute (coming soon)
deepgraph-cli -d ./mydb.db -f queries.cypher
```

---

## Data Import

### CSV Import

#### Node CSV Format

```csv
id,name,age,labels
1,Alice,30,Person
2,Bob,25,Person
3,TechCorp,,Organization
```

**Required Column**: `id` (used for edge references)

**Special Columns**:
- `labels` - Semicolon-separated node labels (e.g., `Person;Developer`)
- Other columns become node properties

#### Edge CSV Format

```csv
source,target,type,since
1,2,KNOWS,2020
1,3,WORKS_AT,2018
2,3,WORKS_AT,2019
```

**Required Columns**:
- `source` - Source node ID (matches node CSV `id`)
- `target` - Target node ID
- `type` - Relationship type

**Other Columns**: Become edge properties

#### Import Command

```bash
deepgraph-cli \
    --database ./mydb.db \
    --import-csv-nodes nodes.csv \
    --import-csv-edges edges.csv
```

**Output**:
```
DeepGraph Data Import
====================

Importing CSV nodes from: nodes.csv
✅ Imported 1000 nodes in 45ms
Importing CSV edges from: edges.csv
✅ Imported 2500 edges in 23ms

✅ Import complete! Database: ./mydb.db
Nodes: 1000
Edges: 2500
```

### JSON Import

#### Node JSON Format

```json
[
  {
    "id": "1",
    "labels": ["Person"],
    "properties": {
      "name": "Alice",
      "age": 30
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

#### Edge JSON Format

```json
[
  {
    "source": "1",
    "target": "2",
    "type": "KNOWS",
    "properties": {
      "since": 2020
    }
  }
]
```

#### Import Command

```bash
deepgraph-cli \
    --database ./mydb.db \
    --import-json-nodes nodes.json \
    --import-json-edges edges.json
```

---

## Output Formats

### Table Format (Default)

```bash
deepgraph-cli -d ./mydb.db -q "MATCH (n) RETURN n.name, n.age"
```

**Output**:
```
┌──────────┬─────┐
│ name     │ age │
├──────────┼─────┤
│ Alice    │ 30  │
│ Bob      │ 25  │
└──────────┴─────┘
2 row(s) (1ms)
```

### JSON Format

```bash
deepgraph-cli -d ./mydb.db -q "MATCH (n) RETURN n.name, n.age" --output json
```

**Output**:
```json
[
  {
    "name": "Alice",
    "age": 30
  },
  {
    "name": "Bob",
    "age": 25
  }
]
```

### CSV Format

```bash
deepgraph-cli -d ./mydb.db -q "MATCH (n) RETURN n.name, n.age" --output csv
```

**Output**:
```
name,age
Alice,30
Bob,25
```

### Redirect Output to File

```bash
# Save as JSON
deepgraph-cli -d ./mydb.db -q "MATCH (n) RETURN n" --output json > results.json

# Save as CSV
deepgraph-cli -d ./mydb.db -q "MATCH (n) RETURN n" --output csv > results.csv
```

---

## Meta Commands

Meta commands start with `:` and provide REPL functionality:

### `:help`

Show available commands and examples.

```
deepgraph> :help

DeepGraph REPL Commands:
  Cypher Queries:
    MATCH (n) RETURN n        - Execute Cypher query

  Meta Commands:
    :help                     - Show this help
    :exit, :quit              - Exit REPL
    :stats                    - Show database statistics
    :clear                    - Clear screen

  Examples:
    MATCH (n:Person) RETURN n.name, n.age;
    MATCH (n) WHERE n.age > 25 RETURN n;
```

### `:stats`

Display database statistics.

```
deepgraph> :stats

Database Statistics:
  Nodes: 1000
  Edges: 2500
```

### `:exit` / `:quit`

Exit the REPL.

```
deepgraph> :exit
Goodbye!
```

### `:clear`

Clear the screen (ANSI escape codes).

```
deepgraph> :clear
```

---

## Examples

### Example 1: Create and Query a Social Network

```bash
# Start REPL
./target/release/deepgraph-cli --database ./social.db

# Create nodes (data import recommended for bulk operations)
# For this example, let's import from CSV
```

**nodes.csv**:
```csv
id,name,age,city,labels
1,Alice,30,Seattle,Person
2,Bob,25,Portland,Person
3,Carol,28,Seattle,Person
4,Dave,35,Portland,Person
```

**edges.csv**:
```csv
source,target,type,since
1,2,KNOWS,2015
1,3,KNOWS,2018
2,4,KNOWS,2016
3,4,KNOWS,2019
```

**Import**:
```bash
./target/release/deepgraph-cli \
    --database ./social.db \
    --import-csv-nodes nodes.csv \
    --import-csv-edges edges.csv
```

**Query 1: Find all people**:
```cypher
deepgraph> MATCH (n:Person) RETURN n.name, n.age, n.city;
┌───────┬─────┬──────────┐
│ name  │ age │ city     │
├───────┼─────┼──────────┤
│ Alice │ 30  │ Seattle  │
│ Bob   │ 25  │ Portland │
│ Carol │ 28  │ Seattle  │
│ Dave  │ 35  │ Portland │
└───────┴─────┴──────────┘
4 row(s) (2ms)
```

**Query 2: Find people over 25**:
```cypher
deepgraph> MATCH (n:Person) WHERE n.age > 25 RETURN n.name, n.age;
┌───────┬─────┐
│ name  │ age │
├───────┼─────┤
│ Alice │ 30  │
│ Carol │ 28  │
│ Dave  │ 35  │
└───────┴─────┘
3 row(s) (1ms)
```

**Query 3: Find people in Seattle**:
```cypher
deepgraph> MATCH (n:Person) WHERE n.city = "Seattle" RETURN n.name;
┌───────┐
│ name  │
├───────┤
│ Alice │
│ Carol │
└───────┘
2 row(s) (1ms)
```

### Example 2: Export Data

```bash
# Export as JSON
./target/release/deepgraph-cli \
    -d ./social.db \
    -q "MATCH (n:Person) RETURN n.name, n.age, n.city" \
    --output json > people.json

# Export as CSV
./target/release/deepgraph-cli \
    -d ./social.db \
    -q "MATCH (n:Person) RETURN n.name, n.age, n.city" \
    --output csv > people.csv
```

### Example 3: Scripted Queries

**script.sh**:
```bash
#!/bin/bash
DB="./mydb.db"
CLI="./target/release/deepgraph-cli"

echo "People Count:"
$CLI -d $DB -q "MATCH (n:Person) RETURN count(n)"

echo ""
echo "Young People:"
$CLI -d $DB -q "MATCH (n:Person) WHERE n.age < 30 RETURN n.name, n.age"
```

---

## Troubleshooting

### Issue: "Failed to open database"

**Symptom**:
```
❌ Failed to open database: Permission denied (os error 13)
```

**Solutions**:
1. Check file permissions: `ls -la mydb.db`
2. Ensure the directory exists: `mkdir -p ./data`
3. Use absolute paths: `--database /path/to/mydb.db`

### Issue: "Parse error"

**Symptom**:
```
❌ Error: Parse error: Expected MATCH, CREATE, or DELETE
```

**Solutions**:
1. Check Cypher syntax: [Cypher Guide](./CYPHER_GUIDE.md)
2. Ensure semicolons are correct
3. Check for typos in keywords

### Issue: Database Locked

**Symptom**:
```
❌ Failed to open database: Database is locked
```

**Solutions**:
1. Close other REPL instances
2. Wait for transactions to complete
3. Check for zombie processes: `ps aux | grep deepgraph-cli`

### Issue: Import Errors

**Symptom**:
```
⚠️  5 errors encountered
```

**Solutions**:
1. Check CSV format (headers, column names)
2. Ensure `id` column exists in nodes CSV
3. Verify `source` and `target` IDs exist in edges CSV
4. Check for special characters (escape quotes)

### Issue: Slow Queries

**Symptoms**:
- Queries take > 1 second
- REPL is unresponsive

**Solutions**:
1. Add indexes for frequently queried properties
2. Use label filtering: `MATCH (n:Person)` instead of `MATCH (n)`
3. Limit results: `MATCH (n) RETURN n LIMIT 100`
4. Check query plan (future feature)

---

## Command Reference

### CLI Arguments

| Argument | Short | Description | Example |
|----------|-------|-------------|---------|
| `--database` | `-d` | Database path | `-d ./mydb.db` |
| `--query` | `-q` | Execute query and exit | `-q "MATCH (n) RETURN n"` |
| `--file` | `-f` | Read queries from file | `-f queries.cypher` |
| `--output` | | Output format (table, json, csv) | `--output json` |
| `--import-csv-nodes` | | Import CSV nodes | `--import-csv-nodes nodes.csv` |
| `--import-csv-edges` | | Import CSV edges | `--import-csv-edges edges.csv` |
| `--import-json-nodes` | | Import JSON nodes | `--import-json-nodes nodes.json` |
| `--import-json-edges` | | Import JSON edges | `--import-json-edges edges.json` |
| `--help` | `-h` | Show help | `--help` |
| `--version` | `-V` | Show version | `--version` |

### Meta Commands

| Command | Description |
|---------|-------------|
| `:help` | Show help |
| `:stats` | Show database statistics |
| `:exit`, `:quit` | Exit REPL |
| `:clear` | Clear screen |

---

## Performance Tips

1. **Use Disk Storage for Large Datasets**
   ```bash
   deepgraph-cli --database ./large_graph.db
   ```

2. **Import Data in Bulk**
   - Use CSV/JSON import instead of individual queries
   - Batch size: 1000-10000 records optimal

3. **Filter Early in Queries**
   ```cypher
   # Good: Filter by label first
   MATCH (n:Person) WHERE n.age > 25 RETURN n
   
   # Bad: Full scan then filter
   MATCH (n) WHERE n.age > 25 RETURN n
   ```

4. **Use Persistent Databases**
   - In-memory: Fast but not persistent
   - Disk: Persistent, slightly slower, supports large graphs

---

## Next Steps

- [Cypher Query Guide](./CYPHER_GUIDE.md) - Learn Cypher syntax
- [Disk Storage Guide](./DISK_STORAGE_GUIDE.md) - Configure persistent storage
- [Import Guide](./IMPORT_GUIDE.md) - Advanced data import
- [Python API](../PYTHON_QUICKSTART.md) - Use DeepGraph from Python

---

## Support

- **GitHub Issues**: https://github.com/deepskilling/deepgraph/issues
- **Documentation**: https://github.com/deepskilling/deepgraph/tree/main/doc

---

**DeepGraph CLI** - Making graph databases accessible from the command line! 🚀
