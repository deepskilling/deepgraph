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
            // Full scan - now implemented!
            self.storage.get_all_nodes()
        };
        
        // Convert nodes to result rows with properties
        let mut columns = vec!["_node_id".to_string()];
        let rows: Vec<HashMap<String, PropertyValue>> = nodes
            .into_iter()
            .map(|node| {
                let mut row = HashMap::new();
                
                // Add node ID
                row.insert("_node_id".to_string(), 
                    PropertyValue::String(node.id().to_string()));
                
                // Add all node properties to row for property access
                for (key, value) in node.properties().iter() {
                    row.insert(key.clone(), value.clone());
                    // Track columns dynamically
                    if !columns.contains(key) {
                        columns.push(key.clone());
                    }
                }
                
                row
            })
            .collect();
        
        Ok(QueryResult::with_data(columns, rows))
    }
    
    /// Execute a filter operation
    fn execute_filter(
        &self,
        source: &PhysicalPlan,
        predicate: &crate::query::ast::Expression,
    ) -> Result<QueryResult> {
        // Get source results
        let source_result = self.execute(source)?;
        
        // Evaluate predicate on each row
        let filtered_rows: Vec<HashMap<String, PropertyValue>> = source_result
            .rows
            .into_iter()
            .filter(|row| self.evaluate_predicate(predicate, row).unwrap_or(false))
            .collect();
        
        Ok(QueryResult::with_data(source_result.columns, filtered_rows))
    }
    
    /// Evaluate a predicate expression on a row
    fn evaluate_predicate(
        &self,
        expr: &crate::query::ast::Expression,
        row: &HashMap<String, PropertyValue>,
    ) -> Result<bool> {
        use crate::query::ast::Expression;
        
        match expr {
            // Logical operators
            Expression::And(left, right) => {
                let left_val = self.evaluate_predicate(left, row)?;
                let right_val = self.evaluate_predicate(right, row)?;
                Ok(left_val && right_val)
            }
            Expression::Or(left, right) => {
                let left_val = self.evaluate_predicate(left, row)?;
                let right_val = self.evaluate_predicate(right, row)?;
                Ok(left_val || right_val)
            }
            Expression::Not(inner) => {
                let val = self.evaluate_predicate(inner, row)?;
                Ok(!val)
            }
            
            // Comparison operators
            Expression::Eq(left, right) => {
                let left_val = self.evaluate_value(left, row)?;
                let right_val = self.evaluate_value(right, row)?;
                Ok(left_val == right_val)
            }
            Expression::Ne(left, right) => {
                let left_val = self.evaluate_value(left, row)?;
                let right_val = self.evaluate_value(right, row)?;
                Ok(left_val != right_val)
            }
            Expression::Lt(left, right) => {
                let left_val = self.evaluate_value(left, row)?;
                let right_val = self.evaluate_value(right, row)?;
                Ok(self.compare_values(&left_val, &right_val)? < 0)
            }
            Expression::Le(left, right) => {
                let left_val = self.evaluate_value(left, row)?;
                let right_val = self.evaluate_value(right, row)?;
                Ok(self.compare_values(&left_val, &right_val)? <= 0)
            }
            Expression::Gt(left, right) => {
                let left_val = self.evaluate_value(left, row)?;
                let right_val = self.evaluate_value(right, row)?;
                Ok(self.compare_values(&left_val, &right_val)? > 0)
            }
            Expression::Ge(left, right) => {
                let left_val = self.evaluate_value(left, row)?;
                let right_val = self.evaluate_value(right, row)?;
                Ok(self.compare_values(&left_val, &right_val)? >= 0)
            }
            
            _ => {
                // For other expressions, try to evaluate as value and check if truthy
                let val = self.evaluate_value(expr, row)?;
                Ok(match val {
                    PropertyValue::Boolean(b) => b,
                    PropertyValue::Null => false,
                    _ => true,
                })
            }
        }
    }
    
    /// Evaluate an expression to a value
    fn evaluate_value(
        &self,
        expr: &crate::query::ast::Expression,
        row: &HashMap<String, PropertyValue>,
    ) -> Result<PropertyValue> {
        use crate::query::ast::Expression;
        
        match expr {
            Expression::Literal(val) => Ok(val.clone()),
            
            Expression::Variable(name) => {
                // Look up variable in row
                row.get(name)
                    .cloned()
                    .ok_or_else(|| crate::error::DeepGraphError::InvalidOperation(
                        format!("Variable not found: {}", name)
                    ))
            }
            
            Expression::Property(base, prop) => {
                // For property access like n.age, evaluate base then get property
                if let Expression::Variable(var_name) = base.as_ref() {
                    // Look up property directly in row (we flattened it in scan)
                    row.get(prop)
                        .cloned()
                        .ok_or_else(|| crate::error::DeepGraphError::InvalidOperation(
                            format!("Property not found: {}.{}", var_name, prop)
                        ))
                } else {
                    Err(crate::error::DeepGraphError::InvalidOperation(
                        "Complex property access not yet supported".to_string()
                    ))
                }
            }
            
            // Arithmetic operators
            Expression::Add(left, right) => {
                let left_val = self.evaluate_value(left, row)?;
                let right_val = self.evaluate_value(right, row)?;
                self.add_values(&left_val, &right_val)
            }
            Expression::Sub(left, right) => {
                let left_val = self.evaluate_value(left, row)?;
                let right_val = self.evaluate_value(right, row)?;
                self.sub_values(&left_val, &right_val)
            }
            Expression::Mul(left, right) => {
                let left_val = self.evaluate_value(left, row)?;
                let right_val = self.evaluate_value(right, row)?;
                self.mul_values(&left_val, &right_val)
            }
            Expression::Div(left, right) => {
                let left_val = self.evaluate_value(left, row)?;
                let right_val = self.evaluate_value(right, row)?;
                self.div_values(&left_val, &right_val)
            }
            
            Expression::Neg(inner) => {
                let val = self.evaluate_value(inner, row)?;
                match val {
                    PropertyValue::Integer(i) => Ok(PropertyValue::Integer(-i)),
                    PropertyValue::Float(f) => Ok(PropertyValue::Float(-f)),
                    _ => Err(crate::error::DeepGraphError::InvalidOperation(
                        "Cannot negate non-numeric value".to_string()
                    )),
                }
            }
            
            _ => Err(crate::error::DeepGraphError::InvalidOperation(
                format!("Expression evaluation not yet implemented: {:?}", expr)
            )),
        }
    }
    
    /// Compare two property values
    fn compare_values(&self, left: &PropertyValue, right: &PropertyValue) -> Result<i32> {
        match (left, right) {
            (PropertyValue::Integer(l), PropertyValue::Integer(r)) => Ok(l.cmp(r) as i32),
            (PropertyValue::Float(l), PropertyValue::Float(r)) => {
                if l < r { Ok(-1) }
                else if l > r { Ok(1) }
                else { Ok(0) }
            }
            (PropertyValue::Integer(l), PropertyValue::Float(r)) => {
                let l = *l as f64;
                if l < *r { Ok(-1) }
                else if l > *r { Ok(1) }
                else { Ok(0) }
            }
            (PropertyValue::Float(l), PropertyValue::Integer(r)) => {
                let r = *r as f64;
                if l < &r { Ok(-1) }
                else if l > &r { Ok(1) }
                else { Ok(0) }
            }
            (PropertyValue::String(l), PropertyValue::String(r)) => Ok(l.cmp(r) as i32),
            (PropertyValue::Boolean(l), PropertyValue::Boolean(r)) => Ok(l.cmp(r) as i32),
            _ => Err(crate::error::DeepGraphError::InvalidOperation(
                "Cannot compare incompatible types".to_string()
            )),
        }
    }
    
    /// Add two property values
    fn add_values(&self, left: &PropertyValue, right: &PropertyValue) -> Result<PropertyValue> {
        match (left, right) {
            (PropertyValue::Integer(l), PropertyValue::Integer(r)) => Ok(PropertyValue::Integer(l + r)),
            (PropertyValue::Float(l), PropertyValue::Float(r)) => Ok(PropertyValue::Float(l + r)),
            (PropertyValue::Integer(l), PropertyValue::Float(r)) => Ok(PropertyValue::Float(*l as f64 + r)),
            (PropertyValue::Float(l), PropertyValue::Integer(r)) => Ok(PropertyValue::Float(l + *r as f64)),
            (PropertyValue::String(l), PropertyValue::String(r)) => Ok(PropertyValue::String(format!("{}{}", l, r))),
            _ => Err(crate::error::DeepGraphError::InvalidOperation(
                "Cannot add incompatible types".to_string()
            )),
        }
    }
    
    /// Subtract two property values
    fn sub_values(&self, left: &PropertyValue, right: &PropertyValue) -> Result<PropertyValue> {
        match (left, right) {
            (PropertyValue::Integer(l), PropertyValue::Integer(r)) => Ok(PropertyValue::Integer(l - r)),
            (PropertyValue::Float(l), PropertyValue::Float(r)) => Ok(PropertyValue::Float(l - r)),
            (PropertyValue::Integer(l), PropertyValue::Float(r)) => Ok(PropertyValue::Float(*l as f64 - r)),
            (PropertyValue::Float(l), PropertyValue::Integer(r)) => Ok(PropertyValue::Float(l - *r as f64)),
            _ => Err(crate::error::DeepGraphError::InvalidOperation(
                "Cannot subtract incompatible types".to_string()
            )),
        }
    }
    
    /// Multiply two property values
    fn mul_values(&self, left: &PropertyValue, right: &PropertyValue) -> Result<PropertyValue> {
        match (left, right) {
            (PropertyValue::Integer(l), PropertyValue::Integer(r)) => Ok(PropertyValue::Integer(l * r)),
            (PropertyValue::Float(l), PropertyValue::Float(r)) => Ok(PropertyValue::Float(l * r)),
            (PropertyValue::Integer(l), PropertyValue::Float(r)) => Ok(PropertyValue::Float(*l as f64 * r)),
            (PropertyValue::Float(l), PropertyValue::Integer(r)) => Ok(PropertyValue::Float(l * *r as f64)),
            _ => Err(crate::error::DeepGraphError::InvalidOperation(
                "Cannot multiply incompatible types".to_string()
            )),
        }
    }
    
    /// Divide two property values
    fn div_values(&self, left: &PropertyValue, right: &PropertyValue) -> Result<PropertyValue> {
        match (left, right) {
            (PropertyValue::Integer(l), PropertyValue::Integer(r)) => {
                if *r == 0 {
                    return Err(crate::error::DeepGraphError::InvalidOperation(
                        "Division by zero".to_string()
                    ));
                }
                Ok(PropertyValue::Integer(l / r))
            }
            (PropertyValue::Float(l), PropertyValue::Float(r)) => {
                if *r == 0.0 {
                    return Err(crate::error::DeepGraphError::InvalidOperation(
                        "Division by zero".to_string()
                    ));
                }
                Ok(PropertyValue::Float(l / r))
            }
            (PropertyValue::Integer(l), PropertyValue::Float(r)) => {
                if *r == 0.0 {
                    return Err(crate::error::DeepGraphError::InvalidOperation(
                        "Division by zero".to_string()
                    ));
                }
                Ok(PropertyValue::Float(*l as f64 / r))
            }
            (PropertyValue::Float(l), PropertyValue::Integer(r)) => {
                if *r == 0 {
                    return Err(crate::error::DeepGraphError::InvalidOperation(
                        "Division by zero".to_string()
                    ));
                }
                Ok(PropertyValue::Float(l / *r as f64))
            }
            _ => Err(crate::error::DeepGraphError::InvalidOperation(
                "Cannot divide incompatible types".to_string()
            )),
        }
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

