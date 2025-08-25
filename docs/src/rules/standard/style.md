# Style Rules

These rules enforce consistent styling choices throughout your markdown documents.

## Rules in This Category

- **[MD013](./md013.html)** - Line length
- **MD003** - Heading style
- **MD035** - Horizontal rule style
- **MD036** - Emphasis used instead of a heading
- **MD044** - Proper names should have correct capitalization
- **MD049** - Emphasis style should be consistent
- **MD050** - Strong style should be consistent

## Why Style Rules Matter

Consistent style:
- Creates professional, polished documentation
- Improves readability and scanning
- Reduces cognitive load for readers
- Maintains brand and project consistency
- Facilitates team collaboration

## Common Style Choices

### Heading Styles

```markdown
# ATX Style Heading (Recommended)

Setext Style Heading
====================
```

### Emphasis Styles

```markdown
*Italic with asterisks*
_Italic with underscores_

**Bold with asterisks**
__Bold with underscores__
```

### Horizontal Rules

```markdown
---
***
___
```

## Quick Configuration

```toml
# .mdbook-lint.toml

# Configure MD013 - Line length
[rules.MD013]
line_length = 100
code_blocks = false
tables = false

# Configure MD003 - Heading style
[rules.MD003]
style = "atx"  # Options: "atx", "setext", "consistent"

# Configure MD035 - Horizontal rule style
[rules.MD035]
style = "---"  # Use three hyphens

# Configure MD049 - Emphasis style
[rules.MD049]
style = "asterisk"  # Options: "asterisk", "underscore", "consistent"

# Configure MD050 - Strong style
[rules.MD050]
style = "asterisk"  # Options: "asterisk", "underscore", "consistent"
```

## Style Guide Template

Create a consistent style guide for your project:

```toml
# .mdbook-lint.toml - Project Style Guide

# Line length for readability
[rules.MD013]
line_length = 80

# ATX headings only
[rules.MD003]
style = "atx"

# Consistent emphasis
[rules.MD049]
style = "asterisk"

[rules.MD050]
style = "asterisk"

# Three hyphens for horizontal rules
[rules.MD035]
style = "---"
```

## Best Practices

1. **Choose and document**: Pick a style and document it
2. **Be consistent**: Use the same style throughout
3. **Consider your audience**: Technical vs. general readers
4. **Think about rendering**: How it looks in your target output
5. **Automate checks**: Use CI/CD to enforce style

## Related Categories

- [Heading Rules](./headings.html) - Detailed heading formatting
- [Whitespace Rules](./whitespace.html) - Spacing and indentation
- [Code Rules](./code.html) - Code formatting standards