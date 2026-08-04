//! MDBOOK004: No duplicate chapter titles across the book
//!
//! This rule validates that chapter titles are unique across the entire book.

use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};
use std::collections::HashMap;

/// MDBOOK004: No duplicate chapter titles across the book
///
/// This rule checks that each chapter has a unique title within the book.
/// Note: This rule is designed to work with individual chapters and will
/// need cross-file coordination to detect duplicates across the entire book.
///
/// Configuration:
/// - `case_sensitive` (default true): whether titles differing only in case count
///   as duplicates.
/// - `ignore_prefixes` (default empty): leading prefixes stripped from a title
///   before comparison, so "Chapter Setup" and "Setup" compare equal.
pub struct MDBOOK004 {
    /// Whether title comparison is case-sensitive
    case_sensitive: bool,
    /// Prefixes stripped from a title before comparison
    ignore_prefixes: Vec<String>,
}

impl Default for MDBOOK004 {
    fn default() -> Self {
        Self {
            case_sensitive: true,
            ignore_prefixes: Vec::new(),
        }
    }
}

impl MDBOOK004 {
    /// Create an instance from rule configuration.
    ///
    /// Recognized keys (both `snake_case` and `kebab-case` accepted):
    /// - `case_sensitive`: compare titles case-sensitively (default true).
    /// - `ignore_prefixes`: array of prefixes to strip before comparing (default none).
    pub fn from_config(config: &toml::Value) -> Self {
        let get = |snake: &str, kebab: &str| config.get(snake).or_else(|| config.get(kebab));
        let defaults = Self::default();

        Self {
            case_sensitive: get("case_sensitive", "case-sensitive")
                .and_then(|v| v.as_bool())
                .unwrap_or(defaults.case_sensitive),
            ignore_prefixes: get("ignore_prefixes", "ignore-prefixes")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or(defaults.ignore_prefixes),
        }
    }

    /// Reduce a title to the key used for duplicate comparison
    ///
    /// The first matching prefix is stripped (with any whitespace that follows it),
    /// then the result is case-folded when `case_sensitive` is off.
    fn comparison_key(&self, title: &str) -> String {
        let mut key = title;

        for prefix in &self.ignore_prefixes {
            if let Some(rest) = strip_prefix(key, prefix, self.case_sensitive) {
                key = rest.trim_start();
                break;
            }
        }

        if self.case_sensitive {
            key.to_string()
        } else {
            key.to_lowercase()
        }
    }
}

/// Strip `prefix` from the front of `key`, comparing case-insensitively when asked
///
/// Works on char boundaries so a prefix whose case mapping changes byte length
/// cannot produce an invalid slice.
fn strip_prefix<'a>(key: &'a str, prefix: &str, case_sensitive: bool) -> Option<&'a str> {
    if case_sensitive {
        return key.strip_prefix(prefix);
    }

    let mut key_chars = key.char_indices();
    for prefix_char in prefix.chars() {
        match key_chars.next() {
            Some((_, key_char)) if key_char.to_lowercase().eq(prefix_char.to_lowercase()) => {}
            _ => return None,
        }
    }

    Some(key_chars.next().map(|(i, _)| &key[i..]).unwrap_or(""))
}

impl AstRule for MDBOOK004 {
    fn id(&self) -> &'static str {
        "MDBOOK004"
    }

    fn name(&self) -> &'static str {
        "no-duplicate-chapter-titles"
    }

    fn description(&self) -> &'static str {
        "Chapter titles should be unique across the book"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::MdBook).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_ast<'a>(
        &self,
        document: &Document,
        ast: &'a AstNode<'a>,
    ) -> mdbook_lint_core::error::Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let mut title_positions = HashMap::new();

        // Extract all heading titles and their positions
        for node in ast.descendants() {
            if let NodeValue::Heading(_heading) = &node.data.borrow().value
                && let Some((line, column)) = document.node_position(node)
            {
                let title = document.node_text(node).trim().to_string();

                if !title.is_empty() {
                    let key = self.comparison_key(&title);

                    // Check for duplicates within the same document
                    if let Some((prev_line, _)) = title_positions.get(&key) {
                        violations.push(self.create_violation(
                            format!(
                                "Duplicate chapter title '{title}' found (also at line {prev_line})"
                            ),
                            line,
                            column,
                            Severity::Error,
                        ));
                    } else {
                        title_positions.insert(key, (line, column));
                    }
                }
            }
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::test_helpers::{
        MarkdownBuilder, assert_no_violations, assert_violation_at_line,
        assert_violation_contains_message, assert_violation_count,
    };

    #[test]
    fn test_mdbook004_no_duplicates() {
        let content = MarkdownBuilder::new()
            .heading(1, "Introduction")
            .blank_line()
            .paragraph("This is the introduction.")
            .blank_line()
            .heading(2, "Getting Started")
            .blank_line()
            .paragraph("How to get started.")
            .blank_line()
            .heading(2, "Advanced Topics")
            .blank_line()
            .paragraph("Advanced material.")
            .build();

        assert_no_violations(MDBOOK004::default(), &content);
    }

    #[test]
    fn test_mdbook004_within_document_duplicates() {
        let content = MarkdownBuilder::new()
            .heading(1, "Introduction")
            .blank_line()
            .paragraph("First introduction.")
            .blank_line()
            .heading(2, "Getting Started")
            .blank_line()
            .paragraph("How to get started.")
            .blank_line()
            .heading(1, "Introduction")
            .blank_line()
            .paragraph("Second introduction - duplicate!")
            .build();

        let violations = assert_violation_count(MDBOOK004::default(), &content, 1);
        assert_violation_contains_message(&violations, "Duplicate chapter title 'Introduction'");
        assert_violation_contains_message(&violations, "also at line 1");
        assert_violation_at_line(&violations, 9);
    }

    #[test]
    fn test_mdbook004_case_sensitive() {
        let content = MarkdownBuilder::new()
            .heading(1, "Introduction")
            .blank_line()
            .heading(1, "introduction")
            .blank_line()
            .heading(1, "INTRODUCTION")
            .build();

        // These should be treated as different titles (case-sensitive)
        assert_no_violations(MDBOOK004::default(), &content);
    }

    #[test]
    fn test_mdbook004_defaults_match_unconfigured_behavior() {
        let rule = MDBOOK004::from_config(&toml::Value::Table(Default::default()));
        assert!(rule.case_sensitive);
        assert!(rule.ignore_prefixes.is_empty());
    }

    #[test]
    fn test_mdbook004_case_insensitive_when_configured() {
        let content = MarkdownBuilder::new()
            .heading(1, "Introduction")
            .blank_line()
            .heading(1, "introduction")
            .build();

        let rule = MDBOOK004::from_config(&toml::toml! { case_sensitive = false });
        let violations = assert_violation_count(rule, &content, 1);
        assert_violation_contains_message(&violations, "Duplicate chapter title 'introduction'");
    }

    #[test]
    fn test_mdbook004_ignore_prefixes() {
        let content = MarkdownBuilder::new()
            .heading(1, "Chapter Setup")
            .blank_line()
            .heading(1, "Setup")
            .build();

        // Without the prefix configured these are distinct titles
        assert_no_violations(MDBOOK004::default(), &content);

        let rule = MDBOOK004::from_config(&toml::toml! { ignore_prefixes = ["Chapter", "Part"] });
        let violations = assert_violation_count(rule, &content, 1);
        assert_violation_contains_message(&violations, "Duplicate chapter title 'Setup'");
    }

    #[test]
    fn test_mdbook004_ignore_prefixes_respect_case_sensitivity() {
        let case_sensitive = MDBOOK004::from_config(&toml::toml! { ignore_prefixes = ["Chapter"] });
        // A prefix that differs in case is not stripped when comparison is case-sensitive
        assert_eq!(
            case_sensitive.comparison_key("chapter Setup"),
            "chapter Setup"
        );
        assert_eq!(case_sensitive.comparison_key("Chapter Setup"), "Setup");

        let case_insensitive = MDBOOK004::from_config(&toml::toml! {
            ignore_prefixes = ["Chapter"]
            case_sensitive = false
        });
        assert_eq!(case_insensitive.comparison_key("chapter Setup"), "setup");
    }

    #[test]
    fn test_mdbook004_kebab_case_config_keys() {
        let rule = MDBOOK004::from_config(&toml::toml! {
            "case-sensitive" = false
            "ignore-prefixes" = ["Ch."]
        });
        assert!(!rule.case_sensitive);
        assert_eq!(rule.ignore_prefixes, vec!["Ch.".to_string()]);
    }

    #[test]
    fn test_mdbook004_different_heading_levels() {
        let content = MarkdownBuilder::new()
            .heading(1, "Setup")
            .blank_line()
            .heading(2, "Setup")
            .blank_line()
            .heading(3, "Setup")
            .build();

        // Even different heading levels should be considered duplicates
        let violations = assert_violation_count(MDBOOK004::default(), &content, 2);
        assert_violation_contains_message(&violations, "Duplicate chapter title 'Setup'");
    }

    #[test]
    fn test_mdbook004_empty_headings_ignored() {
        let content = MarkdownBuilder::new()
            .line("# ")
            .blank_line()
            .line("## ")
            .blank_line()
            .heading(1, "Real Title")
            .build();

        // Empty headings should be ignored
        assert_no_violations(MDBOOK004::default(), &content);
    }

    #[test]
    fn test_mdbook004_heading_text_from_links_counts_for_duplicates() {
        let content = MarkdownBuilder::new()
            .line("# Intro [link](https://example.com)")
            .blank_line()
            .line("# Intro link")
            .build();

        let violations = assert_violation_count(MDBOOK004::default(), &content, 1);
        assert_violation_contains_message(&violations, "Duplicate chapter title 'Intro link'");
        assert_violation_contains_message(&violations, "also at line 1");
        assert_violation_at_line(&violations, 3);
    }

    #[test]
    fn test_mdbook004_ignores_headings_inside_code_fences() {
        let content = MarkdownBuilder::new()
            .line("```")
            .line("# Not a real heading")
            .line("```")
            .blank_line()
            .line("# Not a real heading")
            .build();

        // Only the last line is a real heading node.
        assert_no_violations(MDBOOK004::default(), &content);
    }

    #[test]
    fn test_mdbook004_whitespace_handling() {
        let content = MarkdownBuilder::new()
            .line("# Introduction ")
            .blank_line()
            .line("#  Introduction")
            .blank_line()
            .line("# Introduction  ")
            .blank_line()
            .line("   # Introduction  ") // 4 Spaces - treated as heading
            .blank_line()
            .line("    # Introduction  ") // 5 Spaces - treated as indented code block, thus, not a duplicate
            .build();

        // Whitespace should be trimmed, so these are duplicates
        let violations = assert_violation_count(MDBOOK004::default(), &content, 3);
        assert_violation_contains_message(&violations, "Duplicate chapter title 'Introduction'");
    }

    #[test]
    fn test_mdbook004_unicode_normalization() {
        let content: String = MarkdownBuilder::new()
            .line("# Introduction")
            .blank_line()
            .line("# Introductio\\u006E")
            .build();

        assert_violation_count(MDBOOK004::default(), &content, 0);
    }

    #[test]
    fn test_mdbook004_rule_metadata() {
        use mdbook_lint_core::rule::AstRule;
        let rule = MDBOOK004::default();
        assert_eq!(AstRule::id(&rule), "MDBOOK004");
        assert_eq!(AstRule::name(&rule), "no-duplicate-chapter-titles");
        assert!(AstRule::description(&rule).contains("unique"));
    }
}
