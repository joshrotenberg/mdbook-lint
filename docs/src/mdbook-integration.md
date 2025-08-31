# mdBook Integration

mdbook-lint integrates seamlessly with mdBook as a preprocessor, automatically checking your markdown files during the build process. This guide provides comprehensive documentation for configuring and using mdbook-lint with mdBook projects.

## Table of Contents

- [Installation](#installation)
- [Basic Setup](#basic-setup)
- [Configuration Options](#configuration-options)
- [Advanced Configuration](#advanced-configuration)
- [GitHub Actions Integration](#github-actions-integration)
- [Troubleshooting](#troubleshooting)
- [Common Scenarios](#common-scenarios)

> **Note:** If you're unsure whether to use mdbook-lint as a preprocessor or standalone in CI, see our [CI vs Preprocessor](./ci-vs-preprocessor.md) guide.

## Installation

### Using Cargo (Recommended)

```bash
cargo install mdbook-lint
```

### Using Pre-built Binaries

Download the latest release from [GitHub Releases](https://github.com/joshrotenberg/mdbook-lint/releases):

```bash
# Linux/macOS
curl -L https://github.com/joshrotenberg/mdbook-lint/releases/latest/download/mdbook-lint-$(uname -s)-$(uname -m).tar.gz | tar xz
sudo mv mdbook-lint /usr/local/bin/

# Windows (PowerShell)
Invoke-WebRequest -Uri https://github.com/joshrotenberg/mdbook-lint/releases/latest/download/mdbook-lint-Windows-x86_64.zip -OutFile mdbook-lint.zip
Expand-Archive mdbook-lint.zip -DestinationPath .
```

### Using GitHub Action

```yaml
- name: Install mdbook-lint
  uses: joshrotenberg/mdbook-lint-action@v1
```

## Basic Setup

### Minimal Configuration

Add mdbook-lint to your `book.toml`:

```toml
[preprocessor.mdbook-lint]
```

This enables mdbook-lint with default settings. It will:
- Run all standard markdown rules (MD001-MD059)
- Run all mdBook-specific rules (MDBOOK001-MDBOOK025)
- Report violations as warnings (won't fail the build)

### Running mdBook with Linting

```bash
# Build with linting
mdbook build

# Serve with live reload and linting
mdbook serve

# Test to verify everything works
mdbook test
```

## Configuration Options

### Complete Configuration Example

```toml
[preprocessor.mdbook-lint]
# Control build behavior
fail-on-warnings = false  # Set to true to fail builds on any violation

# Rule configuration
disabled-rules = ["MD013", "MD033", "MD041"]  # Disable specific rules
enabled-rules = []  # If set, ONLY these rules will run

# Rule categories (groups of rules)
disabled-categories = ["whitespace", "html"]  # Disable entire categories
enabled-categories = []  # If set, ONLY these categories will run

# File filtering
include = ["src/**/*.md"]  # Only lint these files (glob patterns)
exclude = ["src/drafts/**", "src/archive/**"]  # Exclude these files

# Output configuration
output = "concise"  # Options: "concise", "detailed", "json"

# Rule-specific configuration
[preprocessor.mdbook-lint.rules]
# Configure individual rules
MD013 = { line_length = 100 }
MD024 = { siblings_only = true }
MD026 = { punctuation = ".,;:!" }
```

### Configuration Through External File

You can also use a separate `.mdbook-lint.toml` file in your project root:

```toml
# .mdbook-lint.toml
[core]
fail_on_warnings = true

[rules]
# Default state for all rules
default = true

[rules.disabled]
MD013 = true  # Line length
MD033 = true  # Inline HTML
MD041 = true  # First line heading

[rules.config]
MD013 = { line_length = 100 }
MD024 = { siblings_only = true }
```

## Advanced Configuration

### Rule Categories

Rules are organized into categories for easier management:

```toml
[preprocessor.mdbook-lint]
# Disable all whitespace-related rules
disabled-categories = ["whitespace"]

# Or enable only specific categories
enabled-categories = ["headings", "links", "code"]
```

Available categories:
- `headings` - Heading structure and formatting
- `lists` - List formatting and consistency
- `whitespace` - Spacing, indentation, line breaks
- `code` - Code blocks and inline code
- `links` - Link validation and formatting
- `html` - HTML usage in markdown
- `mdbook` - mdBook-specific rules

### Custom Rule Sets

Define different rule sets for different environments:

```toml
# Development: Lenient settings
[preprocessor.mdbook-lint]
fail-on-warnings = false
disabled-rules = ["MD013", "MD033", "MD041"]

# For CI, use environment variables to override:
# MDBOOK_PREPROCESSOR__MDBOOK_LINT__FAIL_ON_WARNINGS=true mdbook build
```

### Per-File Rule Overrides

Use HTML comments in your markdown files to disable rules:

```markdown
<!-- mdbook-lint-disable MD013 MD033 -->
This content won't be checked for line length or HTML usage.
<!-- mdbook-lint-enable MD013 MD033 -->

<!-- mdbook-lint-disable-next-line MD001 -->
### This heading can skip levels
```

## GitHub Actions Integration

### Basic Workflow

```yaml
name: Documentation

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  lint-and-build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Cache cargo dependencies
        uses: Swatinem/rust-cache@v2
      
      - name: Install mdbook and mdbook-lint
        run: |
          cargo install mdbook
          cargo install mdbook-lint
      
      - name: Build documentation
        run: mdbook build
        env:
# Override settings for CI

          MDBOOK_PREPROCESSOR__MDBOOK_LINT__FAIL_ON_WARNINGS: true
      
      - name: Deploy to GitHub Pages
        if: github.ref == 'refs/heads/main' && github.event_name == 'push'
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./book
```

### Using mdbook-lint-action

```yaml
name: Lint Documentation

on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Lint markdown files
        uses: joshrotenberg/mdbook-lint-action@v1
        with:
          format: sarif
          output-file: results.sarif
          config: .mdbook-lint.toml
      
      - name: Upload SARIF results
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: results.sarif
```

### Matrix Testing with Different Configurations

```yaml
name: Test Documentation

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        lint-config:
          - strict
          - standard
          - lenient
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install dependencies
        run: |
          cargo install mdbook
          cargo install mdbook-lint
      
      - name: Test with ${{ matrix.lint-config }} configuration
        run: mdbook build
        env:
          MDBOOK_PREPROCESSOR__MDBOOK_LINT__CONFIG: .mdbook-lint.${{ matrix.lint-config }}.toml
```

## Troubleshooting

### Common Issues and Solutions

#### Preprocessor Not Running

**Problem**: mdbook-lint doesn't seem to run during builds.

**Solutions**:
1. Verify installation:
   ```bash
   mdbook-lint --version
   which mdbook-lint
   ```

2. Check `book.toml` has the preprocessor section:
   ```toml
   [preprocessor.mdbook-lint]
   ```

3. Run with verbose output:
   ```bash
   mdbook build -v 2>&1 | grep mdbook-lint
   ```

4. Ensure mdbook-lint is in PATH:
   ```bash
   export PATH="$HOME/.cargo/bin:$PATH"
   ```

#### Configuration Not Applied

**Problem**: Settings in `book.toml` aren't being used.

**Configuration Precedence** (highest to lowest):
1. Environment variables (e.g., `MDBOOK_PREPROCESSOR__MDBOOK_LINT__FAIL_ON_WARNINGS`)
2. `book.toml` preprocessor settings
3. `.mdbook-lint.toml` file in project root
4. Built-in defaults

**Debug Configuration**:
```bash
# Check what configuration is being used
mdbook-lint lint --debug-config src/chapter1.md

# Test with explicit config
mdbook-lint lint --config .mdbook-lint.toml src/
```

#### Build Fails Unexpectedly

**Problem**: Build fails even with `fail-on-warnings = false`.

**Check for**:
1. Syntax errors in configuration files
2. Invalid rule IDs in enabled/disabled lists
3. Conflicting configuration sources

**Debug Steps**:
```bash
# Run linter standalone to see all issues
mdbook-lint lint src/

# Check preprocessor output
mdbook build 2>&1 | grep -A 5 "mdbook-lint"
```

#### Performance Issues

**Problem**: Builds are slow with mdbook-lint enabled.

**Solutions**:
1. Exclude unnecessary files:
   ```toml
   [preprocessor.mdbook-lint]
   exclude = ["src/generated/**", "src/vendor/**"]
   ```

2. Disable expensive rules:
   ```toml
   disabled-rules = ["MD013", "MD053"]  # Line length checks can be slow
   ```

3. Use caching in CI:
   ```yaml
   - uses: actions/cache@v3
     with:
       path: ~/.cargo
       key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
   ```

## Common Scenarios

### Scenario 1: Strict CI, Lenient Local Development

**Local `book.toml`**:
```toml
[preprocessor.mdbook-lint]
fail-on-warnings = false
disabled-rules = ["MD013"]  # Allow long lines locally
```

**CI Override**:
```yaml
- name: Build with strict linting
  run: mdbook build
  env:
    MDBOOK_PREPROCESSOR__MDBOOK_LINT__FAIL_ON_WARNINGS: true
    MDBOOK_PREPROCESSOR__MDBOOK_LINT__DISABLED_RULES: ""
```

### Scenario 2: Different Rules for Different Chapters

**book.toml**:
```toml
# Strict rules for main content
[preprocessor.mdbook-lint]
include = ["src/chapters/**"]
fail-on-warnings = true

# Separate preprocessor for examples (lenient)
[preprocessor.mdbook-lint-examples]
command = "mdbook-lint"
renderer = ["html"]
include = ["src/examples/**"]
disabled-rules = ["MD013", "MD033", "MD041"]
```

### Scenario 3: Progressive Rule Adoption

Start lenient and gradually enable more rules:

```toml
# Phase 1: Critical rules only
[preprocessor.mdbook-lint]
enabled-rules = ["MDBOOK001", "MDBOOK002", "MDBOOK003", "MD040", "MD041"]

# Phase 2: Add formatting rules
# enabled-rules = ["MDBOOK001", "MDBOOK002", "MDBOOK003", "MD040", "MD041", "MD001", "MD003", "MD009"]

# Phase 3: Full rule set
# Comment out enabled-rules to run all rules
```

### Scenario 4: Integration with Code Quality Tools

```yaml
name: Documentation Quality

on: [push, pull_request]

jobs:
  quality-checks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
# Markdown linting

      - name: Lint markdown
        uses: joshrotenberg/mdbook-lint-action@v1
        with:
          format: sarif
          output-file: mdbook-lint.sarif
      
# Spell checking

      - name: Spell check
        uses: streetsidesoftware/cspell-action@v2
      
# Link checking

      - name: Check links
        uses: lycheeverse/lychee-action@v1
        with:
          args: --verbose --no-progress './book/**/*.html'
      
# Upload all results

      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: mdbook-lint.sarif
```

## Configuration Discovery

mdbook-lint automatically searches for configuration files in the following order:

1. Explicit `--config` flag (CLI mode only)
2. Environment variable: `MDBOOK_LINT_CONFIG`
3. `book.toml` preprocessor configuration
4. Configuration file discovery (searches up the directory tree):
   - `.mdbook-lint.toml`
   - `mdbook-lint.toml`
   - `.mdbook-lint.yaml`
   - `.mdbook-lint.yml`
   - `.mdbook-lint.json`

The first configuration found is used. Settings from multiple sources are not merged.

## Environment Variables

All preprocessor settings can be overridden using environment variables:

```bash
# Format: MDBOOK_PREPROCESSOR__MDBOOK_LINT__<SETTING>
export MDBOOK_PREPROCESSOR__MDBOOK_LINT__FAIL_ON_WARNINGS=true
export MDBOOK_PREPROCESSOR__MDBOOK_LINT__DISABLED_RULES="MD013,MD033"
export MDBOOK_PREPROCESSOR__MDBOOK_LINT__OUTPUT=json

mdbook build
```

## Output Formats

### Concise (Default)

```
src/chapter1.md:10:1: MD041 First line in file should be a top-level heading
src/chapter2.md:25:81: MD013 Line length exceeds 80 characters
```

### Detailed

```
File: src/chapter1.md
  Line 10, Column 1: MD041 - First line in file should be a top-level heading
    The first line of the file should be a top-level (h1) heading.
    Consider adding '# Title' at the beginning of the file.
```

### JSON

```json
{
  "files": [
    {
      "path": "src/chapter1.md",
      "violations": [
        {
          "rule": "MD041",
          "line": 10,
          "column": 1,
          "severity": "warning",
          "message": "First line in file should be a top-level heading"
        }
      ]
    }
  ]
}
```

## Next Steps

- Review the [Rules Reference](./rules-reference.md) for all available rules
- Learn about [Creating Custom Rules](./contributing.md#creating-custom-rules)
- See [Configuration Reference](./configuration-reference.md) for all options
- Check out [Example Configurations](./example-configuration.md) for real-world setups
