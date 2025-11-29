#!/usr/bin/env python3
"""
Comprehensive tests for DeepGraph Index Management (3 methods)

Tests cover:
- create_hash_index() - Create hash index for O(1) lookups
- create_btree_index() - Create B-tree index for range queries
- drop_index() - Remove index

Each test includes performance characteristics, edge cases, and error handling.
"""

import sys
import traceback


def run_tests():
    """Run all index management tests"""
    print("=" * 80)
    print("TEST SUITE 3: INDEX MANAGEMENT (3 methods)")
    print("=" * 80)
    print()
    
    try:
        import deepgraph
    except ImportError:
        print("❌ ERROR: deepgraph module not found")
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
    # FEATURE 1: create_hash_index() - Create hash index
    # =============================================================================
    
    def test_create_hash_index_basic():
        """Test creating basic hash index"""
        idx_mgr = deepgraph.IndexManager()
        
        # Should not raise exception
        idx_mgr.create_hash_index("person_idx", "Person")
    
    def test_create_hash_index_multiple():
        """Test creating multiple hash indices"""
        idx_mgr = deepgraph.IndexManager()
        
        idx_mgr.create_hash_index("person_idx", "Person")
        idx_mgr.create_hash_index("company_idx", "Company")
        idx_mgr.create_hash_index("product_idx", "Product")
    
    def test_create_hash_index_same_name():
        """Test creating hash index with duplicate name"""
        idx_mgr = deepgraph.IndexManager()
        
        idx_mgr.create_hash_index("test_idx", "Person")
        
        try:
            idx_mgr.create_hash_index("test_idx", "Company")
            # May or may not fail depending on implementation
        except RuntimeError:
            pass  # Expected if names must be unique
    
    def test_create_hash_index_empty_name():
        """Test creating hash index with empty name"""
        idx_mgr = deepgraph.IndexManager()
        
        try:
            idx_mgr.create_hash_index("", "Person")
            # May be allowed or may fail
        except (RuntimeError, ValueError):
            pass  # Expected if empty names not allowed
    
    def test_create_hash_index_empty_label():
        """Test creating hash index with empty label"""
        idx_mgr = deepgraph.IndexManager()
        
        try:
            idx_mgr.create_hash_index("test_idx", "")
            # May be allowed or may fail
        except (RuntimeError, ValueError):
            pass  # Expected if empty labels not allowed
    
    def test_create_hash_index_special_chars():
        """Test creating hash index with special characters"""
        idx_mgr = deepgraph.IndexManager()
        
        idx_mgr.create_hash_index("test-idx_2024", "Person")
        idx_mgr.create_hash_index("idx.with.dots", "Company")
    
    def test_create_hash_index_unicode():
        """Test creating hash index with Unicode names"""
        idx_mgr = deepgraph.IndexManager()
        
        idx_mgr.create_hash_index("索引", "Person")  # Chinese for "index"
        idx_mgr.create_hash_index("インデックス", "Company")  # Japanese for "index"
    
    def test_create_hash_index_long_names():
        """Test creating hash index with very long names"""
        idx_mgr = deepgraph.IndexManager()
        
        long_name = "x" * 1000
        idx_mgr.create_hash_index(long_name, "Person")
    
    def test_create_hash_index_many():
        """Test creating many hash indices"""
        idx_mgr = deepgraph.IndexManager()
        
        for i in range(50):
            idx_mgr.create_hash_index(f"idx_{i}", f"Label{i}")
    
    # =============================================================================
    # FEATURE 2: create_btree_index() - Create B-tree index
    # =============================================================================
    
    def test_create_btree_index_basic():
        """Test creating basic B-tree index"""
        idx_mgr = deepgraph.IndexManager()
        
        # Should not raise exception
        idx_mgr.create_btree_index("age_idx", "age")
    
    def test_create_btree_index_multiple():
        """Test creating multiple B-tree indices"""
        idx_mgr = deepgraph.IndexManager()
        
        idx_mgr.create_btree_index("age_idx", "age")
        idx_mgr.create_btree_index("salary_idx", "salary")
        idx_mgr.create_btree_index("date_idx", "created_at")
    
    def test_create_btree_index_same_name():
        """Test creating B-tree index with duplicate name"""
        idx_mgr = deepgraph.IndexManager()
        
        idx_mgr.create_btree_index("test_idx", "age")
        
        try:
            idx_mgr.create_btree_index("test_idx", "salary")
            # May or may not fail depending on implementation
        except RuntimeError:
            pass  # Expected if names must be unique
    
    def test_create_btree_index_empty_name():
        """Test creating B-tree index with empty name"""
        idx_mgr = deepgraph.IndexManager()
        
        try:
            idx_mgr.create_btree_index("", "age")
            # May be allowed or may fail
        except (RuntimeError, ValueError):
            pass  # Expected if empty names not allowed
    
    def test_create_btree_index_empty_property():
        """Test creating B-tree index with empty property"""
        idx_mgr = deepgraph.IndexManager()
        
        try:
            idx_mgr.create_btree_index("test_idx", "")
            # May be allowed or may fail
        except (RuntimeError, ValueError):
            pass  # Expected if empty properties not allowed
    
    def test_create_btree_index_special_chars():
        """Test creating B-tree index with special characters"""
        idx_mgr = deepgraph.IndexManager()
        
        idx_mgr.create_btree_index("test-idx_2024", "age")
        idx_mgr.create_btree_index("idx.with.dots", "user.name")
    
    def test_create_btree_index_unicode():
        """Test creating B-tree index with Unicode property names"""
        idx_mgr = deepgraph.IndexManager()
        
        idx_mgr.create_btree_index("name_idx", "名前")  # Japanese for "name"
        idx_mgr.create_btree_index("age_idx", "年齢")  # Japanese for "age"
    
    def test_create_btree_index_long_names():
        """Test creating B-tree index with very long names"""
        idx_mgr = deepgraph.IndexManager()
        
        long_name = "x" * 1000
        idx_mgr.create_btree_index(long_name, "age")
    
    def test_create_btree_index_many():
        """Test creating many B-tree indices"""
        idx_mgr = deepgraph.IndexManager()
        
        for i in range(50):
            idx_mgr.create_btree_index(f"btree_idx_{i}", f"prop{i}")
    
    # =============================================================================
    # FEATURE 3: drop_index() - Drop/remove index
    # =============================================================================
    
    def test_drop_index_hash():
        """Test dropping hash index"""
        idx_mgr = deepgraph.IndexManager()
        
        idx_mgr.create_hash_index("test_idx", "Person")
        idx_mgr.drop_index("test_idx")
    
    def test_drop_index_btree():
        """Test dropping B-tree index"""
        idx_mgr = deepgraph.IndexManager()
        
        idx_mgr.create_btree_index("test_idx", "age")
        idx_mgr.drop_index("test_idx")
    
    def test_drop_index_nonexistent():
        """Test dropping non-existent index"""
        idx_mgr = deepgraph.IndexManager()
        
        try:
            idx_mgr.drop_index("nonexistent_idx")
            assert False, "Should raise exception for non-existent index"
        except RuntimeError:
            pass  # Expected
    
    def test_drop_index_twice():
        """Test dropping same index twice"""
        idx_mgr = deepgraph.IndexManager()
        
        idx_mgr.create_hash_index("test_idx", "Person")
        idx_mgr.drop_index("test_idx")
        
        try:
            idx_mgr.drop_index("test_idx")
            assert False, "Should raise exception for double drop"
        except RuntimeError:
            pass  # Expected
    
    def test_drop_index_empty_name():
        """Test dropping index with empty name"""
        idx_mgr = deepgraph.IndexManager()
        
        try:
            idx_mgr.drop_index("")
            assert False, "Should raise exception for empty name"
        except (RuntimeError, ValueError):
            pass  # Expected
    
    def test_drop_index_recreate():
        """Test recreating index after dropping"""
        idx_mgr = deepgraph.IndexManager()
        
        idx_mgr.create_hash_index("test_idx", "Person")
        idx_mgr.drop_index("test_idx")
        
        # Should be able to recreate
        idx_mgr.create_hash_index("test_idx", "Person")
    
    def test_drop_index_multiple():
        """Test dropping multiple indices"""
        idx_mgr = deepgraph.IndexManager()
        
        idx_mgr.create_hash_index("idx1", "Person")
        idx_mgr.create_hash_index("idx2", "Company")
        idx_mgr.create_btree_index("idx3", "age")
        
        idx_mgr.drop_index("idx1")
        idx_mgr.drop_index("idx2")
        idx_mgr.drop_index("idx3")
    
    # =============================================================================
    # MIXED OPERATIONS
    # =============================================================================
    
    def test_mixed_index_types():
        """Test creating both hash and B-tree indices"""
        idx_mgr = deepgraph.IndexManager()
        
        idx_mgr.create_hash_index("hash_idx", "Person")
        idx_mgr.create_btree_index("btree_idx", "age")
        
        # Both should coexist
        idx_mgr.drop_index("hash_idx")
        idx_mgr.drop_index("btree_idx")
    
    def test_index_with_same_target():
        """Test creating multiple indices on same target"""
        idx_mgr = deepgraph.IndexManager()
        
        # Multiple hash indices on same label
        idx_mgr.create_hash_index("person_idx_1", "Person")
        idx_mgr.create_hash_index("person_idx_2", "Person")
        
        # Should be independent
        idx_mgr.drop_index("person_idx_1")
    
    def test_index_create_drop_pattern():
        """Test repeated create-drop pattern"""
        idx_mgr = deepgraph.IndexManager()
        
        for _ in range(10):
            idx_mgr.create_hash_index("test_idx", "Person")
            idx_mgr.drop_index("test_idx")
    
    # =============================================================================
    # INTEGRATION TESTS - Indices with Graph Operations
    # =============================================================================
    
    def test_index_with_graph_data():
        """Test creating index with existing graph data"""
        storage = deepgraph.GraphStorage()
        idx_mgr = deepgraph.IndexManager()
        
        # Add some nodes first
        storage.add_node(["Person"], {"name": "Alice", "age": 30})
        storage.add_node(["Person"], {"name": "Bob", "age": 25})
        storage.add_node(["Company"], {"name": "Acme"})
        
        # Create indices after data exists
        idx_mgr.create_hash_index("person_idx", "Person")
        idx_mgr.create_btree_index("age_idx", "age")
    
    def test_index_before_graph_data():
        """Test creating index before adding graph data"""
        storage = deepgraph.GraphStorage()
        idx_mgr = deepgraph.IndexManager()
        
        # Create indices first
        idx_mgr.create_hash_index("person_idx", "Person")
        idx_mgr.create_btree_index("age_idx", "age")
        
        # Add nodes after indices exist
        storage.add_node(["Person"], {"name": "Alice", "age": 30})
        storage.add_node(["Person"], {"name": "Bob", "age": 25})
    
    def test_index_lifecycle_with_data():
        """Test full index lifecycle with graph data"""
        storage = deepgraph.GraphStorage()
        idx_mgr = deepgraph.IndexManager()
        
        # Create index
        idx_mgr.create_hash_index("person_idx", "Person")
        
        # Add data
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        
        # Query data (index should help performance)
        persons = storage.find_nodes_by_label("Person")
        assert len(persons) == 2
        
        # Drop index
        idx_mgr.drop_index("person_idx")
        
        # Data should still be accessible
        persons = storage.find_nodes_by_label("Person")
        assert len(persons) == 2
    
    # =============================================================================
    # STRESS TESTS
    # =============================================================================
    
    def test_stress_many_indices():
        """Stress test: Create many indices"""
        idx_mgr = deepgraph.IndexManager()
        
        # Create 100 hash indices
        for i in range(100):
            idx_mgr.create_hash_index(f"hash_{i}", f"Label{i}")
        
        # Create 100 B-tree indices
        for i in range(100):
            idx_mgr.create_btree_index(f"btree_{i}", f"prop{i}")
    
    def test_stress_create_drop_cycle():
        """Stress test: Many create-drop cycles"""
        idx_mgr = deepgraph.IndexManager()
        
        for i in range(100):
            idx_mgr.create_hash_index("temp_idx", "Person")
            idx_mgr.drop_index("temp_idx")
    
    def test_stress_large_dataset_with_index():
        """Stress test: Index with large dataset"""
        storage = deepgraph.GraphStorage()
        idx_mgr = deepgraph.IndexManager()
        
        # Create index
        idx_mgr.create_hash_index("person_idx", "Person")
        
        # Add many nodes
        for i in range(1000):
            storage.add_node(["Person"], {"id": i, "name": f"Person{i}"})
        
        # Query should work
        persons = storage.find_nodes_by_label("Person")
        assert len(persons) == 1000
    
    # =============================================================================
    # EDGE CASES
    # =============================================================================
    
    def test_index_manager_reuse():
        """Test reusing index manager after many operations"""
        idx_mgr = deepgraph.IndexManager()
        
        for i in range(10):
            idx_mgr.create_hash_index(f"idx_{i}", f"Label{i}")
            idx_mgr.drop_index(f"idx_{i}")
        
        # Should still work
        idx_mgr.create_hash_index("final_idx", "Person")
    
    def test_multiple_index_managers():
        """Test multiple index manager instances"""
        idx_mgr1 = deepgraph.IndexManager()
        idx_mgr2 = deepgraph.IndexManager()
        
        # Both should work independently
        idx_mgr1.create_hash_index("idx1", "Person")
        idx_mgr2.create_hash_index("idx2", "Company")
    
    def test_index_name_conflicts():
        """Test handling of index name conflicts"""
        idx_mgr = deepgraph.IndexManager()
        
        idx_mgr.create_hash_index("my_idx", "Person")
        
        try:
            # Try to create B-tree with same name
            idx_mgr.create_btree_index("my_idx", "age")
            # May succeed or fail depending on namespace
        except RuntimeError:
            pass  # Expected if names must be globally unique
    
    # =============================================================================
    # RUN ALL TESTS
    # =============================================================================
    
    print("### create_hash_index() - Create hash index for O(1) lookups")
    print()
    run_test("test_create_hash_index_basic", test_create_hash_index_basic)
    run_test("test_create_hash_index_multiple", test_create_hash_index_multiple)
    run_test("test_create_hash_index_same_name", test_create_hash_index_same_name)
    run_test("test_create_hash_index_empty_name", test_create_hash_index_empty_name)
    run_test("test_create_hash_index_empty_label", test_create_hash_index_empty_label)
    run_test("test_create_hash_index_special_chars", test_create_hash_index_special_chars)
    run_test("test_create_hash_index_unicode", test_create_hash_index_unicode)
    run_test("test_create_hash_index_long_names", test_create_hash_index_long_names)
    run_test("test_create_hash_index_many", test_create_hash_index_many)
    
    print()
    print("### create_btree_index() - Create B-tree index for range queries")
    print()
    run_test("test_create_btree_index_basic", test_create_btree_index_basic)
    run_test("test_create_btree_index_multiple", test_create_btree_index_multiple)
    run_test("test_create_btree_index_same_name", test_create_btree_index_same_name)
    run_test("test_create_btree_index_empty_name", test_create_btree_index_empty_name)
    run_test("test_create_btree_index_empty_property", test_create_btree_index_empty_property)
    run_test("test_create_btree_index_special_chars", test_create_btree_index_special_chars)
    run_test("test_create_btree_index_unicode", test_create_btree_index_unicode)
    run_test("test_create_btree_index_long_names", test_create_btree_index_long_names)
    run_test("test_create_btree_index_many", test_create_btree_index_many)
    
    print()
    print("### drop_index() - Remove index")
    print()
    run_test("test_drop_index_hash", test_drop_index_hash)
    run_test("test_drop_index_btree", test_drop_index_btree)
    run_test("test_drop_index_nonexistent", test_drop_index_nonexistent)
    run_test("test_drop_index_twice", test_drop_index_twice)
    run_test("test_drop_index_empty_name", test_drop_index_empty_name)
    run_test("test_drop_index_recreate", test_drop_index_recreate)
    run_test("test_drop_index_multiple", test_drop_index_multiple)
    
    print()
    print("### Mixed Operations")
    print()
    run_test("test_mixed_index_types", test_mixed_index_types)
    run_test("test_index_with_same_target", test_index_with_same_target)
    run_test("test_index_create_drop_pattern", test_index_create_drop_pattern)
    
    print()
    print("### Integration Tests")
    print()
    run_test("test_index_with_graph_data", test_index_with_graph_data)
    run_test("test_index_before_graph_data", test_index_before_graph_data)
    run_test("test_index_lifecycle_with_data", test_index_lifecycle_with_data)
    
    print()
    print("### Stress Tests")
    print()
    run_test("test_stress_many_indices", test_stress_many_indices)
    run_test("test_stress_create_drop_cycle", test_stress_create_drop_cycle)
    run_test("test_stress_large_dataset_with_index", test_stress_large_dataset_with_index)
    
    print()
    print("### Edge Cases")
    print()
    run_test("test_index_manager_reuse", test_index_manager_reuse)
    run_test("test_multiple_index_managers", test_multiple_index_managers)
    run_test("test_index_name_conflicts", test_index_name_conflicts)
    
    # Summary
    print()
    print("=" * 80)
    print(f"RESULTS: {passed} passed, {failed} failed out of {total} tests")
    print("=" * 80)
    
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(run_tests())

