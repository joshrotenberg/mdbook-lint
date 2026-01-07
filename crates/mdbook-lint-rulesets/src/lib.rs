//! Modular rulesets for mdbook-lint
//!
//! This crate provides comprehensive markdown linting rules for mdBook projects and general markdown files.
//! Rules are organized into two main categories, each available as an optional feature.
//!
//! # Overview
//!
//! The `mdbook-lint-rulesets` crate implements the actual linting rules used by mdbook-lint.
//! It provides:
//!
//! - **59 standard markdown rules** (MD001-MD059) based on the markdownlint specification
//! - **7 mdBook-specific rules** (MDBOOK001-MDBOOK007) for mdBook project validation
//! - **Automatic fix support** for many rules to correct issues automatically
//! - **Configurable rules** with sensible defaults
//!
//! # Features
//!
//! - `standard` (default): Standard markdown linting rules
//! - `mdbook` (default): mdBook-specific linting rules
//! - `content`: Content quality rules (CONTENT001-005) - optional, off by default
//!
//! # Rule Categories
//!
//! ## Standard Markdown Rules (MD001-MD059)
//!
//! These rules cover common markdown style and formatting issues:
//!
//! - **Heading rules** (MD001-MD003, MD018-MD025): Heading hierarchy, style, and formatting
//! - **List rules** (MD004-MD007, MD029-MD032): List formatting, indentation, and consistency
//! - **Whitespace rules** (MD009-MD012, MD027-MD028): Trailing spaces, blank lines, tabs
//! - **Link rules** (MD034, MD039, MD042): URL formatting and link text
//! - **Code rules** (MD038, MD040, MD046, MD048): Code block formatting and fencing
//! - **Emphasis rules** (MD036-MD037, MD049-MD050): Bold and italic formatting
//!
//! ## mdBook-Specific Rules (MDBOOK001-MDBOOK007)
//!
//! These rules validate mdBook-specific requirements:
//!
//! - **MDBOOK001**: Code blocks should have language tags
//! - **MDBOOK002**: SUMMARY.md should follow mdBook structure
//! - **MDBOOK003**: Internal links should be valid
//! - **MDBOOK004**: Part titles should be formatted correctly
//! - **MDBOOK005**: Chapter paths should be relative
//! - **MDBOOK006**: Draft chapters should have content or be marked
//! - **MDBOOK007**: Separator syntax should be correct
//!
//! # Usage
//!
//! ## Basic Setup
//!
//! ```rust
//! use mdbook_lint_rulesets::{StandardRuleProvider, MdBookRuleProvider};
//! use mdbook_lint_core::PluginRegistry;
//!
//! let mut registry = PluginRegistry::new();
//!
//! // Register standard markdown rules
//! registry.register_provider(Box::new(StandardRuleProvider))?;
//!
//! // Register mdBook-specific rules
//! registry.register_provider(Box::new(MdBookRuleProvider))?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Using with the Lint Engine
//!
//! ```rust
//! use mdbook_lint_core::{PluginRegistry, Document};
//! use mdbook_lint_rulesets::StandardRuleProvider;
//! use std::path::PathBuf;
//!
//! // Create registry and register provider
//! let mut registry = PluginRegistry::new();
//! registry.register_provider(Box::new(StandardRuleProvider))?;
//!
//! // Create engine with registered rules
//! let engine = registry.create_engine()?;
//!
//! // Lint a document
//! let content = "# My Document\n\n## Section\n";
//! let doc = Document::new(content.to_string(), PathBuf::from("README.md"))?;
//! let violations = engine.lint_document(&doc)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Automatic Fixes
//!
//! Many rules support automatic fixing of violations. Rules with fix support include:
//!
//! - **MD009**: Remove trailing spaces
//! - **MD010**: Replace hard tabs with spaces
//! - **MD012**: Remove multiple consecutive blank lines
//! - **MD018**: Add space after hash in ATX headings
//! - **MD019**: Fix multiple spaces after hash
//! - **MD020**: Remove spaces inside closed ATX headings
//! - **MD021**: Fix multiple spaces inside closed ATX headings
//! - **MD023**: Remove indentation from headings
//! - **MD027**: Fix multiple spaces after blockquote symbol
//! - **MD030**: Fix spaces after list markers
//! - **MD034**: Wrap bare URLs in angle brackets
//! - **MD047**: Ensure files end with single newline
//!
//! # Configuration
//!
//! Rules can be configured through the `RuleConfig` trait. Example configuration:
//!
//! ```toml
//! [rules.MD013]
//! line_length = 120
//! code_blocks = false
//!
//! [rules.MD009]
//! br_spaces = 2
//! strict = false
//! ```

// Re-export core types for convenience
pub use mdbook_lint_core::{
    Document, Result, Rule, RuleCategory, RuleMetadata, RuleProvider, RuleRegistry, RuleStability,
    Violation,
};

// Standard markdown rules
#[cfg(feature = "standard")]
pub mod standard;
#[cfg(feature = "standard")]
pub use standard::StandardRuleProvider;

// mdBook-specific rules
#[cfg(feature = "mdbook")]
pub mod mdbook;
#[cfg(feature = "mdbook")]
pub use mdbook::MdBookRuleProvider;

// Content quality rules (optional, off by default)
#[cfg(feature = "content")]
pub mod content;
#[cfg(feature = "content")]
pub use content::ContentRuleProvider;
