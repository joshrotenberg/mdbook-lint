# mdbook-lint

[![Crates.io](https://img.shields.io/crates/v/mdbook-lint.svg)](<https://crates.io/crates/mdbook-lint>)
[![Documentation](https://docs.rs/mdbook-lint/badge.svg)](<https://docs.rs/mdbook-lint>)
[![Build Status](https://github.com/joshrotenberg/mdbook-lint/workflows/CI/badge.svg)](<https://github.com/joshrotenberg/mdbook-lint/actions>)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](<https://github.com/joshrotenberg/mdbook-lint#license>)

A fast, configurable linter designed for mdBook projects. Works as both an mdBook preprocessor and standalone CLI tool.

📖 **[Documentation](https://joshrotenberg.github.io/mdbook-lint/)** | 🚀 **[Getting Started](https://joshrotenberg.github.io/mdbook-lint/getting-started.html)**

## Installation

### From Prebuilt Binaries (Recommended)

Download the latest release for your platform from [GitHub Releases](https://github.com/joshrotenberg/mdbook-lint/releases):

- **Linux (x86_64)**: `mdbook-lint-linux-x86_64`
- **Linux (musl)**: `mdbook-lint-linux-x86_64-musl` (static binary, no dependencies)
- **Windows**: `mdbook-lint-windows-x86_64.exe`
- **macOS (Intel)**: `mdbook-lint-macos-x86_64`
- **macOS (Apple Silicon)**: `mdbook-lint-macos-aarch64`

Extract and add to your PATH, or use with GitHub Actions (see [CI Integration](#ci-integration)).

### From Cargo

```bash
cargo install mdbook-lint
```

Verify the installation:

```bash
mdbook-lint --version
```

## Features

- ✅ **Native mdBook integration** - Seamless preprocessor integration
- ✅ **67 linting rules** - 54 standard markdown + 13 mdBook-specific rules  
- ✅ **Auto-fix support** - Automatically fix common issues with 12 rules
- ✅ **Fast performance** - Lint entire books in seconds
- ✅ **Configurable** - Disable rules, set custom parameters
- ✅ **Cross-platform** - Prebuilt binaries for all major platforms

## Usage

### mdBook Preprocessor (Primary Use Case)

Add to your `book.toml`:

```toml
[preprocessor.mdbook-lint]
```

Then run `mdbook build` as usual. The linter will automatically check all your markdown files and report issues during the build process.

### CLI (Standalone)

```bash
# Lint files
mdbook-lint lint README.md src/*.md

# Auto-fix violations where possible
mdbook-lint lint --fix src/*.md

# Preview what would be fixed
mdbook-lint lint --fix --dry-run src/*.md

# Show available rules
mdbook-lint rules
```

## Configuration

Create a `.mdbook-lint.toml` file (also supports YAML/JSON):

```toml
# Enable only specific rules
[rules]
default = false
[rules.enabled]
MD001 = true
MD009 = true

# Or use traditional configuration
fail-on-warnings = true
disabled-rules = ["MD013"]

# Configure specific rules
[MD007]
indent = 4

[MD009]
br_spaces = 2  # Allow 2 trailing spaces for line breaks
```

See the [example configuration](https://github.com/joshrotenberg/mdbook-lint/blob/main/example-mdbook-lint.toml) for all available options.

## Rules

- **54 standard rules** (MD001-MD059) - All the usual markdown linting
- **13 mdBook rules** (MDBOOK001-012, MDBOOK025) - mdBook-specific checks

Run `mdbook-lint rules --detailed` to see all available rules.

## CI Integration

### GitHub Actions

```yaml
name: Lint Documentation
on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Download mdbook-lint
        run: |
          curl -sSL https://github.com/joshrotenberg/mdbook-lint/releases/latest/download/mdbook-lint-linux-x86_64 -o mdbook-lint
          chmod +x mdbook-lint
      
      - name: Lint markdown files
        run: ./mdbook-lint lint --fail-on-warnings docs/
```

## Contributing

Contributions are welcome! See our [Contributing Guide](https://joshrotenberg.github.io/mdbook-lint/contributing.html) for complete information.

## License

MIT OR Apache-2.0
