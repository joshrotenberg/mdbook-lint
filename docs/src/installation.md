# Installation

mdbook-lint can be installed through several methods depending on your needs.

## Homebrew (macOS/Linux)

If you use Homebrew, you can install mdbook-lint from the tap:

```bash
brew tap joshrotenberg/brew
brew install mdbook-lint
```

## From Crates.io

Install via Cargo from [crates.io](https://crates.io/crates/mdbook-lint):

```bash
cargo install mdbook-lint
```

By default, this includes all rule sets:
- **standard** - 55 markdown syntax rules (MD001-MD060)
- **mdbook** - 18 mdBook-specific rules (MDBOOK001-MDBOOK025)
- **content** - 10 content quality rules (CONTENT001-CONTENT011)

To install without specific rule sets:

```bash
# Without content rules
cargo install mdbook-lint --no-default-features --features standard,mdbook,lsp

# Only standard markdown rules
cargo install mdbook-lint --no-default-features --features standard,lsp
```

## From Source

To install the latest development version or contribute to the project:

```bash
git clone https://github.com/joshrotenberg/mdbook-lint.git
cd mdbook-lint
cargo install --path .
```

## Pre-built Binaries

Pre-built binaries for common platforms are available on the [GitHub releases page](https://github.com/joshrotenberg/mdbook-lint/releases).

Download the appropriate binary for your platform and add it to your PATH.

## Requirements

- Rust 1.88 or later (if building from source)
- No runtime dependencies required

## Verification

After installation, verify that mdbook-lint is working correctly:

```bash
mdbook-lint --version
```

You should see output similar to:

```
mdbook-lint 0.1.0
```

## Next Steps

Once installed, head to the [Getting Started](./getting-started.md) guide to learn how to use mdbook-lint with your projects.
