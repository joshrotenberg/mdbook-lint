//! Rule provider system and lint engine.

use crate::error::Result;
use crate::registry::RuleRegistry;
use serde_json::Value;

/// Trait for rule providers to register rules with the engine
pub trait RuleProvider: Send + Sync {
    /// Unique identifier for this rule provider
    fn provider_id(&self) -> &'static str;

    /// Human-readable description of this rule provider
    fn description(&self) -> &'static str;

    /// Version of this rule provider
    fn version(&self) -> &'static str;

    /// Register all rules from this provider with the registry
    fn register_rules(&self, registry: &mut RuleRegistry);

    /// Provider-specific configuration schema
    fn config_schema(&self) -> Option<Value> {
        None
    }

    /// List of rule IDs that this provider registers
    fn rule_ids(&self) -> Vec<&'static str> {
        Vec::new()
    }

    /// Provider initialization hook
    fn initialize(&self) -> Result<()> {
        Ok(())
    }
}

/// Registry for managing rule providers and creating engines
#[derive(Default)]
pub struct PluginRegistry {
    providers: Vec<Box<dyn RuleProvider>>,
}

impl PluginRegistry {
    /// Create a new empty plugin registry
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Register a rule provider
    pub fn register_provider(&mut self, provider: Box<dyn RuleProvider>) -> Result<()> {
        // Initialize the provider
        provider.initialize()?;

        // Check for duplicate provider IDs
        let provider_id = provider.provider_id();
        if self
            .providers
            .iter()
            .any(|p| p.provider_id() == provider_id)
        {
            return Err(crate::error::MdBookLintError::plugin_error(format!(
                "Provider with ID '{provider_id}' is already registered"
            )));
        }

        self.providers.push(provider);
        Ok(())
    }

    /// Get all registered providers
    pub fn providers(&self) -> &[Box<dyn RuleProvider>] {
        &self.providers
    }

    /// Get a provider by ID
    pub fn get_provider(&self, id: &str) -> Option<&dyn RuleProvider> {
        self.providers
            .iter()
            .find(|p| p.provider_id() == id)
            .map(|p| p.as_ref())
    }

    /// Create a rule registry with all registered providers
    pub fn create_rule_registry(&self) -> Result<RuleRegistry> {
        let mut registry = RuleRegistry::new();

        for provider in &self.providers {
            provider.register_rules(&mut registry);
        }

        Ok(registry)
    }

    /// Create a lint engine with all registered providers
    pub fn create_engine(&self) -> Result<LintEngine> {
        let registry = self.create_rule_registry()?;
        Ok(LintEngine::with_registry(registry))
    }

    /// List all available rule IDs from all providers
    pub fn available_rule_ids(&self) -> Vec<String> {
        let mut rule_ids = Vec::new();

        for provider in &self.providers {
            for rule_id in provider.rule_ids() {
                rule_ids.push(rule_id.to_string());
            }
        }

        rule_ids.sort();
        rule_ids.dedup();
        rule_ids
    }

    /// Get provider information for debugging/introspection
    pub fn provider_info(&self) -> Vec<ProviderInfo> {
        self.providers
            .iter()
            .map(|p| ProviderInfo {
                id: p.provider_id().to_string(),
                description: p.description().to_string(),
                version: p.version().to_string(),
                rule_count: p.rule_ids().len(),
            })
            .collect()
    }
}

/// Information about a registered provider (for debugging/introspection)
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub id: String,
    pub description: String,
    pub version: String,
    pub rule_count: usize,
}

/// Markdown linting engine
pub struct LintEngine {
    registry: RuleRegistry,
}

impl LintEngine {
    /// Create a new lint engine with no rules
    pub fn new() -> Self {
        Self {
            registry: RuleRegistry::new(),
        }
    }

    /// Create a lint engine with an existing rule registry
    pub fn with_registry(registry: RuleRegistry) -> Self {
        Self { registry }
    }

    /// Get the underlying rule registry
    pub fn registry(&self) -> &RuleRegistry {
        &self.registry
    }

    /// Get a mutable reference to the rule registry
    pub fn registry_mut(&mut self) -> &mut RuleRegistry {
        &mut self.registry
    }

    /// Lint a document with all registered rules
    pub fn lint_document(&self, document: &crate::Document) -> Result<Vec<crate::Violation>> {
        self.registry.check_document_optimized(document)
    }

    /// Lint a document with specific configuration
    pub fn lint_document_with_config(
        &self,
        document: &crate::Document,
        config: &crate::Config,
    ) -> Result<Vec<crate::Violation>> {
        self.registry
            .check_document_optimized_with_config(document, config)
    }

    /// Lint content string directly (convenience method)
    pub fn lint_content(&self, content: &str, path: &str) -> Result<Vec<crate::Violation>> {
        let document = crate::Document::new(content.to_string(), std::path::PathBuf::from(path))?;
        self.lint_document(&document)
    }

    /// Get all available rule IDs
    pub fn available_rules(&self) -> Vec<&'static str> {
        self.registry.rule_ids()
    }

    /// Get enabled rules based on configuration
    pub fn enabled_rules(&self, config: &crate::Config) -> Vec<&dyn crate::rule::Rule> {
        self.registry.get_enabled_rules(config)
    }
}

impl Default for LintEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::{Rule, RuleCategory, RuleMetadata};
    use std::path::PathBuf;

    // Test rule for plugin system testing
    struct TestRule;

    impl Rule for TestRule {
        fn id(&self) -> &'static str {
            "TEST001"
        }
        fn name(&self) -> &'static str {
            "test-rule"
        }
        fn description(&self) -> &'static str {
            "A test rule"
        }
        fn metadata(&self) -> RuleMetadata {
            RuleMetadata::stable(RuleCategory::Structure)
        }
        fn check_with_ast<'a>(
            &self,
            _document: &crate::Document,
            _ast: Option<&'a comrak::nodes::AstNode<'a>>,
        ) -> Result<Vec<crate::Violation>> {
            Ok(vec![])
        }
    }

    // Test provider
    struct TestProvider;

    impl RuleProvider for TestProvider {
        fn provider_id(&self) -> &'static str {
            "test-provider"
        }
        fn description(&self) -> &'static str {
            "Test provider"
        }
        fn version(&self) -> &'static str {
            "0.1.0"
        }

        fn register_rules(&self, registry: &mut RuleRegistry) {
            registry.register(Box::new(TestRule));
        }

        fn rule_ids(&self) -> Vec<&'static str> {
            vec!["TEST001"]
        }
    }

    #[test]
    fn test_plugin_registry_basic() {
        let mut registry = PluginRegistry::new();
        assert_eq!(registry.providers().len(), 0);

        registry.register_provider(Box::new(TestProvider)).unwrap();
        assert_eq!(registry.providers().len(), 1);

        let provider = registry.get_provider("test-provider").unwrap();
        assert_eq!(provider.provider_id(), "test-provider");
        assert_eq!(provider.description(), "Test provider");
    }

    #[test]
    fn test_plugin_registry_duplicate_id() {
        let mut registry = PluginRegistry::new();
        registry.register_provider(Box::new(TestProvider)).unwrap();

        // Should fail with duplicate ID
        let result = registry.register_provider(Box::new(TestProvider));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("already registered"));
    }

    #[test]
    fn test_create_engine_from_registry() {
        let mut registry = PluginRegistry::new();
        registry.register_provider(Box::new(TestProvider)).unwrap();

        let engine = registry.create_engine().unwrap();
        let rule_ids = engine.available_rules();
        assert!(rule_ids.contains(&"TEST001"));
    }

    #[test]
    fn test_available_rule_ids() {
        let mut registry = PluginRegistry::new();
        registry.register_provider(Box::new(TestProvider)).unwrap();

        let rule_ids = registry.available_rule_ids();
        assert_eq!(rule_ids, vec!["TEST001"]);
    }

    #[test]
    fn test_provider_info() {
        let mut registry = PluginRegistry::new();
        registry.register_provider(Box::new(TestProvider)).unwrap();

        let info = registry.provider_info();
        assert_eq!(info.len(), 1);
        assert_eq!(info[0].id, "test-provider");
        assert_eq!(info[0].description, "Test provider");
        assert_eq!(info[0].version, "0.1.0");
        assert_eq!(info[0].rule_count, 1);
    }

    #[test]
    fn test_get_provider_not_found() {
        let registry = PluginRegistry::new();
        assert!(registry.get_provider("nonexistent").is_none());
    }

    #[test]
    fn test_create_rule_registry() {
        let mut registry = PluginRegistry::new();
        registry.register_provider(Box::new(TestProvider)).unwrap();

        let rule_registry = registry.create_rule_registry().unwrap();
        assert!(!rule_registry.is_empty());
    }

    // Test provider with initialization failure
    struct FailingProvider;

    impl RuleProvider for FailingProvider {
        fn provider_id(&self) -> &'static str {
            "failing-provider"
        }
        fn description(&self) -> &'static str {
            "Failing test provider"
        }
        fn version(&self) -> &'static str {
            "0.1.0"
        }
        fn register_rules(&self, _registry: &mut RuleRegistry) {}
        fn initialize(&self) -> Result<()> {
            Err(crate::error::MdBookLintError::plugin_error(
                "Initialization failed",
            ))
        }
    }

    #[test]
    fn test_provider_initialization_failure() {
        let mut registry = PluginRegistry::new();
        let result = registry.register_provider(Box::new(FailingProvider));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Initialization failed"));
    }

    // Test provider with config schema
    struct ConfigurableProvider;

    impl RuleProvider for ConfigurableProvider {
        fn provider_id(&self) -> &'static str {
            "configurable-provider"
        }
        fn description(&self) -> &'static str {
            "Configurable test provider"
        }
        fn version(&self) -> &'static str {
            "0.1.0"
        }
        fn register_rules(&self, _registry: &mut RuleRegistry) {}
        fn config_schema(&self) -> Option<Value> {
            Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "enabled": {"type": "boolean"}
                }
            }))
        }
    }

    #[test]
    fn test_provider_with_config_schema() {
        let provider = ConfigurableProvider;
        let schema = provider.config_schema();
        assert!(schema.is_some());
        let schema = schema.unwrap();
        assert_eq!(schema["type"], "object");
    }

    #[test]
    fn test_lint_engine_with_registry() {
        let mut rule_registry = RuleRegistry::new();
        rule_registry.register(Box::new(TestRule));

        let engine = LintEngine::with_registry(rule_registry);
        let rules = engine.available_rules();
        assert!(rules.contains(&"TEST001"));
    }

    #[test]
    fn test_lint_engine_api() {
        let mut registry = PluginRegistry::new();
        registry.register_provider(Box::new(TestProvider)).unwrap();
        let engine = registry.create_engine().unwrap();

        // Test basic content linting
        let _violations = engine.lint_content("# Test\n", "test.md").unwrap();

        // Test document linting
        let document =
            crate::Document::new("# Test".to_string(), PathBuf::from("test.md")).unwrap();
        let _violations = engine.lint_document(&document).unwrap();
    }


}
