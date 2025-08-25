# API Documentation

This page provides comprehensive API documentation for mdbook-lint's core libraries and rule implementations.

## Core Library (mdbook-lint-core)

The `mdbook-lint-core` crate provides the foundational infrastructure for markdown linting.

### Overview

The `mdbook-lint-core` crate provides:
- **Plugin-based architecture** for extensible rule sets
- **AST and text-based linting** with efficient document processing
- **Violation reporting** with detailed position tracking and severity levels
- **Automatic fix infrastructure** for correctable violations
- **Configuration system** for customizing rule behavior
- **Document abstraction** with markdown parsing via comrak

### Architecture

The core follows a plugin-based architecture where rules are provided by external crates:

```text
┌─────────────────┐
│   Application   │
└────────┬────────┘
         │
┌────────▼────────┐
│  PluginRegistry │ ◄─── Registers rule providers
└────────┬────────┘
         │
┌────────▼────────┐
│   LintEngine    │ ◄─── Orchestrates linting
└────────┬────────┘
         │
┌────────▼────────┐
│     Rules       │ ◄─── Individual rule implementations
└─────────────────┘
```

### Key Components

#### Document Processing
- `Document` - Represents a markdown file with content and metadata
- `Position` - Tracks line/column positions for violations
- `NodeContext` - Provides AST node information during linting

#### Rule System
- `Rule` trait - Core interface for all linting rules
- `AstRule` - Rules that process markdown AST nodes
- `TextRule` - Rules that process raw text content
- `RuleProvider` - Plugin interface for rule registration

#### Violation Reporting
- `Violation` - Represents a linting issue with location and severity
- `Severity` - Error, Warning, or Info levels
- `Fix` - Automatic fix information for violations

#### Engine and Registry
- `LintEngine` - Main orchestrator for document linting
- `PluginRegistry` - Manages rule providers and configuration

## Rulesets Library (mdbook-lint-rulesets)

The `mdbook-lint-rulesets` crate implements all linting rules.

### Overview

The `mdbook-lint-rulesets` crate implements the actual linting rules used by mdbook-lint.
It provides:
- **59 standard markdown rules** (MD001-MD059) based on the markdownlint specification
- **13 mdBook-specific rules** (MDBOOK001-MDBOOK012, MDBOOK025) for mdBook project validation
- **Automatic fix support** for many rules to correct issues automatically
- **Configurable rules** with sensible defaults

### Rule Categories

#### Standard Markdown Rules (MD001-MD059)

These rules cover common markdown style and formatting issues:

- **Heading rules** (MD001-MD003, MD018-MD025): Heading hierarchy, style, and formatting
- **List rules** (MD004-MD007, MD029-MD032): List formatting, indentation, and consistency
- **Whitespace rules** (MD009-MD012, MD027-MD028): Trailing spaces, blank lines, tabs
- **Link rules** (MD034, MD039, MD042): URL formatting and link text
- **Code rules** (MD038, MD040, MD046, MD048): Code block formatting and fencing
- **Emphasis rules** (MD036-MD037, MD049-MD050): Bold and italic formatting

#### mdBook-Specific Rules (MDBOOK001-MDBOOK012, MDBOOK025)

These rules validate mdBook-specific requirements:

- **MDBOOK001**: Code blocks should have language tags for proper syntax highlighting
- **MDBOOK002**: Validate internal link paths and anchors
- **MDBOOK003**: SUMMARY.md should follow proper mdBook structure
- **MDBOOK005**: Detect orphaned files not referenced in SUMMARY.md
- **MDBOOK006**: Validate cross-reference links between chapters
- **MDBOOK007**: Validate file include syntax and paths
- **MDBOOK008**: Check rustdoc_include directive usage
- **MDBOOK009**: Validate playground directive syntax
- **MDBOOK010**: Check for invalid math block syntax
- **MDBOOK011**: Validate template include syntax
- **MDBOOK012**: Check file include range syntax
- **MDBOOK025**: Ensure proper heading structure in SUMMARY.md

### Automatic Fixes

Many rules support automatic fixing to correct violations:

#### Whitespace and Formatting
- **MD009**: Remove trailing spaces while preserving line breaks
- **MD010**: Convert tabs to spaces with configurable spacing
- **MD012**: Remove excessive consecutive blank lines
- **MD018**: Add space after hash in headings
- **MD019**: Remove multiple spaces after hash in headings
- **MD020**: Remove spaces inside hash-surrounded headings  
- **MD021**: Remove multiple spaces inside hash-surrounded headings
- **MD023**: Remove indentation from headings
- **MD027**: Remove spaces after blockquote markers
- **MD030**: Ensure proper spacing after list markers
- **MD034**: Replace bare URLs with proper link syntax
- **MD047**: Ensure files end with single newline

#### Coming Soon
- Additional formatting rules
- More sophisticated content restructuring
- Context-aware fixes for complex violations

## Usage Examples

### Basic Library Usage

```rust
use mdbook_lint_core::{PluginRegistry, LintEngine};
use mdbook_lint_rulesets::{StandardRuleProvider, MdBookRuleProvider};

// Create a registry with standard rules
let mut registry = PluginRegistry::new();
registry.register(StandardRuleProvider::new())?;
registry.register(MdBookRuleProvider::new())?;

// Create engine and lint a document
let engine = LintEngine::from_registry(registry);
let document = Document::from_file("README.md")?;
let violations = engine.lint_document(&document)?;
```

### Custom Rule Implementation

```rust
use mdbook_lint_core::{Document, AstRule, Violation, Position};
use comrak::nodes::{AstNode, NodeValue};

pub struct MyCustomRule;

impl AstRule for MyCustomRule {
    fn id(&self) -> &'static str { "CUSTOM001" }
    fn description(&self) -> &'static str { "My custom rule" }
    
    fn lint_ast(&self, document: &Document, node: &AstNode) -> Vec<Violation> {
        // Your custom linting logic here
        Vec::new()
    }
}
```

### Configuration Usage

```rust
use mdbook_lint_core::Config;

let config = Config::from_file(".mdbook-lint.toml")?;
let engine = LintEngine::from_config(config)?;
```

## Individual Rule Documentation

Each rule provides detailed documentation including:
- **Purpose and rationale** - Why the rule exists
- **Examples** - Correct and incorrect markdown
- **Configuration options** - Customizable behavior
- **Automatic fixes** - What fixes are available
- **Related rules** - Connected or overlapping rules

### Featured Rules

#### [MD001 - Heading Increment](./rules/standard/md001.html)
Ensures heading levels increment sequentially for proper document structure.

#### [MD009 - No Trailing Spaces](./rules/standard/md009.html) ⚡ Auto-fix
Removes trailing whitespace while preserving intentional line breaks.

#### [MDBOOK001 - Code Block Language Tags](./rules/mdbook/mdbook001.html)
Ensures code blocks have language tags for proper syntax highlighting in mdBook.

### Quick Rule Reference

| Rule | Description | Auto-fix |
|------|-------------|----------|
| MD001 | Heading increment | |
| MD009 | No trailing spaces | ⚡ |
| MD010 | Hard tabs | ⚡ |
| MD012 | Multiple blank lines | ⚡ |
| MD018 | No space after hash | ⚡ |
| MD019 | Multiple spaces after hash | ⚡ |
| MD020 | No space in closed headings | ⚡ |
| MD021 | Multiple spaces in closed headings | ⚡ |
| MD023 | Headings start at beginning | ⚡ |
| MD027 | Multiple spaces after blockquote | ⚡ |
| MD030 | Spaces after list markers | ⚡ |
| MD034 | Bare URL used | ⚡ |
| MD047 | Files should end with newline | ⚡ |

*⚡ indicates automatic fix support*

## Full API Reference

For complete API documentation with all types, traits, and functions:

### Generate Documentation
```bash
# Generate and open documentation
cargo doc --open

# Generate documentation for all features
cargo doc --all-features --open

# Generate documentation without dependencies
cargo doc --no-deps --open
```

### Online Documentation
- **crates.io**: [mdbook-lint-core](https://docs.rs/mdbook-lint-core), [mdbook-lint-rulesets](https://docs.rs/mdbook-lint-rulesets)
- **Repository**: [GitHub documentation](https://joshrotenberg.github.io/mdbook-lint/)

## Integration Patterns

### mdBook Preprocessor
```toml
# book.toml
[preprocessor.mdbook-lint]
fail-on-warnings = true
```

### CI/CD Integration
```bash
# Fail build on any violations
mdbook-lint lint --fail-on-warnings docs/

# Auto-fix and commit changes
mdbook-lint lint --fix docs/
git add docs/
git commit -m "docs: auto-fix markdown violations"
```

### Editor Integration
Configure your editor to run mdbook-lint on save for real-time feedback.