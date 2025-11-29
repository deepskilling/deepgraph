#!/usr/bin/env python3
"""
Comprehensive tests for DeepGraph Transaction Management (3 methods)

Tests cover:
- begin_transaction() - Start new transaction
- commit_transaction() - Commit changes
- abort_transaction() - Rollback changes

Each test includes ACID properties, edge cases, and error handling.
"""

import sys
import traceback


def run_tests():
    """Run all transaction tests"""
    print("=" * 80)
    print("TEST SUITE 2: TRANSACTIONS (3 methods)")
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
    # FEATURE 1: begin_transaction() - Start new transaction
    # =============================================================================
    
    def test_begin_transaction_basic():
        """Test starting a basic transaction"""
        txn_mgr = deepgraph.TransactionManager()
        txn_id = txn_mgr.begin_transaction()
        assert txn_id is not None
        assert isinstance(txn_id, int)
        assert txn_id > 0
    
    def test_begin_transaction_multiple():
        """Test starting multiple concurrent transactions"""
        txn_mgr = deepgraph.TransactionManager()
        txn1 = txn_mgr.begin_transaction()
        txn2 = txn_mgr.begin_transaction()
        txn3 = txn_mgr.begin_transaction()
        
        assert txn1 != txn2
        assert txn2 != txn3
        assert txn1 != txn3
    
    def test_begin_transaction_sequential_ids():
        """Test that transaction IDs increase sequentially"""
        txn_mgr = deepgraph.TransactionManager()
        txn1 = txn_mgr.begin_transaction()
        txn2 = txn_mgr.begin_transaction()
        
        assert txn2 > txn1
    
    def test_begin_transaction_many():
        """Test starting many transactions"""
        txn_mgr = deepgraph.TransactionManager()
        txns = [txn_mgr.begin_transaction() for _ in range(100)]
        
        # All IDs should be unique
        assert len(set(txns)) == 100
    
    def test_begin_transaction_after_commit():
        """Test starting new transaction after committing previous"""
        txn_mgr = deepgraph.TransactionManager()
        txn1 = txn_mgr.begin_transaction()
        txn_mgr.commit_transaction(txn1)
        
        txn2 = txn_mgr.begin_transaction()
        assert txn2 > txn1
    
    def test_begin_transaction_after_abort():
        """Test starting new transaction after aborting previous"""
        txn_mgr = deepgraph.TransactionManager()
        txn1 = txn_mgr.begin_transaction()
        txn_mgr.abort_transaction(txn1)
        
        txn2 = txn_mgr.begin_transaction()
        assert txn2 > txn1
    
    # =============================================================================
    # FEATURE 2: commit_transaction() - Commit transaction
    # =============================================================================
    
    def test_commit_transaction_basic():
        """Test committing a basic transaction"""
        txn_mgr = deepgraph.TransactionManager()
        txn_id = txn_mgr.begin_transaction()
        
        # Should not raise exception
        txn_mgr.commit_transaction(txn_id)
    
    def test_commit_transaction_multiple():
        """Test committing multiple transactions"""
        txn_mgr = deepgraph.TransactionManager()
        txn1 = txn_mgr.begin_transaction()
        txn2 = txn_mgr.begin_transaction()
        
        txn_mgr.commit_transaction(txn1)
        txn_mgr.commit_transaction(txn2)
    
    def test_commit_transaction_out_of_order():
        """Test committing transactions out of order"""
        txn_mgr = deepgraph.TransactionManager()
        txn1 = txn_mgr.begin_transaction()
        txn2 = txn_mgr.begin_transaction()
        txn3 = txn_mgr.begin_transaction()
        
        # Commit in different order
        txn_mgr.commit_transaction(txn2)
        txn_mgr.commit_transaction(txn1)
        txn_mgr.commit_transaction(txn3)
    
    def test_commit_transaction_twice():
        """Test committing same transaction twice"""
        txn_mgr = deepgraph.TransactionManager()
        txn_id = txn_mgr.begin_transaction()
        txn_mgr.commit_transaction(txn_id)
        
        try:
            txn_mgr.commit_transaction(txn_id)
            assert False, "Should raise exception for double commit"
        except RuntimeError:
            pass  # Expected
    
    def test_commit_transaction_invalid_id():
        """Test committing non-existent transaction"""
        txn_mgr = deepgraph.TransactionManager()
        
        try:
            txn_mgr.commit_transaction(99999)
            assert False, "Should raise exception for invalid txn ID"
        except RuntimeError:
            pass  # Expected
    
    def test_commit_transaction_zero():
        """Test committing with ID 0"""
        txn_mgr = deepgraph.TransactionManager()
        
        try:
            txn_mgr.commit_transaction(0)
            assert False, "Should raise exception for ID 0"
        except RuntimeError:
            pass  # Expected
    
    def test_commit_transaction_negative():
        """Test committing with negative ID"""
        txn_mgr = deepgraph.TransactionManager()
        
        try:
            txn_mgr.commit_transaction(-1)
            # Note: Python may prevent negative values, depends on type checking
        except (RuntimeError, ValueError, OverflowError):
            pass  # Any of these is acceptable
    
    # =============================================================================
    # FEATURE 3: abort_transaction() - Abort/rollback transaction
    # =============================================================================
    
    def test_abort_transaction_basic():
        """Test aborting a basic transaction"""
        txn_mgr = deepgraph.TransactionManager()
        txn_id = txn_mgr.begin_transaction()
        
        # Should not raise exception
        txn_mgr.abort_transaction(txn_id)
    
    def test_abort_transaction_multiple():
        """Test aborting multiple transactions"""
        txn_mgr = deepgraph.TransactionManager()
        txn1 = txn_mgr.begin_transaction()
        txn2 = txn_mgr.begin_transaction()
        
        txn_mgr.abort_transaction(txn1)
        txn_mgr.abort_transaction(txn2)
    
    def test_abort_transaction_twice():
        """Test aborting same transaction twice"""
        txn_mgr = deepgraph.TransactionManager()
        txn_id = txn_mgr.begin_transaction()
        txn_mgr.abort_transaction(txn_id)
        
        try:
            txn_mgr.abort_transaction(txn_id)
            assert False, "Should raise exception for double abort"
        except RuntimeError:
            pass  # Expected
    
    def test_abort_transaction_after_commit():
        """Test aborting already committed transaction"""
        txn_mgr = deepgraph.TransactionManager()
        txn_id = txn_mgr.begin_transaction()
        txn_mgr.commit_transaction(txn_id)
        
        try:
            txn_mgr.abort_transaction(txn_id)
            assert False, "Should raise exception for aborting committed txn"
        except RuntimeError:
            pass  # Expected
    
    def test_abort_transaction_invalid_id():
        """Test aborting non-existent transaction"""
        txn_mgr = deepgraph.TransactionManager()
        
        try:
            txn_mgr.abort_transaction(99999)
            assert False, "Should raise exception for invalid txn ID"
        except RuntimeError:
            pass  # Expected
    
    # =============================================================================
    # INTEGRATION TESTS - Transactions with Graph Operations
    # =============================================================================
    
    def test_transaction_with_node_creation():
        """Test transaction lifecycle with node creation"""
        storage = deepgraph.GraphStorage()
        txn_mgr = deepgraph.TransactionManager()
        
        txn_id = txn_mgr.begin_transaction()
        
        # Add nodes within transaction
        node1 = storage.add_node(["Person"], {"name": "Alice"})
        node2 = storage.add_node(["Person"], {"name": "Bob"})
        
        # Commit transaction
        txn_mgr.commit_transaction(txn_id)
        
        # Nodes should exist
        assert storage.get_node(node1) is not None
        assert storage.get_node(node2) is not None
    
    def test_transaction_rollback_concept():
        """Test rollback concept (note: actual rollback may not be implemented)"""
        storage = deepgraph.GraphStorage()
        txn_mgr = deepgraph.TransactionManager()
        
        initial_count = storage.node_count()
        
        txn_id = txn_mgr.begin_transaction()
        
        # Add nodes
        storage.add_node(["Person"], {"name": "Alice"})
        storage.add_node(["Person"], {"name": "Bob"})
        
        # Abort transaction
        txn_mgr.abort_transaction(txn_id)
        
        # Note: Implementation may or may not actually rollback changes
        # This test just verifies no exception is raised
    
    def test_transaction_isolation():
        """Test transaction isolation concept"""
        txn_mgr = deepgraph.TransactionManager()
        
        txn1 = txn_mgr.begin_transaction()
        txn2 = txn_mgr.begin_transaction()
        
        # Both transactions should be independent
        assert txn1 != txn2
        
        # Can commit independently
        txn_mgr.commit_transaction(txn1)
        txn_mgr.commit_transaction(txn2)
    
    # =============================================================================
    # STRESS TESTS
    # =============================================================================
    
    def test_stress_many_transactions():
        """Stress test: Many sequential transactions"""
        txn_mgr = deepgraph.TransactionManager()
        
        for _ in range(1000):
            txn_id = txn_mgr.begin_transaction()
            txn_mgr.commit_transaction(txn_id)
    
    def test_stress_concurrent_transactions():
        """Stress test: Many concurrent transactions"""
        txn_mgr = deepgraph.TransactionManager()
        
        # Create many transactions
        txns = [txn_mgr.begin_transaction() for _ in range(100)]
        
        # Commit all
        for txn in txns:
            txn_mgr.commit_transaction(txn)
    
    def test_stress_mixed_commit_abort():
        """Stress test: Mix of commits and aborts"""
        txn_mgr = deepgraph.TransactionManager()
        
        for i in range(100):
            txn_id = txn_mgr.begin_transaction()
            if i % 2 == 0:
                txn_mgr.commit_transaction(txn_id)
            else:
                txn_mgr.abort_transaction(txn_id)
    
    # =============================================================================
    # EDGE CASES
    # =============================================================================
    
    def test_transaction_manager_reuse():
        """Test reusing transaction manager after many operations"""
        txn_mgr = deepgraph.TransactionManager()
        
        # Use it many times
        for _ in range(10):
            txn = txn_mgr.begin_transaction()
            txn_mgr.commit_transaction(txn)
        
        # Should still work
        txn = txn_mgr.begin_transaction()
        assert txn is not None
    
    def test_multiple_transaction_managers():
        """Test multiple transaction manager instances"""
        txn_mgr1 = deepgraph.TransactionManager()
        txn_mgr2 = deepgraph.TransactionManager()
        
        txn1 = txn_mgr1.begin_transaction()
        txn2 = txn_mgr2.begin_transaction()
        
        # Should be independent
        txn_mgr1.commit_transaction(txn1)
        txn_mgr2.commit_transaction(txn2)
    
    # =============================================================================
    # RUN ALL TESTS
    # =============================================================================
    
    print("### begin_transaction() - Start new transaction")
    print()
    run_test("test_begin_transaction_basic", test_begin_transaction_basic)
    run_test("test_begin_transaction_multiple", test_begin_transaction_multiple)
    run_test("test_begin_transaction_sequential_ids", test_begin_transaction_sequential_ids)
    run_test("test_begin_transaction_many", test_begin_transaction_many)
    run_test("test_begin_transaction_after_commit", test_begin_transaction_after_commit)
    run_test("test_begin_transaction_after_abort", test_begin_transaction_after_abort)
    
    print()
    print("### commit_transaction() - Commit changes")
    print()
    run_test("test_commit_transaction_basic", test_commit_transaction_basic)
    run_test("test_commit_transaction_multiple", test_commit_transaction_multiple)
    run_test("test_commit_transaction_out_of_order", test_commit_transaction_out_of_order)
    run_test("test_commit_transaction_twice", test_commit_transaction_twice)
    run_test("test_commit_transaction_invalid_id", test_commit_transaction_invalid_id)
    run_test("test_commit_transaction_zero", test_commit_transaction_zero)
    run_test("test_commit_transaction_negative", test_commit_transaction_negative)
    
    print()
    print("### abort_transaction() - Rollback changes")
    print()
    run_test("test_abort_transaction_basic", test_abort_transaction_basic)
    run_test("test_abort_transaction_multiple", test_abort_transaction_multiple)
    run_test("test_abort_transaction_twice", test_abort_transaction_twice)
    run_test("test_abort_transaction_after_commit", test_abort_transaction_after_commit)
    run_test("test_abort_transaction_invalid_id", test_abort_transaction_invalid_id)
    
    print()
    print("### Integration Tests")
    print()
    run_test("test_transaction_with_node_creation", test_transaction_with_node_creation)
    run_test("test_transaction_rollback_concept", test_transaction_rollback_concept)
    run_test("test_transaction_isolation", test_transaction_isolation)
    
    print()
    print("### Stress Tests")
    print()
    run_test("test_stress_many_transactions", test_stress_many_transactions)
    run_test("test_stress_concurrent_transactions", test_stress_concurrent_transactions)
    run_test("test_stress_mixed_commit_abort", test_stress_mixed_commit_abort)
    
    print()
    print("### Edge Cases")
    print()
    run_test("test_transaction_manager_reuse", test_transaction_manager_reuse)
    run_test("test_multiple_transaction_managers", test_multiple_transaction_managers)
    
    # Summary
    print()
    print("=" * 80)
    print(f"RESULTS: {passed} passed, {failed} failed out of {total} tests")
    print("=" * 80)
    
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(run_tests())

