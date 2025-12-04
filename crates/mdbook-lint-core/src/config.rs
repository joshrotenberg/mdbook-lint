//! Core configuration types for mdbook-lint-core
//!
//! This module contains the minimal configuration types needed by the core
//! linting engine. The full configuration is handled by the CLI crate.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core configuration for the linting engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// List of enabled rule categories
    #[serde(rename = "enabled-categories", default)]
    pub enabled_categories: Vec<String>,

    /// List of disabled rule categories
    #[serde(rename = "disabled-categories", default)]
    pub disabled_categories: Vec<String>,

    /// List of explicitly enabled rules
    #[serde(rename = "enabled-rules", default)]
    pub enabled_rules: Vec<String>,

    /// List of explicitly disabled rules
    #[serde(rename = "disabled-rules", default)]
    pub disabled_rules: Vec<String>,

    /// How to handle deprecated rule warnings
    #[serde(rename = "deprecated-warning", default)]
    pub deprecated_warning: DeprecatedWarningLevel,

    /// Enable markdownlint compatibility mode (disables rules that are disabled by default in markdownlint)
    #[serde(rename = "markdownlint-compatible", default)]
    pub markdownlint_compatible: bool,

    /// Global auto-fix setting (default: true when --fix is used)
    /// Can be overridden per-rule in rule-specific configuration
    #[serde(rename = "auto-fix", default = "default_auto_fix")]
    pub auto_fix: bool,

    /// Rule-specific configuration
    #[serde(flatten)]
    pub rule_configs: HashMap<String, toml::Value>,
}

fn default_auto_fix() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled_categories: Vec::new(),
            disabled_categories: Vec::new(),
            enabled_rules: Vec::new(),
            disabled_rules: Vec::new(),
            deprecated_warning: DeprecatedWarningLevel::default(),
            markdownlint_compatible: false,
            auto_fix: true, // Default to true - fixes are applied when --fix is used
            rule_configs: HashMap::new(),
        }
    }
}

/// How to handle deprecated rule warnings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DeprecatedWarningLevel {
    /// Show warning messages for deprecated rules (default)
    #[default]
    Warn,
    /// Show info messages for deprecated rules
    Info,
    /// Don't show any messages for deprecated rules
    Silent,
}

impl Config {
    /// Check if a rule should be run based on configuration
    pub fn should_run_rule(
        &self,
        rule_id: &str,
        rule_category: &str,
        rule_enabled_by_default: bool,
    ) -> bool {
        // Explicit rule configuration takes precedence
        if self.enabled_rules.contains(&rule_id.to_string()) {
            return true;
        }
        if self.disabled_rules.contains(&rule_id.to_string()) {
            return false;
        }

        // Category configuration takes precedence over default
        if self.enabled_categories.contains(&rule_category.to_string()) {
            return true;
        }
        if self
            .disabled_categories
            .contains(&rule_category.to_string())
        {
            return false;
        }

        // Use default enabled state
        rule_enabled_by_default
    }

    /// Check if auto-fix is enabled for a specific rule
    ///
    /// Returns true if:
    /// 1. The rule has `auto-fix = true` in its config, OR
    /// 2. The rule has no `auto-fix` setting and global `auto-fix` is true (default)
    ///
    /// Returns false if:
    /// 1. The rule has `auto-fix = false` in its config, OR
    /// 2. The rule has no `auto-fix` setting and global `auto-fix` is false
    pub fn should_auto_fix_rule(&self, rule_id: &str) -> bool {
        // Check for rule-specific auto-fix setting
        if let Some(rule_config) = self.rule_configs.get(rule_id)
            && let Some(auto_fix) = rule_config.get("auto-fix").and_then(|v| v.as_bool())
        {
            return auto_fix;
        }

        // Fall back to global setting
        self.auto_fix
    }
}
