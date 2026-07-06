//! Tests for ignore-related configuration wired through the engine.
//!
//! Covers two regressions:
//! - #411: the global `ignore-paths` / `ignore_paths` option now parses into
//!   `Config`.
//! - #412: `MdBookRuleProvider` now threads rule configuration, so MDBOOK005
//!   honors `ignore_patterns`.

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use mdbook_lint_core::{Document, PluginRegistry};
    use mdbook_lint_rulesets::MdBookRuleProvider;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_ignore_paths_config_parses_both_spellings() {
        // Underscore spelling (as used in the example config / issue #411)
        let config =
            Config::from_toml_str("ignore_paths = [\"vendor/\", \"*.backup.md\"]").unwrap();
        assert_eq!(config.core.ignore_paths, vec!["vendor/", "*.backup.md"]);

        // Kebab spelling (matches the rest of the config keys)
        let config = Config::from_toml_str("ignore-paths = [\"drafts/\"]").unwrap();
        assert_eq!(config.core.ignore_paths, vec!["drafts/"]);

        // Absent: defaults to empty
        let config = Config::from_toml_str("").unwrap();
        assert!(config.core.ignore_paths.is_empty());
    }

    #[test]
    fn test_mdbook005_ignore_patterns_threads_through_engine() {
        // Reproduces issue #412 end-to-end through the provider/engine path,
        // which is where the config was previously dropped.
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(
            root.join("SUMMARY.md"),
            "# Summary\n\n- [Chapter 1](chapter1.md)\n",
        )
        .unwrap();
        fs::write(root.join("chapter1.md"), "# Chapter 1").unwrap();
        fs::write(root.join("not-found.md"), "# Not Found").unwrap();

        let summary = Document::new(
            fs::read_to_string(root.join("SUMMARY.md")).unwrap(),
            root.join("SUMMARY.md"),
        )
        .unwrap();

        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(MdBookRuleProvider))
            .unwrap();

        // Without config: not-found.md is reported as an orphan.
        let config = Config::from_toml_str("enabled-rules = [\"MDBOOK005\"]").unwrap();
        let engine = registry
            .create_engine_with_config(Some(&config.core))
            .unwrap();
        let violations = engine
            .lint_document_with_config(&summary, &config.core)
            .unwrap();
        let orphans: Vec<_> = violations
            .iter()
            .filter(|v| v.rule_id == "MDBOOK005")
            .collect();
        assert_eq!(orphans.len(), 1, "orphan should be reported without config");

        // With ignore_patterns: the orphan is suppressed.
        let config = Config::from_toml_str(
            "enabled-rules = [\"MDBOOK005\"]\n[MDBOOK005]\nignore_patterns = [\"not-found.md\"]\n",
        )
        .unwrap();
        let engine = registry
            .create_engine_with_config(Some(&config.core))
            .unwrap();
        let violations = engine
            .lint_document_with_config(&summary, &config.core)
            .unwrap();
        let orphans: Vec<_> = violations
            .iter()
            .filter(|v| v.rule_id == "MDBOOK005")
            .collect();
        assert_eq!(
            orphans.len(),
            0,
            "ignore_patterns should suppress the orphan through the engine"
        );
    }
}
