//! Multi-Version Concurrency Control (MVCC)
//!
//! Provides snapshot isolation for concurrent transactions

pub mod version;
pub mod snapshot;
pub mod txn_manager;
pub mod deadlock;

pub use version::{Version, VersionChain};
pub use snapshot::Snapshot;
pub use txn_manager::{TransactionManager, TransactionId, TransactionStatus};
pub use deadlock::{DeadlockDetector, ResourceId};

use std::sync::atomic::{AtomicU64, Ordering};

/// Global transaction ID counter
static GLOBAL_TXN_ID: AtomicU64 = AtomicU64::new(1);

/// Generate next transaction ID
pub fn next_txn_id() -> TransactionId {
    TransactionId(GLOBAL_TXN_ID.fetch_add(1, Ordering::SeqCst))
}

/// Timestamp for MVCC
pub type Timestamp = u64;

/// Get current timestamp
pub fn current_timestamp() -> Timestamp {
    GLOBAL_TXN_ID.load(Ordering::SeqCst)
}

