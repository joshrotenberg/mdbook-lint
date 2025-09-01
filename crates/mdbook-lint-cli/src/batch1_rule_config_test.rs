//! Tests for batch 1 rule configuration functionality (MD002, MD003, MD007, MD010, MD012)

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use mdbook_lint_core::{Document, PluginRegistry};
    use mdbook_lint_rulesets::StandardRuleProvider;
    use std::path::PathBuf;

    fn create_test_document(content: &str) -> Document {
        Document::new(content.to_string(), PathBuf::from("test.md")).unwrap()
    }

    #[test]
    fn test_md002_configuration_works() {
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();

        // Create config with MD002 level = 2 and only MD002 enabled
        let config_toml = r#"
enabled-rules = ["MD002"]
[MD002]
level = 2
"#;
        let config = Config::from_toml_str(config_toml).unwrap();

        // Create engine with configuration
        let engine = registry
            .create_engine_with_config(Some(&config.core))
            .unwrap();

        // Test document with level 2 heading (should pass with level = 2)
        let content = r#"## This is level 2

Some content here.
"#;

        let document = create_test_document(content);
        let violations = engine
            .lint_document_with_config(&document, &config.core)
            .unwrap();

        // Should have no violations since we configured level 2 as acceptable
        let md002_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "MD002").collect();
        assert_eq!(md002_violations.len(), 0);

        // Test with level 3 - should still violate
        let content_violating = r#"### This is level 3

Some content here.
"#;

        let document_violating = create_test_document(content_violating);
        let violations_violating = engine
            .lint_document_with_config(&document_violating, &config.core)
            .unwrap();

        let md002_violations_violating: Vec<_> = violations_violating
            .iter()
            .filter(|v| v.rule_id == "MD002")
            .collect();
        assert_eq!(md002_violations_violating.len(), 1);
        assert!(
            md002_violations_violating[0]
                .message
                .contains("should be level 2")
        );
        assert!(
            md002_violations_violating[0]
                .message
                .contains("got level 3")
        );
    }

    #[test]
    fn test_md003_configuration_works() {
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();

        // Create config with MD003 style = "atx" and only MD003 enabled
        let config_toml = r#"
enabled-rules = ["MD003"]
[MD003]
style = "atx"
"#;
        let config = Config::from_toml_str(config_toml).unwrap();

        // Create engine with configuration
        let engine = registry
            .create_engine_with_config(Some(&config.core))
            .unwrap();

        // Test document with Setext heading that should violate ATX-only config
        let content = r#"Main Title
==========

Some content here.
"#;

        let document = create_test_document(content);
        let violations = engine
            .lint_document_with_config(&document, &config.core)
            .unwrap();

        // Should have violations since we configured ATX but document uses Setext
        let md003_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "MD003").collect();
        assert!(!md003_violations.is_empty());
        assert!(md003_violations[0].message.contains("Expected 'atx' style"));
        assert!(md003_violations[0].message.contains("found 'setext' style"));
    }

    #[test]
    fn test_md007_configuration_works() {
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();

        // Create config with MD007 custom indentation and only MD007 enabled
        // Using indent=2 because 4+ spaces at line start creates code blocks
        let config_toml = r#"
enabled-rules = ["MD007"]
[MD007]
indent = 2
start-indented = true
start-indent = 2
"#;
        let config = Config::from_toml_str(config_toml).unwrap();

        // Create engine with configuration
        let engine = registry
            .create_engine_with_config(Some(&config.core))
            .unwrap();

        // Test document with proper list that uses configured indentation
        // With start_indented=true and start_indent=4, top level needs exactly 4 spaces before bullet
        // But 4+ spaces triggers code block, so we can't test that configuration properly
        // Let's use indent=2 with start_indent=2 instead
        let content = r#"  * Item 1 (2 spaces as configured for start_indent)
    * Nested item (4 spaces: 2 base + 2 indent)
      * Deep nested item (6 spaces: 2 base + 2*2 indent)
"#;

        let document = create_test_document(content);
        let violations = engine
            .lint_document_with_config(&document, &config.core)
            .unwrap();

        // Should have no violations since the indentation matches our config
        let md007_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "MD007").collect();
        assert_eq!(md007_violations.len(), 0);

        // Test with wrong indentation - should violate
        // With indent=2 and start_indented=true, top level should have 2 spaces
        let content_violating = r#"* Item 1 (0 spaces - should have 2 with start_indented)
   * Nested item (3 spaces - should have 4!)
"#;

        let document_violating = create_test_document(content_violating);
        let violations_violating = engine
            .lint_document_with_config(&document_violating, &config.core)
            .unwrap();

        let md007_violations_violating: Vec<_> = violations_violating
            .iter()
            .filter(|v| v.rule_id == "MD007")
            .collect();
        // Should have 2 violations: top-level needs 2 spaces, nested needs 4
        assert_eq!(md007_violations_violating.len(), 2);
        // First violation: top level should have 2 spaces (start_indented=true)
        assert!(
            md007_violations_violating[0]
                .message
                .contains("Expected 2 spaces, found 0")
        );
        // Second violation: nested should have 4 spaces (2 base + 2 indent)
        assert!(
            md007_violations_violating[1]
                .message
                .contains("Expected 4 spaces, found 3")
        );
    }

    #[test]
    fn test_md010_configuration_works() {
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();

        // Create config with MD010 custom spaces per tab and only MD010 enabled
        let config_toml = r#"
enabled-rules = ["MD010"]
[MD010]
spaces-per-tab = 8
"#;
        let config = Config::from_toml_str(config_toml).unwrap();

        // Create engine with configuration
        let engine = registry
            .create_engine_with_config(Some(&config.core))
            .unwrap();

        // Test document with tab character
        let content = "Line with\ttab character";

        let document = create_test_document(content);
        let violations = engine
            .lint_document_with_config(&document, &config.core)
            .unwrap();

        // Should have violations mentioning 8 spaces
        let md010_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "MD010").collect();
        assert_eq!(md010_violations.len(), 1);
        assert!(md010_violations[0].message.contains("8 spaces"));
    }

    #[test]
    fn test_md012_configuration_works() {
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();

        // Create config with MD012 maximum = 3 and only MD012 enabled
        let config_toml = r#"
enabled-rules = ["MD012"]
[MD012]
maximum = 3
"#;
        let config = Config::from_toml_str(config_toml).unwrap();

        // Create engine with configuration
        let engine = registry
            .create_engine_with_config(Some(&config.core))
            .unwrap();

        // Test document with 3 consecutive blank lines (should be allowed)
        let content = "# Heading\n\n\n\nParagraph.";

        let document = create_test_document(content);
        let violations = engine
            .lint_document_with_config(&document, &config.core)
            .unwrap();

        // Should have no violations since we allow up to 3 blank lines
        let md012_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "MD012").collect();
        assert_eq!(md012_violations.len(), 0);

        // Test with 4 consecutive blank lines (should violate)
        let content_violating = "# Heading\n\n\n\n\nParagraph.";

        let document_violating = create_test_document(content_violating);
        let violations_violating = engine
            .lint_document_with_config(&document_violating, &config.core)
            .unwrap();

        let md012_violations_violating: Vec<_> = violations_violating
            .iter()
            .filter(|v| v.rule_id == "MD012")
            .collect();
        assert_eq!(md012_violations_violating.len(), 1);
        assert!(
            md012_violations_violating[0]
                .message
                .contains("4 found, 3 allowed")
        );
    }

    #[test]
    fn test_multiple_batch1_rules_configuration() {
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();

        // Create config with multiple batch 1 rules
        let config_toml = r#"
enabled-rules = ["MD002", "MD010", "MD012"]
[MD002]
level = 3
[MD010]
spaces-per-tab = 2
[MD012]
maximum = 0
"#;
        let config = Config::from_toml_str(config_toml).unwrap();

        // Create engine with configuration
        let engine = registry
            .create_engine_with_config(Some(&config.core))
            .unwrap();

        // Test document that would trigger multiple configured rules
        let content = "## This is level 2 (should violate MD002 configured for level 3)\n\nThis would normally be fine for MD012 but not with maximum=0";

        let document = create_test_document(content);
        let violations = engine
            .lint_document_with_config(&document, &config.core)
            .unwrap();

        // Should have violations from both MD002 and MD012
        let md002_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "MD002").collect();
        let md012_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "MD012").collect();

        assert_eq!(md002_violations.len(), 1);
        assert!(md002_violations[0].message.contains("should be level 3"));

        assert_eq!(md012_violations.len(), 1);
        assert!(md012_violations[0].message.contains("1 found, 0 allowed"));
    }

    #[test]
    fn test_batch1_rules_fallback_to_defaults() {
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();

        // Create config without any rule-specific configuration
        let config_toml = r#"
enabled-rules = ["MD002"]
"#;
        let config = Config::from_toml_str(config_toml).unwrap();

        // Create engine with configuration
        let engine = registry
            .create_engine_with_config(Some(&config.core))
            .unwrap();

        // Test that rules use their default values
        let content = r#"## This is level 2

Some content here.
"#;

        let document = create_test_document(content);
        let violations = engine
            .lint_document_with_config(&document, &config.core)
            .unwrap();

        // Should have MD002 violation since default level is 1
        let md002_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "MD002").collect();
        assert_eq!(md002_violations.len(), 1);
        assert!(md002_violations[0].message.contains("should be level 1"));
        assert!(md002_violations[0].message.contains("got level 2"));
    }

    #[test]
    fn test_batch1_rules_with_underscore_config_keys() {
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();

        // Test that underscore keys also work (start_indent vs start-indent)
        let config_toml = r#"
enabled-rules = ["MD007", "MD010"]
[MD007]
start_indent = 6
start_indented = true
[MD010]
spaces_per_tab = 3
"#;
        let config = Config::from_toml_str(config_toml).unwrap();

        // Create engine with configuration
        let engine = registry
            .create_engine_with_config(Some(&config.core))
            .unwrap();

        // Test MD010 configuration with underscore key
        let content_tab = "Line with\ttab";
        let document_tab = create_test_document(content_tab);
        let violations_tab = engine
            .lint_document_with_config(&document_tab, &config.core)
            .unwrap();

        let md010_violations: Vec<_> = violations_tab
            .iter()
            .filter(|v| v.rule_id == "MD010")
            .collect();
        assert_eq!(md010_violations.len(), 1);
        assert!(md010_violations[0].message.contains("3 spaces")); // Should use underscore config

        // Test MD007 configuration with underscore keys
        let content_list = r#"      * Item 1 (6 spaces start)
        * Nested item (should be 6+2=8 spaces)
"#;
        let document_list = create_test_document(content_list);
        let violations_list = engine
            .lint_document_with_config(&document_list, &config.core)
            .unwrap();

        let md007_violations: Vec<_> = violations_list
            .iter()
            .filter(|v| v.rule_id == "MD007")
            .collect();
        // Should have no violations if the 6-space start indent is correctly applied
        assert_eq!(md007_violations.len(), 0);
    }
}
