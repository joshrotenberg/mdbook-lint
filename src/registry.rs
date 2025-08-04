use crate::{Document, config::Config, error::Result, rule::Rule, violation::Violation};

/// Registry for managing linting rules
pub struct RuleRegistry {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Register a rule with the registry
    ///
    /// Rules are stored in registration order and will be executed
    /// in the same order during document checking.
    pub fn register(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    /// Get all registered rules
    pub fn rules(&self) -> &[Box<dyn Rule>] {
        &self.rules
    }

    /// Get a rule by ID
    ///
    /// Returns the first rule with the matching ID, or None if no such rule exists.
    pub fn get_rule(&self, id: &str) -> Option<&dyn Rule> {
        self.rules.iter().find(|r| r.id() == id).map(|r| r.as_ref())
    }

    /// Get all rule IDs
    ///
    /// Returns a vector of all registered rule IDs in registration order.
    pub fn rule_ids(&self) -> Vec<&'static str> {
        self.rules.iter().map(|r| r.id()).collect()
    }

    /// Get rules that should be enabled based on configuration
    ///
    /// This method applies configuration filters to determine which rules
    /// should actually run, considering:
    /// - Explicitly enabled/disabled rules
    /// - Rule deprecation status
    /// - Category-based filtering
    pub fn get_enabled_rules(&self, config: &Config) -> Vec<&dyn Rule> {
        self.rules
            .iter()
            .filter(|rule| self.should_run_rule(rule.as_ref(), config))
            .map(|rule| rule.as_ref())
            .collect()
    }

    /// Check if a rule should run based on configuration and metadata
    ///
    /// This implements the rule filtering logic that considers:
    /// 1. Explicitly disabled rules (always excluded)
    /// 2. Explicitly enabled rules (always included, with deprecation warnings)
    /// 3. Category-based filtering (enabled/disabled categories)
    /// 4. Default behavior (exclude deprecated rules unless explicitly enabled)
    pub fn should_run_rule(&self, rule: &dyn Rule, config: &Config) -> bool {
        let rule_id = rule.id();
        let metadata = rule.metadata();

        // Check explicit disabled rules first
        if config.disabled_rules.contains(&rule_id.to_string()) {
            return false;
        }

        // Check explicit enabled rules
        if config.enabled_rules.contains(&rule_id.to_string()) {
            // Show deprecation warning if needed
            if metadata.deprecated {
                self.show_deprecation_warning(rule, config);
            }
            return true;
        }

        // If enabled_rules is specified, only run rules in that list
        if !config.enabled_rules.is_empty() {
            return false;
        }

        // Check category-based filtering
        let category_name = self.category_to_string(&metadata.category);

        // If disabled categories specified, exclude rules in those categories
        if config.disabled_categories.contains(&category_name) {
            return false;
        }

        // If enabled categories specified, only include rules in those categories
        if !config.enabled_categories.is_empty()
            && !config.enabled_categories.contains(&category_name)
        {
            return false;
        }

        // For rules not explicitly configured, only enable non-deprecated rules by default
        !metadata.deprecated
    }

    /// Convert RuleCategory to string for configuration matching
    fn category_to_string(&self, category: &crate::rule::RuleCategory) -> String {
        match category {
            crate::rule::RuleCategory::Structure => "structure".to_string(),
            crate::rule::RuleCategory::Formatting => "style".to_string(),
            crate::rule::RuleCategory::Content => "code".to_string(),
            crate::rule::RuleCategory::Links => "links".to_string(),
            crate::rule::RuleCategory::Accessibility => "accessibility".to_string(),
            crate::rule::RuleCategory::MdBook => "mdbook".to_string(),
        }
    }

    /// Show deprecation warning based on configuration
    ///
    /// Displays deprecation warnings according to the configured warning level.
    fn show_deprecation_warning(&self, rule: &dyn Rule, config: &Config) {
        let metadata = rule.metadata();

        if !metadata.deprecated {
            return;
        }

        let message = if let Some(replacement) = metadata.replacement {
            format!(
                "Rule {} is deprecated - {}. Consider using {} instead.",
                rule.id(),
                metadata
                    .deprecated_reason
                    .unwrap_or("superseded by newer implementation"),
                replacement
            )
        } else {
            format!(
                "Rule {} is deprecated - {}.",
                rule.id(),
                metadata
                    .deprecated_reason
                    .unwrap_or("no longer recommended")
            )
        };

        match config.deprecated_warning {
            crate::config::DeprecatedWarningLevel::Warn => {
                eprintln!("Warning: {message}");
            }
            crate::config::DeprecatedWarningLevel::Info => {
                eprintln!("Info: {message}");
            }
            crate::config::DeprecatedWarningLevel::Silent => {
                // No output
            }
        }
    }

    /// Check a document with enabled rules using a single AST parse
    pub fn check_document_optimized_with_config(
        &self,
        document: &Document,
        config: &Config,
    ) -> Result<Vec<Violation>> {
        use comrak::Arena;

        // Parse AST once
        let arena = Arena::new();
        let ast = document.parse_ast(&arena);

        let mut all_violations = Vec::new();
        let enabled_rules = self.get_enabled_rules(config);

        // Run enabled rules with the pre-parsed AST
        for rule in enabled_rules {
            let violations = rule.check_with_ast(document, Some(ast))?;
            all_violations.extend(violations);
        }

        Ok(all_violations)
    }

    /// Check a document with enabled rules
    pub fn check_document_with_config(
        &self,
        document: &Document,
        config: &Config,
    ) -> Result<Vec<Violation>> {
        let mut all_violations = Vec::new();
        let enabled_rules = self.get_enabled_rules(config);

        for rule in enabled_rules {
            let violations = rule.check(document)?;
            all_violations.extend(violations);
        }

        Ok(all_violations)
    }

    /// Check a document with all rules using a single AST parse
    pub fn check_document_optimized(&self, document: &Document) -> Result<Vec<Violation>> {
        use comrak::Arena;

        // Parse AST once
        let arena = Arena::new();
        let ast = document.parse_ast(&arena);

        let mut all_violations = Vec::new();

        // Run all rules with the pre-parsed AST
        for rule in &self.rules {
            let violations = rule.check_with_ast(document, Some(ast))?;
            all_violations.extend(violations);
        }

        Ok(all_violations)
    }

    /// Check a document with all rules
    pub fn check_document(&self, document: &Document) -> Result<Vec<Violation>> {
        let mut all_violations = Vec::new();

        for rule in &self.rules {
            let violations = rule.check(document)?;
            all_violations.extend(violations);
        }

        Ok(all_violations)
    }

    /// Get the number of registered rules
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

impl Default for RuleRegistry {
    /// Create a new empty registry
    ///
    /// Note: Unlike the original implementation, this does NOT register
    /// any default rules. This is intentional for the core library to
    /// remain rule-agnostic.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::{Rule, RuleCategory, RuleMetadata};
    use std::path::PathBuf;

    // Test rule for registry testing
    struct TestRule {
        id: &'static str,
        name: &'static str,
    }

    impl TestRule {
        fn new(id: &'static str, name: &'static str) -> Self {
            Self { id, name }
        }
    }

    impl Rule for TestRule {
        fn id(&self) -> &'static str {
            self.id
        }

        fn name(&self) -> &'static str {
            self.name
        }

        fn description(&self) -> &'static str {
            "A test rule for testing"
        }

        fn metadata(&self) -> RuleMetadata {
            RuleMetadata::stable(RuleCategory::Structure)
        }

        fn check_with_ast<'a>(
            &self,
            _document: &Document,
            _ast: Option<&'a comrak::nodes::AstNode<'a>>,
        ) -> Result<Vec<Violation>> {
            Ok(vec![self.create_violation(
                format!("Test violation from {}", self.id),
                1,
                1,
                crate::violation::Severity::Warning,
            )])
        }
    }

    #[test]
    fn test_empty_registry() {
        let registry = RuleRegistry::new();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
        assert_eq!(registry.rule_ids(), Vec::<&str>::new());
    }

    #[test]
    fn test_rule_registration() {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(TestRule::new("TEST001", "test-rule-1")));
        registry.register(Box::new(TestRule::new("TEST002", "test-rule-2")));

        assert_eq!(registry.len(), 2);
        assert!(!registry.is_empty());
        assert_eq!(registry.rule_ids(), vec!["TEST001", "TEST002"]);
    }

    #[test]
    fn test_get_rule() {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(TestRule::new("TEST001", "test-rule")));

        let rule = registry.get_rule("TEST001").unwrap();
        assert_eq!(rule.id(), "TEST001");
        assert_eq!(rule.name(), "test-rule");

        assert!(registry.get_rule("NONEXISTENT").is_none());
    }

    #[test]
    fn test_rule_filtering_with_config() {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(TestRule::new("TEST001", "test-rule-1")));
        registry.register(Box::new(TestRule::new("TEST002", "test-rule-2")));

        // Default config should enable all non-deprecated rules
        let config = Config::default();
        let enabled = registry.get_enabled_rules(&config);
        assert_eq!(enabled.len(), 2);

        // Config with enabled rules should only run those rules
        let config = Config {
            enabled_rules: vec!["TEST001".to_string()],
            ..Default::default()
        };
        let enabled = registry.get_enabled_rules(&config);
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].id(), "TEST001");

        // Config with disabled rules should exclude them
        let config = Config {
            disabled_rules: vec!["TEST002".to_string()],
            ..Default::default()
        };
        let enabled = registry.get_enabled_rules(&config);
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].id(), "TEST001");
    }

    #[test]
    fn test_document_checking() {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(TestRule::new("TEST001", "test-rule")));

        let document = Document::new("# Test".to_string(), PathBuf::from("test.md")).unwrap();

        // Test optimized checking
        let violations = registry.check_document_optimized(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "TEST001");

        // Test traditional checking
        let violations = registry.check_document(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "TEST001");

        // Test config-based checking
        let config = Config::default();
        let violations = registry
            .check_document_optimized_with_config(&document, &config)
            .unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "TEST001");
    }

    #[test]
    fn test_default_registry_is_empty() {
        let registry = RuleRegistry::default();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }
}
