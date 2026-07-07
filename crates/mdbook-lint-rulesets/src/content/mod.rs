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
use mdbook_lint_core::Config;

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

    fn register_rules_with_config(&self, registry: &mut RuleRegistry, config: Option<&Config>) {
        let cfg = |id: &str| config.and_then(|c| c.rule_configs.get(id));

        let content001 = match cfg("CONTENT001") {
            Some(c) => content001::CONTENT001::from_config(c),
            None => content001::CONTENT001::default(),
        };
        registry.register(Box::new(content001));

        let content002 = match cfg("CONTENT002") {
            Some(c) => content002::CONTENT002::from_config(c),
            None => content002::CONTENT002::default(),
        };
        registry.register(Box::new(content002));

        let content003 = match cfg("CONTENT003") {
            Some(c) => content003::CONTENT003::from_config(c),
            None => content003::CONTENT003::default(),
        };
        registry.register(Box::new(content003));

        let content004 = match cfg("CONTENT004") {
            Some(c) => content004::CONTENT004::from_config(c),
            None => content004::CONTENT004::default(),
        };
        registry.register(Box::new(content004));

        let content005 = match cfg("CONTENT005") {
            Some(c) => content005::CONTENT005::from_config(c),
            None => content005::CONTENT005::default(),
        };
        registry.register(Box::new(content005));

        registry.register(Box::new(content006::CONTENT006));

        let content007 = match cfg("CONTENT007") {
            Some(c) => content007::CONTENT007::from_config(c),
            None => content007::CONTENT007::default(),
        };
        registry.register(Box::new(content007));

        let content009 = match cfg("CONTENT009") {
            Some(c) => content009::CONTENT009::from_config(c),
            None => content009::CONTENT009::default(),
        };
        registry.register(Box::new(content009));

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

#[cfg(test)]
mod provider_tests {
    use super::ContentRuleProvider;
    use mdbook_lint_core::{Config, Document, PluginRegistry};
    use std::path::PathBuf;

    /// End-to-end: config set on the CONTENT provider actually reaches the rule.
    /// This is the regression guard for #415 (the provider previously ignored
    /// all configuration).
    #[test]
    fn test_content009_max_depth_threads_through_provider() {
        // h1..h5; CONTENT009 default max depth is 4, so the h5 is flagged.
        let content = "# H1\n\n## H2\n\n### H3\n\n#### H4\n\n##### H5\n\nsome text\n";
        let doc = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();

        let build = |toml: &str| {
            let config: Config = toml::from_str(toml).unwrap();
            let mut registry = PluginRegistry::new();
            registry
                .register_provider(Box::new(ContentRuleProvider))
                .unwrap();
            let engine = registry.create_engine_with_config(Some(&config)).unwrap();
            engine
                .lint_document_with_config(&doc, &config)
                .unwrap()
                .into_iter()
                .filter(|v| v.rule_id == "CONTENT009")
                .count()
        };

        // Without config: the h5 is too deep.
        assert_eq!(
            build("enabled-rules = [\"CONTENT009\"]"),
            1,
            "CONTENT009 should flag the h5 at the default depth"
        );

        // With max_depth raised: no violation.
        assert_eq!(
            build("enabled-rules = [\"CONTENT009\"]\n[CONTENT009]\nmax_depth = 6"),
            0,
            "max_depth config must thread through the provider and suppress the violation"
        );
    }
}
