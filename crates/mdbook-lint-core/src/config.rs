//! Core configuration types for mdbook-lint-core
//!
//! This module contains the minimal configuration types needed by the core
//! linting engine. The full configuration is handled by the CLI crate.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core configuration for the linting engine
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

    /// Rule-specific configuration
    #[serde(flatten)]
    pub rule_configs: HashMap<String, toml::Value>,
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
}
