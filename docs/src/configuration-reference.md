# Configuration Reference

This page provides a complete reference for all configuration options available in mdbook-lint.

## Configuration File Format

mdbook-lint uses TOML format for configuration files. The default configuration file is `.mdbook-lint.toml` in your project root.

```toml
# Global settings
fail-on-warnings = false
disabled-rules = []

# Rule-specific configuration
[rules.MD013]
line-length = 80

[rules.MD007]
indent = 2
```

## Global Configuration

### fail-on-warnings
- **Type**: boolean
- **Default**: `false`
- **Description**: When `true`, mdbook-lint exits with a non-zero code if any warnings are found.

```toml
fail-on-warnings = true
```

### disabled-rules
- **Type**: array of strings
- **Default**: `[]`
- **Description**: List of rule IDs to disable globally.

```toml
disabled-rules = ["MD013", "MD033", "MD041"]
```

## Rule-Specific Configuration

### MD001 - Heading levels
No additional configuration options.

### MD003 - Heading style
- **style**: `"atx"` | `"setext"` | `"consistent"`
- **Default**: `"consistent"`

```toml
[rules.MD003]
style = "atx"
```

### MD004 - Unordered list style
- **style**: `"dash"` | `"asterisk"` | `"plus"` | `"consistent"`
- **Default**: `"consistent"`

```toml
[rules.MD004]
style = "dash"
```

### MD007 - Unordered list indentation
- **indent**: integer (number of spaces)
- **Default**: `2`

```toml
[rules.MD007]
indent = 4
```

### MD009 - Trailing whitespace
- **br-spaces**: integer (number of trailing spaces allowed for line breaks)
- **Default**: `2`

```toml
[rules.MD009]
br-spaces = 2
```

### MD012 - Multiple consecutive blank lines
- **maximum**: integer (maximum consecutive blank lines)
- **Default**: `1`

```toml
[rules.MD012]
maximum = 2
```

### MD013 - Line length
- **line-length**: integer (maximum line length)
- **Default**: `80`
- **code-blocks**: boolean (check line length in code blocks)
- **Default**: `true`
- **tables**: boolean (check line length in tables)
- **Default**: `true`
- **headings**: boolean (check line length in headings)
- **Default**: `true`

```toml
[rules.MD013]
line-length = 120
code-blocks = false
tables = false
headings = true
```

### MD025 - Multiple top level headings
- **level**: integer (heading level to check)
- **Default**: `1`

```toml
[rules.MD025]
level = 1
```

### MD029 - Ordered list item prefix
- **style**: `"one"` | `"ordered"` | `"zero"`
- **Default**: `"one"`

```toml
[rules.MD029]
style = "ordered"
```

### MD033 - Inline HTML
- **allowed-elements**: array of strings (HTML elements to allow)
- **Default**: `[]`

```toml
[rules.MD033]
allowed-elements = ["br", "sub", "sup"]
```

### MD035 - Horizontal rule style
- **style**: string (horizontal rule style)
- **Default**: `"consistent"`

```toml
[rules.MD035]
style = "---"
```

### MD036 - Emphasis used instead of heading
- **punctuation**: string (punctuation marks that indicate emphasis)
- **Default**: `".,;:!"`

```toml
[rules.MD036]
punctuation = ".,;:!?"
```

## mdBook Integration Configuration

When using mdbook-lint as an mdBook preprocessor, configuration can be specified in `book.toml`:

```toml
[preprocessor.mdbook-lint]
fail-on-warnings = true
disabled-rules = ["MD025"]

# Rule configuration in book.toml
[preprocessor.mdbook-lint.rules.MD013]
line-length = 100
```

## Configuration Precedence

Configuration is loaded in the following order (later sources override earlier ones):

1. Built-in defaults
2. `.mdbook-lint.toml` in project root
3. `book.toml` preprocessor configuration (when used as preprocessor)
4. Command-line arguments

## Environment Variables

- `MDBOOK_LINT_CONFIG`: Path to custom configuration file
- `MDBOOK_LINT_LOG`: Log level (`error`, `warn`, `info`, `debug`, `trace`)

```bash
export MDBOOK_LINT_CONFIG=custom-config.toml
export MDBOOK_LINT_LOG=debug
mdbook-lint lint .
```

## Configuration Examples

### Strict Configuration
```toml
fail-on-warnings = true
disabled-rules = []

[rules.MD013]
line-length = 80
code-blocks = true
tables = true

[rules.MD007]
indent = 2
```

### Relaxed Configuration
```toml
fail-on-warnings = false
disabled-rules = ["MD013", "MD033", "MD041"]

[rules.MD012]
maximum = 3

[rules.MD029]
style = "ordered"
```

### mdBook-Optimized Configuration
```toml
# Common mdBook adjustments
disabled-rules = ["MD025"]  # Multiple H1s are OK in books
fail-on-warnings = true

[rules.MD013]
line-length = 100  # Slightly longer lines for books

[rules.MD033]
allowed-elements = ["br", "kbd", "sub", "sup"]  # Common HTML in docs
```

## Validation

To validate your configuration file:

```bash
mdbook-lint lint --config .mdbook-lint.toml --dry-run
```

## Next Steps

- Learn about specific rules in [Rules Reference](./rules-reference.md)
- See [CLI Usage](./cli-usage.md) for command-line configuration options
- Check [mdBook Integration](./mdbook-integration.md) for preprocessor setup