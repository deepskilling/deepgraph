//! Community detection algorithms

use crate::error::Result;
use crate::graph::NodeId;
use crate::storage::GraphStorage;
use std::collections::{HashMap, HashSet};

/// Result of Louvain community detection
#[derive(Debug, Clone)]
pub struct LouvainResult {
    /// Community assignment for each node
    pub communities: HashMap<NodeId, usize>,
    /// Modularity score
    pub modularity: f64,
    /// Number of communities found
    pub num_communities: usize,
    /// Number of iterations performed
    pub iterations: usize,
}

impl LouvainResult {
    /// Get all nodes in a specific community
    pub fn nodes_in_community(&self, community_id: usize) -> Vec<NodeId> {
        self.communities
            .iter()
            .filter(|(_, &comm)| comm == community_id)
            .map(|(&node, _)| node)
            .collect()
    }

    /// Get community sizes
    pub fn community_sizes(&self) -> HashMap<usize, usize> {
        let mut sizes = HashMap::new();
        for &community in self.communities.values() {
            *sizes.entry(community).or_insert(0) += 1;
        }
        sizes
    }
}

/// Louvain community detection algorithm
///
/// Detects communities (densely connected groups) in the graph
/// by optimizing the modularity measure.
///
/// # Arguments
/// * `storage` - Graph storage
/// * `max_iterations` - Maximum number of iterations (typically 10-100)
/// * `min_improvement` - Minimum modularity improvement to continue (typically 1e-4)
///
/// # Returns
/// LouvainResult with community assignments and modularity score
///
/// # Example
/// ```rust,ignore
/// use deepgraph::algorithms::louvain;
/// use deepgraph::storage::GraphStorage;
/// 
/// let storage = GraphStorage::new();
/// // ... add nodes and edges ...
/// let result = louvain(&storage, 100, 1e-4)?;
/// println!("Found {} communities with modularity {}", 
///          result.num_communities, result.modularity);
/// ```
pub fn louvain(
    storage: &GraphStorage,
    max_iterations: usize,
    _min_improvement: f64,
) -> Result<LouvainResult> {
    let all_nodes = storage.get_all_nodes();
    let num_nodes = all_nodes.len();

    if num_nodes == 0 {
        return Ok(LouvainResult {
            communities: HashMap::new(),
            modularity: 0.0,
            num_communities: 0,
            iterations: 0,
        });
    }

    // Initialize: each node in its own community
    let mut communities: HashMap<NodeId, usize> = all_nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id(), i))
        .collect();

    // Build adjacency and degree information
    let mut adjacency: HashMap<NodeId, HashMap<NodeId, f64>> = HashMap::new();
    let mut total_weight = 0.0;
    let mut node_degrees: HashMap<NodeId, f64> = HashMap::new();

    for node in &all_nodes {
        let node_id = node.id();
        let mut neighbors = HashMap::new();
        let mut degree = 0.0;

        // Process outgoing edges
        if let Ok(edges) = storage.get_outgoing_edges(node_id) {
            for edge in edges {
                let weight = 1.0; // Could extract from edge properties
                neighbors.insert(edge.to(), weight);
                degree += weight;
                total_weight += weight;
            }
        }

        adjacency.insert(node_id, neighbors);
        node_degrees.insert(node_id, degree);
    }

    let m = total_weight;
    let mut current_modularity = calculate_modularity(&communities, &adjacency, &node_degrees, m);
    let mut iterations = 0;

    // Phase 1: Optimize communities
    for iteration in 0..max_iterations {
        let mut improved = false;
        iterations = iteration + 1;

        for node in &all_nodes {
            let node_id = node.id();
            let current_comm = communities[&node_id];

            // Try moving node to neighboring communities
            let mut neighbor_communities = HashSet::new();
            if let Some(neighbors) = adjacency.get(&node_id) {
                for &neighbor_id in neighbors.keys() {
                    neighbor_communities.insert(communities[&neighbor_id]);
                }
            }

            let mut best_comm = current_comm;
            let mut best_modularity = current_modularity;

            for &target_comm in &neighbor_communities {
                if target_comm == current_comm {
                    continue;
                }

                // Try moving node to target community
                communities.insert(node_id, target_comm);
                let new_modularity =
                    calculate_modularity(&communities, &adjacency, &node_degrees, m);

                if new_modularity > best_modularity {
                    best_modularity = new_modularity;
                    best_comm = target_comm;
                    improved = true;
                }
            }

            // Apply best move
            communities.insert(node_id, best_comm);
        }

        current_modularity = calculate_modularity(&communities, &adjacency, &node_degrees, m);

        // Check for convergence
        if !improved {
            break;
        }
    }

    // Renumber communities to be consecutive
    let unique_comms: HashSet<_> = communities.values().copied().collect();
    let mut comm_map: HashMap<usize, usize> = HashMap::new();
    for (new_id, &old_id) in unique_comms.iter().enumerate() {
        comm_map.insert(old_id, new_id);
    }

    for (_node, comm) in communities.iter_mut() {
        *comm = comm_map[comm];
    }

    let num_communities = unique_comms.len();

    Ok(LouvainResult {
        communities,
        modularity: current_modularity,
        num_communities,
        iterations,
    })
}

/// Calculate modularity of a community assignment
fn calculate_modularity(
    communities: &HashMap<NodeId, usize>,
    adjacency: &HashMap<NodeId, HashMap<NodeId, f64>>,
    node_degrees: &HashMap<NodeId, f64>,
    total_weight: f64,
) -> f64 {
    let mut modularity = 0.0;

    for (node_i, neighbors) in adjacency.iter() {
        let comm_i = communities[node_i];
        let degree_i = node_degrees[node_i];

        for (node_j, &weight) in neighbors.iter() {
            let comm_j = communities[node_j];

            if comm_i == comm_j {
                let degree_j = node_degrees[node_j];
                let expected = (degree_i * degree_j) / (2.0 * total_weight);
                modularity += weight - expected;
            }
        }
    }

    modularity / (2.0 * total_weight)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Node;

    #[test]
    fn test_louvain_simple() {
        let storage = GraphStorage::new();

        // Create two densely connected groups
        // Group 1: nodes 1-2-3 (fully connected)
        // Group 2: nodes 4-5-6 (fully connected)
        // Sparse connection: 3-4

        let mut ids = Vec::new();
        for _ in 0..6 {
            let node = Node::new(vec!["Node".to_string()]);
            ids.push(storage.add_node(node).unwrap());
        }

        // Group 1 edges
        storage
            .add_edge_simple(ids[0], ids[1], "CONNECTS".to_string())
            .unwrap();
        storage
            .add_edge_simple(ids[1], ids[2], "CONNECTS".to_string())
            .unwrap();
        storage
            .add_edge_simple(ids[2], ids[0], "CONNECTS".to_string())
            .unwrap();

        // Group 2 edges
        storage
            .add_edge_simple(ids[3], ids[4], "CONNECTS".to_string())
            .unwrap();
        storage
            .add_edge_simple(ids[4], ids[5], "CONNECTS".to_string())
            .unwrap();
        storage
            .add_edge_simple(ids[5], ids[3], "CONNECTS".to_string())
            .unwrap();

        // Bridge
        storage
            .add_edge_simple(ids[2], ids[3], "CONNECTS".to_string())
            .unwrap();

        let result = louvain(&storage, 100, 1e-4).unwrap();

        // Should find 2 communities
        assert!(result.num_communities >= 2);
        assert!(result.modularity > 0.0);
    }
}

