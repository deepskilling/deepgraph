//! Graph traversal algorithms (BFS, DFS)

use crate::error::Result;
use crate::graph::NodeId;
use crate::storage::GraphStorage;
use std::collections::{HashMap, HashSet, VecDeque};

/// Result of BFS traversal
#[derive(Debug, Clone)]
pub struct BFSResult {
    /// Visited nodes in order
    pub visited: Vec<NodeId>,
    /// Distance from start node to each visited node
    pub distances: HashMap<NodeId, usize>,
    /// Parent of each node in the BFS tree
    pub parents: HashMap<NodeId, Option<NodeId>>,
}

/// Result of DFS traversal
#[derive(Debug, Clone)]
pub struct DFSResult {
    /// Visited nodes in order
    pub visited: Vec<NodeId>,
    /// Discovery time for each node
    pub discovery_time: HashMap<NodeId, usize>,
    /// Finish time for each node
    pub finish_time: HashMap<NodeId, usize>,
    /// Parent of each node in the DFS tree
    pub parents: HashMap<NodeId, Option<NodeId>>,
}

/// Breadth-First Search (BFS) traversal
///
/// # Arguments
/// * `storage` - Graph storage to traverse
/// * `start_node` - Starting node ID
/// * `max_depth` - Optional maximum depth to traverse (None for unlimited)
///
/// # Returns
/// BFSResult containing visited nodes, distances, and parent relationships
///
/// # Example
/// ```rust,ignore
/// use deepgraph::algorithms::bfs;
/// use deepgraph::storage::GraphStorage;
/// 
/// let storage = GraphStorage::new();
/// // ... add nodes and edges ...
/// let result = bfs(&storage, start_id, None)?;
/// println!("Visited {} nodes", result.visited.len());
/// ```
pub fn bfs(
    storage: &GraphStorage,
    start_node: NodeId,
    max_depth: Option<usize>,
) -> Result<BFSResult> {
    // Verify start node exists
    storage.get_node(start_node)?;

    let mut visited = Vec::new();
    let mut distances = HashMap::new();
    let mut parents = HashMap::new();
    let mut queue = VecDeque::new();
    let mut visited_set = HashSet::new();

    // Initialize with start node
    queue.push_back(start_node);
    visited_set.insert(start_node);
    distances.insert(start_node, 0);
    parents.insert(start_node, None);

    while let Some(current) = queue.pop_front() {
        visited.push(current);
        let current_dist = distances[&current];

        // Check max depth
        if let Some(max_d) = max_depth {
            if current_dist >= max_d {
                continue;
            }
        }

        // Get neighbors (outgoing edges)
        if let Ok(edges) = storage.get_outgoing_edges(current) {
            for edge in edges {
                let neighbor = edge.to();
                if !visited_set.contains(&neighbor) {
                    visited_set.insert(neighbor);
                    queue.push_back(neighbor);
                    distances.insert(neighbor, current_dist + 1);
                    parents.insert(neighbor, Some(current));
                }
            }
        }
    }

    Ok(BFSResult {
        visited,
        distances,
        parents,
    })
}

/// Depth-First Search (DFS) traversal
///
/// # Arguments
/// * `storage` - Graph storage to traverse
/// * `start_node` - Starting node ID
///
/// # Returns
/// DFSResult containing visited nodes, discovery/finish times, and parent relationships
///
/// # Example
/// ```rust,ignore
/// use deepgraph::algorithms::dfs;
/// use deepgraph::storage::GraphStorage;
/// 
/// let storage = GraphStorage::new();
/// // ... add nodes and edges ...
/// let result = dfs(&storage, start_id)?;
/// println!("Visited {} nodes", result.visited.len());
/// ```
pub fn dfs(storage: &GraphStorage, start_node: NodeId) -> Result<DFSResult> {
    // Verify start node exists
    storage.get_node(start_node)?;

    let mut visited = Vec::new();
    let mut discovery_time = HashMap::new();
    let mut finish_time = HashMap::new();
    let mut parents = HashMap::new();
    let mut visited_set = HashSet::new();
    let mut time = 0;

    // DFS recursive helper
    fn dfs_visit(
        storage: &GraphStorage,
        node: NodeId,
        visited: &mut Vec<NodeId>,
        discovery_time: &mut HashMap<NodeId, usize>,
        finish_time: &mut HashMap<NodeId, usize>,
        parents: &mut HashMap<NodeId, Option<NodeId>>,
        visited_set: &mut HashSet<NodeId>,
        time: &mut usize,
    ) -> Result<()> {
        visited_set.insert(node);
        *time += 1;
        discovery_time.insert(node, *time);
        visited.push(node);

        // Visit neighbors
        if let Ok(edges) = storage.get_outgoing_edges(node) {
            for edge in edges {
                let neighbor = edge.to();
                if !visited_set.contains(&neighbor) {
                    parents.insert(neighbor, Some(node));
                    dfs_visit(
                        storage,
                        neighbor,
                        visited,
                        discovery_time,
                        finish_time,
                        parents,
                        visited_set,
                        time,
                    )?;
                }
            }
        }

        *time += 1;
        finish_time.insert(node, *time);
        Ok(())
    }

    parents.insert(start_node, None);
    dfs_visit(
        storage,
        start_node,
        &mut visited,
        &mut discovery_time,
        &mut finish_time,
        &mut parents,
        &mut visited_set,
        &mut time,
    )?;

    Ok(DFSResult {
        visited,
        discovery_time,
        finish_time,
        parents,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Node;

    #[test]
    fn test_bfs_simple() {
        let storage = GraphStorage::new();

        // Create a simple graph: 1 -> 2 -> 3
        let n1 = Node::new(vec!["Node".to_string()]);
        let n2 = Node::new(vec!["Node".to_string()]);
        let n3 = Node::new(vec!["Node".to_string()]);

        let id1 = storage.add_node(n1).unwrap();
        let id2 = storage.add_node(n2).unwrap();
        let id3 = storage.add_node(n3).unwrap();

        storage
            .add_edge_simple(id1, id2, "CONNECTS".to_string())
            .unwrap();
        storage
            .add_edge_simple(id2, id3, "CONNECTS".to_string())
            .unwrap();

        let result = bfs(&storage, id1, None).unwrap();
        assert_eq!(result.visited.len(), 3);
        assert_eq!(*result.distances.get(&id1).unwrap(), 0);
        assert_eq!(*result.distances.get(&id2).unwrap(), 1);
        assert_eq!(*result.distances.get(&id3).unwrap(), 2);
    }

    #[test]
    fn test_dfs_simple() {
        let storage = GraphStorage::new();

        // Create a simple graph: 1 -> 2 -> 3
        let n1 = Node::new(vec!["Node".to_string()]);
        let n2 = Node::new(vec!["Node".to_string()]);
        let n3 = Node::new(vec!["Node".to_string()]);

        let id1 = storage.add_node(n1).unwrap();
        let id2 = storage.add_node(n2).unwrap();
        let id3 = storage.add_node(n3).unwrap();

        storage
            .add_edge_simple(id1, id2, "CONNECTS".to_string())
            .unwrap();
        storage
            .add_edge_simple(id2, id3, "CONNECTS".to_string())
            .unwrap();

        let result = dfs(&storage, id1).unwrap();
        assert_eq!(result.visited.len(), 3);
        assert!(result.discovery_time.contains_key(&id1));
        assert!(result.finish_time.contains_key(&id1));
    }
}

