//! CONTENT007: Consistent terminology
//!
//! Detects inconsistent use of terms within a document. For example, using both
//! "config" and "configuration", or "setup" and "set up" inconsistently.

use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use std::collections::HashMap;

/// Common term variants that should be used consistently
/// Each group contains terms that are variants of each other
const DEFAULT_TERM_GROUPS: &[&[&str]] = &[
    &["config", "configuration"],
    &["setup", "set up", "set-up"],
    &["login", "log in", "log-in"],
    &["signup", "sign up", "sign-up"],
    &["email", "e-mail"],
    &["filename", "file name", "file-name"],
    &["username", "user name", "user-name"],
    &["database", "data base"],
    &["website", "web site", "web-site"],
    &["checkbox", "check box", "check-box"],
    &["dropdown", "drop down", "drop-down"],
    &["frontend", "front end", "front-end"],
    &["backend", "back end", "back-end"],
    &["ok", "okay"],
    &["grey", "gray"],
    &["cancelled", "canceled"],
    &["colour", "color"],
    &["behaviour", "behavior"],
    &["licence", "license"],
    &["synchronise", "synchronize"],
    &["analyse", "analyze"],
];

/// CONTENT007: Detects inconsistent terminology usage
///
/// This rule identifies when different variants of the same term are used
/// within a document, which can confuse readers and appear unprofessional.
#[derive(Clone)]
pub struct CONTENT007 {
    /// Term groups to check for consistency
    term_groups: Vec<Vec<String>>,
    /// Minimum occurrences to trigger a warning
    min_occurrences: usize,
}

impl Default for CONTENT007 {
    fn default() -> Self {
        let term_groups = DEFAULT_TERM_GROUPS
            .iter()
            .map(|group| group.iter().map(|s| (*s).to_string()).collect())
            .collect();

        Self {
            term_groups,
            min_occurrences: 1,
        }
    }
}

impl CONTENT007 {
    /// Create with custom term groups
    #[allow(dead_code)]
    pub fn with_term_groups(term_groups: Vec<Vec<String>>) -> Self {
        Self {
            term_groups,
            min_occurrences: 1,
        }
    }

    /// Find all occurrences of terms in the document
    fn find_term_occurrences(
        &self,
        document: &Document,
    ) -> HashMap<usize, Vec<(String, usize, usize)>> {
        // Map from term_group_index -> list of (term, line, column)
        let mut occurrences: HashMap<usize, Vec<(String, usize, usize)>> = HashMap::new();
        let mut in_code_block = false;

        for (line_idx, line) in document.lines.iter().enumerate() {
            let trimmed = line.trim();

            // Track code blocks - skip code content
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }

            if in_code_block {
                continue;
            }

            // Skip inline code spans for matching
            let line_lower = line.to_lowercase();

            for (group_idx, group) in self.term_groups.iter().enumerate() {
                for term in group {
                    let term_lower = term.to_lowercase();

                    // Find all occurrences of this term
                    let mut search_start = 0;
                    while let Some(pos) = line_lower[search_start..].find(&term_lower) {
                        let actual_pos = search_start + pos;

                        // Check word boundaries
                        let before_ok = actual_pos == 0
                            || !line_lower[..actual_pos]
                                .chars()
                                .last()
                                .map(|c| c.is_alphanumeric())
                                .unwrap_or(false);

                        let after_pos = actual_pos + term_lower.len();
                        let after_ok = after_pos >= line_lower.len()
                            || !line_lower[after_pos..]
                                .chars()
                                .next()
                                .map(|c| c.is_alphanumeric())
                                .unwrap_or(false);

                        if before_ok && after_ok {
                            // Check if this is inside inline code
                            let before_text = &line[..actual_pos];
                            let backtick_count = before_text.matches('`').count();
                            if backtick_count % 2 == 0 {
                                // Not inside inline code
                                occurrences.entry(group_idx).or_default().push((
                                    term.clone(),
                                    line_idx + 1,
                                    actual_pos + 1,
                                ));
                            }
                        }

                        search_start = actual_pos + 1;
                    }
                }
            }
        }

        occurrences
    }
}

impl Rule for CONTENT007 {
    fn id(&self) -> &'static str {
        "CONTENT007"
    }

    fn name(&self) -> &'static str {
        "consistent-terminology"
    }

    fn description(&self) -> &'static str {
        "Terms should be used consistently throughout a document"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Content).introduced_in("mdbook-lint v0.14.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> mdbook_lint_core::error::Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let occurrences = self.find_term_occurrences(document);

        for (group_idx, term_occurrences) in occurrences {
            // Count how many of each term variant is used
            let mut term_counts: HashMap<&str, Vec<(usize, usize)>> = HashMap::new();
            for (term, line, col) in &term_occurrences {
                term_counts
                    .entry(term.as_str())
                    .or_default()
                    .push((*line, *col));
            }

            // Only flag if multiple variants are used
            if term_counts.len() > 1 {
                // Find the most common variant
                let most_common = term_counts
                    .iter()
                    .max_by_key(|(_, locs)| locs.len())
                    .map(|(term, _)| *term)
                    .unwrap_or("");

                // Report violations for less common variants
                for (term, locations) in &term_counts {
                    if *term != most_common && locations.len() >= self.min_occurrences {
                        let group = &self.term_groups[group_idx];
                        for (line, col) in locations {
                            violations.push(self.create_violation(
                                format!(
                                    "Inconsistent terminology: '{}' used here, but '{}' is used more frequently. \
                                     Consider using '{}' consistently (variants: {})",
                                    term,
                                    most_common,
                                    most_common,
                                    group.join(", ")
                                ),
                                *line,
                                *col,
                                Severity::Info,
                            ));
                        }
                    }
                }
            }
        }

        // Sort violations by line number
        violations.sort_by_key(|v| (v.line, v.column));

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_document(content: &str) -> Document {
        Document::new(content.to_string(), PathBuf::from("test.md")).unwrap()
    }

    #[test]
    fn test_consistent_config() {
        let content = "# Settings

The config file is located in the config directory.
Edit the config to change settings.";
        let doc = create_test_document(content);
        let rule = CONTENT007::default();
        let violations = rule.check(&doc).unwrap();
        // All uses are "config" - consistent
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_inconsistent_config() {
        let content = "# Settings

The config file is in the config directory.
Edit the configuration to change settings.";
        let doc = create_test_document(content);
        let rule = CONTENT007::default();
        let violations = rule.check(&doc).unwrap();
        // "config" appears twice, "configuration" once - should flag "configuration"
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("configuration"));
        assert!(violations[0].message.contains("config"));
    }

    #[test]
    fn test_inconsistent_setup() {
        let content = "# Setup Guide

First, set up your environment.
The setup process takes a few minutes.
After setup, run the application.";
        let doc = create_test_document(content);
        let rule = CONTENT007::default();
        let violations = rule.check(&doc).unwrap();
        // "setup" appears 3 times (heading + 2 body), "set up" once
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("set up"));
    }

    #[test]
    fn test_code_blocks_ignored() {
        let content = "# Configuration

Use configuration files.

```
config = load_config()
```

The configuration is loaded.";
        let doc = create_test_document(content);
        let rule = CONTENT007::default();
        let violations = rule.check(&doc).unwrap();
        // "config" in code block should be ignored
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_inline_code_ignored() {
        let content = "# Configuration

The `config` variable holds the configuration.
Update the configuration as needed.";
        let doc = create_test_document(content);
        let rule = CONTENT007::default();
        let violations = rule.check(&doc).unwrap();
        // "config" in inline code should be ignored
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_word_boundaries() {
        let content = "# Reconfiguration

This is about reconfiguration.
The configuration must be valid.";
        let doc = create_test_document(content);
        let rule = CONTENT007::default();
        let violations = rule.check(&doc).unwrap();
        // "reconfiguration" should not match "config"
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_case_insensitive() {
        let content = "# Setup

Setup is important.
The SETUP guide is here.
Let's set up the system.";
        let doc = create_test_document(content);
        let rule = CONTENT007::default();
        let violations = rule.check(&doc).unwrap();
        // "Setup", "SETUP" count as same; "set up" is different
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_british_american_spelling() {
        let content = "# Color Settings

The color scheme is customizable.
You can change the colour of any element.
The colour picker is easy to use.";
        let doc = create_test_document(content);
        let rule = CONTENT007::default();
        let violations = rule.check(&doc).unwrap();
        // "colour" appears twice, "color" twice - should flag one
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_email_variations() {
        let content = "# Contact

Send an email to support.
Your e-mail will be answered within 24 hours.";
        let doc = create_test_document(content);
        let rule = CONTENT007::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_no_term_groups_match() {
        let content = "# Introduction

This is a simple document with no conflicting terms.
It just has regular content.";
        let doc = create_test_document(content);
        let rule = CONTENT007::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_custom_term_groups() {
        let content = "# API Reference

Use the API to fetch data.
The interface provides methods.";
        let term_groups = vec![vec!["api".to_string(), "interface".to_string()]];
        let rule = CONTENT007::with_term_groups(term_groups);
        let doc = create_test_document(content);
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_frontend_backend() {
        let content = "# Architecture

The frontend handles user interaction.
The front-end is built with React.
The backend processes requests.";
        let doc = create_test_document(content);
        let rule = CONTENT007::default();
        let violations = rule.check(&doc).unwrap();
        // "frontend" vs "front-end"
        assert_eq!(violations.len(), 1);
    }
}
