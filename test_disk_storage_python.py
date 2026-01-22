#!/usr/bin/env python3
"""Quick test for DiskStorage Python bindings"""

import deepgraph
import tempfile
import shutil

def test_disk_storage():
    # Create temp directory for test
    temp_dir = tempfile.mkdtemp()
    print(f"✅ Created temp directory: {temp_dir}")
    
    try:
        # Test 1: Create storage
        storage = deepgraph.DiskStorage(f"{temp_dir}/test_graph.db")
        print("✅ DiskStorage created successfully")
        
        # Test 2: Add nodes
        alice_id = storage.add_node(["Person"], {"name": "Alice", "age": 30})
        print(f"✅ Added Alice: {alice_id}")
        
        bob_id = storage.add_node(["Person"], {"name": "Bob", "age": 25})
        print(f"✅ Added Bob: {bob_id}")
        
        # Test 3: Get nodes
        alice = storage.get_node(alice_id)
        assert alice is not None
        assert alice['properties']['name'] == "Alice"
        assert alice['properties']['age'] == 30
        print(f"✅ Retrieved Alice: {alice['properties']['name']}")
        
        # Test 4: Count nodes
        count = storage.node_count()
        assert count == 2
        print(f"✅ Node count: {count}")
        
        # Test 5: Find by label
        people = storage.find_nodes_by_label("Person")
        assert len(people) == 2
        print(f"✅ Found {len(people)} Person nodes")
        
        # Test 6: Get stats
        stats = storage.stats()
        print(f"✅ Stats: {stats}")
        assert stats['node_count'] == 2
        assert stats['size_on_disk_bytes'] > 0
        
        # Test 7: Flush
        storage.flush()
        print("✅ Flushed to disk")
        
        # Test 8: Persistence - close and reopen
        del storage
        print("✅ Closed storage")
        
        storage2 = deepgraph.DiskStorage(f"{temp_dir}/test_graph.db")
        count2 = storage2.node_count()
        assert count2 == 2
        print(f"✅ Reopened storage, still has {count2} nodes")
        
        alice2 = storage2.get_node(alice_id)
        assert alice2 is not None
        assert alice2['properties']['name'] == "Alice"
        print("✅ Data persisted correctly!")
        
        # Test 9: Cypher query
        result = storage2.execute_cypher("MATCH (n:Person) WHERE n.age > 25 RETURN n;")
        print(f"✅ Cypher query returned {result['row_count']} results")
        assert result['row_count'] == 1  # Only Alice (age 30)
        assert result['rows'][0]['age'] == 30
        print(f"✅ Query result: {result['rows'][0]['name']} (age {result['rows'][0]['age']})")
        
        print("\n🎉 All tests passed!")
        return True
        
    except Exception as e:
        print(f"\n❌ Test failed: {e}")
        import traceback
        traceback.print_exc()
        return False
    finally:
        # Cleanup
        shutil.rmtree(temp_dir)
        print(f"✅ Cleaned up temp directory")

if __name__ == "__main__":
    success = test_disk_storage()
    exit(0 if success else 1)
