use mdbook_lint_core::{MdBookLintError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Configuration for mdbook-lint CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Core linting configuration
    #[serde(flatten)]
    pub core: mdbook_lint_core::Config,

    /// Whether to fail builds on warnings (CLI-specific)
    #[serde(rename = "fail-on-warnings", default)]
    pub fail_on_warnings: bool,

    /// Whether to fail builds on errors (CLI-specific)
    #[serde(rename = "fail-on-errors", default = "default_fail_on_errors")]
    pub fail_on_errors: bool,

    /// How to handle malformed markdown (CLI-specific)
    #[serde(rename = "malformed-markdown", default)]
    pub malformed_markdown: MalformedMarkdownAction,
}

/// How to handle malformed markdown
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MalformedMarkdownAction {
    /// Quit on malformed markdown with error
    Error,
    /// Warn but continue processing
    #[default]
    Warn,
    /// Silently skip malformed files
    Skip,
}

// DeprecatedWarningLevel moved to core

/// Rule categories for bulk configuration
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleCategory {
    /// Document structure rules (headings, organization)
    Structure,
    /// Style and formatting rules
    Style,
    /// Whitespace and line ending rules
    Whitespace,
    /// Code block related rules
    Code,
    /// Link and reference rules
    Links,
    /// mdBook-specific rules
    MdBook,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            core: mdbook_lint_core::Config::default(),
            fail_on_warnings: false,
            fail_on_errors: true,
            malformed_markdown: MalformedMarkdownAction::Warn,
        }
    }
}

fn default_fail_on_errors() -> bool {
    true
}

#[allow(dead_code)]
impl Config {
    /// Load configuration from a file, auto-detecting format by extension
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| {
            MdBookLintError::config_error(format!(
                "Failed to read config file {}: {}",
                path.display(),
                e
            ))
        })?;

        // Check if this is a markdownlint.json file
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name == "markdownlint.json" || name == ".markdownlint.json")
            .unwrap_or(false)
        {
            return Self::from_markdownlint_json(&content);
        }

        // Auto-detect format from file extension
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("toml") => Self::from_toml_str(&content),
            Some("yaml") | Some("yml") => Self::from_yaml_str(&content),
            Some("json") => Self::from_json_str(&content),
            _ => {
                // Try to detect format from content if extension is ambiguous
                Self::detect_format_from_content(&content).map_err(|e| {
                    MdBookLintError::config_error(format!(
                        "Unable to determine config format for {}: {}",
                        path.display(),
                        e
                    ))
                })
            }
        }
    }
    /// Parse configuration from a TOML string
    pub fn from_toml_str(content: &str) -> Result<Self> {
        // First, parse as raw TOML to check for rules section
        let toml_value: toml::Value = toml::from_str(content).map_err(|e| {
            MdBookLintError::config_error(format!("Failed to parse TOML configuration: {e}"))
        })?;
        
        // Check if there's a rules section with default=false
        let has_rules_section = toml_value.get("rules").is_some();
        let rules_default = toml_value
            .get("rules")
            .and_then(|r| r.get("default"))
            .and_then(|d| d.as_bool())
            .unwrap_or(true);
        
        // Parse the config normally
        let mut config: Self = toml::from_str(content).map_err(|e| {
            MdBookLintError::config_error(format!("Failed to parse TOML configuration: {e}"))
        })?;
        
        // If there's a rules section with default=false, handle it specially
        if has_rules_section && !rules_default {
            // Clear enabled rules and only add those explicitly enabled in rules.enabled
            config.core.enabled_rules.clear();
            
            if let Some(rules) = toml_value.get("rules")
                && let Some(enabled) = rules.get("enabled")
                && let Some(enabled_map) = enabled.as_table()
            {
                for (rule_id, value) in enabled_map {
                    if value.as_bool().unwrap_or(false) {
                        config.core.enabled_rules.push(rule_id.clone());
                    }
                }
            }
        }
        
        Ok(config)
    }

    /// Parse configuration from a YAML string
    pub fn from_yaml_str(content: &str) -> Result<Self> {
        serde_yaml::from_str(content).map_err(|e| {
            MdBookLintError::config_error(format!("Failed to parse YAML configuration: {e}"))
        })
    }

    /// Parse configuration from a JSON string
    pub fn from_json_str(content: &str) -> Result<Self> {
        serde_json::from_str(content).map_err(|e| {
            MdBookLintError::config_error(format!("Failed to parse JSON configuration: {e}"))
        })
    }

    /// Serialize configuration to TOML string
    pub fn to_toml_string(&self) -> Result<String> {
        toml::to_string_pretty(self)
            .map_err(|e| MdBookLintError::config_error(format!("Failed to serialize to TOML: {e}")))
    }

    /// Serialize configuration to YAML string
    pub fn to_yaml_string(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| MdBookLintError::config_error(format!("Failed to serialize to YAML: {e}")))
    }

    /// Serialize configuration to JSON string
    pub fn to_json_string(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| MdBookLintError::config_error(format!("Failed to serialize to JSON: {e}")))
    }

    /// Create configuration from markdownlint.json format
    pub fn from_markdownlint_json(content: &str) -> Result<Self> {
        let markdownlint_config: serde_json::Value =
            serde_json::from_str(content).map_err(|e| {
                MdBookLintError::config_error(format!(
                    "Failed to parse markdownlint.json configuration: {e}"
                ))
            })?;

        Self::from_markdownlint_config(&markdownlint_config)
    }

    /// Convert markdownlint configuration format to our internal format
    pub fn from_markdownlint_config(config: &serde_json::Value) -> Result<Self> {
        let mut config_result = Config::default();
        let mut rule_configs = HashMap::new();

        if let Some(obj) = config.as_object() {
            for (key, value) in obj {
                // Skip comment fields
                if key.starts_with("//") {
                    continue;
                }

                match key.as_str() {
                    // Global settings
                    "default" => {
                        if let Some(default_val) = value.as_bool()
                            && !default_val
                        {
                            // If default is false, start with empty enabled categories
                            config_result.core.enabled_categories.clear();
                        }
                    }

                    // Map markdownlint rule names to MD### rule IDs
                    "heading-style" => {
                        if let Some(style_config) = Self::convert_markdownlint_rule(value, "MD003")
                        {
                            rule_configs.insert("MD003".to_string(), style_config);
                        }
                    }
                    "heading-start-left" => {
                        if value.as_bool() == Some(true) {
                            // This is enforced by default in MD018
                        }
                    }
                    "heading-increment" => {
                        if value.as_bool() == Some(true) {
                            // This is enforced by MD001
                        }
                    }
                    "no-duplicate-heading" => {
                        if let Some(config) = Self::convert_markdownlint_rule(value, "MD024") {
                            rule_configs.insert("MD024".to_string(), config);
                        }
                    }
                    "first-heading-h1" => {
                        if let Some(config) = Self::convert_markdownlint_rule(value, "MD041") {
                            rule_configs.insert("MD041".to_string(), config);
                        }
                    }
                    "blanks-around-headings" => {
                        if let Some(config) = Self::convert_markdownlint_rule(value, "MD022") {
                            rule_configs.insert("MD022".to_string(), config);
                        }
                    }
                    "ul-style" => {
                        if let Some(config) = Self::convert_markdownlint_rule(value, "MD004") {
                            rule_configs.insert("MD004".to_string(), config);
                        }
                    }
                    "ul-indent" => {
                        if let Some(config) = Self::convert_markdownlint_rule(value, "MD007") {
                            rule_configs.insert("MD007".to_string(), config);
                        }
                    }
                    "ol-prefix" => {
                        if let Some(config) = Self::convert_markdownlint_rule(value, "MD029") {
                            rule_configs.insert("MD029".to_string(), config);
                        }
                    }
                    "line-length" => {
                        if let Some(config) = Self::convert_line_length_config(value) {
                            rule_configs.insert("MD013".to_string(), config);
                        }
                    }
                    "no-hard-tabs" => {
                        if value.as_bool() == Some(true) {
                            // This is enforced by MD010
                        }
                    }
                    "no-trailing-spaces" => {
                        if value.as_bool() == Some(false) {
                            config_result.core.disabled_rules.push("MD009".to_string());
                        } else if let Some(config) = Self::convert_markdownlint_rule(value, "MD009")
                        {
                            rule_configs.insert("MD009".to_string(), config);
                        }
                    }
                    "no-multiple-blanks" => {
                        if let Some(config) = Self::convert_markdownlint_rule(value, "MD012") {
                            rule_configs.insert("MD012".to_string(), config);
                        }
                    }
                    "code-block-style" => {
                        if let Some(config) = Self::convert_markdownlint_rule(value, "MD046") {
                            rule_configs.insert("MD046".to_string(), config);
                        }
                    }
                    "code-fence-style" => {
                        if let Some(config) = Self::convert_markdownlint_rule(value, "MD048") {
                            rule_configs.insert("MD048".to_string(), config);
                        }
                    }
                    "fenced-code-language" => {
                        if let Some(config) = Self::convert_markdownlint_rule(value, "MD040") {
                            rule_configs.insert("MD040".to_string(), config);
                        }
                    }
                    "no-inline-html" => {
                        if let Some(config) = Self::convert_inline_html_config(value) {
                            rule_configs.insert("MD033".to_string(), config);
                        }
                    }
                    "emphasis-style" => {
                        if let Some(config) = Self::convert_markdownlint_rule(value, "MD049") {
                            rule_configs.insert("MD049".to_string(), config);
                        }
                    }
                    "strong-style" => {
                        if let Some(config) = Self::convert_markdownlint_rule(value, "MD050") {
                            rule_configs.insert("MD050".to_string(), config);
                        }
                    }
                    "hr-style" => {
                        if let Some(config) = Self::convert_markdownlint_rule(value, "MD035") {
                            rule_configs.insert("MD035".to_string(), config);
                        }
                    }
                    _ => {
                        // For unknown rules, check if it's a boolean disable
                        if value.as_bool() == Some(false) {
                            // Try to map the rule name to an MD### rule and disable it
                            if let Some(md_rule) = Self::markdownlint_to_md_rule(key) {
                                config_result.core.disabled_rules.push(md_rule);
                            }
                        }
                    }
                }
            }
        }

        config_result.core.rule_configs = rule_configs;
        Ok(config_result)
    }

    /// Convert markdownlint rule configuration to TOML value
    fn convert_markdownlint_rule(value: &serde_json::Value, _rule_id: &str) -> Option<toml::Value> {
        // Convert JSON value to TOML value
        serde_json::from_value::<toml::Value>(value.clone()).ok()
    }

    /// Special handling for line-length configuration
    fn convert_line_length_config(value: &serde_json::Value) -> Option<toml::Value> {
        if let Some(obj) = value.as_object() {
            let mut config = toml::value::Table::new();

            if let Some(length) = obj.get("line_length").and_then(|v| v.as_i64()) {
                config.insert("line-length".to_string(), toml::Value::Integer(length));
            }

            if let Some(code_blocks) = obj.get("code_blocks").and_then(|v| v.as_bool()) {
                config.insert(
                    "ignore-code-blocks".to_string(),
                    toml::Value::Boolean(!code_blocks),
                );
            }

            if let Some(tables) = obj.get("tables").and_then(|v| v.as_bool()) {
                config.insert("ignore-tables".to_string(), toml::Value::Boolean(!tables));
            }

            Some(toml::Value::Table(config))
        } else {
            None
        }
    }

    /// Special handling for inline HTML configuration
    fn convert_inline_html_config(value: &serde_json::Value) -> Option<toml::Value> {
        if let Some(obj) = value.as_object() {
            let mut config = toml::value::Table::new();

            if let Some(allowed) = obj.get("allowed_elements")
                && let Ok(toml_allowed) = serde_json::from_value::<toml::Value>(allowed.clone())
            {
                config.insert("allowed-elements".to_string(), toml_allowed);
            }

            Some(toml::Value::Table(config))
        } else {
            None
        }
    }

    /// Map markdownlint rule names to MD### rule IDs
    fn markdownlint_to_md_rule(markdownlint_name: &str) -> Option<String> {
        match markdownlint_name {
            "heading-style" => Some("MD003".to_string()),
            "heading-start-left" => Some("MD018".to_string()),
            "heading-increment" => Some("MD001".to_string()),
            "no-duplicate-heading" => Some("MD024".to_string()),
            "first-heading-h1" => Some("MD041".to_string()),
            "blanks-around-headings" => Some("MD022".to_string()),
            "ul-style" => Some("MD004".to_string()),
            "ul-indent" => Some("MD007".to_string()),
            "ol-prefix" => Some("MD029".to_string()),
            "line-length" => Some("MD013".to_string()),
            "no-hard-tabs" => Some("MD010".to_string()),
            "no-trailing-spaces" => Some("MD009".to_string()),
            "no-multiple-blanks" => Some("MD012".to_string()),
            "code-block-style" => Some("MD046".to_string()),
            "code-fence-style" => Some("MD048".to_string()),
            "fenced-code-language" => Some("MD040".to_string()),
            "no-inline-html" => Some("MD033".to_string()),
            "emphasis-style" => Some("MD049".to_string()),
            "strong-style" => Some("MD050".to_string()),
            "hr-style" => Some("MD035".to_string()),
            "no-bare-urls" => Some("MD034".to_string()),
            "no-alt-text" => Some("MD045".to_string()),
            "no-space-in-code" => Some("MD038".to_string()),
            "no-space-in-emphasis" => Some("MD037".to_string()),
            "no-space-in-links" => Some("MD039".to_string()),
            "no-empty-links" => Some("MD042".to_string()),
            "reference-links-images" => Some("MD052".to_string()),
            "link-image-style" => Some("MD051".to_string()),
            "single-trailing-newline" => Some("MD047".to_string()),
            _ => None,
        }
    }

    /// Attempt to detect configuration format from content and parse
    fn detect_format_from_content(content: &str) -> Result<Self> {
        let trimmed = content.trim();

        // Try JSON first (starts with { or [)
        if (trimmed.starts_with('{') || trimmed.starts_with('['))
            && let Ok(config) = Self::from_json_str(content)
        {
            return Ok(config);
        }

        // Try YAML (often starts with --- or has key: value format)
        if (trimmed.starts_with("---") || content.contains(": "))
            && let Ok(config) = Self::from_yaml_str(content)
        {
            return Ok(config);
        }

        // Fall back to TOML
        Self::from_toml_str(content).map_err(|e| {
            MdBookLintError::config_error(format!("Failed to parse as TOML, YAML, or JSON: {e}"))
        })
    }

    /// Load configuration from mdBook preprocessor config
    pub fn from_preprocessor_config(config: &serde_json::Value) -> Result<Self> {
        let preprocessor_config = config
            .get("preprocessor")
            .and_then(|p| p.get("lint"))
            .unwrap_or(&serde_json::Value::Null);

        if preprocessor_config.is_null() {
            return Ok(Self::default());
        }

        // Convert JSON to TOML-compatible format for parsing
        let toml_value: toml::Value =
            serde_json::from_value(preprocessor_config.clone()).map_err(|e| {
                MdBookLintError::config_error(format!("Failed to convert preprocessor config: {e}"))
            })?;

        toml_value.try_into().map_err(|e| {
            MdBookLintError::config_error(format!(
                "Failed to parse preprocessor configuration: {e}"
            ))
        })
    }

    /// Check if a rule should be enabled based on configuration
    pub fn should_run_rule(&self, rule_id: &str) -> bool {
        // Explicit disabled rules always take precedence
        if self.core.disabled_rules.contains(&rule_id.to_string()) {
            return false;
        }

        // If we have explicit enabled rules, only those should run
        if !self.core.enabled_rules.is_empty() {
            return self.core.enabled_rules.contains(&rule_id.to_string());
        }

        // Check markdownlint compatibility mode - disable rules that are disabled by default in markdownlint
        if self.core.markdownlint_compatible && rule_id == "MD044" {
            return false; // proper-names: disabled by default in markdownlint
        }

        // Check category-based configuration
        let category = self.get_rule_category(rule_id);
        let category_name = self.category_to_string(&category);

        if self.core.disabled_categories.contains(&category_name) {
            return false;
        }

        // If enabled_categories is not empty, only run rules in enabled categories
        if !self.core.enabled_categories.is_empty() {
            return self.core.enabled_categories.contains(&category_name);
        }

        // Default: run all rules except those in disabled categories
        true
    }

    /// Get the category for a rule based on its ID
    fn get_rule_category(&self, rule_id: &str) -> RuleCategory {
        match rule_id {
            // Structure rules
            "MD001" | "MD022" | "MD023" | "MD041" | "MD043" => RuleCategory::Structure,

            // Style rules
            "MD003" | "MD004" | "MD035" | "MD049" | "MD050" => RuleCategory::Style,

            // Whitespace rules
            "MD009" | "MD010" | "MD012" | "MD047" => RuleCategory::Whitespace,

            // Code rules
            "MD013" | "MD014" | "MD031" | "MD038" | "MD040" | "MD046" | "MD048" => {
                RuleCategory::Code
            }

            // Link rules
            "MD011" | "MD034" | "MD039" | "MD042" | "MD051" | "MD052" | "MD053" | "MD054" => {
                RuleCategory::Links
            }

            // mdBook-specific rules
            rule_id if rule_id.starts_with("MDBOOK") => RuleCategory::MdBook,

            // Default to structure for unknown rules
            _ => RuleCategory::Structure,
        }
    }

    /// Convert rule category to string
    fn category_to_string(&self, category: &RuleCategory) -> String {
        match category {
            RuleCategory::Structure => "structure".to_string(),
            RuleCategory::Style => "style".to_string(),
            RuleCategory::Whitespace => "whitespace".to_string(),
            RuleCategory::Code => "code".to_string(),
            RuleCategory::Links => "links".to_string(),
            RuleCategory::MdBook => "mdbook".to_string(),
        }
    }

    /// Get rule-specific configuration
    pub fn get_rule_config(&self, rule_id: &str) -> Option<&toml::Value> {
        self.core.rule_configs.get(rule_id)
    }

    /// Discover config file by searching common locations
    ///
    /// Searches in this order:
    /// 1. Current directory
    /// 2. Parent directories up to root
    ///
    /// Looks for these filenames:
    /// - .mdbook-lint.toml (preferred)
    /// - mdbook-lint.toml
    /// - .mdbook-lint.yaml
    /// - .mdbook-lint.yml
    /// - .mdbook-lint.json
    pub fn discover_config(start_dir: Option<&Path>) -> Option<PathBuf> {
        const CONFIG_NAMES: &[&str] = &[
            ".mdbook-lint.toml",
            "mdbook-lint.toml",
            ".mdbook-lint.yaml",
            ".mdbook-lint.yml",
            ".mdbook-lint.json",
        ];

        let start = start_dir
            .map(|p| p.to_path_buf())
            .or_else(|| env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));

        let mut current = start.clone();
        loop {
            // Check for config files in current directory
            for config_name in CONFIG_NAMES {
                let config_path = current.join(config_name);
                if config_path.exists() && config_path.is_file() {
                    eprintln!("DEBUG: Found config at {}", config_path.display());
                    return Some(config_path);
                }
            }

            // Move to parent directory
            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            } else {
                break;
            }
        }

        eprintln!(
            "DEBUG: No config file found searching from {}",
            start.display()
        );
        None
    }

    /// Merge this config with another, with the other taking precedence
    pub fn merge(&mut self, other: Config) {
        if other.fail_on_warnings {
            self.fail_on_warnings = other.fail_on_warnings;
        }
        if !other.fail_on_errors {
            self.fail_on_errors = other.fail_on_errors;
        }
        if other.core.markdownlint_compatible {
            self.core.markdownlint_compatible = other.core.markdownlint_compatible;
        }

        // Merge rule lists
        if !other.core.enabled_rules.is_empty() {
            self.core.enabled_rules = other.core.enabled_rules;
        }
        if !other.core.disabled_rules.is_empty() {
            self.core.disabled_rules = other.core.disabled_rules;
        }
        if !other.core.enabled_categories.is_empty() {
            self.core.enabled_categories = other.core.enabled_categories;
        }
        if !other.core.disabled_categories.is_empty() {
            self.core.disabled_categories = other.core.disabled_categories;
        }

        // Merge rule-specific configs
        self.core.rule_configs.extend(other.core.rule_configs);
    }
}

impl FromStr for Config {
    type Err = MdBookLintError;

    /// Auto-detect format and parse configuration from string
    ///
    /// Tries to parse the input as JSON, TOML, then YAML in that order.
    fn from_str(content: &str) -> Result<Self> {
        // Try JSON first (most structured)
        if let Ok(config) = Self::from_json_str(content) {
            return Ok(config);
        }

        // Try TOML next (common for Rust projects)
        if let Ok(config) = Self::from_toml_str(content) {
            return Ok(config);
        }

        // Try YAML last
        if let Ok(config) = Self::from_yaml_str(content) {
            return Ok(config);
        }

        Err(MdBookLintError::config_error(
            "Could not parse configuration as JSON, TOML, or YAML",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert!(!config.fail_on_warnings);
        assert!(config.fail_on_errors);
        assert!(matches!(
            config.malformed_markdown,
            MalformedMarkdownAction::Warn
        ));
        assert_eq!(config.core.enabled_categories.len(), 0);
        assert_eq!(config.core.disabled_categories.len(), 0);
        assert_eq!(config.core.enabled_rules.len(), 0);
        assert_eq!(config.core.disabled_rules.len(), 0);
    }

    #[test]
    fn test_rule_categorization() {
        let config = Config::default();

        assert_eq!(config.get_rule_category("MD001"), RuleCategory::Structure);
        assert_eq!(config.get_rule_category("MD013"), RuleCategory::Code);
        assert_eq!(config.get_rule_category("MDBOOK001"), RuleCategory::MdBook);
        assert_eq!(config.get_rule_category("MD009"), RuleCategory::Whitespace);
        assert_eq!(config.get_rule_category("MD011"), RuleCategory::Links);
    }

    #[test]
    fn test_should_run_rule_explicit_disabled() {
        let config = Config {
            core: mdbook_lint_core::Config {
                disabled_rules: vec!["MD001".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };

        assert!(!config.should_run_rule("MD001"));
        assert!(config.should_run_rule("MD013"));
    }

    #[test]
    fn test_should_run_rule_explicit_enabled() {
        let config = Config {
            core: mdbook_lint_core::Config {
                enabled_rules: vec!["MD001".to_string()],
                disabled_rules: vec!["MD001".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };

        // Disabled rules take precedence
        assert!(!config.should_run_rule("MD001"));
        assert!(!config.should_run_rule("MD013")); // Not in enabled rules
    }

    #[test]
    fn test_should_run_rule_category_based() {
        let config = Config {
            core: mdbook_lint_core::Config {
                enabled_categories: vec!["structure".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };

        assert!(config.should_run_rule("MD001")); // structure category
        assert!(!config.should_run_rule("MD013")); // style category
    }

    #[test]
    fn test_should_run_rule_disabled_category() {
        let config = Config {
            core: mdbook_lint_core::Config {
                disabled_categories: vec!["code".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };

        assert!(config.should_run_rule("MD001")); // structure
        assert!(!config.should_run_rule("MD013")); // code - disabled category
    }

    #[test]
    fn test_config_from_toml() {
        let toml_config = r#"
fail-on-warnings = true
fail-on-errors = false
malformed-markdown = "error"
enabled-rules = ["MD001", "MD013"]
disabled-rules = ["MD009"]

[MD013]
line-length = 120
ignore-code-blocks = true
"#;

        let config = Config::from_toml_str(toml_config).unwrap();

        assert!(config.fail_on_warnings);
        assert!(!config.fail_on_errors);
        assert!(matches!(
            config.malformed_markdown,
            MalformedMarkdownAction::Error
        ));
        assert_eq!(config.core.enabled_rules, vec!["MD001", "MD013"]);
        assert_eq!(config.core.disabled_rules, vec!["MD009"]);

        let md013_config = config.get_rule_config("MD013").unwrap();
        assert_eq!(
            md013_config.get("line-length").unwrap().as_integer(),
            Some(120)
        );
        assert_eq!(
            md013_config.get("ignore-code-blocks").unwrap().as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_config_from_preprocessor() {
        let json_config = serde_json::json!({
            "book": {
                "title": "Test Book"
            },
            "preprocessor": {
                "lint": {
                    "fail-on-warnings": true,
                    "enabled-rules": ["MD001"],
                    "MD013": {
                        "line-length": 100
                    }
                }
            }
        });

        let config = Config::from_preprocessor_config(&json_config).unwrap();

        assert!(config.fail_on_warnings);
        assert_eq!(config.core.enabled_rules, vec!["MD001"]);

        let md013_config = config.get_rule_config("MD013").unwrap();
        assert_eq!(
            md013_config.get("line-length").unwrap().as_integer(),
            Some(100)
        );
    }

    #[test]
    fn test_config_merge() {
        let mut base_config = Config {
            core: mdbook_lint_core::Config {
                enabled_rules: vec!["MD001".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };

        let mut override_config = Config {
            fail_on_warnings: true,
            core: mdbook_lint_core::Config {
                enabled_rules: vec!["MD013".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        override_config.core.rule_configs.insert(
            "MD013".to_string(),
            toml::Value::try_from([("line-length", 100)]).unwrap(),
        );

        base_config.merge(override_config);

        assert!(base_config.fail_on_warnings);
        assert_eq!(base_config.core.enabled_rules, vec!["MD013"]);
        assert!(base_config.core.rule_configs.contains_key("MD013"));
    }

    #[test]
    fn test_config_from_yaml() {
        let yaml_config = r#"
fail-on-warnings: true
fail-on-errors: false
malformed-markdown: error
enabled-rules:
  - MD001
  - MD013
disabled-rules:
  - MD009
MD013:
  line-length: 120
  ignore-code-blocks: true
"#;

        let config = Config::from_yaml_str(yaml_config).unwrap();

        assert!(config.fail_on_warnings);
        assert!(!config.fail_on_errors);
        assert!(matches!(
            config.malformed_markdown,
            MalformedMarkdownAction::Error
        ));
        assert_eq!(config.core.enabled_rules, vec!["MD001", "MD013"]);
        assert_eq!(config.core.disabled_rules, vec!["MD009"]);

        let md013_config = config.get_rule_config("MD013").unwrap();
        assert_eq!(
            md013_config.get("line-length").unwrap().as_integer(),
            Some(120)
        );
        assert_eq!(
            md013_config.get("ignore-code-blocks").unwrap().as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_config_from_json() {
        let json_config = r#"{
    "fail-on-warnings": true,
    "fail-on-errors": false,
    "malformed-markdown": "error",
    "enabled-rules": ["MD001", "MD013"],
    "disabled-rules": ["MD009"],
    "MD013": {
        "line-length": 120,
        "ignore-code-blocks": true
    }
}"#;

        let config = Config::from_json_str(json_config).unwrap();

        assert!(config.fail_on_warnings);
        assert!(!config.fail_on_errors);
        assert!(matches!(
            config.malformed_markdown,
            MalformedMarkdownAction::Error
        ));
        assert_eq!(config.core.enabled_rules, vec!["MD001", "MD013"]);
        assert_eq!(config.core.disabled_rules, vec!["MD009"]);

        let md013_config = config.get_rule_config("MD013").unwrap();
        assert_eq!(
            md013_config.get("line-length").unwrap().as_integer(),
            Some(120)
        );
        assert_eq!(
            md013_config.get("ignore-code-blocks").unwrap().as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_detect_format_from_content_json() {
        let json_content = r#"{
    "fail-on-warnings": true,
    "enabled-rules": ["MD001"]
}"#;

        let config = Config::detect_format_from_content(json_content).unwrap();
        assert!(config.fail_on_warnings);
        assert_eq!(config.core.enabled_rules, vec!["MD001"]);
    }

    #[test]
    fn test_detect_format_from_content_yaml() {
        let yaml_content = r#"---
fail-on-warnings: true
enabled-rules:
  - MD001
"#;

        let config = Config::detect_format_from_content(yaml_content).unwrap();
        assert!(config.fail_on_warnings);
        assert_eq!(config.core.enabled_rules, vec!["MD001"]);
    }

    #[test]
    fn test_detect_format_from_content_yaml_no_frontmatter() {
        let yaml_content = r#"fail-on-warnings: true
enabled-rules:
  - MD001
"#;

        let config = Config::detect_format_from_content(yaml_content).unwrap();
        assert!(config.fail_on_warnings);
        assert_eq!(config.core.enabled_rules, vec!["MD001"]);
    }

    #[test]
    fn test_detect_format_from_content_toml_fallback() {
        let toml_content = r#"fail-on-warnings = true
enabled-rules = ["MD001"]
"#;

        let config = Config::detect_format_from_content(toml_content).unwrap();
        assert!(config.fail_on_warnings);
        assert_eq!(config.core.enabled_rules, vec!["MD001"]);
    }

    #[test]
    fn test_yaml_with_categories() {
        let yaml_config = r#"
enabled-categories:
  - structure
  - style
disabled-categories:
  - whitespace
deprecated-warning: info
"#;

        let config = Config::from_yaml_str(yaml_config).unwrap();

        assert_eq!(config.core.enabled_categories, vec!["structure", "style"]);
        assert_eq!(config.core.disabled_categories, vec!["whitespace"]);
        assert!(matches!(
            config.core.deprecated_warning,
            mdbook_lint_core::config::DeprecatedWarningLevel::Info
        ));
    }

    #[test]
    fn test_json_with_categories() {
        let json_config = r#"{
    "enabled-categories": ["structure", "style"],
    "disabled-categories": ["whitespace"],
    "deprecated-warning": "silent"
}"#;

        let config = Config::from_json_str(json_config).unwrap();

        assert_eq!(config.core.enabled_categories, vec!["structure", "style"]);
        assert_eq!(config.core.disabled_categories, vec!["whitespace"]);
        assert!(matches!(
            config.core.deprecated_warning,
            mdbook_lint_core::config::DeprecatedWarningLevel::Silent
        ));
    }

    #[test]
    fn test_invalid_format_detection() {
        let invalid_content = "this is not valid config in any format [}";
        let result = Config::detect_format_from_content(invalid_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_markdownlint_json_parsing() {
        let markdownlint_config = r#"{
    "heading-style": {
        "style": "atx"
    },
    "line-length": {
        "line_length": 120,
        "code_blocks": false,
        "tables": false
    },
    "ul-style": {
        "style": "dash"
    },
    "no-inline-html": {
        "allowed_elements": ["br", "sup", "sub"]
    },
    "no-hard-tabs": true,
    "no-trailing-spaces": false
}"#;

        let config = Config::from_markdownlint_json(markdownlint_config).unwrap();

        // Check that rule configs were converted
        let md013_config = config.get_rule_config("MD013").unwrap();
        assert_eq!(
            md013_config.get("line-length").unwrap().as_integer(),
            Some(120)
        );
        assert_eq!(
            md013_config.get("ignore-code-blocks").unwrap().as_bool(),
            Some(true)
        );

        let md004_config = config.get_rule_config("MD004").unwrap();
        assert_eq!(md004_config.get("style").unwrap().as_str(), Some("dash"));

        let md033_config = config.get_rule_config("MD033").unwrap();
        let allowed_elements = md033_config
            .get("allowed-elements")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(allowed_elements.len(), 3);

        // Check disabled rules
        assert!(config.core.disabled_rules.contains(&"MD009".to_string()));
    }

    #[test]
    fn test_markdownlint_json_file_detection() {
        let markdownlint_config = r#"{
    "heading-style": {
        "style": "atx"
    },
    "line-length": {
        "line_length": 100
    }
}"#;

        // Test with markdownlint.json filename
        let temp_dir = tempfile::tempdir().unwrap();
        let markdownlint_path = temp_dir.path().join("markdownlint.json");
        std::fs::write(&markdownlint_path, markdownlint_config).unwrap();

        let config = Config::from_file(&markdownlint_path).unwrap();
        let md013_config = config.get_rule_config("MD013").unwrap();
        assert_eq!(
            md013_config.get("line-length").unwrap().as_integer(),
            Some(100)
        );

        // Test with .markdownlint.json filename
        let dotmarkdownlint_path = temp_dir.path().join(".markdownlint.json");
        std::fs::write(&dotmarkdownlint_path, markdownlint_config).unwrap();

        let config2 = Config::from_file(&dotmarkdownlint_path).unwrap();
        let md013_config2 = config2.get_rule_config("MD013").unwrap();
        assert_eq!(
            md013_config2.get("line-length").unwrap().as_integer(),
            Some(100)
        );
    }

    #[test]
    fn test_markdownlint_rule_mapping() {
        assert_eq!(
            Config::markdownlint_to_md_rule("heading-style"),
            Some("MD003".to_string())
        );
        assert_eq!(
            Config::markdownlint_to_md_rule("line-length"),
            Some("MD013".to_string())
        );
        assert_eq!(Config::markdownlint_to_md_rule("unknown-rule"), None);
    }

    #[test]
    fn test_config_from_toml_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let toml_config = r#"
fail-on-warnings = true
enabled-rules = ["MD001", "MD013"]

[MD013]
line-length = 100
"#;

        let mut temp_file = NamedTempFile::with_suffix(".toml").unwrap();
        temp_file.write_all(toml_config.as_bytes()).unwrap();

        let config = Config::from_file(temp_file.path()).unwrap();
        assert!(config.fail_on_warnings);
        assert_eq!(config.core.enabled_rules, vec!["MD001", "MD013"]);

        let md013_config = config.get_rule_config("MD013").unwrap();
        assert_eq!(
            md013_config.get("line-length").unwrap().as_integer(),
            Some(100)
        );
    }

    #[test]
    fn test_config_from_yaml_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let yaml_config = r#"
fail-on-warnings: true
enabled-rules:
  - MD001
  - MD013
MD013:
  line-length: 100
"#;

        let mut temp_file = NamedTempFile::with_suffix(".yaml").unwrap();
        temp_file.write_all(yaml_config.as_bytes()).unwrap();

        let config = Config::from_file(temp_file.path()).unwrap();
        assert!(config.fail_on_warnings);
        assert_eq!(config.core.enabled_rules, vec!["MD001", "MD013"]);

        let md013_config = config.get_rule_config("MD013").unwrap();
        assert_eq!(
            md013_config.get("line-length").unwrap().as_integer(),
            Some(100)
        );
    }

    #[test]
    fn test_config_from_yml_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let yaml_config = r#"
fail-on-warnings: true
enabled-rules:
  - MD001
  - MD013
"#;

        let mut temp_file = NamedTempFile::with_suffix(".yml").unwrap();
        temp_file.write_all(yaml_config.as_bytes()).unwrap();

        let config = Config::from_file(temp_file.path()).unwrap();
        assert!(config.fail_on_warnings);
        assert_eq!(config.core.enabled_rules, vec!["MD001", "MD013"]);
    }

    #[test]
    fn test_config_from_json_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let json_config = r#"{
    "fail-on-warnings": true,
    "enabled-rules": ["MD001", "MD013"],
    "MD013": {
        "line-length": 100
    }
}"#;

        let mut temp_file = NamedTempFile::with_suffix(".json").unwrap();
        temp_file.write_all(json_config.as_bytes()).unwrap();

        let config = Config::from_file(temp_file.path()).unwrap();
        assert!(config.fail_on_warnings);
        assert_eq!(config.core.enabled_rules, vec!["MD001", "MD013"]);

        let md013_config = config.get_rule_config("MD013").unwrap();
        assert_eq!(
            md013_config.get("line-length").unwrap().as_integer(),
            Some(100)
        );
    }

    #[test]
    fn test_config_from_file_no_extension() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let toml_config = r#"
fail-on-warnings = true
enabled-rules = ["MD001"]
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_config.as_bytes()).unwrap();

        // Should fall back to content detection and parse as TOML
        let config = Config::from_file(temp_file.path()).unwrap();
        assert!(config.fail_on_warnings);
        assert_eq!(config.core.enabled_rules, vec!["MD001"]);
    }

    #[test]
    fn test_config_from_file_mixed_content_detection() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let json_config = r#"{
    "fail-on-warnings": true,
    "enabled-rules": ["MD001"]
}"#;

        // Create file without .json extension
        let mut temp_file = NamedTempFile::with_suffix(".config").unwrap();
        temp_file.write_all(json_config.as_bytes()).unwrap();

        // Should detect JSON from content
        let config = Config::from_file(temp_file.path()).unwrap();
        assert!(config.fail_on_warnings);
        assert_eq!(config.core.enabled_rules, vec!["MD001"]);
    }

    #[test]
    fn test_from_str_auto_detection() {
        use std::str::FromStr;

        // Test JSON auto-detection
        let json_config = r#"{
            "fail-on-warnings": true,
            "enabled-rules": ["MD001"]
        }"#;
        let config = Config::from_str(json_config).unwrap();
        assert!(config.fail_on_warnings);
        assert_eq!(config.core.enabled_rules, vec!["MD001"]);

        // Test TOML auto-detection
        let toml_config = r#"
fail-on-warnings = true
enabled-rules = ["MD002"]
        "#;
        let config = Config::from_str(toml_config).unwrap();
        assert!(config.fail_on_warnings);
        assert_eq!(config.core.enabled_rules, vec!["MD002"]);

        // Test YAML auto-detection
        let yaml_config = r#"
fail-on-warnings: true
enabled-rules:
  - MD003
        "#;
        let config = Config::from_str(yaml_config).unwrap();
        assert!(config.fail_on_warnings);
        assert_eq!(config.core.enabled_rules, vec!["MD003"]);

        // Test parse() method (uses FromStr)
        let config: Config = json_config.parse().unwrap();
        assert!(config.fail_on_warnings);
        assert_eq!(config.core.enabled_rules, vec!["MD001"]);

        // Test invalid format
        let invalid_config = "this is not valid config";
        assert!(Config::from_str(invalid_config).is_err());
    }

    #[test]
    fn test_config_file_not_found() {
        let result = Config::from_file("nonexistent.toml");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to read config file")
        );
    }

    #[test]
    fn test_config_invalid_file_content() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let invalid_config = "this is not valid config content [}";

        let mut temp_file = NamedTempFile::with_suffix(".toml").unwrap();
        temp_file.write_all(invalid_config.as_bytes()).unwrap();

        let result = Config::from_file(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_markdownlint_compatible_mode() {
        let config = Config {
            core: mdbook_lint_core::Config {
                markdownlint_compatible: true,
                ..Default::default()
            },
            ..Default::default()
        };

        // MD044 should be disabled in markdownlint compatibility mode
        assert!(!config.should_run_rule("MD044"));

        // MD034 should still be enabled (it's enabled by default in markdownlint too)
        assert!(config.should_run_rule("MD034"));

        // Other rules should still be enabled
        assert!(config.should_run_rule("MD001"));
        assert!(config.should_run_rule("MD013"));
        assert!(config.should_run_rule("MD022"));
    }

    #[test]
    fn test_markdownlint_compatible_mode_with_explicit_enable() {
        let config = Config {
            core: mdbook_lint_core::Config {
                markdownlint_compatible: true,
                enabled_rules: vec!["MD034".to_string(), "MD044".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };

        // Explicit enabled rules should override compatibility mode
        assert!(config.should_run_rule("MD044"));

        // But other rules should be disabled (since enabled_rules is not empty)
        assert!(!config.should_run_rule("MD001"));
        assert!(!config.should_run_rule("MD013"));
    }

    #[test]
    fn test_markdownlint_compatible_mode_config_parsing() {
        let toml_config = r#"
markdownlint-compatible = true
fail-on-warnings = true
"#;

        let config = Config::from_toml_str(toml_config).unwrap();
        assert!(config.core.markdownlint_compatible);
        assert!(config.fail_on_warnings);

        // Check that compatibility mode works
        assert!(!config.should_run_rule("MD044"));
        assert!(config.should_run_rule("MD034")); // MD034 is enabled in markdownlint too
        assert!(config.should_run_rule("MD013"));
    }
}
