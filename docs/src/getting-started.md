# Getting Started

This guide will walk you through using mdbook-lint for the first time.

## Quick Start

The fastest way to get started is to run mdbook-lint on some markdown files:

```bash
# Lint a single file
mdbook-lint lint README.md

# Lint multiple files
mdbook-lint lint src/*.md docs/*.md

# Lint all markdown files in a directory
mdbook-lint lint .

# Auto-fix violations where possible
mdbook-lint lint --fix src/*.md
```

## Understanding the Output

When mdbook-lint finds issues, it will display them like this:

```
README.md:15:1: MD013: Line length (line too long, 85 > 80)
src/intro.md:3:1: MD001: Heading levels should only increment by one level at a time
```

Each line shows:

- **File and location**: `filename:line:column`
- **Rule ID**: The specific rule that was violated (e.g., MD013)
- **Description**: What the issue is and how to fix it

## Your First Configuration

Create a `.mdbook-lint.toml` file in your project root:

```toml
# Fail the build on warnings
fail-on-warnings = true

# Disable rules that don't fit your project
disabled-rules = ["MD013"]  # Allow long lines

# Configure specific rules
[MD007]
indent = 4  # Use 4-space indentation for lists
```

## Using with mdBook

To integrate mdbook-lint with your mdBook project:

1. **Add to book.toml**:

   ```toml
   [preprocessor.mdbook-lint]
   ```

2. **Build your book**:

   ```bash

   mdbook build
   ```

mdbook-lint will now check your markdown files every time you build your book.

> **Choosing Your Integration:** You can run mdbook-lint either as an mdBook preprocessor (shown above) OR as a standalone tool in CI. See [CI vs Preprocessor](./ci-vs-preprocessor.md) to understand when to use each approach.

## Automatic Fixing

mdbook-lint can automatically fix some common violations:

```bash
# Fix violations automatically
mdbook-lint lint --fix docs/

# Preview what would be fixed without applying changes
mdbook-lint lint --fix --dry-run docs/

# Apply all fixes, including potentially risky ones
mdbook-lint lint --fix-unsafe docs/
```

Currently supported fixes include:

- **MD009**: Trailing spaces
- **MD010**: Hard tabs → spaces
- **MD012**: Multiple blank lines
- **MD018/MD019**: Heading spacing issues
- **MD022**: Blank lines around headings
- **MD023**: Indented headings
- **MD027**: Blockquote spacing
- **MD030**: List marker spacing
- **MD034**: Bare URLs
- **MD047**: Missing trailing newline
- And more!

## Common Workflow

Here's a typical workflow for using mdbook-lint:

1. **Initial setup**: Add configuration file and run first lint
2. **Auto-fix simple issues**: Use `--fix` to handle common problems
3. **Fix remaining issues**: Address structural problems manually
4. **Customize rules**: Disable rules that don't fit your style
5. **Integrate with build**: Add to mdBook or CI pipeline
6. **Maintain quality**: Regular linting keeps documentation clean

## Exploring Rules

To see all available rules:

```bash
# List all rules
mdbook-lint rules

# Show detailed rule descriptions
mdbook-lint rules --detailed

# Show only enabled rules
mdbook-lint rules --enabled
```

## Next Steps

- Learn about [Configuration](./configuration.md) options
- Explore [CLI Usage](./cli-usage.md) in detail
- Set up [mdBook Integration](./mdbook-integration.md)
- Browse the [Rules Reference](./rules-reference.md)
