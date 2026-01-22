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
    >>> # Add nodes
    >>> alice_id = storage.add_node(
    ...     labels=["Person"],
    ...     properties={"name": "Alice", "age": 30}
    ... )
    >>> bob_id = storage.add_node(
    ...     labels=["Person"],
    ...     properties={"name": "Bob", "age": 25}
    ... )
    >>> 
    >>> # Execute a Cypher query
    >>> result = storage.execute_cypher("MATCH (n:Person) WHERE n.age > 25 RETURN n;")
    >>> for row in result['rows']:
    ...     print(row['name'], row['age'])
    ... 
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
    # Core storage
    PyGraphStorage,
    PyTransactionManager,
    # Indexing
    PyIndexManager,
    # WAL and recovery
    PyWAL,
    PyWALRecovery,
    # Query processing
    PyCypherParser,
    PyQueryPlanner,
    PyQueryExecutor,
    # MVCC
    PySnapshot,
    PyDeadlockDetector,
    # Graph Algorithms
    py_bfs,
    py_dfs,
    py_dijkstra,
    py_connected_components,
    py_pagerank,
    py_triangle_count,
    py_louvain,
    py_node2vec,
    # Metadata
    __version__,
    __author__,
)

__all__ = [
    # Core
    "PyGraphStorage",
    "PyTransactionManager",
    "GraphStorage",
    "TransactionManager",
    # Indexing
    "PyIndexManager",
    "IndexManager",
    # WAL
    "PyWAL",
    "PyWALRecovery",
    "WAL",
    "WALRecovery",
    # Query
    "PyCypherParser",
    "PyQueryPlanner",
    "PyQueryExecutor",
    "CypherParser",
    "QueryPlanner",
    "QueryExecutor",
    # MVCC
    "PySnapshot",
    "PyDeadlockDetector",
    "Snapshot",
    "DeadlockDetector",
    # Algorithms
    "py_bfs",
    "py_dfs",
    "py_dijkstra",
    "py_connected_components",
    "py_pagerank",
    "py_triangle_count",
    "py_louvain",
    "py_node2vec",
    "bfs",
    "dfs",
    "dijkstra",
    "connected_components",
    "pagerank",
    "triangle_count",
    "louvain",
    "node2vec",
    # Metadata
    "__version__",
    "__author__",
]

# Convenience aliases
GraphStorage = PyGraphStorage
TransactionManager = PyTransactionManager
IndexManager = PyIndexManager
WAL = PyWAL
WALRecovery = PyWALRecovery
CypherParser = PyCypherParser
QueryPlanner = PyQueryPlanner
QueryExecutor = PyQueryExecutor

# Algorithm aliases
bfs = py_bfs
dfs = py_dfs
dijkstra = py_dijkstra
connected_components = py_connected_components
pagerank = py_pagerank
triangle_count = py_triangle_count
louvain = py_louvain
node2vec = py_node2vec
Snapshot = PySnapshot
DeadlockDetector = PyDeadlockDetector

