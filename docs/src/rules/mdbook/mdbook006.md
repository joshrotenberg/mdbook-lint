# MDBOOK006 - Invalid Cross-Reference Links

**Severity**: Error  
**Category**: mdBook-specific  
**Auto-fix**: Not available

## Rule Description

This rule validates internal cross-reference links that point to headings in other files. It ensures that both the target file exists and contains the referenced heading anchor.

## Why This Rule Exists

Valid cross-references are crucial because:
- Ensures readers can navigate between related sections
- Prevents broken links in published documentation
- Maintains documentation integrity across chapters
- Enables proper mdBook navigation features
- Helps identify outdated references after refactoring

## Examples

### ❌ Incorrect (violates rule)

```markdown
<!-- Target heading doesn't exist -->
See [configuration details](./config.md#database-setup)

<!-- File exists but anchor is wrong -->
Check the [API reference](./api.md#rest-endpoints)

<!-- Typo in anchor -->
Read about [authentication](./auth.md#authentification)
```

### ✅ Correct

```markdown
<!-- Valid file and anchor -->
See [configuration details](./config.md#database-configuration)

<!-- Correct anchor format -->
Check the [API reference](./api.md#rest-api-endpoints)

<!-- Valid cross-reference -->
Read about [authentication](./auth.md#authentication)
```

## What This Rule Checks

1. **File existence**: Target `.md` file must exist
2. **Heading presence**: Referenced heading must exist in target file
3. **Anchor format**: Anchor must match mdBook's heading ID generation
4. **Case sensitivity**: Anchors are case-insensitive but should match

### Anchor Generation Rules

mdBook generates anchors from headings:
- Convert to lowercase
- Replace spaces with hyphens
- Remove special characters (except hyphens and underscores)
- Handle duplicate anchors with numeric suffixes

```markdown
## Hello World!           → #hello-world
## User's Guide          → #users-guide
## API Reference (v2)    → #api-reference-v2
## 1.2.3 Version Notes   → #123-version-notes
```

## Configuration

```toml
[rules.MDBOOK006]
check_external = false   # Don't check external URLs (default: false)
ignore_missing = false   # Report missing files (default: false)
case_sensitive = false   # Case-sensitive anchor matching (default: false)
```

## Common Issues and Solutions

### Issue: Heading Text Changed
When heading text changes, anchors break:

```markdown
<!-- Original heading in config.md -->
## Database Setup

<!-- Link that breaks after heading change -->
[See database setup](./config.md#database-setup)

<!-- After heading renamed to "Database Configuration" -->
[See database configuration](./config.md#database-configuration)
```

**Solution**: Update all cross-references when changing headings

### Issue: Special Characters in Headings
Special characters affect anchor generation:

```markdown
<!-- Heading with special characters -->
## What's New? (2024)

<!-- Wrong anchor -->
[What's new](./changelog.md#whats-new-2024)  ✗

<!-- Correct anchor -->
[What's new](./changelog.md#whats-new-2024)   ✓
```

### Issue: Duplicate Headings
Multiple headings with same text get numeric suffixes:

```markdown
<!-- In api.md -->
## Authentication
...
## Authentication  <!-- Gets #authentication-1 -->

<!-- Linking to second occurrence -->
[Second auth section](./api.md#authentication-1)
```

## When to Disable

Consider disabling this rule if:
- You're in the middle of major documentation restructuring
- Your build process generates files dynamically
- You use external tools that create anchors differently
- You have many legacy cross-references to update

### Disable in Config

```toml
# .mdbook-lint.toml
disabled_rules = ["MDBOOK006"]
```

### Disable Inline

```markdown
<!-- mdbook-lint-disable MDBOOK006 -->
[Temporarily broken cross-reference](./future.md#todo-section)
<!-- mdbook-lint-enable MDBOOK006 -->
```

## Best Practices

1. **Use descriptive anchors**: Make headings clear and unique
2. **Test after refactoring**: Verify links after changing headings
3. **Maintain a link registry**: Document important cross-references
4. **Use consistent heading style**: Makes anchors predictable
5. **Avoid special characters**: Simplifies anchor generation

### Cross-Reference Patterns

```markdown
<!-- Good: Clear, specific references -->
[See configuration options](./config.md#available-options)
[Authentication guide](./auth.md#oauth2-setup)
[API error codes](./api.md#error-handling)

<!-- Avoid: Vague or fragile references -->
[See here](./config.md#options)
[More info](./auth.md#setup)
[Errors](./api.md#errors)
```

## Tools and Tips

### Finding Broken Cross-References

```bash
# Run linter to find all cross-reference issues
mdbook-lint lint --rules MDBOOK006 docs/

# Test all links with mdBook
mdbook test
```

### Generating Anchor Lists

```bash
# List all headings and their anchors
grep "^#" docs/**/*.md | sed 's/#\+//'
```

## Related Rules

- [MDBOOK002](./mdbook002.html) - Invalid internal link
- [MD051](../standard/md051.html) - Link fragments are valid
- [MD052](../standard/md052.html) - Reference links and images

## References

- [mdBook - Links and References](https://rust-lang.github.io/mdBook/format/markdown.html#links)
- [CommonMark - Links](https://spec.commonmark.org/0.30/#links)