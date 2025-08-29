# mdBook-Specific Rules

These rules are specifically designed for mdBook projects, validating mdBook-specific syntax, conventions, and structure.

## Rules

| Rule ID | Name | Description |
|---------|------|-------------|
| [MDBOOK001](./mdbook001.md) | code-block-language | Code blocks should have language tags |
| [MDBOOK002](./mdbook002.md) | summary-structure | SUMMARY.md should follow mdBook structure |
| [MDBOOK003](./mdbook003.md) | internal-links | Internal links should be valid |
| [MDBOOK004](./mdbook004.md) | part-titles | Part titles should be formatted correctly |
| [MDBOOK005](./mdbook005.md) | chapter-paths | Chapter paths should be relative |
| [MDBOOK006](./mdbook006.md) | draft-chapters | Draft chapters should have content or be marked |
| [MDBOOK007](./mdbook007.md) | separator-syntax | Separator syntax should be correct |

## Why mdBook-Specific Rules?

mdBook extends standard Markdown with special features:

1. **SUMMARY.md Structure**: Defines book organization
2. **Include Syntax**: `\{{#include file.md}}`
3. **Playground Links**: `\{{#playground file.rs}}`
4. **Hidden Lines**: Lines starting with `#` in Rust code blocks
5. **Quiz Support**: Interactive quizzes in documentation
6. **Custom Renderers**: Different output formats

These rules ensure your mdBook project:
- Builds correctly
- Renders properly in all output formats
- Maintains consistent structure
- Follows mdBook best practices

## SUMMARY.md Structure

The `SUMMARY.md` file is the backbone of any mdBook project:

```markdown
# Summary

[Introduction](./introduction.md)

# User Guide

- [Installation](./guide/installation.md)
- [Getting Started](./guide/getting-started.md)
  - [Basic Usage](./guide/basic-usage.md)
  - [Advanced Usage](./guide/advanced-usage.md)

# Reference

- [Configuration](./reference/configuration.md)
- [API](./reference/api.md)

---

[Contributors](./contributors.md)
```

Rules MDBOOK002-MDBOOK007 validate various aspects of this structure.

## Common mdBook Issues

### Missing Language Tags

**Problem**: Code blocks without language tags don't get syntax highlighting.

```markdown
```
fn main() {
    println!("No highlighting!");
}
```
```

**Solution**: Always specify the language.

```markdown
```rust
fn main() {
    println!("Properly highlighted!");
}
```
```

### Broken Internal Links

**Problem**: Links to non-existent chapters break navigation.

```markdown
- [Missing Chapter](./does-not-exist.md)
```

**Solution**: Ensure all linked files exist.

### Invalid SUMMARY.md Format

**Problem**: Incorrect indentation or syntax breaks book generation.

```markdown
- [Chapter 1](./chapter1.md)
    - [Wrong indent](./sub.md)  # Should be 2 spaces, not 4
[Missing dash](./chapter2.md)    # Should be "- [...]"
```

**Solution**: Follow mdBook's SUMMARY.md conventions.

## Integration with CI/CD

Use these rules in your CI pipeline:

```yaml
# .github/workflows/mdbook.yml
name: mdBook Checks

on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run mdbook-lint
        run: |
          cargo install mdbook-lint
          mdbook-lint check
```

## Configuration

Enable only mdBook rules:

```toml
# .mdbook-lint.toml
[rules]
# Disable all standard rules
"MD*" = false

# Enable only mdBook rules
"MDBOOK*" = true
```

Or enable both sets:

```toml
[rules]
# Use defaults (all rules enabled)

# Customize specific mdBook rules
[MDBOOK001]
allow_missing = false
```

## Best Practices

1. **Run Checks Before Building**: Catch issues early
2. **Include in Pre-commit Hooks**: Prevent broken commits
3. **Document Exceptions**: If disabling rules, explain why
4. **Test Rendering**: Lint checks complement, not replace, build tests
5. **Version Control SUMMARY.md**: Track structure changes

## Related Standard Rules

Some standard rules are particularly relevant for mdBook:

- [MD041](../standard/md041.md) - First line should be a heading (important for chapters)
- [MD025](../standard/md025.md) - Single H1 (one main heading per chapter)
- [MD051](../standard/md051.md) - Link fragments (for cross-references)

## References

- [mdBook Documentation](https://rust-lang.github.io/mdBook/)
- [SUMMARY.md Format](https://rust-lang.github.io/mdBook/format/summary.html)
- [mdBook Specific Markdown](https://rust-lang.github.io/mdBook/format/mdbook.html)