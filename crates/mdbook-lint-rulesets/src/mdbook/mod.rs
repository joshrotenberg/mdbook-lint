//! mdBook-specific linting rules (MDBOOK001-007)
//!
//! This module contains implementations of mdBook-specific linting rules
//! that extend standard markdown linting for mdBook projects.

mod mdbook001;
mod mdbook002;
mod mdbook003;
mod mdbook004;
mod mdbook005;
mod mdbook006;
mod mdbook007;
mod mdbook025;

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

    fn register_rules(&self, registry: &mut RuleRegistry) {
        registry.register(Box::new(mdbook001::MDBOOK001));
        registry.register(Box::new(mdbook002::MDBOOK002));
        registry.register(Box::new(mdbook003::MDBOOK003));
        registry.register(Box::new(mdbook004::MDBOOK004));
        registry.register(Box::new(mdbook005::MDBOOK005::default()));
        registry.register(Box::new(mdbook006::MDBOOK006::default()));
        registry.register(Box::new(mdbook007::MDBOOK007::default()));
        registry.register(Box::new(mdbook025::MDBOOK025));
    }

    fn rule_ids(&self) -> Vec<&'static str> {
        vec![
            "MDBOOK001",
            "MDBOOK002",
            "MDBOOK003",
            "MDBOOK004",
            "MDBOOK005",
            "MDBOOK006",
            "MDBOOK007",
            "MDBOOK025",
        ]
    }
}
