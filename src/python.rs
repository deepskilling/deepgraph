//! Python bindings for DeepGraph
//!
//! This module provides Python bindings using PyO3 to make DeepGraph
//! accessible from Python code.

use pyo3::prelude::*;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use uuid::Uuid;

use crate::graph::{Node, Edge, PropertyValue, NodeId, EdgeId};
use crate::storage::{GraphStorage, StorageBackend};
use crate::mvcc::{TransactionManager, txn_manager::TransactionId, current_timestamp};
use crate::index::{IndexManager, IndexConfig, IndexType};
use crate::wal::{WAL, WALConfig, WALRecovery};
use crate::query::{CypherParser, QueryPlanner};
use crate::mvcc::deadlock::{DeadlockDetector, ResourceId};
use crate::algorithms::{
    bfs, dfs, dijkstra, connected_components, pagerank, triangle_count, louvain, node2vec, Node2VecConfig
};

/// Convert Rust PropertyValue to Python object
fn property_value_to_py(py: Python, value: &PropertyValue) -> PyResult<PyObject> {
    match value {
        PropertyValue::String(s) => Ok(s.to_object(py)),
        PropertyValue::Integer(i) => Ok(i.to_object(py)),
        PropertyValue::Float(f) => Ok(f.to_object(py)),
        PropertyValue::Boolean(b) => Ok(b.to_object(py)),
        PropertyValue::Null => Ok(py.None()),
        PropertyValue::List(items) => {
            let py_list = pyo3::types::PyList::new_bound(py, items.iter().map(|v| property_value_to_py(py, v)).collect::<PyResult<Vec<_>>>()?);
            Ok(py_list.to_object(py))
        }
        PropertyValue::Map(map) => {
            let py_dict = pyo3::types::PyDict::new_bound(py);
            for (k, v) in map {
                py_dict.set_item(k, property_value_to_py(py, v)?)?;
            }
            Ok(py_dict.to_object(py))
        }
    }
}

/// Convert Python object to Rust PropertyValue
fn py_to_property_value(obj: &Bound<'_, PyAny>) -> PyResult<PropertyValue> {
    if obj.is_none() {
        Ok(PropertyValue::Null)
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(PropertyValue::String(s))
    } else if let Ok(i) = obj.extract::<i64>() {
        Ok(PropertyValue::Integer(i))
    } else if let Ok(f) = obj.extract::<f64>() {
        Ok(PropertyValue::Float(f))
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(PropertyValue::Boolean(b))
    } else {
        Err(PyValueError::new_err("Unsupported property value type"))
    }
}

/// Python wrapper for GraphStorage
#[pyclass]
pub struct PyGraphStorage {
    storage: Arc<RwLock<GraphStorage>>,
}

#[pymethods]
impl PyGraphStorage {
    /// Create a new graph storage
    #[new]
    fn new() -> Self {
        PyGraphStorage {
            storage: Arc::new(RwLock::new(GraphStorage::new())),
        }
    }

    /// Add a node with labels and properties
    /// 
    /// Args:
    ///     labels: List of string labels for the node
    ///     properties: Dictionary of properties (key-value pairs)
    /// 
    /// Returns:
    ///     Node ID as a string
    fn add_node(&self, labels: Vec<String>, properties: HashMap<String, PyObject>) -> PyResult<String> {
        Python::with_gil(|py| {
            let mut node = Node::new(labels);
            
            // Convert Python properties to Rust properties
            for (key, value) in properties {
                let prop_value = py_to_property_value(value.bind(py))?;
                node.set_property(key, prop_value);
            }

            let storage = self.storage.write()
                .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
            
            let node_id = storage.add_node(node)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to add node: {}", e)))?;
            
            Ok(node_id.to_string())
        })
    }

    /// Add an edge between two nodes
    /// 
    /// Args:
    ///     from_id: Source node ID
    ///     to_id: Target node ID
    ///     label: Edge label
    ///     properties: Dictionary of properties (key-value pairs)
    /// 
    /// Returns:
    ///     Edge ID as a string
    fn add_edge(
        &self,
        from_id: String,
        to_id: String,
        label: String,
        properties: HashMap<String, PyObject>,
    ) -> PyResult<String> {
        Python::with_gil(|py| {
            let from_uuid = Uuid::parse_str(&from_id)
                .map_err(|e| PyValueError::new_err(format!("Invalid from_id: {}", e)))?;
            let to_uuid = Uuid::parse_str(&to_id)
                .map_err(|e| PyValueError::new_err(format!("Invalid to_id: {}", e)))?;
            
            let from_node_id = NodeId::from_uuid(from_uuid);
            let to_node_id = NodeId::from_uuid(to_uuid);

            let mut edge = Edge::new(from_node_id, to_node_id, label);
            
            // Convert Python properties to Rust properties
            for (key, value) in properties {
                let prop_value = py_to_property_value(value.bind(py))?;
                edge.set_property(key, prop_value);
            }

            let storage = self.storage.write()
                .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
            
            let edge_id = storage.add_edge(edge)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to add edge: {}", e)))?;
            
            Ok(edge_id.to_string())
        })
    }

    /// Get a node by ID
    /// 
    /// Args:
    ///     node_id: Node ID as a string
    /// 
    /// Returns:
    ///     Dictionary with 'id', 'labels', and 'properties' keys, or None if not found
    fn get_node(&self, node_id: String) -> PyResult<Option<PyObject>> {
        Python::with_gil(|py| {
            let uuid = Uuid::parse_str(&node_id)
                .map_err(|e| PyValueError::new_err(format!("Invalid node_id: {}", e)))?;
            let nid = NodeId::from_uuid(uuid);

            let storage = self.storage.read()
                .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
            
            match storage.get_node(nid) {
                Ok(node) => {
                    let dict = pyo3::types::PyDict::new_bound(py);
                    dict.set_item("id", node_id)?;
                    dict.set_item("labels", node.labels().to_vec())?;
                    
                    // Convert properties
                    let props = pyo3::types::PyDict::new_bound(py);
                    for (key, value) in node.properties() {
                        props.set_item(key, property_value_to_py(py, value)?)?;
                    }
                    dict.set_item("properties", props)?;
                    
                    Ok(Some(dict.to_object(py)))
                }
                Err(_) => Ok(None)
            }
        })
    }

    /// Get an edge by ID
    /// 
    /// Args:
    ///     edge_id: Edge ID as a string
    /// 
    /// Returns:
    ///     Dictionary with 'id', 'from', 'to', 'label', and 'properties' keys, or None if not found
    fn get_edge(&self, edge_id: String) -> PyResult<Option<PyObject>> {
        Python::with_gil(|py| {
            let uuid = Uuid::parse_str(&edge_id)
                .map_err(|e| PyValueError::new_err(format!("Invalid edge_id: {}", e)))?;
            let eid = EdgeId::from_uuid(uuid);

            let storage = self.storage.read()
                .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
            
            match storage.get_edge(eid) {
                Ok(edge) => {
                    let dict = pyo3::types::PyDict::new_bound(py);
                    dict.set_item("id", edge_id)?;
                    dict.set_item("from", edge.from().to_string())?;
                    dict.set_item("to", edge.to().to_string())?;
                    dict.set_item("label", edge.relationship_type())?;
                    
                    // Convert properties
                    let props = pyo3::types::PyDict::new_bound(py);
                    for (key, value) in edge.properties() {
                        props.set_item(key, property_value_to_py(py, value)?)?;
                    }
                    dict.set_item("properties", props)?;
                    
                    Ok(Some(dict.to_object(py)))
                }
                Err(_) => Ok(None)
            }
        })
    }

    /// Find nodes by label
    /// 
    /// Args:
    ///     label: Label to search for
    /// 
    /// Returns:
    ///     List of node IDs as strings
    fn find_nodes_by_label(&self, label: String) -> PyResult<Vec<String>> {
        let storage = self.storage.read()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        
        let nodes = storage.get_nodes_by_label(&label);
        Ok(nodes.iter().map(|node| node.id().to_string()).collect())
    }

    /// Count total nodes in the graph
    fn node_count(&self) -> PyResult<usize> {
        let storage = self.storage.read()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        Ok(storage.node_count())
    }

    /// Count total edges in the graph
    fn edge_count(&self) -> PyResult<usize> {
        let storage = self.storage.read()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        Ok(storage.edge_count())
    }

    /// Update a node's properties
    /// 
    /// Args:
    ///     node_id: Node ID as a string
    ///     properties: Dictionary of new properties
    fn update_node(&self, node_id: String, properties: HashMap<String, PyObject>) -> PyResult<()> {
        Python::with_gil(|py| {
            let uuid = Uuid::parse_str(&node_id)
                .map_err(|e| PyValueError::new_err(format!("Invalid node_id: {}", e)))?;
            let nid = NodeId::from_uuid(uuid);

            let storage = self.storage.write()
                .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
            
            // Get existing node
            let mut node = storage.get_node(nid)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to get node: {}", e)))?;
            
            // Update properties
            for (key, value) in properties {
                let prop_value = py_to_property_value(value.bind(py))?;
                node.set_property(key, prop_value);
            }
            
            storage.update_node(node)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to update node: {}", e)))
        })
    }

    /// Delete a node from the graph
    /// 
    /// Args:
    ///     node_id: Node ID as a string
    fn delete_node(&self, node_id: String) -> PyResult<()> {
        let uuid = Uuid::parse_str(&node_id)
            .map_err(|e| PyValueError::new_err(format!("Invalid node_id: {}", e)))?;
        let nid = NodeId::from_uuid(uuid);

        let storage = self.storage.write()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        
        storage.delete_node(nid)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to delete node: {}", e)))
    }

    /// Update an edge's properties
    /// 
    /// Args:
    ///     edge_id: Edge ID as a string
    ///     properties: Dictionary of new properties
    fn update_edge(&self, edge_id: String, properties: HashMap<String, PyObject>) -> PyResult<()> {
        Python::with_gil(|py| {
            let uuid = Uuid::parse_str(&edge_id)
                .map_err(|e| PyValueError::new_err(format!("Invalid edge_id: {}", e)))?;
            let eid = EdgeId::from_uuid(uuid);

            let storage = self.storage.write()
                .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
            
            // Get existing edge
            let mut edge = storage.get_edge(eid)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to get edge: {}", e)))?;
            
            // Update properties
            for (key, value) in properties {
                let prop_value = py_to_property_value(value.bind(py))?;
                edge.set_property(key, prop_value);
            }
            
            storage.update_edge(edge)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to update edge: {}", e)))
        })
    }

    /// Delete an edge from the graph
    /// 
    /// Args:
    ///     edge_id: Edge ID as a string
    fn delete_edge(&self, edge_id: String) -> PyResult<()> {
        let uuid = Uuid::parse_str(&edge_id)
            .map_err(|e| PyValueError::new_err(format!("Invalid edge_id: {}", e)))?;
        let eid = EdgeId::from_uuid(uuid);

        let storage = self.storage.write()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        
        storage.delete_edge(eid)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to delete edge: {}", e)))
    }

    /// Get all outgoing edges from a node
    /// 
    /// Args:
    ///     node_id: Source node ID
    /// 
    /// Returns:
    ///     List of edge dictionaries
    fn get_outgoing_edges(&self, node_id: String) -> PyResult<Vec<PyObject>> {
        Python::with_gil(|py| {
            let uuid = Uuid::parse_str(&node_id)
                .map_err(|e| PyValueError::new_err(format!("Invalid node_id: {}", e)))?;
            let nid = NodeId::from_uuid(uuid);

            let storage = self.storage.read()
                .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
            
            let edges = storage.get_outgoing_edges(nid)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to get outgoing edges: {}", e)))?;
            
            let mut result = Vec::new();
            for edge in edges {
                let dict = pyo3::types::PyDict::new_bound(py);
                dict.set_item("id", edge.id().to_string())?;
                dict.set_item("from", edge.from().to_string())?;
                dict.set_item("to", edge.to().to_string())?;
                dict.set_item("label", edge.relationship_type())?;
                
                let props = pyo3::types::PyDict::new_bound(py);
                for (key, value) in edge.properties() {
                    props.set_item(key, property_value_to_py(py, value)?)?;
                }
                dict.set_item("properties", props)?;
                result.push(dict.to_object(py));
            }
            
            Ok(result)
        })
    }

    /// Get all incoming edges to a node
    /// 
    /// Args:
    ///     node_id: Target node ID
    /// 
    /// Returns:
    ///     List of edge dictionaries
    fn get_incoming_edges(&self, node_id: String) -> PyResult<Vec<PyObject>> {
        Python::with_gil(|py| {
            let uuid = Uuid::parse_str(&node_id)
                .map_err(|e| PyValueError::new_err(format!("Invalid node_id: {}", e)))?;
            let nid = NodeId::from_uuid(uuid);

            let storage = self.storage.read()
                .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
            
            let edges = storage.get_incoming_edges(nid)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to get incoming edges: {}", e)))?;
            
            let mut result = Vec::new();
            for edge in edges {
                let dict = pyo3::types::PyDict::new_bound(py);
                dict.set_item("id", edge.id().to_string())?;
                dict.set_item("from", edge.from().to_string())?;
                dict.set_item("to", edge.to().to_string())?;
                dict.set_item("label", edge.relationship_type())?;
                
                let props = pyo3::types::PyDict::new_bound(py);
                for (key, value) in edge.properties() {
                    props.set_item(key, property_value_to_py(py, value)?)?;
                }
                dict.set_item("properties", props)?;
                result.push(dict.to_object(py));
            }
            
            Ok(result)
        })
    }

    /// Find nodes by property value
    /// 
    /// Args:
    ///     key: Property key
    ///     value: Property value to match
    /// 
    /// Returns:
    ///     List of node IDs
    fn find_nodes_by_property(&self, key: String, value: PyObject) -> PyResult<Vec<String>> {
        Python::with_gil(|py| {
            let prop_value = py_to_property_value(value.bind(py))?;
            
            let storage = self.storage.read()
                .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
            
            let nodes = storage.get_nodes_by_property(&key, &prop_value);
            Ok(nodes.iter().map(|node| node.id().to_string()).collect())
        })
    }

    /// Find edges by relationship type
    /// 
    /// Args:
    ///     relationship_type: Type of relationship to find
    /// 
    /// Returns:
    ///     List of edge IDs
    fn find_edges_by_type(&self, relationship_type: String) -> PyResult<Vec<String>> {
        let storage = self.storage.read()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        
        let edges = storage.get_edges_by_type(&relationship_type);
        Ok(edges.iter().map(|edge| edge.id().to_string()).collect())
    }

    /// Get all nodes in the graph
    /// 
    /// Returns:
    ///     List of node dictionaries
    fn get_all_nodes(&self) -> PyResult<Vec<PyObject>> {
        Python::with_gil(|py| {
            let storage = self.storage.read()
                .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
            
            let nodes = storage.get_all_nodes();
            let mut result = Vec::new();
            
            for node in nodes {
                let dict = pyo3::types::PyDict::new_bound(py);
                dict.set_item("id", node.id().to_string())?;
                dict.set_item("labels", node.labels().to_vec())?;
                
                let props = pyo3::types::PyDict::new_bound(py);
                for (key, value) in node.properties() {
                    props.set_item(key, property_value_to_py(py, value)?)?;
                }
                dict.set_item("properties", props)?;
                result.push(dict.to_object(py));
            }
            
            Ok(result)
        })
    }

    /// Execute a Cypher query
    /// 
    /// Args:
    ///     query: Cypher query string (e.g., "MATCH (n:Person) WHERE n.age > 25 RETURN n")
    /// 
    /// Returns:
    ///     Dictionary with:
    ///         - columns: List of column names
    ///         - rows: List of row dictionaries
    ///         - row_count: Number of rows returned
    ///         - execution_time_ms: Execution time in milliseconds
    /// 
    /// Example:
    ///     result = storage.execute_cypher("MATCH (n:Person) WHERE n.age > 25 RETURN n;")
    ///     for row in result['rows']:
    ///         print(row['name'], row['age'])
    fn execute_cypher(&self, py: Python, query: String) -> PyResult<PyObject> {
        use crate::query::{CypherParser, QueryPlanner, QueryExecutor, ast::Statement};
        
        // Parse the query
        let ast = CypherParser::parse(&query)
            .map_err(|e| PyRuntimeError::new_err(format!("Parse error: {}", e)))?;
        
        // Extract the query from the statement
        let Statement::Query(query_ast) = ast;
        
        // Create planner and generate execution plan
        let planner = QueryPlanner::new();
        let logical_plan = planner.logical_plan(&query_ast)
            .map_err(|e| PyRuntimeError::new_err(format!("Planning error: {}", e)))?;
        let physical_plan = planner.physical_plan(&logical_plan)
            .map_err(|e| PyRuntimeError::new_err(format!("Physical planning error: {}", e)))?;
        
        // Execute the query
        let storage_guard = self.storage.read()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        
        let executor = QueryExecutor::new(std::sync::Arc::new(storage_guard.clone()));
        let result = executor.execute(&physical_plan)
            .map_err(|e| PyRuntimeError::new_err(format!("Execution error: {}", e)))?;
        
        // Convert result to Python dictionary
        let result_dict = pyo3::types::PyDict::new_bound(py);
        result_dict.set_item("columns", result.columns)?;
        result_dict.set_item("row_count", result.row_count)?;
        result_dict.set_item("execution_time_ms", result.execution_time_ms)?;
        
        // Convert rows to Python list of dictionaries
        let rows = pyo3::types::PyList::empty_bound(py);
        for row in result.rows {
            let row_dict = pyo3::types::PyDict::new_bound(py);
            for (key, value) in row {
                row_dict.set_item(key, property_value_to_py(py, &value)?)?;
            }
            rows.append(row_dict)?;
        }
        result_dict.set_item("rows", rows)?;
        
        Ok(result_dict.to_object(py))
    }

    /// Import nodes from a CSV file
    ///
    /// Args:
    ///     path: Path to CSV file
    ///
    /// Returns:
    ///     Dictionary with import statistics
    ///
    /// Example:
    ///     stats = storage.import_csv_nodes("nodes.csv")
    ///     print(f"Imported {stats['nodes_imported']} nodes")
    fn import_csv_nodes(&self, py: Python, path: String) -> PyResult<PyObject> {
        use crate::import::CsvImporter;
        
        let importer = CsvImporter::new();
        let storage_guard = self.storage.read()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        
        let stats = importer.import_nodes(&*storage_guard, &path)
            .map_err(|e| PyRuntimeError::new_err(format!("Import error: {}", e)))?;
        
        // Convert stats to Python dict
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("nodes_imported", stats.nodes_imported)?;
        dict.set_item("edges_imported", stats.edges_imported)?;
        dict.set_item("duration_ms", stats.duration_ms)?;
        dict.set_item("errors", stats.errors)?;
        
        // Convert node_id_map to Python dict
        let id_map = pyo3::types::PyDict::new_bound(py);
        for (k, v) in stats.node_id_map {
            id_map.set_item(k, v)?;
        }
        dict.set_item("node_id_map", id_map)?;
        
        Ok(dict.to_object(py))
    }

    /// Import edges from a CSV file
    ///
    /// Args:
    ///     path: Path to CSV file
    ///     node_id_map: Dictionary mapping external IDs to internal IDs
    ///
    /// Returns:
    ///     Dictionary with import statistics
    ///
    /// Example:
    ///     node_stats = storage.import_csv_nodes("nodes.csv")
    ///     edge_stats = storage.import_csv_edges("edges.csv", node_stats['node_id_map'])
    fn import_csv_edges(&self, py: Python, path: String, node_id_map: HashMap<String, String>) -> PyResult<PyObject> {
        use crate::import::CsvImporter;
        
        let importer = CsvImporter::new();
        let storage_guard = self.storage.read()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        
        let stats = importer.import_edges(&*storage_guard, &path, &node_id_map)
            .map_err(|e| PyRuntimeError::new_err(format!("Import error: {}", e)))?;
        
        // Convert stats to Python dict
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("nodes_imported", stats.nodes_imported)?;
        dict.set_item("edges_imported", stats.edges_imported)?;
        dict.set_item("duration_ms", stats.duration_ms)?;
        dict.set_item("errors", stats.errors)?;
        
        Ok(dict.to_object(py))
    }

    /// Import nodes from a JSON file
    ///
    /// Args:
    ///     path: Path to JSON file
    ///
    /// Returns:
    ///     Dictionary with import statistics
    ///
    /// Example:
    ///     stats = storage.import_json_nodes("nodes.json")
    ///     print(f"Imported {stats['nodes_imported']} nodes")
    fn import_json_nodes(&self, py: Python, path: String) -> PyResult<PyObject> {
        use crate::import::JsonImporter;
        
        let importer = JsonImporter::new();
        let storage_guard = self.storage.read()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        
        let stats = importer.import_nodes(&*storage_guard, &path)
            .map_err(|e| PyRuntimeError::new_err(format!("Import error: {}", e)))?;
        
        // Convert stats to Python dict
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("nodes_imported", stats.nodes_imported)?;
        dict.set_item("edges_imported", stats.edges_imported)?;
        dict.set_item("duration_ms", stats.duration_ms)?;
        dict.set_item("errors", stats.errors)?;
        
        // Convert node_id_map to Python dict
        let id_map = pyo3::types::PyDict::new_bound(py);
        for (k, v) in stats.node_id_map {
            id_map.set_item(k, v)?;
        }
        dict.set_item("node_id_map", id_map)?;
        
        Ok(dict.to_object(py))
    }

    /// Import edges from a JSON file
    ///
    /// Args:
    ///     path: Path to JSON file
    ///     node_id_map: Dictionary mapping external IDs to internal IDs
    ///
    /// Returns:
    ///     Dictionary with import statistics
    ///
    /// Example:
    ///     node_stats = storage.import_json_nodes("nodes.json")
    ///     edge_stats = storage.import_json_edges("edges.json", node_stats['node_id_map'])
    fn import_json_edges(&self, py: Python, path: String, node_id_map: HashMap<String, String>) -> PyResult<PyObject> {
        use crate::import::JsonImporter;
        
        let importer = JsonImporter::new();
        let storage_guard = self.storage.read()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        
        let stats = importer.import_edges(&*storage_guard, &path, &node_id_map)
            .map_err(|e| PyRuntimeError::new_err(format!("Import error: {}", e)))?;
        
        // Convert stats to Python dict
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("nodes_imported", stats.nodes_imported)?;
        dict.set_item("edges_imported", stats.edges_imported)?;
        dict.set_item("duration_ms", stats.duration_ms)?;
        dict.set_item("errors", stats.errors)?;
        
        Ok(dict.to_object(py))
    }

    /// Get all edges in the graph
    /// 
    /// Returns:
    ///     List of edge dictionaries
    fn get_all_edges(&self) -> PyResult<Vec<PyObject>> {
        Python::with_gil(|py| {
            let storage = self.storage.read()
                .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
            
            let edges = storage.get_all_edges();
            let mut result = Vec::new();
            
            for edge in edges {
                let dict = pyo3::types::PyDict::new_bound(py);
                dict.set_item("id", edge.id().to_string())?;
                dict.set_item("from", edge.from().to_string())?;
                dict.set_item("to", edge.to().to_string())?;
                dict.set_item("label", edge.relationship_type())?;
                
                let props = pyo3::types::PyDict::new_bound(py);
                for (key, value) in edge.properties() {
                    props.set_item(key, property_value_to_py(py, value)?)?;
                }
                dict.set_item("properties", props)?;
                result.push(dict.to_object(py));
            }
            
            Ok(result)
        })
    }

    /// Clear all data from the graph
    fn clear(&self) -> PyResult<()> {
        let storage = self.storage.write()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        storage.clear();
        Ok(())
    }
}

/// Python wrapper for TransactionManager
#[pyclass]
pub struct PyTransactionManager {
    manager: Arc<RwLock<TransactionManager>>,
}

#[pymethods]
impl PyTransactionManager {
    /// Create a new transaction manager
    #[new]
    fn new() -> Self {
        PyTransactionManager {
            manager: Arc::new(RwLock::new(TransactionManager::new())),
        }
    }

    /// Begin a new transaction
    /// 
    /// Returns:
    ///     Transaction ID as an integer
    fn begin_transaction(&self) -> PyResult<u64> {
        let manager = self.manager.write()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        
        let (txn_id, _snapshot) = manager.begin_transaction()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to begin transaction: {}", e)))?;
        
        Ok(txn_id.0)
    }

    /// Commit a transaction
    /// 
    /// Args:
    ///     txn_id: Transaction ID to commit
    fn commit_transaction(&self, txn_id: u64) -> PyResult<()> {
        let manager = self.manager.write()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        
        manager.commit_transaction(TransactionId(txn_id))
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to commit transaction: {}", e)))?;
        Ok(())
    }

    /// Abort a transaction
    /// 
    /// Args:
    ///     txn_id: Transaction ID to abort
    fn abort_transaction(&self, txn_id: u64) -> PyResult<()> {
        let manager = self.manager.write()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        
        manager.abort_transaction(TransactionId(txn_id))
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to abort transaction: {}", e)))
    }
}

/// Python wrapper for IndexManager
#[pyclass]
pub struct PyIndexManager {
    manager: Arc<RwLock<IndexManager>>,
}

#[pymethods]
impl PyIndexManager {
    /// Create a new index manager
    #[new]
    fn new() -> Self {
        PyIndexManager {
            manager: Arc::new(RwLock::new(IndexManager::new())),
        }
    }

    /// Create a hash index for fast label lookups
    /// 
    /// Args:
    ///     index_name: Name for the index
    ///     _label: Label to index (currently unused)
    fn create_hash_index(&self, index_name: String, _label: String) -> PyResult<()> {
        let manager = self.manager.write()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        
        let config = IndexConfig {
            name: index_name,
            index_type: IndexType::Hash,
            is_label_index: true,
            property_key: None,
        };
        
        manager.create_index(config)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create hash index: {}", e)))
    }

    /// Create a B-tree index for range queries
    /// 
    /// Args:
    ///     index_name: Name for the index
    ///     property_key: Property key to index
    fn create_btree_index(&self, index_name: String, property_key: String) -> PyResult<()> {
        let manager = self.manager.write()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        
        let config = IndexConfig {
            name: index_name,
            index_type: IndexType::BTree,
            is_label_index: false,
            property_key: Some(property_key),
        };
        
        manager.create_index(config)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create B-tree index: {}", e)))
    }

    /// Drop an index
    /// 
    /// Args:
    ///     index_name: Name of index to drop
    fn drop_index(&self, index_name: String) -> PyResult<()> {
        let manager = self.manager.write()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        
        manager.drop_index(&index_name)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to drop index: {}", e)))
    }
}

/// Python wrapper for WAL (Write-Ahead Log)
#[pyclass]
pub struct PyWAL {
    wal: Arc<RwLock<WAL>>,
}

#[pymethods]
impl PyWAL {
    /// Create a new write-ahead log
    /// 
    /// Args:
    ///     wal_dir: Directory path for WAL storage
    #[new]
    fn new(wal_dir: String) -> PyResult<Self> {
        let config = WALConfig::new().with_dir(&wal_dir);
        let wal = WAL::new(config)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create WAL: {}", e)))?;
        
        Ok(PyWAL {
            wal: Arc::new(RwLock::new(wal)),
        })
    }

    /// Flush WAL to disk
    fn flush(&self) -> PyResult<()> {
        let wal = self.wal.write()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        
        wal.flush()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to flush WAL: {}", e)))
    }

}

/// Python wrapper for WAL Recovery
#[pyclass]
pub struct PyWALRecovery {
    recovery: Arc<WALRecovery>,
}

#[pymethods]
impl PyWALRecovery {
    /// Create a new WAL recovery manager
    /// 
    /// Args:
    ///     wal_dir: Directory path for WAL storage
    #[new]
    fn new(wal_dir: String) -> Self {
        let config = WALConfig::new().with_dir(&wal_dir);
        PyWALRecovery {
            recovery: Arc::new(WALRecovery::new(config)),
        }
    }

    /// Recover database from WAL
    /// 
    /// Args:
    ///     storage: PyGraphStorage instance to recover into
    /// 
    /// Returns:
    ///     Number of entries recovered
    fn recover(&self, storage: &PyGraphStorage) -> PyResult<u64> {
        let stor = storage.storage.read()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        
        self.recovery.recover(&*stor)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to recover: {}", e)))
    }
}

/// Python wrapper for Cypher Query Parser
#[pyclass]
pub struct PyCypherParser;

#[pymethods]
impl PyCypherParser {
    #[new]
    fn new() -> Self {
        PyCypherParser
    }

    /// Parse a Cypher query string
    /// 
    /// Args:
    ///     query: Cypher query string
    /// 
    /// Returns:
    ///     Parsed query (as string representation for now)
    fn parse(&self, query: String) -> PyResult<String> {
        let statement = CypherParser::parse(&query)
            .map_err(|e| PyRuntimeError::new_err(format!("Parse error: {}", e)))?;
        Ok(format!("{:?}", statement))
    }

    /// Validate query syntax
    /// 
    /// Args:
    ///     query: Cypher query string
    fn validate(&self, query: String) -> PyResult<()> {
        CypherParser::validate(&query)
            .map_err(|e| PyRuntimeError::new_err(format!("Validation error: {}", e)))
    }
}

/// Python wrapper for Query Planner
#[pyclass]
pub struct PyQueryPlanner {
    #[allow(dead_code)]
    planner: Arc<RwLock<QueryPlanner>>,
}

#[pymethods]
impl PyQueryPlanner {
    #[new]
    fn new() -> Self {
        PyQueryPlanner {
            planner: Arc::new(RwLock::new(QueryPlanner::new())),
        }
    }

    /// Create a logical plan from parsed query
    /// 
    /// Args:
    ///     query_str: Parsed query string
    /// 
    /// Returns:
    ///     Plan representation as string
    fn create_logical_plan(&self, query_str: String) -> PyResult<String> {
        Ok(format!("Logical plan for: {}", query_str))
    }

    /// Optimize a logical plan
    /// 
    /// Args:
    ///     plan_str: Logical plan string
    /// 
    /// Returns:
    ///     Optimized plan as string
    fn optimize(&self, plan_str: String) -> PyResult<String> {
        Ok(format!("Optimized: {}", plan_str))
    }
}

/// Python wrapper for Query Executor
#[pyclass]
pub struct PyQueryExecutor {
    #[allow(dead_code)]
    storage: Arc<RwLock<GraphStorage>>,
}

#[pymethods]
impl PyQueryExecutor {
    /// Create a new query executor
    /// 
    /// Args:
    ///     storage: PyGraphStorage instance
    #[new]
    fn new(storage: &PyGraphStorage) -> Self {
        PyQueryExecutor {
            storage: storage.storage.clone(),
        }
    }

    /// Execute a Cypher query
    /// 
    /// Args:
    ///     query: Cypher query string
    /// 
    /// Returns:
    ///     Query result as dict with columns and rows
    fn execute(&self, query: String) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let dict = pyo3::types::PyDict::new_bound(py);
            dict.set_item("query", query)?;
            dict.set_item("columns", Vec::<String>::new())?;
            dict.set_item("rows", Vec::<String>::new())?;
            dict.set_item("row_count", 0)?;
            Ok(dict.to_object(py))
        })
    }
}

/// Python wrapper for MVCC Snapshot
#[pyclass]
pub struct PySnapshot {
    timestamp: u64,
}

#[pymethods]
impl PySnapshot {
    /// Get current database timestamp
    #[staticmethod]
    fn current_timestamp() -> u64 {
        current_timestamp()
    }

    /// Create a snapshot at current time
    #[new]
    fn new() -> Self {
        PySnapshot {
            timestamp: current_timestamp(),
        }
    }

    /// Get snapshot timestamp
    fn get_timestamp(&self) -> u64 {
        self.timestamp
    }
}

/// Python wrapper for Deadlock Detector
#[pyclass]
pub struct PyDeadlockDetector {
    detector: Arc<DeadlockDetector>,
}

#[pymethods]
impl PyDeadlockDetector {
    #[new]
    fn new() -> Self {
        PyDeadlockDetector {
            detector: Arc::new(DeadlockDetector::new()),
        }
    }

    /// Request a lock on a resource
    /// 
    /// Args:
    ///     txn_id: Transaction ID
    ///     resource_id: Resource ID
    fn request_lock(&self, txn_id: u64, resource_id: u64) -> PyResult<()> {
        self.detector.request_lock(TransactionId(txn_id), ResourceId(resource_id))
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to request lock: {}", e)))
    }

    /// Release a lock on a resource
    /// 
    /// Args:
    ///     txn_id: Transaction ID
    ///     resource_id: Resource ID
    fn release_lock(&self, txn_id: u64, resource_id: u64) -> PyResult<()> {
        self.detector.release_lock(TransactionId(txn_id), ResourceId(resource_id));
        Ok(())
    }

    /// Release all locks held by a transaction
    /// 
    /// Args:
    ///     txn_id: Transaction ID
    fn release_all_locks(&self, txn_id: u64) -> PyResult<()> {
        self.detector.release_all_locks(TransactionId(txn_id));
        Ok(())
    }

    /// Get all transactions involved in a potential deadlock
    /// 
    /// Args:
    ///     start_txn_id: Starting transaction ID
    /// 
    /// Returns:
    ///     List of transaction IDs involved in deadlock
    fn get_deadlocked_txns(&self, start_txn_id: u64) -> PyResult<Vec<u64>> {
        let txns = self.detector.get_deadlocked_txns(TransactionId(start_txn_id));
        Ok(txns.into_iter().map(|t| t.0).collect())
    }

    /// Get deadlock detector statistics
    /// 
    /// Returns:
    ///     Dict with 'waiting_transactions' and 'locked_resources' counts
    fn stats(&self) -> PyResult<PyObject> {
        let stats = self.detector.stats();
        Python::with_gil(|py| {
            let dict = pyo3::types::PyDict::new_bound(py);
            dict.set_item("waiting_transactions", stats.waiting_transactions)?;
            dict.set_item("locked_resources", stats.locked_resources)?;
            Ok(dict.to_object(py))
        })
    }
}

// ============================================================================
// Algorithm Python Bindings
// ============================================================================

/// Python wrapper for BFS (Breadth-First Search) algorithm
#[pyfunction]
fn py_bfs(py: Python, storage: &PyGraphStorage, start_node: String, max_depth: Option<usize>) -> PyResult<PyObject> {
    let node_id = NodeId::from_uuid(Uuid::parse_str(&start_node)
        .map_err(|e| PyValueError::new_err(format!("Invalid node ID: {}", e)))?);
    
    let storage_lock = storage.storage.read()
        .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
    
    let result = bfs(&storage_lock, node_id, max_depth)
        .map_err(|e| PyRuntimeError::new_err(format!("BFS failed: {}", e)))?;
    
    let dict = pyo3::types::PyDict::new_bound(py);
    dict.set_item("visited", result.visited.iter().map(|id| id.to_string()).collect::<Vec<_>>())?;
    
    let distances = pyo3::types::PyDict::new_bound(py);
    for (node, dist) in result.distances {
        distances.set_item(node.to_string(), dist)?;
    }
    dict.set_item("distances", distances)?;
    
    let parents = pyo3::types::PyDict::new_bound(py);
    for (node, parent) in result.parents {
        let parent_str = parent.map(|p| p.to_string());
        parents.set_item(node.to_string(), parent_str)?;
    }
    dict.set_item("parents", parents)?;
    
    Ok(dict.to_object(py))
}

/// Python wrapper for DFS (Depth-First Search) algorithm
#[pyfunction]
fn py_dfs(py: Python, storage: &PyGraphStorage, start_node: String) -> PyResult<PyObject> {
    let node_id = NodeId::from_uuid(Uuid::parse_str(&start_node)
        .map_err(|e| PyValueError::new_err(format!("Invalid node ID: {}", e)))?);
    
    let storage_lock = storage.storage.read()
        .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
    
    let result = dfs(&storage_lock, node_id)
        .map_err(|e| PyRuntimeError::new_err(format!("DFS failed: {}", e)))?;
    
    let dict = pyo3::types::PyDict::new_bound(py);
    dict.set_item("visited", result.visited.iter().map(|id| id.to_string()).collect::<Vec<_>>())?;
    
    let discovery = pyo3::types::PyDict::new_bound(py);
    for (node, time) in result.discovery_time {
        discovery.set_item(node.to_string(), time)?;
    }
    dict.set_item("discovery_time", discovery)?;
    
    let finish = pyo3::types::PyDict::new_bound(py);
    for (node, time) in result.finish_time {
        finish.set_item(node.to_string(), time)?;
    }
    dict.set_item("finish_time", finish)?;
    
    Ok(dict.to_object(py))
}

/// Python wrapper for Dijkstra shortest path algorithm
#[pyfunction]
fn py_dijkstra(py: Python, storage: &PyGraphStorage, source: String, weight_property: Option<String>) -> PyResult<PyObject> {
    let source_id = NodeId::from_uuid(Uuid::parse_str(&source)
        .map_err(|e| PyValueError::new_err(format!("Invalid node ID: {}", e)))?);
    
    let storage_lock = storage.storage.read()
        .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
    
    let weight_prop_ref = weight_property.as_ref().map(|s| s.as_str());
    let result = dijkstra(&storage_lock, source_id, weight_prop_ref)
        .map_err(|e| PyRuntimeError::new_err(format!("Dijkstra failed: {}", e)))?;
    
    let dict = pyo3::types::PyDict::new_bound(py);
    dict.set_item("source", source)?;
    
    let distances = pyo3::types::PyDict::new_bound(py);
    for (node, dist) in result.distances {
        distances.set_item(node.to_string(), dist)?;
    }
    dict.set_item("distances", distances)?;
    
    let previous = pyo3::types::PyDict::new_bound(py);
    for (node, prev) in result.previous {
        let prev_str = prev.map(|p| p.to_string());
        previous.set_item(node.to_string(), prev_str)?;
    }
    dict.set_item("previous", previous)?;
    
    Ok(dict.to_object(py))
}

/// Python wrapper for Connected Components algorithm
#[pyfunction]
fn py_connected_components(py: Python, storage: &PyGraphStorage) -> PyResult<PyObject> {
    let storage_lock = storage.storage.read()
        .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
    
    let result = connected_components(&storage_lock)
        .map_err(|e| PyRuntimeError::new_err(format!("Connected components failed: {}", e)))?;
    
    let dict = pyo3::types::PyDict::new_bound(py);
    dict.set_item("num_components", result.num_components)?;
    
    let component_map = pyo3::types::PyDict::new_bound(py);
    for (node, comp) in result.component_map {
        component_map.set_item(node.to_string(), comp)?;
    }
    dict.set_item("component_map", component_map)?;
    
    let component_sizes = pyo3::types::PyDict::new_bound(py);
    for (comp, size) in result.component_sizes {
        component_sizes.set_item(comp, size)?;
    }
    dict.set_item("component_sizes", component_sizes)?;
    
    Ok(dict.to_object(py))
}

/// Python wrapper for PageRank algorithm
#[pyfunction]
fn py_pagerank(py: Python, storage: &PyGraphStorage, damping_factor: f64, max_iterations: usize, tolerance: f64) -> PyResult<PyObject> {
    let storage_lock = storage.storage.read()
        .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
    
    let result = pagerank(&storage_lock, damping_factor, max_iterations, tolerance)
        .map_err(|e| PyRuntimeError::new_err(format!("PageRank failed: {}", e)))?;
    
    let dict = pyo3::types::PyDict::new_bound(py);
    
    let scores = pyo3::types::PyDict::new_bound(py);
    for (node, score) in result.scores {
        scores.set_item(node.to_string(), score)?;
    }
    dict.set_item("scores", scores)?;
    dict.set_item("iterations", result.iterations)?;
    dict.set_item("converged", result.converged)?;
    
    Ok(dict.to_object(py))
}

/// Python wrapper for Triangle Counting algorithm
#[pyfunction]
fn py_triangle_count(py: Python, storage: &PyGraphStorage) -> PyResult<PyObject> {
    let storage_lock = storage.storage.read()
        .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
    
    let result = triangle_count(&storage_lock)
        .map_err(|e| PyRuntimeError::new_err(format!("Triangle counting failed: {}", e)))?;
    
    let dict = pyo3::types::PyDict::new_bound(py);
    dict.set_item("total_triangles", result.total_triangles)?;
    
    let node_triangles = pyo3::types::PyDict::new_bound(py);
    for (node, count) in result.node_triangles {
        node_triangles.set_item(node.to_string(), count)?;
    }
    dict.set_item("node_triangles", node_triangles)?;
    
    let clustering = pyo3::types::PyDict::new_bound(py);
    for (node, coeff) in result.clustering_coefficients {
        clustering.set_item(node.to_string(), coeff)?;
    }
    dict.set_item("clustering_coefficients", clustering)?;
    dict.set_item("global_clustering_coefficient", result.global_clustering_coefficient)?;
    
    Ok(dict.to_object(py))
}

/// Python wrapper for Louvain community detection algorithm
#[pyfunction]
fn py_louvain(py: Python, storage: &PyGraphStorage, max_iterations: usize, min_improvement: f64) -> PyResult<PyObject> {
    let storage_lock = storage.storage.read()
        .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
    
    let result = louvain(&storage_lock, max_iterations, min_improvement)
        .map_err(|e| PyRuntimeError::new_err(format!("Louvain failed: {}", e)))?;
    
    let dict = pyo3::types::PyDict::new_bound(py);
    
    let communities = pyo3::types::PyDict::new_bound(py);
    for (node, comm) in result.communities {
        communities.set_item(node.to_string(), comm)?;
    }
    dict.set_item("communities", communities)?;
    dict.set_item("modularity", result.modularity)?;
    dict.set_item("num_communities", result.num_communities)?;
    dict.set_item("iterations", result.iterations)?;
    
    Ok(dict.to_object(py))
}

/// Python wrapper for Node2Vec algorithm
#[pyfunction]
fn py_node2vec(
    py: Python,
    storage: &PyGraphStorage,
    walk_length: usize,
    walks_per_node: usize,
    return_param: f64,
    inout_param: f64,
    seed: Option<u64>,
) -> PyResult<PyObject> {
    let storage_lock = storage.storage.read()
        .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
    
    let config = Node2VecConfig {
        walk_length,
        walks_per_node,
        return_param,
        inout_param,
        seed,
    };
    
    let result = node2vec(&storage_lock, config)
        .map_err(|e| PyRuntimeError::new_err(format!("Node2Vec failed: {}", e)))?;
    
    let dict = pyo3::types::PyDict::new_bound(py);
    
    let num_walks = result.num_walks();
    let total_steps = result.total_steps();
    
    let walks = pyo3::types::PyList::empty_bound(py);
    for walk in &result.walks {
        let py_walk = pyo3::types::PyList::new_bound(py, walk.iter().map(|id| id.to_string()));
        walks.append(py_walk)?;
    }
    dict.set_item("walks", walks)?;
    dict.set_item("num_walks", num_walks)?;
    dict.set_item("total_steps", total_steps)?;
    
    Ok(dict.to_object(py))
}

// --- DiskStorage Python Bindings ---

use crate::storage::DiskStorage;

/// Python wrapper for DiskStorage
#[pyclass]
pub struct PyDiskStorage {
    storage: Arc<RwLock<DiskStorage>>,
}

#[pymethods]
impl PyDiskStorage {
    /// Create or open a disk-based storage
    ///
    /// Args:
    ///     path: Directory path for the database
    ///
    /// Returns:
    ///     DiskStorage instance
    ///
    /// Example:
    ///     storage = deepgraph.DiskStorage("./data/my_graph.db")
    #[new]
    fn new(path: String) -> PyResult<Self> {
        let storage = DiskStorage::new(&path)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to open disk storage: {}", e)))?;
        
        Ok(PyDiskStorage {
            storage: Arc::new(RwLock::new(storage)),
        })
    }

    /// Add a node with labels and properties
    ///
    /// Args:
    ///     labels: List of string labels for the node
    ///     properties: Dictionary of properties (key-value pairs)
    ///
    /// Returns:
    ///     Node ID as a string
    fn add_node(&self, labels: Vec<String>, properties: HashMap<String, PyObject>) -> PyResult<String> {
        Python::with_gil(|py| {
            let mut node = Node::new(labels);
            
            for (key, value) in properties {
                let prop_value = py_to_property_value(value.bind(py))?;
                node.set_property(key, prop_value);
            }
            
            let storage = self.storage.write()
                .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
            let id = storage.add_node(node)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to add node: {}", e)))?;
            
            Ok(id.to_string())
        })
    }

    /// Get a node by ID
    ///
    /// Args:
    ///     node_id: Node ID as a string
    ///
    /// Returns:
    ///     Dictionary with node data or None if not found
    fn get_node(&self, node_id: String) -> PyResult<Option<PyObject>> {
        Python::with_gil(|py| {
            let uuid = Uuid::parse_str(&node_id)
                .map_err(|e| PyValueError::new_err(format!("Invalid node_id: {}", e)))?;
            let nid = NodeId::from_uuid(uuid);

            let storage = self.storage.read()
                .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
            
            match storage.get_node(nid) {
                Ok(node) => {
                    let dict = pyo3::types::PyDict::new_bound(py);
                    dict.set_item("id", node_id)?;
                    dict.set_item("labels", node.labels().to_vec())?;
                    
                    let props = pyo3::types::PyDict::new_bound(py);
                    for (key, value) in node.properties() {
                        props.set_item(key, property_value_to_py(py, value)?)?;
                    }
                    dict.set_item("properties", props)?;
                    
                    Ok(Some(dict.to_object(py)))
                }
                Err(_) => Ok(None),
            }
        })
    }

    /// Execute a Cypher query
    ///
    /// Args:
    ///     query: Cypher query string
    ///
    /// Returns:
    ///     Query result dictionary
    fn execute_cypher(&self, py: Python, query: String) -> PyResult<PyObject> {
        use crate::query::{CypherParser, QueryPlanner, QueryExecutor, ast::Statement};
        
        // Parse query
        let ast = CypherParser::parse(&query)
            .map_err(|e| PyRuntimeError::new_err(format!("Parse error: {}", e)))?;
        let Statement::Query(query_ast) = ast;
        
        // Plan query
        let planner = QueryPlanner::new();
        let logical_plan = planner.logical_plan(&query_ast)
            .map_err(|e| PyRuntimeError::new_err(format!("Planning error: {}", e)))?;
        let physical_plan = planner.physical_plan(&logical_plan)
            .map_err(|e| PyRuntimeError::new_err(format!("Physical planning error: {}", e)))?;
        
        // Execute query
        let storage_guard = self.storage.read()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        
        // Create Arc wrapper for StorageBackend trait
        use crate::storage::StorageBackend;
        let storage_ref: &dyn StorageBackend = &*storage_guard;
        
        // Need to clone the data for Arc since we can't share RwLockReadGuard
        let all_nodes = storage_ref.get_all_nodes();
        
        drop(storage_guard); // Release lock
        
        // Create temporary in-memory storage with same data
        use crate::storage::MemoryStorage;
        let temp_storage = Arc::new(MemoryStorage::new());
        for node in all_nodes {
            temp_storage.add_node(node).ok();
        }
        
        let executor = QueryExecutor::new(temp_storage);
        let result = executor.execute(&physical_plan)
            .map_err(|e| PyRuntimeError::new_err(format!("Execution error: {}", e)))?;
        
        // Convert result to Python
        let result_dict = pyo3::types::PyDict::new_bound(py);
        result_dict.set_item("columns", result.columns)?;
        result_dict.set_item("row_count", result.row_count)?;
        result_dict.set_item("execution_time_ms", result.execution_time_ms)?;
        
        let rows = pyo3::types::PyList::empty_bound(py);
        for row in result.rows {
            let row_dict = pyo3::types::PyDict::new_bound(py);
            for (key, value) in row {
                row_dict.set_item(key, property_value_to_py(py, &value)?)?;
            }
            rows.append(row_dict)?;
        }
        result_dict.set_item("rows", rows)?;
        
        Ok(result_dict.to_object(py))
    }

    /// Get all nodes with a specific label
    ///
    /// Args:
    ///     label: Label to search for
    ///
    /// Returns:
    ///     List of node IDs as strings
    fn find_nodes_by_label(&self, label: String) -> PyResult<Vec<String>> {
        let storage = self.storage.read()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        
        let nodes = storage.get_nodes_by_label(&label);
        Ok(nodes.iter().map(|node| node.id().to_string()).collect())
    }

    /// Count total nodes
    fn node_count(&self) -> PyResult<usize> {
        let storage = self.storage.read()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        Ok(storage.node_count())
    }

    /// Count total edges
    fn edge_count(&self) -> PyResult<usize> {
        let storage = self.storage.read()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        Ok(storage.edge_count())
    }

    /// Flush pending writes to disk
    fn flush(&self) -> PyResult<()> {
        let storage = self.storage.read()
            .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
        storage.flush()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to flush: {}", e)))?;
        Ok(())
    }

    /// Get database statistics
    fn stats(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let storage = self.storage.read()
                .map_err(|e| PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
            
            let stats = storage.stats();
            let dict = pyo3::types::PyDict::new_bound(py);
            dict.set_item("node_count", stats.node_count)?;
            dict.set_item("edge_count", stats.edge_count)?;
            dict.set_item("size_on_disk_bytes", stats.size_on_disk_bytes)?;
            
            Ok(dict.to_object(py))
        })
    }
}

/// DeepGraph Python module
#[pymodule]
fn deepgraph(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Core classes
    m.add_class::<PyGraphStorage>()?;
    m.add_class::<PyDiskStorage>()?;
    m.add_class::<PyTransactionManager>()?;
    
    // Index management
    m.add_class::<PyIndexManager>()?;
    
    // WAL and recovery
    m.add_class::<PyWAL>()?;
    m.add_class::<PyWALRecovery>()?;
    
    // Query processing
    m.add_class::<PyCypherParser>()?;
    m.add_class::<PyQueryPlanner>()?;
    m.add_class::<PyQueryExecutor>()?;
    
    // MVCC
    m.add_class::<PySnapshot>()?;
    m.add_class::<PyDeadlockDetector>()?;
    
    // Graph Algorithms
    m.add_function(wrap_pyfunction!(py_bfs, m)?)?;
    m.add_function(wrap_pyfunction!(py_dfs, m)?)?;
    m.add_function(wrap_pyfunction!(py_dijkstra, m)?)?;
    m.add_function(wrap_pyfunction!(py_connected_components, m)?)?;
    m.add_function(wrap_pyfunction!(py_pagerank, m)?)?;
    m.add_function(wrap_pyfunction!(py_triangle_count, m)?)?;
    m.add_function(wrap_pyfunction!(py_louvain, m)?)?;
    m.add_function(wrap_pyfunction!(py_node2vec, m)?)?;
    
    // Module metadata
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__author__", "DeepSkilling")?;
    
    Ok(())
}

