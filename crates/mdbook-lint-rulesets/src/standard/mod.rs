//! Standard markdown linting rules (MD001-MD059)
//!
//! This module contains implementations of the standard markdown linting rules
//! as defined by the markdownlint specification.

use crate::{RuleProvider, RuleRegistry};

/// Provider for standard markdown rules (MD001-MD059)
pub struct StandardRuleProvider;

impl RuleProvider for StandardRuleProvider {
    fn provider_id(&self) -> &'static str {
        "standard"
    }

    fn description(&self) -> &'static str {
        "Standard markdown linting rules (MD001-MD059)"
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
