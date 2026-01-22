# Phase 4: Cypher Execution Implementation Plan

## Overview

This document tracks the implementation of Phase 4: Critical Features, focusing on **Cypher Execution**.

## Current State

### ✅ What Exists
1. **Cypher Grammar** (`grammar.pest`) - Complete openCypher-compatible grammar
2. **AST Structures** (`ast.rs`) - Complete AST types for all Cypher constructs
3. **Query Planner** (`planner.rs`) - Can convert AST → LogicalPlan → PhysicalPlan
4. **Query Executor** (`executor.rs`) - Can execute Scan, Filter, Project operations

### ❌ Critical Gap
**Parser (`parser.rs` line 24-42)**: The `parse()` function validates syntax but returns a **placeholder AST** instead of actually building the AST from Pest tokens.

```rust
// Current state - PLACEHOLDER!
pub fn parse(query: &str) -> Result<Statement> {
    let _pairs = CypherGrammarParser::parse(Rule::statement, query)?;
    
    // ❌ Returns empty placeholder instead of building AST!
    Ok(Statement::Query(Query::Read(ReadQuery {
        match_clause: MatchClause { patterns: vec![] },
        where_clause: None,
        return_clause: ReturnClause {
            distinct: false,
            items: vec![],
            order_by: None,
            limit: None,
        },
    })))
}
```

## Implementation Tasks

### Task 1: Implement AST Builder ⚡ **CRITICAL**
**File**: `src/query/parser.rs`

**Goal**: Convert Pest parse tree → AST

**What to implement**:
1. `build_statement()` - Convert `Rule::statement` → `Statement`
2. `build_query()` - Convert query rules → `Query`
3. `build_read_query()` - Build `ReadQuery` from MATCH/WHERE/RETURN
4. `build_match_clause()` - Parse MATCH patterns
5. `build_pattern()` - Parse node and relationship patterns
6. `build_node_pattern()` - Parse `(n:Label {prop: value})`
7. `build_where_clause()` - Parse WHERE conditions
8. `build_expression()` - Parse expressions recursively
9. `build_return_clause()` - Parse RETURN items

**Example**:
```rust
pub fn parse(query: &str) -> Result<Statement> {
    let pairs = CypherGrammarParser::parse(Rule::statement, query)
        .map_err(|e| DeepGraphError::ParserError(format!("Parse error: {}", e)))?;
    
    let pair = pairs.into_iter().next()
        .ok_or_else(|| DeepGraphError::ParserError("Empty parse result".to_string()))?;
    
    build_statement(pair)
}

fn build_statement(pair: Pair<Rule>) -> Result<Statement> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::query => return Ok(Statement::Query(build_query(inner)?)),
            _ => {}
        }
    }
    Err(DeepGraphError::ParserError("Invalid statement".to_string()))
}
```

### Task 2: Enhance Executor for WHERE Evaluation
**File**: `src/query/executor.rs`

**Current**: Filter operation exists but doesn't evaluate predicates (line 104-116)

**Goal**: Actually evaluate WHERE conditions

**What to implement**:
```rust
fn execute_filter(&self, source: &PhysicalPlan, predicate: &Expression) -> Result<QueryResult> {
    let source_result = self.execute(source)?;
    
    // ✅ NEW: Evaluate predicate on each row
    let filtered_rows: Vec<_> = source_result
        .rows
        .into_iter()
        .filter(|row| self.evaluate_expression(predicate, row).unwrap_or(false))
        .collect();
    
    Ok(QueryResult::with_data(source_result.columns, filtered_rows))
}

fn evaluate_expression(&self, expr: &Expression, row: &HashMap<String, PropertyValue>) -> Result<bool> {
    match expr {
        Expression::Gt(left, right) => {
            let left_val = self.eval_value(left, row)?;
            let right_val = self.eval_value(right, row)?;
            Ok(left_val > right_val)
        }
        // ... implement all comparison operators
        _ => Ok(false)
    }
}
```

### Task 3: Add Property Access in Executor
**File**: `src/query/executor.rs`

**Goal**: Access node properties in results (e.g., `n.name`, `n.age`)

**What to implement**:
```rust
fn execute_scan(&self, label: Option<&str>) -> Result<QueryResult> {
    let nodes = if let Some(label) = label {
        self.storage.get_nodes_by_label(label)
    } else {
        self.storage.get_all_nodes() // Task 4
    };
    
    // ✅ NEW: Include node properties in result
    let columns = vec!["node".to_string(), "properties".to_string()];
    let rows: Vec<HashMap<String, PropertyValue>> = nodes
        .into_iter()
        .map(|node| {
            let mut row = HashMap::new();
            row.insert("node".to_string(), PropertyValue::String(node.id().to_string()));
            
            // Add all properties as nested map
            for (key, value) in node.properties() {
                row.insert(key.clone(), value.clone());
            }
            
            row
        })
        .collect();
    
    Ok(QueryResult::with_data(columns, rows))
}
```

### Task 4: Implement Full Node Scan
**File**: `src/storage/mod.rs`

**Goal**: Add `get_all_nodes()` to StorageBackend trait

**What to implement**:
```rust
pub trait StorageBackend: Send + Sync {
    // Existing methods...
    
    // ✅ NEW: Get all nodes (for MATCH (n))
    fn get_all_nodes(&self) -> Vec<Node>;
}
```

**File**: `src/storage/memory.rs`

```rust
fn get_all_nodes(&self) -> Vec<Node> {
    self.nodes
        .iter()
        .map(|entry| entry.value().clone())
        .collect()
}
```

### Task 5: End-to-End Integration Test
**File**: `tests/test_cypher_execution.rs`

**Goal**: Test full pipeline: Query string → Results

**Example test**:
```rust
#[test]
fn test_simple_match_return() {
    let storage = Arc::new(GraphStorage::new());
    
    // Add test data
    let mut node = Node::new(vec!["Person".to_string()]);
    node.set_property("name".to_string(), "Alice".into());
    node.set_property("age".to_string(), 30i64.into());
    storage.add_node(node).unwrap();
    
    // Parse query
    let query_str = "MATCH (n:Person) RETURN n.name, n.age;";
    let ast = CypherParser::parse(query_str).unwrap();
    
    // Plan query
    let planner = QueryPlanner::new();
    let Statement::Query(query) = ast;
    let logical = planner.logical_plan(&query).unwrap();
    let physical = planner.physical_plan(&logical).unwrap();
    
    // Execute query
    let executor = QueryExecutor::new(storage);
    let result = executor.execute(&physical).unwrap();
    
    // Verify results
    assert_eq!(result.row_count, 1);
    assert_eq!(result.rows[0].get("name"), Some(&PropertyValue::String("Alice".to_string())));
    assert_eq!(result.rows[0].get("age"), Some(&PropertyValue::Integer(30)));
}
```

### Task 6: Python Bindings
**File**: `src/python.rs`

**Goal**: Add `execute_query()` method to PyGraphStorage

```python
import deepgraph

storage = deepgraph.GraphStorage()
storage.add_node(["Person"], {"name": "Alice", "age": 30})

# ✅ NEW: Execute Cypher queries!
result = storage.query("MATCH (n:Person) WHERE n.age > 25 RETURN n.name, n.age")
print(result.rows)  # [{'name': 'Alice', 'age': 30}]
```

## Success Criteria

- [ ] Parser builds complete AST from query strings
- [ ] Planner converts AST to execution plans
- [ ] Executor runs plans and returns results
- [ ] WHERE clause filtering works
- [ ] Property access works (n.name, n.age)
- [ ] End-to-end tests pass
- [ ] Python bindings work
- [ ] Documentation updated

## Testing Strategy

1. **Unit Tests** - Each component (parser, planner, executor)
2. **Integration Tests** - Full pipeline
3. **Python Tests** - PyO3 bindings
4. **Example Queries** - Real-world Cypher queries

### Example Queries to Support

```cypher
-- Simple MATCH
MATCH (n) RETURN n;

-- MATCH with label
MATCH (n:Person) RETURN n;

-- MATCH with WHERE
MATCH (n:Person) WHERE n.age > 25 RETURN n;

-- MATCH with property access
MATCH (n:Person) RETURN n.name, n.age;

-- MATCH with LIMIT
MATCH (n:Person) RETURN n LIMIT 10;

-- MATCH with ORDER BY
MATCH (n:Person) RETURN n ORDER BY n.age DESC;
```

## Timeline Estimate

- **Task 1 (AST Builder)**: 4-6 hours
- **Task 2 (WHERE evaluation)**: 2-3 hours
- **Task 3 (Property access)**: 1-2 hours
- **Task 4 (Full scan)**: 30 minutes
- **Task 5 (Integration tests)**: 2-3 hours
- **Task 6 (Python bindings)**: 1-2 hours

**Total**: 11-17 hours of focused work

## Current Progress

✅ Phase 1: Foundation - COMPLETE
✅ Phase 2: Core Features - COMPLETE
✅ Phase 3: Algorithms - COMPLETE
🚧 Phase 4: Cypher Execution - IN PROGRESS

**Next Step**: Implement AST builder in parser.rs (Task 1)

## Notes

- The infrastructure is 90% there - we just need to wire it together!
- Parser grammar validation already works
- AST structures are complete
- Planner and executor framework exists
- Just need to implement the "glue" code

---

**Status**: Ready to implement!  
**Started**: January 22, 2026  
**Priority**: 🔴 CRITICAL (Phase 4.1)
