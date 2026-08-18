//! Rule provider system and lint engine.

use crate::config::Config;
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

    /// Register all rules from this provider with the registry, using configuration
    /// This method allows rules to be configured at registration time.
    /// The default implementation calls the legacy register_rules method for backward compatibility.
    fn register_rules_with_config(&self, registry: &mut RuleRegistry, _config: Option<&Config>) {
        // Default implementation calls the old method for backward compatibility
        self.register_rules(registry);
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
        self.create_rule_registry_with_config(None)
    }

    /// Create a rule registry with all registered providers, using configuration
    pub fn create_rule_registry_with_config(
        &self,
        config: Option<&Config>,
    ) -> Result<RuleRegistry> {
        let mut registry = RuleRegistry::new();

        for provider in &self.providers {
            provider.register_rules_with_config(&mut registry, config);
        }

        Ok(registry)
    }

    /// Create a lint engine with all registered providers
    pub fn create_engine(&self) -> Result<LintEngine> {
        self.create_engine_with_config(None)
    }

    /// Create a lint engine with all registered providers, using configuration
    pub fn create_engine_with_config(&self, config: Option<&Config>) -> Result<LintEngine> {
        let registry = self.create_rule_registry_with_config(config)?;
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
    ///
    /// # Arguments
    ///
    /// * `content` - The markdown content to lint
    /// * `source_label` - A label for error messages (e.g., filename). This is NOT read from disk.
    pub fn lint_content(&self, content: &str, source_label: &str) -> Result<Vec<crate::Violation>> {
        let document =
            crate::Document::new(content.to_string(), std::path::PathBuf::from(source_label))?;
        self.lint_document(&document)
    }

    /// Apply a single fix to content
    ///
    /// Returns `Some(fixed_content)` if the fix was applied, `None` if the violation
    /// has no fix or the fix couldn't be applied.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let violations = engine.lint_content("# Test\n\n\n\n", "test.md")?;
    /// if let Some(v) = violations.first() {
    ///     if let Some(fixed) = engine.apply_fix("# Test\n\n\n\n", v) {
    ///         println!("Fixed: {}", fixed);
    ///     }
    /// }
    /// ```
    pub fn apply_fix(&self, content: &str, violation: &crate::Violation) -> Option<String> {
        let fix = violation.fix.as_ref()?;

        let range = fix.byte_range(content)?;
        let replacement = fix.replacement.as_deref().unwrap_or("");
        let mut result = content.to_string();
        result.replace_range(range, replacement);
        Some(result)
    }

    /// Apply all available fixes to content
    ///
    /// Applies fixes from violations that have them, processing in reverse position
    /// order to avoid offset issues. Returns the fixed content and a list of
    /// violations that could not be fixed.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let violations = engine.lint_content(content, "test.md")?;
    /// let (fixed_content, unfixed) = engine.apply_fixes(content, &violations);
    /// if fixed_content != content {
    ///     println!("Applied {} fixes", violations.len() - unfixed.len());
    /// }
    /// ```
    pub fn apply_fixes(
        &self,
        content: &str,
        violations: &[crate::Violation],
    ) -> (String, Vec<crate::Violation>) {
        if violations.is_empty() {
            return (content.to_string(), Vec::new());
        }

        struct PlannedFix<'a> {
            violation_index: usize,
            start: usize,
            end: usize,
            replacement: &'a str,
        }

        // Resolve every source position against the original content. Resolving
        // positions after earlier edits can shift or invalidate later ranges.
        let mut planned = Vec::new();
        for (violation_index, violation) in violations.iter().enumerate() {
            let Some(fix) = violation.fix.as_ref() else {
                continue;
            };
            let Some(range) = fix.byte_range(content) else {
                continue;
            };

            let replacement = fix.replacement.as_deref().unwrap_or("");
            planned.push(PlannedFix {
                violation_index,
                start: range.start,
                end: range.end,
                replacement,
            });
        }

        // Conflicting edits are unsafe to compose. Exact duplicates are safe and
        // are applied once while marking every associated violation as fixed.
        let mut conflicted = vec![false; planned.len()];
        for left in 0..planned.len() {
            for right in (left + 1)..planned.len() {
                let a = &planned[left];
                let b = &planned[right];
                let duplicate =
                    a.start == b.start && a.end == b.end && a.replacement == b.replacement;
                let overlap = (a.start < b.end && b.start < a.end) || a.start == b.start;

                if !duplicate && overlap {
                    conflicted[left] = true;
                    conflicted[right] = true;
                }
            }
        }

        let mut unique_edits: Vec<&PlannedFix<'_>> = Vec::new();
        let mut applied_indices = std::collections::HashSet::new();
        for (planned_index, edit) in planned.iter().enumerate() {
            if conflicted[planned_index] {
                continue;
            }

            applied_indices.insert(edit.violation_index);
            if !unique_edits.iter().any(|existing| {
                existing.start == edit.start
                    && existing.end == edit.end
                    && existing.replacement == edit.replacement
            }) {
                unique_edits.push(edit);
            }
        }

        // Descending byte offsets keep all original ranges valid while editing.
        unique_edits.sort_by(|a, b| b.start.cmp(&a.start).then_with(|| b.end.cmp(&a.end)));

        let mut result = content.to_string();
        for edit in unique_edits {
            result.replace_range(edit.start..edit.end, edit.replacement);
        }

        // Collect violations that weren't fixed
        let unfixed: Vec<crate::Violation> = violations
            .iter()
            .enumerate()
            .filter(|(idx, v)| v.fix.is_none() || !applied_indices.contains(idx))
            .map(|(_, v)| v.clone())
            .collect();

        (result, unfixed)
    }

    /// Get all available rule IDs
    pub fn available_rules(&self) -> Vec<&'static str> {
        self.registry.rule_ids()
    }

    /// Get enabled rules based on configuration
    pub fn enabled_rules(&self, config: &crate::Config) -> Vec<&dyn crate::rule::Rule> {
        self.registry.get_enabled_rules(config)
    }

    /// Lint a collection of documents with collection rules
    ///
    /// Collection rules analyze multiple documents together for cross-document validation.
    /// This method runs all registered collection rules against the provided documents.
    pub fn lint_collection(&self, documents: &[crate::Document]) -> Result<Vec<crate::Violation>> {
        self.registry.check_collection(documents)
    }

    /// Lint a collection of documents with specific configuration
    pub fn lint_collection_with_config(
        &self,
        documents: &[crate::Document],
        config: &crate::Config,
    ) -> Result<Vec<crate::Violation>> {
        self.registry
            .check_collection_with_config(documents, config)
    }

    /// Get all available collection rule IDs
    pub fn available_collection_rules(&self) -> Vec<&'static str> {
        self.registry.collection_rule_ids()
    }

    /// Check if there are any collection rules registered
    pub fn has_collection_rules(&self) -> bool {
        self.registry.has_collection_rules()
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("already registered")
        );
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Initialization failed")
        );
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

    #[test]
    fn test_position_to_offset() {
        let text = "line1\nline2\nline3";

        // Line 1, column 1 = offset 0
        assert_eq!(
            crate::violation::Position { line: 1, column: 1 }.to_byte_offset(text),
            Some(0)
        );

        // Line 1, column 3 = offset 2 ('n' in 'line1')
        assert_eq!(
            crate::violation::Position { line: 1, column: 3 }.to_byte_offset(text),
            Some(2)
        );

        // Line 2, column 1 = offset 6 (after 'line1\n')
        assert_eq!(
            crate::violation::Position { line: 2, column: 1 }.to_byte_offset(text),
            Some(6)
        );

        // Line 3, column 1 = offset 12
        assert_eq!(
            crate::violation::Position { line: 3, column: 1 }.to_byte_offset(text),
            Some(12)
        );

        // Invalid position
        assert_eq!(
            crate::violation::Position {
                line: 10,
                column: 1,
            }
            .to_byte_offset(text),
            None
        );
    }

    #[test]
    fn test_apply_fix_simple() {
        let engine = LintEngine::new();
        let content = "hello world";

        // Create a violation with a fix to replace "world" with "rust"
        let violation = crate::Violation {
            rule_id: "TEST".to_string(),
            rule_name: "test".to_string(),
            message: "test".to_string(),
            line: 1,
            column: 7,
            severity: crate::Severity::Warning,
            fix: Some(crate::violation::Fix {
                description: "Replace world with rust".to_string(),
                replacement: Some("rust".to_string()),
                start: crate::violation::Position { line: 1, column: 7 },
                end: crate::violation::Position {
                    line: 1,
                    column: 12,
                },
            }),
        };

        let result = engine.apply_fix(content, &violation);
        assert_eq!(result, Some("hello rust".to_string()));
    }

    #[test]
    fn test_apply_fix_no_fix() {
        let engine = LintEngine::new();
        let content = "hello world";

        let violation = crate::Violation {
            rule_id: "TEST".to_string(),
            rule_name: "test".to_string(),
            message: "test".to_string(),
            line: 1,
            column: 1,
            severity: crate::Severity::Warning,
            fix: None,
        };

        let result = engine.apply_fix(content, &violation);
        assert_eq!(result, None);
    }

    #[test]
    fn test_apply_fixes_multiple() {
        let engine = LintEngine::new();
        let content = "aaa bbb ccc";

        let violations = vec![
            crate::Violation {
                rule_id: "TEST".to_string(),
                rule_name: "test".to_string(),
                message: "test".to_string(),
                line: 1,
                column: 1,
                severity: crate::Severity::Warning,
                fix: Some(crate::violation::Fix {
                    description: "Replace aaa with AAA".to_string(),
                    replacement: Some("AAA".to_string()),
                    start: crate::violation::Position { line: 1, column: 1 },
                    end: crate::violation::Position { line: 1, column: 4 },
                }),
            },
            crate::Violation {
                rule_id: "TEST".to_string(),
                rule_name: "test".to_string(),
                message: "test".to_string(),
                line: 1,
                column: 9,
                severity: crate::Severity::Warning,
                fix: Some(crate::violation::Fix {
                    description: "Replace ccc with CCC".to_string(),
                    replacement: Some("CCC".to_string()),
                    start: crate::violation::Position { line: 1, column: 9 },
                    end: crate::violation::Position {
                        line: 1,
                        column: 12,
                    },
                }),
            },
        ];

        let (fixed, unfixed) = engine.apply_fixes(content, &violations);
        assert_eq!(fixed, "AAA bbb CCC");
        assert!(unfixed.is_empty());
    }

    #[test]
    fn test_apply_fixes_mixed() {
        let engine = LintEngine::new();
        let content = "hello world";

        let violations = vec![
            crate::Violation {
                rule_id: "TEST1".to_string(),
                rule_name: "test".to_string(),
                message: "has fix".to_string(),
                line: 1,
                column: 7,
                severity: crate::Severity::Warning,
                fix: Some(crate::violation::Fix {
                    description: "Replace".to_string(),
                    replacement: Some("rust".to_string()),
                    start: crate::violation::Position { line: 1, column: 7 },
                    end: crate::violation::Position {
                        line: 1,
                        column: 12,
                    },
                }),
            },
            crate::Violation {
                rule_id: "TEST2".to_string(),
                rule_name: "test".to_string(),
                message: "no fix".to_string(),
                line: 1,
                column: 1,
                severity: crate::Severity::Warning,
                fix: None,
            },
        ];

        let (fixed, unfixed) = engine.apply_fixes(content, &violations);
        assert_eq!(fixed, "hello rust");
        assert_eq!(unfixed.len(), 1);
        assert_eq!(unfixed[0].rule_id, "TEST2");
    }

    #[test]
    fn test_apply_fix_newline_handling() {
        // Test that fixes ending with newline don't cause double newlines
        let engine = LintEngine::new();

        // Content has a line that will be replaced, followed by another line
        let content = "# Old Heading\nNext line\n";

        // Fix replaces the heading line (including its newline) with a new heading
        let violation = crate::Violation {
            rule_id: "TEST".to_string(),
            rule_name: "test".to_string(),
            message: "Replace heading".to_string(),
            line: 1,
            column: 1,
            severity: crate::Severity::Warning,
            fix: Some(crate::violation::Fix {
                description: "Replace heading".to_string(),
                start: crate::violation::Position { line: 1, column: 1 },
                // Ending at the following line explicitly consumes the original
                // line terminator.
                end: crate::violation::Position { line: 2, column: 1 },
                replacement: Some("# New Heading\n".to_string()),
            }),
        };

        let result = engine.apply_fix(content, &violation);
        assert!(result.is_some());
        let fixed = result.unwrap();

        // Should have exactly one newline between the heading and next line, not two
        assert_eq!(fixed, "# New Heading\nNext line\n");
        assert!(!fixed.contains("\n\n"), "Should not have double newlines");
    }

    #[test]
    fn test_apply_fix_does_not_infer_newline_consumption() {
        let engine = LintEngine::new();
        let content = "old\nnext\n";
        let violation = crate::Violation {
            rule_id: "TEST".to_string(),
            rule_name: "test".to_string(),
            message: "Replace line content only".to_string(),
            line: 1,
            column: 1,
            severity: crate::Severity::Warning,
            fix: Some(crate::violation::Fix {
                description: "Replace line content only".to_string(),
                start: crate::violation::Position::line_start(1),
                end: crate::violation::Position::line_end(1, "old"),
                replacement: Some("new\n".to_string()),
            }),
        };

        // The replacement contains a newline, but the exact source range does
        // not consume one. The engine must not reinterpret that intent.
        assert_eq!(
            engine.apply_fix(content, &violation),
            Some("new\n\nnext\n".to_string())
        );
    }

    #[test]
    fn test_apply_fix_no_newline_no_adjustment() {
        // Test that fixes NOT ending with newline work normally
        let engine = LintEngine::new();
        let content = "hello world";

        let violation = crate::Violation {
            rule_id: "TEST".to_string(),
            rule_name: "test".to_string(),
            message: "Replace word".to_string(),
            line: 1,
            column: 7,
            severity: crate::Severity::Warning,
            fix: Some(crate::violation::Fix {
                description: "Replace word".to_string(),
                start: crate::violation::Position { line: 1, column: 7 },
                end: crate::violation::Position {
                    line: 1,
                    column: 12,
                },
                replacement: Some("rust".to_string()),
            }),
        };

        let result = engine.apply_fix(content, &violation);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "hello rust");
    }

    #[test]
    fn test_apply_fixes_skips_conflicting_ranges() {
        let engine = LintEngine::new();
        let content = "### C#\ntext\n";
        let violations = vec![
            crate::Violation {
                rule_id: "MD003".to_string(),
                rule_name: "heading-style".to_string(),
                message: "test".to_string(),
                line: 1,
                column: 1,
                severity: crate::Severity::Error,
                fix: Some(crate::violation::Fix {
                    description: "First whole-line replacement".to_string(),
                    replacement: Some("### C\n".to_string()),
                    start: crate::violation::Position { line: 1, column: 1 },
                    end: crate::violation::Position { line: 1, column: 7 },
                }),
            },
            crate::Violation {
                rule_id: "MD020".to_string(),
                rule_name: "no-missing-space-closed-atx".to_string(),
                message: "test".to_string(),
                line: 1,
                column: 1,
                severity: crate::Severity::Warning,
                fix: Some(crate::violation::Fix {
                    description: "Second whole-line replacement".to_string(),
                    replacement: Some("###C#\n".to_string()),
                    start: crate::violation::Position { line: 1, column: 1 },
                    end: crate::violation::Position { line: 1, column: 7 },
                }),
            },
        ];

        let (fixed, unfixed) = engine.apply_fixes(content, &violations);
        assert_eq!(fixed, content);
        assert_eq!(unfixed.len(), 2);
    }

    #[test]
    fn test_apply_fixes_deduplicates_identical_ranges() {
        let engine = LintEngine::new();
        let content = "#Bad#\nnext\n";
        let make_violation = |rule_id: &str| crate::Violation {
            rule_id: rule_id.to_string(),
            rule_name: "test".to_string(),
            message: "test".to_string(),
            line: 1,
            column: 1,
            severity: crate::Severity::Warning,
            fix: Some(crate::violation::Fix {
                description: "Normalize heading".to_string(),
                replacement: Some("# Bad #\n".to_string()),
                start: crate::violation::Position { line: 1, column: 1 },
                end: crate::violation::Position { line: 2, column: 1 },
            }),
        };
        let violations = vec![make_violation("TEST1"), make_violation("TEST2")];

        let (fixed, unfixed) = engine.apply_fixes(content, &violations);
        assert_eq!(fixed, "# Bad #\nnext\n");
        assert!(unfixed.is_empty());
    }
}
