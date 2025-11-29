//! Configuration management for DeepGraph
//!
//! Provides centralized configuration from multiple sources:
//! - Default values
//! - TOML configuration files
//! - Environment variables
//! - Programmatic builder pattern

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use crate::error::{DeepGraphError, Result};
use log::{info, warn, debug};

/// Main DeepGraph configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepGraphConfig {
    /// Storage configuration
    pub storage: StorageConfig,
    
    /// WAL configuration
    pub wal: WALConfigOptions,
    
    /// Index configuration
    pub index: IndexConfig,
    
    /// Algorithm configuration
    pub algorithm: AlgorithmConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Base data directory
    pub data_dir: String,
    
    /// Enable in-memory caching
    pub enable_cache: bool,
    
    /// Cache size in MB
    pub cache_size_mb: usize,
}

/// WAL configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WALConfigOptions {
    /// Enable WAL
    pub enabled: bool,
    
    /// WAL directory (relative to data_dir or absolute)
    pub wal_dir: String,
    
    /// Segment size in MB
    pub segment_size_mb: usize,
    
    /// Sync on every write (durability vs performance)
    pub sync_on_write: bool,
    
    /// Auto-checkpoint threshold
    pub checkpoint_threshold: usize,
}

/// Index configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    /// Index directory (relative to data_dir or absolute)
    pub index_dir: String,
    
    /// Enable automatic index creation
    pub auto_index: bool,
    
    /// Default index type ("hash" or "btree")
    pub default_index_type: String,
}

/// Algorithm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmConfig {
    /// PageRank default damping factor
    pub pagerank_damping: f64,
    
    /// PageRank default max iterations
    pub pagerank_max_iterations: usize,
    
    /// PageRank convergence tolerance
    pub pagerank_tolerance: f64,
    
    /// Node2Vec default walk length
    pub node2vec_walk_length: usize,
    
    /// Node2Vec default walks per node
    pub node2vec_walks_per_node: usize,
    
    /// Louvain max iterations
    pub louvain_max_iterations: usize,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level: "error", "warn", "info", "debug", "trace"
    pub level: String,
    
    /// Log to file
    pub log_to_file: bool,
    
    /// Log file path
    pub log_file: Option<String>,
    
    /// Log to console
    pub log_to_console: bool,
}

impl Default for DeepGraphConfig {
    fn default() -> Self {
        Self {
            storage: StorageConfig::default(),
            wal: WALConfigOptions::default(),
            index: IndexConfig::default(),
            algorithm: AlgorithmConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: "./data".to_string(),
            enable_cache: true,
            cache_size_mb: 512,
        }
    }
}

impl Default for WALConfigOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            wal_dir: "wal".to_string(),
            segment_size_mb: 64,
            sync_on_write: true,
            checkpoint_threshold: 1000,
        }
    }
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            index_dir: "indices".to_string(),
            auto_index: false,
            default_index_type: "hash".to_string(),
        }
    }
}

impl Default for AlgorithmConfig {
    fn default() -> Self {
        Self {
            pagerank_damping: 0.85,
            pagerank_max_iterations: 100,
            pagerank_tolerance: 1e-6,
            node2vec_walk_length: 80,
            node2vec_walks_per_node: 10,
            louvain_max_iterations: 100,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            log_to_file: false,
            log_file: None,
            log_to_console: true,
        }
    }
}

impl DeepGraphConfig {
    /// Load configuration from a TOML file
    pub fn from_file(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        info!("Loading configuration from {:?}", path);
        
        let contents = fs::read_to_string(&path)
            .map_err(|e| DeepGraphError::IoError(e))?;
        
        let config: Self = toml::from_str(&contents)
            .map_err(|e| DeepGraphError::Unknown(format!("Failed to parse TOML config: {}", e)))?;
        
        debug!("Configuration loaded successfully");
        Ok(config)
    }
    
    /// Load configuration with environment variable overrides
    pub fn from_file_with_env(path: impl Into<PathBuf>) -> Result<Self> {
        let mut config = Self::from_file(path)?;
        config.apply_env_overrides();
        Ok(config)
    }
    
    /// Create configuration from environment variables only
    pub fn from_env() -> Self {
        info!("Loading configuration from environment variables");
        let mut config = Self::default();
        config.apply_env_overrides();
        config
    }
    
    /// Apply environment variable overrides
    pub fn apply_env_overrides(&mut self) {
        // Storage
        if let Ok(val) = std::env::var("DEEPGRAPH_DATA_DIR") {
            debug!("Override data_dir from env: {}", val);
            self.storage.data_dir = val;
        }
        
        if let Ok(val) = std::env::var("DEEPGRAPH_CACHE_SIZE_MB") {
            if let Ok(size) = val.parse() {
                debug!("Override cache_size_mb from env: {}", size);
                self.storage.cache_size_mb = size;
            }
        }
        
        // WAL
        if let Ok(val) = std::env::var("DEEPGRAPH_WAL_ENABLED") {
            if let Ok(enabled) = val.parse() {
                debug!("Override WAL enabled from env: {}", enabled);
                self.wal.enabled = enabled;
            }
        }
        
        if let Ok(val) = std::env::var("DEEPGRAPH_WAL_DIR") {
            debug!("Override wal_dir from env: {}", val);
            self.wal.wal_dir = val;
        }
        
        if let Ok(val) = std::env::var("DEEPGRAPH_WAL_SYNC") {
            if let Ok(sync) = val.parse() {
                debug!("Override WAL sync from env: {}", sync);
                self.wal.sync_on_write = sync;
            }
        }
        
        // Logging
        if let Ok(val) = std::env::var("DEEPGRAPH_LOG_LEVEL") {
            debug!("Override log level from env: {}", val);
            self.logging.level = val;
        }
        
        if let Ok(val) = std::env::var("RUST_LOG") {
            debug!("Override log level from RUST_LOG: {}", val);
            self.logging.level = val;
        }
    }
    
    /// Save configuration to a TOML file
    pub fn save_to_file(&self, path: impl Into<PathBuf>) -> Result<()> {
        let path = path.into();
        info!("Saving configuration to {:?}", path);
        
        let contents = toml::to_string_pretty(self)
            .map_err(|e| DeepGraphError::Unknown(format!("Failed to serialize config: {}", e)))?;
        
        fs::write(&path, contents)
            .map_err(|e| DeepGraphError::IoError(e))?;
        
        info!("Configuration saved successfully");
        Ok(())
    }
    
    /// Get full WAL directory path
    pub fn wal_path(&self) -> PathBuf {
        let wal_dir = PathBuf::from(&self.wal.wal_dir);
        if wal_dir.is_absolute() {
            wal_dir
        } else {
            PathBuf::from(&self.storage.data_dir).join(wal_dir)
        }
    }
    
    /// Get full index directory path
    pub fn index_path(&self) -> PathBuf {
        let index_dir = PathBuf::from(&self.index.index_dir);
        if index_dir.is_absolute() {
            index_dir
        } else {
            PathBuf::from(&self.storage.data_dir).join(index_dir)
        }
    }
    
    /// Initialize logging based on configuration
    pub fn init_logging(&self) -> Result<()> {
        use env_logger::Builder;
        use std::io::Write;
        
        let log_level = match self.logging.level.to_lowercase().as_str() {
            "error" => log::LevelFilter::Error,
            "warn" => log::LevelFilter::Warn,
            "info" => log::LevelFilter::Info,
            "debug" => log::LevelFilter::Debug,
            "trace" => log::LevelFilter::Trace,
            _ => {
                warn!("Invalid log level '{}', defaulting to 'info'", self.logging.level);
                log::LevelFilter::Info
            }
        };
        
        let mut builder = Builder::new();
        builder.filter_level(log_level);
        builder.format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        });
        
        if self.logging.log_to_file {
            if let Some(ref log_file) = self.logging.log_file {
                // For now, we'll use env_logger which writes to stderr
                // A production system might want to use a more sophisticated logging framework
                warn!("File logging configured but not yet implemented: {}", log_file);
            }
        }
        
        builder.try_init()
            .map_err(|e| DeepGraphError::Unknown(format!("Failed to initialize logger: {}", e)))?;
        
        info!("Logging initialized at level: {}", self.logging.level);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = DeepGraphConfig::default();
        assert_eq!(config.storage.data_dir, "./data");
        assert!(config.wal.enabled);
        assert_eq!(config.algorithm.pagerank_damping, 0.85);
    }
    
    #[test]
    fn test_config_paths() {
        let config = DeepGraphConfig::default();
        assert_eq!(config.wal_path(), PathBuf::from("./data/wal"));
        assert_eq!(config.index_path(), PathBuf::from("./data/indices"));
    }
}

