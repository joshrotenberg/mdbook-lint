# API Documentation

This page provides comprehensive API documentation for mdbook-lint's core libraries and rule implementations.

## Core Library (mdbook-lint-core)

The `mdbook-lint-core` crate provides the foundational infrastructure for markdown linting.

{{#rustdoc_include ../../crates/mdbook-lint-core/src/lib.rs:6:14}}

### Architecture

{{#rustdoc_include ../../crates/mdbook-lint-core/src/lib.rs:16:36}}

### Key Types

{{#rustdoc_include ../../crates/mdbook-lint-core/src/lib.rs:88:132}}

### Configuration

{{#rustdoc_include ../../crates/mdbook-lint-core/src/lib.rs:143:160}}

## Rulesets Library (mdbook-lint-rulesets)

The `mdbook-lint-rulesets` crate implements all linting rules.

{{#rustdoc_include ../../crates/mdbook-lint-rulesets/src/lib.rs:6:13}}

### Rule Categories

{{#rustdoc_include ../../crates/mdbook-lint-rulesets/src/lib.rs:20:43}}

### Automatic Fixes

{{#rustdoc_include ../../crates/mdbook-lint-rulesets/src/lib.rs:81:96}}

## Individual Rule Documentation

Each rule includes comprehensive documentation with examples and configuration options.

### Example: MD001 - Heading Increment

{{#rustdoc_include ../../crates/mdbook-lint-rulesets/src/standard/md001.rs:9:57}}

### Example: MD009 - Trailing Spaces

{{#rustdoc_include ../../crates/mdbook-lint-rulesets/src/standard/md009.rs:1:52}}

### Example: MD010 - Hard Tabs

{{#rustdoc_include ../../crates/mdbook-lint-rulesets/src/standard/md010.rs:1:52}}

### Example: MDBOOK001 - Code Block Language Tags

{{#rustdoc_include ../../crates/mdbook-lint-rulesets/src/mdbook/mdbook001.rs:8:83}}

## Full API Reference

For complete API documentation with all types, traits, and functions, run:

```bash
cargo doc --open
```

This will generate and open the full rustdoc documentation in your browser.