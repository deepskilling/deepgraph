//! DeepGraph - A high-performance graph database written in Rust
//!
//! # Architecture
//!
//! DeepGraph is organized into several modules:
//! - `graph`: Core graph data structures (Node, Edge, Property)
//! - `storage`: Storage backends (in-memory, columnar, persistent)
//! - `parser`: Cypher query parser
//! - `transaction`: Transaction management framework
//! - `index`: Indexing system for efficient queries
//! - `query`: Query planning and execution
//! - `wal`: Write-ahead logging for durability
//! - `mvcc`: Multi-version concurrency control

pub mod graph;
pub mod storage;
pub mod parser;
pub mod transaction;
pub mod error;

// Phase 2 modules
pub mod persistence;
pub mod index;
pub mod query;
pub mod wal;
pub mod mvcc;

// Phase 3 modules
pub mod algorithms;

// Python bindings (optional)
#[cfg(feature = "python")]
pub mod python;

pub use error::{DeepGraphError, Result};
pub use graph::{Node, Edge, Property, PropertyValue, NodeId, EdgeId};
pub use storage::{GraphStorage, StorageBackend};
pub use transaction::Transaction;

