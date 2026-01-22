#!/usr/bin/env python3
"""
Test script for DeepGraph REPL/CLI
"""

import subprocess
import os
import sys
import time

def test_cli_help():
    """Test CLI --help"""
    print("\n" + "="*60)
    print("TEST 1: CLI --help")
    print("="*60)
    result = subprocess.run(
        ["./target/release/deepgraph-cli", "--help"],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    assert result.returncode == 0, "CLI help should succeed"
    assert "DeepGraph" in result.stdout, "Help should mention DeepGraph"
    print("✅ PASS")

def test_single_query():
    """Test non-interactive single query"""
    print("\n" + "="*60)
    print("TEST 2: Single Query (Non-Interactive)")
    print("="*60)
    
    # Create a test database
    db_path = "./test_cli_db"
    if os.path.exists(db_path):
        import shutil
        shutil.rmtree(db_path)
    
    # Run a query
    query = "MATCH (n) RETURN n"
    result = subprocess.run(
        ["./target/release/deepgraph-cli", "--database", db_path, "-q", query],
        capture_output=True,
        text=True,
        timeout=5
    )
    
    print(result.stdout)
    if result.stderr:
        print("STDERR:", result.stderr)
    
    assert result.returncode == 0, "Query should succeed"
    assert "(no results)" in result.stdout or "row(s)" in result.stdout, "Should show results"
    print("✅ PASS")

def test_json_output():
    """Test JSON output format"""
    print("\n" + "="*60)
    print("TEST 3: JSON Output Format")
    print("="*60)
    
    db_path = "./test_cli_db"
    query = "MATCH (n) RETURN n"
    
    result = subprocess.run(
        ["./target/release/deepgraph-cli", "--database", db_path, "-q", query, "--output", "json"],
        capture_output=True,
        text=True,
        timeout=5
    )
    
    print(result.stdout)
    assert result.returncode == 0, "Query should succeed"
    # Empty database will return []
    assert "[" in result.stdout, "JSON output should contain array"
    print("✅ PASS")

def test_csv_output():
    """Test CSV output format"""
    print("\n" + "="*60)
    print("TEST 4: CSV Output Format")
    print("="*60)
    
    db_path = "./test_cli_db"
    query = "MATCH (n) RETURN n"
    
    result = subprocess.run(
        ["./target/release/deepgraph-cli", "--database", db_path, "-q", query, "--output", "csv"],
        capture_output=True,
        text=True,
        timeout=5
    )
    
    print(result.stdout)
    assert result.returncode == 0, "Query should succeed"
    print("✅ PASS")

def test_import_csv():
    """Test CSV import"""
    print("\n" + "="*60)
    print("TEST 5: CSV Import")
    print("="*60)
    
    # Create test CSV files
    nodes_csv = "./test_nodes.csv"
    edges_csv = "./test_edges.csv"
    
    with open(nodes_csv, 'w') as f:
        f.write("id,name,age,labels\n")
        f.write("1,Alice,30,Person\n")
        f.write("2,Bob,25,Person\n")
    
    with open(edges_csv, 'w') as f:
        f.write("source,target,type,since\n")
        f.write("1,2,KNOWS,2020\n")
    
    db_path = "./test_import_db"
    if os.path.exists(db_path):
        import shutil
        shutil.rmtree(db_path)
    
    result = subprocess.run(
        [
            "./target/release/deepgraph-cli",
            "--database", db_path,
            "--import-csv-nodes", nodes_csv,
            "--import-csv-edges", edges_csv
        ],
        capture_output=True,
        text=True,
        timeout=10
    )
    
    print(result.stdout)
    if result.stderr:
        print("STDERR:", result.stderr)
    
    assert result.returncode == 0, "Import should succeed"
    assert "Imported" in result.stdout, "Should show import stats"
    
    # Clean up
    os.remove(nodes_csv)
    os.remove(edges_csv)
    print("✅ PASS")

def test_query_after_import():
    """Test querying data after import"""
    print("\n" + "="*60)
    print("TEST 6: Query After Import")
    print("="*60)
    
    db_path = "./test_import_db"
    query = "MATCH (n:Person) RETURN n.name, n.age"
    
    result = subprocess.run(
        ["./target/release/deepgraph-cli", "--database", db_path, "-q", query],
        capture_output=True,
        text=True,
        timeout=5
    )
    
    print(result.stdout)
    assert result.returncode == 0, "Query should succeed"
    assert "Alice" in result.stdout or "Bob" in result.stdout or "row(s)" in result.stdout, "Should show imported data"
    print("✅ PASS")

def cleanup():
    """Clean up test files"""
    import shutil
    for path in ["./test_cli_db", "./test_import_db"]:
        if os.path.exists(path):
            shutil.rmtree(path)
    print("\n✅ Cleanup complete")

def main():
    print("\n" + "="*60)
    print("DeepGraph REPL/CLI Test Suite")
    print("="*60)
    
    # Build first
    print("\nBuilding CLI...")
    result = subprocess.run(
        ["cargo", "build", "--release", "--bin", "deepgraph-cli"],
        capture_output=True,
        text=True
    )
    if result.returncode != 0:
        print("❌ Build failed:")
        print(result.stderr)
        sys.exit(1)
    print("✅ Build complete")
    
    try:
        test_cli_help()
        test_single_query()
        test_json_output()
        test_csv_output()
        test_import_csv()
        test_query_after_import()
        
        print("\n" + "="*60)
        print("ALL TESTS PASSED! ✅")
        print("="*60)
        print("\nREPL/CLI Features Verified:")
        print("  ✅ CLI argument parsing")
        print("  ✅ Single query execution")
        print("  ✅ Table output format")
        print("  ✅ JSON output format")
        print("  ✅ CSV output format")
        print("  ✅ CSV import (nodes & edges)")
        print("  ✅ Query execution after import")
        print("\nInteractive REPL Mode:")
        print("  Run: ./target/release/deepgraph-cli")
        print("  Or:  ./target/release/deepgraph-cli --database mydb.db")
        
    except AssertionError as e:
        print(f"\n❌ TEST FAILED: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ ERROR: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
    finally:
        cleanup()

if __name__ == "__main__":
    main()
