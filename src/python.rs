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
use crate::storage::GraphStorage;
use crate::mvcc::{TransactionManager, txn_manager::TransactionId};

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

/// DeepGraph Python module
#[pymodule]
fn deepgraph(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGraphStorage>()?;
    m.add_class::<PyTransactionManager>()?;
    
    // Module metadata
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__author__", "DeepSkilling")?;
    
    Ok(())
}

