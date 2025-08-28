//! Standard markdown linting rules (MD001-MD059)
//!
//! This module contains implementations of the standard markdown linting rules
//! as defined by the markdownlint specification.

// Standard markdownlint rules (MD001-MD059)
pub mod md001;
pub mod md002;
pub mod md003;
pub mod md004;
pub mod md005;
pub mod md006;
pub mod md007;
pub mod md008; // Placeholder: never implemented
pub mod md009;
pub mod md010;
pub mod md011;
pub mod md012;
pub mod md013;
pub mod md014;
pub mod md015; // Placeholder: merged into MD013
pub mod md016; // Placeholder: gap in numbering
pub mod md017; // Placeholder: covered by MD018-021
pub mod md018;
pub mod md019;
pub mod md020;
pub mod md021;
pub mod md022;
pub mod md023;
pub mod md024;
pub mod md025;
pub mod md026;
pub mod md027;
pub mod md028;
pub mod md029;
pub mod md030;
pub mod md031;
pub mod md032;
pub mod md033;
pub mod md034;
pub mod md035;
pub mod md036;
pub mod md037;
pub mod md038;
pub mod md039;
pub mod md040;
pub mod md041;
pub mod md042;
pub mod md043;
pub mod md044;
pub mod md045;
pub mod md046;
pub mod md047;
pub mod md048;
pub mod md049;
pub mod md050;
pub mod md051;
pub mod md052;
pub mod md053;
pub mod md054;
pub mod md055;
pub mod md056;
pub mod md057; // Placeholder: reserved for future use
pub mod md058;
pub mod md059;

use mdbook_lint_core::{Config, RuleProvider, RuleRegistry};

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

    fn register_rules(&self, registry: &mut RuleRegistry) {
        // Register all standard rules
        registry.register(Box::new(md001::MD001));
        registry.register(Box::new(md002::MD002::default()));
        registry.register(Box::new(md003::MD003::default()));
        registry.register(Box::new(md004::MD004::default()));
        registry.register(Box::new(md005::MD005));
        registry.register(Box::new(md006::MD006));
        registry.register(Box::new(md007::MD007::default()));
        // MD008 is a placeholder
        registry.register(Box::new(md009::MD009::default()));
        registry.register(Box::new(md010::MD010::default()));
        registry.register(Box::new(md011::MD011));
        registry.register(Box::new(md012::MD012::default()));
        registry.register(Box::new(md013::MD013::default()));
        registry.register(Box::new(md014::MD014));
        // MD015-017 are placeholders
        registry.register(Box::new(md018::MD018));
        registry.register(Box::new(md019::MD019));
        registry.register(Box::new(md020::MD020));
        registry.register(Box::new(md021::MD021));
        registry.register(Box::new(md022::MD022));
        registry.register(Box::new(md023::MD023));
        registry.register(Box::new(md024::MD024::default()));
        registry.register(Box::new(md025::MD025::default()));
        registry.register(Box::new(md026::MD026::default()));
        registry.register(Box::new(md027::MD027));
        registry.register(Box::new(md028::MD028));
        registry.register(Box::new(md029::MD029::default()));
        registry.register(Box::new(md030::MD030::default()));
        registry.register(Box::new(md031::MD031));
        registry.register(Box::new(md032::MD032));
        registry.register(Box::new(md033::MD033));
        registry.register(Box::new(md034::MD034));
        registry.register(Box::new(md035::MD035::default()));
        registry.register(Box::new(md036::MD036::default()));
        registry.register(Box::new(md037::MD037));
        registry.register(Box::new(md038::MD038));
        registry.register(Box::new(md039::MD039));
        registry.register(Box::new(md040::MD040));
        registry.register(Box::new(md041::MD041));
        registry.register(Box::new(md042::MD042));
        registry.register(Box::new(md043::MD043::default()));
        registry.register(Box::new(md044::MD044::default()));
        registry.register(Box::new(md045::MD045));
        registry.register(Box::new(md046::MD046::default()));
        registry.register(Box::new(md047::MD047));
        registry.register(Box::new(md048::MD048::default()));
        registry.register(Box::new(md049::MD049::default()));
        registry.register(Box::new(md050::MD050::default()));
        registry.register(Box::new(md051::MD051::default()));
        registry.register(Box::new(md052::MD052::default()));
        registry.register(Box::new(md053::MD053::default()));
        registry.register(Box::new(md054::MD054::default()));
        registry.register(Box::new(md055::MD055::default()));
        registry.register(Box::new(md056::MD056));
        // MD057 is a placeholder
        registry.register(Box::new(md058::MD058));
        registry.register(Box::new(md059::MD059::default()));
    }

    fn rule_ids(&self) -> Vec<&'static str> {
        vec![
            "MD001", "MD002", "MD003", "MD004", "MD005", "MD006", "MD007", "MD009", "MD010",
            "MD011", "MD012", "MD013", "MD014", "MD018", "MD019", "MD020", "MD021", "MD022",
            "MD023", "MD024", "MD025", "MD026", "MD027", "MD028", "MD029", "MD030", "MD031",
            "MD032", "MD033", "MD034", "MD035", "MD036", "MD037", "MD038", "MD039", "MD040",
            "MD041", "MD042", "MD043", "MD044", "MD045", "MD046", "MD047", "MD048", "MD049",
            "MD050", "MD051", "MD052", "MD053", "MD054", "MD055", "MD056", "MD058", "MD059",
        ]
    }

    fn register_rules_with_config(&self, registry: &mut RuleRegistry, config: Option<&Config>) {
        // Register all standard rules with configuration support
        registry.register(Box::new(md001::MD001));
        registry.register(Box::new(md002::MD002::default()));
        registry.register(Box::new(md003::MD003::default()));

        // MD004 - unordered list style
        let md004 = if let Some(cfg) = config.and_then(|c| c.rule_configs.get("MD004")) {
            md004::MD004::from_config(cfg)
        } else {
            md004::MD004::default()
        };
        registry.register(Box::new(md004));

        registry.register(Box::new(md005::MD005));
        registry.register(Box::new(md006::MD006));
        registry.register(Box::new(md007::MD007::default()));
        // MD008 is a placeholder

        // MD009 - no trailing spaces
        let md009 = if let Some(cfg) = config.and_then(|c| c.rule_configs.get("MD009")) {
            md009::MD009::from_config(cfg)
        } else {
            md009::MD009::default()
        };
        registry.register(Box::new(md009));

        registry.register(Box::new(md010::MD010::default()));
        registry.register(Box::new(md011::MD011));
        registry.register(Box::new(md012::MD012::default()));

        // MD013 - line length
        let md013 = if let Some(cfg) = config.and_then(|c| c.rule_configs.get("MD013")) {
            md013::MD013::from_config(cfg)
        } else {
            md013::MD013::default()
        };
        registry.register(Box::new(md013));

        registry.register(Box::new(md014::MD014));
        // MD015-017 are placeholders
        registry.register(Box::new(md018::MD018));
        registry.register(Box::new(md019::MD019));
        registry.register(Box::new(md020::MD020));
        registry.register(Box::new(md021::MD021));
        registry.register(Box::new(md022::MD022));
        registry.register(Box::new(md023::MD023));
        registry.register(Box::new(md024::MD024::default()));
        registry.register(Box::new(md025::MD025::default()));
        registry.register(Box::new(md026::MD026::default()));
        registry.register(Box::new(md027::MD027));
        registry.register(Box::new(md028::MD028));
        registry.register(Box::new(md029::MD029::default()));
        registry.register(Box::new(md030::MD030::default()));
        registry.register(Box::new(md031::MD031));
        registry.register(Box::new(md032::MD032));
        registry.register(Box::new(md033::MD033));
        registry.register(Box::new(md034::MD034));
        registry.register(Box::new(md035::MD035::default()));
        registry.register(Box::new(md036::MD036::default()));
        registry.register(Box::new(md037::MD037));
        registry.register(Box::new(md038::MD038));
        registry.register(Box::new(md039::MD039));
        registry.register(Box::new(md040::MD040));
        registry.register(Box::new(md041::MD041));
        registry.register(Box::new(md042::MD042));
        registry.register(Box::new(md043::MD043::default()));
        registry.register(Box::new(md044::MD044::default()));
        registry.register(Box::new(md045::MD045));
        registry.register(Box::new(md046::MD046::default()));
        registry.register(Box::new(md047::MD047));
        registry.register(Box::new(md048::MD048::default()));
        registry.register(Box::new(md049::MD049::default()));
        registry.register(Box::new(md050::MD050::default()));
        registry.register(Box::new(md051::MD051::default()));
        registry.register(Box::new(md052::MD052::default()));
        registry.register(Box::new(md053::MD053::default()));
        registry.register(Box::new(md054::MD054::default()));
        registry.register(Box::new(md055::MD055::default()));
        registry.register(Box::new(md056::MD056));
        // MD057 is a placeholder
        registry.register(Box::new(md058::MD058));
        registry.register(Box::new(md059::MD059::default()));
    }
}
