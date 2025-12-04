# MDBOOK006 - Internal Cross-References

Internal cross-reference links must point to valid headings in target files.

## Why This Rule Exists

Cross-references between chapters using anchor fragments must resolve to actual
headings in the target file. Invalid fragments create broken navigation.

## Examples

### Incorrect

```markdown
See the [configuration section](./config.md#settings) for details.
```

Where `config.md` has no `## Settings` heading.

### Correct

```markdown
See the [configuration section](./config.md#configuration-options) for details.
```

Where `config.md` contains:

```markdown
## Configuration Options

Content here.
```

## How Fragments Are Generated

mdBook generates fragments from headings:

| Heading | Fragment |
|---------|----------|
| `## Getting Started` | `#getting-started` |
| `## API Reference` | `#api-reference` |
| `## What's New?` | `#whats-new` |

## Configuration

This rule has no configuration options.

## When to Disable

- Books using custom anchor IDs
- Content with JavaScript-based navigation

## Rule Details

- **Rule ID**: MDBOOK006
- **Aliases**: internal-cross-references
- **Category**: MdBook
- **Severity**: Warning
- **Auto-fix**: No

## Related Rules

- [MD051](../standard/md051.md) - Link fragments (same-file)
- [MDBOOK002](./mdbook002.md) - Internal link validation
