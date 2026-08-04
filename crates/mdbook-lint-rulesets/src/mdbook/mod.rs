//! mdBook-specific linting rules (MDBOOK001-025)
//!
//! This module contains implementations of mdBook-specific linting rules
//! that extend standard markdown linting for mdBook projects.

mod anchors;
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
mod mdbook016;
mod mdbook017;
mod mdbook021;
mod mdbook022;
mod mdbook023;
mod mdbook025;

use crate::{RuleProvider, RuleRegistry};
use mdbook_lint_core::Config;

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
        registry.register(Box::new(mdbook002::MDBOOK002::default()));
        registry.register(Box::new(mdbook003::MDBOOK003::default()));
        registry.register(Box::new(mdbook004::MDBOOK004::default()));
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

    fn register_rules_with_config(&self, registry: &mut RuleRegistry, config: Option<&Config>) {
        registry.register(Box::new(mdbook001::MDBOOK001));

        // MDBOOK002 - internal links (supports check_anchors/allow_external/check_images)
        let mdbook002 = match config.and_then(|c| c.rule_configs.get("MDBOOK002")) {
            Some(cfg) => mdbook002::MDBOOK002::from_config(cfg),
            None => mdbook002::MDBOOK002::default(),
        };
        registry.register(Box::new(mdbook002));

        // MDBOOK003 - SUMMARY.md structure
        // (supports allow_draft_chapters/require_part_headers/max_depth)
        let mdbook003 = match config.and_then(|c| c.rule_configs.get("MDBOOK003")) {
            Some(cfg) => mdbook003::MDBOOK003::from_config(cfg),
            None => mdbook003::MDBOOK003::default(),
        };
        registry.register(Box::new(mdbook003));

        // MDBOOK004 - duplicate chapter titles (supports case_sensitive/ignore_prefixes)
        let mdbook004 = match config.and_then(|c| c.rule_configs.get("MDBOOK004")) {
            Some(cfg) => mdbook004::MDBOOK004::from_config(cfg),
            None => mdbook004::MDBOOK004::default(),
        };
        registry.register(Box::new(mdbook004));

        // MDBOOK005 - orphaned files (supports ignore_patterns/exclude_readme/check_nested)
        let mdbook005 = match config.and_then(|c| c.rule_configs.get("MDBOOK005")) {
            Some(cfg) => mdbook005::MDBOOK005::from_config(cfg),
            None => mdbook005::MDBOOK005::default(),
        };
        registry.register(Box::new(mdbook005));

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

        // MDBOOK022 - title directive near top (supports max_line)
        let mdbook022 = match config.and_then(|c| c.rule_configs.get("MDBOOK022")) {
            Some(cfg) => mdbook022::MDBOOK022::from_config(cfg),
            None => mdbook022::MDBOOK022::default(),
        };
        registry.register(Box::new(mdbook022));

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

#[cfg(test)]
mod provider_tests {
    use super::MdBookRuleProvider;
    use mdbook_lint_core::{Config, Document, PluginRegistry};
    use std::path::PathBuf;

    /// Lint `doc` through the mdBook provider with `toml` as the config, counting
    /// the violations reported by `rule_id`.
    fn violations_for(doc: &Document, toml: &str, rule_id: &str) -> usize {
        let config: Config = toml::from_str(toml).unwrap();
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(MdBookRuleProvider))
            .unwrap();
        let engine = registry.create_engine_with_config(Some(&config)).unwrap();
        engine
            .lint_document_with_config(doc, &config)
            .unwrap()
            .into_iter()
            .filter(|v| v.rule_id == rule_id)
            .count()
    }

    #[test]
    fn test_mdbook002_config_threads_through_provider() {
        let content = "# Doc\n\n![Missing](./missing.png)\n";
        let doc = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();

        assert_eq!(
            violations_for(&doc, "enabled-rules = [\"MDBOOK002\"]", "MDBOOK002"),
            0,
            "image paths should not be checked by default"
        );
        assert_eq!(
            violations_for(
                &doc,
                "enabled-rules = [\"MDBOOK002\"]\n[MDBOOK002]\ncheck_images = true",
                "MDBOOK002"
            ),
            1,
            "check_images must thread through the provider"
        );
    }

    #[test]
    fn test_mdbook003_config_threads_through_provider() {
        let content = "# Summary\n\n- [One](one.md)\n    - [Two](two.md)\n";
        let doc = Document::new(content.to_string(), PathBuf::from("SUMMARY.md")).unwrap();

        assert_eq!(
            violations_for(&doc, "enabled-rules = [\"MDBOOK003\"]", "MDBOOK003"),
            0,
            "nesting depth should be unlimited by default"
        );
        assert_eq!(
            violations_for(
                &doc,
                "enabled-rules = [\"MDBOOK003\"]\n[MDBOOK003]\nmax_depth = 1",
                "MDBOOK003"
            ),
            1,
            "max_depth must thread through the provider"
        );
    }

    #[test]
    fn test_mdbook004_config_threads_through_provider() {
        let content = "# Introduction\n\n# introduction\n";
        let doc = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();

        assert_eq!(
            violations_for(&doc, "enabled-rules = [\"MDBOOK004\"]", "MDBOOK004"),
            0,
            "title comparison should be case-sensitive by default"
        );
        assert_eq!(
            violations_for(
                &doc,
                "enabled-rules = [\"MDBOOK004\"]\n[MDBOOK004]\ncase_sensitive = false",
                "MDBOOK004"
            ),
            1,
            "case_sensitive must thread through the provider"
        );
    }
}
