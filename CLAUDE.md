# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

mdbook-lint is a fast markdown linter for mdBook projects written in Rust. It provides both CLI and
mdBook preprocessor functionality with 55 standard markdown rules (MD001-MD060) plus 18 mdBook-specific
rules (MDBOOK001-MDBOOK025) and 5 content quality rules (CONTENT001-CONTENT005).

## Essential Commands

### Pre-commit checks (always run before committing)

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings  
cargo test --lib --all-features
cargo test --test '*' --all-features
```

### Build and Test

```bash
cargo build --release
cargo test --lib test_name                    # Run specific unit test
cargo test --test integration_test_name       # Run specific integration test
cargo test -- --nocapture                     # Show test output
```

### Rule Testing

```bash
# Test specific rule in isolation
cargo test --lib md001

# Test rule with CLI
./target/release/mdbook-lint lint --enable MD001 test.md
./target/release/mdbook-lint lint --disable MD001,MD002 *.md

# Test with configuration
echo "[rules]
default = false
[rules.enabled]
MD001 = true" > test.toml
./target/release/mdbook-lint lint -c test.toml file.md
```

### Performance Debugging

```bash
# Build with release optimizations
cargo build --release

# Test performance with increasing file counts
./target/release/mdbook-lint lint file{1..10}.md
./target/release/mdbook-lint lint file{1..20}.md

# Profile on macOS
./target/release/mdbook-lint lint large-file.md &
PID=$!
sample $PID 5 -file profile.txt
kill $PID
```

## Architecture

### Workspace Structure

- `crates/mdbook-lint-core/` - Core linting engine (Document, LintEngine, PluginRegistry)
- `crates/mdbook-lint-cli/` - CLI binary and mdBook preprocessor
- `crates/mdbook-lint-rulesets/` - Rule implementations (StandardRuleProvider, MdBookRuleProvider)

### Core Flow

1. **Document Processing**: `Document` struct parses markdown via `comrak` AST
2. **Rule System**: Rules implement `Rule` trait with `lint_ast()` method
3. **Plugin Architecture**: `RuleProvider` trait allows pluggable rule sets
4. **Configuration**: Supports TOML/YAML/JSON via `.mdbook-lint.toml`
5. **Fix System**: Rules can provide `Fix` objects with byte positions for auto-correction

### Key Components

- `Document` (`core/src/document.rs`) - Markdown file representation with AST
- `LintEngine` (`core/src/engine.rs`) - Orchestrates linting across all rules
- `RuleRegistry` (`core/src/registry.rs`) - Manages active rules
- `Violation` (`core/src/violation.rs`) - Represents lint issues with location/severity
- `Config` (`cli/src/config.rs`) - Configuration parsing and merging
- `Preprocessor` (`cli/src/preprocessor.rs`) - mdBook integration

### Configuration Precedence

1. Configuration file (`.mdbook-lint.toml`)
2. mdBook preprocessor config (`book.toml`)
3. Environment variables (`MDBOOK_LINT_*`)
4. CLI arguments

## Adding New Rules

1. Create rule file in `crates/mdbook-lint-rulesets/src/rules/standard/md060.rs`
2. Implement `Rule` trait:

```rust
pub struct Md060;
impl Rule for Md060 {
    fn id(&self) -> &str { "MD060" }
    fn description(&self) -> &str { "Rule description" }
    fn lint_ast(&self, ast: &[Node],_src: &str) -> Result<Vec<Violation>> {

        // Implementation
    }
}
```

1. Register in provider (`crates/mdbook-lint-rulesets/src/standard_provider.rs`)

2. Add tests in same file with `#[cfg(test)]`

3. Update documentation in `docs/src/rules-reference.md`

## Performance Critical Patterns

### Avoid

- Regex compilation in loops
- O(n²) substring searching
- Repeated AST traversals
- String allocations in hot paths

### Prefer

- Single-pass AST processing
- Pre-compiled regex stored as static/const
- Iterator-based pattern matching
- Byte offset calculations over line/column lookups

## Release Process

### Current Issues (as of v0.11.5)

- **Partially Fixed**: Workspace dependencies now use workspace references (fixed in #194)
- **Still Broken**: release-plz publishes to crates.io but doesn't create GitHub release/tag
- **Workaround Required**: Manual tag and release creation after crates.io publishing
- See issues #191 and #195 for ongoing work

### Manual Release Fallback (Still Required)

```bash
# After release-plz publishes to crates.io, manually create tag/release:
VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
git tag v$VERSION
git push origin v$VERSION
gh release create v$VERSION --title "v$VERSION" --notes "..." --latest
```

### Recent Releases

- v0.11.5: Published to crates.io, manual tag/release creation required
- v0.11.4: Same issue, manual intervention required

### Release Configuration

- Config: `release-plz.toml`
- Workflow: `.github/workflows/release-plz.yml`
- Binary builds: `.github/workflows/release-binaries.yml` (triggers on release creation)

## Testing Strategy

### Test Categories

- Unit tests: In-file with `#[cfg(test)]`
- Integration tests: `tests/` directory
- Corpus tests: Real-world projects in `tests/corpus/`
- Performance tests: `tests/simple_performance_tests.rs`
- Edge cases: `tests/corpus/edge_cases/`

### Key Test Files

- `fix_functionality_tests.rs` - Auto-fix feature testing
- `mdbook_integration_tests.rs` - mdBook preprocessor tests
- `performance_regression_test.rs` - Performance regression detection
- `ruleset_scenarios_test.rs` - Rule interaction testing

## mdBook Preprocessor Mode

The preprocessor reads from stdin and writes to stdout:

```bash
# Test with minimal input
cargo run --bin mdbook-lint -- preprocessor < crates/mdbook-lint-cli/tests/fixtures/mdbook/minimal_input.json
```

Key behaviors:

- Preserves mdBook's JSON structure
- Errors don't break build unless `fail-on-warnings = true`
- Configuration discovery from `book.toml` location
- Must recreate engine after loading configuration

## Configuration Formats

### TOML with `[rules]` section

```toml
[rules]
default = false
[rules.enabled]
MD001 = true
MD009 = true
```

### Traditional format

```toml
disabled-rules = ["MD013", "MD033"]
enabled-rules = ["MD001"]
```

### Rule-specific configuration

```toml
[MD007]
indent = 4

[MD009]
br_spaces = 2
```

## Common Issues and Solutions

### Issue: Rules run despite `default = false`

- Fixed in v0.11.4 by parsing TOML in two stages
- Check `Config::from_toml_str()` in `cli/src/config.rs`

### Issue: Fix only works for MD009

- Fixed in v0.11.4 by using actual Fix objects
- See `apply_fixes_to_content()` in `cli/src/main.rs`

### Issue: Wrong configuration pattern in docs

- Fixed in v0.11.5 - use `[MD009]` not `[rules.MD009]` for rule config
- The `[rules]` section is only for default/enabled/disabled settings
- Over 130 documentation instances were corrected

### Issue: Performance degradation with many files

- Check for O(n²) patterns in rule implementation
- Use performance profiling commands above
- Common culprits: MD049, MD051 with pathological input

## Error Handling

- Library (`mdbook-lint-core`): Uses `thiserror` for typed errors
- CLI (`mdbook-lint-cli`): Uses `anyhow` for error context
- Rules return `Result<Vec<Violation>>` for graceful failure
- Preprocessor catches panics to prevent breaking mdBook builds
