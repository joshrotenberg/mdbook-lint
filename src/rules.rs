//! Consolidated rules module for mdbook-lint
//!
//! This module organizes all linting rules available in mdbook-lint:
//! - **Standard Rules (MD001-MD059)**: 59 markdownlint-compatible rules
//! - **mdBook Rules (MDBOOK001-004)**: 4 mdBook-specific rules
//!
//! Total: 63 comprehensive rules for mdBook documentation projects.

// Standard markdownlint rules (MD001-MD059)
pub mod standard;

// mdBook-specific rules (MDBOOK001-005)
pub mod mdbook001;
pub mod mdbook002;
pub mod mdbook003;
pub mod mdbook004;
pub mod mdbook005;

use crate::{engine::RuleProvider, registry::RuleRegistry};

/// Provider for mdBook-specific linting rules
///
/// This provider includes mdBook-specific rules (MDBOOK001-005) that check
/// for mdBook conventions and best practices:
/// - Code block language tags for proper syntax highlighting
/// - Internal link validation within the book structure
/// - SUMMARY.md format validation
/// - Unique chapter title enforcement
/// - Orphaned file detection
///
/// # Rule Coverage
///
/// - **MDBOOK001**: code-block-language - Code blocks should have language tags
/// - **MDBOOK002**: internal-link-validation - Internal links must resolve
/// - **MDBOOK003**: summary-structure - SUMMARY.md format validation
/// - **MDBOOK004**: no-duplicate-chapter-titles - Unique chapter titles
/// - **MDBOOK005**: orphaned-files - Detect files not referenced in SUMMARY.md
pub struct MdBookRuleProvider;

impl RuleProvider for MdBookRuleProvider {
    fn provider_id(&self) -> &'static str {
        "mdbook"
    }

    fn description(&self) -> &'static str {
        "mdBook-specific linting rules for preprocessor functionality"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn register_rules(&self, registry: &mut RuleRegistry) {
        // Register mdBook-specific rules
        registry.register(Box::new(mdbook001::MDBOOK001));
        registry.register(Box::new(mdbook002::MDBOOK002));
        registry.register(Box::new(mdbook003::MDBOOK003));
        registry.register(Box::new(mdbook004::MDBOOK004));
        registry.register(Box::new(mdbook005::MDBOOK005::default()));
    }

    fn rule_ids(&self) -> Vec<&'static str> {
        vec![
            "MDBOOK001",
            "MDBOOK002",
            "MDBOOK003",
            "MDBOOK004",
            "MDBOOK005",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mdbook_provider_metadata() {
        let provider = MdBookRuleProvider;
        assert_eq!(provider.provider_id(), "mdbook");
        assert!(provider.description().contains("mdBook"));
        assert!(!provider.version().is_empty());
    }

    #[test]
    fn test_mdbook_provider_rule_count() {
        let provider = MdBookRuleProvider;
        let rule_ids = provider.rule_ids();

        // Should have 5 mdBook rules
        assert_eq!(rule_ids.len(), 5);

        // Check all mdBook rules are present
        assert!(rule_ids.contains(&"MDBOOK001"));
        assert!(rule_ids.contains(&"MDBOOK002"));
        assert!(rule_ids.contains(&"MDBOOK003"));
        assert!(rule_ids.contains(&"MDBOOK004"));
        assert!(rule_ids.contains(&"MDBOOK005"));

        // Should not contain standard rules
        assert!(!rule_ids.contains(&"MD001"));
    }

    #[test]
    fn test_mdbook_provider_registration() {
        let mut registry = RuleRegistry::new();
        let provider = MdBookRuleProvider;

        // Registry should be empty initially
        assert_eq!(registry.len(), 0);

        // Register the provider's rules
        provider.register_rules(&mut registry);

        // Should now have all mdBook rules
        assert_eq!(registry.len(), 5);

        // Check specific rules are registered
        assert!(registry.get_rule("MDBOOK001").is_some());
        assert!(registry.get_rule("MDBOOK002").is_some());
        assert!(registry.get_rule("MDBOOK003").is_some());
        assert!(registry.get_rule("MDBOOK004").is_some());
        assert!(registry.get_rule("MDBOOK005").is_some());
        assert!(registry.get_rule("MD001").is_none());
    }
}
