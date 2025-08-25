# Testing Strategy

This document describes mdbook-lint's comprehensive testing approach, including our simplified corpus testing and performance validation framework.

## Overview

mdbook-lint uses a multi-layered testing strategy designed for speed, reliability, and comprehensive coverage:

- **Unit Tests**: Fast, focused tests for individual components
- **Integration Tests**: End-to-end testing of CLI and preprocessor functionality
- **Corpus Tests**: Real-world validation using diverse markdown content
- **Performance Tests**: Regression testing for known performance issues
- **Property Tests**: Fuzzing-style tests ensuring stability on any input

## Corpus Testing

Our corpus testing validates correctness and stability using carefully selected real-world content.

### Essential Corpus Files

Located in `tests/corpus/essential/`, these files test key scenarios:

- **`empty_file.md`**: Edge case handling for empty files
- **`unicode_content.md`**: International text and special characters
- **`large_file.md`**: Performance testing with substantial content
- **`mixed_line_endings.md`**: Cross-platform line ending handling
- **`known_violations.md`**: Files with intentional violations for rule validation

### Property-Based Testing

We use property-based testing to ensure mdbook-lint never crashes on any input:

```rust
// Example: Test with various malformed markdown
let malformed_cases = [
    "[unclosed link",
    "**unclosed emphasis", 
    "`unclosed code",
    "```\nunclosed code block",
];

for case in &malformed_cases {
    assert_no_crash(case, &format!("Malformed: {}", case));
}
```

This approach tests:
- Random/invalid UTF-8 sequences
- Malformed markdown syntax
- Pathological nesting patterns
- Mixed valid/invalid content
- Binary-like content

## Performance Testing

Performance tests prevent regressions in known problem areas and ensure consistent speed.

### Regression Tests

Located in `simple_performance_tests.rs`, these tests target historical issues:

#### MD051 Performance Fix
Tests the O(n²) → O(n) optimization for HTML fragment validation:

```rust
// Previously caused exponential slowdown
let html_content = r##"
<a href="#section1">Link 1</a>
<a href="#section2">Link 2</a>
## Section 1 {#section1}
## Section 2 {#section2}
"##;

assert_completes_quickly(&document, Duration::from_millis(100));
```

#### MD049 Infinite Loop Fix
Tests the emphasis parsing fix that prevented infinite loops:

```rust
// Previously caused hangs with patterns like wrapping_*
let emphasis_content = r##"
- `wrapping_*` function calls in code
- `checked_*` operations in backticks
- Normal *emphasis* outside code should work
"##;

assert_completes_quickly(&document, Duration::from_millis(50));
```

### Performance Targets

- **Small files** (< 1KB): Complete in < 50ms
- **Medium files** (1KB-10KB): Complete in < 100ms  
- **Large files** (> 10KB): Complete in < 500ms
- **Pathological cases**: Complete in < 200ms

## Test Execution

### Local Testing

Run the full test suite:

```bash
# All tests
cargo test --all-features

# Specific test categories
cargo test --test simple_corpus_tests      # Corpus validation
cargo test --test simple_performance_tests # Performance regressions
cargo test --lib                          # Unit tests
```

### CI Testing

Our CI runs tests across platforms:

- **Ubuntu, macOS, Windows**: Cross-platform compatibility
- **Stable, Beta Rust**: Forward compatibility
- **Essential corpus tests**: Real-world validation
- **Performance benchmarks**: Regression detection

## Design Philosophy

### Simplified Approach

We prioritize **accuracy and stability over absolute benchmarks** because mdbook-lint is already fast compared to other tools. Our testing focuses on:

1. **Correctness**: Rules work as intended
2. **Stability**: Never crash on any input
3. **Performance**: Prevent regressions
4. **Maintainability**: Simple, focused tests

### Property Testing Benefits

Property-based testing provides confidence that mdbook-lint handles the unpredictable nature of real-world markdown:

- User-generated content with encoding issues
- Generated markdown from various tools
- Partially corrupted files
- Mixed content types

### Performance Testing Strategy

Rather than complex benchmarking, we use targeted regression tests:

- **Known issues**: Test specific problems that were fixed
- **Pathological inputs**: Ensure reasonable performance on edge cases
- **Real content**: Validate speed on actual documentation

## Historical Context

This testing approach was simplified from a more complex framework that included:

- External corpus downloads (The Rust Book, etc.)
- markdownlint compatibility testing
- Extensive generated edge cases
- Complex nightly CI workflows

The current approach maintains essential coverage while being:
- **10x faster** to run locally
- **No external dependencies**
- **Easier to understand and maintain**
- **More reliable** in CI environments

## Contributing to Tests

When adding new functionality:

1. **Add unit tests** for the specific feature
2. **Add integration tests** if it affects CLI/preprocessor behavior
3. **Add corpus tests** if it might affect stability
4. **Add performance tests** if it's performance-sensitive

For bug fixes:
1. **Add a regression test** that would have caught the bug
2. **Ensure it fails** before your fix
3. **Verify it passes** after your fix

See [Contributing](./contributing.md) for detailed guidelines.