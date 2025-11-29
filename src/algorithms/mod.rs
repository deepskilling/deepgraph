//! Graph algorithms module
//!
//! This module provides implementations of common graph algorithms optimized for DeepGraph:
//! - **Traversal**: BFS, DFS
//! - **Shortest Path**: Dijkstra
//! - **Connectivity**: Connected Components
//! - **Centrality**: PageRank
//! - **Structural**: Triangle Counting
//! - **Community**: Louvain Community Detection
//! - **Embedding**: Node2Vec (Biased Random Walk)

pub mod traversal;
pub mod shortest_path;
pub mod connectivity;
pub mod centrality;
pub mod structural;
pub mod community;
pub mod embedding;

pub use traversal::{bfs, dfs, BFSResult, DFSResult};
pub use shortest_path::{dijkstra, DijkstraResult};
pub use connectivity::{connected_components, ConnectedComponentsResult};
pub use centrality::{pagerank, PageRankResult};
pub use structural::{triangle_count, TriangleCountResult};
pub use community::{louvain, LouvainResult};
pub use embedding::{node2vec, Node2VecConfig, Node2VecResult};

