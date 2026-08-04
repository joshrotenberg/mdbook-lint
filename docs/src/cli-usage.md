# CLI Usage

This page documents the command-line interface for mdbook-lint.

## Basic Commands

### preprocessor

Run as an mdBook preprocessor (reads from stdin, writes to stdout). This is the
mode mdBook invokes; you normally do not run it by hand.

```bash
mdbook-lint preprocessor
```

See [mdBook Integration](./mdbook-integration.md) for details.

### lint

Lint markdown files and directories.

```bash
mdbook-lint lint [OPTIONS] [FILES]...
```

### fix

Automatically fix issues in markdown files (shorthand for `lint --fix`).

```bash
mdbook-lint fix [OPTIONS] [FILES]...
```

### rules

List available linting rules by category.

```bash
mdbook-lint rules [OPTIONS]
```

### check

Check a configuration file for validity.

```bash
mdbook-lint check <CONFIG>
```

### init

Generate a default configuration file.

```bash
mdbook-lint init [OPTIONS]
```

### supports

Check whether the preprocessor supports a given renderer (used by mdBook).

```bash
mdbook-lint supports <RENDERER>
```

### rustdoc

Lint module-level documentation (`//!` comments) in Rust source files.

```bash
mdbook-lint rustdoc [OPTIONS] [PATHS]...
```

See [Rustdoc Linting](./rustdoc-linting.md) for detailed documentation.

### lsp

Run as a Language Server Protocol (LSP) server. Available only when
mdbook-lint is built with the `lsp` feature.

```bash
mdbook-lint lsp [OPTIONS]
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
- `--enable <RULES>`: Enable only specific rules (comma-separated)
- `--fix`: Automatically fix violations where possible
- `--fix-unsafe`: Apply all fixes, including potentially unsafe ones
- `--dry-run`: Show what would be fixed without applying changes (requires --fix or --fix-unsafe)
- `--no-backup`: Skip creating backup files when applying fixes
- `--output <FORMAT>`: Output format (default, JSON, GitHub)
- `--color <WHEN>`: Control colored output (auto, always, never)

### Rules Options

- `-d, --detailed`: Show detailed information about each rule
- `-c, --category <CATEGORY>`: Filter by rule category
- `-p, --provider <PROVIDER>`: Show only rules from a specific provider
- `--standard-only`: Show only standard rules (MD001-MD059)
- `--mdbook-only`: Show only mdBook-specific rules
- `--format <FORMAT>`: Output format (default, json)
- `--json`: Output in JSON format (shorthand for `--format json`)

## Output Format

By default, mdbook-lint displays violations in a cargo/rustc-style format with colors:

```text
error[MD001]: Expected heading level 2 but got level 3
  --> src/chapter.md:15:1
     |
  15 | ### Skipped heading level
     | ^^^ heading-increment

warning[MD009]: Trailing spaces detected
  --> src/intro.md:8:42
     |
   8 | This line has trailing spaces   
     |                                  ^ no-trailing-spaces

Found: 1 error(s), 1 warning(s)
```

### Output Formats

- **default**: Colored, human-readable format (shown above)
- **JSON**: Machine-readable JSON output
- **GitHub**: GitHub Actions annotation format

### Controlling Colors

Use `--color` to control colored output:

```bash
# Auto-detect (default) - colors when terminal supports it
mdbook-lint lint docs/

# Always use colors (useful for CI with color support)
mdbook-lint lint --color always docs/

# Never use colors (useful for piping to files)
mdbook-lint lint --color never docs/ > report.txt
```

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
