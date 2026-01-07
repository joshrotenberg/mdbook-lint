# Rustdoc Linting

The `rustdoc` subcommand extracts and lints module-level documentation comments (`//!`) from
Rust source files. This helps maintain high-quality documentation in your Rust crates.

## Basic Usage

```bash
# Lint a single file
mdbook-lint rustdoc src/lib.rs

# Lint all Rust files in a directory (recursive)
mdbook-lint rustdoc src/

# Lint the entire crate
mdbook-lint rustdoc .
```

## How It Works

The rustdoc command:

1. Finds all `.rs` files in the specified paths (recursively for directories)
2. Extracts module-level documentation (`//!` comments) from each file
3. Converts the documentation to markdown
4. Lints the markdown using standard rules
5. Maps violation line numbers back to the original source locations

### What Gets Extracted

Only module-level documentation comments are extracted:

```rust
//! This line IS extracted (module-level doc)
//! This line IS extracted too

/// This line is NOT extracted (item-level doc)
fn example() {}

// This line is NOT extracted (regular comment)
```

The extraction stops when it encounters:

- A regular comment (`//`)
- Any non-comment code
- End of file

## Default Disabled Rules

Some rules are disabled by default for rustdoc because they don't apply well to
documentation comments:

| Rule | Name | Reason |
|------|------|--------|
| MD041 | first-line-heading | Rustdoc often starts with a description, not a heading |
| MD047 | trailing-newline | Doc comments don't have trailing newlines |
| MD025 | single-h1 | Rustdoc idiomatically uses multiple `#` sections |

You can re-enable these rules if needed:

```bash
mdbook-lint rustdoc --enable MD041,MD025 src/
```

## Options

The rustdoc command supports most of the same options as `lint`:

```bash
# Use a specific configuration file
mdbook-lint rustdoc --config .mdbook-lint.toml src/

# Disable specific rules
mdbook-lint rustdoc --disable MD013,MD033 src/

# Enable only specific rules
mdbook-lint rustdoc --enable MD001,MD003,MD018 src/

# Output as JSON
mdbook-lint rustdoc --output json src/

# Fail on warnings (useful in CI)
mdbook-lint rustdoc --fail-on-warnings src/

# Verbose output showing which files are checked
mdbook-lint rustdoc --verbose src/
```

## Directory Handling

When given a directory, the command:

- Recursively finds all `.rs` files
- Skips hidden directories (starting with `.`)
- Skips the `target/` directory
- Processes files in parallel for performance

```bash
# This will skip .git/, target/, and any hidden directories
mdbook-lint rustdoc .
```

## Example Output

```text
warning[MD018]: No space after hash on atx style heading
  --> src/lib.rs:5:3
     |
   5 | //! ##Bad heading
     |   ^ no-missing-space-atx

warning[MD032]: Lists should be surrounded by blank lines
  --> src/parser.rs:12:1
     |
  12 | //! - First item
     | ^^^ blanks-around-lists

Found: 2 warning(s)
```

Note how the line numbers point to the actual source file locations, making it easy to
find and fix issues.

## CI Integration

### GitHub Actions

```yaml
- name: Lint rustdoc
  run: mdbook-lint rustdoc --fail-on-warnings --output github src/
```

The `--output github` format produces GitHub Actions annotations that appear inline in
pull request diffs.

### Generic CI

```yaml
- name: Lint rustdoc
  run: mdbook-lint rustdoc --fail-on-warnings .
```

## Common Patterns

### Linting Before Publishing

Add to your CI pipeline to catch documentation issues before publishing:

```bash
# In your CI script
cargo fmt --check
cargo clippy -- -D warnings
cargo test
mdbook-lint rustdoc --fail-on-warnings .
cargo publish --dry-run
```

### Workspace Projects

For Cargo workspaces, lint each crate:

```bash
mdbook-lint rustdoc crates/
```

Or lint specific crates:

```bash
mdbook-lint rustdoc crates/core/src crates/cli/src
```

### Combining with Regular Linting

If you have both a Rust library and an mdBook:

```bash
# Lint the book
mdbook-lint lint docs/

# Lint the rustdoc
mdbook-lint rustdoc src/
```

## Limitations

- Only extracts `//!` (module-level) documentation, not `///` (item-level)
- Does not parse doc attributes (`#![doc = "..."]`)
- Code blocks inside documentation are not validated for correctness
  (use `cargo test --doc` for that)

## Tips for Better Rustdoc

1. **Use headings consistently**: Stick to conventional sections like `# Examples`,
   `# Panics`, `# Errors`, `# Safety`

2. **Include code examples**: They serve as both documentation and tests

   ```rust
   //! # Examples
   //!
   //! ```rust
   //! let result = my_function(42);
   //! assert_eq!(result, 84);
   //! ```
   ```

3. **Keep line lengths reasonable**: Long lines in doc comments are hard to read in source

4. **Use proper markdown**: Lists need blank lines around them, headings need proper
   formatting

## Next Steps

- See [CLI Usage](./cli-usage.md) for all command options
- See [Configuration](./configuration.md) for configuring rules
- See [CI vs Preprocessor](./ci-vs-preprocessor.md) for CI best practices
