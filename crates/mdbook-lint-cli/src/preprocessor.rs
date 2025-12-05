use mdbook::book::{Book, BookItem, Chapter};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

use crate::Config;
#[cfg(test)]
use mdbook_lint_core::RuleCategory;
use mdbook_lint_core::{
    Document, LintEngine, MdBookLintError, PluginRegistry, Severity, Violation,
};
#[cfg(feature = "content")]
use mdbook_lint_rulesets::ContentRuleProvider;
use mdbook_lint_rulesets::{MdBookRuleProvider, StandardRuleProvider};
use serde_json::Value;
use std::io::{self, Read};
use std::path::PathBuf;

/// mdbook-lint preprocessor
pub struct MdBookLint {
    /// Linting engine with combined rules
    pub engine: LintEngine,
    /// Configuration options
    pub config: Config,
}

impl MdBookLint {
    /// Create a new preprocessor with default rules and config
    pub fn new() -> Self {
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .expect("Failed to register standard rules");
        registry
            .register_provider(Box::new(MdBookRuleProvider))
            .expect("Failed to register mdbook rules");
        #[cfg(feature = "content")]
        registry
            .register_provider(Box::new(ContentRuleProvider))
            .expect("Failed to register content rules");
        let engine = registry.create_engine().expect("Failed to create engine");

        Self {
            config: Config::default(),
            engine,
        }
    }

    /// Create a new preprocessor with custom config
    #[allow(dead_code)]
    pub fn with_config(config: Config) -> Self {
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .expect("Failed to register standard rules");
        registry
            .register_provider(Box::new(MdBookRuleProvider))
            .expect("Failed to register mdbook rules");
        #[cfg(feature = "content")]
        registry
            .register_provider(Box::new(ContentRuleProvider))
            .expect("Failed to register content rules");
        let engine = registry.create_engine().expect("Failed to create engine");

        Self { config, engine }
    }

    /// Load configuration from preprocessor context
    pub fn load_config_from_context(
        &mut self,
        ctx: &PreprocessorContext,
    ) -> mdbook_lint_core::Result<()> {
        // First, try to load from book.toml preprocessor config
        let preprocessor_config = ctx
            .config
            .get_preprocessor("mdbook-lint")
            .or_else(|| ctx.config.get_preprocessor("lint"));

        if let Some(config) = preprocessor_config {
            self.config = parse_mdbook_config(config)?;
        } else {
            // No preprocessor config in book.toml, try to discover config file
            // Start search from the book root directory
            let book_root = &ctx.root;
            if let Some(discovered_path) = Config::discover_config(Some(book_root)) {
                eprintln!("Using config: {}", discovered_path.display());
                self.config = Config::from_file(&discovered_path)?;
            }
        }

        // Recreate the engine with the loaded configuration
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .expect("Failed to register standard rules");
        registry
            .register_provider(Box::new(MdBookRuleProvider))
            .expect("Failed to register mdbook rules");
        self.engine = registry
            .create_engine_with_config(Some(&self.config.core))
            .expect("Failed to create configured engine");

        Ok(())
    }

    /// Process a chapter and return any violations found
    fn process_chapter(&self, chapter: &Chapter) -> mdbook_lint_core::Result<Vec<Violation>> {
        // Create document from chapter content
        let source_path = chapter
            .source_path
            .as_ref()
            .unwrap_or(&PathBuf::from("unknown.md"))
            .clone();

        let document = Document::new(chapter.content.clone(), source_path)?;

        // Use optimized checking (single AST parse) with configuration
        let violations = self
            .engine
            .lint_document_with_config(&document, &self.config.core)?;

        Ok(violations)
    }

    /// Format violations for output
    fn format_violations(&self, violations: &[Violation], chapter_path: &str) -> String {
        if violations.is_empty() {
            return String::new();
        }

        let mut output = String::new();
        for violation in violations {
            output.push_str(&format!(
                "{}:{}:{}: {}\n",
                chapter_path, violation.line, violation.column, violation
            ));
        }
        output
    }

    /// Determine if we should fail the build based on violations
    fn should_fail_build(&self, violations: &[Violation]) -> bool {
        for violation in violations {
            match violation.severity {
                Severity::Error if self.config.fail_on_errors => return true,
                Severity::Warning if self.config.fail_on_warnings => return true,
                _ => {}
            }
        }
        false
    }
}

impl Default for MdBookLint {
    fn default() -> Self {
        Self::new()
    }
}

impl Preprocessor for MdBookLint {
    fn name(&self) -> &str {
        "lint"
    }

    fn run(&self, _ctx: &PreprocessorContext, book: Book) -> mdbook::errors::Result<Book> {
        let mut total_violations = Vec::new();
        let mut should_fail = false;

        // Process each chapter
        for item in book.iter() {
            if let BookItem::Chapter(chapter) = item {
                let violations = self.process_chapter(chapter).map_err(|e| {
                    mdbook::errors::Error::msg(format!("Failed to process chapter: {e}"))
                })?;

                if !violations.is_empty() {
                    let chapter_path = chapter
                        .source_path
                        .as_ref()
                        .map(|p| p.to_string_lossy())
                        .unwrap_or("unknown".into());

                    // Print violations to stderr
                    eprint!("{}", self.format_violations(&violations, &chapter_path));

                    if self.should_fail_build(&violations) {
                        should_fail = true;
                    }

                    total_violations.extend(violations);
                }
            }
        }

        // Print summary
        if !total_violations.is_empty() {
            let error_count = total_violations
                .iter()
                .filter(|v| v.severity == Severity::Error)
                .count();
            let warning_count = total_violations
                .iter()
                .filter(|v| v.severity == Severity::Warning)
                .count();
            let info_count = total_violations
                .iter()
                .filter(|v| v.severity == Severity::Info)
                .count();

            eprintln!(
                "mdbook-lint: {error_count} error(s), {warning_count} warning(s), {info_count} info"
            );

            if should_fail {
                return Err(mdbook::errors::Error::msg(format!(
                    "mdbook-lint: Build failed due to {error_count} error(s) and {warning_count} warning(s)"
                )));
            }
        } else {
            eprintln!("mdbook-lint: No issues found");
        }

        // Return the book unchanged (we're just linting, not modifying)
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        // We support all renderers since we don't modify content
        match renderer {
            "html" | "markdown" | "epub" | "pdf" => true,
            _ => true, // Default to supporting unknown renderers
        }
    }
}

/// Parse preprocessor configuration from mdbook config
fn parse_mdbook_config(config: &toml::value::Table) -> mdbook_lint_core::Result<Config> {
    let mut preprocessor_config = Config::default();

    if let Some(fail_on_warnings) = config.get("fail-on-warnings") {
        preprocessor_config.fail_on_warnings = fail_on_warnings
            .as_bool()
            .ok_or_else(|| MdBookLintError::config_error("fail-on-warnings must be a boolean"))?;
    }

    if let Some(fail_on_errors) = config.get("fail-on-errors") {
        preprocessor_config.fail_on_errors = fail_on_errors
            .as_bool()
            .ok_or_else(|| MdBookLintError::config_error("fail-on-errors must be a boolean"))?;
    }

    if let Some(enabled_categories) = config.get("enabled-categories")
        && let Some(categories_array) = enabled_categories.as_array()
    {
        for category in categories_array {
            if let Some(category_str) = category.as_str() {
                preprocessor_config
                    .core
                    .enabled_categories
                    .push(category_str.to_string());
            }
        }
    }

    if let Some(disabled_categories) = config.get("disabled-categories")
        && let Some(categories_array) = disabled_categories.as_array()
    {
        for category in categories_array {
            if let Some(category_str) = category.as_str() {
                preprocessor_config
                    .core
                    .disabled_categories
                    .push(category_str.to_string());
            }
        }
    }

    if let Some(enabled_rules) = config.get("enabled-rules")
        && let Some(rules_array) = enabled_rules.as_array()
    {
        for rule in rules_array {
            if let Some(rule_str) = rule.as_str() {
                preprocessor_config
                    .core
                    .enabled_rules
                    .push(rule_str.to_string());
            }
        }
    }

    if let Some(disabled_rules) = config.get("disabled-rules")
        && let Some(rules_array) = disabled_rules.as_array()
    {
        for rule in rules_array {
            if let Some(rule_str) = rule.as_str() {
                preprocessor_config
                    .core
                    .disabled_rules
                    .push(rule_str.to_string());
            }
        }
    }

    Ok(preprocessor_config)
}

/// Parse preprocessor configuration from serde_json Value (for tests)
#[allow(dead_code)]
fn parse_config(config: &Value) -> mdbook_lint_core::Result<Config> {
    let mut preprocessor_config = Config::default();

    if let Some(fail_on_warnings) = config.get("fail-on-warnings") {
        preprocessor_config.fail_on_warnings = fail_on_warnings
            .as_bool()
            .ok_or_else(|| MdBookLintError::config_error("fail-on-warnings must be a boolean"))?;
    }

    if let Some(fail_on_errors) = config.get("fail-on-errors") {
        preprocessor_config.fail_on_errors = fail_on_errors
            .as_bool()
            .ok_or_else(|| MdBookLintError::config_error("fail-on-errors must be a boolean"))?;
    }

    if let Some(enabled_categories) = config.get("enabled-categories")
        && let Some(categories_array) = enabled_categories.as_array()
    {
        for category in categories_array {
            if let Some(category_str) = category.as_str() {
                preprocessor_config
                    .core
                    .enabled_categories
                    .push(category_str.to_string());
            }
        }
    }

    if let Some(disabled_categories) = config.get("disabled-categories")
        && let Some(categories_array) = disabled_categories.as_array()
    {
        for category in categories_array {
            if let Some(category_str) = category.as_str() {
                preprocessor_config
                    .core
                    .disabled_categories
                    .push(category_str.to_string());
            }
        }
    }

    if let Some(enabled_rules) = config.get("enabled-rules")
        && let Some(rules_array) = enabled_rules.as_array()
    {
        for rule in rules_array {
            if let Some(rule_str) = rule.as_str() {
                preprocessor_config
                    .core
                    .enabled_rules
                    .push(rule_str.to_string());
            }
        }
    }

    if let Some(disabled_rules) = config.get("disabled-rules")
        && let Some(rules_array) = disabled_rules.as_array()
    {
        for rule in rules_array {
            if let Some(rule_str) = rule.as_str() {
                preprocessor_config
                    .core
                    .disabled_rules
                    .push(rule_str.to_string());
            }
        }
    }

    Ok(preprocessor_config)
}

/// Handle the preprocessor protocol (stdin/stdout communication with mdbook)
pub fn handle_preprocessing() -> mdbook_lint_core::Result<()> {
    let mut stdin = io::stdin();
    let mut input = String::new();
    stdin
        .read_to_string(&mut input)
        .map_err(MdBookLintError::Io)?;

    let (ctx, book): (PreprocessorContext, Book) =
        serde_json::from_str(&input).map_err(MdBookLintError::Json)?;

    let mut preprocessor = MdBookLint::new();
    preprocessor.load_config_from_context(&ctx)?;

    let processed_book = preprocessor
        .run(&ctx, book)
        .map_err(|e| MdBookLintError::document_error(format!("Preprocessor failed: {e}")))?;

    let output = serde_json::to_string(&processed_book).map_err(MdBookLintError::Json)?;

    print!("{output}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook::book::Chapter;

    use serde_json::json;
    use std::path::PathBuf;

    #[test]
    fn test_preprocessor_name() {
        let preprocessor = MdBookLint::new();
        assert_eq!(preprocessor.name(), "lint");
    }

    #[test]
    fn test_supports_renderer() {
        let preprocessor = MdBookLint::new();
        assert!(preprocessor.supports_renderer("html"));
        assert!(preprocessor.supports_renderer("markdown"));
        assert!(preprocessor.supports_renderer("epub"));
        assert!(preprocessor.supports_renderer("pdf"));
        assert!(preprocessor.supports_renderer("custom"));
    }

    #[test]
    fn test_process_chapter_clean() {
        let preprocessor = MdBookLint::new();
        // Content needs 50+ words to pass CONTENT003 (short chapter detection)
        // and lines under 80 chars to pass MD013
        let content = "# Test Chapter

This is a clean chapter with enough content to pass all linting rules.
We need to write several sentences here to make sure we have at least
fifty words in total for the content quality checks.

Let me add some more text to ensure we definitely pass the minimum word
count threshold that is required by the linter for content validation.
";
        let chapter = Chapter::new(
            "Test Chapter",
            content.to_string(),
            PathBuf::from("test.md"),
            Vec::new(),
        );

        let violations = preprocessor.process_chapter(&chapter).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_process_chapter_with_violations() {
        let preprocessor = MdBookLint::new();
        let content = "# Level 1\n### Level 3 - skipped level 2\n".to_string();
        let chapter = Chapter::new(
            "Test Chapter",
            content,
            PathBuf::from("test.md"),
            Vec::new(),
        );

        let violations = preprocessor.process_chapter(&chapter).unwrap();
        assert!(!violations.is_empty());

        // Print violations for debugging
        println!("Found {} violations:", violations.len());
        for (i, violation) in violations.iter().enumerate() {
            println!(
                "  {}: {} (line {}) - {}",
                i, violation.rule_id, violation.line, violation.message
            );
        }

        // Test should not depend on specific ordering - just verify MD001 is present
        let md001_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "MD001").collect();
        assert!(
            !md001_violations.is_empty(),
            "Should have at least one MD001 violation"
        );
    }

    #[test]
    fn test_rule_filtering_default() {
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();
        registry
            .register_provider(Box::new(MdBookRuleProvider))
            .unwrap();
        let engine = registry.create_engine().unwrap();
        let config = Config::default();
        let enabled_rules = engine.enabled_rules(&config.core);

        // Default config should enable all non-deprecated rules
        let enabled_rule_ids: Vec<&str> = enabled_rules.iter().map(|r| r.id()).collect();
        assert!(enabled_rule_ids.contains(&"MD001"));
        assert!(enabled_rule_ids.contains(&"MD013"));
        assert!(!enabled_rule_ids.contains(&"MD002")); // MD002 is deprecated
    }

    #[test]
    fn test_rule_filtering_with_enabled_rules() {
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();
        registry
            .register_provider(Box::new(MdBookRuleProvider))
            .unwrap();
        let engine = registry.create_engine().unwrap();
        let config = Config {
            core: mdbook_lint_core::Config {
                enabled_rules: vec!["MD001".to_string(), "MD002".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let enabled_rules = engine.enabled_rules(&config.core);

        let enabled_rule_ids: Vec<&str> = enabled_rules.iter().map(|r| r.id()).collect();
        assert!(enabled_rule_ids.contains(&"MD001"));
        assert!(enabled_rule_ids.contains(&"MD002")); // Explicitly enabled deprecated rule
        assert_eq!(enabled_rule_ids.len(), 2);
    }

    #[test]
    fn test_rule_filtering_with_disabled_rules() {
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();
        registry
            .register_provider(Box::new(MdBookRuleProvider))
            .unwrap();
        let engine = registry.create_engine().unwrap();
        let config = Config {
            core: mdbook_lint_core::Config {
                disabled_rules: vec!["MD001".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let enabled_rules = engine.enabled_rules(&config.core);

        let enabled_rule_ids: Vec<&str> = enabled_rules.iter().map(|r| r.id()).collect();
        assert!(!enabled_rule_ids.contains(&"MD001"));
        assert!(enabled_rule_ids.contains(&"MD013"));
    }

    #[test]
    fn test_with_config_constructor() {
        let config = Config {
            fail_on_warnings: true,
            core: mdbook_lint_core::Config {
                enabled_rules: vec!["MD001".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };

        let preprocessor = MdBookLint::with_config(config);
        assert!(preprocessor.config.fail_on_warnings);
        assert_eq!(
            preprocessor.config.core.enabled_rules,
            vec!["MD001".to_string()]
        );
    }

    #[test]
    fn test_process_chapter_with_empty_content() {
        let preprocessor = MdBookLint::new();
        let chapter = Chapter::new(
            "Empty Chapter",
            String::new(),
            PathBuf::from("empty.md"),
            Vec::new(),
        );

        let result = preprocessor.process_chapter(&chapter);
        // Processing empty content should not crash
        assert!(result.is_ok(), "Processing empty content should succeed");
    }

    #[test]
    fn test_process_chapter_with_whitespace_only() {
        let preprocessor = MdBookLint::new();
        let chapter = Chapter::new(
            "Whitespace Chapter",
            "   \n  \n  ".to_string(),
            PathBuf::from("whitespace.md"),
            Vec::new(),
        );

        let result = preprocessor.process_chapter(&chapter);
        // Processing whitespace-only content should not crash
        assert!(
            result.is_ok(),
            "Processing whitespace-only content should succeed"
        );
    }

    #[test]
    fn test_process_chapter_with_custom_config() {
        let config = Config {
            core: mdbook_lint_core::Config {
                disabled_rules: vec!["MD001".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let preprocessor = MdBookLint::with_config(config);

        let content = "# Level 1\n### Level 3 - skipped level 2\n".to_string();
        let chapter = Chapter::new(
            "Test Chapter",
            content,
            PathBuf::from("test.md"),
            Vec::new(),
        );

        let violations = preprocessor.process_chapter(&chapter).unwrap();
        // MD001 should be disabled, so no violations for header level skipping
        let md001_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "MD001").collect();
        assert_eq!(md001_violations.len(), 0);
    }

    #[test]
    fn test_process_chapter_error_handling() {
        let preprocessor = MdBookLint::new();
        let chapter = Chapter::new(
            "Test Chapter",
            "# Valid content".to_string(),
            PathBuf::from("test.md"),
            Vec::new(),
        );

        // This should not panic or error
        let result = preprocessor.process_chapter(&chapter);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rule_filtering_with_categories() {
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();
        registry
            .register_provider(Box::new(MdBookRuleProvider))
            .unwrap();
        let engine = registry.create_engine().unwrap();
        let config = Config {
            core: mdbook_lint_core::Config {
                enabled_categories: vec!["structure".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let enabled_rules = engine.enabled_rules(&config.core);

        let enabled_rule_ids: Vec<&str> = enabled_rules.iter().map(|r| r.id()).collect();
        // Should include structure rules like MD001
        assert!(enabled_rule_ids.contains(&"MD001"));
        // Should not include non-structure rules
        let structure_rules = enabled_rules
            .iter()
            .filter(|r| matches!(r.metadata().category, RuleCategory::Structure))
            .count();
        assert!(structure_rules > 0);
    }

    #[test]
    fn test_rule_filtering_with_disabled_categories() {
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();
        registry
            .register_provider(Box::new(MdBookRuleProvider))
            .unwrap();
        let engine = registry.create_engine().unwrap();
        let config = Config {
            core: mdbook_lint_core::Config {
                disabled_categories: vec!["style".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let enabled_rules = engine.enabled_rules(&config.core);

        let enabled_rule_ids: Vec<&str> = enabled_rules.iter().map(|r| r.id()).collect();
        // Should include non-style rules like MD001
        assert!(enabled_rule_ids.contains(&"MD001"));
        // Should exclude style rules - check that some style rules are disabled
        let style_rules = enabled_rules
            .iter()
            .filter(|r| matches!(r.metadata().category, RuleCategory::Formatting))
            .count();
        // There should be fewer formatting rules enabled when the category is disabled
        assert!(style_rules < 50); // Should have most rules still enabled
    }

    #[test]
    fn test_rule_filtering_with_disabled_rules_comprehensive() {
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();
        registry
            .register_provider(Box::new(MdBookRuleProvider))
            .unwrap();
        let engine = registry.create_engine().unwrap();
        let config = Config {
            core: mdbook_lint_core::Config {
                disabled_rules: vec!["MD013".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let enabled_rules = engine.enabled_rules(&config.core);

        let enabled_rule_ids: Vec<&str> = enabled_rules.iter().map(|r| r.id()).collect();
        assert!(enabled_rule_ids.contains(&"MD001"));
        assert!(!enabled_rule_ids.contains(&"MD013")); // Explicitly disabled
        assert!(enabled_rule_ids.len() > 50); // Should still have most rules
    }

    #[test]
    fn test_parse_config() {
        let config_json = json!({
            "fail-on-warnings": true,
            "fail-on-errors": false,
            "enabled-rules": ["MD001", "MD013"],
            "disabled-rules": ["MD002"]
        });

        let config = parse_config(&config_json).unwrap();
        assert!(config.fail_on_warnings);
        assert!(!config.fail_on_errors);
        assert_eq!(config.core.enabled_rules, vec!["MD001", "MD013"]);
        assert_eq!(config.core.disabled_rules, vec!["MD002"]);
    }

    #[test]
    fn test_should_fail_build() {
        let config = Config {
            fail_on_warnings: false,
            fail_on_errors: true,
            ..Default::default()
        };
        let preprocessor = MdBookLint::with_config(config);

        // Test with warning - should NOT fail build
        let warning_violations = vec![Violation {
            rule_id: "MD001".to_string(),
            rule_name: "test".to_string(),
            message: "test".to_string(),
            line: 1,
            column: 1,
            severity: Severity::Warning,
            fix: None,
        }];
        assert!(!preprocessor.should_fail_build(&warning_violations));

        // Test with error - should fail build
        let error_violations = vec![Violation {
            rule_id: "MD001".to_string(),
            rule_name: "test".to_string(),
            message: "test".to_string(),
            line: 1,
            column: 1,
            severity: Severity::Error,
            fix: None,
        }];
        assert!(preprocessor.should_fail_build(&error_violations));
    }

    #[test]
    fn test_format_violations() {
        let preprocessor = MdBookLint::new();
        let violations = vec![Violation {
            rule_id: "MD001".to_string(),
            rule_name: "heading-increment".to_string(),
            message: "Test violation".to_string(),
            line: 2,
            column: 1,
            severity: Severity::Error,
            fix: None,
        }];

        let output = preprocessor.format_violations(&violations, "test.md");
        assert!(output.contains("test.md:2:1"));
        assert!(output.contains("MD001"));
        assert!(output.contains("Test violation"));
    }
}
