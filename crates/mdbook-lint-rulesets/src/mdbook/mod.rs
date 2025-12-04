//! mdBook-specific linting rules (MDBOOK001-025)
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
mod mdbook008;
mod mdbook009;
mod mdbook010;
mod mdbook011;
mod mdbook012;
mod mdbook021;
mod mdbook022;
mod mdbook023;
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
        registry.register(Box::new(mdbook008::MDBOOK008));
        registry.register(Box::new(mdbook009::MDBOOK009));
        registry.register(Box::new(mdbook010::MDBOOK010));
        registry.register(Box::new(mdbook011::MDBOOK011));
        registry.register(Box::new(mdbook012::MDBOOK012));
        registry.register(Box::new(mdbook021::MDBOOK021));
        registry.register(Box::new(mdbook022::MDBOOK022::default()));
        registry.register(Box::new(mdbook023::MDBOOK023::default()));
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
            "MDBOOK008",
            "MDBOOK009",
            "MDBOOK010",
            "MDBOOK011",
            "MDBOOK012",
            "MDBOOK021",
            "MDBOOK022",
            "MDBOOK023",
            "MDBOOK025",
        ]
    }
}
