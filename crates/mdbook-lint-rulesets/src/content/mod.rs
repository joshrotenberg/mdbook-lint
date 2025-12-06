//! Content quality linting rules (CONTENT001+)
//!
//! This module contains rules for detecting content quality issues
//! such as TODO comments, placeholder text, and incomplete sections.

mod content001;
mod content002;
mod content003;
mod content004;
mod content005;
mod content006;
mod content007;
mod content009;
mod content010;
mod content011;

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
        registry.register(Box::new(content003::CONTENT003::default()));
        registry.register(Box::new(content004::CONTENT004::default()));
        registry.register(Box::new(content005::CONTENT005::default()));
        registry.register(Box::new(content006::CONTENT006));
        registry.register(Box::new(content007::CONTENT007::default()));
        registry.register(Box::new(content009::CONTENT009::default()));
        registry.register(Box::new(content010::CONTENT010));
        registry.register(Box::new(content011::CONTENT011));
    }

    fn rule_ids(&self) -> Vec<&'static str> {
        vec![
            "CONTENT001",
            "CONTENT002",
            "CONTENT003",
            "CONTENT004",
            "CONTENT005",
            "CONTENT006",
            "CONTENT007",
            "CONTENT009",
            "CONTENT010",
            "CONTENT011",
        ]
    }
}
