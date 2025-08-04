# Rules Reference

This page provides a comprehensive reference for all linting rules available in mdbook-lint.

## Standard Markdown Rules (MD001-MD059)

mdbook-lint implements 59 standard markdown linting rules based on the widely-used markdownlint specification.

### Heading Rules

#### MD001 - Heading levels should only increment by one level at a time
Ensures proper heading hierarchy in your documents.

```markdown
# Good
## Good

# Bad
### Bad (skips level 2)
```

#### MD003 - Heading style
Enforces consistent heading styles throughout your document.

#### MD018 - No space after hash on atx style heading
Requires a space after the hash in ATX-style headings.

#### MD019 - Multiple spaces after hash on atx style heading
Prevents multiple spaces after the hash in ATX-style headings.

### List Rules

#### MD004 - Unordered list style
Enforces consistent marker style for unordered lists.

#### MD005 - Inconsistent indentation for list items at the same level
Ensures consistent indentation within lists.

#### MD007 - Unordered list indentation
Controls indentation levels for nested lists.

### Line Rules

#### MD009 - Trailing whitespace
Removes unnecessary whitespace at the end of lines.

#### MD010 - Hard tabs
Prevents the use of hard tab characters.

#### MD012 - Multiple consecutive blank lines
Limits consecutive blank lines.

#### MD013 - Line length
Enforces maximum line length limits.

### Link Rules

#### MD034 - Bare URL used
Requires proper link formatting for URLs.

#### MD039 - Spaces inside link text
Prevents spaces around link text.

## mdBook-Specific Rules (MDBOOK001-004)

These rules are specifically designed for mdBook projects and check mdBook-specific syntax and conventions.

### MDBOOK001 - SUMMARY.md structure
Validates the structure of your SUMMARY.md file according to mdBook conventions.

### MDBOOK002 - Internal link validation
Checks that internal links reference valid files and sections within your book.

### MDBOOK003 - Missing referenced files
Ensures all files referenced in SUMMARY.md actually exist.

### MDBOOK004 - mdBook syntax validation
Validates mdBook-specific markdown syntax and features.

## Rule Configuration

Many rules can be customized through configuration. See the [Configuration Reference](./configuration-reference.md) for details.

### Example Rule Configuration

```toml
[rules.MD013]
line-length = 120
code-blocks = false

[rules.MD007]
indent = 2
```

## Disabling Rules

Rules can be disabled globally or for specific files:

```toml
# Disable globally
disabled-rules = ["MD013", "MD033"]
```

```markdown
<!-- mdbook-lint-disable MD013 -->
This line can be very long and won't trigger the line length rule.
<!-- mdbook-lint-enable MD013 -->
```

## Getting Help

To see all available rules with descriptions:

```bash
mdbook-lint rules --detailed
```

To see only enabled rules:

```bash
mdbook-lint rules --enabled
```

## Next Steps

- Learn about [Configuration Reference](./configuration-reference.md)
- Explore [CLI Usage](./cli-usage.md) for rule-specific commands
- Check [Contributing](./contributing.md) to request new rules