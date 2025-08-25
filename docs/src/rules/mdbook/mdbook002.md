# MDBOOK002 - Invalid Internal Link

**Severity**: Error  
**Category**: mdBook-specific  
**Auto-fix**: Not available

## Rule Description

This rule validates internal links within mdBook projects, ensuring they point to valid files and anchors. Broken internal links create a poor reading experience and navigation issues.

## Why This Rule Exists

Valid internal links are crucial because:
- Ensures readers can navigate between chapters
- Prevents 404 errors in generated documentation
- Maintains documentation integrity
- Enables proper mdBook navigation features
- Helps identify renamed or moved files

## Examples

### ❌ Incorrect (violates rule)

```markdown
<!-- Link to non-existent file -->
See [configuration](./configs.md) for details.

<!-- Link to non-existent anchor -->
Check the [installation section](./setup.md#install)

<!-- Broken relative path -->
Read more in [the guide](../guides/intro.md)
```

### ✅ Correct

```markdown
<!-- Valid file link -->
See [configuration](./configuration.md) for details.

<!-- Valid anchor link -->
Check the [installation section](./getting-started.md#installation)

<!-- Correct relative path -->
Read more in [the introduction](./introduction.md)

<!-- External links are not checked -->
Visit [Rust website](https://www.rust-lang.org)
```

## What This Rule Checks

1. **File existence**: Verifies linked `.md` files exist
2. **Anchor validity**: Confirms heading anchors are present
3. **Path resolution**: Validates relative paths from current file
4. **SUMMARY.md links**: Ensures all chapter links are valid

## Configuration

```toml
[rules.MDBOOK002]
check_anchors = true     # Validate heading anchors (default: true)
allow_external = true    # Skip external URLs (default: true)
check_images = false     # Also validate image paths (default: false)
```

## Common Issues and Solutions

### Issue: File Renamed
```markdown
<!-- Before -->
[Old name](./old-filename.md)

<!-- After -->
[New name](./new-filename.md)
```

### Issue: Heading Changed
```markdown
<!-- Heading changed from "## Installation" to "## Setup" -->
<!-- Before -->
[Install](./guide.md#installation)

<!-- After -->
[Install](./guide.md#setup)
```

### Issue: File Moved
```markdown
<!-- File moved to subdirectory -->
<!-- Before -->
[Guide](./guide.md)

<!-- After -->
[Guide](./user-guide/guide.md)
```

## When to Disable

Consider disabling this rule if:
- You're in the middle of a major restructuring
- Your build process generates files dynamically
- You have external link checking handled separately

### Disable in Config

```toml
# .mdbook-lint.toml
disabled_rules = ["MDBOOK002"]
```

### Disable Inline

```markdown
<!-- mdbook-lint-disable MDBOOK002 -->
[Temporarily broken link](./todo.md)
<!-- mdbook-lint-enable MDBOOK002 -->
```

## Tips for Compliance

1. **Use relative paths**: More maintainable than absolute paths
2. **Update links when renaming**: Use search and replace
3. **Test navigation**: Click through links after changes
4. **Use anchor generation tools**: Ensure correct anchor format

### Anchor Format

mdBook generates anchors from headings using these rules:
- Convert to lowercase
- Replace spaces with hyphens
- Remove special characters
- Handle duplicates with numbers

```markdown
## Hello World!     <!-- #hello-world -->
## User's Guide     <!-- #users-guide -->
## 1.2.3 Version    <!-- #123-version -->
```

## Related Rules

- [MD042](../standard/md042.html) - No empty links
- [MD051](../standard/md051.html) - Link fragments are valid
- [MDBOOK003](./mdbook003.html) - SUMMARY.md structure
- [MDBOOK006](./mdbook006.html) - Cross-reference validation

## References

- [mdBook Documentation - SUMMARY.md](https://rust-lang.github.io/mdBook/format/summary.html)
- [mdBook Link Format](https://rust-lang.github.io/mdBook/format/markdown.html#links)