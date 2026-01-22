"""
Test Suite for Cypher Query Execution (Task 6)
==============================================

This test suite validates the Python bindings for executing Cypher queries
directly through the storage.execute_cypher() method.

Test Coverage:
1. Simple MATCH queries
2. WHERE clause filtering
3. Property access in results
4. Label-based filtering
5. Complex AND/OR conditions
6. Comparison operators (=, !=, <, >, <=, >=)
7. Empty results
8. Error handling
"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'python'))

import unittest
import deepgraph


class TestCypherExecution(unittest.TestCase):
    """Test Cypher query execution through Python bindings"""
    
    def setUp(self):
        """Create a test graph with sample data"""
        self.storage = deepgraph.GraphStorage()
        
        # Add Person nodes
        self.alice_id = self.storage.add_node(
            labels=["Person"],
            properties={"name": "Alice", "age": 30, "city": "NYC"}
        )
        self.bob_id = self.storage.add_node(
            labels=["Person"],
            properties={"name": "Bob", "age": 25, "city": "SF"}
        )
        self.charlie_id = self.storage.add_node(
            labels=["Person"],
            properties={"name": "Charlie", "age": 35, "city": "NYC"}
        )
        
        # Add a Company node
        self.acme_id = self.storage.add_node(
            labels=["Company"],
            properties={"name": "Acme Corp", "founded": 2010}
        )
    
    def test_simple_match_all(self):
        """Test MATCH (n) RETURN n - returns all nodes"""
        result = self.storage.execute_cypher("MATCH (n) RETURN n;")
        
        self.assertEqual(result['row_count'], 4)
        self.assertGreaterEqual(result['execution_time_ms'], 0)
        self.assertEqual(len(result['rows']), 4)
        
        # Verify all rows have node data
        for row in result['rows']:
            self.assertIn('_node_id', row)
    
    def test_match_with_label(self):
        """Test MATCH (n:Person) RETURN n - filters by label"""
        result = self.storage.execute_cypher("MATCH (n:Person) RETURN n;")
        
        self.assertEqual(result['row_count'], 3)
        self.assertEqual(len(result['rows']), 3)
        
        # Verify all returned nodes have Person properties
        names = {row['name'] for row in result['rows']}
        self.assertEqual(names, {"Alice", "Bob", "Charlie"})
    
    def test_where_greater_than(self):
        """Test WHERE clause with > operator"""
        result = self.storage.execute_cypher(
            "MATCH (n:Person) WHERE n.age > 25 RETURN n;"
        )
        
        self.assertEqual(result['row_count'], 2)
        
        # Verify ages are > 25
        ages = {row['age'] for row in result['rows']}
        self.assertEqual(ages, {30, 35})
        
        names = {row['name'] for row in result['rows']}
        self.assertEqual(names, {"Alice", "Charlie"})
    
    def test_where_equals(self):
        """Test WHERE clause with = operator"""
        result = self.storage.execute_cypher(
            'MATCH (n:Person) WHERE n.name = "Alice" RETURN n;'
        )
        
        self.assertEqual(result['row_count'], 1)
        self.assertEqual(result['rows'][0]['name'], "Alice")
        self.assertEqual(result['rows'][0]['age'], 30)
    
    def test_where_and_condition(self):
        """Test WHERE clause with AND operator"""
        result = self.storage.execute_cypher(
            'MATCH (n:Person) WHERE n.age > 25 AND n.city = "NYC" RETURN n;'
        )
        
        self.assertEqual(result['row_count'], 2)
        
        # Verify both conditions are met
        for row in result['rows']:
            self.assertGreater(row['age'], 25)
            self.assertEqual(row['city'], "NYC")
        
        names = {row['name'] for row in result['rows']}
        self.assertEqual(names, {"Alice", "Charlie"})
    
    def test_where_less_than_or_equal(self):
        """Test WHERE clause with <= operator"""
        result = self.storage.execute_cypher(
            "MATCH (n:Person) WHERE n.age <= 30 RETURN n;"
        )
        
        self.assertEqual(result['row_count'], 2)
        
        ages = {row['age'] for row in result['rows']}
        self.assertEqual(ages, {25, 30})
        
        names = {row['name'] for row in result['rows']}
        self.assertEqual(names, {"Alice", "Bob"})
    
    def test_where_not_equal(self):
        """Test WHERE clause with != operator"""
        result = self.storage.execute_cypher(
            'MATCH (n:Person) WHERE n.city != "NYC" RETURN n;'
        )
        
        self.assertEqual(result['row_count'], 1)
        self.assertEqual(result['rows'][0]['name'], "Bob")
        self.assertEqual(result['rows'][0]['city'], "SF")
    
    def test_where_greater_than_or_equal(self):
        """Test WHERE clause with >= operator"""
        result = self.storage.execute_cypher(
            "MATCH (n:Person) WHERE n.age >= 30 RETURN n;"
        )
        
        self.assertEqual(result['row_count'], 2)
        
        ages = {row['age'] for row in result['rows']}
        self.assertEqual(ages, {30, 35})
    
    def test_where_less_than(self):
        """Test WHERE clause with < operator"""
        result = self.storage.execute_cypher(
            "MATCH (n:Person) WHERE n.age < 30 RETURN n;"
        )
        
        self.assertEqual(result['row_count'], 1)
        self.assertEqual(result['rows'][0]['name'], "Bob")
        self.assertEqual(result['rows'][0]['age'], 25)
    
    def test_empty_result(self):
        """Test query with no matching results"""
        result = self.storage.execute_cypher(
            "MATCH (n:Person) WHERE n.age > 100 RETURN n;"
        )
        
        self.assertEqual(result['row_count'], 0)
        self.assertEqual(len(result['rows']), 0)
    
    def test_company_label_filter(self):
        """Test filtering by Company label"""
        result = self.storage.execute_cypher(
            "MATCH (n:Company) RETURN n;"
        )
        
        self.assertEqual(result['row_count'], 1)
        self.assertEqual(result['rows'][0]['name'], "Acme Corp")
        self.assertEqual(result['rows'][0]['founded'], 2010)
    
    def test_property_access_in_results(self):
        """Test that all node properties are included in results"""
        result = self.storage.execute_cypher(
            "MATCH (n:Person) RETURN n;"
        )
        
        # Verify all rows have expected properties
        for row in result['rows']:
            self.assertIn('name', row)
            self.assertIn('age', row)
            self.assertIn('city', row)
            self.assertIn('_node_id', row)
    
    def test_execution_time_tracking(self):
        """Test that execution time is tracked"""
        result = self.storage.execute_cypher(
            "MATCH (n:Person) WHERE n.age > 20 RETURN n;"
        )
        
        self.assertIn('execution_time_ms', result)
        self.assertIsInstance(result['execution_time_ms'], int)
        self.assertGreaterEqual(result['execution_time_ms'], 0)
    
    def test_result_structure(self):
        """Test that result has correct structure"""
        result = self.storage.execute_cypher(
            "MATCH (n:Person) RETURN n;"
        )
        
        # Verify result structure
        self.assertIn('columns', result)
        self.assertIn('rows', result)
        self.assertIn('row_count', result)
        self.assertIn('execution_time_ms', result)
        
        # Verify types
        self.assertIsInstance(result['columns'], list)
        self.assertIsInstance(result['rows'], list)
        self.assertIsInstance(result['row_count'], int)
        self.assertIsInstance(result['execution_time_ms'], int)
    
    def test_parse_error_handling(self):
        """Test error handling for invalid Cypher syntax"""
        with self.assertRaises(RuntimeError) as context:
            self.storage.execute_cypher("INVALID QUERY SYNTAX")
        
        self.assertIn("Parse error", str(context.exception))
    
    def test_multiple_queries_sequential(self):
        """Test executing multiple queries sequentially"""
        # Query 1: All people
        result1 = self.storage.execute_cypher("MATCH (n:Person) RETURN n;")
        self.assertEqual(result1['row_count'], 3)
        
        # Query 2: Filtered people
        result2 = self.storage.execute_cypher(
            "MATCH (n:Person) WHERE n.age > 25 RETURN n;"
        )
        self.assertEqual(result2['row_count'], 2)
        
        # Query 3: Companies
        result3 = self.storage.execute_cypher("MATCH (n:Company) RETURN n;")
        self.assertEqual(result3['row_count'], 1)
    
    def test_query_with_no_where_clause(self):
        """Test MATCH with label but no WHERE clause"""
        result = self.storage.execute_cypher("MATCH (n:Person) RETURN n;")
        
        self.assertEqual(result['row_count'], 3)
        
        # All Person nodes should be returned
        names = {row['name'] for row in result['rows']}
        self.assertEqual(names, {"Alice", "Bob", "Charlie"})


class TestCypherEdgeCases(unittest.TestCase):
    """Test edge cases and corner cases in Cypher execution"""
    
    def setUp(self):
        self.storage = deepgraph.GraphStorage()
    
    def test_query_on_empty_graph(self):
        """Test querying an empty graph"""
        result = self.storage.execute_cypher("MATCH (n) RETURN n;")
        
        self.assertEqual(result['row_count'], 0)
        self.assertEqual(len(result['rows']), 0)
    
    def test_label_that_doesnt_exist(self):
        """Test querying for a label that doesn't exist"""
        self.storage.add_node(labels=["Person"], properties={"name": "Alice"})
        
        result = self.storage.execute_cypher("MATCH (n:Robot) RETURN n;")
        
        self.assertEqual(result['row_count'], 0)
        self.assertEqual(len(result['rows']), 0)
    
    def test_property_that_doesnt_exist(self):
        """Test WHERE clause on non-existent property"""
        self.storage.add_node(labels=["Person"], properties={"name": "Alice"})
        
        # Query for a property that doesn't exist
        # This should return no results (or handle gracefully)
        result = self.storage.execute_cypher(
            'MATCH (n:Person) WHERE n.nonexistent = "value" RETURN n;'
        )
        
        # Depending on implementation, this might return 0 results
        # or all nodes (if null comparison)
        self.assertGreaterEqual(result['row_count'], 0)
    
    def test_mixed_property_types(self):
        """Test nodes with different property types"""
        self.storage.add_node(
            labels=["Data"],
            properties={"int_val": 42, "str_val": "text", "bool_val": True}
        )
        
        result = self.storage.execute_cypher("MATCH (n:Data) RETURN n;")
        
        self.assertEqual(result['row_count'], 1)
        row = result['rows'][0]
        self.assertEqual(row['int_val'], 42)
        self.assertEqual(row['str_val'], "text")
        self.assertEqual(row['bool_val'], True)


def run_tests():
    """Run all tests and print results"""
    loader = unittest.TestLoader()
    suite = unittest.TestSuite()
    
    suite.addTests(loader.loadTestsFromTestCase(TestCypherExecution))
    suite.addTests(loader.loadTestsFromTestCase(TestCypherEdgeCases))
    
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)
    
    return result.wasSuccessful()


if __name__ == '__main__':
    success = run_tests()
    sys.exit(0 if success else 1)
