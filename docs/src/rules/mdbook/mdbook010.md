# MDBOOK010 - Preprocessor Validation

Missing or invalid preprocessor configuration.

## Why This Rule Exists

mdBook preprocessors transform content before rendering. Using preprocessor
directives without proper configuration causes silent failures or build errors.

## Examples

### Incorrect

Using a directive without configuring the preprocessor:

```text
\{{#katex}}
E = mc^2
\{{/katex}}
```

Without `[preprocessor.katex]` in `book.toml`.

### Correct

First, configure in `book.toml`:

```toml
[preprocessor.katex]
```

Then use the directive:

```text
\{{#katex}}
E = mc^2
\{{/katex}}
```

## Common Preprocessors

| Preprocessor | Purpose |
|--------------|---------|
| `katex` | Math equations |
| `mermaid` | Diagrams |
| `toc` | Table of contents |
| `template` | Template expansion |
| `admonish` | Callout boxes |

## Configuration

This rule has no configuration options.

## Rule Details

- **Rule ID**: MDBOOK010
- **Aliases**: preprocessor-validation
- **Category**: MdBook
- **Severity**: Warning
- **Stability**: Experimental
- **Auto-fix**: No

## Related Rules

- [MDBOOK011](./mdbook011.md) - Template validation
