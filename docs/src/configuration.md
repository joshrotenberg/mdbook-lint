# Configuration

mdbook-lint can be configured through configuration files to customize its behavior for your project.

## Configuration File

Create a `.mdbook-lint.toml` file in your project root:

```toml
# Basic settings
fail-on-warnings = false
disabled-rules = ["MD013", "MD033"]

# Rule-specific configuration
[rules.MD007]
indent = 2

[rules.MD013]
line-length = 120
```

## Configuration Options

### Global Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `fail-on-warnings` | boolean | `false` | Exit with error code when warnings are found |
| `disabled-rules` | array | `[]` | List of rule IDs to disable |

### Rule Configuration

Individual rules can be configured in the `[rules.<RULE_ID>]` sections.

#### MD007 - Unordered list indentation
```toml
[rules.MD007]
indent = 2  # Number of spaces for list indentation
```

#### MD013 - Line length
```toml
[rules.MD013]
line-length = 80     # Maximum line length
code-blocks = false  # Check line length in code blocks
tables = false       # Check line length in tables
```

## Configuration Precedence

Configuration is loaded in the following order (later sources override earlier ones):

1. Built-in defaults
2. `.mdbook-lint.toml` in project root
3. Command-line arguments

## Examples

### Strict Configuration
```toml
fail-on-warnings = true
disabled-rules = []  # Enable all rules

[rules.MD013]
line-length = 80
```

### Relaxed Configuration
```toml
fail-on-warnings = false
disabled-rules = ["MD013", "MD033", "MD041"]

[rules.MD007]
indent = 4
```

### mdBook-focused Configuration
```toml
# Disable rules that conflict with mdBook conventions
disabled-rules = ["MD025"]  # Multiple top-level headers are OK in mdBook

fail-on-warnings = true
```

## Next Steps

- Learn about [CLI Usage](./cli-usage.md)
- Browse the [Configuration Reference](./configuration-reference.md) for all options
- See [Rules Reference](./rules-reference.md) for rule-specific settings