//! Modular rulesets for mdbook-lint
//!
//! This crate provides rule implementations organized by category:
//! - Standard markdown rules (MD001-MD059) under the `standard` feature
//! - mdBook-specific rules (MDBOOK001-007) under the `mdbook` feature
//!
//! # Features
//!
//! - `standard` (default): Standard markdown linting rules
//! - `mdbook` (default): mdBook-specific linting rules
//!
//! # Usage
//!
//! ```rust
//! use mdbook_lint_rulesets::StandardRuleProvider;
//! use mdbook_lint_core::PluginRegistry;
//!
//! let mut registry = PluginRegistry::new();
//! registry.register_provider(Box::new(StandardRuleProvider))?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
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
