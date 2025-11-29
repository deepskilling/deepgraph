//! Tests for configuration and logging

use deepgraph::DeepGraphConfig;
use std::env;

#[test]
fn test_default_config() {
    let config = DeepGraphConfig::default();
    assert_eq!(config.storage.data_dir, "./data");
    assert!(config.wal.enabled);
    assert_eq!(config.wal.segment_size_mb, 64);
    assert_eq!(config.algorithm.pagerank_damping, 0.85);
}

#[test]
fn test_config_paths() {
    let config = DeepGraphConfig::default();
    let wal_path = config.wal_path();
    let index_path = config.index_path();
    
    assert!(wal_path.to_str().unwrap().contains("data"));
    assert!(wal_path.to_str().unwrap().contains("wal"));
    assert!(index_path.to_str().unwrap().contains("indices"));
}

#[test]
fn test_env_override() {
    env::set_var("DEEPGRAPH_DATA_DIR", "/tmp/test_data");
    env::set_var("DEEPGRAPH_CACHE_SIZE_MB", "1024");
    env::set_var("DEEPGRAPH_WAL_SYNC", "false");
    
    let config = DeepGraphConfig::from_env();
    
    assert_eq!(config.storage.data_dir, "/tmp/test_data");
    assert_eq!(config.storage.cache_size_mb, 1024);
    assert_eq!(config.wal.sync_on_write, false);
    
    // Clean up
    env::remove_var("DEEPGRAPH_DATA_DIR");
    env::remove_var("DEEPGRAPH_CACHE_SIZE_MB");
    env::remove_var("DEEPGRAPH_WAL_SYNC");
}

#[test]
fn test_config_save_load() {
    use tempfile::NamedTempFile;
    
    let temp_file = NamedTempFile::new().unwrap();
    let config_path = temp_file.path();
    
    // Create and save config
    let mut config = DeepGraphConfig::default();
    config.storage.data_dir = "/custom/path".to_string();
    config.algorithm.pagerank_damping = 0.90;
    
    config.save_to_file(&config_path).unwrap();
    
    // Load config
    let loaded_config = DeepGraphConfig::from_file(&config_path).unwrap();
    
    assert_eq!(loaded_config.storage.data_dir, "/custom/path");
    assert_eq!(loaded_config.algorithm.pagerank_damping, 0.90);
}

#[test]
fn test_algorithm_config() {
    let config = DeepGraphConfig::default();
    
    // PageRank
    assert_eq!(config.algorithm.pagerank_damping, 0.85);
    assert_eq!(config.algorithm.pagerank_max_iterations, 100);
    assert_eq!(config.algorithm.pagerank_tolerance, 1e-6);
    
    // Node2Vec
    assert_eq!(config.algorithm.node2vec_walk_length, 80);
    assert_eq!(config.algorithm.node2vec_walks_per_node, 10);
    
    // Louvain
    assert_eq!(config.algorithm.louvain_max_iterations, 100);
}

