# mdbook-lint

[![Crates.io](https://img.shields.io/crates/v/mdbook-lint.svg)](<https://crates.io/crates/mdbook-lint>)
[![Documentation](https://docs.rs/mdbook-lint/badge.svg)](<https://docs.rs/mdbook-lint>)
[![Build Status](https://github.com/joshrotenberg/mdbook-lint/workflows/CI/badge.svg)](<https://github.com/joshrotenberg/mdbook-lint/actions>)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](<https://github.com/joshrotenberg/mdbook-lint#license>)

A fast, configurable linter designed for [mdBook](https://rust-lang.github.io/mdBook/) projects. Works as both an mdBook preprocessor and standalone CLI tool.

**[Documentation](https://joshrotenberg.github.io/mdbook-lint/)** | **[Getting Started](https://joshrotenberg.github.io/mdbook-lint/getting-started.html)**

## What is mdBook?

[mdBook](https://rust-lang.github.io/mdBook/) is a command-line tool for creating books from Markdown files. It's widely used in the Rust ecosystem for documentation, including [The Rust Programming Language](https://doc.rust-lang.org/book/) book, and is popular for technical documentation projects of all kinds. mdBook renders Markdown into a clean, searchable HTML book format with navigation, search, and syntax highlighting.

mdbook-lint helps ensure your mdBook documentation maintains consistent quality by catching common issues before they reach readers.

## Installation

### Homebrew (macOS/Linux)

```bash
brew tap joshrotenberg/brew
brew install mdbook-lint
```

### From Cargo

```bash
cargo install mdbook-lint
```

By default, this includes all rule sets (standard, mdBook, and content rules). To install without specific rule sets:

```bash
# Without content rules (CONTENT001-011)
cargo install mdbook-lint --no-default-features --features standard,mdbook,lsp

# Only standard markdown rules
cargo install mdbook-lint --no-default-features --features standard,lsp
```

### From Prebuilt Binaries

Download the latest release for your platform from [GitHub Releases](https://github.com/joshrotenberg/mdbook-lint/releases):

- **Linux (x86_64)**: `mdbook-lint-linux-x86_64`
- **Linux (musl)**: `mdbook-lint-linux-x86_64-musl` (static binary, no dependencies)
- **Windows**: `mdbook-lint-windows-x86_64.exe`
- **macOS (Intel)**: `mdbook-lint-macos-x86_64`
- **macOS (Apple Silicon)**: `mdbook-lint-macos-aarch64`

Extract and add to your PATH, or use with GitHub Actions (see [CI Integration](#ci-integration)).

Verify the installation:

```bash
mdbook-lint --version
```

## Features

- **Native mdBook integration** - Seamless preprocessor integration
- **83 linting rules** - 55 standard markdown + 18 mdBook-specific + 10 content rules  
- **Auto-fix support** - Automatically fix common issues with 41 rules
- **Fast performance** - Lint entire books in seconds
- **Configurable** - Disable rules, set custom parameters
- **Cross-platform** - Prebuilt binaries for all major platforms

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

# Auto-fix violations (using the fix subcommand)
mdbook-lint fix src/*.md

# Preview what would be fixed
mdbook-lint fix --dry-run src/*.md

# Alternative: use lint with --fix flag
mdbook-lint lint --fix src/*.md

# Show available rules
mdbook-lint rules
```

Output uses cargo-style formatting with colors:

```text
error[MD001]: Expected heading level 2 but got level 3
  --> src/chapter.md:15:1
     |
  15 | ### Skipped heading level
     | ^^^ heading-increment
```

## Configuration

Create a `.mdbook-lint.toml` file (also supports YAML/JSON):

```toml
# Disable rules that don't fit your project
disabled-rules = ["MD013", "MD033"]

# Configure specific rules
[MD007]
indent = 4

[MD009]
br_spaces = 2  # Allow 2 trailing spaces for line breaks

[MD003]
style = "atx"  # Use # style headings
```

Generate a configuration file with all options documented:

```bash
mdbook-lint init --include-all
```

**Configuration examples:**

- [example-mdbook-lint.toml](https://github.com/joshrotenberg/mdbook-lint/blob/main/crates/mdbook-lint-cli/example-mdbook-lint.toml) - Comprehensive reference with all 83 rules documented
- [docs/.mdbook-lint.toml](https://github.com/joshrotenberg/mdbook-lint/blob/main/docs/.mdbook-lint.toml) - Real-world example used by this project's documentation

## Rules

- **55 standard rules** (MD001-MD060) - All the usual markdown linting
- **18 mdBook rules** (MDBOOK001-MDBOOK025) - mdBook-specific checks
- **10 content rules** (CONTENT001-CONTENT011) - Content quality checks including TODO detection, placeholder text, terminology consistency, link quality, and more

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

## Compatibility

mdbook-lint supports mdBook versions 0.4.x and 0.5.x. The tool automatically detects and handles differences in mdBook's JSON protocol between versions, so it works seamlessly regardless of which mdBook version you have installed.

For detailed compatibility information, see the [Compatibility Guide](https://joshrotenberg.github.io/mdbook-lint/compatibility.html).

## Contributing

Contributions are welcome! See our [Contributing Guide](https://joshrotenberg.github.io/mdbook-lint/contributing.html) for complete information.

## Acknowledgments

mdbook-lint builds on the excellent work of:

- [markdownlint](https://github.com/DavidAnson/markdownlint) - The original Node.js markdown linter that defined the standard rule set (MD001-MD059)
- [rumdl](https://github.com/rvben/rumdl) - A fast Rust markdown linter that inspired our implementation approach

We aim to be compatible with markdownlint's rule definitions while adding mdBook-specific functionality.

## License

MIT OR Apache-2.0
