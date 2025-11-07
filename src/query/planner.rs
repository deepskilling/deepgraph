//! Query planner for optimization
//!
//! Transforms AST into optimized execution plans

use crate::error::Result;
use crate::query::ast::*;
use std::collections::HashMap;

/// Logical query plan (high-level operations)
#[derive(Debug, Clone)]
pub enum LogicalPlan {
    /// Scan all nodes
    NodeScan {
        variable: String,
        labels: Vec<String>,
    },
    
    /// Index lookup
    IndexLookup {
        variable: String,
        label: String,
        property: String,
        value: String,
    },
    
    /// Filter operation
    Filter {
        source: Box<LogicalPlan>,
        condition: Expression,
    },
    
    /// Project (return specific fields)
    Project {
        source: Box<LogicalPlan>,
        items: Vec<ReturnItem>,
    },
    
    /// Join two plans
    Join {
        left: Box<LogicalPlan>,
        right: Box<LogicalPlan>,
    },
    
    /// Limit results
    Limit {
        source: Box<LogicalPlan>,
        count: i64,
    },
}

/// Physical query plan (execution details)
#[derive(Debug, Clone)]
pub enum PhysicalPlan {
    /// Scan nodes from storage
    Scan {
        label: Option<String>,
    },
    
    /// Use hash index
    HashIndexScan {
        index_name: String,
        key: Vec<u8>,
    },
    
    /// Use B-tree index with range
    BTreeRangeScan {
        index_name: String,
        start: Vec<u8>,
        end: Vec<u8>,
    },
    
    /// Filter rows
    Filter {
        source: Box<PhysicalPlan>,
        predicate: Expression,
    },
    
    /// Project columns
    Project {
        source: Box<PhysicalPlan>,
        columns: Vec<String>,
    },
}

/// Query planner
pub struct QueryPlanner {
    /// Statistics for cost estimation
    stats: PlannerStats,
}

/// Statistics for query planning
#[derive(Debug, Clone, Default)]
pub struct PlannerStats {
    /// Estimated node count
    pub node_count: usize,
    /// Estimated edge count
    pub edge_count: usize,
    /// Available indices
    pub indices: HashMap<String, IndexStats>,
}

/// Index statistics
#[derive(Debug, Clone)]
pub struct IndexStats {
    /// Index type (hash or btree)
    pub index_type: String,
    /// Number of entries
    pub entry_count: usize,
}

impl QueryPlanner {
    /// Create a new query planner
    pub fn new() -> Self {
        Self {
            stats: PlannerStats::default(),
        }
    }
    
    /// Create planner with statistics
    pub fn with_stats(stats: PlannerStats) -> Self {
        Self { stats }
    }
    
    /// Generate logical plan from AST
    pub fn logical_plan(&self, query: &Query) -> Result<LogicalPlan> {
        match query {
            Query::Read(read_query) => self.plan_read_query(read_query),
            Query::Write(_write_query) => {
                // TODO: Plan write queries
                Ok(LogicalPlan::NodeScan {
                    variable: "n".to_string(),
                    labels: vec![],
                })
            }
        }
    }
    
    /// Plan a read query
    fn plan_read_query(&self, query: &ReadQuery) -> Result<LogicalPlan> {
        // Start with node scan
        let mut plan = self.plan_match(&query.match_clause)?;
        
        // Add filter if WHERE exists
        if let Some(where_clause) = &query.where_clause {
            plan = LogicalPlan::Filter {
                source: Box::new(plan),
                condition: where_clause.condition.clone(),
            };
        }
        
        // Add projection for RETURN
        plan = LogicalPlan::Project {
            source: Box::new(plan),
            items: query.return_clause.items.clone(),
        };
        
        // Add limit if specified
        if let Some(limit) = query.return_clause.limit {
            plan = LogicalPlan::Limit {
                source: Box::new(plan),
                count: limit,
            };
        }
        
        Ok(plan)
    }
    
    /// Plan MATCH clause
    fn plan_match(&self, match_clause: &MatchClause) -> Result<LogicalPlan> {
        if match_clause.patterns.is_empty() {
            return Ok(LogicalPlan::NodeScan {
                variable: "n".to_string(),
                labels: vec![],
            });
        }
        
        // For now, simple node scan
        // TODO: Analyze patterns and use indices
        Ok(LogicalPlan::NodeScan {
            variable: "n".to_string(),
            labels: vec![],
        })
    }
    
    /// Optimize logical plan into physical plan
    pub fn physical_plan(&self, logical: &LogicalPlan) -> Result<PhysicalPlan> {
        match logical {
            LogicalPlan::NodeScan { labels, .. } => {
                // Check if we have an index for this label
                let label = labels.first().cloned();
                Ok(PhysicalPlan::Scan { label })
            }
            
            LogicalPlan::Filter { source, condition } => {
                let source_plan = self.physical_plan(source)?;
                Ok(PhysicalPlan::Filter {
                    source: Box::new(source_plan),
                    predicate: condition.clone(),
                })
            }
            
            LogicalPlan::Project { source, items } => {
                let source_plan = self.physical_plan(source)?;
                let columns: Vec<String> = items
                    .iter()
                    .filter_map(|item| {
                        if let Expression::Variable(name) = &item.expression {
                            Some(name.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                
                Ok(PhysicalPlan::Project {
                    source: Box::new(source_plan),
                    columns,
                })
            }
            
            LogicalPlan::Limit { source, .. } => {
                // For now, just pass through
                // TODO: Push limit down for optimization
                self.physical_plan(source)
            }
            
            _ => {
                // Fallback to simple scan
                Ok(PhysicalPlan::Scan { label: None })
            }
        }
    }
    
    /// Estimate cost of a logical plan
    pub fn estimate_cost(&self, plan: &LogicalPlan) -> f64 {
        match plan {
            LogicalPlan::NodeScan { .. } => {
                // Full scan cost = node count
                self.stats.node_count as f64
            }
            
            LogicalPlan::IndexLookup { .. } => {
                // Index lookup cost = O(log n)
                (self.stats.node_count as f64).log2()
            }
            
            LogicalPlan::Filter { source, .. } => {
                // Filter cost = source cost + evaluation
                self.estimate_cost(source) + self.stats.node_count as f64 * 0.1
            }
            
            LogicalPlan::Project { source, .. } => {
                // Project has minimal overhead
                self.estimate_cost(source) + 1.0
            }
            
            LogicalPlan::Limit { source, count } => {
                // Limit reduces cost
                self.estimate_cost(source).min(*count as f64)
            }
            
            LogicalPlan::Join { left, right } => {
                // Join cost = product of inputs
                self.estimate_cost(left) * self.estimate_cost(right)
            }
        }
    }
}

impl Default for QueryPlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planner_creation() {
        let planner = QueryPlanner::new();
        assert_eq!(planner.stats.node_count, 0);
    }

    #[test]
    fn test_cost_estimation() {
        let mut stats = PlannerStats::default();
        stats.node_count = 1000;
        let planner = QueryPlanner::with_stats(stats);
        
        let plan = LogicalPlan::NodeScan {
            variable: "n".to_string(),
            labels: vec![],
        };
        
        let cost = planner.estimate_cost(&plan);
        assert_eq!(cost, 1000.0);
    }

    #[test]
    fn test_index_lookup_cost() {
        let mut stats = PlannerStats::default();
        stats.node_count = 1000;
        let planner = QueryPlanner::with_stats(stats);
        
        let plan = LogicalPlan::IndexLookup {
            variable: "n".to_string(),
            label: "Person".to_string(),
            property: "name".to_string(),
            value: "Alice".to_string(),
        };
        
        let cost = planner.estimate_cost(&plan);
        assert!(cost < 100.0); // Much cheaper than full scan
    }
}

