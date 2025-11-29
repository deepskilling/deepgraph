//! Graph connectivity algorithms

use crate::error::Result;
use crate::graph::NodeId;
use crate::storage::GraphStorage;
use std::collections::{HashMap, HashSet, VecDeque};

/// Result of connected components analysis
#[derive(Debug, Clone)]
pub struct ConnectedComponentsResult {
    /// Component ID for each node
    pub component_map: HashMap<NodeId, usize>,
    /// Number of components
    pub num_components: usize,
    /// Size of each component
    pub component_sizes: HashMap<usize, usize>,
}

impl ConnectedComponentsResult {
    /// Get all nodes in a specific component
    pub fn nodes_in_component(&self, component_id: usize) -> Vec<NodeId> {
        self.component_map
            .iter()
            .filter(|(_, &comp)| comp == component_id)
            .map(|(&node, _)| node)
            .collect()
    }

    /// Check if two nodes are in the same component
    pub fn are_connected(&self, node1: NodeId, node2: NodeId) -> bool {
        match (
            self.component_map.get(&node1),
            self.component_map.get(&node2),
        ) {
            (Some(&c1), Some(&c2)) => c1 == c2,
            _ => false,
        }
    }
}

/// Find all connected components in an undirected graph
///
/// Uses BFS to identify weakly connected components.
/// For directed graphs, treats edges as undirected.
///
/// # Arguments
/// * `storage` - Graph storage
///
/// # Returns
/// ConnectedComponentsResult with component assignments
///
/// # Example
/// ```rust,ignore
/// use deepgraph::algorithms::connected_components;
/// use deepgraph::storage::GraphStorage;
/// 
/// let storage = GraphStorage::new();
/// // ... add nodes and edges ...
/// let result = connected_components(&storage)?;
/// println!("Found {} components", result.num_components);
/// ```
pub fn connected_components(storage: &GraphStorage) -> Result<ConnectedComponentsResult> {
    let mut component_map = HashMap::new();
    let mut component_sizes = HashMap::new();
    let mut visited = HashSet::new();
    let mut component_id = 0;

    // Get all nodes
    let all_nodes = storage.get_all_nodes();

    for node_data in all_nodes {
        let node_id = node_data.id();

        if visited.contains(&node_id) {
            continue;
        }

        // BFS to find all nodes in this component
        let mut queue = VecDeque::new();
        queue.push_back(node_id);
        visited.insert(node_id);

        let mut component_size = 0;

        while let Some(current) = queue.pop_front() {
            component_map.insert(current, component_id);
            component_size += 1;

            // Check outgoing edges
            if let Ok(edges) = storage.get_outgoing_edges(current) {
                for edge in edges {
                    let neighbor = edge.to();
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                    }
                }
            }

            // Check incoming edges (treat as undirected)
            if let Ok(edges) = storage.get_incoming_edges(current) {
                for edge in edges {
                    let neighbor = edge.from();
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        component_sizes.insert(component_id, component_size);
        component_id += 1;
    }

    Ok(ConnectedComponentsResult {
        component_map,
        num_components: component_id,
        component_sizes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Node;

    #[test]
    fn test_connected_components() {
        let storage = GraphStorage::new();

        // Create two separate components: (1-2) and (3-4)
        let n1 = Node::new(vec!["Node".to_string()]);
        let n2 = Node::new(vec!["Node".to_string()]);
        let n3 = Node::new(vec!["Node".to_string()]);
        let n4 = Node::new(vec!["Node".to_string()]);

        let id1 = storage.add_node(n1).unwrap();
        let id2 = storage.add_node(n2).unwrap();
        let id3 = storage.add_node(n3).unwrap();
        let id4 = storage.add_node(n4).unwrap();

        storage
            .add_edge_simple(id1, id2, "CONNECTS".to_string())
            .unwrap();
        storage
            .add_edge_simple(id3, id4, "CONNECTS".to_string())
            .unwrap();

        let result = connected_components(&storage).unwrap();

        assert_eq!(result.num_components, 2);
        assert!(!result.are_connected(id1, id3));
        assert!(result.are_connected(id1, id2));
    }
}

