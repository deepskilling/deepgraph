#!/usr/bin/env python3
"""Test CSV/JSON import Python bindings"""

import deepgraph
import tempfile
import os

def test_csv_import():
    print("\n=== Testing CSV Import ===")
    
    # Create temp directory
    temp_dir = tempfile.mkdtemp()
    
    try:
        # Create test CSV files
        nodes_csv = os.path.join(temp_dir, "nodes.csv")
        with open(nodes_csv, 'w') as f:
            f.write("id,labels,name,age,city\n")
            f.write("1,Person,Alice,30,NYC\n")
            f.write("2,Person;Employee,Bob,25,SF\n")
            f.write("3,Company,Acme,,NYC\n")
        
        edges_csv = os.path.join(temp_dir, "edges.csv")
        with open(edges_csv, 'w') as f:
            f.write("from,to,type,since\n")
            f.write("1,2,KNOWS,2020\n")
            f.write("2,3,WORKS_AT,2021\n")
        
        # Test import
        storage = deepgraph.GraphStorage()
        
        # Import nodes
        print("  Importing nodes from CSV...")
        node_stats = storage.import_csv_nodes(nodes_csv)
        print(f"  ✅ Imported {node_stats['nodes_imported']} nodes")
        print(f"  ⏱  Duration: {node_stats['duration_ms']}ms")
        assert node_stats['nodes_imported'] == 3
        assert len(node_stats['node_id_map']) == 3
        assert len(node_stats['errors']) == 0
        
        # Verify nodes
        count = storage.node_count()
        assert count == 3
        print(f"  ✅ Verified {count} nodes in storage")
        
        # Import edges
        print("  Importing edges from CSV...")
        edge_stats = storage.import_csv_edges(edges_csv, node_stats['node_id_map'])
        print(f"  ✅ Imported {edge_stats['edges_imported']} edges")
        print(f"  ⏱  Duration: {edge_stats['duration_ms']}ms")
        assert edge_stats['edges_imported'] == 2
        assert len(edge_stats['errors']) == 0
        
        # Verify edges
        edge_count = storage.edge_count()
        assert edge_count == 2
        print(f"  ✅ Verified {edge_count} edges in storage")
        
        print("\n✅ CSV Import Test Passed!\n")
        return True
        
    except Exception as e:
        print(f"\n❌ CSV Import Test Failed: {e}\n")
        import traceback
        traceback.print_exc()
        return False
    finally:
        # Cleanup
        import shutil
        shutil.rmtree(temp_dir)

def test_json_import():
    print("\n=== Testing JSON Import ===")
    
    # Create temp directory
    temp_dir = tempfile.mkdtemp()
    
    try:
        # Create test JSON files
        nodes_json = os.path.join(temp_dir, "nodes.json")
        with open(nodes_json, 'w') as f:
            f.write('''[
                {"id": "1", "labels": ["Person"], "properties": {"name": "Alice", "age": 30}},
                {"id": "2", "labels": ["Person", "Employee"], "properties": {"name": "Bob", "age": 25}},
                {"id": "3", "labels": ["Company"], "properties": {"name": "Acme Corp"}}
            ]''')
        
        edges_json = os.path.join(temp_dir, "edges.json")
        with open(edges_json, 'w') as f:
            f.write('''[
                {"from": "1", "to": "2", "type": "KNOWS", "properties": {"since": 2020}},
                {"from": "2", "to": "3", "type": "WORKS_AT", "properties": {"since": 2021}}
            ]''')
        
        # Test import
        storage = deepgraph.GraphStorage()
        
        # Import nodes
        print("  Importing nodes from JSON...")
        node_stats = storage.import_json_nodes(nodes_json)
        print(f"  ✅ Imported {node_stats['nodes_imported']} nodes")
        print(f"  ⏱  Duration: {node_stats['duration_ms']}ms")
        assert node_stats['nodes_imported'] == 3
        assert len(node_stats['node_id_map']) == 3
        assert len(node_stats['errors']) == 0
        
        # Verify nodes
        count = storage.node_count()
        assert count == 3
        print(f"  ✅ Verified {count} nodes in storage")
        
        # Import edges
        print("  Importing edges from JSON...")
        edge_stats = storage.import_json_edges(edges_json, node_stats['node_id_map'])
        print(f"  ✅ Imported {edge_stats['edges_imported']} edges")
        print(f"  ⏱  Duration: {edge_stats['duration_ms']}ms")
        assert edge_stats['edges_imported'] == 2
        assert len(edge_stats['errors']) == 0
        
        # Verify edges
        edge_count = storage.edge_count()
        assert edge_count == 2
        print(f"  ✅ Verified {edge_count} edges in storage")
        
        print("\n✅ JSON Import Test Passed!\n")
        return True
        
    except Exception as e:
        print(f"\n❌ JSON Import Test Failed: {e}\n")
        import traceback
        traceback.print_exc()
        return False
    finally:
        # Cleanup
        import shutil
        shutil.rmtree(temp_dir)

def test_type_inference():
    print("\n=== Testing Type Inference ===")
    
    temp_dir = tempfile.mkdtemp()
    
    try:
        # Create CSV with different types
        csv_file = os.path.join(temp_dir, "types.csv")
        with open(csv_file, 'w') as f:
            f.write("id,labels,name,age,salary,active,rating\n")
            f.write("1,Person,Alice,30,75000.50,true,4.5\n")
            f.write("2,Person,Bob,25,60000,false,3.8\n")
        
        storage = deepgraph.GraphStorage()
        stats = storage.import_csv_nodes(csv_file)
        
        print(f"  ✅ Imported {stats['nodes_imported']} nodes with typed properties")
        assert stats['nodes_imported'] == 2
        assert len(stats['errors']) == 0
        
        print("\n✅ Type Inference Test Passed!\n")
        return True
        
    except Exception as e:
        print(f"\n❌ Type Inference Test Failed: {e}\n")
        import traceback
        traceback.print_exc()
        return False
    finally:
        import shutil
        shutil.rmtree(temp_dir)

def main():
    print("\n" + "="*50)
    print("DeepGraph Import Python Bindings Test Suite")
    print("="*50)
    
    results = []
    
    # Run all tests
    results.append(("CSV Import", test_csv_import()))
    results.append(("JSON Import", test_json_import()))
    results.append(("Type Inference", test_type_inference()))
    
    # Summary
    print("\n" + "="*50)
    print("Test Summary")
    print("="*50)
    
    passed = sum(1 for _, result in results if result)
    total = len(results)
    
    for name, result in results:
        status = "✅ PASS" if result else "❌ FAIL"
        print(f"  {status}: {name}")
    
    print(f"\nTotal: {passed}/{total} tests passed")
    
    if passed == total:
        print("\n🎉 All tests passed!\n")
        return 0
    else:
        print(f"\n❌ {total - passed} test(s) failed\n")
        return 1

if __name__ == "__main__":
    exit(main())
