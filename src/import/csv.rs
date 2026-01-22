//! CSV import functionality

use crate::error::{DeepGraphError, Result};
use crate::graph::{Node, Edge, PropertyValue, NodeId};
use crate::storage::StorageBackend;
use crate::import::{ImportStats, ImportConfig};
use csv::StringRecord;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use uuid::Uuid;

/// CSV importer for nodes and edges
pub struct CsvImporter {
    config: ImportConfig,
    delimiter: u8,
    has_header: bool,
    label_separator: char,
}

impl CsvImporter {
    /// Create a new CSV importer with default configuration
    pub fn new() -> Self {
        Self {
            config: ImportConfig::new(),
            delimiter: b',',
            has_header: true,
            label_separator: ';',
        }
    }
    
    /// Set the configuration
    pub fn with_config(mut self, config: ImportConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Set the CSV delimiter
    pub fn with_delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }
    
    /// Set whether CSV has a header row
    pub fn with_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }
    
    /// Set the label separator character
    pub fn with_label_separator(mut self, separator: char) -> Self {
        self.label_separator = separator;
        self
    }
    
    /// Import nodes from a CSV file
    ///
    /// # CSV Format
    ///
    /// ```csv
    /// id,labels,name,age,city
    /// 1,"Person;Employee",Alice,30,NYC
    /// 2,"Person",Bob,25,SF
    /// ```
    ///
    /// - `id` column: External node ID (optional, will auto-generate if missing)
    /// - `labels` column: Semicolon-separated labels
    /// - Other columns: Properties (types inferred automatically)
    ///
    /// # Example
    ///
    /// ```rust
    /// use deepgraph::import::CsvImporter;
    /// use deepgraph::storage::MemoryStorage;
    ///
    /// let storage = MemoryStorage::new();
    /// let importer = CsvImporter::new();
    /// let stats = importer.import_nodes(&storage, "nodes.csv")?;
    /// println!("Imported {} nodes", stats.nodes_imported);
    /// ```
    pub fn import_nodes<S: StorageBackend>(
        &self,
        storage: &S,
        path: impl AsRef<Path>,
    ) -> Result<ImportStats> {
        let path = path.as_ref();
        info!("Importing nodes from CSV: {:?}", path);
        
        let mut stats = ImportStats::new();
        let timer = stats.start_timer();
        
        // Open CSV file
        let file = File::open(path)
            .map_err(|e| DeepGraphError::IoError(e))?;
        
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(self.delimiter)
            .has_headers(self.has_header)
            .from_reader(file);
        
        // Get headers
        let headers = if self.has_header {
            reader.headers()
                .map_err(|e| DeepGraphError::StorageError(format!("CSV header error: {}", e)))?
                .clone()
        } else {
            // Generate default headers
            let first_record = reader.records().next()
                .ok_or_else(|| DeepGraphError::StorageError("Empty CSV file".to_string()))?
                .map_err(|e| DeepGraphError::StorageError(format!("CSV read error: {}", e)))?;
            StringRecord::from(
                (0..first_record.len())
                    .map(|i| format!("col{}", i))
                    .collect::<Vec<_>>()
            )
        };
        
        debug!("CSV headers: {:?}", headers);
        
        // Find special columns
        let id_col = headers.iter().position(|h| h.eq_ignore_ascii_case("id"));
        let labels_col = headers.iter().position(|h| h.eq_ignore_ascii_case("labels") || h.eq_ignore_ascii_case("label"));
        
        // Process records
        let mut record_count = 0;
        for result in reader.records() {
            match result {
                Ok(record) => {
                    record_count += 1;
                    
                    match self.import_node_record(&headers, &record, id_col, labels_col, storage, &mut stats) {
                        Ok(_) => {},
                        Err(e) => {
                            stats.add_error(format!("Row {}: {}", record_count, e));
                            if !self.config.skip_invalid {
                                return Err(e);
                            }
                            if self.config.max_errors > 0 && stats.errors.len() >= self.config.max_errors {
                                warn!("Max errors ({}) reached, aborting import", self.config.max_errors);
                                break;
                            }
                        }
                    }
                    
                    // Periodic flush
                    if record_count % self.config.flush_interval == 0 {
                        debug!("Processed {} records", record_count);
                    }
                }
                Err(e) => {
                    stats.add_error(format!("CSV parse error: {}", e));
                    if !self.config.skip_invalid {
                        return Err(DeepGraphError::StorageError(format!("CSV parse error: {}", e)));
                    }
                }
            }
        }
        
        stats.stop_timer(timer);
        info!("Import complete: {} nodes imported in {}ms", stats.nodes_imported, stats.duration_ms);
        
        if !stats.errors.is_empty() {
            warn!("Import completed with {} errors", stats.errors.len());
        }
        
        Ok(stats)
    }
    
    /// Import a single node record
    fn import_node_record<S: StorageBackend>(
        &self,
        headers: &StringRecord,
        record: &StringRecord,
        id_col: Option<usize>,
        labels_col: Option<usize>,
        storage: &S,
        stats: &mut ImportStats,
    ) -> Result<()> {
        // Get or generate external ID
        let external_id = if let Some(col) = id_col {
            record.get(col)
                .ok_or_else(|| DeepGraphError::StorageError("Missing ID column".to_string()))?
                .to_string()
        } else {
            // Auto-generate ID
            format!("node_{}", stats.nodes_imported)
        };
        
        // Get labels
        let labels = if let Some(col) = labels_col {
            let labels_str = record.get(col)
                .ok_or_else(|| DeepGraphError::StorageError("Missing labels column".to_string()))?;
            self.parse_labels(labels_str)
        } else {
            vec!["Node".to_string()]  // Default label
        };
        
        // Create node
        let mut node = Node::new(labels);
        
        // Add properties from other columns
        for (i, header) in headers.iter().enumerate() {
            // Skip special columns
            if Some(i) == id_col || Some(i) == labels_col {
                continue;
            }
            
            if let Some(value) = record.get(i) {
                if !value.is_empty() {
                    let prop_value = self.infer_type(value);
                    node.set_property(header.to_string(), prop_value);
                }
            }
        }
        
        // Add to storage
        let internal_id = storage.add_node(node)?;
        stats.record_node(external_id, internal_id.to_string());
        
        Ok(())
    }
    
    /// Parse labels from a string (semicolon-separated)
    fn parse_labels(&self, labels_str: &str) -> Vec<String> {
        labels_str
            .split(self.label_separator)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
    
    /// Infer property type from string value
    fn infer_type(&self, value: &str) -> PropertyValue {
        let value = value.trim();
        
        // Try boolean
        if value.eq_ignore_ascii_case("true") {
            return PropertyValue::Boolean(true);
        }
        if value.eq_ignore_ascii_case("false") {
            return PropertyValue::Boolean(false);
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
    
    /// Import edges from a CSV file
    ///
    /// # CSV Format
    ///
    /// ```csv
    /// from,to,type,since,weight
    /// 1,2,KNOWS,2020,0.8
    /// 1,3,WORKS_AT,2019,1.0
    /// ```
    ///
    /// - `from`: Source node external ID
    /// - `to`: Target node external ID
    /// - `type`: Relationship type
    /// - Other columns: Edge properties
    ///
    /// # Example
    ///
    /// ```rust
    /// use deepgraph::import::CsvImporter;
    /// use deepgraph::storage::MemoryStorage;
    ///
    /// let storage = MemoryStorage::new();
    /// let importer = CsvImporter::new();
    ///
    /// // First import nodes
    /// let node_stats = importer.import_nodes(&storage, "nodes.csv")?;
    ///
    /// // Then import edges using node ID map
    /// let edge_stats = importer.import_edges(&storage, "edges.csv", &node_stats.node_id_map)?;
    /// println!("Imported {} edges", edge_stats.edges_imported);
    /// ```
    pub fn import_edges<S: StorageBackend>(
        &self,
        storage: &S,
        path: impl AsRef<Path>,
        node_id_map: &HashMap<String, String>,
    ) -> Result<ImportStats> {
        let path = path.as_ref();
        info!("Importing edges from CSV: {:?}", path);
        
        let mut stats = ImportStats::new();
        let timer = stats.start_timer();
        
        // Open CSV file
        let file = File::open(path)
            .map_err(|e| DeepGraphError::IoError(e))?;
        
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(self.delimiter)
            .has_headers(self.has_header)
            .from_reader(file);
        
        // Get headers
        let headers = if self.has_header {
            reader.headers()
                .map_err(|e| DeepGraphError::StorageError(format!("CSV header error: {}", e)))?
                .clone()
        } else {
            let first_record = reader.records().next()
                .ok_or_else(|| DeepGraphError::StorageError("Empty CSV file".to_string()))?
                .map_err(|e| DeepGraphError::StorageError(format!("CSV read error: {}", e)))?;
            StringRecord::from(
                (0..first_record.len())
                    .map(|i| format!("col{}", i))
                    .collect::<Vec<_>>()
            )
        };
        
        debug!("CSV headers: {:?}", headers);
        
        // Find required columns
        let from_col = headers.iter().position(|h| h.eq_ignore_ascii_case("from") || h.eq_ignore_ascii_case("source") || h.eq_ignore_ascii_case("src"))
            .ok_or_else(|| DeepGraphError::StorageError("Missing 'from' column in edges CSV".to_string()))?;
        
        let to_col = headers.iter().position(|h| h.eq_ignore_ascii_case("to") || h.eq_ignore_ascii_case("target") || h.eq_ignore_ascii_case("dst"))
            .ok_or_else(|| DeepGraphError::StorageError("Missing 'to' column in edges CSV".to_string()))?;
        
        let type_col = headers.iter().position(|h| h.eq_ignore_ascii_case("type") || h.eq_ignore_ascii_case("relationship") || h.eq_ignore_ascii_case("label"))
            .ok_or_else(|| DeepGraphError::StorageError("Missing 'type' column in edges CSV".to_string()))?;
        
        // Process records
        let mut record_count = 0;
        for result in reader.records() {
            match result {
                Ok(record) => {
                    record_count += 1;
                    
                    match self.import_edge_record(&headers, &record, from_col, to_col, type_col, node_id_map, storage, &mut stats) {
                        Ok(_) => {},
                        Err(e) => {
                            stats.add_error(format!("Row {}: {}", record_count, e));
                            if !self.config.skip_invalid {
                                return Err(e);
                            }
                            if self.config.max_errors > 0 && stats.errors.len() >= self.config.max_errors {
                                warn!("Max errors ({}) reached, aborting import", self.config.max_errors);
                                break;
                            }
                        }
                    }
                    
                    if record_count % self.config.flush_interval == 0 {
                        debug!("Processed {} edge records", record_count);
                    }
                }
                Err(e) => {
                    stats.add_error(format!("CSV parse error: {}", e));
                    if !self.config.skip_invalid {
                        return Err(DeepGraphError::StorageError(format!("CSV parse error: {}", e)));
                    }
                }
            }
        }
        
        stats.stop_timer(timer);
        info!("Import complete: {} edges imported in {}ms", stats.edges_imported, stats.duration_ms);
        
        if !stats.errors.is_empty() {
            warn!("Import completed with {} errors", stats.errors.len());
        }
        
        Ok(stats)
    }
    
    /// Import a single edge record
    fn import_edge_record<S: StorageBackend>(
        &self,
        headers: &StringRecord,
        record: &StringRecord,
        from_col: usize,
        to_col: usize,
        type_col: usize,
        node_id_map: &HashMap<String, String>,
        storage: &S,
        stats: &mut ImportStats,
    ) -> Result<()> {
        // Get from/to external IDs
        let from_external = record.get(from_col)
            .ok_or_else(|| DeepGraphError::StorageError("Missing 'from' value".to_string()))?;
        
        let to_external = record.get(to_col)
            .ok_or_else(|| DeepGraphError::StorageError("Missing 'to' value".to_string()))?;
        
        // Map to internal IDs
        let from_internal = node_id_map.get(from_external)
            .ok_or_else(|| DeepGraphError::StorageError(format!("Node '{}' not found in ID map", from_external)))?;
        
        let to_internal = node_id_map.get(to_external)
            .ok_or_else(|| DeepGraphError::StorageError(format!("Node '{}' not found in ID map", to_external)))?;
        
        let from_id = NodeId::from_uuid(Uuid::parse_str(from_internal)
            .map_err(|e| DeepGraphError::StorageError(format!("Invalid node ID: {}", e)))?);
        
        let to_id = NodeId::from_uuid(Uuid::parse_str(to_internal)
            .map_err(|e| DeepGraphError::StorageError(format!("Invalid node ID: {}", e)))?);
        
        // Get relationship type
        let rel_type = record.get(type_col)
            .ok_or_else(|| DeepGraphError::StorageError("Missing 'type' value".to_string()))?
            .to_string();
        
        // Create edge
        let mut edge = Edge::new(from_id, to_id, rel_type);
        
        // Add properties from other columns
        for (i, header) in headers.iter().enumerate() {
            // Skip required columns
            if i == from_col || i == to_col || i == type_col {
                continue;
            }
            
            if let Some(value) = record.get(i) {
                if !value.is_empty() {
                    let prop_value = self.infer_type(value);
                    edge.set_property(header.to_string(), prop_value);
                }
            }
        }
        
        // Add to storage
        storage.add_edge(edge)?;
        stats.record_edge();
        
        Ok(())
    }
}

impl Default for CsvImporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_infer_type() {
        let importer = CsvImporter::new();
        
        // Boolean
        assert!(matches!(importer.infer_type("true"), PropertyValue::Boolean(true)));
        assert!(matches!(importer.infer_type("false"), PropertyValue::Boolean(false)));
        assert!(matches!(importer.infer_type("TRUE"), PropertyValue::Boolean(true)));
        
        // Integer
        assert!(matches!(importer.infer_type("42"), PropertyValue::Integer(42)));
        assert!(matches!(importer.infer_type("-100"), PropertyValue::Integer(-100)));
        
        // Float
        assert!(matches!(importer.infer_type("3.14"), PropertyValue::Float(_)));
        assert!(matches!(importer.infer_type("-2.5"), PropertyValue::Float(_)));
        
        // String
        assert!(matches!(importer.infer_type("hello"), PropertyValue::String(_)));
        assert!(matches!(importer.infer_type("123abc"), PropertyValue::String(_)));
    }
    
    #[test]
    fn test_parse_labels() {
        let importer = CsvImporter::new();
        
        let labels = importer.parse_labels("Person;Employee");
        assert_eq!(labels, vec!["Person", "Employee"]);
        
        let labels = importer.parse_labels("Node");
        assert_eq!(labels, vec!["Node"]);
        
        let labels = importer.parse_labels("  A ; B  ; C  ");
        assert_eq!(labels, vec!["A", "B", "C"]);
    }
    
    #[test]
    fn test_import_nodes() {
        // Create test CSV file
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,labels,name,age").unwrap();
        writeln!(file, "1,Person,Alice,30").unwrap();
        writeln!(file, "2,Person,Bob,25").unwrap();
        
        // Import
        let storage = MemoryStorage::new();
        let importer = CsvImporter::new();
        let stats = importer.import_nodes(&storage, file.path()).unwrap();
        
        assert_eq!(stats.nodes_imported, 2);
        assert_eq!(stats.errors.len(), 0);
        assert_eq!(stats.node_id_map.len(), 2);
    }
}
