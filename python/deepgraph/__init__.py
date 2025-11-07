"""
DeepGraph - High-Performance Graph Database

A production-ready graph database with full ACID guarantees, 
advanced indexing, and intelligent query optimization.

Example:
    >>> import deepgraph
    >>> 
    >>> # Create a graph storage
    >>> storage = deepgraph.PyGraphStorage()
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

For more information, visit: https://github.com/deepskilling/deepgraph
"""

from .deepgraph import (
    PyGraphStorage,
    PyTransactionManager,
    __version__,
    __author__,
)

__all__ = [
    "PyGraphStorage",
    "PyTransactionManager",
    "__version__",
    "__author__",
]

# Convenience aliases
GraphStorage = PyGraphStorage
TransactionManager = PyTransactionManager

