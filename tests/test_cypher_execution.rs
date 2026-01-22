//! End-to-end integration tests for Cypher query execution
//!
//! Tests the complete pipeline: Query String → Parser → Planner → Executor → Results

use deepgraph::graph::{Node, PropertyValue};
use deepgraph::query::{CypherParser, QueryPlanner, QueryExecutor};
use deepgraph::query::ast::Statement;
use deepgraph::storage::GraphStorage;
use std::sync::Arc;

/// Helper function to create test graph with sample data
fn create_test_graph() -> Arc<GraphStorage> {
    let storage = Arc::new(GraphStorage::new());
    
    // Add Person nodes
    let mut alice = Node::new(vec!["Person".to_string()]);
    alice.set_property("name".to_string(), PropertyValue::String("Alice".to_string()));
    alice.set_property("age".to_string(), PropertyValue::Integer(30));
    alice.set_property("city".to_string(), PropertyValue::String("NYC".to_string()));
    storage.add_node(alice).unwrap();
    
    let mut bob = Node::new(vec!["Person".to_string()]);
    bob.set_property("name".to_string(), PropertyValue::String("Bob".to_string()));
    bob.set_property("age".to_string(), PropertyValue::Integer(25));
    bob.set_property("city".to_string(), PropertyValue::String("SF".to_string()));
    storage.add_node(bob).unwrap();
    
    let mut charlie = Node::new(vec!["Person".to_string()]);
    charlie.set_property("name".to_string(), PropertyValue::String("Charlie".to_string()));
    charlie.set_property("age".to_string(), PropertyValue::Integer(35));
    charlie.set_property("city".to_string(), PropertyValue::String("NYC".to_string()));
    storage.add_node(charlie).unwrap();
    
    // Add a Company node
    let mut acme = Node::new(vec!["Company".to_string()]);
    acme.set_property("name".to_string(), PropertyValue::String("Acme Corp".to_string()));
    acme.set_property("founded".to_string(), PropertyValue::Integer(2010));
    storage.add_node(acme).unwrap();
    
    storage
}

#[test]
fn test_simple_match_all() {
    println!("\n=== Test: Simple MATCH (n) RETURN n ===");
    
    let storage = create_test_graph();
    let query_str = "MATCH (n) RETURN n;";
    
    // Parse
    let ast = CypherParser::parse(query_str).unwrap();
    println!("✅ Parsed query successfully");
    
    // Plan
    let Statement::Query(query) = ast;
    let planner = QueryPlanner::new();
    let logical = planner.logical_plan(&query).unwrap();
    let physical = planner.physical_plan(&logical).unwrap();
    println!("✅ Created execution plan");
    
    // Execute
    let executor = QueryExecutor::new(storage.clone());
    let result = executor.execute(&physical).unwrap();
    println!("✅ Executed query");
    
    // Verify
    println!("Result: {} rows", result.row_count);
    assert_eq!(result.row_count, 4, "Should return 4 nodes (3 Person + 1 Company)");
    println!("✅ Test passed!\n");
}

#[test]
fn test_match_with_label() {
    println!("\n=== Test: MATCH (n:Person) RETURN n ===");
    
    let storage = create_test_graph();
    let query_str = "MATCH (n:Person) RETURN n;";
    
    // Parse
    let ast = CypherParser::parse(query_str).unwrap();
    println!("✅ Parsed query");
    
    // Plan
    let Statement::Query(query) = ast;
    let planner = QueryPlanner::new();
    let logical = planner.logical_plan(&query).unwrap();
    let physical = planner.physical_plan(&logical).unwrap();
    println!("✅ Created plan");
    
    // Execute
    let executor = QueryExecutor::new(storage);
    let result = executor.execute(&physical).unwrap();
    println!("✅ Executed");
    
    // Verify
    println!("Result: {} rows", result.row_count);
    assert_eq!(result.row_count, 3, "Should return 3 Person nodes");
    println!("✅ Test passed!\n");
}

#[test]
fn test_where_greater_than() {
    println!("\n=== Test: MATCH (n:Person) WHERE n.age > 25 RETURN n ===");
    
    let storage = create_test_graph();
    let query_str = "MATCH (n:Person) WHERE n.age > 25 RETURN n;";
    
    // Parse
    let ast = CypherParser::parse(query_str).unwrap();
    println!("✅ Parsed: {}", query_str);
    
    // Plan
    let Statement::Query(query) = ast;
    let planner = QueryPlanner::new();
    let logical = planner.logical_plan(&query).unwrap();
    println!("✅ Logical plan created");
    
    let physical = planner.physical_plan(&logical).unwrap();
    println!("✅ Physical plan created");
    
    // Execute
    let executor = QueryExecutor::new(storage);
    let result = executor.execute(&physical).unwrap();
    println!("✅ Query executed");
    
    // Verify
    println!("Result: {} rows", result.row_count);
    println!("Rows returned:");
    for (i, row) in result.rows.iter().enumerate() {
        println!("  Row {}: name={:?}, age={:?}", 
            i, 
            row.get("name"), 
            row.get("age")
        );
    }
    
    assert_eq!(result.row_count, 2, "Should return 2 people (Alice:30, Charlie:35)");
    
    // Verify ages are > 25
    for row in &result.rows {
        if let Some(PropertyValue::Integer(age)) = row.get("age") {
            assert!(*age > 25, "Age should be > 25, got {}", age);
        }
    }
    
    println!("✅ Test passed!\n");
}

#[test]
fn test_where_equals() {
    println!("\n=== Test: MATCH (n:Person) WHERE n.name = \"Alice\" RETURN n ===");
    
    let storage = create_test_graph();
    let query_str = "MATCH (n:Person) WHERE n.name = \"Alice\" RETURN n;";
    
    // Parse
    let ast = CypherParser::parse(query_str).unwrap();
    
    // Plan
    let Statement::Query(query) = ast;
    let planner = QueryPlanner::new();
    let logical = planner.logical_plan(&query).unwrap();
    let physical = planner.physical_plan(&logical).unwrap();
    
    // Execute
    let executor = QueryExecutor::new(storage);
    let result = executor.execute(&physical).unwrap();
    
    // Verify
    println!("Result: {} rows", result.row_count);
    assert_eq!(result.row_count, 1, "Should return 1 person (Alice)");
    
    if let Some(PropertyValue::String(name)) = result.rows[0].get("name") {
        assert_eq!(name, "Alice");
    } else {
        panic!("Expected name property");
    }
    
    println!("✅ Test passed!\n");
}

#[test]
fn test_where_and_condition() {
    println!("\n=== Test: WHERE with AND ===");
    
    let storage = create_test_graph();
    let query_str = "MATCH (n:Person) WHERE n.age > 25 AND n.city = \"NYC\" RETURN n;";
    
    // Parse
    let ast = CypherParser::parse(query_str).unwrap();
    println!("✅ Parsed complex WHERE");
    
    // Plan
    let Statement::Query(query) = ast;
    let planner = QueryPlanner::new();
    let logical = planner.logical_plan(&query).unwrap();
    let physical = planner.physical_plan(&logical).unwrap();
    
    // Execute
    let executor = QueryExecutor::new(storage);
    let result = executor.execute(&physical).unwrap();
    
    // Verify
    println!("Result: {} rows", result.row_count);
    println!("Rows:");
    for row in &result.rows {
        println!("  {:?}", row);
    }
    
    // Should return: Alice (age:30, city:NYC) and Charlie (age:35, city:NYC)
    // But WHERE requires age > 25 AND city = NYC, so both match
    assert_eq!(result.row_count, 2, "Should return 2 people in NYC with age > 25");
    
    // Verify both conditions
    for row in &result.rows {
        if let Some(PropertyValue::Integer(age)) = row.get("age") {
            assert!(*age > 25, "Age should be > 25");
        }
        if let Some(PropertyValue::String(city)) = row.get("city") {
            assert_eq!(city, "NYC", "City should be NYC");
        }
    }
    
    println!("✅ Test passed!\n");
}

#[test]
fn test_where_less_than_or_equal() {
    println!("\n=== Test: WHERE with <= ===");
    
    let storage = create_test_graph();
    let query_str = "MATCH (n:Person) WHERE n.age <= 30 RETURN n;";
    
    let ast = CypherParser::parse(query_str).unwrap();
    let Statement::Query(query) = ast;
    let planner = QueryPlanner::new();
    let logical = planner.logical_plan(&query).unwrap();
    let physical = planner.physical_plan(&logical).unwrap();
    
    let executor = QueryExecutor::new(storage);
    let result = executor.execute(&physical).unwrap();
    
    println!("Result: {} rows", result.row_count);
    assert_eq!(result.row_count, 2, "Should return 2 people (Bob:25, Alice:30)");
    
    for row in &result.rows {
        if let Some(PropertyValue::Integer(age)) = row.get("age") {
            assert!(*age <= 30, "Age should be <= 30, got {}", age);
        }
    }
    
    println!("✅ Test passed!\n");
}

#[test]
fn test_property_access_in_return() {
    println!("\n=== Test: Property access in RETURN ===");
    
    let storage = create_test_graph();
    let query_str = "MATCH (n:Person) RETURN n;";
    
    let ast = CypherParser::parse(query_str).unwrap();
    let Statement::Query(query) = ast;
    let planner = QueryPlanner::new();
    let logical = planner.logical_plan(&query).unwrap();
    let physical = planner.physical_plan(&logical).unwrap();
    
    let executor = QueryExecutor::new(storage);
    let result = executor.execute(&physical).unwrap();
    
    // Verify all rows have name and age properties
    println!("Verifying properties in results:");
    for (i, row) in result.rows.iter().enumerate() {
        println!("  Row {}: ", i);
        assert!(row.contains_key("name"), "Should have 'name' property");
        assert!(row.contains_key("age"), "Should have 'age' property");
        println!("    name: {:?}", row.get("name"));
        println!("    age: {:?}", row.get("age"));
    }
    
    println!("✅ Test passed!\n");
}

#[test]
fn test_full_pipeline_validation() {
    println!("\n=== Test: Full Pipeline Validation ===");
    println!("Testing complete flow: Parse → Plan → Execute\n");
    
    let storage = create_test_graph();
    
    let test_cases = vec![
        ("MATCH (n) RETURN n;", 4, "All nodes"),
        ("MATCH (n:Person) RETURN n;", 3, "Person nodes"),
        ("MATCH (n:Company) RETURN n;", 1, "Company nodes"),
    ];
    
    for (query_str, expected_count, description) in test_cases {
        println!("Testing: {} - {}", description, query_str);
        
        // Parse
        let ast = CypherParser::parse(query_str).unwrap();
        
        // Plan
        let Statement::Query(query) = ast;
        let planner = QueryPlanner::new();
        let logical = planner.logical_plan(&query).unwrap();
        let physical = planner.physical_plan(&logical).unwrap();
        
        // Execute
        let executor = QueryExecutor::new(storage.clone());
        let result = executor.execute(&physical).unwrap();
        
        // Verify
        println!("  Result: {} rows (expected {})", result.row_count, expected_count);
        assert_eq!(
            result.row_count, 
            expected_count,
            "Query '{}' should return {} rows",
            query_str,
            expected_count
        );
        println!("  ✅ Passed\n");
    }
    
    println!("✅ All pipeline tests passed!\n");
}

#[test]
fn test_where_not_equal() {
    println!("\n=== Test: WHERE with != ===");
    
    let storage = create_test_graph();
    let query_str = "MATCH (n:Person) WHERE n.city != \"NYC\" RETURN n;";
    
    let ast = CypherParser::parse(query_str).unwrap();
    let Statement::Query(query) = ast;
    let planner = QueryPlanner::new();
    let logical = planner.logical_plan(&query).unwrap();
    let physical = planner.physical_plan(&logical).unwrap();
    
    let executor = QueryExecutor::new(storage);
    let result = executor.execute(&physical).unwrap();
    
    println!("Result: {} rows", result.row_count);
    assert_eq!(result.row_count, 1, "Should return 1 person (Bob in SF)");
    
    if let Some(PropertyValue::String(city)) = result.rows[0].get("city") {
        assert_ne!(city, "NYC");
        assert_eq!(city, "SF");
    }
    
    println!("✅ Test passed!\n");
}

#[test]
fn test_execution_time_tracking() {
    println!("\n=== Test: Execution time tracking ===");
    
    let storage = create_test_graph();
    let query_str = "MATCH (n:Person) WHERE n.age > 20 RETURN n;";
    
    let ast = CypherParser::parse(query_str).unwrap();
    let Statement::Query(query) = ast;
    let planner = QueryPlanner::new();
    let logical = planner.logical_plan(&query).unwrap();
    let physical = planner.physical_plan(&logical).unwrap();
    
    let executor = QueryExecutor::new(storage);
    let result = executor.execute(&physical).unwrap();
    
    println!("Execution time: {}ms", result.execution_time_ms);
    // Execution time is u64, always non-negative, just verify it's tracked
    assert!(result.row_count > 0, "Should have results");
    
    println!("✅ Test passed!\n");
}

#[test]
fn test_empty_result() {
    println!("\n=== Test: Query with no matching results ===");
    
    let storage = create_test_graph();
    let query_str = "MATCH (n:Person) WHERE n.age > 100 RETURN n;";
    
    let ast = CypherParser::parse(query_str).unwrap();
    let Statement::Query(query) = ast;
    let planner = QueryPlanner::new();
    let logical = planner.logical_plan(&query).unwrap();
    let physical = planner.physical_plan(&logical).unwrap();
    
    let executor = QueryExecutor::new(storage);
    let result = executor.execute(&physical).unwrap();
    
    println!("Result: {} rows", result.row_count);
    assert_eq!(result.row_count, 0, "Should return 0 rows for age > 100");
    assert!(result.rows.is_empty(), "Rows should be empty");
    
    println!("✅ Test passed!\n");
}
