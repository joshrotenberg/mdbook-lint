//! Standard markdown rules (MD001-MD059).

use crate::{engine::RuleProvider, registry::RuleRegistry};

/// Provider for standard markdown rules (MD001-MD059)
pub struct StandardRuleProvider;

impl RuleProvider for StandardRuleProvider {
    fn provider_id(&self) -> &'static str {
        "mdbook-lint-standard"
    }

    fn description(&self) -> &'static str {
        "Standard markdownlint-compatible rules (MD001-MD059)"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn register_rules(&self, registry: &mut RuleRegistry) {
        // Structure and heading rules
        registry.register(Box::new(crate::rules::standard::md001::MD001));
        registry.register(Box::new(crate::rules::standard::md002::MD002::new()));
        registry.register(Box::new(crate::rules::standard::md003::MD003::new()));

        // MD004-MD007: List rules
        registry.register(Box::new(crate::rules::standard::md004::MD004::new()));
        registry.register(Box::new(crate::rules::standard::md005::MD005));
        registry.register(Box::new(crate::rules::standard::md006::MD006));
        registry.register(Box::new(crate::rules::standard::md007::MD007::new()));
        registry.register(Box::new(crate::rules::standard::md008::MD008));

        // MD009-MD014: Whitespace and formatting rules
        registry.register(Box::new(crate::rules::standard::md009::MD009::new()));
        registry.register(Box::new(crate::rules::standard::md010::MD010::new()));
        registry.register(Box::new(crate::rules::standard::md011::MD011));
        registry.register(Box::new(crate::rules::standard::md012::MD012::new()));
        registry.register(Box::new(crate::rules::standard::md013::MD013::new()));
        registry.register(Box::new(crate::rules::standard::md014::MD014));
        registry.register(Box::new(crate::rules::standard::md015::MD015));
        registry.register(Box::new(crate::rules::standard::md016::MD016));
        registry.register(Box::new(crate::rules::standard::md017::MD017));

        // MD018-MD028: Link and image rules
        registry.register(Box::new(crate::rules::standard::md018::MD018));
        registry.register(Box::new(crate::rules::standard::md019::MD019));
        registry.register(Box::new(crate::rules::standard::md020::MD020));
        registry.register(Box::new(crate::rules::standard::md021::MD021));
        registry.register(Box::new(crate::rules::standard::md022::MD022));
        registry.register(Box::new(crate::rules::standard::md023::MD023));
        registry.register(Box::new(crate::rules::standard::md024::MD024::new()));
        registry.register(Box::new(crate::rules::standard::md025::MD025::new()));
        registry.register(Box::new(crate::rules::standard::md026::MD026::new()));
        registry.register(Box::new(crate::rules::standard::md027::MD027));
        registry.register(Box::new(crate::rules::standard::md028::MD028));

        // MD029-MD032: Advanced formatting rules
        registry.register(Box::new(crate::rules::standard::md029::MD029::new()));
        registry.register(Box::new(crate::rules::standard::md030::MD030::new()));
        registry.register(Box::new(crate::rules::standard::md031::MD031));
        registry.register(Box::new(crate::rules::standard::md032::MD032));

        // MD033-MD040: HTML and code rules
        registry.register(Box::new(crate::rules::standard::md033::MD033));
        registry.register(Box::new(crate::rules::standard::md034::MD034));
        registry.register(Box::new(crate::rules::standard::md035::MD035::new()));
        registry.register(Box::new(crate::rules::standard::md036::MD036::new()));
        registry.register(Box::new(crate::rules::standard::md037::MD037));
        registry.register(Box::new(crate::rules::standard::md038::MD038));
        registry.register(Box::new(crate::rules::standard::md039::MD039));
        registry.register(Box::new(crate::rules::standard::md040::MD040));

        // MD041-MD048: Document structure rules
        registry.register(Box::new(crate::rules::standard::md041::MD041));
        registry.register(Box::new(crate::rules::standard::md042::MD042));
        registry.register(Box::new(crate::rules::standard::md043::MD043::new()));
        registry.register(Box::new(crate::rules::standard::md044::MD044::new()));
        registry.register(Box::new(crate::rules::standard::md045::MD045));
        registry.register(Box::new(crate::rules::standard::md046::MD046::new()));
        registry.register(Box::new(crate::rules::standard::md047::MD047));
        registry.register(Box::new(crate::rules::standard::md048::MD048::new()));

        // MD049-MD059: Advanced linting rules
        registry.register(Box::new(crate::rules::standard::md049::MD049::new()));
        registry.register(Box::new(crate::rules::standard::md050::MD050::new()));
        registry.register(Box::new(crate::rules::standard::md051::MD051::new()));
        registry.register(Box::new(crate::rules::standard::md052::MD052::new()));
        registry.register(Box::new(crate::rules::standard::md053::MD053::new()));
        registry.register(Box::new(crate::rules::standard::md054::MD054::new()));
        registry.register(Box::new(crate::rules::standard::md055::MD055::new()));
        registry.register(Box::new(crate::rules::standard::md056::MD056::new()));
        registry.register(Box::new(crate::rules::standard::md057::MD057));
        registry.register(Box::new(crate::rules::standard::md058::MD058));
        registry.register(Box::new(crate::rules::standard::md059::MD059::new()));
    }

    fn rule_ids(&self) -> Vec<&'static str> {
        vec![
            "MD001", "MD002", "MD003", "MD004", "MD005", "MD006", "MD007", "MD008", "MD009",
            "MD010", "MD011", "MD012", "MD013", "MD014", "MD015", "MD016", "MD017", "MD018",
            "MD019", "MD020", "MD021", "MD022", "MD023", "MD024", "MD025", "MD026", "MD027",
            "MD028", "MD029", "MD030", "MD031", "MD032", "MD033", "MD034", "MD035", "MD036",
            "MD037", "MD038", "MD039", "MD040", "MD041", "MD042", "MD043", "MD044", "MD045",
            "MD046", "MD047", "MD048", "MD049", "MD050", "MD051", "MD052", "MD053", "MD054",
            "MD055", "MD056", "MD057", "MD058", "MD059",
        ]
    }
}

/// Create a lint engine with standard rules only
///
/// This is a convenience function for creating an engine with just the
/// standard markdownlint-compatible rules.
pub fn create_standard_engine() -> crate::LintEngine {
    let mut registry = crate::PluginRegistry::new();
    registry
        .register_provider(Box::new(StandardRuleProvider))
        .unwrap();
    registry.create_engine().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_provider_metadata() {
        let provider = StandardRuleProvider;
        assert_eq!(provider.provider_id(), "mdbook-lint-standard");
        assert!(provider.description().contains("markdownlint"));
        assert!(!provider.version().is_empty());
    }

    #[test]
    fn test_standard_provider_rule_count() {
        let provider = StandardRuleProvider;
        let rule_ids = provider.rule_ids();

        // Should have 59 standard rules (MD001-MD059 with all gaps filled)
        assert_eq!(rule_ids.len(), 59);

        // Check some key rules are present
        assert!(rule_ids.contains(&"MD001"));
        assert!(rule_ids.contains(&"MD013"));
        assert!(rule_ids.contains(&"MD040"));
        assert!(rule_ids.contains(&"MD059"));

        // Should not contain mdBook rules
        assert!(!rule_ids.contains(&"MDBOOK001"));
    }

    #[test]
    fn test_standard_provider_registration() {
        let mut registry = crate::RuleRegistry::new();
        let provider = StandardRuleProvider;

        // Registry should be empty initially
        assert_eq!(registry.len(), 0);

        // Register the provider's rules
        provider.register_rules(&mut registry);

        // Should now have all standard rules
        assert_eq!(registry.len(), 59);

        // Check specific rules are registered
        assert!(registry.get_rule("MD001").is_some());
        assert!(registry.get_rule("MD013").is_some());
        assert!(registry.get_rule("MD040").is_some());
        assert!(registry.get_rule("MDBOOK001").is_none());
    }

    #[test]
    fn test_create_standard_engine() {
        let engine = create_standard_engine();
        let rule_ids = engine.available_rules();

        // Should have all standard rules
        assert_eq!(rule_ids.len(), 59);
        assert!(rule_ids.contains(&"MD001"));
        assert!(rule_ids.contains(&"MD059"));

        // Should not have mdBook rules
        assert!(!rule_ids.contains(&"MDBOOK001"));
    }
}
