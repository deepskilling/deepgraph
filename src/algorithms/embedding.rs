//! Graph embedding algorithms (Node2Vec, etc.)

use crate::error::Result;
use crate::graph::NodeId;
use crate::storage::GraphStorage;
use rand::prelude::*;
use std::collections::HashMap;

/// Configuration for Node2Vec algorithm
#[derive(Debug, Clone)]
pub struct Node2VecConfig {
    /// Walk length (number of nodes in each walk)
    pub walk_length: usize,
    /// Number of walks per node
    pub walks_per_node: usize,
    /// Return parameter (p) - controls likelihood of revisiting previous node
    pub return_param: f64,
    /// In-out parameter (q) - controls BFS vs DFS behavior
    pub inout_param: f64,
    /// Random seed for reproducibility
    pub seed: Option<u64>,
}

impl Default for Node2VecConfig {
    fn default() -> Self {
        Self {
            walk_length: 80,
            walks_per_node: 10,
            return_param: 1.0,
            inout_param: 1.0,
            seed: None,
        }
    }
}

/// Result of Node2Vec sampling
#[derive(Debug, Clone)]
pub struct Node2VecResult {
    /// Generated random walks (sequences of node IDs)
    pub walks: Vec<Vec<NodeId>>,
    /// Configuration used
    pub config: Node2VecConfig,
}

impl Node2VecResult {
    /// Get total number of walks generated
    pub fn num_walks(&self) -> usize {
        self.walks.len()
    }

    /// Get total number of steps across all walks
    pub fn total_steps(&self) -> usize {
        self.walks.iter().map(|w| w.len()).sum()
    }

    /// Get walks that start from a specific node
    pub fn walks_from_node(&self, node_id: NodeId) -> Vec<&Vec<NodeId>> {
        self.walks
            .iter()
            .filter(|w| w.first() == Some(&node_id))
            .collect()
    }
}

/// Node2Vec biased random walk sampler
///
/// Generates random walks with configurable exploration-exploitation tradeoff.
/// These walks can be used as input to Word2Vec-style embedding algorithms.
///
/// # Arguments
/// * `storage` - Graph storage
/// * `config` - Node2Vec configuration
///
/// # Returns
/// Node2VecResult containing generated random walks
///
/// # Example
/// ```rust,ignore
/// use deepgraph::algorithms::{node2vec, Node2VecConfig};
/// use deepgraph::storage::GraphStorage;
/// 
/// let storage = GraphStorage::new();
/// // ... add nodes and edges ...
/// 
/// let config = Node2VecConfig {
///     walk_length: 80,
///     walks_per_node: 10,
///     return_param: 1.0,  // p
///     inout_param: 1.0,   // q
///     seed: Some(42),
/// };
/// 
/// let result = node2vec(&storage, config)?;
/// println!("Generated {} walks", result.num_walks());
/// ```
pub fn node2vec(storage: &GraphStorage, config: Node2VecConfig) -> Result<Node2VecResult> {
    let all_nodes = storage.get_all_nodes();

    if all_nodes.is_empty() {
        return Ok(Node2VecResult {
            walks: Vec::new(),
            config,
        });
    }

    // Initialize RNG
    let mut rng: StdRng = if let Some(seed) = config.seed {
        StdRng::seed_from_u64(seed)
    } else {
        StdRng::from_entropy()
    };

    let mut walks = Vec::new();

    // Precompute neighbors for all nodes
    let mut neighbors_map: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for node in &all_nodes {
        let node_id = node.id();
        let mut neighbors = Vec::new();

        if let Ok(edges) = storage.get_outgoing_edges(node_id) {
            for edge in edges {
                neighbors.push(edge.to());
            }
        }

        neighbors_map.insert(node_id, neighbors);
    }

    // Generate walks
    for node in &all_nodes {
        let start_node = node.id();

        for _ in 0..config.walks_per_node {
            let walk = generate_biased_walk(
                start_node,
                &neighbors_map,
                config.walk_length,
                config.return_param,
                config.inout_param,
                &mut rng,
            );

            if !walk.is_empty() {
                walks.push(walk);
            }
        }
    }

    Ok(Node2VecResult { walks, config })
}

/// Generate a single biased random walk
fn generate_biased_walk(
    start_node: NodeId,
    neighbors_map: &HashMap<NodeId, Vec<NodeId>>,
    walk_length: usize,
    p: f64,
    q: f64,
    rng: &mut StdRng,
) -> Vec<NodeId> {
    let mut walk = vec![start_node];

    if walk_length < 2 {
        return walk;
    }

    // Get first step (uniform random)
    if let Some(neighbors) = neighbors_map.get(&start_node) {
        if neighbors.is_empty() {
            return walk;
        }

        let next = neighbors[rng.gen_range(0..neighbors.len())];
        walk.push(next);
    } else {
        return walk;
    }

    // Subsequent steps use biased sampling
    for _ in 2..walk_length {
        let current = *walk.last().unwrap();
        let previous = walk[walk.len() - 2];

        if let Some(neighbors) = neighbors_map.get(&current) {
            if neighbors.is_empty() {
                break;
            }

            // Calculate transition probabilities
            let mut weights = Vec::new();
            let mut total_weight = 0.0;

            for &neighbor in neighbors {
                let weight = if neighbor == previous {
                    // Return to previous node
                    1.0 / p
                } else if is_neighbor(previous, neighbor, neighbors_map) {
                    // Neighbor of previous (BFS-like)
                    1.0
                } else {
                    // Not connected to previous (DFS-like)
                    1.0 / q
                };

                weights.push(weight);
                total_weight += weight;
            }

            // Sample next node based on weights
            let rand_val: f64 = rng.gen::<f64>() * total_weight;
            let mut cumulative = 0.0;
            let mut selected = neighbors[0];

            for (i, &weight) in weights.iter().enumerate() {
                cumulative += weight;
                if rand_val <= cumulative {
                    selected = neighbors[i];
                    break;
                }
            }

            walk.push(selected);
        } else {
            break;
        }
    }

    walk
}

/// Check if two nodes are neighbors
fn is_neighbor(node1: NodeId, node2: NodeId, neighbors_map: &HashMap<NodeId, Vec<NodeId>>) -> bool {
    if let Some(neighbors) = neighbors_map.get(&node1) {
        neighbors.contains(&node2)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Node;

    #[test]
    fn test_node2vec_simple() {
        let storage = GraphStorage::new();

        // Create a simple chain: 1 -> 2 -> 3 -> 4
        let mut ids = Vec::new();
        for _ in 0..4 {
            let node = Node::new(vec!["Node".to_string()]);
            ids.push(storage.add_node(node).unwrap());
        }

        storage
            .add_edge_simple(ids[0], ids[1], "CONNECTS".to_string())
            .unwrap();
        storage
            .add_edge_simple(ids[1], ids[2], "CONNECTS".to_string())
            .unwrap();
        storage
            .add_edge_simple(ids[2], ids[3], "CONNECTS".to_string())
            .unwrap();

        let config = Node2VecConfig {
            walk_length: 10,
            walks_per_node: 5,
            return_param: 1.0,
            inout_param: 1.0,
            seed: Some(42),
        };

        let result = node2vec(&storage, config).unwrap();

        // Should generate walks_per_node * num_nodes walks
        assert_eq!(result.num_walks(), 5 * 4);

        // Each walk should start with length <= walk_length
        for walk in &result.walks {
            assert!(walk.len() <= 10);
            assert!(!walk.is_empty());
        }
    }
}

