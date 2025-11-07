# Contributing to DeepGraph

Thank you for your interest in contributing to DeepGraph! This document provides guidelines and information for contributors.

## Development Setup

### Prerequisites
- Rust 1.70 or higher
- Cargo
- Git
- A text editor or IDE with Rust support

### Setting Up Your Development Environment

1. Clone the repository:
```bash
git clone <repository-url>
cd deepgraph
```

2. Build the project:
```bash
cargo build
```

3. Run tests to ensure everything works:
```bash
cargo test
```

4. Run the demo:
```bash
cargo run --bin deepgraph-cli
```

## Development Workflow

### Making Changes

1. Create a feature branch:
```bash
git checkout -b feature/your-feature-name
```

2. Make your changes, following the code style guidelines below.

3. Write tests for your changes:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_your_feature() {
        // Your test code
    }
}
```

4. Run tests and ensure they pass:
```bash
cargo test
```

5. Run benchmarks if you've made performance-related changes:
```bash
cargo bench
```

6. Commit your changes with a clear message:
```bash
git commit -m "Add feature: description of your changes"
```

## Code Style Guidelines

### Rust Style
- Follow the official Rust style guide
- Use `rustfmt` to format your code:
```bash
cargo fmt
```

- Use `clippy` to catch common mistakes:
```bash
cargo clippy
```

### Naming Conventions
- Use snake_case for functions and variables
- Use PascalCase for types and traits
- Use SCREAMING_SNAKE_CASE for constants
- Use descriptive names that convey intent

### Documentation
- Document all public APIs with doc comments (///)
- Include examples in documentation when helpful:
```rust
/// Creates a new node with the given labels.
///
/// # Examples
///
/// ```
/// let node = Node::new(vec!["Person".to_string()]);
/// ```
pub fn new(labels: Vec<String>) -> Self {
    // implementation
}
```

### Error Handling
- Use the `Result` type for operations that can fail
- Create specific error variants for different failure modes
- Provide helpful error messages

### Testing
- Write unit tests for individual functions
- Write integration tests for complex workflows
- Use descriptive test names: `test_operation_succeeds_when_condition`
- Test both success and failure cases
- Use property-based testing for complex logic (proptest)

## Project Structure

Understanding the codebase:

```
src/
├── lib.rs           # Public API and module declarations
├── graph.rs         # Core data structures (Node, Edge, Property)
├── storage.rs       # Storage engine implementation
├── parser.rs        # Query parser (placeholder in Phase 1)
├── transaction.rs   # Transaction management
├── error.rs         # Error types and handling
└── bin/
    └── cli.rs       # CLI demo application

tests/
└── integration_tests.rs  # Integration tests

benches/
└── graph_ops.rs     # Performance benchmarks
```

## Adding New Features

### Adding a New Module

1. Create the module file in `src/`
2. Add it to `lib.rs`:
```rust
pub mod your_module;
```
3. Export public types in `lib.rs` if needed:
```rust
pub use your_module::{YourType};
```
4. Write comprehensive tests
5. Update documentation

### Adding New Tests

Place tests in one of these locations:
- **Unit tests**: At the bottom of the module file in a `#[cfg(test)] mod tests` block
- **Integration tests**: In the `tests/` directory
- **Doc tests**: In documentation comments

### Adding Benchmarks

Add benchmarks to `benches/graph_ops.rs`:

```rust
fn bench_your_operation(c: &mut Criterion) {
    c.bench_function("your_operation", |b| {
        b.iter(|| {
            // Code to benchmark
            black_box(your_operation());
        });
    });
}

criterion_group!(benches, ..., bench_your_operation);
```

## Testing Guidelines

### Running Tests

```bash
# All tests
cargo test

# Specific module
cargo test storage

# With output
cargo test -- --nocapture

# Single test
cargo test test_name
```

### Test Coverage

Aim for high test coverage:
- Test happy paths
- Test error conditions
- Test edge cases
- Test concurrent access where applicable

### Benchmark Guidelines

When writing benchmarks:
- Use `black_box()` to prevent compiler optimizations
- Benchmark realistic workloads
- Document what you're measuring
- Compare with baseline before/after changes

## Performance Considerations

When contributing performance-sensitive code:
- Profile your changes
- Consider memory allocation patterns
- Use appropriate data structures
- Document performance characteristics
- Add benchmarks for critical paths

## Phase-Specific Guidelines

### Phase 1 (Current - Foundation)
Focus on:
- Correctness over optimization
- Clean, understandable APIs
- Comprehensive tests
- Good documentation

### Phase 2 (Core Features)
Will focus on:
- Query optimization
- Proper indexing
- ACID transaction implementation
- Performance improvements

### Future Phases
Will add:
- Advanced features (full-text search, vector indices)
- Production hardening
- Multi-language bindings

## Commit Message Guidelines

Use clear, descriptive commit messages:

```
Add feature: brief description

More detailed explanation if needed.
- Bullet points for specific changes
- Reference issue numbers if applicable

Fixes #123
```

Types of commits:
- `Add feature:` - New functionality
- `Fix:` - Bug fixes
- `Refactor:` - Code improvements without changing behavior
- `Test:` - Adding or updating tests
- `Docs:` - Documentation changes
- `Perf:` - Performance improvements
- `Chore:` - Maintenance tasks

## Review Process

When submitting changes:
1. Ensure all tests pass
2. Run `cargo fmt` and `cargo clippy`
3. Update documentation if needed
4. Add tests for new functionality
5. Provide a clear description of changes

## Questions and Help

If you need help:
- Check existing documentation
- Look at similar code in the project
- Review tests for usage examples
- Ask questions in issues or discussions

## Code of Conduct

- Be respectful and professional
- Welcome newcomers
- Focus on constructive feedback
- Help others learn and grow

## License

By contributing to DeepGraph, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to DeepGraph! Your efforts help make this project better for everyone.

