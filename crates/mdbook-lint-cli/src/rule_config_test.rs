//! Tests for rule configuration functionality

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use mdbook_lint_core::{Document, PluginRegistry};
    use mdbook_lint_rulesets::StandardRuleProvider;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_document(content: &str) -> Document {
        Document::new(content.to_string(), PathBuf::from("test.md")).unwrap()
    }

    #[test]
    fn test_md013_configuration_works() {
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();

        // Create config with MD013 line-length = 120 and only MD013 enabled
        let config_toml = r#"
enabled-rules = ["MD013"]
[MD013]
line-length = 120
ignore-code-blocks = true
ignore-tables = false
ignore-headings = false
"#;
        let config = Config::from_toml_str(config_toml).unwrap();

        // Create engine with configuration
        let engine = registry
            .create_engine_with_config(Some(&config.core))
            .unwrap();

        // Test with 100-character line (should pass with line-length = 120)
        let content = "This line is exactly 100 characters long to test if MD013 configuration works properly here.";
        let document = create_test_document(content);

        let violations = engine
            .lint_document_with_config(&document, &config.core)
            .unwrap();

        // Should have no violations because line is under 120 characters
        assert_eq!(
            violations.len(),
            0,
            "Expected no violations with line-length=120"
        );

        // Test with default configuration (should fail)
        let engine_default = registry.create_engine().unwrap();
        let violations_default = engine_default
            .lint_document_with_config(&document, &mdbook_lint_core::Config::default())
            .unwrap();

        // Should have 1 violation because line is over default 80 characters
        assert_eq!(
            violations_default.len(),
            1,
            "Expected 1 violation with default line-length=80"
        );
        assert!(violations_default[0].message.contains("100 characters"));
    }

    #[test]
    fn test_md009_configuration_works() {
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();

        // Create config with MD009 br-spaces = 4 and only MD009 enabled
        let config_toml = r#"
enabled-rules = ["MD009"]
[MD009]
br-spaces = 4
strict = false
"#;
        let config = Config::from_toml_str(config_toml).unwrap();

        // Create engine with configuration
        let engine = registry
            .create_engine_with_config(Some(&config.core))
            .unwrap();

        // Test with 3 trailing spaces (should pass with br-spaces = 4)
        let content = "Line with 3 trailing spaces   \nAnother line\n";
        let document = create_test_document(content);

        let violations = engine
            .lint_document_with_config(&document, &config.core)
            .unwrap();

        // Should have no violations because 3 spaces < 4 allowed
        assert_eq!(
            violations.len(),
            0,
            "Expected no violations with br-spaces=4"
        );

        // Test with 5 trailing spaces (should fail)
        let content_5_spaces = "Line with 5 trailing spaces     \nAnother line\n";
        let document_5_spaces = create_test_document(content_5_spaces);

        let violations_5_spaces = engine
            .lint_document_with_config(&document_5_spaces, &config.core)
            .unwrap();

        // Should have 1 violation because 5 spaces > 4 allowed
        assert_eq!(
            violations_5_spaces.len(),
            1,
            "Expected 1 violation with 5 trailing spaces"
        );
    }

    #[test]
    fn test_md004_configuration_works() {
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();

        // Create config with MD004 style = "asterisk" and only MD004 enabled
        let config_toml = r#"
enabled-rules = ["MD004"]
[MD004]
style = "asterisk"
"#;
        let config = Config::from_toml_str(config_toml).unwrap();

        // Create engine with configuration
        let engine = registry
            .create_engine_with_config(Some(&config.core))
            .unwrap();

        // Test with mixed list styles (should fail for non-asterisk items)
        let content = r#"* First item with asterisk
+ Second item with plus
- Third item with dash
"#;
        let document = create_test_document(content);

        let violations = engine
            .lint_document_with_config(&document, &config.core)
            .unwrap();

        // Should have 2 violations for + and - items
        assert_eq!(
            violations.len(),
            2,
            "Expected 2 violations for non-asterisk items"
        );
        assert!(violations[0].message.contains("expected '*'"));
        assert!(violations[1].message.contains("expected '*'"));

        // Test with all asterisk items (should pass)
        let content_asterisk = r#"* First item
* Second item
* Third item
"#;
        let document_asterisk = create_test_document(content_asterisk);

        let violations_asterisk = engine
            .lint_document_with_config(&document_asterisk, &config.core)
            .unwrap();

        // Should have no violations
        assert_eq!(
            violations_asterisk.len(),
            0,
            "Expected no violations with all asterisk items"
        );
    }

    #[test]
    fn test_configuration_file_loading() {
        let temp_dir = TempDir::new().unwrap();

        // Create a config file
        let config_path = temp_dir.path().join(".mdbook-lint.toml");
        std::fs::write(
            &config_path,
            r#"
[MD013]
line-length = 100

[MD009]
br-spaces = 3

[MD004]
style = "dash"
"#,
        )
        .unwrap();

        // Load config from file
        let config = Config::from_file(&config_path).unwrap();

        // Verify configuration values are loaded correctly
        let md013_config = config.core.rule_configs.get("MD013").unwrap();
        assert_eq!(
            md013_config.get("line-length").unwrap().as_integer(),
            Some(100)
        );

        let md009_config = config.core.rule_configs.get("MD009").unwrap();
        assert_eq!(md009_config.get("br-spaces").unwrap().as_integer(), Some(3));

        let md004_config = config.core.rule_configs.get("MD004").unwrap();
        assert_eq!(md004_config.get("style").unwrap().as_str(), Some("dash"));
    }

    #[test]
    fn test_backward_compatibility() {
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();

        // Create engine without configuration (should use old register_rules method)
        let engine = registry.create_engine().unwrap();

        // Test that rules work with default values
        let content = "Line that is longer than the default 80 characters to test backward compatibility here.";
        let document = create_test_document(content);

        let violations = engine
            .lint_document_with_config(&document, &mdbook_lint_core::Config::default())
            .unwrap();

        // Should have 1 violation with default MD013 settings
        let md013_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "MD013").collect();
        assert_eq!(md013_violations.len(), 1);
        assert!(
            md013_violations[0]
                .message
                .contains("expected no more than 80")
        );
    }
}
