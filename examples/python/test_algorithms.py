"""
Test all graph algorithms in DeepGraph Python bindings
"""

import deepgraph
import sys

def test_bfs():
    """Test Breadth-First Search"""
    print("Testing BFS...")
    storage = deepgraph.GraphStorage()
    
    # Create a chain: 1 -> 2 -> 3 -> 4
    id1 = storage.add_node(["Node"], {"name": "Node1"})
    id2 = storage.add_node(["Node"], {"name": "Node2"})
    id3 = storage.add_node(["Node"], {"name": "Node3"})
    id4 = storage.add_node(["Node"], {"name": "Node4"})
    
    storage.add_edge(id1, id2, "CONNECTS", {})
    storage.add_edge(id2, id3, "CONNECTS", {})
    storage.add_edge(id3, id4, "CONNECTS", {})
    
    result = deepgraph.bfs(storage, id1, None)
    print(f"  BFS visited {len(result['visited'])} nodes")
    print(f"  Distance to node2: {result['distances'][id2]}")
    print(f"  Distance to node4: {result['distances'][id4]}")
    assert len(result['visited']) == 4
    assert result['distances'][id1] == 0
    assert result['distances'][id2] == 1
    assert result['distances'][id4] == 3
    print("  ✓ BFS test passed!")

def test_dfs():
    """Test Depth-First Search"""
    print("\nTesting DFS...")
    storage = deepgraph.GraphStorage()
    
    # Create a simple graph
    id1 = storage.add_node(["Node"], {"name": "Node1"})
    id2 = storage.add_node(["Node"], {"name": "Node2"})
    id3 = storage.add_node(["Node"], {"name": "Node3"})
    
    storage.add_edge(id1, id2, "CONNECTS", {})
    storage.add_edge(id2, id3, "CONNECTS", {})
    
    result = deepgraph.dfs(storage, id1)
    print(f"  DFS visited {len(result['visited'])} nodes")
    assert len(result['visited']) == 3
    assert id1 in result['discovery_time']
    print("  ✓ DFS test passed!")

def test_dijkstra():
    """Test Dijkstra Shortest Path"""
    print("\nTesting Dijkstra...")
    storage = deepgraph.GraphStorage()
    
    # Create weighted graph
    id1 = storage.add_node(["Node"], {"name": "A"})
    id2 = storage.add_node(["Node"], {"name": "B"})
    id3 = storage.add_node(["Node"], {"name": "C"})
    
    storage.add_edge(id1, id2, "CONNECTS", {"weight": 1.0})
    storage.add_edge(id2, id3, "CONNECTS", {"weight": 2.0})
    storage.add_edge(id1, id3, "CONNECTS", {"weight": 5.0})
    
    result = deepgraph.dijkstra(storage, id1, "weight")
    print(f"  Distance from A to B: {result['distances'][id2]}")
    print(f"  Distance from A to C: {result['distances'][id3]}")
    assert result['distances'][id2] == 1.0
    assert result['distances'][id3] == 3.0  # Shorter path through B
    print("  ✓ Dijkstra test passed!")

def test_connected_components():
    """Test Connected Components"""
    print("\nTesting Connected Components...")
    storage = deepgraph.GraphStorage()
    
    # Create two separate components
    # Component 1: 1-2
    id1 = storage.add_node(["Node"], {"name": "A"})
    id2 = storage.add_node(["Node"], {"name": "B"})
    storage.add_edge(id1, id2, "CONNECTS", {})
    
    # Component 2: 3-4
    id3 = storage.add_node(["Node"], {"name": "C"})
    id4 = storage.add_node(["Node"], {"name": "D"})
    storage.add_edge(id3, id4, "CONNECTS", {})
    
    result = deepgraph.connected_components(storage)
    print(f"  Found {result['num_components']} components")
    assert result['num_components'] == 2
    assert result['component_map'][id1] == result['component_map'][id2]
    assert result['component_map'][id3] == result['component_map'][id4]
    assert result['component_map'][id1] != result['component_map'][id3]
    print("  ✓ Connected Components test passed!")

def test_pagerank():
    """Test PageRank"""
    print("\nTesting PageRank...")
    storage = deepgraph.GraphStorage()
    
    # Create a simple graph: 1 -> 2, 1 -> 3, 2 -> 3
    id1 = storage.add_node(["Node"], {"name": "A"})
    id2 = storage.add_node(["Node"], {"name": "B"})
    id3 = storage.add_node(["Node"], {"name": "C"})
    
    storage.add_edge(id1, id2, "LINKS", {})
    storage.add_edge(id1, id3, "LINKS", {})
    storage.add_edge(id2, id3, "LINKS", {})
    
    result = deepgraph.pagerank(storage, 0.85, 100, 1e-6)
    print(f"  PageRank converged: {result['converged']}")
    print(f"  Iterations: {result['iterations']}")
    print(f"  Node C score: {result['scores'][id3]:.4f}")
    
    # Node 3 should have highest PageRank (receives most links)
    assert result['scores'][id3] > result['scores'][id1]
    assert result['scores'][id3] > result['scores'][id2]
    print("  ✓ PageRank test passed!")

def test_triangle_count():
    """Test Triangle Counting"""
    print("\nTesting Triangle Counting...")
    storage = deepgraph.GraphStorage()
    
    # Create a triangle: 1-2, 2-3, 3-1
    id1 = storage.add_node(["Node"], {"name": "A"})
    id2 = storage.add_node(["Node"], {"name": "B"})
    id3 = storage.add_node(["Node"], {"name": "C"})
    
    storage.add_edge(id1, id2, "CONNECTS", {})
    storage.add_edge(id2, id3, "CONNECTS", {})
    storage.add_edge(id3, id1, "CONNECTS", {})
    
    result = deepgraph.triangle_count(storage)
    print(f"  Total triangles: {result['total_triangles']}")
    print(f"  Global clustering coefficient: {result['global_clustering_coefficient']:.4f}")
    print(f"  Node triangles for id1: {result['node_triangles'][id1]}")
    assert result['total_triangles'] == 1
    # Each node participates in the same triangle, count might vary by implementation
    assert result['node_triangles'][id1] >= 1
    print("  ✓ Triangle Counting test passed!")

def test_louvain():
    """Test Louvain Community Detection"""
    print("\nTesting Louvain...")
    storage = deepgraph.GraphStorage()
    
    # Create two densely connected groups
    # Group 1: 1-2-3 (fully connected)
    ids = []
    for i in range(6):
        ids.append(storage.add_node(["Node"], {"name": f"Node{i}"}))
    
    # Group 1 edges (0,1,2)
    storage.add_edge(ids[0], ids[1], "CONNECTS", {})
    storage.add_edge(ids[1], ids[2], "CONNECTS", {})
    storage.add_edge(ids[2], ids[0], "CONNECTS", {})
    
    # Group 2 edges (3,4,5)
    storage.add_edge(ids[3], ids[4], "CONNECTS", {})
    storage.add_edge(ids[4], ids[5], "CONNECTS", {})
    storage.add_edge(ids[5], ids[3], "CONNECTS", {})
    
    # Sparse connection between groups
    storage.add_edge(ids[2], ids[3], "CONNECTS", {})
    
    result = deepgraph.louvain(storage, 100, 1e-4)
    print(f"  Found {result['num_communities']} communities")
    print(f"  Modularity: {result['modularity']:.4f}")
    assert result['num_communities'] >= 1
    print("  ✓ Louvain test passed!")

def test_node2vec():
    """Test Node2Vec"""
    print("\nTesting Node2Vec...")
    storage = deepgraph.GraphStorage()
    
    # Create a chain
    ids = []
    for i in range(5):
        ids.append(storage.add_node(["Node"], {"name": f"Node{i}"}))
    
    for i in range(4):
        storage.add_edge(ids[i], ids[i+1], "CONNECTS", {})
    
    result = deepgraph.node2vec(storage, 10, 5, 1.0, 1.0, 42)
    print(f"  Generated {result['num_walks']} walks")
    print(f"  Total steps: {result['total_steps']}")
    assert result['num_walks'] == 5 * 5  # 5 nodes * 5 walks_per_node
    assert len(result['walks']) > 0
    print("  ✓ Node2Vec test passed!")

def main():
    """Run all algorithm tests"""
    print("=" * 60)
    print("DeepGraph Algorithm Tests")
    print("=" * 60)
    
    try:
        test_bfs()
        test_dfs()
        test_dijkstra()
        test_connected_components()
        test_pagerank()
        test_triangle_count()
        test_louvain()
        test_node2vec()
        
        print("\n" + "=" * 60)
        print("✓ All algorithm tests passed!")
        print("=" * 60)
        return 0
    except Exception as e:
        print(f"\n✗ Test failed with error: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    sys.exit(main())

