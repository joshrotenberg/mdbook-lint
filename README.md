# mdbook-lint

[![Crates.io](https://img.shields.io/crates/v/mdbook-lint.svg)](https://crates.io/crates/mdbook-lint)
[![Documentation](https://docs.rs/mdbook-lint/badge.svg)](https://docs.rs/mdbook-lint)
[![Build Status](https://github.com/joshrotenberg/mdbook-lint/workflows/CI/badge.svg)](https://github.com/joshrotenberg/mdbook-lint/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/joshrotenberg/mdbook-lint#license)

A fast, configurable linter for mdBook projects.

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

## Usage

### CLI

```bash
# Lint files
mdbook-lint lint README.md src/*.md

# Auto-fix violations where possible
mdbook-lint lint --fix src/*.md

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
- **7 mdBook rules** (MDBOOK001-007) - mdBook-specific checks

Run `mdbook-lint rules --detailed` to see all available rules.

## Contributing

Contributions are welcome! See our [Contributing Guide](https://joshrotenberg.github.io/mdbook-lint/contributing.html) for complete information.

## License

MIT OR Apache-2.0
