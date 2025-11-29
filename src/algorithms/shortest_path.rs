//! Shortest path algorithms

use crate::error::{DeepGraphError, Result};
use crate::graph::{NodeId, PropertyValue};
use crate::storage::GraphStorage;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Result of Dijkstra's shortest path algorithm
#[derive(Debug, Clone)]
pub struct DijkstraResult {
    /// Shortest distance from source to each node
    pub distances: HashMap<NodeId, f64>,
    /// Previous node in the shortest path
    pub previous: HashMap<NodeId, Option<NodeId>>,
    /// Source node
    pub source: NodeId,
}

impl DijkstraResult {
    /// Get the shortest path from source to target
    pub fn path_to(&self, target: NodeId) -> Option<Vec<NodeId>> {
        if !self.distances.contains_key(&target) {
            return None;
        }

        let mut path = Vec::new();
        let mut current = Some(target);

        while let Some(node) = current {
            path.push(node);
            current = *self.previous.get(&node)?;
        }

        path.reverse();
        Some(path)
    }

    /// Get the distance to a target node
    pub fn distance_to(&self, target: NodeId) -> Option<f64> {
        self.distances.get(&target).copied()
    }
}

#[derive(Clone, Copy)]
struct State {
    cost: f64,
    node: NodeId,
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for State {}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

/// Dijkstra's shortest path algorithm
///
/// Finds shortest paths from a source node to all reachable nodes.
/// Edge weights are extracted from the "weight" property (default 1.0).
///
/// # Arguments
/// * `storage` - Graph storage
/// * `source` - Source node ID
/// * `weight_property` - Optional property name for edge weights (default: "weight")
///
/// # Returns
/// DijkstraResult with distances and paths
///
/// # Example
/// ```rust,ignore
/// use deepgraph::algorithms::dijkstra;
/// use deepgraph::storage::GraphStorage;
/// 
/// let storage = GraphStorage::new();
/// // ... add nodes and edges ...
/// let result = dijkstra(&storage, source_id, Some("weight"))?;
/// if let Some(path) = result.path_to(target_id) {
///     println!("Shortest path: {:?}", path);
/// }
/// ```
pub fn dijkstra(
    storage: &GraphStorage,
    source: NodeId,
    weight_property: Option<&str>,
) -> Result<DijkstraResult> {
    // Verify source node exists
    storage.get_node(source)?;

    let weight_key = weight_property.unwrap_or("weight");

    let mut distances: HashMap<NodeId, f64> = HashMap::new();
    let mut previous: HashMap<NodeId, Option<NodeId>> = HashMap::new();
    let mut heap = BinaryHeap::new();
    let mut visited = HashSet::new();

    // Initialize source
    distances.insert(source, 0.0);
    previous.insert(source, None);
    heap.push(State {
        cost: 0.0,
        node: source,
    });

    while let Some(State { cost, node }) = heap.pop() {
        // Skip if already visited
        if visited.contains(&node) {
            continue;
        }
        visited.insert(node);

        // Skip if we found a better path already
        if let Some(&dist) = distances.get(&node) {
            if cost > dist {
                continue;
            }
        }

        // Check neighbors
        if let Ok(edges) = storage.get_outgoing_edges(node) {
            for edge in edges {
                let neighbor = edge.to();

                // Get edge weight
                let weight = edge
                    .properties()
                    .get(weight_key)
                    .and_then(|v| match v {
                        PropertyValue::Float(f) => Some(*f),
                        PropertyValue::Integer(i) => Some(*i as f64),
                        _ => None,
                    })
                    .unwrap_or(1.0);

                if weight < 0.0 {
                    return Err(DeepGraphError::InvalidOperation(
                        "Negative edge weights not supported in Dijkstra".to_string(),
                    ));
                }

                let next_cost = cost + weight;

                // Update if we found a better path
                let current_dist = distances.get(&neighbor).copied().unwrap_or(f64::INFINITY);
                if next_cost < current_dist {
                    distances.insert(neighbor, next_cost);
                    previous.insert(neighbor, Some(node));
                    heap.push(State {
                        cost: next_cost,
                        node: neighbor,
                    });
                }
            }
        }
    }

    Ok(DijkstraResult {
        distances,
        previous,
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Edge, Node};
    use std::collections::HashMap;

    #[test]
    fn test_dijkstra_simple() {
        let storage = GraphStorage::new();

        // Create graph: 1 --(1)--> 2 --(2)--> 3
        let n1 = Node::new(vec!["Node".to_string()]);
        let n2 = Node::new(vec!["Node".to_string()]);
        let n3 = Node::new(vec!["Node".to_string()]);

        let id1 = storage.add_node(n1).unwrap();
        let id2 = storage.add_node(n2).unwrap();
        let id3 = storage.add_node(n3).unwrap();

        let mut props1 = HashMap::new();
        props1.insert("weight".to_string(), PropertyValue::Float(1.0));
        let mut props2 = HashMap::new();
        props2.insert("weight".to_string(), PropertyValue::Float(2.0));

        storage
            .add_edge_with_properties(id1, id2, "CONNECTS".to_string(), props1)
            .unwrap();
        storage
            .add_edge_with_properties(id2, id3, "CONNECTS".to_string(), props2)
            .unwrap();

        let result = dijkstra(&storage, id1, Some("weight")).unwrap();

        assert_eq!(*result.distances.get(&id1).unwrap(), 0.0);
        assert_eq!(*result.distances.get(&id2).unwrap(), 1.0);
        assert_eq!(*result.distances.get(&id3).unwrap(), 3.0);

        let path = result.path_to(id3).unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], id1);
        assert_eq!(path[2], id3);
    }
}

