# Whitespace Rules

These rules ensure consistent whitespace usage throughout your markdown documents.

## Rules in This Category

### Auto-fix Available ✓

- **[MD009](./md009.html)** - No trailing spaces
- **[MD010](./md010.html)** - Hard tabs
- **[MD012](./md012.html)** - Multiple consecutive blank lines
- **[MD027](./md027.html)** - Multiple spaces after blockquote symbol
- **[MD047](./md047.html)** - Files should end with a single newline

## Why Whitespace Matters

Consistent whitespace usage:
- Improves readability and maintainability
- Prevents version control issues (unnecessary diffs)
- Ensures consistent rendering across different viewers
- Follows standard text file conventions
- Reduces file size

## Quick Configuration

```toml
# .mdbook-lint.toml

# Configure MD009 - Trailing spaces
[MD009]
br_spaces = 2  # Allow 2 spaces for line breaks

# Configure MD010 - Hard tabs
[MD010]
spaces_per_tab = 4  # Convert tabs to 4 spaces

# Configure MD012 - Multiple blank lines
[MD012]
maximum = 1  # Allow max 1 consecutive blank line

# Configure MD027 - Blockquote spacing
[MD027]
spaces = 1  # Require 1 space after >
```

## Disable All Whitespace Rules

```toml
# .mdbook-lint.toml
disabled_rules = ["MD009", "MD010", "MD012", "MD027", "MD047"]
```

## Related Categories

- [Heading Rules](./headings.html) - Heading formatting and structure
- [List Rules](./lists.html) - List formatting and indentation