//! Core linting engine for mdbook-lint
//!
//! This library provides markdown linting functionality with mdBook support.
//! It includes both standard markdown rules (MD001-MD059) and mdBook-specific
//! rules (MDBOOK001-007).
//!
//! # Basic Usage
//!
//! ```rust
//! use mdbook_lint_core::{create_engine_with_all_rules, Document};
//! use std::path::PathBuf;
//!
//! let engine = create_engine_with_all_rules();
//! let document = Document::new("# Hello".to_string(), PathBuf::from("test.md"))?;
//! let violations = engine.lint_document(&document)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Custom Rule Sets
//!
//! ```rust
//! use mdbook_lint_core::{PluginRegistry, StandardRuleProvider, MdBookRuleProvider};
//!
//! let mut registry = PluginRegistry::new();
//! registry.register_provider(Box::new(StandardRuleProvider))?;
//! registry.register_provider(Box::new(MdBookRuleProvider))?;
//! let engine = registry.create_engine()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod config;
pub mod deduplication;
pub mod document;
pub mod engine;
pub mod error;
pub mod registry;
pub mod rule;
pub mod rules;
pub mod standard_provider;
#[cfg(test)]
pub mod test_helpers;
pub mod violation;

// Re-export core types for convenience
pub use config::Config;
pub use document::Document;
pub use engine::{LintEngine, PluginRegistry, RuleProvider};
pub use error::{
    ConfigError, DocumentError, ErrorContext, IntoMdBookLintError, MdBookLintError, MdlntError,
    PluginError, Result, RuleError,
};
pub use registry::RuleRegistry;
pub use rule::{AstRule, Rule, RuleCategory, RuleMetadata, RuleStability};
pub use violation::{Severity, Violation};

// Re-export rule providers
pub use rules::MdBookRuleProvider;
pub use standard_provider::StandardRuleProvider;

/// Current version of mdbook-lint-core
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Human-readable name
pub const NAME: &str = "mdbook-lint-core";

/// Description
pub const DESCRIPTION: &str = "Core linting engine for mdbook-lint";

/// Create a lint engine with all available rules (standard + mdBook)
pub fn create_engine_with_all_rules() -> LintEngine {
    let mut registry = PluginRegistry::new();

    // Register standard markdown rules (MD001-MD059)
    registry
        .register_provider(Box::new(StandardRuleProvider))
        .unwrap();

    // Register mdBook-specific rules (MDBOOK001-007)
    registry
        .register_provider(Box::new(MdBookRuleProvider))
        .unwrap();

    registry.create_engine().unwrap()
}

/// Create a lint engine with only standard markdown rules
pub fn create_standard_engine() -> LintEngine {
    let mut registry = PluginRegistry::new();
    registry
        .register_provider(Box::new(StandardRuleProvider))
        .unwrap();
    registry.create_engine().unwrap()
}

/// Create a lint engine with only mdBook-specific rules
pub fn create_mdbook_engine() -> LintEngine {
    let mut registry = PluginRegistry::new();
    registry
        .register_provider(Box::new(MdBookRuleProvider))
        .unwrap();
    registry.create_engine().unwrap()
}

/// Common imports
pub mod prelude {
    pub use crate::{
        Document, create_engine_with_all_rules, create_mdbook_engine, create_standard_engine,
        engine::{LintEngine, PluginRegistry, RuleProvider},
        error::{ErrorContext, IntoMdBookLintError, MdBookLintError, MdlntError, Result},
        registry::RuleRegistry,
        rule::{AstRule, Rule, RuleCategory, RuleMetadata, RuleStability},
        violation::{Severity, Violation},
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        assert_eq!(NAME, "mdbook-lint-core");
        assert!(DESCRIPTION.contains("linting engine"));
    }

    #[test]
    fn test_create_all_rules_engine() {
        let engine = create_engine_with_all_rules();
        let rules = engine.available_rules();
        assert!(
            rules.len() >= 60,
            "Expected at least 60 rules, got {}",
            rules.len()
        );
    }

    #[test]
    fn test_create_standard_engine() {
        let engine = create_standard_engine();
        let rules = engine.available_rules();
        assert!(
            rules.len() >= 50,
            "Expected at least 50 standard rules, got {}",
            rules.len()
        );
    }

    #[test]
    fn test_create_mdbook_engine() {
        let engine = create_mdbook_engine();
        let rules = engine.available_rules();
        assert!(
            rules.len() >= 4,
            "Expected at least 4 mdbook rules, got {}",
            rules.len()
        );
    }

    #[test]
    fn test_basic_engine_creation() {
        let engine = LintEngine::new();
        assert_eq!(engine.available_rules().len(), 0);
    }

    #[test]
    fn test_plugin_registry_creation() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.providers().len(), 0);
    }

    #[test]
    fn test_rule_registry_creation() {
        let registry = RuleRegistry::new();
        assert!(registry.is_empty());
    }
}
