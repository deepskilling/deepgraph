"""
DeepGraph - High-Performance Graph Database

A production-ready graph database with full ACID guarantees, 
advanced indexing, and intelligent query optimization.

Example:
    >>> import deepgraph
    >>> 
    >>> # Create a graph storage
    >>> storage = deepgraph.GraphStorage()
    >>> 
    >>> # Add a node
    >>> node_id = storage.add_node(
    ...     labels=["Person"],
    ...     properties={"name": "Alice", "age": 30}
    ... )
    >>> 
    >>> # Get the node
    >>> node = storage.get_node(node_id)
    >>> print(node)
    >>> 
    >>> # Use indices for fast lookups
    >>> idx_mgr = deepgraph.IndexManager()
    >>> idx_mgr.create_hash_index("person_idx", "Person")
    >>> 
    >>> # Use WAL for durability
    >>> wal = deepgraph.WAL("./data/wal")
    >>> wal.flush()

For more information, visit: https://github.com/deepskilling/deepgraph
"""

from .deepgraph import (
    PyGraphStorage,
    PyTransactionManager,
    PyIndexManager,
    PyWAL,
    __version__,
    __author__,
)

__all__ = [
    "PyGraphStorage",
    "PyTransactionManager",
    "PyIndexManager",
    "PyWAL",
    "__version__",
    "__author__",
    # Convenience aliases
    "GraphStorage",
    "TransactionManager",
    "IndexManager",
    "WAL",
]

# Convenience aliases
GraphStorage = PyGraphStorage
TransactionManager = PyTransactionManager
IndexManager = PyIndexManager
WAL = PyWAL

