# mdbook-lint

[![Crates.io](https://img.shields.io/crates/v/mdbook-lint.svg)](https://crates.io/crates/mdbook-lint)
[![Documentation](https://docs.rs/mdbook-lint/badge.svg)](https://docs.rs/mdbook-lint)
[![Build Status](https://github.com/joshrotenberg/mdbook-lint/workflows/CI/badge.svg)](https://github.com/joshrotenberg/mdbook-lint/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/joshrotenberg/mdbook-lint#license)

A fast linter for mdBook projects.

📖 **[Documentation](https://joshrotenberg.github.io/mdbook-lint/)** | 🚀 **[Getting Started](https://joshrotenberg.github.io/mdbook-lint/getting-started.html)**

## Installation

```bash
cargo install mdbook-lint
```

## Usage

### CLI

```bash
# Lint files
mdbook-lint lint README.md src/*.md

# Show available rules
mdbook-lint rules
```

### mdBook Preprocessor

Add to your `book.toml`:

```toml
[preprocessor.mdbook-lint]
```

Then run `mdbook build` as usual.

## Configuration

Create a `.mdbook-lint.toml` file:

```toml
fail-on-warnings = true
disabled-rules = ["MD013"]  # Disable line length rule
```

## Rules

- **59 standard rules** (MD001-MD059) - All the usual markdown linting
- **4 mdBook rules** (MDBOOK001-004) - mdBook-specific checks

Run `mdbook-lint rules --detailed` to see all available rules.

## License

MIT OR Apache-2.0
