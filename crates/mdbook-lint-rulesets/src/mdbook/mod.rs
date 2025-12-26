//! mdBook-specific linting rules (MDBOOK001-025)
//!
//! This module contains implementations of mdBook-specific linting rules
//! that extend standard markdown linting for mdBook projects.

pub mod mdbook001;
pub mod mdbook002;
pub mod mdbook003;
pub mod mdbook004;
pub mod mdbook005;
pub mod mdbook006;
pub mod mdbook007;
pub mod mdbook008;
pub mod mdbook009;
pub mod mdbook010;
pub mod mdbook011;
pub mod mdbook012;
pub mod mdbook016;
pub mod mdbook017;
pub mod mdbook021;
pub mod mdbook022;
pub mod mdbook023;
pub mod mdbook025;

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
        registry.register(Box::new(mdbook016::MDBOOK016));
        registry.register(Box::new(mdbook017::MDBOOK017));
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
            "MDBOOK016",
            "MDBOOK017",
            "MDBOOK021",
            "MDBOOK022",
            "MDBOOK023",
            "MDBOOK025",
        ]
    }
}
