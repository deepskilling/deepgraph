//! Query executor for running physical plans
//!
//! Executes optimized query plans against the storage engine

use crate::error::Result;
use crate::graph::PropertyValue;
use crate::query::planner::PhysicalPlan;
use crate::storage::StorageBackend;
use std::collections::HashMap;
use std::sync::Arc;

/// Query execution result
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Column names
    pub columns: Vec<String>,
    /// Rows of data
    pub rows: Vec<HashMap<String, PropertyValue>>,
    /// Number of rows returned
    pub row_count: usize,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

impl QueryResult {
    /// Create a new empty result
    pub fn empty() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            row_count: 0,
            execution_time_ms: 0,
        }
    }
    
    /// Create result with data
    pub fn with_data(columns: Vec<String>, rows: Vec<HashMap<String, PropertyValue>>) -> Self {
        let row_count = rows.len();
        Self {
            columns,
            rows,
            row_count,
            execution_time_ms: 0,
        }
    }
}

/// Query executor
pub struct QueryExecutor<S: StorageBackend> {
    /// Storage backend
    storage: Arc<S>,
}

impl<S: StorageBackend> QueryExecutor<S> {
    /// Create a new executor
    pub fn new(storage: Arc<S>) -> Self {
        Self { storage }
    }
    
    /// Execute a physical plan
    pub fn execute(&self, plan: &PhysicalPlan) -> Result<QueryResult> {
        let start = std::time::Instant::now();
        
        let mut result = match plan {
            PhysicalPlan::Scan { label } => self.execute_scan(label.as_deref())?,
            PhysicalPlan::Filter { source, predicate } => {
                self.execute_filter(source, predicate)?
            }
            PhysicalPlan::Project { source, columns } => {
                self.execute_project(source, columns)?
            }
            _ => QueryResult::empty(),
        };
        
        result.execution_time_ms = start.elapsed().as_millis() as u64;
        Ok(result)
    }
    
    /// Execute a scan operation
    fn execute_scan(&self, label: Option<&str>) -> Result<QueryResult> {
        let nodes = if let Some(label) = label {
            self.storage.get_nodes_by_label(label)
        } else {
            // Full scan - not implemented yet in trait
            vec![]
        };
        
        // Convert nodes to result rows
        let columns = vec!["node".to_string()];
        let rows: Vec<HashMap<String, PropertyValue>> = nodes
            .into_iter()
            .map(|node| {
                let mut row = HashMap::new();
                // Serialize node ID as string for now
                row.insert("node".to_string(), 
                    PropertyValue::String(node.id().to_string()));
                row
            })
            .collect();
        
        Ok(QueryResult::with_data(columns, rows))
    }
    
    /// Execute a filter operation
    fn execute_filter(
        &self,
        source: &PhysicalPlan,
        _predicate: &crate::query::ast::Expression,
    ) -> Result<QueryResult> {
        // Get source results
        let source_result = self.execute(source)?;
        
        // TODO: Evaluate predicate on each row
        // For now, just return source results
        Ok(source_result)
    }
    
    /// Execute a projection
    fn execute_project(
        &self,
        source: &PhysicalPlan,
        columns: &[String],
    ) -> Result<QueryResult> {
        let source_result = self.execute(source)?;
        
        // Project only requested columns
        let rows: Vec<HashMap<String, PropertyValue>> = source_result
            .rows
            .into_iter()
            .map(|row| {
                let mut projected = HashMap::new();
                for col in columns {
                    if let Some(value) = row.get(col) {
                        projected.insert(col.clone(), value.clone());
                    }
                }
                projected
            })
            .collect();
        
        Ok(QueryResult::with_data(columns.to_vec(), rows))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;

    #[test]
    fn test_executor_creation() {
        let storage = Arc::new(MemoryStorage::new());
        let _executor = QueryExecutor::new(storage);
        // Executor created successfully
    }

    #[test]
    fn test_empty_scan() {
        let storage = Arc::new(MemoryStorage::new());
        let executor = QueryExecutor::new(storage);
        
        let plan = PhysicalPlan::Scan { label: None };
        let result = executor.execute(&plan).unwrap();
        
        assert_eq!(result.row_count, 0);
    }

    #[test]
    fn test_scan_with_label() {
        let storage = Arc::new(MemoryStorage::new());
        
        // Add some test data
        let node = crate::graph::Node::new(vec!["Person".to_string()]);
        storage.add_node(node).unwrap();
        
        let executor = QueryExecutor::new(storage);
        let plan = PhysicalPlan::Scan {
            label: Some("Person".to_string()),
        };
        
        let result = executor.execute(&plan).unwrap();
        assert_eq!(result.row_count, 1);
    }
}

