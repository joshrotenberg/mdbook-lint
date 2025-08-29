# Configuration

mdbook-lint supports multiple configuration formats and provides flexible options for customizing linting behavior.

## Configuration File Formats

mdbook-lint automatically detects and supports multiple configuration formats:

- **TOML**: `.mdbook-lint.toml` or `mdbook-lint.toml` (recommended)
- **YAML**: `.mdbook-lint.yaml` or `.mdbook-lint.yml`  
- **JSON**: `.mdbook-lint.json`
- **markdownlint**: `.markdownlint.json` (for compatibility)

## Configuration Discovery

mdbook-lint searches for configuration files in the following order:

1. Current directory
2. Parent directories (recursively up to root)
3. Custom path via `MDBOOK_LINT_CONFIG` environment variable

The first configuration file found is used.

## Basic Configuration

### TOML Format (Recommended)

```toml
# Global settings
fail-on-warnings = false
fail-on-errors = true
disabled-rules = ["MD013", "MD033"]
enabled-rules = ["MD001", "MD002"]

# Rule-specific configuration
[MD007]
indent = 2

[MD013]
line-length = 120
code-blocks = false
```

### YAML Format

```yaml
fail-on-warnings: false
fail-on-errors: true
disabled-rules:
  - MD013
  - MD033
enabled-rules:
  - MD001
  - MD002

MD007:
  indent: 2

MD013:
  line-length: 120
  code-blocks: false
```

### JSON Format

```json
{
  "fail-on-warnings": false,
  "fail-on-errors": true,
  "disabled-rules": ["MD013", "MD033"],
  "enabled-rules": ["MD001", "MD002"],
  "MD007": {
    "indent": 2
  },
  "MD013": {
    "line-length": 120,
    "code-blocks": false
  }
}
```

## Advanced Rule Control

### The `[rules]` Section

The `[rules]` section provides fine-grained control over which rules run:

```toml
[rules]
# Disable all rules by default
default = false

# Only enable specific rules
[rules.enabled]
MD001 = true
MD002 = true
MD013 = true

# Rule-specific configuration still works
[MD013]
line-length = 120
```

This is particularly useful for:
- Gradual adoption of linting rules
- Testing specific rules
- Creating minimal rule sets

### Category-Based Rule Management

Control entire categories of rules at once:

```toml
# Enable/disable rule categories
enabled-categories = ["headings", "lists"]
disabled-categories = ["whitespace"]

# Individual rules override categories
enabled-rules = ["MD009"]  # Enable even though whitespace is disabled
disabled-rules = ["MD001"]  # Disable even though headings is enabled
```

Available categories:
- `headings` - Heading-related rules
- `lists` - List formatting rules
- `whitespace` - Whitespace and blank line rules
- `code` - Code block and inline code rules
- `style` - General style rules
- `links` - Link and reference rules
- `mdbook` - mdBook-specific rules

## markdownlint Compatibility

mdbook-lint can read `.markdownlint.json` files for compatibility:

```json
{
  "default": false,
  "MD001": true,
  "MD013": {
    "line_length": 120,
    "code_blocks": false
  }
}
```

Enable full markdownlint compatibility mode:

```toml
markdownlint-compatible = true
```

This disables rules that are disabled by default in markdownlint.

## Global Configuration Options

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `fail-on-warnings` | boolean | `false` | Exit with error code on warnings |
| `fail-on-errors` | boolean | `true` | Exit with error code on errors |
| `disabled-rules` | array | `[]` | List of rule IDs to disable |
| `enabled-rules` | array | `[]` | List of rule IDs to explicitly enable |
| `enabled-categories` | array | `[]` | List of categories to enable |
| `disabled-categories` | array | `[]` | List of categories to disable |
| `markdownlint-compatible` | boolean | `false` | Enable markdownlint compatibility |
| `deprecated-warning` | string | `"warn"` | How to handle deprecated rules (`"warn"`, `"info"`, `"silent"`) |
| `malformed-markdown` | string | `"warn"` | How to handle malformed markdown (`"error"`, `"warn"`, `"skip"`) |

## Configuration Precedence

Configuration is resolved in the following order (later overrides earlier):

1. Built-in defaults
2. Configuration file (`.mdbook-lint.toml`, etc.)
3. mdBook preprocessor config (in `book.toml`)
4. Environment variables (`MDBOOK_LINT_*`)
5. Command-line arguments

## Environment Variables

- `MDBOOK_LINT_CONFIG` - Path to custom configuration file
- `MDBOOK_LINT_LOG` - Log level (`error`, `warn`, `info`, `debug`, `trace`)

## mdBook Integration

When used as an mdBook preprocessor, configuration can be specified in `book.toml`:

```toml
[preprocessor.mdbook-lint]
fail-on-warnings = true
disabled-rules = ["MD025"]

[preprocessor.mdbook-lint.MD013]
line-length = 100
```

## Example Configurations

### Minimal - Only Critical Rules

```toml
[rules]
default = false

[rules.enabled]
MD001 = true  # Heading levels should increment
MD003 = true  # Heading style consistency
MD009 = true  # No trailing spaces
MD047 = true  # File should end with newline
```

### Strict - All Rules with Custom Settings

```toml
fail-on-warnings = true
fail-on-errors = true

[MD007]
indent = 2

[MD013]
line-length = 80
code-blocks = true
tables = true
headings = true

[MD024]
siblings-only = true

[MD029]
style = "ordered"
```

### mdBook Projects

```toml
# Disable rules that conflict with mdBook conventions
disabled-rules = [
    "MD025",  # Multiple H1s are OK in books
    "MD041",  # First line doesn't need to be H1
]

# mdBook-specific rules
enabled-categories = ["mdbook"]

[MD013]
line-length = 100  # Longer lines for documentation
```

### Progressive Adoption

Start with a few rules and gradually enable more:

```toml
# Phase 1: Start with formatting rules
[rules]
default = false

[rules.enabled]
MD009 = true  # Trailing spaces
MD010 = true  # Hard tabs
MD012 = true  # Multiple blank lines

# Phase 2: Add heading rules (uncomment when ready)
# MD001 = true  # Heading increment
# MD003 = true  # Heading style

# Phase 3: Add more rules...
```

### Migration from markdownlint

If migrating from markdownlint, start with compatibility mode:

```toml
markdownlint-compatible = true

# Then gradually customize...
[MD013]
line-length = 100
```

## Configuration Validation

To validate your configuration:

```bash
# Check if configuration is valid
mdbook-lint lint --config .mdbook-lint.toml --dry-run

# Show which rules are enabled
mdbook-lint rules --config .mdbook-lint.toml
```

## Next Steps

- [Configuration Reference](./configuration-reference.md) - Complete list of options
- [Rules Reference](./rules-reference.md) - All rules and their configurations
- [CLI Usage](./cli-usage.md) - Command-line options