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
[rules.MD007]
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

## Common Workflow

Here's a typical workflow for using mdbook-lint:

1. **Initial setup**: Add configuration file and run first lint
2. **Fix major issues**: Address structural problems first
3. **Customize rules**: Disable rules that don't fit your style
4. **Integrate with build**: Add to mdBook or CI pipeline
5. **Maintain quality**: Regular linting keeps documentation clean

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