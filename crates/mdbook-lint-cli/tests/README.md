# Integration Tests for mdbook-lint

This directory contains integration tests that verify mdbook-lint functionality in real-world scenarios.

## Test Structure

- `mdbook_integration_tests.rs` - Main integration test suite
- `common/mod.rs` - Shared test utilities and helpers
- `fixtures/` - Test data and sample files
  - `markdown/` - Sample markdown files for testing
  - `mdbook/` - mdBook preprocessor input fixtures

## Working Tests

### ✅ CLI Tests (All Working)
- `test_cli_with_clean_mdbook_content` - Clean content validation
- `test_cli_mdbook_only_flag` - mdBook-only rule filtering
- `test_cli_lint_mdbook_files` - Multi-file linting
- `test_cli_with_nested_directories` - Nested directory handling
- `test_directory_creation_fix` - Directory creation verification

### 🔄 Preprocessor Tests (Currently Ignored)
The preprocessor tests are currently disabled with `#[ignore]` due to mdBook JSON protocol format issues.

#### Known Issue: mdBook JSON Format
The preprocessor integration tests fail because the mdBook `PreprocessorContext` and `Book` types require specific JSON structure that's more complex than initially implemented.

**Error Pattern:**
```
Error: JSON error: missing field `__non_exhaustive` at line X column Y
```

**Root Cause:**
mdBook's internal types use Rust's `#[non_exhaustive]` attribute and have complex nested structures that are difficult to mock manually.

**Next Steps:**
1. Study mdBook's actual preprocessor JSON format by running a real preprocessor
2. Use mdBook's own test utilities or generate fixtures from real mdBook projects
3. Consider using mdBook's `mdbook test` command to generate authentic JSON
4. Alternative: Focus on end-to-end testing with actual mdBook projects

## Test Utilities

### `TempMdBook`
Helper for creating temporary mdBook project structures:
- ✅ **Directory Creation**: Fixed to handle nested paths properly
- ✅ **File Creation**: Works with complex directory structures
- ✅ **Configuration**: Supports custom preprocessor configs
- 🔄 **JSON Generation**: Needs mdBook format compatibility

### `ViolationExpectation`
System for asserting expected rule violations:
- ✅ **Exact Counts**: `ViolationExpectation::new("MD001", 2)`
- ✅ **Minimum Counts**: `ViolationExpectation::at_least("MDBOOK001", 1)`
- ✅ **Multiple Rules**: `verify_violations(&output, &[...])`

## Running Tests

```bash
# Run all integration tests
cargo test --test mdbook_integration_tests

# Run only CLI tests (all working)
cargo test --test mdbook_integration_tests test_cli

# Run specific test
cargo test --test mdbook_integration_tests test_cli_mdbook_only_flag
```

## Adding New Tests

1. **CLI Tests**: Use `cli_command()` helper with various arguments
2. **Fixture Tests**: Add files to `fixtures/markdown/` and use helpers
3. **Complex Scenarios**: Use `TempMdBook` for dynamic test structures

## Future Improvements

1. **Preprocessor Integration**: Resolve mdBook JSON format compatibility
2. **Performance Tests**: Add benchmarking for large projects
3. **Corpus Testing**: Add tests with real-world mdBook projects
4. **Error Scenarios**: Test malformed input handling