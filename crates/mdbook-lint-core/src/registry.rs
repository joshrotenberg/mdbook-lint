use crate::{
    Document, config::Config, error::Result, rule::CollectionRule, rule::Rule, rule::RuleStability,
    violation::Violation,
};

/// Registry for managing linting rules
pub struct RuleRegistry {
    rules: Vec<Box<dyn Rule>>,
    collection_rules: Vec<Box<dyn CollectionRule>>,
}

impl RuleRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            collection_rules: Vec::new(),
        }
    }

    /// Register a rule with the registry
    ///
    /// Rules are stored in registration order and will be executed
    /// in the same order during document checking.
    pub fn register(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    /// Register a collection rule with the registry
    ///
    /// Collection rules analyze multiple documents together rather than
    /// processing documents one at a time. They are useful for cross-document
    /// validation such as checking for duplicate identifiers or validating
    /// inter-document links.
    pub fn register_collection_rule(&mut self, rule: Box<dyn CollectionRule>) {
        self.collection_rules.push(rule);
    }

    /// Get all registered collection rules
    pub fn collection_rules(&self) -> &[Box<dyn CollectionRule>] {
        &self.collection_rules
    }

    /// Get collection rule IDs
    pub fn collection_rule_ids(&self) -> Vec<&'static str> {
        self.collection_rules.iter().map(|r| r.id()).collect()
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

    /// Get rules that should be enabled based on configuration and rule overrides for a specific document
    ///
    /// This method applies configuration filters and handles rule overrides:
    /// - Basic configuration filtering (enabled/disabled rules, deprecation, categories)
    /// - Rule override resolution (context-specific rules can override general rules)
    pub fn get_enabled_rules_with_overrides(
        &self,
        document: &Document,
        config: &Config,
    ) -> Vec<&dyn Rule> {
        let mut enabled_rules: Vec<&dyn Rule> = self
            .rules
            .iter()
            .filter(|rule| self.should_run_rule(rule.as_ref(), config))
            .map(|rule| rule.as_ref())
            .collect();

        // Handle rule overrides - remove overridden rules when override conditions are met
        let mut rules_to_remove = Vec::new();

        for rule in &enabled_rules {
            let metadata = rule.metadata();
            if let Some(overrides_rule_id) = metadata.overrides {
                // Check if this override rule is applicable for this document
                if self.is_override_applicable(rule.id(), document) {
                    // This override rule is applicable, so mark the overridden rule for removal
                    rules_to_remove.push(overrides_rule_id);
                }
            }
        }

        // Remove overridden rules
        enabled_rules.retain(|rule| !rules_to_remove.contains(&rule.id()));

        enabled_rules
    }

    /// Check if a rule override is applicable for a specific document
    /// This is used for rules like MDBOOK025 that should override based on file name/context
    /// rather than just violation presence
    fn is_override_applicable(&self, rule_id: &str, document: &Document) -> bool {
        match rule_id {
            "MDBOOK025" => {
                // MDBOOK025 overrides MD025 for SUMMARY.md files
                document
                    .path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name == "SUMMARY.md")
                    .unwrap_or(false)
            }
            _ => false,
        }
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

        // Check markdownlint compatibility mode - disable rules that are disabled by default in markdownlint
        if config.markdownlint_compatible && rule_id == "MD044" {
            return false; // proper-names: disabled by default in markdownlint
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

        // An experimental rule that was not explicitly selected runs only when
        // opted into. This is checked after the filters above so that
        // disabled-rules and category filtering still win.
        if metadata.stability == RuleStability::Experimental {
            return config
                .experimental_rules
                .iter()
                .any(|selector| selector == "*" || selector == rule_id);
        }

        // Otherwise activation is decided by stability alone.
        metadata.runs_by_default()
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
        let enabled_rules = self.get_enabled_rules_with_overrides(document, config);

        // Run enabled rules with the pre-parsed AST
        for rule in enabled_rules {
            let violations = rule.check_with_ast(document, Some(ast))?;
            all_violations.extend(violations);
        }

        // Apply deduplication to eliminate duplicate violations
        let dedup_config = crate::deduplication::DeduplicationConfig::default();
        let deduplicated_violations =
            crate::deduplication::deduplicate_violations(all_violations, &dedup_config);

        Ok(deduplicated_violations)
    }

    /// Check a document with enabled rules
    pub fn check_document_with_config(
        &self,
        document: &Document,
        config: &Config,
    ) -> Result<Vec<Violation>> {
        let mut all_violations = Vec::new();
        let enabled_rules = self.get_enabled_rules_with_overrides(document, config);

        for rule in enabled_rules {
            let violations = rule.check(document)?;
            all_violations.extend(violations);
        }

        // Apply deduplication to eliminate duplicate violations
        let dedup_config = crate::deduplication::DeduplicationConfig::default();
        let deduplicated_violations =
            crate::deduplication::deduplicate_violations(all_violations, &dedup_config);

        Ok(deduplicated_violations)
    }

    /// Check a document with all rules using a single AST parse
    pub fn check_document_optimized(&self, document: &Document) -> Result<Vec<Violation>> {
        // Use default config when no config is provided
        let default_config = Config::default();
        self.check_document_optimized_with_config(document, &default_config)
    }

    /// Check a document with all rules
    pub fn check_document(&self, document: &Document) -> Result<Vec<Violation>> {
        let mut all_violations = Vec::new();

        for rule in &self.rules {
            let violations = rule.check(document)?;
            all_violations.extend(violations);
        }

        // Apply deduplication to eliminate duplicate violations
        let dedup_config = crate::deduplication::DeduplicationConfig::default();
        let deduplicated_violations =
            crate::deduplication::deduplicate_violations(all_violations, &dedup_config);

        Ok(deduplicated_violations)
    }

    /// Check a collection of documents with all collection rules
    ///
    /// This method runs all registered collection rules against the provided documents.
    /// Collection rules can see all documents at once, allowing for cross-document validation.
    pub fn check_collection(&self, documents: &[Document]) -> Result<Vec<Violation>> {
        let mut all_violations = Vec::new();

        for rule in &self.collection_rules {
            let violations = rule.check_collection(documents)?;
            all_violations.extend(violations);
        }

        Ok(all_violations)
    }

    /// Check a collection of documents with collection rules, respecting configuration
    pub fn check_collection_with_config(
        &self,
        documents: &[Document],
        config: &Config,
    ) -> Result<Vec<Violation>> {
        let mut all_violations = Vec::new();

        for rule in &self.collection_rules {
            let rule_id = rule.id();

            // Check if rule is disabled
            if config.disabled_rules.contains(&rule_id.to_string()) {
                continue;
            }

            // If enabled_rules is specified, only run rules in that list
            if !config.enabled_rules.is_empty()
                && !config.enabled_rules.contains(&rule_id.to_string())
            {
                continue;
            }

            let violations = rule.check_collection(documents)?;
            all_violations.extend(violations);
        }

        Ok(all_violations)
    }

    /// Get the number of registered rules
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Get the number of registered collection rules
    pub fn collection_rules_len(&self) -> usize {
        self.collection_rules.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Check if there are any collection rules registered
    pub fn has_collection_rules(&self) -> bool {
        !self.collection_rules.is_empty()
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

    /// Test rule with a configurable stability level.
    struct StabilityRule {
        id: &'static str,
        metadata: RuleMetadata,
    }

    impl StabilityRule {
        fn new(id: &'static str, metadata: RuleMetadata) -> Self {
            Self { id, metadata }
        }
    }

    impl Rule for StabilityRule {
        fn id(&self) -> &'static str {
            self.id
        }

        fn name(&self) -> &'static str {
            "stability-test-rule"
        }

        fn description(&self) -> &'static str {
            "A rule used to test stability-based activation"
        }

        fn metadata(&self) -> RuleMetadata {
            self.metadata.clone()
        }

        fn check_with_ast<'a>(
            &self,
            _document: &Document,
            _ast: Option<&'a comrak::nodes::AstNode<'a>>,
        ) -> Result<Vec<Violation>> {
            Ok(vec![])
        }
    }

    /// Build a registry holding one rule of each stability level.
    fn stability_registry() -> RuleRegistry {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(StabilityRule::new(
            "STABLE001",
            RuleMetadata::stable(RuleCategory::Structure),
        )));
        registry.register(Box::new(StabilityRule::new(
            "EXPER001",
            RuleMetadata::experimental(RuleCategory::Structure),
        )));
        registry.register(Box::new(StabilityRule::new(
            "EXPER002",
            RuleMetadata::experimental(RuleCategory::Structure),
        )));
        registry.register(Box::new(StabilityRule::new(
            "DEPREC001",
            RuleMetadata::deprecated(RuleCategory::Structure, "obsolete", None),
        )));
        registry.register(Box::new(StabilityRule::new(
            "RESERV001",
            RuleMetadata::reserved("never implemented"),
        )));
        registry
    }

    /// IDs of the rules the registry would run under `config`.
    fn active_ids(registry: &RuleRegistry, config: &Config) -> Vec<&'static str> {
        let mut ids: Vec<&'static str> = registry
            .get_enabled_rules(config)
            .iter()
            .map(|r| r.id())
            .collect();
        ids.sort();
        ids
    }

    #[test]
    fn test_only_stable_rules_run_by_default() {
        // Issue #468: experimental rules previously ran by default because the
        // fallback excluded only deprecated rules.
        let registry = stability_registry();
        assert_eq!(active_ids(&registry, &Config::default()), vec!["STABLE001"]);
    }

    #[test]
    fn test_explicit_selection_enables_experimental_rule() {
        let registry = stability_registry();
        let config = Config {
            enabled_rules: vec!["EXPER001".to_string()],
            ..Default::default()
        };
        // enabled_rules means "only these", so the stable rule is excluded too.
        assert_eq!(active_ids(&registry, &config), vec!["EXPER001"]);
    }

    #[test]
    fn test_experimental_rules_option_adds_to_defaults() {
        let registry = stability_registry();

        // A single ID opts in just that rule, alongside the stable defaults.
        let config = Config {
            experimental_rules: vec!["EXPER001".to_string()],
            ..Default::default()
        };
        assert_eq!(
            active_ids(&registry, &config),
            vec!["EXPER001", "STABLE001"]
        );

        // "*" opts in every experimental rule.
        let config = Config {
            experimental_rules: vec!["*".to_string()],
            ..Default::default()
        };
        assert_eq!(
            active_ids(&registry, &config),
            vec!["EXPER001", "EXPER002", "STABLE001"]
        );
    }

    #[test]
    fn test_disabled_rules_beat_experimental_opt_in() {
        let registry = stability_registry();
        let config = Config {
            experimental_rules: vec!["*".to_string()],
            disabled_rules: vec!["EXPER001".to_string()],
            ..Default::default()
        };
        assert_eq!(
            active_ids(&registry, &config),
            vec!["EXPER002", "STABLE001"]
        );
    }

    #[test]
    fn test_disabled_category_beats_experimental_opt_in() {
        let registry = stability_registry();
        let config = Config {
            experimental_rules: vec!["*".to_string()],
            disabled_categories: vec!["structure".to_string()],
            ..Default::default()
        };
        assert!(active_ids(&registry, &config).is_empty());
    }

    #[test]
    fn test_enabled_category_does_not_opt_into_experimental() {
        // Selecting a category expresses interest in a topic, not in unstable
        // diagnostics. Stability is orthogonal to category.
        let registry = stability_registry();
        let config = Config {
            enabled_categories: vec!["structure".to_string()],
            ..Default::default()
        };
        assert_eq!(active_ids(&registry, &config), vec!["STABLE001"]);
    }

    #[test]
    fn test_deprecated_and_reserved_stay_off_by_default() {
        let registry = stability_registry();
        let active = active_ids(&registry, &Config::default());
        assert!(!active.contains(&"DEPREC001"));
        assert!(!active.contains(&"RESERV001"));

        // Deprecated rules remain explicitly enableable, as before.
        let config = Config {
            enabled_rules: vec!["DEPREC001".to_string()],
            deprecated_warning: crate::config::DeprecatedWarningLevel::Silent,
            ..Default::default()
        };
        assert_eq!(active_ids(&registry, &config), vec!["DEPREC001"]);
    }

    #[test]
    fn test_runs_by_default_matches_stability() {
        assert!(RuleMetadata::stable(RuleCategory::Structure).runs_by_default());
        assert!(!RuleMetadata::experimental(RuleCategory::Structure).runs_by_default());
        assert!(!RuleMetadata::deprecated(RuleCategory::Structure, "x", None).runs_by_default());
        assert!(!RuleMetadata::reserved("x").runs_by_default());
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
