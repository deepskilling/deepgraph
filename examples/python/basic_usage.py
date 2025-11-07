#!/usr/bin/env python3
"""
Basic usage example for DeepGraph Python bindings.

This example demonstrates:
- Creating a graph storage
- Adding nodes and edges
- Querying the graph
- Using transactions
"""

import deepgraph

def main():
    print("🚀 DeepGraph Python Example\n")
    print(f"Version: {deepgraph.__version__}")
    print(f"Author: {deepgraph.__author__}\n")

    # Create a graph storage
    print("1. Creating graph storage...")
    storage = deepgraph.GraphStorage()
    
    # Add nodes
    print("\n2. Adding nodes...")
    alice_id = storage.add_node(
        labels=["Person"],
        properties={
            "name": "Alice",
            "age": 30,
            "city": "San Francisco"
        }
    )
    print(f"   Added Alice: {alice_id}")
    
    bob_id = storage.add_node(
        labels=["Person"],
        properties={
            "name": "Bob",
            "age": 25,
            "city": "New York"
        }
    )
    print(f"   Added Bob: {bob_id}")
    
    company_id = storage.add_node(
        labels=["Company"],
        properties={
            "name": "TechCorp",
            "founded": 2010
        }
    )
    print(f"   Added Company: {company_id}")
    
    # Add edges
    print("\n3. Adding edges...")
    friendship_id = storage.add_edge(
        from_id=alice_id,
        to_id=bob_id,
        label="KNOWS",
        properties={"since": 2020}
    )
    print(f"   Added friendship: {friendship_id}")
    
    works_at_id = storage.add_edge(
        from_id=alice_id,
        to_id=company_id,
        label="WORKS_AT",
        properties={"role": "Engineer", "since": 2019}
    )
    print(f"   Added employment: {works_at_id}")
    
    # Query nodes
    print("\n4. Querying nodes...")
    alice = storage.get_node(alice_id)
    if alice:
        print(f"   Alice details: {alice}")
    
    # Find nodes by label
    print("\n5. Finding nodes by label...")
    people = storage.find_nodes_by_label("Person")
    print(f"   Found {len(people)} person node(s): {people}")
    
    companies = storage.find_nodes_by_label("Company")
    print(f"   Found {len(companies)} company node(s): {companies}")
    
    # Get edge details
    print("\n6. Getting edge details...")
    friendship = storage.get_edge(friendship_id)
    if friendship:
        print(f"   Friendship: {friendship}")
    
    works_at = storage.get_edge(works_at_id)
    if works_at:
        print(f"   Works at: {works_at}")
    
    # Graph statistics
    print("\n7. Graph statistics...")
    print(f"   Total nodes: {storage.node_count()}")
    print(f"   Total edges: {storage.edge_count()}")
    
    # Transactions
    print("\n8. Using transactions...")
    txn_manager = deepgraph.TransactionManager()
    
    txn_id = txn_manager.begin_transaction()
    print(f"   Started transaction: {txn_id}")
    
    # Perform operations within transaction
    charlie_id = storage.add_node(
        labels=["Person"],
        properties={"name": "Charlie", "age": 28}
    )
    print(f"   Added Charlie in transaction: {charlie_id}")
    
    # Commit transaction
    txn_manager.commit_transaction(txn_id)
    print(f"   Committed transaction: {txn_id}")
    
    print(f"\n   Final node count: {storage.node_count()}")
    
    print("\n✅ Example completed successfully!")

if __name__ == "__main__":
    main()

