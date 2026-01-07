//! Core linting engine for mdbook-lint
//!
//! This crate provides the foundational infrastructure for markdown linting with mdBook support.
//! It defines the core abstractions and engine that powers mdbook-lint's rule-based linting system.
//!
//! # Overview
//!
//! The `mdbook-lint-core` crate provides:
//!
//! - **Plugin-based architecture** for extensible rule sets
//! - **AST and text-based linting** with efficient document processing
//! - **Violation reporting** with detailed position tracking and severity levels
//! - **Automatic fix infrastructure** for correctable violations
//! - **Configuration system** for customizing rule behavior
//! - **Document abstraction** with markdown parsing via comrak
//!
//! # Architecture
//!
//! The core follows a plugin-based architecture where rules are provided by external crates:
//!
//! ```text
//! ┌─────────────────┐
//! │   Application   │
//! └────────┬────────┘
//!          │
//! ┌────────▼────────┐
//! │  PluginRegistry │ ◄─── Registers rule providers
//! └────────┬────────┘
//!          │
//! ┌────────▼────────┐
//! │   LintEngine    │ ◄─── Orchestrates linting
//! └────────┬────────┘
//!          │
//! ┌────────▼────────┐
//! │     Rules       │ ◄─── Individual rule implementations
//! └─────────────────┘
//! ```
//!
//! # Basic Usage
//!
//! ## Creating a Lint Engine
//!
//! ```rust
//! use mdbook_lint_core::{PluginRegistry, Document};
//! use std::path::PathBuf;
//!
//! // Create an empty engine (no rules registered)
//! let registry = PluginRegistry::new();
//! let engine = registry.create_engine()?;
//!
//! // Lint a document
//! let document = Document::new("# Hello\n\nWorld".to_string(), PathBuf::from("test.md"))?;
//! let violations = engine.lint_document(&document)?;
//!
//! // No violations since no rules are registered
//! assert_eq!(violations.len(), 0);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## With Rule Providers
//!
//! ```rust,no_run
//! use mdbook_lint_core::{PluginRegistry, Document};
//! // Assumes mdbook-lint-rulesets is available
//! // use mdbook_lint_rulesets::{StandardRuleProvider, MdBookRuleProvider};
//! use std::path::PathBuf;
//!
//! let mut registry = PluginRegistry::new();
//!
//! // Register rule providers
//! // registry.register_provider(Box::new(StandardRuleProvider))?;
//! // registry.register_provider(Box::new(MdBookRuleProvider))?;
//!
//! // Create engine with registered rules
//! let engine = registry.create_engine()?;
//!
//! // Lint a document
//! let content = "# Title\n\n\n\nToo many blank lines";
//! let document = Document::new(content.to_string(), PathBuf::from("test.md"))?;
//! let violations = engine.lint_document(&document)?;
//!
//! // Process violations
//! for violation in violations {
//!     println!("{}:{} - {}", violation.rule_id, violation.line, violation.message);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Key Types
//!
//! ## Document
//!
//! Represents a markdown file with its content and metadata:
//!
//! ```rust
//! use mdbook_lint_core::Document;
//! use std::path::PathBuf;
//! use comrak::Arena;
//!
//! let doc = Document::new(
//!     "# My Document\n\nContent here".to_string(),
//!     PathBuf::from("doc.md")
//! )?;
//!
//! // Parse AST with comrak Arena
//! let arena = Arena::new();
//! let ast = doc.parse_ast(&arena);
//!
//! // Get document lines
//! let lines = &doc.lines;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Violation
//!
//! Represents a linting violation with location and optional fix:
//!
//! ```rust
//! use mdbook_lint_core::violation::{Violation, Severity, Fix, Position};
//!
//! let violation = Violation {
//!     rule_id: "MD001".to_string(),
//!     rule_name: "heading-increment".to_string(),
//!     message: "Heading levels should increment by one".to_string(),
//!     line: 5,
//!     column: 1,
//!     severity: Severity::Warning,
//!     fix: Some(Fix {
//!         description: "Change heading level".to_string(),
//!         replacement: Some("## Correct Level".to_string()),
//!         start: Position { line: 5, column: 1 },
//!         end: Position { line: 5, column: 20 },
//!     }),
//! };
//! ```
//!
//! ## Rule Traits
//!
//! Rules can be implemented using different traits based on their needs:
//!
//! - `Rule` - Base trait for all rules
//! - `AstRule` - For rules that analyze the markdown AST
//! - `TextRule` - For rules that analyze raw text
//! - `RuleWithConfig` - For rules that support configuration
//!
//! # Configuration
//!
//! Rules can be configured through TOML configuration files:
//!
//! ```toml
//! # .mdbook-lint.toml
//! [rules.MD013]
//! line_length = 120
//! code_blocks = false
//!
//! [rules.MD009]
//! br_spaces = 2
//!
//! # Disable specific rules
//! [rules]
//! MD002 = false
//! MD041 = false
//! ```
//!
//! # Features
//!
//! This crate has no optional features. All functionality is included by default.

pub mod config;
pub mod deduplication;
pub mod document;
pub mod engine;
pub mod error;
pub mod registry;
pub mod rule;
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

/// Current version of mdbook-lint-core
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Human-readable name
pub const NAME: &str = "mdbook-lint-core";

/// Description
pub const DESCRIPTION: &str = "Core linting engine for mdbook-lint";

/// Create a lint engine with all available rules (standard + mdBook)
/// Note: Requires mdbook-lint-rulesets dependency for rule providers
pub fn create_engine_with_all_rules() -> LintEngine {
    panic!(
        "create_engine_with_all_rules() is deprecated. Use mdbook-lint-rulesets crate providers directly."
    )
}

/// Create a lint engine with only standard markdown rules
/// Note: Requires mdbook-lint-rulesets dependency for rule providers
pub fn create_standard_engine() -> LintEngine {
    panic!(
        "create_standard_engine() is deprecated. Use mdbook-lint-rulesets crate providers directly."
    )
}

/// Create a lint engine with only mdBook-specific rules
/// Note: Requires mdbook-lint-rulesets dependency for rule providers
pub fn create_mdbook_engine() -> LintEngine {
    panic!(
        "create_mdbook_engine() is deprecated. Use mdbook-lint-rulesets crate providers directly."
    )
}

/// Common imports
pub mod prelude {
    pub use crate::{
        Document,
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
    #[should_panic]
    fn test_create_all_rules_engine_deprecated() {
        create_engine_with_all_rules();
    }

    #[test]
    #[should_panic]
    fn test_create_standard_engine_deprecated() {
        create_standard_engine();
    }

    #[test]
    #[should_panic]
    fn test_create_mdbook_engine_deprecated() {
        create_mdbook_engine();
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
