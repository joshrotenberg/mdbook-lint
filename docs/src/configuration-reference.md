# Configuration Reference

Complete reference for all configuration options in mdbook-lint.

## Global Configuration Options

### fail-on-warnings

- **Type**: `boolean`
- **Default**: `false`
- **Description**: Exit with error code when warnings are found

### fail-on-errors

- **Type**: `boolean`
- **Default**: `true`
- **Description**: Exit with error code when errors are found

### disabled-rules

- **Type**: `array<string>`
- **Default**: `[]`
- **Description**: List of rule IDs to disable globally
- **Example**: `["MD013", "MD033"]`

### enabled-rules

- **Type**: `array<string>`
- **Default**: `[]`
- **Description**: List of rule IDs to explicitly enable
- **Example**: `["MD001", "MD002"]`

### enabled-categories

- **Type**: `array<string>`
- **Default**: `[]`
- **Description**: List of rule categories to enable
- **Valid values**: `headings`, `lists`, `whitespace`, `code`, `style`, `links`, `mdbook`

### disabled-categories

- **Type**: `array<string>`
- **Default**: `[]`
- **Description**: List of rule categories to disable

### markdownlint-compatible

- **Type**: `boolean`
- **Default**: `false`
- **Description**: Enable markdownlint compatibility mode (disables rules that are disabled by default in markdownlint)

### deprecated-warning

- **Type**: `string`
- **Default**: `"warn"`
- **Description**: How to handle deprecated rule warnings
- **Valid values**: `"warn"`, `"info"`, `"silent"`

### malformed-markdown

- **Type**: `string`
- **Default**: `"warn"`
- **Description**: How to handle malformed markdown
- **Valid values**: `"error"`, `"warn"`, `"skip"`

## Rules Section Configuration

### rules.default

- **Type**: `boolean`
- **Default**: `true`
- **Description**: Whether rules are enabled by default

### rules.enabled

- **Type**: `table<string, boolean>`
- **Description**: Map of rule IDs to enable when `default = false`

### rules.disabled

- **Type**: `table<string, boolean>`
- **Description**: Map of rule IDs to disable when `default = true`

**Example:**

```toml
[rules]
default = false

[rules.enabled]
MD001 = true
MD002 = true
MD009 = true
```

## Rule-Specific Configuration

### MD002 - First heading should be a top-level heading

```toml
[MD002]
level = 1  # Expected level of first heading (default: 1)
```

### MD003 - Heading style

```toml
[MD003]
style = "consistent"  # Options: "consistent", "atx", "atx_closed", "setext"
```

### MD004 - Unordered list style

```toml
[MD004]
style = "consistent"  # Options: "consistent", "asterisk", "plus", "dash"
```

### MD007 - Unordered list indentation

```toml
[MD007]
indent = 2  # Spaces for indentation (default: 2)
start_indented = false  # Allow first level to be indented
```

### MD009 - Trailing spaces

```toml
[MD009]
br_spaces = 2  # Spaces for line breaks (default: 2)
list_item_empty_lines = false  # Allow spaces in empty list items
strict = false  # Strict mode for all trailing spaces
```

### MD010 - Hard tabs

```toml
[MD010]
code_blocks = true  # Include code blocks (default: true)
spaces_per_tab = 4  # Spaces per tab for reporting (default: 4)
```

### MD012 - Multiple consecutive blank lines

```toml
[MD012]
maximum = 1  # Maximum consecutive blank lines (default: 1)
```

### MD013 - Line length

```toml
[MD013]
line_length = 80  # Maximum line length (default: 80)
code_blocks = false  # Check code blocks
tables = false  # Check tables
headings = true  # Check headings
heading_line_length = 80  # Separate limit for headings
strict = false  # Strict length checking
stern = false  # Stern length checking
```

### MD024 - Multiple headings with same content

```toml
[MD024]
siblings_only = false  # Only check sibling headings (default: false)
```

### MD025 - Multiple top-level headings

```toml
[MD025]
level = 1  # Heading level to check (default: 1)
front_matter_title = true  # Use front matter title
```

### MD026 - Trailing punctuation in heading

```toml
[MD026]
punctuation = ".,;:!?"  # Punctuation to check (default: ".,;:!?")
```

### MD029 - Ordered list item prefix

```toml
[MD029]
style = "one_or_ordered"  # Options: "one", "ordered", "one_or_ordered", "zero"
```

### MD030 - Spaces after list markers

```toml
[MD030]
ul_single = 1  # Spaces after single-line unordered list marker
ul_multi = 1  # Spaces after multi-line unordered list marker
ol_single = 1  # Spaces after single-line ordered list marker
ol_multi = 1  # Spaces after multi-line ordered list marker
```

### MD035 - Horizontal rule style

```toml
[MD035]
style = "consistent"  # Style to enforce or "consistent"
```

### MD036 - Emphasis used instead of heading

```toml
[MD036]
punctuation = ".,;:!?"  # Punctuation at end (default: ".,;:!?")
```

### MD043 - Required heading structure

```toml
[MD043]
headings = ["# Summary", "## Overview"]  # Required headings in order
required_headings = ["# Summary", "## Overview"]  # Alternative name
headers = ["# Summary", "## Overview"]  # Alternative name (deprecated)
```

### MD044 - Proper names should have correct capitalization

```toml
[MD044]
names = ["JavaScript", "GitHub", "TypeScript"]  # Proper names
code_blocks = false  # Include code blocks
html_elements = false  # Include HTML elements
```

### MD046 - Code block style

```toml
[MD046]
style = "consistent"  # Options: "consistent", "fenced", "indented"
```

### MD048 - Code fence style

```toml
[MD048]
style = "consistent"  # Options: "consistent", "backtick", "tilde"
```

### MD049 - Emphasis style

```toml
[MD049]
style = "consistent"  # Options: "consistent", "asterisk", "underscore"
```

### MD050 - Strong style

```toml
[MD050]
style = "consistent"  # Options: "consistent", "asterisk", "underscore"
```

### MD051 - Link fragments should be valid

```toml
[MD051]
# No configuration options
```

### MD052 - Reference links and images should use label

```toml
[MD052]
shortcut_syntax = false  # Allow shortcut syntax
```

### MD053 - Link and image reference definitions need labels

```toml
[MD053]
ignored_definitions = ["//"]  # Definitions to ignore
```

### MD054 - Link and image reference definitions should be used

```toml
[MD054]
# No configuration options
```

### MD055 - Table pipe style

```toml
[MD055]
style = "consistent"  # Options: "consistent", "leading_only", "trailing_only", "leading_and_trailing", "no_leading_or_trailing"
```

### MD056 - Table column count

```toml
[MD056]
# No configuration options
```

### MD058 - Tables should be surrounded by blank lines

```toml
[MD058]
# No configuration options
```

### MD059 - Tables should not have empty cells

```toml
[MD059]
allowed_corner_cells = false  # Allow empty corner cells
```

## mdBook-Specific Rules

mdBook-specific rules (MDBOOK001-MDBOOK025) generally don't have configuration options, as they check for mdBook-specific patterns and conventions.

## Configuration File Examples

### Minimal Configuration

```toml
disabled-rules = ["MD013", "MD033"]
```

### Comprehensive Configuration

```toml
fail-on-warnings = true
fail-on-errors = true
markdownlint-compatible = false
deprecated-warning = "warn"
malformed-markdown = "error"

enabled-categories = ["headings", "lists"]
disabled-rules = ["MD041"]

[MD002]
level = 1

[MD003]
style = "atx"

[MD007]
indent = 2
start_indented = false

[MD009]
br_spaces = 2
strict = false

[MD013]
line_length = 100
code_blocks = false
tables = false

[MD024]
siblings_only = true

[MD029]
style = "ordered"

[MD030]
ul_single = 1
ol_single = 1

[MD044]
names = ["JavaScript", "TypeScript", "GitHub", "mdBook"]
code_blocks = false
```

### Using Rules Section

```toml
[rules]
default = false

[rules.enabled]
MD001 = true
MD002 = true
MD003 = true
MD009 = true
MD047 = true

[MD003]
style = "atx"

[MD009]
br_spaces = 2
```
