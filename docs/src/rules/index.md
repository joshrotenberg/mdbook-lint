# Rules Reference

mdbook-lint provides comprehensive markdown linting with two rule categories.

## [Standard Markdown Rules](./standard/index.md)

59 rules (MD001-MD059) based on the widely-used markdownlint specification.
These rules ensure consistent markdown formatting and style.

### Categories

- [Heading Rules](./standard/headings.md) - Heading hierarchy and formatting
- [List Rules](./standard/lists.md) - List formatting and consistency
- [Whitespace Rules](./standard/whitespace.md) - Trailing spaces, blank lines
- [Link Rules](./standard/links.md) - URL formatting and link text
- [Code Rules](./standard/code.md) - Code block formatting and fencing
- [Style Rules](./standard/style.md) - Emphasis and formatting consistency

## [mdBook-Specific Rules](./mdbook/index.md)

Rules specifically designed for mdBook projects, validating mdBook-specific
syntax and conventions.

### Rules

- [MDBOOK001](./mdbook/mdbook001.md) - Code blocks should have language tags
- [MDBOOK002](./mdbook/mdbook002.md) - SUMMARY.md structure validation
- [MDBOOK003](./mdbook/mdbook003.md) - Internal link validation
- [MDBOOK004](./mdbook/mdbook004.md) - Part title formatting
- [MDBOOK005](./mdbook/mdbook005.md) - Chapter path validation
- [MDBOOK006](./mdbook/mdbook006.md) - Draft chapter validation
- [MDBOOK007](./mdbook/mdbook007.md) - Separator syntax validation

## Quick Reference

### Rules with Automatic Fix Support

The following rules can automatically fix violations:

- **MD009** - Remove trailing spaces
- **MD010** - Replace hard tabs with spaces
- **MD012** - Remove multiple consecutive blank lines
- **MD018** - Add space after hash in ATX headings
- **MD019** - Fix multiple spaces after hash
- **MD020** - Remove spaces inside closed ATX headings
- **MD021** - Fix multiple spaces inside closed ATX headings
- **MD023** - Remove indentation from headings
- **MD027** - Fix multiple spaces after blockquote symbol
- **MD030** - Fix spaces after list markers
- **MD034** - Wrap bare URLs in angle brackets
- **MD047** - Ensure files end with single newline

### Disabling Rules

Rules can be disabled globally or for specific files:

```toml
# Disable globally
[rules]
MD002 = false
MD041 = false

# Disable for specific files
[ignore]
MD013 = ["CHANGELOG.md", "docs/api/*.md"]
```
