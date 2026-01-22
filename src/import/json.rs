//! JSON import functionality

use crate::error::{DeepGraphError, Result};
use crate::graph::{Node, Edge, PropertyValue, NodeId};
use crate::storage::StorageBackend;
use crate::import::{ImportStats, ImportConfig};
use log::{debug, info, warn};
use serde_json::{Value, Map};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use uuid::Uuid;

/// JSON importer for nodes and edges
pub struct JsonImporter {
    config: ImportConfig,
}

impl JsonImporter {
    /// Create a new JSON importer with default configuration
    pub fn new() -> Self {
        Self {
            config: ImportConfig::new(),
        }
    }
    
    /// Set the configuration
    pub fn with_config(mut self, config: ImportConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Import nodes from a JSON file
    ///
    /// # JSON Format
    ///
    /// ```json
    /// [
    ///   {
    ///     "id": "1",
    ///     "labels": ["Person", "Employee"],
    ///     "properties": {
    ///       "name": "Alice",
    ///       "age": 30,
    ///       "city": "NYC"
    ///     }
    ///   },
    ///   {
    ///     "id": "2",
    ///     "labels": ["Person"],
    ///     "properties": {
    ///       "name": "Bob",
    ///       "age": 25
    ///     }
    ///   }
    /// ]
    /// ```
    ///
    /// # Example
    ///
    /// ```rust
    /// use deepgraph::import::JsonImporter;
    /// use deepgraph::storage::MemoryStorage;
    ///
    /// let storage = MemoryStorage::new();
    /// let importer = JsonImporter::new();
    /// let stats = importer.import_nodes(&storage, "nodes.json")?;
    /// println!("Imported {} nodes", stats.nodes_imported);
    /// ```
    pub fn import_nodes<S: StorageBackend>(
        &self,
        storage: &S,
        path: impl AsRef<Path>,
    ) -> Result<ImportStats> {
        let path = path.as_ref();
        info!("Importing nodes from JSON: {:?}", path);
        
        let mut stats = ImportStats::new();
        let timer = stats.start_timer();
        
        // Open JSON file
        let file = File::open(path)
            .map_err(|e| DeepGraphError::IoError(e))?;
        let reader = BufReader::new(file);
        
        // Parse JSON array
        let nodes: Vec<Value> = serde_json::from_reader(reader)
            .map_err(|e| DeepGraphError::JsonError(e))?;
        
        debug!("Parsed {} node records", nodes.len());
        
        // Process each node
        for (i, node_value) in nodes.iter().enumerate() {
            match self.import_node_value(node_value, storage, &mut stats) {
                Ok(_) => {},
                Err(e) => {
                    stats.add_error(format!("Node {}: {}", i, e));
                    if !self.config.skip_invalid {
                        return Err(e);
                    }
                    if self.config.max_errors > 0 && stats.errors.len() >= self.config.max_errors {
                        warn!("Max errors ({}) reached, aborting import", self.config.max_errors);
                        break;
                    }
                }
            }
            
            if (i + 1) % self.config.flush_interval == 0 {
                debug!("Processed {} nodes", i + 1);
            }
        }
        
        stats.stop_timer(timer);
        info!("Import complete: {} nodes imported in {}ms", stats.nodes_imported, stats.duration_ms);
        
        if !stats.errors.is_empty() {
            warn!("Import completed with {} errors", stats.errors.len());
        }
        
        Ok(stats)
    }
    
    /// Import a single node from JSON value
    fn import_node_value<S: StorageBackend>(
        &self,
        value: &Value,
        storage: &S,
        stats: &mut ImportStats,
    ) -> Result<()> {
        let obj = value.as_object()
            .ok_or_else(|| DeepGraphError::StorageError("Expected JSON object".to_string()))?;
        
        // Get external ID (optional)
        let external_id = obj.get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("node_{}", stats.nodes_imported));
        
        // Get labels
        let labels = self.parse_labels(obj)?;
        
        // Create node
        let mut node = Node::new(labels);
        
        // Get properties
        if let Some(props) = obj.get("properties") {
            if let Some(props_obj) = props.as_object() {
                for (key, value) in props_obj {
                    let prop_value = self.json_to_property_value(value)?;
                    node.set_property(key.clone(), prop_value);
                }
            }
        }
        
        // Add to storage
        let internal_id = storage.add_node(node)?;
        stats.record_node(external_id, internal_id.to_string());
        
        Ok(())
    }
    
    /// Parse labels from JSON object
    fn parse_labels(&self, obj: &Map<String, Value>) -> Result<Vec<String>> {
        if let Some(labels_value) = obj.get("labels") {
            if let Some(labels_array) = labels_value.as_array() {
                let labels: Result<Vec<String>> = labels_array
                    .iter()
                    .map(|v| {
                        v.as_str()
                            .ok_or_else(|| DeepGraphError::StorageError("Label must be string".to_string()))
                            .map(|s| s.to_string())
                    })
                    .collect();
                return labels;
            } else if let Some(label_str) = labels_value.as_str() {
                // Single label as string
                return Ok(vec![label_str.to_string()]);
            }
        }
        
        // Default label if missing
        Ok(vec!["Node".to_string()])
    }
    
    /// Convert JSON value to PropertyValue
    fn json_to_property_value(&self, value: &Value) -> Result<PropertyValue> {
        match value {
            Value::Null => Ok(PropertyValue::Null),
            Value::Bool(b) => Ok(PropertyValue::Boolean(*b)),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(PropertyValue::Integer(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(PropertyValue::Float(f))
                } else {
                    Err(DeepGraphError::StorageError("Invalid number".to_string()))
                }
            }
            Value::String(s) => Ok(PropertyValue::String(s.clone())),
            Value::Array(_) | Value::Object(_) => {
                // For complex types, serialize to JSON string
                Ok(PropertyValue::String(value.to_string()))
            }
        }
    }
    
    /// Import edges from a JSON file
    ///
    /// # JSON Format
    ///
    /// ```json
    /// [
    ///   {
    ///     "from": "1",
    ///     "to": "2",
    ///     "type": "KNOWS",
    ///     "properties": {
    ///       "since": 2020,
    ///       "weight": 0.8
    ///     }
    ///   },
    ///   {
    ///     "from": "1",
    ///     "to": "3",
    ///     "type": "WORKS_AT",
    ///     "properties": {
    ///       "since": 2019
    ///     }
    ///   }
    /// ]
    /// ```
    ///
    /// # Example
    ///
    /// ```rust
    /// use deepgraph::import::JsonImporter;
    /// use deepgraph::storage::MemoryStorage;
    ///
    /// let storage = MemoryStorage::new();
    /// let importer = JsonImporter::new();
    ///
    /// // First import nodes
    /// let node_stats = importer.import_nodes(&storage, "nodes.json")?;
    ///
    /// // Then import edges
    /// let edge_stats = importer.import_edges(&storage, "edges.json", &node_stats.node_id_map)?;
    /// println!("Imported {} edges", edge_stats.edges_imported);
    /// ```
    pub fn import_edges<S: StorageBackend>(
        &self,
        storage: &S,
        path: impl AsRef<Path>,
        node_id_map: &HashMap<String, String>,
    ) -> Result<ImportStats> {
        let path = path.as_ref();
        info!("Importing edges from JSON: {:?}", path);
        
        let mut stats = ImportStats::new();
        let timer = stats.start_timer();
        
        // Open JSON file
        let file = File::open(path)
            .map_err(|e| DeepGraphError::IoError(e))?;
        let reader = BufReader::new(file);
        
        // Parse JSON array
        let edges: Vec<Value> = serde_json::from_reader(reader)
            .map_err(|e| DeepGraphError::JsonError(e))?;
        
        debug!("Parsed {} edge records", edges.len());
        
        // Process each edge
        for (i, edge_value) in edges.iter().enumerate() {
            match self.import_edge_value(edge_value, node_id_map, storage, &mut stats) {
                Ok(_) => {},
                Err(e) => {
                    stats.add_error(format!("Edge {}: {}", i, e));
                    if !self.config.skip_invalid {
                        return Err(e);
                    }
                    if self.config.max_errors > 0 && stats.errors.len() >= self.config.max_errors {
                        warn!("Max errors ({}) reached, aborting import", self.config.max_errors);
                        break;
                    }
                }
            }
            
            if (i + 1) % self.config.flush_interval == 0 {
                debug!("Processed {} edges", i + 1);
            }
        }
        
        stats.stop_timer(timer);
        info!("Import complete: {} edges imported in {}ms", stats.edges_imported, stats.duration_ms);
        
        if !stats.errors.is_empty() {
            warn!("Import completed with {} errors", stats.errors.len());
        }
        
        Ok(stats)
    }
    
    /// Import a single edge from JSON value
    fn import_edge_value<S: StorageBackend>(
        &self,
        value: &Value,
        node_id_map: &HashMap<String, String>,
        storage: &S,
        stats: &mut ImportStats,
    ) -> Result<()> {
        let obj = value.as_object()
            .ok_or_else(|| DeepGraphError::StorageError("Expected JSON object".to_string()))?;
        
        // Get from/to external IDs
        let from_external = obj.get("from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DeepGraphError::StorageError("Missing 'from' field".to_string()))?;
        
        let to_external = obj.get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DeepGraphError::StorageError("Missing 'to' field".to_string()))?;
        
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
        let rel_type = obj.get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DeepGraphError::StorageError("Missing 'type' field".to_string()))?
            .to_string();
        
        // Create edge
        let mut edge = Edge::new(from_id, to_id, rel_type);
        
        // Get properties
        if let Some(props) = obj.get("properties") {
            if let Some(props_obj) = props.as_object() {
                for (key, value) in props_obj {
                    let prop_value = self.json_to_property_value(value)?;
                    edge.set_property(key.clone(), prop_value);
                }
            }
        }
        
        // Add to storage
        storage.add_edge(edge)?;
        stats.record_edge();
        
        Ok(())
    }
}

impl Default for JsonImporter {
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
    fn test_json_to_property_value() {
        let importer = JsonImporter::new();
        
        // Null
        assert!(matches!(importer.json_to_property_value(&Value::Null).unwrap(), PropertyValue::Null));
        
        // Boolean
        assert!(matches!(importer.json_to_property_value(&Value::Bool(true)).unwrap(), PropertyValue::Boolean(true)));
        
        // Integer
        assert!(matches!(importer.json_to_property_value(&serde_json::json!(42)).unwrap(), PropertyValue::Integer(42)));
        
        // Float
        assert!(matches!(importer.json_to_property_value(&serde_json::json!(3.14)).unwrap(), PropertyValue::Float(_)));
        
        // String
        assert!(matches!(importer.json_to_property_value(&Value::String("hello".to_string())).unwrap(), PropertyValue::String(_)));
    }
    
    #[test]
    fn test_import_nodes() {
        // Create test JSON file
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"[
            {{"id": "1", "labels": ["Person"], "properties": {{"name": "Alice", "age": 30}}}},
            {{"id": "2", "labels": ["Person"], "properties": {{"name": "Bob", "age": 25}}}}
        ]"#).unwrap();
        
        // Import
        let storage = MemoryStorage::new();
        let importer = JsonImporter::new();
        let stats = importer.import_nodes(&storage, file.path()).unwrap();
        
        assert_eq!(stats.nodes_imported, 2);
        assert_eq!(stats.errors.len(), 0);
        assert_eq!(stats.node_id_map.len(), 2);
    }
}
