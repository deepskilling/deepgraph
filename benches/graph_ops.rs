//! Performance benchmarks for DeepGraph operations

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use deepgraph::{GraphStorage, Node, Edge, PropertyValue};
use deepgraph::index::IndexManager;
use deepgraph::mvcc::TransactionManager;
use deepgraph::wal::{WAL, WALConfig, WALOperation};
use tempfile::tempdir;

fn bench_node_creation(c: &mut Criterion) {
    c.bench_function("node_creation", |b| {
        b.iter(|| {
            let mut node = Node::new(vec!["Person".to_string()]);
            node.set_property("name".to_string(), "Alice".into());
            node.set_property("age".to_string(), 30i64.into());
            black_box(node);
        });
    });
}

fn bench_node_insertion(c: &mut Criterion) {
    let storage = GraphStorage::new();
    
    c.bench_function("node_insertion", |b| {
        b.iter(|| {
            let mut node = Node::new(vec!["Person".to_string()]);
            node.set_property("name".to_string(), "Alice".into());
            storage.add_node(black_box(node)).unwrap();
        });
    });
}

fn bench_node_lookup(c: &mut Criterion) {
    let storage = GraphStorage::new();
    let mut node = Node::new(vec!["Person".to_string()]);
    node.set_property("name".to_string(), "Alice".into());
    let id = storage.add_node(node).unwrap();
    
    c.bench_function("node_lookup", |b| {
        b.iter(|| {
            storage.get_node(black_box(id)).unwrap();
        });
    });
}

fn bench_edge_creation(c: &mut Criterion) {
    let storage = GraphStorage::new();
    let node1 = Node::new(vec!["Person".to_string()]);
    let node2 = Node::new(vec!["Person".to_string()]);
    let id1 = storage.add_node(node1).unwrap();
    let id2 = storage.add_node(node2).unwrap();
    
    c.bench_function("edge_creation", |b| {
        b.iter(|| {
            let mut edge = Edge::new(id1, id2, "KNOWS".to_string());
            edge.set_property("since".to_string(), 2020i64.into());
            storage.add_edge(black_box(edge)).unwrap();
        });
    });
}

fn bench_query_by_label(c: &mut Criterion) {
    let storage = GraphStorage::new();
    
    // Populate with nodes
    for i in 0..1000 {
        let mut node = Node::new(vec!["Person".to_string()]);
        node.set_property("id".to_string(), (i as i64).into());
        storage.add_node(node).unwrap();
    }
    
    c.bench_function("query_by_label_1000_nodes", |b| {
        b.iter(|| {
            storage.get_nodes_by_label(black_box("Person"));
        });
    });
}

fn bench_query_by_property(c: &mut Criterion) {
    let storage = GraphStorage::new();
    
    // Populate with nodes
    for i in 0..1000 {
        let mut node = Node::new(vec!["Person".to_string()]);
        node.set_property("age".to_string(), ((i % 100) as i64).into());
        storage.add_node(node).unwrap();
    }
    
    c.bench_function("query_by_property_1000_nodes", |b| {
        b.iter(|| {
            storage.get_nodes_by_property(
                black_box("age"),
                black_box(&PropertyValue::Integer(30))
            );
        });
    });
}

fn bench_graph_traversal(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_traversal");
    
    for size in [10, 100, 1000].iter() {
        let storage = GraphStorage::new();
        
        // Create a chain of nodes
        let mut prev_id = None;
        for i in 0..*size {
            let mut node = Node::new(vec!["Node".to_string()]);
            node.set_property("id".to_string(), (i as i64).into());
            let id = storage.add_node(node).unwrap();
            
            if let Some(prev) = prev_id {
                let edge = Edge::new(prev, id, "NEXT".to_string());
                storage.add_edge(edge).unwrap();
            }
            prev_id = Some(id);
        }
        
        let start_id = storage.get_all_nodes()[0].id();
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut current = black_box(start_id);
                let mut count = 0;
                
                while let Ok(edges) = storage.get_outgoing_edges(current) {
                    if edges.is_empty() {
                        break;
                    }
                    current = edges[0].to();
                    count += 1;
                    if count >= 100 {
                        break; // Limit iterations for fairness
                    }
                }
            });
        });
    }
    
    group.finish();
}

fn bench_concurrent_reads(c: &mut Criterion) {
    let storage = GraphStorage::new();
    
    // Populate with nodes
    let mut node_ids = Vec::new();
    for i in 0..100 {
        let mut node = Node::new(vec!["Person".to_string()]);
        node.set_property("id".to_string(), (i as i64).into());
        let id = storage.add_node(node).unwrap();
        node_ids.push(id);
    }
    
    c.bench_function("concurrent_reads_100_nodes", |b| {
        b.iter(|| {
            for id in &node_ids {
                storage.get_node(black_box(*id)).unwrap();
            }
        });
    });
}

fn bench_property_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("property_operations");
    
    group.bench_function("set_property", |b| {
        b.iter(|| {
            let mut node = Node::new(vec!["Person".to_string()]);
            node.set_property(black_box("name".to_string()), black_box("Alice".into()));
            node.set_property(black_box("age".to_string()), black_box(30i64.into()));
            node.set_property(black_box("email".to_string()), black_box("alice@example.com".into()));
            black_box(node);
        });
    });
    
    group.bench_function("get_property", |b| {
        let mut node = Node::new(vec!["Person".to_string()]);
        node.set_property("name".to_string(), "Alice".into());
        node.set_property("age".to_string(), 30i64.into());
        
        b.iter(|| {
            black_box(node.get_property(black_box("name")));
            black_box(node.get_property(black_box("age")));
        });
    });
    
    group.finish();
}

// Phase 2 benchmarks

fn bench_hash_index_lookup(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let manager = IndexManager::new(dir.path().to_string_lossy().to_string()).unwrap();
    
    // Populate index
    for i in 0..10000 {
        let key = format!("Person:{}", i);
        manager.insert_hash("person_label", key.as_bytes(), i as u64).unwrap();
    }
    
    c.bench_function("hash_index_lookup_10k", |b| {
        b.iter(|| {
            let key = "Person:5000";
            manager.lookup_hash(black_box("person_label"), black_box(key.as_bytes())).unwrap();
        });
    });
}

fn bench_btree_range_query(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let manager = IndexManager::new(dir.path().to_string_lossy().to_string()).unwrap();
    
    // Populate B-tree
    for i in 0..10000 {
        let key = format!("{:08}", i); // Zero-padded for lexicographic order
        manager.insert_btree("age_index", key.as_bytes(), i as u64).unwrap();
    }
    
    c.bench_function("btree_range_query_10k", |b| {
        b.iter(|| {
            let start = format!("{:08}", 2500);
            let end = format!("{:08}", 7500);
            manager.range_btree(
                black_box("age_index"),
                black_box(start.as_bytes()),
                black_box(end.as_bytes())
            ).unwrap();
        });
    });
}

fn bench_wal_append(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let config = WALConfig::new()
        .with_dir(dir.path().to_string_lossy().to_string())
        .with_sync(false); // Benchmark without sync
    let wal = WAL::new(config).unwrap();
    
    c.bench_function("wal_append", |b| {
        b.iter(|| {
            let node = Node::new(vec!["Test".to_string()]);
            wal.append(black_box(1), black_box(WALOperation::InsertNode { node })).unwrap();
        });
    });
}

fn bench_wal_append_with_sync(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let config = WALConfig::new()
        .with_dir(dir.path().to_string_lossy().to_string())
        .with_sync(true); // With fsync
    let wal = WAL::new(config).unwrap();
    
    c.bench_function("wal_append_with_sync", |b| {
        b.iter(|| {
            let node = Node::new(vec!["Test".to_string()]);
            wal.append(black_box(1), black_box(WALOperation::InsertNode { node })).unwrap();
        });
    });
}

fn bench_mvcc_transaction(c: &mut Criterion) {
    let txn_manager = TransactionManager::new();
    
    c.bench_function("mvcc_begin_commit", |b| {
        b.iter(|| {
            let (txn_id, _snapshot) = txn_manager.begin_transaction().unwrap();
            txn_manager.commit_transaction(black_box(txn_id)).unwrap();
        });
    });
}

fn bench_mvcc_concurrent_transactions(c: &mut Criterion) {
    let txn_manager = TransactionManager::new();
    
    c.bench_function("mvcc_concurrent_10_txns", |b| {
        b.iter(|| {
            let mut txns = Vec::new();
            
            // Begin 10 transactions
            for _ in 0..10 {
                let (txn_id, _snapshot) = txn_manager.begin_transaction().unwrap();
                txns.push(txn_id);
            }
            
            // Commit all
            for txn_id in txns {
                txn_manager.commit_transaction(txn_id).unwrap();
            }
        });
    });
}

fn bench_index_vs_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("index_vs_scan");
    let storage = GraphStorage::new();
    
    // Populate with 1000 nodes
    for i in 0..1000 {
        let mut node = Node::new(vec!["Person".to_string()]);
        node.set_property("id".to_string(), (i as i64).into());
        node.set_property("age".to_string(), ((i % 100) as i64).into());
        storage.add_node(node).unwrap();
    }
    
    // Setup index
    let dir = tempdir().unwrap();
    let manager = IndexManager::new(dir.path().to_string_lossy().to_string()).unwrap();
    
    for (i, node) in storage.get_nodes_by_label("Person").iter().enumerate() {
        let key = format!("Person:{}", i);
        manager.insert_hash("person_id", key.as_bytes(), node.id().0).unwrap();
    }
    
    // Benchmark table scan
    group.bench_function("table_scan_1000", |b| {
        b.iter(|| {
            storage.get_nodes_by_label(black_box("Person"));
        });
    });
    
    // Benchmark index lookup
    group.bench_function("index_lookup_1000", |b| {
        b.iter(|| {
            let key = "Person:500";
            manager.lookup_hash(black_box("person_id"), black_box(key.as_bytes())).unwrap();
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_node_creation,
    bench_node_insertion,
    bench_node_lookup,
    bench_edge_creation,
    bench_query_by_label,
    bench_query_by_property,
    bench_graph_traversal,
    bench_concurrent_reads,
    bench_property_operations,
    // Phase 2 benchmarks
    bench_hash_index_lookup,
    bench_btree_range_query,
    bench_wal_append,
    bench_wal_append_with_sync,
    bench_mvcc_transaction,
    bench_mvcc_concurrent_transactions,
    bench_index_vs_scan,
);

criterion_main!(benches);

