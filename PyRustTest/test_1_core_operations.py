#!/usr/bin/env python3
"""
Comprehensive tests for DeepGraph Core Storage Operations (20 methods)

Tests cover:
- CRUD operations (8 methods)
- Graph traversal (2 methods)
- Advanced queries (3 methods)
- Bulk operations (5 methods)
- Statistics (2 methods)

Each test includes edge cases, error handling, and corner scenarios.
"""

import sys
import traceback


def run_tests():
    """Run all core operation tests"""
    print("=" * 80)
    print("TEST SUITE 1: CORE OPERATIONS (20 methods)")
    print("=" * 80)
    print()
    
    try:
        import deepgraph
    except ImportError:
        print("❌ ERROR: deepgraph module not found")
        print("   Please install: maturin develop --release --features python")
        return 1
    
    passed = 0
    failed = 0
    total = 0
    
    def run_test(test_name, test_func):
        nonlocal passed, failed, total
        total += 1
        try:
            test_func()
            print(f"✅ {test_name}")
            passed += 1
        except AssertionError as e:
            print(f"❌ {test_name}")
            print(f"   Assertion failed: {e}")
            failed += 1
        except Exception as e:
            print(f"❌ {test_name}")
            print(f"   Exception: {e}")
            traceback.print_exc()
            failed += 1
    
    # =============================================================================
    # FEATURE 1: add_node() - Create nodes with labels and properties
    # =============================================================================
    
    def test_add_node_basic():
        """Test basic node creation with single label"""
        storage = deepgraph.GraphStorage()
        node_id = storage.add_node(["Person"], {"name": "Alice", "age": 30})
        assert node_id is not None
        assert isinstance(node_id, str)
        assert len(node_id) > 0
    
    def test_add_node_multiple_labels():
        """Test node with multiple labels"""
        storage = deepgraph.GraphStorage()
        node_id = storage.add_node(["Person", "Engineer", "Manager"], {"name": "Bob"})
        assert node_id is not None
    
    def test_add_node_no_labels():
        """Test node with empty labels list"""
        storage = deepgraph.GraphStorage()
        node_id = storage.add_node([], {"name": "Charlie"})
        assert node_id is not None
    
    def test_add_node_no_properties():
        """Test node with no properties"""
        storage = deepgraph.GraphStorage()
        node_id = storage.add_node(["Person"], {})
        assert node_id is not None
    
    def test_add_node_empty():
        """Test node with no labels and no properties"""
        storage = deepgraph.GraphStorage()
        node_id = storage.add_node([], {})
        assert node_id is not None
    
    def test_add_node_all_property_types():
        """Test node with all supported property types"""
        storage = deepgraph.GraphStorage()
        node_id = storage.add_node(["Test"], {
            "string": "hello",
            "int": 42,
            "float": 3.14,
            "bool_true": True,
            "bool_false": False,
            "none": None,
        })
        assert node_id is not None
    
    def test_add_node_unicode_properties():
        """Test node with Unicode characters"""
        storage = deepgraph.GraphStorage()
        node_id = storage.add_node(["Person"], {
            "name": "Müller",
            "city": "北京",
            "emoji": "🎉"
        })
        assert node_id is not None
    
    def test_add_node_large_properties():
        """Test node with large property values"""
        storage = deepgraph.GraphStorage()
        node_id = storage.add_node(["Test"], {
            "large_string": "x" * 10000,
            "large_int": 9999999999999999,
            "large_float": 1.7976931348623157e+308  # Near max float
        })
        assert node_id is not None
    
    def test_add_node_many_properties():
        """Test node with many properties"""
        storage = deepgraph.GraphStorage()
        props = {f"prop_{i}": i for i in range(100)}
        node_id = storage.add_node(["Test"], props)
        assert node_id is not None
    
    def test_add_node_special_characters():
        """Test node with special characters in property keys"""
        storage = deepgraph.GraphStorage()
        node_id = storage.add_node(["Test"], {
            "key-with-dash": "value1",
            "key_with_underscore": "value2",
            "key.with.dot": "value3",
        })
        assert node_id is not None
    
    # =============================================================================
    # FEATURE 2: get_node() - Retrieve node by ID
    # =============================================================================
    
    def test_get_node_basic():
        """Test retrieving an existing node"""
        storage = deepgraph.GraphStorage()
        node_id = storage.add_node(["Person"], {"name": "Alice"})
        node = storage.get_node(node_id)
        assert node is not None
    
    def test_get_node_invalid_id():
        """Test retrieving non-existent node"""
        storage = deepgraph.GraphStorage()
        try:
            storage.get_node("00000000-0000-0000-0000-000000000000")
            assert False, "Should raise exception for invalid ID"
        except RuntimeError:
            pass  # Expected
    
    def test_get_node_empty_string():
        """Test get_node with empty string ID"""
        storage = deepgraph.GraphStorage()
        try:
            storage.get_node("")
            assert False, "Should raise exception for empty ID"
        except (RuntimeError, ValueError):
            pass  # Expected
    
    def test_get_node_after_delete():
        """Test retrieving deleted node"""
        storage = deepgraph.GraphStorage()
        node_id = storage.add_node(["Person"], {"name": "Alice"})
        storage.delete_node(node_id)
        try:
            storage.get_node(node_id)
            assert False, "Should raise exception for deleted node"
        except RuntimeError:
            pass  # Expected
    
    # =============================================================================
    # FEATURE 3: update_node() - Update node properties
    # =============================================================================
    
    def test_update_node_basic():
        """Test basic node property update"""
        storage = deepgraph.GraphStorage()
        node_id = storage.add_node(["Person"], {"name": "Alice", "age": 30})
        storage.update_node(node_id, {"age": 31, "city": "NYC"})
        node = storage.get_node(node_id)
        assert node is not None
    
    def test_update_node_empty_properties():
        """Test updating node with empty properties"""
        storage = deepgraph.GraphStorage()
        node_id = storage.add_node(["Person"], {"name": "Bob"})
        storage.update_node(node_id, {})  # Should not fail
    
    def test_update_node_invalid_id():
        """Test updating non-existent node"""
        storage = deepgraph.GraphStorage()
        try:
            storage.update_node("00000000-0000-0000-0000-000000000000", {"name": "Test"})
            assert False, "Should raise exception for invalid ID"
        except RuntimeError:
            pass  # Expected
    
    def test_update_node_overwrite_all():
        """Test overwriting all properties"""
        storage = deepgraph.GraphStorage()
        node_id = storage.add_node(["Person"], {"name": "Alice", "age": 30})
        storage.update_node(node_id, {"occupation": "Engineer"})
        # Note: Behavior depends on implementation (merge vs replace)
    
    # =============================================================================
    # FEATURE 4: delete_node() - Delete node and connected edges
    # =============================================================================
    
    def test_delete_node_basic():
        """Test basic node deletion"""
        storage = deepgraph.GraphStorage()
        node_id = storage.add_node(["Person"], {"name": "Alice"})
        storage.delete_node(node_id)
        try:
            storage.get_node(node_id)
            assert False, "Node should be deleted"
        except RuntimeError:
            pass  # Expected
    
    def test_delete_node_with_edges():
        """Test deleting node with connected edges"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        edge = storage.add_edge(node1, node2, "KNOWS", {})
        
        storage.delete_node(node1)
        # Edge should also be deleted
        try:
            storage.get_edge(edge)
            assert False, "Edge should be deleted with node"
        except RuntimeError:
            pass  # Expected
    
    def test_delete_node_invalid_id():
        """Test deleting non-existent node"""
        storage = deepgraph.GraphStorage()
        try:
            storage.delete_node("00000000-0000-0000-0000-000000000000")
            assert False, "Should raise exception for invalid ID"
        except RuntimeError:
            pass  # Expected
    
    def test_delete_node_twice():
        """Test deleting same node twice"""
        storage = deepgraph.GraphStorage()
        node_id = storage.add_node(["Person"], {"name": "Alice"})
        storage.delete_node(node_id)
        try:
            storage.delete_node(node_id)
            assert False, "Should raise exception for double delete"
        except RuntimeError:
            pass  # Expected
    
    # =============================================================================
    # FEATURE 5: add_edge() - Create edge between nodes
    # =============================================================================
    
    def test_add_edge_basic():
        """Test basic edge creation"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        edge_id = storage.add_edge(node1, node2, "KNOWS", {"since": 2020})
        assert edge_id is not None
        assert isinstance(edge_id, str)
    
    def test_add_edge_no_properties():
        """Test edge with no properties"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        edge_id = storage.add_edge(node1, node2, "KNOWS", {})
        assert edge_id is not None
    
    def test_add_edge_self_loop():
        """Test edge from node to itself"""
        storage = deepgraph.GraphStorage()
        node = storage.add_node(["Person"], {"name": "Alice"})
        edge_id = storage.add_edge(node, node, "LIKES", {})
        assert edge_id is not None
    
    def test_add_edge_multiple_between_same_nodes():
        """Test multiple edges between same pair of nodes"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        edge1 = storage.add_edge(node1, node2, "KNOWS", {})
        edge2 = storage.add_edge(node1, node2, "WORKS_WITH", {})
        assert edge1 != edge2
    
    def test_add_edge_invalid_from_node():
        """Test edge with invalid source node"""
        storage = deepgraph.GraphStorage()
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        try:
            storage.add_edge("00000000-0000-0000-0000-000000000000", node2, "KNOWS", {})
            assert False, "Should raise exception for invalid source node"
        except RuntimeError:
            pass  # Expected
    
    def test_add_edge_invalid_to_node():
        """Test edge with invalid target node"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        try:
            storage.add_edge(node1, "00000000-0000-0000-0000-000000000000", "KNOWS", {})
            assert False, "Should raise exception for invalid target node"
        except RuntimeError:
            pass  # Expected
    
    def test_add_edge_unicode_type():
        """Test edge with Unicode relationship type"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        edge_id = storage.add_edge(node1, node2, "认识", {})  # "knows" in Chinese
        assert edge_id is not None
    
    # =============================================================================
    # FEATURE 6: get_edge() - Retrieve edge by ID
    # =============================================================================
    
    def test_get_edge_basic():
        """Test retrieving an existing edge"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        edge_id = storage.add_edge(node1, node2, "KNOWS", {"since": 2020})
        edge = storage.get_edge(edge_id)
        assert edge is not None
    
    def test_get_edge_invalid_id():
        """Test retrieving non-existent edge"""
        storage = deepgraph.GraphStorage()
        try:
            storage.get_edge("00000000-0000-0000-0000-000000000000")
            assert False, "Should raise exception for invalid ID"
        except RuntimeError:
            pass  # Expected
    
    def test_get_edge_after_delete():
        """Test retrieving deleted edge"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        edge_id = storage.add_edge(node1, node2, "KNOWS", {})
        storage.delete_edge(edge_id)
        try:
            storage.get_edge(edge_id)
            assert False, "Should raise exception for deleted edge"
        except RuntimeError:
            pass  # Expected
    
    # =============================================================================
    # FEATURE 7: update_edge() - Update edge properties
    # =============================================================================
    
    def test_update_edge_basic():
        """Test basic edge property update"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        edge_id = storage.add_edge(node1, node2, "KNOWS", {"since": 2020})
        storage.update_edge(edge_id, {"since": 2021, "strength": "strong"})
    
    def test_update_edge_empty_properties():
        """Test updating edge with empty properties"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        edge_id = storage.add_edge(node1, node2, "KNOWS", {"since": 2020})
        storage.update_edge(edge_id, {})  # Should not fail
    
    def test_update_edge_invalid_id():
        """Test updating non-existent edge"""
        storage = deepgraph.GraphStorage()
        try:
            storage.update_edge("00000000-0000-0000-0000-000000000000", {"test": "value"})
            assert False, "Should raise exception for invalid ID"
        except RuntimeError:
            pass  # Expected
    
    # =============================================================================
    # FEATURE 8: delete_edge() - Delete edge
    # =============================================================================
    
    def test_delete_edge_basic():
        """Test basic edge deletion"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        edge_id = storage.add_edge(node1, node2, "KNOWS", {})
        storage.delete_edge(edge_id)
        try:
            storage.get_edge(edge_id)
            assert False, "Edge should be deleted"
        except RuntimeError:
            pass  # Expected
    
    def test_delete_edge_nodes_remain():
        """Test that deleting edge doesn't delete nodes"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        edge_id = storage.add_edge(node1, node2, "KNOWS", {})
        storage.delete_edge(edge_id)
        
        # Nodes should still exist
        assert storage.get_node(node1) is not None
        assert storage.get_node(node2) is not None
    
    def test_delete_edge_invalid_id():
        """Test deleting non-existent edge"""
        storage = deepgraph.GraphStorage()
        try:
            storage.delete_edge("00000000-0000-0000-0000-000000000000")
            assert False, "Should raise exception for invalid ID"
        except RuntimeError:
            pass  # Expected
    
    # =============================================================================
    # FEATURE 9: get_outgoing_edges() - Get edges from a node
    # =============================================================================
    
    def test_get_outgoing_edges_basic():
        """Test getting outgoing edges"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        node3 = storage.add_node(["Person"], {"name": "Charlie"})
        
        storage.add_edge(node1, node2, "KNOWS", {})
        storage.add_edge(node1, node3, "KNOWS", {})
        
        edges = storage.get_outgoing_edges(node1)
        assert len(edges) == 2
    
    def test_get_outgoing_edges_none():
        """Test node with no outgoing edges"""
        storage = deepgraph.GraphStorage()
        node = storage.add_node(["Person"], {"name": "Alice"})
        edges = storage.get_outgoing_edges(node)
        assert len(edges) == 0
    
    def test_get_outgoing_edges_self_loop():
        """Test outgoing edges with self-loop"""
        storage = deepgraph.GraphStorage()
        node = storage.add_node(["Person"], {"name": "Alice"})
        storage.add_edge(node, node, "LIKES", {})
        edges = storage.get_outgoing_edges(node)
        assert len(edges) == 1
    
    def test_get_outgoing_edges_invalid_node():
        """Test getting edges from non-existent node"""
        storage = deepgraph.GraphStorage()
        try:
            storage.get_outgoing_edges("00000000-0000-0000-0000-000000000000")
            assert False, "Should raise exception for invalid node"
        except RuntimeError:
            pass  # Expected
    
    # =============================================================================
    # FEATURE 10: get_incoming_edges() - Get edges to a node
    # =============================================================================
    
    def test_get_incoming_edges_basic():
        """Test getting incoming edges"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        node3 = storage.add_node(["Person"], {"name": "Charlie"})
        
        storage.add_edge(node2, node1, "KNOWS", {})
        storage.add_edge(node3, node1, "KNOWS", {})
        
        edges = storage.get_incoming_edges(node1)
        assert len(edges) == 2
    
    def test_get_incoming_edges_none():
        """Test node with no incoming edges"""
        storage = deepgraph.GraphStorage()
        node = storage.add_node(["Person"], {"name": "Alice"})
        edges = storage.get_incoming_edges(node)
        assert len(edges) == 0
    
    def test_get_incoming_edges_self_loop():
        """Test incoming edges with self-loop"""
        storage = deepgraph.GraphStorage()
        node = storage.add_node(["Person"], {"name": "Alice"})
        storage.add_edge(node, node, "LIKES", {})
        edges = storage.get_incoming_edges(node)
        assert len(edges) == 1
    
    def test_get_incoming_edges_invalid_node():
        """Test getting edges to non-existent node"""
        storage = deepgraph.GraphStorage()
        try:
            storage.get_incoming_edges("00000000-0000-0000-0000-000000000000")
            assert False, "Should raise exception for invalid node"
        except RuntimeError:
            pass  # Expected
    
    # =============================================================================
    # FEATURE 11: find_nodes_by_label() - Find nodes by label
    # =============================================================================
    
    def test_find_nodes_by_label_basic():
        """Test finding nodes by label"""
        storage = deepgraph.GraphStorage()
        storage.add_node(["Person"], {"name": "Alice"})
        storage.add_node(["Person"], {"name": "Bob"})
        storage.add_node(["Company"], {"name": "Acme"})
        
        persons = storage.find_nodes_by_label("Person")
        assert len(persons) == 2
    
    def test_find_nodes_by_label_none():
        """Test finding nodes with non-existent label"""
        storage = deepgraph.GraphStorage()
        storage.add_node(["Person"], {"name": "Alice"})
        nodes = storage.find_nodes_by_label("NonExistent")
        assert len(nodes) == 0
    
    def test_find_nodes_by_label_empty_graph():
        """Test finding nodes in empty graph"""
        storage = deepgraph.GraphStorage()
        nodes = storage.find_nodes_by_label("Person")
        assert len(nodes) == 0
    
    def test_find_nodes_by_label_multiple_labels():
        """Test finding nodes that have multiple labels"""
        storage = deepgraph.GraphStorage()
        storage.add_node(["Person", "Engineer"], {"name": "Alice"})
        storage.add_node(["Person", "Manager"], {"name": "Bob"})
        
        persons = storage.find_nodes_by_label("Person")
        assert len(persons) == 2
        
        engineers = storage.find_nodes_by_label("Engineer")
        assert len(engineers) == 1
    
    # =============================================================================
    # FEATURE 12: find_nodes_by_property() - Find nodes by property
    # =============================================================================
    
    def test_find_nodes_by_property_basic():
        """Test finding nodes by property key-value"""
        storage = deepgraph.GraphStorage()
        storage.add_node(["Person"], {"name": "Alice", "age": 30})
        storage.add_node(["Person"], {"name": "Bob", "age": 30})
        storage.add_node(["Person"], {"name": "Charlie", "age": 25})
        
        nodes_age_30 = storage.find_nodes_by_property("age", 30)
        assert len(nodes_age_30) == 2
    
    def test_find_nodes_by_property_none():
        """Test finding nodes with non-existent property"""
        storage = deepgraph.GraphStorage()
        storage.add_node(["Person"], {"name": "Alice"})
        nodes = storage.find_nodes_by_property("nonexistent", "value")
        assert len(nodes) == 0
    
    def test_find_nodes_by_property_string():
        """Test finding by string property"""
        storage = deepgraph.GraphStorage()
        storage.add_node(["Person"], {"name": "Alice"})
        storage.add_node(["Person"], {"name": "Bob"})
        
        nodes = storage.find_nodes_by_property("name", "Alice")
        assert len(nodes) == 1
    
    def test_find_nodes_by_property_int():
        """Test finding by integer property"""
        storage = deepgraph.GraphStorage()
        storage.add_node(["Person"], {"age": 30})
        storage.add_node(["Person"], {"age": 25})
        
        nodes = storage.find_nodes_by_property("age", 30)
        assert len(nodes) == 1
    
    def test_find_nodes_by_property_float():
        """Test finding by float property"""
        storage = deepgraph.GraphStorage()
        storage.add_node(["Product"], {"price": 19.99})
        storage.add_node(["Product"], {"price": 29.99})
        
        nodes = storage.find_nodes_by_property("price", 19.99)
        assert len(nodes) == 1
    
    def test_find_nodes_by_property_bool():
        """Test finding by boolean property"""
        storage = deepgraph.GraphStorage()
        storage.add_node(["User"], {"active": True})
        storage.add_node(["User"], {"active": False})
        
        active_users = storage.find_nodes_by_property("active", True)
        assert len(active_users) == 1
    
    # =============================================================================
    # FEATURE 13: find_edges_by_type() - Find edges by relationship type
    # =============================================================================
    
    def test_find_edges_by_type_basic():
        """Test finding edges by type"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        node3 = storage.add_node(["Person"], {"name": "Charlie"})
        
        storage.add_edge(node1, node2, "KNOWS", {})
        storage.add_edge(node1, node3, "KNOWS", {})
        storage.add_edge(node2, node3, "WORKS_WITH", {})
        
        knows_edges = storage.find_edges_by_type("KNOWS")
        assert len(knows_edges) == 2
        
        works_edges = storage.find_edges_by_type("WORKS_WITH")
        assert len(works_edges) == 1
    
    def test_find_edges_by_type_none():
        """Test finding edges with non-existent type"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        storage.add_edge(node1, node2, "KNOWS", {})
        
        edges = storage.find_edges_by_type("NONEXISTENT")
        assert len(edges) == 0
    
    def test_find_edges_by_type_empty_graph():
        """Test finding edges in empty graph"""
        storage = deepgraph.GraphStorage()
        edges = storage.find_edges_by_type("KNOWS")
        assert len(edges) == 0
    
    # =============================================================================
    # FEATURE 14: get_all_nodes() - Get all nodes in the graph
    # =============================================================================
    
    def test_get_all_nodes_basic():
        """Test getting all nodes"""
        storage = deepgraph.GraphStorage()
        storage.add_node(["Person"], {"name": "Alice"})
        storage.add_node(["Person"], {"name": "Bob"})
        storage.add_node(["Company"], {"name": "Acme"})
        
        all_nodes = storage.get_all_nodes()
        assert len(all_nodes) == 3
    
    def test_get_all_nodes_empty():
        """Test getting nodes from empty graph"""
        storage = deepgraph.GraphStorage()
        all_nodes = storage.get_all_nodes()
        assert len(all_nodes) == 0
    
    def test_get_all_nodes_after_delete():
        """Test getting nodes after deletion"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        storage.delete_node(node1)
        
        all_nodes = storage.get_all_nodes()
        assert len(all_nodes) == 1
    
    # =============================================================================
    # FEATURE 15: get_all_edges() - Get all edges in the graph
    # =============================================================================
    
    def test_get_all_edges_basic():
        """Test getting all edges"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        node3 = storage.add_node(["Person"], {"name": "Charlie"})
        
        storage.add_edge(node1, node2, "KNOWS", {})
        storage.add_edge(node2, node3, "KNOWS", {})
        
        all_edges = storage.get_all_edges()
        assert len(all_edges) == 2
    
    def test_get_all_edges_empty():
        """Test getting edges from empty graph"""
        storage = deepgraph.GraphStorage()
        all_edges = storage.get_all_edges()
        assert len(all_edges) == 0
    
    def test_get_all_edges_after_delete():
        """Test getting edges after deletion"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        edge1 = storage.add_edge(node1, node2, "KNOWS", {})
        edge2 = storage.add_edge(node2, node1, "LIKES", {})
        
        storage.delete_edge(edge1)
        
        all_edges = storage.get_all_edges()
        assert len(all_edges) == 1
    
    # =============================================================================
    # FEATURE 16: node_count() - Get count of nodes
    # =============================================================================
    
    def test_node_count_basic():
        """Test node count"""
        storage = deepgraph.GraphStorage()
        assert storage.node_count() == 0
        
        storage.add_node(["Person"], {"name": "Alice"})
        assert storage.node_count() == 1
        
        storage.add_node(["Person"], {"name": "Bob"})
        assert storage.node_count() == 2
    
    def test_node_count_after_delete():
        """Test node count after deletion"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        assert storage.node_count() == 2
        
        storage.delete_node(node1)
        assert storage.node_count() == 1
    
    def test_node_count_large():
        """Test node count with many nodes"""
        storage = deepgraph.GraphStorage()
        for i in range(100):
            storage.add_node(["Test"], {"id": i})
        assert storage.node_count() == 100
    
    # =============================================================================
    # FEATURE 17: edge_count() - Get count of edges
    # =============================================================================
    
    def test_edge_count_basic():
        """Test edge count"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        
        assert storage.edge_count() == 0
        
        storage.add_edge(node1, node2, "KNOWS", {})
        assert storage.edge_count() == 1
    
    def test_edge_count_after_delete():
        """Test edge count after deletion"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        
        edge1 = storage.add_edge(node1, node2, "KNOWS", {})
        edge2 = storage.add_edge(node2, node1, "LIKES", {})
        assert storage.edge_count() == 2
        
        storage.delete_edge(edge1)
        assert storage.edge_count() == 1
    
    # =============================================================================
    # FEATURE 18: clear() - Clear entire graph
    # =============================================================================
    
    def test_clear_basic():
        """Test clearing graph"""
        storage = deepgraph.GraphStorage()
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        storage.add_edge(node1, node2, "KNOWS", {})
        
        storage.clear()
        
        assert storage.node_count() == 0
        assert storage.edge_count() == 0
    
    def test_clear_empty():
        """Test clearing empty graph"""
        storage = deepgraph.GraphStorage()
        storage.clear()  # Should not fail
        assert storage.node_count() == 0
    
    def test_clear_and_reuse():
        """Test using graph after clear"""
        storage = deepgraph.GraphStorage()
        storage.add_node(["Person"], {"name": "Alice"})
        storage.clear()
        
        node = storage.add_node(["Person"], {"name": "Bob"})
        assert node is not None
        assert storage.node_count() == 1
    
    # =============================================================================
    # STRESS TESTS
    # =============================================================================
    
    def test_stress_many_nodes():
        """Stress test: Create many nodes"""
        storage = deepgraph.GraphStorage()
        count = 1000
        for i in range(count):
            storage.add_node(["Test"], {"id": i})
        assert storage.node_count() == count
    
    def test_stress_many_edges():
        """Stress test: Create many edges"""
        storage = deepgraph.GraphStorage()
        nodes = [storage.add_node(["Test"], {"id": i}) for i in range(100)]
        
        edge_count = 0
        for i in range(len(nodes)):
            for j in range(i + 1, min(i + 11, len(nodes))):
                storage.add_edge(nodes[i], nodes[j], "CONNECTS", {})
                edge_count += 1
        
        assert storage.edge_count() == edge_count
    
    def test_stress_deep_traversal():
        """Stress test: Deep graph traversal"""
        storage = deepgraph.GraphStorage()
        nodes = [storage.add_node(["Chain"], {"id": i}) for i in range(100)]
        
        for i in range(len(nodes) - 1):
            storage.add_edge(nodes[i], nodes[i + 1], "NEXT", {})
        
        # Get all outgoing edges from first node (should be 1)
        edges = storage.get_outgoing_edges(nodes[0])
        assert len(edges) == 1
    
    # =============================================================================
    # RUN ALL TESTS
    # =============================================================================
    
    print("### CRUD Operations (8 methods)")
    print()
    run_test("test_add_node_basic", test_add_node_basic)
    run_test("test_add_node_multiple_labels", test_add_node_multiple_labels)
    run_test("test_add_node_no_labels", test_add_node_no_labels)
    run_test("test_add_node_no_properties", test_add_node_no_properties)
    run_test("test_add_node_empty", test_add_node_empty)
    run_test("test_add_node_all_property_types", test_add_node_all_property_types)
    run_test("test_add_node_unicode_properties", test_add_node_unicode_properties)
    run_test("test_add_node_large_properties", test_add_node_large_properties)
    run_test("test_add_node_many_properties", test_add_node_many_properties)
    run_test("test_add_node_special_characters", test_add_node_special_characters)
    
    run_test("test_get_node_basic", test_get_node_basic)
    run_test("test_get_node_invalid_id", test_get_node_invalid_id)
    run_test("test_get_node_empty_string", test_get_node_empty_string)
    run_test("test_get_node_after_delete", test_get_node_after_delete)
    
    run_test("test_update_node_basic", test_update_node_basic)
    run_test("test_update_node_empty_properties", test_update_node_empty_properties)
    run_test("test_update_node_invalid_id", test_update_node_invalid_id)
    run_test("test_update_node_overwrite_all", test_update_node_overwrite_all)
    
    run_test("test_delete_node_basic", test_delete_node_basic)
    run_test("test_delete_node_with_edges", test_delete_node_with_edges)
    run_test("test_delete_node_invalid_id", test_delete_node_invalid_id)
    run_test("test_delete_node_twice", test_delete_node_twice)
    
    run_test("test_add_edge_basic", test_add_edge_basic)
    run_test("test_add_edge_no_properties", test_add_edge_no_properties)
    run_test("test_add_edge_self_loop", test_add_edge_self_loop)
    run_test("test_add_edge_multiple_between_same_nodes", test_add_edge_multiple_between_same_nodes)
    run_test("test_add_edge_invalid_from_node", test_add_edge_invalid_from_node)
    run_test("test_add_edge_invalid_to_node", test_add_edge_invalid_to_node)
    run_test("test_add_edge_unicode_type", test_add_edge_unicode_type)
    
    run_test("test_get_edge_basic", test_get_edge_basic)
    run_test("test_get_edge_invalid_id", test_get_edge_invalid_id)
    run_test("test_get_edge_after_delete", test_get_edge_after_delete)
    
    run_test("test_update_edge_basic", test_update_edge_basic)
    run_test("test_update_edge_empty_properties", test_update_edge_empty_properties)
    run_test("test_update_edge_invalid_id", test_update_edge_invalid_id)
    
    run_test("test_delete_edge_basic", test_delete_edge_basic)
    run_test("test_delete_edge_nodes_remain", test_delete_edge_nodes_remain)
    run_test("test_delete_edge_invalid_id", test_delete_edge_invalid_id)
    
    print()
    print("### Graph Traversal (2 methods)")
    print()
    run_test("test_get_outgoing_edges_basic", test_get_outgoing_edges_basic)
    run_test("test_get_outgoing_edges_none", test_get_outgoing_edges_none)
    run_test("test_get_outgoing_edges_self_loop", test_get_outgoing_edges_self_loop)
    run_test("test_get_outgoing_edges_invalid_node", test_get_outgoing_edges_invalid_node)
    
    run_test("test_get_incoming_edges_basic", test_get_incoming_edges_basic)
    run_test("test_get_incoming_edges_none", test_get_incoming_edges_none)
    run_test("test_get_incoming_edges_self_loop", test_get_incoming_edges_self_loop)
    run_test("test_get_incoming_edges_invalid_node", test_get_incoming_edges_invalid_node)
    
    print()
    print("### Advanced Queries (3 methods)")
    print()
    run_test("test_find_nodes_by_label_basic", test_find_nodes_by_label_basic)
    run_test("test_find_nodes_by_label_none", test_find_nodes_by_label_none)
    run_test("test_find_nodes_by_label_empty_graph", test_find_nodes_by_label_empty_graph)
    run_test("test_find_nodes_by_label_multiple_labels", test_find_nodes_by_label_multiple_labels)
    
    run_test("test_find_nodes_by_property_basic", test_find_nodes_by_property_basic)
    run_test("test_find_nodes_by_property_none", test_find_nodes_by_property_none)
    run_test("test_find_nodes_by_property_string", test_find_nodes_by_property_string)
    run_test("test_find_nodes_by_property_int", test_find_nodes_by_property_int)
    run_test("test_find_nodes_by_property_float", test_find_nodes_by_property_float)
    run_test("test_find_nodes_by_property_bool", test_find_nodes_by_property_bool)
    
    run_test("test_find_edges_by_type_basic", test_find_edges_by_type_basic)
    run_test("test_find_edges_by_type_none", test_find_edges_by_type_none)
    run_test("test_find_edges_by_type_empty_graph", test_find_edges_by_type_empty_graph)
    
    print()
    print("### Bulk Operations (5 methods)")
    print()
    run_test("test_get_all_nodes_basic", test_get_all_nodes_basic)
    run_test("test_get_all_nodes_empty", test_get_all_nodes_empty)
    run_test("test_get_all_nodes_after_delete", test_get_all_nodes_after_delete)
    
    run_test("test_get_all_edges_basic", test_get_all_edges_basic)
    run_test("test_get_all_edges_empty", test_get_all_edges_empty)
    run_test("test_get_all_edges_after_delete", test_get_all_edges_after_delete)
    
    run_test("test_node_count_basic", test_node_count_basic)
    run_test("test_node_count_after_delete", test_node_count_after_delete)
    run_test("test_node_count_large", test_node_count_large)
    
    run_test("test_edge_count_basic", test_edge_count_basic)
    run_test("test_edge_count_after_delete", test_edge_count_after_delete)
    
    run_test("test_clear_basic", test_clear_basic)
    run_test("test_clear_empty", test_clear_empty)
    run_test("test_clear_and_reuse", test_clear_and_reuse)
    
    print()
    print("### Stress Tests")
    print()
    run_test("test_stress_many_nodes", test_stress_many_nodes)
    run_test("test_stress_many_edges", test_stress_many_edges)
    run_test("test_stress_deep_traversal", test_stress_deep_traversal)
    
    # Summary
    print()
    print("=" * 80)
    print(f"RESULTS: {passed} passed, {failed} failed out of {total} tests")
    print("=" * 80)
    
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(run_tests())

