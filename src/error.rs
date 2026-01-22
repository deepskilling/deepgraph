//! Error types for DeepGraph

use thiserror::Error;

/// The main error type for DeepGraph operations
#[derive(Error, Debug)]
pub enum DeepGraphError {
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Edge not found: {0}")]
    EdgeNotFound(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Property not found: {0}")]
    PropertyNotFound(String),

    #[error("Invalid node ID: {0}")]
    InvalidNodeId(String),

    #[error("Invalid edge ID: {0}")]
    InvalidEdgeId(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("Parser error: {0}")]
    ParserError(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Invalid property type: expected {expected}, got {actual}")]
    InvalidPropertyType { expected: String, actual: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type alias for DeepGraph operations
pub type Result<T> = std::result::Result<T, DeepGraphError>;

