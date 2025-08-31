# Installation

mdbook-lint can be installed through several methods depending on your needs.

## From Crates.io (Recommended)

The easiest way to install mdbook-lint is through Cargo:

```bash
cargo install mdbook-lint
```

This will install the latest stable version from [crates.io](https://crates.io/crates/mdbook-lint).

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
