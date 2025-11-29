# PyRustTest - Comprehensive DeepGraph Python Bindings Test Suite

## Overview

This directory contains comprehensive tests for all DeepGraph Python bindings, covering edge cases, corner cases, and error scenarios.

## Test Structure

| Test File | Features Tested | Test Count |
|-----------|----------------|------------|
| `test_1_core_operations.py` | Core Storage (20 methods) | 50+ tests |
| `test_2_transactions.py` | Transaction Manager (3 methods) | 25+ tests |
| `test_3_indexing.py` | Index Manager (3 methods) | 30+ tests |
| `test_4_durability.py` | WAL & Recovery (3 methods) | 20+ tests |
| `test_5_query_language.py` | Cypher Query System (5 methods) | 25+ tests |
| `test_6_concurrency.py` | MVCC & Deadlock (7 methods) | 30+ tests |
| `test_7_algorithms.py` | Graph Algorithms (8 functions) | 40+ tests |
| `test_8_integration.py` | End-to-end workflows | 20+ tests |

**Total**: 240+ comprehensive tests

## Running Tests

### Run All Tests
```bash
python -m pytest PyRustTest/ -v
```

### Run Individual Test Files
```bash
python PyRustTest/test_1_core_operations.py
python PyRustTest/test_2_transactions.py
python PyRustTest/test_3_indexing.py
# ... etc
```

### Run with Coverage
```bash
pytest PyRustTest/ --cov=deepgraph --cov-report=html
```

## Test Coverage

Each test suite covers:
- ✅ **Happy path** - Normal, expected usage
- ✅ **Edge cases** - Boundary conditions, empty inputs
- ✅ **Error cases** - Invalid inputs, missing data
- ✅ **Corner cases** - Unusual but valid scenarios
- ✅ **Performance** - Large datasets, stress tests
- ✅ **Concurrency** - Multi-threaded scenarios

## Prerequisites

```bash
# Install DeepGraph with Python bindings
cd /path/to/deepgraph
maturin develop --release --features python

# Install test dependencies
pip install pytest pytest-cov pytest-timeout
```

## Test Philosophy

1. **Comprehensive** - Test every method with multiple scenarios
2. **Isolated** - Each test is independent
3. **Clear** - Descriptive test names and assertions
4. **Fast** - Quick feedback loop (< 30 seconds total)
5. **Documented** - Comments explain what's being tested

## Contributing

When adding new features to DeepGraph:
1. Add corresponding tests to the appropriate test file
2. Ensure edge cases are covered
3. Run the full test suite before committing
4. Update this README if adding new test files

---

**DeepGraph** - Production-Ready Graph Database  
© 2025 DeepSkilling. Licensed under MIT.

