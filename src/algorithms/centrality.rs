//! Centrality algorithms (PageRank, etc.)

use crate::error::Result;
use crate::graph::NodeId;
use crate::storage::GraphStorage;
use std::collections::HashMap;

/// Result of PageRank algorithm
#[derive(Debug, Clone)]
pub struct PageRankResult {
    /// PageRank score for each node
    pub scores: HashMap<NodeId, f64>,
    /// Number of iterations performed
    pub iterations: usize,
    /// Whether the algorithm converged
    pub converged: bool,
}

impl PageRankResult {
    /// Get top N nodes by PageRank score
    pub fn top_nodes(&self, n: usize) -> Vec<(NodeId, f64)> {
        let mut scores: Vec<_> = self.scores.iter().map(|(&k, &v)| (k, v)).collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores.into_iter().take(n).collect()
    }
}

/// PageRank algorithm
///
/// Computes the PageRank score for each node in the graph.
/// PageRank is a measure of node importance based on the graph structure.
///
/// # Arguments
/// * `storage` - Graph storage
/// * `damping_factor` - Damping factor (typically 0.85)
/// * `max_iterations` - Maximum number of iterations
/// * `tolerance` - Convergence tolerance
///
/// # Returns
/// PageRankResult with scores for each node
///
/// # Example
/// ```rust,ignore
/// use deepgraph::algorithms::pagerank;
/// use deepgraph::storage::GraphStorage;
/// 
/// let storage = GraphStorage::new();
/// // ... add nodes and edges ...
/// let result = pagerank(&storage, 0.85, 100, 1e-6)?;
/// let top10 = result.top_nodes(10);
/// println!("Top 10 nodes by PageRank: {:?}", top10);
/// ```
pub fn pagerank(
    storage: &GraphStorage,
    damping_factor: f64,
    max_iterations: usize,
    tolerance: f64,
) -> Result<PageRankResult> {
    let all_nodes = storage.get_all_nodes();
    let num_nodes = all_nodes.len();

    if num_nodes == 0 {
        return Ok(PageRankResult {
            scores: HashMap::new(),
            iterations: 0,
            converged: true,
        });
    }

    let initial_rank = 1.0 / num_nodes as f64;
    let mut ranks: HashMap<NodeId, f64> = all_nodes
        .iter()
        .map(|node| (node.id(), initial_rank))
        .collect();

    let mut new_ranks = ranks.clone();
    let mut converged = false;

    // Precompute outgoing edge counts
    let mut out_degrees: HashMap<NodeId, usize> = HashMap::new();
    for node in &all_nodes {
        let node_id = node.id();
        let out_edges = storage.get_outgoing_edges(node_id).unwrap_or_default();
        out_degrees.insert(node_id, out_edges.len());
    }

    for iteration in 0..max_iterations {
        let mut max_diff: f64 = 0.0;

        // Calculate new PageRank for each node
        for node in &all_nodes {
            let node_id = node.id();
            let mut rank_sum = 0.0;

            // Sum contributions from incoming edges
            if let Ok(incoming_edges) = storage.get_incoming_edges(node_id) {
                for edge in incoming_edges {
                    let from_node = edge.from();
                    let from_rank = ranks.get(&from_node).copied().unwrap_or(0.0);
                    let from_out_degree = out_degrees.get(&from_node).copied().unwrap_or(1);

                    if from_out_degree > 0 {
                        rank_sum += from_rank / from_out_degree as f64;
                    }
                }
            }

            // Apply PageRank formula
            let new_rank = (1.0 - damping_factor) / num_nodes as f64 + damping_factor * rank_sum;
            new_ranks.insert(node_id, new_rank);

            // Track convergence
            let diff = (new_rank - ranks.get(&node_id).unwrap()).abs();
            max_diff = max_diff.max(diff);
        }

        // Update ranks
        ranks = new_ranks.clone();

        // Check convergence
        if max_diff < tolerance {
            converged = true;
            return Ok(PageRankResult {
                scores: ranks,
                iterations: iteration + 1,
                converged,
            });
        }
    }

    Ok(PageRankResult {
        scores: ranks,
        iterations: max_iterations,
        converged,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Node;

    #[test]
    fn test_pagerank_simple() {
        let storage = GraphStorage::new();

        // Create a simple graph: 1 -> 2, 1 -> 3, 2 -> 3
        let n1 = Node::new(vec!["Node".to_string()]);
        let n2 = Node::new(vec!["Node".to_string()]);
        let n3 = Node::new(vec!["Node".to_string()]);

        let id1 = storage.add_node(n1).unwrap();
        let id2 = storage.add_node(n2).unwrap();
        let id3 = storage.add_node(n3).unwrap();

        storage
            .add_edge_simple(id1, id2, "LINKS".to_string())
            .unwrap();
        storage
            .add_edge_simple(id1, id3, "LINKS".to_string())
            .unwrap();
        storage
            .add_edge_simple(id2, id3, "LINKS".to_string())
            .unwrap();

        let result = pagerank(&storage, 0.85, 100, 1e-6).unwrap();

        // Node 3 should have highest PageRank (receives links from 1 and 2)
        let rank3 = result.scores.get(&id3).unwrap();
        let rank1 = result.scores.get(&id1).unwrap();
        let rank2 = result.scores.get(&id2).unwrap();

        assert!(rank3 > rank1);
        assert!(rank3 > rank2);
        assert!(result.converged);
    }
}

