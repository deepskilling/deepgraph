//! Persistence layer for durable storage
//!
//! Provides save/load functionality using Parquet format for efficient
//! storage and fast loading of graph data.

pub mod parquet_io;
pub mod snapshot;

pub use parquet_io::{ParquetWriter, ParquetReader};
pub use snapshot::{Snapshot, SnapshotManager};

use crate::error::Result;
use std::path::Path;

/// Trait for persistable storage backends
pub trait Persistable {
    /// Save the storage to a directory
    fn save(&self, path: &Path) -> Result<()>;
    
    /// Load the storage from a directory
    fn load(&mut self, path: &Path) -> Result<()>;
    
    /// Create a snapshot
    fn snapshot(&self, path: &Path) -> Result<Snapshot>;
    
    /// Restore from a snapshot
    fn restore(&mut self, snapshot: &Snapshot) -> Result<()>;
}

