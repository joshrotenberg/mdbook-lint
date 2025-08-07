//! mdBook-specific linting rules (MDBOOK001-007)
//!
//! This module contains implementations of mdBook-specific linting rules
//! that extend standard markdown linting for mdBook projects.

use crate::{RuleProvider, RuleRegistry};

/// Provider for mdBook-specific rules (MDBOOK001-007)
pub struct MdBookRuleProvider;

impl RuleProvider for MdBookRuleProvider {
    fn provider_id(&self) -> &'static str {
        "mdbook"
    }

    fn description(&self) -> &'static str {
        "mdBook-specific linting rules (MDBOOK001-007)"
    }

    fn version(&self) -> &'static str {
        "0.4.1"
    }

    fn register_rules(&self, _registry: &mut RuleRegistry) {
        // TODO: Register actual rules once moved from core
    }

    fn rule_ids(&self) -> Vec<&'static str> {
        // TODO: Return actual rule IDs once moved from core
        vec![]
    }
}
