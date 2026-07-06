# Example Configuration

This page provides a complete, fully-commented example configuration file for mdbook-lint.

## Quick Start

1. Copy the configuration below to `.mdbook-lint.toml` in your project root
2. Uncomment and modify only the settings you want to change
3. All settings are optional - mdbook-lint works with sensible defaults

## Complete Example Configuration

```toml
{{#include ../../crates/mdbook-lint-cli/example-mdbook-lint.toml}}
```

## Common Configuration Patterns

### Minimal Configuration

For most projects, a minimal configuration is sufficient:

```toml
# .mdbook-lint.toml
fail-on-warnings = true
disabled_rules = ["MD013"]  # Disable line length if not needed
```

### Strict Configuration

For projects requiring strict markdown compliance:

```toml
# Fail on any issues
fail-on-warnings = true

# Strict whitespace rules
[MD009]
strict = true  # No trailing spaces at all

[MD010]
code_blocks = true  # Check tabs in code blocks

# Require code block languages
[MD040]
language_optional = false

# Strict line length
[MD013]
line_length = 80
strict = true
```

### Documentation Project

For technical documentation or mdBook projects:

```toml
# mdBook-specific checks
[MDBOOK002]
check_anchors = true
check_images = true

[MDBOOK005]
ignore_patterns = ["drafts/**", "archive/**"]

# Allow longer lines for documentation
[MD013]
line_length = 100
code_blocks = false  # Don't check code block line length
tables = false  # Don't check table line length

# Require proper code highlighting
[MD040]
language_optional = false
```

### Blog or Content Site

For blogs or content-heavy sites:

```toml
# Relaxed rules for content
disabled_rules = [
    "MD013",  # No line length limit
    "MD033",  # Allow inline HTML
    "MD041"   # First line doesn't need to be H1
]

# Allow emphasis for styling
[MD036]
punctuation = ""  # Don't check for punctuation

# Consistent emphasis style
[MD049]
style = "asterisk"

[MD050]
style = "asterisk"
```

## Integration Configurations

### GitHub Actions

```toml
# For CI/CD pipelines
fail-on-warnings = true
```

### mdBook Preprocessor

```toml
[preprocessor]
fail_on_warnings = false  # Warning but don't fail build
renderer = ["html"]  # Only run for HTML output
```

## Rule Categories Quick Reference

### Disable All Rules in a Category

```toml
# Disable all heading rules
disabled_rules = [
    "MD001", "MD002", "MD003", "MD018", "MD019",
    "MD020", "MD021", "MD022", "MD023", "MD024",
    "MD025", "MD026"
]

# Disable all whitespace rules
disabled_rules = [
    "MD009", "MD010", "MD012", "MD027", "MD028", "MD047"
]

# Disable all list rules
disabled_rules = [
    "MD004", "MD005", "MD006", "MD007", "MD029",
    "MD030", "MD032"
]
```

## Tips

1. **Start minimal**: Begin with defaults and add configuration as needed
2. **Document choices**: Comment why certain rules are disabled
3. **Version control**: Commit `.mdbook-lint.toml` to your repository
4. **Team agreement**: Discuss and agree on rules with your team

## Next Steps

- See [Configuration Reference](./configuration-reference.md) for detailed options
- Check [Rules Reference](./rules-reference.md) for all available rules
- Learn about [mdBook Integration](./mdbook-integration.md) for book projects
