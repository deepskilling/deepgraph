//! Data import module for DeepGraph
//!
//! Supports importing graph data from CSV and JSON files.

pub mod csv;
pub mod json;

pub use csv::CsvImporter;
pub use json::JsonImporter;

use std::collections::HashMap;
use std::time::Instant;

/// Statistics from an import operation
#[derive(Debug, Clone)]
pub struct ImportStats {
    /// Number of nodes successfully imported
    pub nodes_imported: usize,
    
    /// Number of edges successfully imported
    pub edges_imported: usize,
    
    /// Errors encountered during import
    pub errors: Vec<String>,
    
    /// Duration of import in milliseconds
    pub duration_ms: u64,
    
    /// Node ID mapping (external ID → internal NodeId)
    pub node_id_map: HashMap<String, String>,
}

impl ImportStats {
    /// Create new import stats
    pub fn new() -> Self {
        Self {
            nodes_imported: 0,
            edges_imported: 0,
            errors: Vec::new(),
            duration_ms: 0,
            node_id_map: HashMap::new(),
        }
    }
    
    /// Start timing
    pub fn start_timer(&mut self) -> Instant {
        Instant::now()
    }
    
    /// Stop timing and record duration
    pub fn stop_timer(&mut self, start: Instant) {
        self.duration_ms = start.elapsed().as_millis() as u64;
    }
    
    /// Add an error
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }
    
    /// Record node import
    pub fn record_node(&mut self, external_id: String, internal_id: String) {
        self.nodes_imported += 1;
        self.node_id_map.insert(external_id, internal_id);
    }
    
    /// Record edge import
    pub fn record_edge(&mut self) {
        self.edges_imported += 1;
    }
}

impl Default for ImportStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for import operations
#[derive(Debug, Clone)]
pub struct ImportConfig {
    /// Batch size for bulk operations
    pub batch_size: usize,
    
    /// Flush to disk after every N records
    pub flush_interval: usize,
    
    /// Skip invalid records instead of failing
    pub skip_invalid: bool,
    
    /// Maximum errors before aborting (0 = unlimited)
    pub max_errors: usize,
}

impl ImportConfig {
    /// Create a new import configuration with defaults
    pub fn new() -> Self {
        Self {
            batch_size: 1000,
            flush_interval: 5000,
            skip_invalid: true,
            max_errors: 100,
        }
    }
    
    /// Set batch size
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }
    
    /// Set flush interval
    pub fn with_flush_interval(mut self, interval: usize) -> Self {
        self.flush_interval = interval;
        self
    }
    
    /// Set whether to skip invalid records
    pub fn with_skip_invalid(mut self, skip: bool) -> Self {
        self.skip_invalid = skip;
        self
    }
    
    /// Set maximum errors
    pub fn with_max_errors(mut self, max: usize) -> Self {
        self.max_errors = max;
        self
    }
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self::new()
    }
}
