//! Structural graph algorithms (Triangle Counting, etc.)

use crate::error::Result;
use crate::graph::NodeId;
use crate::storage::GraphStorage;
use std::collections::{HashMap, HashSet};

/// Result of triangle counting
#[derive(Debug, Clone)]
pub struct TriangleCountResult {
    /// Total number of triangles in the graph
    pub total_triangles: usize,
    /// Number of triangles each node participates in
    pub node_triangles: HashMap<NodeId, usize>,
    /// Clustering coefficient for each node
    pub clustering_coefficients: HashMap<NodeId, f64>,
    /// Global clustering coefficient
    pub global_clustering_coefficient: f64,
}

/// Count triangles in the graph
///
/// A triangle is a set of three nodes that are all connected to each other.
/// This implementation treats the graph as undirected.
///
/// # Arguments
/// * `storage` - Graph storage
///
/// # Returns
/// TriangleCountResult with triangle counts and clustering coefficients
///
/// # Example
/// ```rust,ignore
/// use deepgraph::algorithms::triangle_count;
/// use deepgraph::storage::GraphStorage;
/// 
/// let storage = GraphStorage::new();
/// // ... add nodes and edges ...
/// let result = triangle_count(&storage)?;
/// println!("Found {} triangles", result.total_triangles);
/// ```
pub fn triangle_count(storage: &GraphStorage) -> Result<TriangleCountResult> {
    let all_nodes = storage.get_all_nodes();
    let mut triangles_per_node: HashMap<NodeId, usize> = HashMap::new();
    let mut total_triangles = 0;

    // Build adjacency sets for each node (treat as undirected)
    let mut adjacency: HashMap<NodeId, HashSet<NodeId>> = HashMap::new();

    for node in &all_nodes {
        let node_id = node.id();
        let mut neighbors = HashSet::new();

        // Add outgoing neighbors
        if let Ok(edges) = storage.get_outgoing_edges(node_id) {
            for edge in edges {
                neighbors.insert(edge.to());
            }
        }

        // Add incoming neighbors (treat as undirected)
        if let Ok(edges) = storage.get_incoming_edges(node_id) {
            for edge in edges {
                neighbors.insert(edge.from());
            }
        }

        adjacency.insert(node_id, neighbors);
        triangles_per_node.insert(node_id, 0);
    }

    // Count triangles
    let node_ids: Vec<NodeId> = all_nodes.iter().map(|n| n.id()).collect();

    for (i, &node1) in node_ids.iter().enumerate() {
        let neighbors1 = adjacency.get(&node1).unwrap();

        for (_j, &node2) in node_ids.iter().enumerate().skip(i + 1) {
            // Check if node1 and node2 are connected
            if !neighbors1.contains(&node2) {
                continue;
            }

            let neighbors2 = adjacency.get(&node2).unwrap();

            // Find common neighbors (potential third vertex of triangle)
            for &node3 in neighbors1.iter() {
                if node3 != node2 && neighbors2.contains(&node3) {
                    // Found a triangle: node1-node2-node3
                    *triangles_per_node.get_mut(&node1).unwrap() += 1;
                    *triangles_per_node.get_mut(&node2).unwrap() += 1;
                    *triangles_per_node.get_mut(&node3).unwrap() += 1;

                    // Only count once (avoid triple counting)
                    if node1 < node2 && node2 < node3 {
                        total_triangles += 1;
                    }
                }
            }
        }
    }

    // Calculate clustering coefficients
    let mut clustering_coefficients = HashMap::new();
    let mut total_coefficient_sum = 0.0;
    let mut nodes_with_coefficient = 0;

    for node in &all_nodes {
        let node_id = node.id();
        let degree = adjacency.get(&node_id).unwrap().len();

        if degree < 2 {
            clustering_coefficients.insert(node_id, 0.0);
            continue;
        }

        let triangles = *triangles_per_node.get(&node_id).unwrap() as f64;
        let max_possible = (degree * (degree - 1)) as f64 / 2.0;
        let coefficient = triangles / max_possible;

        clustering_coefficients.insert(node_id, coefficient);
        total_coefficient_sum += coefficient;
        nodes_with_coefficient += 1;
    }

    let global_clustering_coefficient = if nodes_with_coefficient > 0 {
        total_coefficient_sum / nodes_with_coefficient as f64
    } else {
        0.0
    };

    Ok(TriangleCountResult {
        total_triangles,
        node_triangles: triangles_per_node,
        clustering_coefficients,
        global_clustering_coefficient,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Node;

    #[test]
    fn test_triangle_count() {
        let storage = GraphStorage::new();

        // Create a triangle: 1-2, 2-3, 3-1
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
        storage
            .add_edge_simple(id3, id1, "CONNECTS".to_string())
            .unwrap();

        let result = triangle_count(&storage).unwrap();

        assert_eq!(result.total_triangles, 1);
        assert_eq!(*result.node_triangles.get(&id1).unwrap(), 1);
        assert_eq!(*result.node_triangles.get(&id2).unwrap(), 1);
        assert_eq!(*result.node_triangles.get(&id3).unwrap(), 1);
    }
}

