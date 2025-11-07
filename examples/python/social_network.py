#!/usr/bin/env python3
"""
Social Network Graph Example

This example demonstrates building a social network graph with:
- Users with profiles
- Friendships between users
- Posts and likes
- Groups and memberships
"""

import deepgraph
from datetime import datetime

def create_social_network():
    """Create and populate a social network graph."""
    
    print("📱 Building Social Network Graph\n")
    
    # Create storage
    storage = deepgraph.GraphStorage()
    
    # Add users
    print("1. Creating users...")
    users = {}
    
    users['alice'] = storage.add_node(
        labels=["User"],
        properties={
            "username": "alice",
            "email": "alice@example.com",
            "age": 28,
            "verified": True
        }
    )
    
    users['bob'] = storage.add_node(
        labels=["User"],
        properties={
            "username": "bob",
            "email": "bob@example.com",
            "age": 32,
            "verified": True
        }
    )
    
    users['charlie'] = storage.add_node(
        labels=["User"],
        properties={
            "username": "charlie",
            "email": "charlie@example.com",
            "age": 25,
            "verified": False
        }
    )
    
    users['diana'] = storage.add_node(
        labels=["User"],
        properties={
            "username": "diana",
            "email": "diana@example.com",
            "age": 30,
            "verified": True
        }
    )
    
    print(f"   Created {len(users)} users")
    
    # Add friendships
    print("\n2. Creating friendships...")
    friendships = []
    
    # Alice's friends
    friendships.append(storage.add_edge(
        from_id=users['alice'],
        to_id=users['bob'],
        label="FRIENDS_WITH",
        properties={"since": 2019}
    ))
    
    friendships.append(storage.add_edge(
        from_id=users['alice'],
        to_id=users['charlie'],
        label="FRIENDS_WITH",
        properties={"since": 2020}
    ))
    
    # Bob's friends
    friendships.append(storage.add_edge(
        from_id=users['bob'],
        to_id=users['diana'],
        label="FRIENDS_WITH",
        properties={"since": 2018}
    ))
    
    # Charlie and Diana are friends
    friendships.append(storage.add_edge(
        from_id=users['charlie'],
        to_id=users['diana'],
        label="FRIENDS_WITH",
        properties={"since": 2021}
    ))
    
    print(f"   Created {len(friendships)} friendships")
    
    # Add posts
    print("\n3. Creating posts...")
    posts = {}
    
    posts['post1'] = storage.add_node(
        labels=["Post"],
        properties={
            "content": "Hello, world! 🌍",
            "timestamp": 1635724800,
            "likes": 42
        }
    )
    
    posts['post2'] = storage.add_node(
        labels=["Post"],
        properties={
            "content": "Beautiful sunset today!",
            "timestamp": 1635811200,
            "likes": 156
        }
    )
    
    # Link posts to authors
    storage.add_edge(
        from_id=users['alice'],
        to_id=posts['post1'],
        label="POSTED",
        properties={"timestamp": 1635724800}
    )
    
    storage.add_edge(
        from_id=users['bob'],
        to_id=posts['post2'],
        label="POSTED",
        properties={"timestamp": 1635811200}
    )
    
    # Add likes
    storage.add_edge(
        from_id=users['bob'],
        to_id=posts['post1'],
        label="LIKES",
        properties={"timestamp": 1635725000}
    )
    
    storage.add_edge(
        from_id=users['charlie'],
        to_id=posts['post1'],
        label="LIKES",
        properties={"timestamp": 1635725100}
    )
    
    storage.add_edge(
        from_id=users['alice'],
        to_id=posts['post2'],
        label="LIKES",
        properties={"timestamp": 1635811500}
    )
    
    print(f"   Created {len(posts)} posts with likes")
    
    # Add a group
    print("\n4. Creating group...")
    group_id = storage.add_node(
        labels=["Group"],
        properties={
            "name": "Rust Developers",
            "description": "A community for Rust enthusiasts",
            "members": 0
        }
    )
    
    # Add memberships
    for user_name, user_id in users.items():
        storage.add_edge(
            from_id=user_id,
            to_id=group_id,
            label="MEMBER_OF",
            properties={"joined": 2022, "role": "member"}
        )
    
    print(f"   Created group with {len(users)} members")
    
    # Print statistics
    print("\n5. Network Statistics:")
    print(f"   Total nodes: {storage.node_count()}")
    print(f"   Total edges: {storage.edge_count()}")
    print(f"   Users: {len(storage.find_nodes_by_label('User'))}")
    print(f"   Posts: {len(storage.find_nodes_by_label('Post'))}")
    print(f"   Groups: {len(storage.find_nodes_by_label('Group'))}")
    
    # Query example
    print("\n6. Query Examples:")
    alice_node = storage.get_node(users['alice'])
    if alice_node:
        print(f"   Alice's profile: {alice_node['properties']}")
    
    return storage, users, posts

def main():
    storage, users, posts = create_social_network()
    print("\n✅ Social network created successfully!")
    
    # Demonstrate some queries
    print("\n7. Advanced Queries:")
    
    # Find all verified users
    all_users = storage.find_nodes_by_label("User")
    verified_count = 0
    for user_id in all_users:
        user = storage.get_node(user_id)
        if user and user['properties'].get('verified'):
            verified_count += 1
    
    print(f"   Verified users: {verified_count}/{len(all_users)}")
    
    # Analyze posts
    all_posts = storage.find_nodes_by_label("Post")
    total_likes = 0
    for post_id in all_posts:
        post = storage.get_node(post_id)
        if post:
            total_likes += post['properties'].get('likes', 0)
    
    print(f"   Total likes across all posts: {total_likes}")
    print(f"   Average likes per post: {total_likes / len(all_posts):.1f}")

if __name__ == "__main__":
    main()

