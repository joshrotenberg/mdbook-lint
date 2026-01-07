# Introduction

Welcome to mdbook-lint, a fast and comprehensive markdown linter designed specifically for mdBook projects.

## What is mdbook-lint

mdbook-lint is a command-line tool and mdBook preprocessor that helps you maintain
high-quality markdown documentation by detecting common issues, enforcing consistent
style, and providing mdBook-specific linting rules.

## Key Features

- **Fast Performance**: Built in Rust for speed and efficiency
- **Comprehensive Rule Set**: 55 standard markdown rules, 18 mdBook-specific rules, and 10 content rules (83 total)
- **Flexible Integration**: Works as a standalone CLI tool or as an mdBook preprocessor
- **Rustdoc Linting**: Lint module-level documentation (`//!` comments) in Rust source files
- **Configurable**: Customize rules and behavior through configuration files
- **Zero Dependencies**: Self-contained binary with no external dependencies

## Why Use mdbook-lint

Documentation quality matters. Consistent, well-formatted markdown makes your documentation:

- **More readable** for contributors and users
- **Easier to maintain** across large documentation projects
- **More professional** in appearance and structure
- **Less prone to rendering issues** in mdBook

## Getting Started

Ready to improve your documentation quality? Head over to the
[Installation](./installation.md) guide to get started, or jump straight to
[Getting Started](./getting-started.md) for a quick walkthrough.

## Community and Support

mdbook-lint is open source and welcomes contributions. Visit our [GitHub repository](https://github.com/joshrotenberg/mdbook-lint) to:

- Report issues
- Request features
- Contribute code
- Browse the source

For development information, see our [Contributing](./contributing.md) guide.

## Acknowledgments

mdbook-lint builds on the excellent work of:

- [markdownlint](https://github.com/DavidAnson/markdownlint) - The original Node.js markdown linter that defined the standard rule set (MD001-MD059)
- [rumdl](https://github.com/rvben/rumdl) - A fast Rust markdown linter that inspired our implementation approach

We aim to be compatible with markdownlint's rule definitions while adding mdBook-specific functionality.
