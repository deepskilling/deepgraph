//! Demonstration of DeepGraph configuration and logging

use deepgraph::{DeepGraphConfig, GraphStorage, Node};
use deepgraph::algorithms::bfs;
use log::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DeepGraph Configuration Demo ===\n");
    
    // 1. Load configuration from file
    println!("1. Loading configuration from config.toml...");
    let config = DeepGraphConfig::from_file("config.toml")?;
    
    // Initialize logging based on configuration
    config.init_logging()?;
    
    info!("DeepGraph initialized with config");
    info!("Data directory: {}", config.storage.data_dir);
    info!("WAL enabled: {}", config.wal.enabled);
    info!("WAL path: {:?}", config.wal_path());
    
    // 2. Create storage
    println!("\n2. Creating graph storage...");
    let storage = GraphStorage::new();
    info!("GraphStorage created");
    
    // 3. Add some nodes
    println!("\n3. Adding nodes...");
    let node1 = Node::new(vec!["Person".to_string()]);
    let node2 = Node::new(vec!["Person".to_string()]);
    let node3 = Node::new(vec!["Person".to_string()]);
    
    let id1 = storage.add_node(node1)?;
    let id2 = storage.add_node(node2)?;
    let id3 = storage.add_node(node3)?;
    
    info!("Created nodes: {}, {}, {}", id1, id2, id3);
    
    // 4. Add edges
    println!("\n4. Adding edges...");
    storage.add_edge_simple(id1, id2, "KNOWS".to_string())?;
    storage.add_edge_simple(id2, id3, "KNOWS".to_string())?;
    info!("Created edges");
    
    // 5. Run algorithm (with logging)
    println!("\n5. Running BFS algorithm...");
    let result = bfs(&storage, id1, None)?;
    
    println!("\n=== Results ===");
    println!("BFS visited {} nodes", result.visited.len());
    println!("Distance from node1 to node3: {:?}", result.distances.get(&id3));
    
    // 6. Display configuration values
    println!("\n=== Configuration Values ===");
    println!("PageRank damping: {}", config.algorithm.pagerank_damping);
    println!("Node2Vec walk length: {}", config.algorithm.node2vec_walk_length);
    println!("Cache size: {} MB", config.storage.cache_size_mb);
    
    // 7. Test environment variable override
    println!("\n=== Environment Variable Overrides ===");
    println!("Set DEEPGRAPH_LOG_LEVEL to override log level");
    println!("Set DEEPGRAPH_DATA_DIR to change data directory");
    println!("See env.example for all available variables");
    
    // 8. Save config example
    println!("\n=== Saving Configuration ===");
    let output_config = DeepGraphConfig::default();
    output_config.save_to_file("config.example.toml")?;
    println!("Saved default configuration to config.example.toml");
    
    info!("Demo completed successfully");
    println!("\n✅ Configuration demo complete!");
    
    Ok(())
}

