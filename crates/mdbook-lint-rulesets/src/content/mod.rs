//! Content quality linting rules (CONTENT001+)
//!
//! This module contains rules for detecting content quality issues
//! such as TODO comments, placeholder text, and incomplete sections.

mod content001;
mod content002;

use crate::{RuleProvider, RuleRegistry};

/// Provider for content quality rules (CONTENT001+)
pub struct ContentRuleProvider;

impl RuleProvider for ContentRuleProvider {
    fn provider_id(&self) -> &'static str {
        "content"
    }

    fn description(&self) -> &'static str {
        "Content quality linting rules (CONTENT001+)"
    }

    fn version(&self) -> &'static str {
        "0.12.0"
    }

    fn register_rules(&self, registry: &mut RuleRegistry) {
        registry.register(Box::new(content001::CONTENT001::default()));
        registry.register(Box::new(content002::CONTENT002::default()));
    }

    fn rule_ids(&self) -> Vec<&'static str> {
        vec!["CONTENT001", "CONTENT002"]
    }
}
