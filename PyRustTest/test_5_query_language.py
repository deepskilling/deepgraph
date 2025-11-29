#!/usr/bin/env python3
"""
Comprehensive tests for DeepGraph Query Language (Cypher) (5 methods)

Tests cover:
- CypherParser.parse() - Parse Cypher query to AST
- CypherParser.validate() - Validate query syntax
- QueryPlanner.create_logical_plan() - Create logical plan
- QueryPlanner.optimize() - Optimize query plan
- QueryExecutor.execute() - Execute query plan

Each test includes query parsing, validation, planning, and execution.
"""

import sys
import traceback


def run_tests():
    """Run all query language tests"""
    print("=" * 80)
    print("TEST SUITE 5: QUERY LANGUAGE (Cypher) (5 methods)")
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
            # traceback.print_exc()  # Uncomment for debugging
            failed += 1
    
    # =============================================================================
    # FEATURE 1: CypherParser.parse() - Parse Cypher query
    # =============================================================================
    
    def test_parse_basic_match():
        """Test parsing basic MATCH query"""
        parser = deepgraph.CypherParser()
        
        query = "MATCH (n) RETURN n"
        ast = parser.parse(query)
        assert ast is not None
    
    def test_parse_match_with_label():
        """Test parsing MATCH with node label"""
        parser = deepgraph.CypherParser()
        
        query = "MATCH (p:Person) RETURN p"
        ast = parser.parse(query)
        assert ast is not None
    
    def test_parse_match_with_property():
        """Test parsing MATCH with property filter"""
        parser = deepgraph.CypherParser()
        
        query = "MATCH (p:Person {name: 'Alice'}) RETURN p"
        ast = parser.parse(query)
        assert ast is not None
    
    def test_parse_match_relationship():
        """Test parsing MATCH with relationship"""
        parser = deepgraph.CypherParser()
        
        query = "MATCH (a)-[r:KNOWS]->(b) RETURN a, r, b"
        ast = parser.parse(query)
        assert ast is not None
    
    def test_parse_where_clause():
        """Test parsing MATCH with WHERE clause"""
        parser = deepgraph.CypherParser()
        
        query = "MATCH (p:Person) WHERE p.age > 30 RETURN p"
        ast = parser.parse(query)
        assert ast is not None
    
    def test_parse_return_multiple():
        """Test parsing RETURN with multiple items"""
        parser = deepgraph.CypherParser()
        
        query = "MATCH (p:Person) RETURN p.name, p.age"
        ast = parser.parse(query)
        assert ast is not None
    
    def test_parse_limit():
        """Test parsing query with LIMIT"""
        parser = deepgraph.CypherParser()
        
        query = "MATCH (n) RETURN n LIMIT 10"
        ast = parser.parse(query)
        assert ast is not None
    
    def test_parse_order_by():
        """Test parsing query with ORDER BY"""
        parser = deepgraph.CypherParser()
        
        query = "MATCH (p:Person) RETURN p ORDER BY p.age"
        ast = parser.parse(query)
        assert ast is not None
    
    def test_parse_create_node():
        """Test parsing CREATE node query"""
        parser = deepgraph.CypherParser()
        
        query = "CREATE (p:Person {name: 'Bob'})"
        ast = parser.parse(query)
        assert ast is not None
    
    def test_parse_create_relationship():
        """Test parsing CREATE relationship query"""
        parser = deepgraph.CypherParser()
        
        query = "CREATE (a)-[r:KNOWS]->(b)"
        ast = parser.parse(query)
        assert ast is not None
    
    def test_parse_empty_query():
        """Test parsing empty query"""
        parser = deepgraph.CypherParser()
        
        try:
            ast = parser.parse("")
            # May succeed with empty AST or raise exception
        except (RuntimeError, ValueError):
            pass  # Expected
    
    def test_parse_whitespace_only():
        """Test parsing whitespace-only query"""
        parser = deepgraph.CypherParser()
        
        try:
            ast = parser.parse("   \n\t  ")
            # May succeed or fail
        except (RuntimeError, ValueError):
            pass  # Expected
    
    # =============================================================================
    # FEATURE 2: CypherParser.validate() - Validate query syntax
    # =============================================================================
    
    def test_validate_valid_query():
        """Test validating valid query"""
        parser = deepgraph.CypherParser()
        
        query = "MATCH (n) RETURN n"
        is_valid = parser.validate(query)
        # Should not raise exception
    
    def test_validate_invalid_syntax():
        """Test validating query with syntax error"""
        parser = deepgraph.CypherParser()
        
        query = "MATCH (n RETURN n"  # Missing closing paren
        try:
            is_valid = parser.validate(query)
            # May return False or raise exception
        except RuntimeError:
            pass  # Expected
    
    def test_validate_incomplete_query():
        """Test validating incomplete query"""
        parser = deepgraph.CypherParser()
        
        query = "MATCH (n)"  # No RETURN
        try:
            is_valid = parser.validate(query)
            # May be valid or invalid depending on grammar
        except RuntimeError:
            pass  # May fail
    
    def test_validate_empty_query():
        """Test validating empty query"""
        parser = deepgraph.CypherParser()
        
        try:
            is_valid = parser.validate("")
            # Should fail or return False
        except (RuntimeError, ValueError):
            pass  # Expected
    
    def test_validate_nonsense_query():
        """Test validating nonsense query"""
        parser = deepgraph.CypherParser()
        
        query = "XYZABC FOO BAR"
        try:
            is_valid = parser.validate(query)
            # Should fail or return False
        except RuntimeError:
            pass  # Expected
    
    def test_validate_multiple_queries():
        """Test validating multiple queries in sequence"""
        parser = deepgraph.CypherParser()
        
        queries = [
            "MATCH (n) RETURN n",
            "MATCH (p:Person) RETURN p",
            "CREATE (n:Test)",
        ]
        
        for query in queries:
            parser.validate(query)  # Should not raise
    
    # =============================================================================
    # FEATURE 3: QueryPlanner.create_logical_plan() - Create logical plan
    # =============================================================================
    
    def test_create_logical_plan_basic():
        """Test creating basic logical plan"""
        storage = deepgraph.GraphStorage()
        parser = deepgraph.CypherParser()
        planner = deepgraph.QueryPlanner()
        
        query = "MATCH (n) RETURN n"
        ast = parser.parse(query)
        
        logical_plan = planner.create_logical_plan(ast)
        assert logical_plan is not None
    
    def test_create_logical_plan_with_filter():
        """Test creating logical plan with WHERE clause"""
        storage = deepgraph.GraphStorage()
        parser = deepgraph.CypherParser()
        planner = deepgraph.QueryPlanner()
        
        query = "MATCH (p:Person) WHERE p.age > 30 RETURN p"
        ast = parser.parse(query)
        
        logical_plan = planner.create_logical_plan(ast)
        assert logical_plan is not None
    
    def test_create_logical_plan_with_relationship():
        """Test creating logical plan with relationship"""
        storage = deepgraph.GraphStorage()
        parser = deepgraph.CypherParser()
        planner = deepgraph.QueryPlanner()
        
        query = "MATCH (a)-[r:KNOWS]->(b) RETURN a, b"
        ast = parser.parse(query)
        
        logical_plan = planner.create_logical_plan(ast)
        assert logical_plan is not None
    
    def test_create_logical_plan_multiple_calls():
        """Test creating multiple logical plans"""
        storage = deepgraph.GraphStorage()
        parser = deepgraph.CypherParser()
        planner = deepgraph.QueryPlanner()
        
        queries = [
            "MATCH (n) RETURN n",
            "MATCH (p:Person) RETURN p",
            "MATCH (c:Company) RETURN c",
        ]
        
        for query in queries:
            ast = parser.parse(query)
            plan = planner.create_logical_plan(ast)
            assert plan is not None
    
    # =============================================================================
    # FEATURE 4: QueryPlanner.optimize() - Optimize query plan
    # =============================================================================
    
    def test_optimize_basic_plan():
        """Test optimizing basic plan"""
        storage = deepgraph.GraphStorage()
        parser = deepgraph.CypherParser()
        planner = deepgraph.QueryPlanner()
        
        query = "MATCH (n) RETURN n"
        ast = parser.parse(query)
        logical_plan = planner.create_logical_plan(ast)
        
        optimized_plan = planner.optimize(logical_plan)
        assert optimized_plan is not None
    
    def test_optimize_idempotent():
        """Test that optimizing twice is safe"""
        storage = deepgraph.GraphStorage()
        parser = deepgraph.CypherParser()
        planner = deepgraph.QueryPlanner()
        
        query = "MATCH (n) RETURN n"
        ast = parser.parse(query)
        logical_plan = planner.create_logical_plan(ast)
        
        opt1 = planner.optimize(logical_plan)
        opt2 = planner.optimize(opt1)
        
        assert opt1 is not None
        assert opt2 is not None
    
    def test_optimize_multiple_plans():
        """Test optimizing multiple plans"""
        storage = deepgraph.GraphStorage()
        parser = deepgraph.CypherParser()
        planner = deepgraph.QueryPlanner()
        
        queries = [
            "MATCH (n) RETURN n",
            "MATCH (p:Person) WHERE p.age > 30 RETURN p",
            "MATCH (a)-[r]->(b) RETURN a, b",
        ]
        
        for query in queries:
            ast = parser.parse(query)
            logical = planner.create_logical_plan(ast)
            optimized = planner.optimize(logical)
            assert optimized is not None
    
    # =============================================================================
    # FEATURE 5: QueryExecutor.execute() - Execute query plan
    # =============================================================================
    
    def test_execute_basic_query():
        """Test executing basic query"""
        storage = deepgraph.GraphStorage()
        parser = deepgraph.CypherParser()
        planner = deepgraph.QueryPlanner()
        executor = deepgraph.QueryExecutor(storage)
        
        # Add some data
        storage.add_node(["Person"], {"name": "Alice"})
        storage.add_node(["Person"], {"name": "Bob"})
        
        # Execute query
        query = "MATCH (n) RETURN n"
        ast = parser.parse(query)
        logical_plan = planner.create_logical_plan(ast)
        physical_plan = planner.optimize(logical_plan)
        
        result = executor.execute(physical_plan)
        assert result is not None
    
    def test_execute_on_empty_graph():
        """Test executing query on empty graph"""
        storage = deepgraph.GraphStorage()
        parser = deepgraph.CypherParser()
        planner = deepgraph.QueryPlanner()
        executor = deepgraph.QueryExecutor(storage)
        
        query = "MATCH (n) RETURN n"
        ast = parser.parse(query)
        logical_plan = planner.create_logical_plan(ast)
        physical_plan = planner.optimize(logical_plan)
        
        result = executor.execute(physical_plan)
        assert result is not None  # Should return empty result
    
    def test_execute_multiple_queries():
        """Test executing multiple queries"""
        storage = deepgraph.GraphStorage()
        parser = deepgraph.CypherParser()
        planner = deepgraph.QueryPlanner()
        executor = deepgraph.QueryExecutor(storage)
        
        # Add data
        storage.add_node(["Person"], {"name": "Alice"})
        storage.add_node(["Company"], {"name": "Acme"})
        
        queries = [
            "MATCH (n) RETURN n",
            "MATCH (p:Person) RETURN p",
            "MATCH (c:Company) RETURN c",
        ]
        
        for query in queries:
            ast = parser.parse(query)
            logical = planner.create_logical_plan(ast)
            physical = planner.optimize(logical)
            result = executor.execute(physical)
            assert result is not None
    
    # =============================================================================
    # INTEGRATION TESTS - Full Query Pipeline
    # =============================================================================
    
    def test_full_query_pipeline():
        """Test complete query pipeline: parse -> plan -> optimize -> execute"""
        storage = deepgraph.GraphStorage()
        
        # Add test data
        alice = storage.add_node(["Person"], {"name": "Alice", "age": 30})
        bob = storage.add_node(["Person"], {"name": "Bob", "age": 25})
        storage.add_edge(alice, bob, "KNOWS", {})
        
        # Parse
        parser = deepgraph.CypherParser()
        query = "MATCH (p:Person) RETURN p"
        ast = parser.parse(query)
        
        # Plan
        planner = deepgraph.QueryPlanner()
        logical_plan = planner.create_logical_plan(ast)
        physical_plan = planner.optimize(logical_plan)
        
        # Execute
        executor = deepgraph.QueryExecutor(storage)
        result = executor.execute(physical_plan)
        
        assert result is not None
    
    def test_query_pipeline_with_validation():
        """Test query pipeline with validation step"""
        storage = deepgraph.GraphStorage()
        parser = deepgraph.CypherParser()
        
        query = "MATCH (n) RETURN n"
        
        # Validate first
        parser.validate(query)
        
        # Then parse and execute
        ast = parser.parse(query)
        planner = deepgraph.QueryPlanner()
        logical = planner.create_logical_plan(ast)
        physical = planner.optimize(logical)
        executor = deepgraph.QueryExecutor(storage)
        result = executor.execute(physical)
        
        assert result is not None
    
    def test_query_pipeline_error_recovery():
        """Test query pipeline with invalid query handling"""
        storage = deepgraph.GraphStorage()
        parser = deepgraph.CypherParser()
        
        invalid_query = "MATCH (n RETURN n"  # Syntax error
        
        try:
            parser.validate(invalid_query)
            assert False, "Should raise exception for invalid query"
        except RuntimeError:
            pass  # Expected
        
        # Valid query should still work
        valid_query = "MATCH (n) RETURN n"
        parser.validate(valid_query)
    
    # =============================================================================
    # EDGE CASES
    # =============================================================================
    
    def test_parser_reuse():
        """Test reusing parser for multiple queries"""
        parser = deepgraph.CypherParser()
        
        for i in range(10):
            query = f"MATCH (n) RETURN n"
            ast = parser.parse(query)
            assert ast is not None
    
    def test_planner_reuse():
        """Test reusing planner for multiple plans"""
        storage = deepgraph.GraphStorage()
        parser = deepgraph.CypherParser()
        planner = deepgraph.QueryPlanner()
        
        for i in range(10):
            query = "MATCH (n) RETURN n"
            ast = parser.parse(query)
            plan = planner.create_logical_plan(ast)
            assert plan is not None
    
    def test_executor_reuse():
        """Test reusing executor for multiple queries"""
        storage = deepgraph.GraphStorage()
        storage.add_node(["Test"], {})
        
        parser = deepgraph.CypherParser()
        planner = deepgraph.QueryPlanner()
        executor = deepgraph.QueryExecutor(storage)
        
        for i in range(10):
            query = "MATCH (n) RETURN n"
            ast = parser.parse(query)
            logical = planner.create_logical_plan(ast)
            physical = planner.optimize(logical)
            result = executor.execute(physical)
            assert result is not None
    
    def test_multiple_planners():
        """Test multiple planner instances"""
        storage = deepgraph.GraphStorage()
        parser = deepgraph.CypherParser()
        
        planner1 = deepgraph.QueryPlanner()
        planner2 = deepgraph.QueryPlanner()
        
        query = "MATCH (n) RETURN n"
        ast = parser.parse(query)
        
        plan1 = planner1.create_logical_plan(ast)
        plan2 = planner2.create_logical_plan(ast)
        
        assert plan1 is not None
        assert plan2 is not None
    
    # =============================================================================
    # RUN ALL TESTS
    # =============================================================================
    
    print("### CypherParser.parse() - Parse Cypher query to AST")
    print()
    run_test("test_parse_basic_match", test_parse_basic_match)
    run_test("test_parse_match_with_label", test_parse_match_with_label)
    run_test("test_parse_match_with_property", test_parse_match_with_property)
    run_test("test_parse_match_relationship", test_parse_match_relationship)
    run_test("test_parse_where_clause", test_parse_where_clause)
    run_test("test_parse_return_multiple", test_parse_return_multiple)
    run_test("test_parse_limit", test_parse_limit)
    run_test("test_parse_order_by", test_parse_order_by)
    run_test("test_parse_create_node", test_parse_create_node)
    run_test("test_parse_create_relationship", test_parse_create_relationship)
    run_test("test_parse_empty_query", test_parse_empty_query)
    run_test("test_parse_whitespace_only", test_parse_whitespace_only)
    
    print()
    print("### CypherParser.validate() - Validate query syntax")
    print()
    run_test("test_validate_valid_query", test_validate_valid_query)
    run_test("test_validate_invalid_syntax", test_validate_invalid_syntax)
    run_test("test_validate_incomplete_query", test_validate_incomplete_query)
    run_test("test_validate_empty_query", test_validate_empty_query)
    run_test("test_validate_nonsense_query", test_validate_nonsense_query)
    run_test("test_validate_multiple_queries", test_validate_multiple_queries)
    
    print()
    print("### QueryPlanner.create_logical_plan() - Create logical plan")
    print()
    run_test("test_create_logical_plan_basic", test_create_logical_plan_basic)
    run_test("test_create_logical_plan_with_filter", test_create_logical_plan_with_filter)
    run_test("test_create_logical_plan_with_relationship", test_create_logical_plan_with_relationship)
    run_test("test_create_logical_plan_multiple_calls", test_create_logical_plan_multiple_calls)
    
    print()
    print("### QueryPlanner.optimize() - Optimize query plan")
    print()
    run_test("test_optimize_basic_plan", test_optimize_basic_plan)
    run_test("test_optimize_idempotent", test_optimize_idempotent)
    run_test("test_optimize_multiple_plans", test_optimize_multiple_plans)
    
    print()
    print("### QueryExecutor.execute() - Execute query plan")
    print()
    run_test("test_execute_basic_query", test_execute_basic_query)
    run_test("test_execute_on_empty_graph", test_execute_on_empty_graph)
    run_test("test_execute_multiple_queries", test_execute_multiple_queries)
    
    print()
    print("### Integration Tests - Full Query Pipeline")
    print()
    run_test("test_full_query_pipeline", test_full_query_pipeline)
    run_test("test_query_pipeline_with_validation", test_query_pipeline_with_validation)
    run_test("test_query_pipeline_error_recovery", test_query_pipeline_error_recovery)
    
    print()
    print("### Edge Cases")
    print()
    run_test("test_parser_reuse", test_parser_reuse)
    run_test("test_planner_reuse", test_planner_reuse)
    run_test("test_executor_reuse", test_executor_reuse)
    run_test("test_multiple_planners", test_multiple_planners)
    
    # Summary
    print()
    print("=" * 80)
    print(f"RESULTS: {passed} passed, {failed} failed out of {total} tests")
    print("=" * 80)
    
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(run_tests())

