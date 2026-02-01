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
    /// Book source directory (set from PreprocessorContext in preprocessor mode)
    book_src_dir: Option<PathBuf>,
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
            book_src_dir: None,
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

        Self {
            config,
            engine,
            book_src_dir: None,
        }
    }

    /// Load configuration from preprocessor context
    ///
    /// Configuration is loaded from multiple sources with the following precedence
    /// (later sources override earlier ones):
    /// 1. Default configuration
    /// 2. `.mdbook-lint.toml` file (if found in book root or parent directories)
    /// 3. `[preprocessor.lint]` section in `book.toml`
    pub fn load_config_from_context(
        &mut self,
        ctx: &PreprocessorContext,
    ) -> mdbook_lint_core::Result<()> {
        let book_root = &ctx.root;

        // Compute the book source directory from context
        // This is root + book.src (which defaults to "src")
        let src_dir_name = ctx.config.book.src.to_str().unwrap_or("src").to_string();
        self.book_src_dir = Some(book_root.join(&src_dir_name));

        // First, try to discover and load .mdbook-lint.toml config file
        if let Some(discovered_path) = Config::discover_config(Some(book_root)) {
            self.config = Config::from_file(&discovered_path)?;
        }

        // Then, merge with book.toml preprocessor config (takes precedence)
        let preprocessor_config = ctx
            .config
            .get_preprocessor("mdbook-lint")
            .or_else(|| ctx.config.get_preprocessor("lint"));

        if let Some(config) = preprocessor_config {
            let book_toml_config = parse_mdbook_config(config)?;
            self.config.merge(book_toml_config);
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
        // When running in preprocessor mode, source_path is relative to the book source directory
        // We need to resolve it to an absolute path for rules that check file existence
        let source_path = chapter
            .source_path
            .as_ref()
            .unwrap_or(&PathBuf::from("unknown.md"))
            .clone();

        // If we have a book source directory, resolve the path to absolute
        let resolved_path = if let Some(ref book_src) = self.book_src_dir {
            book_src.join(&source_path)
        } else {
            source_path
        };

        let document = Document::with_book_src_dir(
            chapter.content.clone(),
            resolved_path,
            self.book_src_dir.clone(),
        )?;

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

/// Recursively remove null values from a JSON value, except for specific fields.
///
/// This is necessary because mdbook sends JSON with null values (e.g., `"description": null`),
/// but mdbook's Config struct deserializes through `toml::Value` which doesn't support null.
/// By stripping nulls before deserialization, we avoid the "invalid type: null, expected any
/// valid TOML value" error.
///
/// However, we preserve `__non_exhaustive` fields because mdbook's Book struct requires them
/// for deserialization (they deserialize to `()` which accepts null).
fn strip_null_values(value: Value) -> Value {
    match value {
        Value::Array(arr) => Value::Array(arr.into_iter().map(strip_null_values).collect()),
        Value::Object(obj) => Value::Object(
            obj.into_iter()
                .filter(|(k, v)| !v.is_null() || k == "__non_exhaustive")
                .map(|(k, v)| (k, strip_null_values(v)))
                .collect(),
        ),
        other => other,
    }
}

/// Normalize mdbook JSON format for compatibility between 0.4.x and 0.5.x.
///
/// mdbook 0.5.x made several changes to the JSON format:
/// 1. Book struct's field name changed from `sections` to `items`
/// 2. The `__non_exhaustive` field was removed from the JSON output
///
/// This function converts the 0.5.x format back to 0.4.x format so we can deserialize
/// using the mdbook 0.4.x types we depend on.
///
/// The input is expected to be a JSON array: `[PreprocessorContext, Book]`
/// Returns (normalized_value, is_mdbook_05) tuple.
fn normalize_mdbook_json(value: Value) -> (Value, bool) {
    let is_05 = detect_mdbook_05_format(&value);
    let normalized = normalize_mdbook_json_inner(value);
    (normalized, is_05)
}

/// Detect if the JSON is in mdbook 0.5.x format (uses "items" instead of "sections")
fn detect_mdbook_05_format(value: &Value) -> bool {
    if let Value::Array(arr) = value {
        // The Book object is the second element in the array
        if let Some(book) = arr.get(1) {
            return book.get("items").is_some() && book.get("sections").is_none();
        }
    }
    false
}

/// Inner normalization function that converts items -> sections
fn normalize_mdbook_json_inner(value: Value) -> Value {
    match value {
        Value::Array(arr) => {
            Value::Array(arr.into_iter().map(normalize_mdbook_json_inner).collect())
        }
        Value::Object(mut obj) => {
            // If this object has "items" but not "sections", it's a mdbook 0.5.x Book
            // We need to:
            // 1. Rename "items" to "sections"
            // 2. Add "__non_exhaustive": null (required by mdbook 0.4.x deserialization)
            if obj.contains_key("items")
                && !obj.contains_key("sections")
                && let Some(items) = obj.remove("items")
            {
                obj.insert("sections".to_string(), normalize_mdbook_json_inner(items));
                // Add __non_exhaustive if not present (mdbook 0.5.x removed it)
                if !obj.contains_key("__non_exhaustive") {
                    obj.insert("__non_exhaustive".to_string(), Value::Null);
                }
            }

            // Recursively normalize nested objects
            Value::Object(
                obj.into_iter()
                    .map(|(k, v)| (k, normalize_mdbook_json_inner(v)))
                    .collect(),
            )
        }
        other => other,
    }
}

/// Denormalize output for mdbook 0.5.x compatibility.
///
/// Converts "sections" back to "items" and removes "__non_exhaustive" fields
/// for mdbook 0.5.x compatibility.
fn denormalize_for_mdbook_05(value: Value) -> Value {
    match value {
        Value::Array(arr) => Value::Array(arr.into_iter().map(denormalize_for_mdbook_05).collect()),
        Value::Object(mut obj) => {
            // Remove __non_exhaustive (mdbook 0.5.x doesn't use it)
            obj.remove("__non_exhaustive");

            // If this object has "sections" but not "items", rename to "items"
            if obj.contains_key("sections")
                && !obj.contains_key("items")
                && let Some(sections) = obj.remove("sections")
            {
                obj.insert("items".to_string(), denormalize_for_mdbook_05(sections));
            }

            // Recursively denormalize nested objects
            Value::Object(
                obj.into_iter()
                    .map(|(k, v)| (k, denormalize_for_mdbook_05(v)))
                    .collect(),
            )
        }
        other => other,
    }
}

/// Handle the preprocessor protocol (stdin/stdout communication with mdbook)
pub fn handle_preprocessing() -> mdbook_lint_core::Result<()> {
    let mut stdin = io::stdin();
    let mut input = String::new();
    stdin
        .read_to_string(&mut input)
        .map_err(MdBookLintError::Io)?;

    // Parse as generic JSON first, then normalize and clean before deserializing.
    // 1. Normalize: Convert mdbook 0.5.x format (items) to 0.4.x format (sections)
    // 2. Strip nulls: Remove null values that toml::Value can't handle
    let json_value: Value = serde_json::from_str(&input).map_err(MdBookLintError::Json)?;
    let (normalized, is_mdbook_05) = normalize_mdbook_json(json_value);
    let cleaned = strip_null_values(normalized);

    let (ctx, book): (PreprocessorContext, Book) =
        serde_json::from_value(cleaned).map_err(MdBookLintError::Json)?;

    let mut preprocessor = MdBookLint::new();
    preprocessor.load_config_from_context(&ctx)?;

    let processed_book = preprocessor
        .run(&ctx, book)
        .map_err(|e| MdBookLintError::document_error(format!("Preprocessor failed: {e}")))?;

    // Serialize the book back to JSON
    let output_value = serde_json::to_value(&processed_book).map_err(MdBookLintError::Json)?;

    // If input was mdbook 0.5.x format, convert output back to 0.5.x format
    let final_output = if is_mdbook_05 {
        denormalize_for_mdbook_05(output_value)
    } else {
        output_value
    };

    let output = serde_json::to_string(&final_output).map_err(MdBookLintError::Json)?;

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

    #[test]
    fn test_strip_null_values() {
        // Test that null values are stripped from JSON objects
        let input = json!({
            "title": "My Book",
            "description": null,
            "authors": ["Author"],
            "nested": {
                "field": "value",
                "null_field": null
            },
            "array_with_nulls": [1, null, 3]
        });

        let result = strip_null_values(input);

        // Top-level null should be removed
        assert!(result.get("title").is_some());
        assert!(result.get("description").is_none());
        assert!(result.get("authors").is_some());

        // Nested null should be removed
        let nested = result.get("nested").unwrap();
        assert!(nested.get("field").is_some());
        assert!(nested.get("null_field").is_none());

        // Nulls in arrays are kept (arrays preserve structure)
        let arr = result.get("array_with_nulls").unwrap().as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_strip_null_values_preserves_non_exhaustive() {
        // __non_exhaustive fields should be preserved even when null
        let input = json!({
            "sections": [],
            "__non_exhaustive": null
        });

        let result = strip_null_values(input);

        assert!(result.get("sections").is_some());
        assert!(result.get("__non_exhaustive").is_some());
    }

    #[test]
    fn test_strip_null_values_preserves_valid_data() {
        let input = json!({
            "string": "hello",
            "number": 42,
            "bool": true,
            "array": [1, 2, 3],
            "object": {"key": "value"}
        });

        let result = strip_null_values(input.clone());

        // All non-null values should be preserved
        assert_eq!(result, input);
    }

    #[test]
    fn test_normalize_mdbook_json_v04_format() {
        // mdbook 0.4.x format uses "sections"
        let input = json!([
            {"root": "/tmp", "config": {}, "renderer": "html", "mdbook_version": "0.4.52"},
            {"sections": [{"Chapter": {"name": "Test", "content": "# Test"}}], "__non_exhaustive": null}
        ]);

        let (result, is_05) = normalize_mdbook_json(input.clone());

        // Should not detect as mdbook 0.5.x
        assert!(!is_05);

        // Should remain unchanged - already in 0.4.x format
        let book = result.as_array().unwrap().get(1).unwrap();
        assert!(book.get("sections").is_some());
        assert!(book.get("items").is_none());
    }

    #[test]
    fn test_normalize_mdbook_json_v05_format() {
        // mdbook 0.5.x format uses "items" and has no __non_exhaustive field
        let input = json!([
            {"root": "/tmp", "config": {}, "renderer": "html", "mdbook_version": "0.5.1"},
            {"items": [{"Chapter": {"name": "Test", "content": "# Test"}}]}
        ]);

        let (result, is_05) = normalize_mdbook_json(input);

        // Should detect as mdbook 0.5.x
        assert!(is_05);

        // "items" should be renamed to "sections"
        let book = result.as_array().unwrap().get(1).unwrap();
        assert!(book.get("sections").is_some());
        assert!(book.get("items").is_none());

        // __non_exhaustive should be added for mdbook 0.4.x compatibility
        assert!(book.get("__non_exhaustive").is_some());

        // Content should be preserved
        let sections = book.get("sections").unwrap().as_array().unwrap();
        assert_eq!(sections.len(), 1);
    }

    #[test]
    fn test_normalize_mdbook_json_nested_items() {
        // Chapters can have nested sub_items in 0.5.x
        let input = json!([
            {"root": "/tmp", "config": {}, "renderer": "html", "mdbook_version": "0.5.1"},
            {
                "items": [
                    {
                        "Chapter": {
                            "name": "Parent",
                            "content": "# Parent",
                            "sub_items": [
                                {"Chapter": {"name": "Child", "content": "# Child"}}
                            ]
                        }
                    }
                ],
                "__non_exhaustive": null
            }
        ]);

        let (result, _is_05) = normalize_mdbook_json(input);

        // Top-level "items" should be renamed to "sections"
        let book = result.as_array().unwrap().get(1).unwrap();
        assert!(book.get("sections").is_some());

        // sub_items should remain as "sub_items" (not renamed)
        let sections = book.get("sections").unwrap().as_array().unwrap();
        let chapter = sections[0].get("Chapter").unwrap();
        assert!(chapter.get("sub_items").is_some());
    }

    #[test]
    fn test_normalize_preserves_other_fields() {
        // Ensure normalization doesn't affect unrelated fields
        // Note: This is a non-standard structure (not mdbook format), so we use the inner function
        let input = json!({
            "items": [1, 2, 3],
            "other_field": "value",
            "nested": {
                "items": ["a", "b"],
                "data": 42
            }
        });

        let result = normalize_mdbook_json_inner(input);

        // "items" at each level should become "sections"
        assert!(result.get("sections").is_some());
        assert!(result.get("other_field").is_some());

        let nested = result.get("nested").unwrap();
        assert!(nested.get("sections").is_some());
        assert!(nested.get("data").is_some());
    }

    #[test]
    fn test_full_mdbook_04_json_parsing() {
        // Full mdbook 0.4.x JSON payload
        let input = json!([
            {
                "root": "/tmp/test",
                "config": {
                    "book": {
                        "title": "Test Book",
                        "authors": ["Author"],
                        "description": null,
                        "language": "en",
                        "text-direction": null
                    },
                    "output": {"html": {}},
                    "preprocessor": {}
                },
                "renderer": "html",
                "mdbook_version": "0.4.52"
            },
            {
                "sections": [
                    {
                        "Chapter": {
                            "name": "Introduction",
                            "content": "# Introduction\n\nWelcome!",
                            "number": [1],
                            "sub_items": [],
                            "path": "intro.md",
                            "source_path": "intro.md",
                            "parent_names": []
                        }
                    }
                ],
                "__non_exhaustive": null
            }
        ]);

        // Apply both normalization and null stripping
        let (normalized, is_05) = normalize_mdbook_json(input);
        let cleaned = strip_null_values(normalized);

        // Should not detect as mdbook 0.5.x
        assert!(!is_05);

        // Verify structure is correct for deserialization
        let arr = cleaned.as_array().unwrap();
        assert_eq!(arr.len(), 2);

        let ctx = &arr[0];
        assert!(ctx.get("root").is_some());
        assert!(ctx.get("config").is_some());

        let book = &arr[1];
        assert!(book.get("sections").is_some());
        assert!(book.get("__non_exhaustive").is_some());

        // Null values should be stripped from config
        let config = ctx.get("config").unwrap();
        let book_config = config.get("book").unwrap();
        assert!(book_config.get("description").is_none()); // null stripped
        assert!(book_config.get("title").is_some()); // non-null preserved
    }

    #[test]
    fn test_full_mdbook_05_json_parsing() {
        // Full mdbook 0.5.x JSON payload (no __non_exhaustive field)
        let input = json!([
            {
                "root": "/tmp/test",
                "config": {
                    "book": {
                        "title": "Test Book",
                        "authors": ["Author"],
                        "description": null,
                        "language": "en",
                        "text-direction": null
                    },
                    "output": {"html": {}},
                    "preprocessor": {}
                },
                "renderer": "html",
                "mdbook_version": "0.5.1"
            },
            {
                "items": [
                    {
                        "Chapter": {
                            "name": "Introduction",
                            "content": "# Introduction\n\nWelcome!",
                            "number": [1],
                            "sub_items": [],
                            "path": "intro.md",
                            "source_path": "intro.md",
                            "parent_names": []
                        }
                    }
                ]
            }
        ]);

        // Apply both normalization and null stripping
        let (normalized, is_05) = normalize_mdbook_json(input);
        let cleaned = strip_null_values(normalized);

        // Should detect as mdbook 0.5.x
        assert!(is_05);

        // Verify structure is correct for deserialization
        let arr = cleaned.as_array().unwrap();
        assert_eq!(arr.len(), 2);

        let book = &arr[1];
        // "items" should have been converted to "sections"
        assert!(book.get("sections").is_some());
        assert!(book.get("items").is_none());
        // __non_exhaustive should have been added
        assert!(book.get("__non_exhaustive").is_some());
    }

    #[test]
    fn test_denormalize_for_mdbook_05() {
        // Test that output is properly converted back to mdbook 0.5.x format
        let input = json!({
            "sections": [
                {"Chapter": {"name": "Test", "content": "# Test"}}
            ],
            "__non_exhaustive": null
        });

        let result = denormalize_for_mdbook_05(input);

        // "sections" should be renamed to "items"
        assert!(result.get("items").is_some());
        assert!(result.get("sections").is_none());

        // __non_exhaustive should be removed
        assert!(result.get("__non_exhaustive").is_none());
    }

    #[test]
    fn test_roundtrip_mdbook_05() {
        // Test full roundtrip: mdbook 0.5.x input -> normalize -> denormalize -> mdbook 0.5.x output
        let input = json!([
            {"root": "/tmp", "config": {}, "renderer": "html", "mdbook_version": "0.5.1"},
            {"items": [{"Chapter": {"name": "Test", "content": "# Test"}}]}
        ]);

        let (normalized, is_05) = normalize_mdbook_json(input.clone());
        assert!(is_05);

        // Verify normalization worked
        let book = normalized.as_array().unwrap().get(1).unwrap();
        assert!(book.get("sections").is_some());

        // Denormalize the book part (simulating what handle_preprocessing does)
        let denormalized_book = denormalize_for_mdbook_05(book.clone());

        // Should be back to mdbook 0.5.x format
        assert!(denormalized_book.get("items").is_some());
        assert!(denormalized_book.get("sections").is_none());
        assert!(denormalized_book.get("__non_exhaustive").is_none());
    }
}
