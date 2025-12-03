# Link Rules

These rules ensure proper link formatting and validation in markdown documents.

## Rules in This Category

### Auto-fix Available ✓

- **[MD034](./md034.html)** - Bare URL used

### Other Link Rules

- **MD011** - Reversed link syntax
- **MD039** - Spaces inside link text
- **MD042** - No empty links
- **MD051** - Link fragments are valid
- **MD052** - Reference links and images should use a label
- **MD053** - Link and image reference definitions should be needed
- **MD054** - Link and image style
- **MD059** - Link and image reference style

## Why Link Rules Matter

Proper link formatting:

- Ensures links are clickable in all renderers
- Improves accessibility with descriptive text
- Maintains consistent link style
- Prevents broken references
- Enhances document navigation

## Common Link Formats

### Inline Links

```markdown
[Link text](https://example.com)
[Relative link](./other-page.md)
[Anchor link](#section-heading)
```

### Reference Links

```markdown
[Link text][reference]
[Another link][1]

[reference]: https://example.com
[1]: ./other-page.md
```

### Autolinks

```markdown
<https://example.com>
<user@example.com>
```

## Quick Configuration

```toml
# .mdbook-lint.toml

# No configuration for MD034 - it auto-fixes bare URLs

# Disable specific link rules
disabled_rules = ["MD051", "MD052"]
```

## Best Practices

1. **Use descriptive link text**: Avoid "click here" or "link"
2. **Prefer relative paths**: For internal documentation links
3. **Check anchors**: Ensure heading anchors exist
4. **Use reference style**: For frequently used URLs
5. **Wrap bare URLs**: Use `<URL>` syntax or proper links

## Related Categories

- [mdBook Rules](../mdbook/index.html) - mdBook-specific link validation
- [Style Rules](./style.html) - General formatting rules
