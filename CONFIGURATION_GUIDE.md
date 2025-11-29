# DeepGraph Configuration Guide

## Overview

DeepGraph provides a flexible, multi-layered configuration system that supports:

1. ✅ **Default values** - Sensible defaults for all settings
2. ✅ **TOML configuration files** - Human-readable configuration
3. ✅ **Environment variables** - Override any setting via environment
4. ✅ **Programmatic API** - Configure via Rust/Python code
5. ✅ **Logging framework** - Built-in logging with configurable levels

---

## Configuration Priority

Settings are applied in this order (later overrides earlier):

1. **Hard-coded defaults** (in code)
2. **Configuration file** (`config.toml`)
3. **Environment variables** (`DEEPGRAPH_*`)
4. **Programmatic overrides** (runtime API calls)

---

## Configuration File (config.toml)

### Location

Place `config.toml` in your application's root directory or specify the path:

```rust
// Load from specific path
let config = DeepGraphConfig::from_file("./my_config.toml")?;

// Load with environment variable overrides
let config = DeepGraphConfig::from_file_with_env("config.toml")?;
```

### Complete Example

```toml
# config.toml

[storage]
data_dir = "./data"
enable_cache = true
cache_size_mb = 512

[wal]
enabled = true
wal_dir = "wal"
segment_size_mb = 64
sync_on_write = true
checkpoint_threshold = 1000

[index]
index_dir = "indices"
auto_index = false
default_index_type = "hash"

[algorithm]
pagerank_damping = 0.85
pagerank_max_iterations = 100
pagerank_tolerance = 0.000001
node2vec_walk_length = 80
node2vec_walks_per_node = 10
louvain_max_iterations = 100

[logging]
level = "info"
log_to_file = false
log_to_console = true
```

---

## Environment Variables

Override any configuration setting via environment variables:

### Storage

```bash
export DEEPGRAPH_DATA_DIR=./production_data
export DEEPGRAPH_CACHE_SIZE_MB=1024
```

### WAL (Write-Ahead Log)

```bash
export DEEPGRAPH_WAL_ENABLED=true
export DEEPGRAPH_WAL_DIR=./data/wal
export DEEPGRAPH_WAL_SYNC=true
```

### Logging

```bash
export DEEPGRAPH_LOG_LEVEL=debug

# Or use RUST_LOG for more control
export RUST_LOG=deepgraph=debug,info
```

### Complete Example

```bash
# env.example
DEEPGRAPH_DATA_DIR=./data
DEEPGRAPH_CACHE_SIZE_MB=512
DEEPGRAPH_WAL_ENABLED=true
DEEPGRAPH_WAL_DIR=./data/wal
DEEPGRAPH_WAL_SYNC=true
DEEPGRAPH_LOG_LEVEL=info
```

---

## Programmatic Configuration

### Rust API

```rust
use deepgraph::DeepGraphConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load from file
    let config = DeepGraphConfig::from_file("config.toml")?;
    
    // 2. Initialize logging
    config.init_logging()?;
    
    // 3. Access configuration values
    println!("Data directory: {}", config.storage.data_dir);
    println!("WAL enabled: {}", config.wal.enabled);
    
    // 4. Get computed paths
    let wal_path = config.wal_path();
    let index_path = config.index_path();
    
    // Use the configuration...
    
    Ok(())
}
```

### Load from Environment Only

```rust
use deepgraph::DeepGraphConfig;

fn main() {
    // Load configuration from environment variables only
    let config = DeepGraphConfig::from_env();
    config.init_logging().expect("Failed to initialize logging");
    
    // Use config...
}
```

### Modify and Save

```rust
use deepgraph::DeepGraphConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load default configuration
    let mut config = DeepGraphConfig::default();
    
    // Modify settings
    config.storage.data_dir = "/var/lib/deepgraph".to_string();
    config.wal.segment_size_mb = 128;
    config.algorithm.pagerank_damping = 0.90;
    
    // Save to file
    config.save_to_file("config.toml")?;
    
    Ok(())
}
```

---

## Logging Configuration

### Log Levels

Available log levels (from most to least verbose):

- `trace` - Very detailed debugging information
- `debug` - Debug information useful for development
- `info` - Informational messages (default)
- `warn` - Warning messages
- `error` - Error messages only

### Configure via TOML

```toml
[logging]
level = "info"              # or "debug", "warn", "error", "trace"
log_to_file = false         # Write logs to file
log_file = "./deepgraph.log"  # File path (if log_to_file = true)
log_to_console = true       # Write logs to stderr/console
```

### Configure via Environment

```bash
# Set log level
export DEEPGRAPH_LOG_LEVEL=debug

# Or use RUST_LOG for module-level control
export RUST_LOG=deepgraph=debug,deepgraph::algorithms=trace
```

### Log Output Format

```
2025-01-07 14:32:15 [INFO] DeepGraph initialized with config
2025-01-07 14:32:15 [INFO] Data directory: ./data
2025-01-07 14:32:15 [INFO] WAL enabled: true
2025-01-07 14:32:16 [INFO] Starting BFS from node 123e4567-...
2025-01-07 14:32:16 [INFO] BFS completed: visited 100 nodes
```

---

## Configuration Reference

### Storage Configuration

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `data_dir` | String | `"./data"` | Base directory for all persistent files |
| `enable_cache` | Boolean | `true` | Enable in-memory caching |
| `cache_size_mb` | Integer | `512` | Cache size in megabytes |

### WAL Configuration

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `enabled` | Boolean | `true` | Enable Write-Ahead Logging |
| `wal_dir` | String | `"wal"` | WAL directory (relative or absolute) |
| `segment_size_mb` | Integer | `64` | Segment file size in MB |
| `sync_on_write` | Boolean | `true` | Sync to disk on each write (durability) |
| `checkpoint_threshold` | Integer | `1000` | Auto-checkpoint after N entries |

### Index Configuration

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `index_dir` | String | `"indices"` | Index directory (relative or absolute) |
| `auto_index` | Boolean | `false` | Auto-create indices for hot properties |
| `default_index_type` | String | `"hash"` | Default index type ("hash" or "btree") |

### Algorithm Configuration

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `pagerank_damping` | Float | `0.85` | PageRank damping factor |
| `pagerank_max_iterations` | Integer | `100` | PageRank max iterations |
| `pagerank_tolerance` | Float | `0.000001` | PageRank convergence tolerance |
| `node2vec_walk_length` | Integer | `80` | Node2Vec walk length |
| `node2vec_walks_per_node` | Integer | `10` | Node2Vec walks per node |
| `louvain_max_iterations` | Integer | `100` | Louvain max iterations |

---

## Production Deployment

### Recommended Production Settings

```toml
[storage]
data_dir = "/var/lib/deepgraph"
enable_cache = true
cache_size_mb = 2048

[wal]
enabled = true
wal_dir = "/var/lib/deepgraph/wal"
segment_size_mb = 128
sync_on_write = true
checkpoint_threshold = 10000

[logging]
level = "warn"
log_to_file = true
log_file = "/var/log/deepgraph/deepgraph.log"
log_to_console = false
```

### Docker Deployment

```dockerfile
# Dockerfile
FROM rust:1.75

WORKDIR /app
COPY . .

# Build application
RUN cargo build --release

# Create data directories
RUN mkdir -p /data/wal /data/indices

# Copy configuration
COPY config.production.toml /app/config.toml

# Set environment variables
ENV DEEPGRAPH_DATA_DIR=/data
ENV DEEPGRAPH_LOG_LEVEL=info

VOLUME ["/data"]

CMD ["./target/release/your-app"]
```

### Docker Compose

```yaml
version: '3.8'

services:
  deepgraph:
    image: your-deepgraph-app:latest
    environment:
      - DEEPGRAPH_DATA_DIR=/data
      - DEEPGRAPH_WAL_DIR=/data/wal
      - DEEPGRAPH_LOG_LEVEL=info
    volumes:
      - deepgraph-data:/data
    restart: unless-stopped

volumes:
  deepgraph-data:
```

---

## Best Practices

### 1. Use Configuration Files for Defaults

Store your base configuration in `config.toml`:

```toml
[storage]
data_dir = "./data"
cache_size_mb = 512
```

### 2. Use Environment Variables for Deployment

Override per-environment settings:

```bash
# Production
export DEEPGRAPH_DATA_DIR=/var/lib/deepgraph
export DEEPGRAPH_CACHE_SIZE_MB=2048
export DEEPGRAPH_LOG_LEVEL=warn

# Development
export DEEPGRAPH_LOG_LEVEL=debug
```

### 3. Version Control Configuration

- ✅ **DO** commit `config.toml` (default configuration)
- ✅ **DO** commit `env.example` (template for environment variables)
- ❌ **DO NOT** commit `.env` (actual secrets/credentials)
- ❌ **DO NOT** commit production configs with secrets

### 4. Initialize Logging Early

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config and initialize logging FIRST
    let config = DeepGraphConfig::from_file_with_env("config.toml")?;
    config.init_logging()?;
    
    // Now all log::info!() calls will work
    log::info!("Application started");
    
    // ... rest of application
    
    Ok(())
}
```

---

## Troubleshooting

### Issue: Configuration file not found

**Solution**: Ensure `config.toml` is in the correct location or specify full path:

```rust
let config = DeepGraphConfig::from_file("./config/deepgraph.toml")?;
```

### Issue: Environment variables not working

**Solution**: Check variable names and ensure they're exported:

```bash
# Check if variable is set
echo $DEEPGRAPH_LOG_LEVEL

# Verify it's exported
export | grep DEEPGRAPH
```

### Issue: Logs not appearing

**Solution**: 
1. Check log level configuration
2. Ensure `init_logging()` is called
3. Check if `log_to_console` is enabled

```rust
let config = DeepGraphConfig::from_env();
config.init_logging()?;  // Must call this!
```

---

## Example: Complete Application

```rust
use deepgraph::{DeepGraphConfig, GraphStorage, Node};
use log::{info, warn, error};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load configuration
    let config = DeepGraphConfig::from_file_with_env("config.toml")?;
    
    // 2. Initialize logging
    config.init_logging()?;
    
    info!("DeepGraph application starting");
    info!("Data directory: {}", config.storage.data_dir);
    
    // 3. Create storage
    let storage = GraphStorage::new();
    info!("Storage initialized");
    
    // 4. Your application logic
    let node = Node::new(vec!["Person".to_string()]);
    match storage.add_node(node) {
        Ok(id) => info!("Created node: {}", id),
        Err(e) => error!("Failed to create node: {}", e),
    }
    
    info!("Application completed successfully");
    Ok(())
}
```

---

## Summary

DeepGraph's configuration system provides:

- ✅ **Flexible configuration** - File, environment, or programmatic
- ✅ **Sensible defaults** - Works out of the box
- ✅ **Environment-aware** - Easy deployment across environments
- ✅ **Integrated logging** - Built-in logging framework
- ✅ **Production-ready** - Suitable for production deployments

For more information, see:
- `config.toml` - Example configuration file
- `env.example` - Environment variable template
- `examples/rust/config_demo.rs` - Working example

---

**DeepGraph** - High-Performance Graph Database  
© 2025 DeepSkilling. Licensed under MIT.

