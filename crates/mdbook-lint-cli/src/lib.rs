//! mdbook-lint CLI and preprocessor
//!
//! This crate provides the command-line interface and mdBook preprocessor
//! functionality for mdbook-lint. It builds on the mdbook-lint-core library
//! to provide CLI tools and mdBook integration.
//!
//! # Basic Usage
//!
//! ```rust
//! use mdbook_lint::{Document, PluginRegistry};
//! use mdbook_lint_rulesets::{StandardRuleProvider, MdBookRuleProvider};
//! use std::path::PathBuf;
//!
//! let mut registry = PluginRegistry::new();
//! registry.register_provider(Box::new(StandardRuleProvider)).unwrap();
//! registry.register_provider(Box::new(MdBookRuleProvider)).unwrap();
//! let engine = registry.create_engine().unwrap();
//! let document = Document::new("# Hello".to_string(), PathBuf::from("test.md"))?;
//! let violations = engine.lint_document(&document)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod config;
pub mod preprocessor;

#[cfg(test)]
mod rule_config_test;

#[cfg(test)]
mod batch1_rule_config_test;

// Re-export everything from core
pub use mdbook_lint_core::*;

// Re-export CLI-specific types
pub use config::Config;
pub use preprocessor::MdBookLint;

/// Current version of mdbook-lint CLI
pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Human-readable name for CLI
pub const CLI_NAME: &str = "mdbook-lint";

/// Description for CLI
pub const CLI_DESCRIPTION: &str = "A fast markdown linter for mdBook projects";

/// Common imports for CLI usage
pub mod cli_prelude {
    pub use crate::{
        Config,
        MdBookLint,
        // Re-export core prelude
        prelude::*,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_version_info() {
        assert_eq!(CLI_NAME, "mdbook-lint");
        assert!(CLI_DESCRIPTION.contains("mdBook"));
    }

    #[test]
    fn test_core_functionality_available() {
        // Verify we can use core functionality through re-exports
        use mdbook_lint_rulesets::{MdBookRuleProvider, StandardRuleProvider};

        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();
        registry
            .register_provider(Box::new(MdBookRuleProvider))
            .unwrap();
        let engine = registry.create_engine().unwrap();
        let rules = engine.available_rules();
        assert!(
            rules.len() >= 60,
            "Expected at least 60 rules, got {}",
            rules.len()
        );
    }
}
