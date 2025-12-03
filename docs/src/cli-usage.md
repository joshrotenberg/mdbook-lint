# CLI Usage

This page documents the command-line interface for mdbook-lint.

## Basic Commands

### lint

Lint markdown files and directories.

```bash
mdbook-lint lint [OPTIONS] [PATHS]...
```

### rules

List available linting rules.

```bash
mdbook-lint rules [OPTIONS]
```

### help

Show help information.

```bash
mdbook-lint help [COMMAND]
```

## Options

### Global Options

- `-h, --help`: Print help information
- `-V, --version`: Print version information
- `-v, --verbose`: Enable verbose output
- `-q, --quiet`: Suppress non-error output

### Lint Options

- `--config <FILE>`: Use specific configuration file
- `--fail-on-warnings`: Exit with error code on warnings
- `--disable <RULES>`: Disable specific rules (comma-separated)
- `--fix`: Automatically fix violations where possible
- `--fix-unsafe`: Apply all fixes, including potentially unsafe ones
- `--dry-run`: Show what would be fixed without applying changes (requires --fix or --fix-unsafe)
- `--no-backup`: Skip creating backup files when applying fixes

### Rules Options

- `--detailed`: Show detailed rule descriptions
- `--enabled`: Show only enabled rules
- `--format <FORMAT>`: Output format (text, json)

## Examples

```bash
# Lint current directory
mdbook-lint lint .

# Lint specific files
mdbook-lint lint README.md src/chapter1.md

# Lint with custom config
mdbook-lint lint --config custom-lint.toml src/

# Auto-fix violations where possible
mdbook-lint lint --fix docs/

# Preview fixes without applying them
mdbook-lint lint --fix --dry-run docs/

# Apply all fixes including potentially unsafe ones
mdbook-lint lint --fix-unsafe docs/

# Fix without creating backup files
mdbook-lint lint --fix --no-backup docs/

# Show all rules with descriptions
mdbook-lint rules --detailed

# Lint and fail on warnings
mdbook-lint lint --fail-on-warnings docs/
```

## Exit Codes

- `0`: Success (no errors)
- `1`: Linting errors found
- `2`: Invalid arguments or configuration

## Next Steps

- Learn about [mdBook Integration](./mdbook-integration.md)
- See [Configuration Reference](./configuration-reference.md) for all options
