//! MD002: First heading should be a top-level heading
//!
//! This rule checks that the first heading in a document is a top-level heading (h1).
//! Note: This rule is deprecated in the original markdownlint but included for completeness.

use comrak::nodes::AstNode;
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Fix, Position, Severity, Violation},
};

/// Rule to check that the first heading is a top-level heading
pub struct MD002 {
    /// The level that the first heading should be (default: 1)
    level: u32,
}

impl MD002 {
    /// Create a new MD002 rule with default settings (level 1)
    pub fn new() -> Self {
        Self { level: 1 }
    }

    /// Create a new MD002 rule with custom level
    #[allow(dead_code)]
    pub fn with_level(level: u32) -> Self {
        Self { level }
    }

    /// Create MD002 from configuration
    pub fn from_config(config: &toml::Value) -> Self {
        let mut rule = Self::new();

        if let Some(level) = config.get("level").and_then(|v| v.as_integer()) {
            rule.level = level as u32;
        }

        rule
    }
}

impl Default for MD002 {
    fn default() -> Self {
        Self::new()
    }
}

impl AstRule for MD002 {
    fn id(&self) -> &'static str {
        "MD002"
    }

    fn name(&self) -> &'static str {
        "first-heading-h1"
    }

    fn description(&self) -> &'static str {
        "First heading should be a top-level heading"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::deprecated(
            RuleCategory::Structure,
            "Superseded by MD041 which offers improved implementation",
            Some("MD041"),
        )
        .introduced_in("markdownlint v0.1.0")
    }

    fn can_fix(&self) -> bool {
        true
    }

    fn check_ast<'a>(&self, document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let headings = document.headings(ast);

        // Find the first heading
        if let Some(first_heading) = headings.first()
            && let Some(heading_level) = Document::heading_level(first_heading)
            && heading_level != self.level
            && let Some((line, column)) = document.node_position(first_heading)
        {
            let heading_text = document.node_text(first_heading);
            let message = format!(
                "First heading should be level {} but got level {}{}",
                self.level,
                heading_level,
                if heading_text.is_empty() {
                    String::new()
                } else {
                    format!(": {}", heading_text.trim())
                }
            );

            // Create fix by changing the heading level
            let line_content = &document.lines[line - 1];

            let fixed_line = if line_content.trim_start().starts_with('#') {
                // ATX heading - adjust the number of hashes
                let trimmed = line_content.trim_start();
                let content_start = trimmed.find(|c: char| c != '#').unwrap_or(trimmed.len());
                let heading_content = if content_start < trimmed.len() {
                    &trimmed[content_start..]
                } else {
                    ""
                };
                format!("{}{}\n", "#".repeat(self.level as usize), heading_content)
            } else {
                // For Setext headings, convert to ATX with correct level
                format!(
                    "{} {}\n",
                    "#".repeat(self.level as usize),
                    heading_text.trim()
                )
            };

            let fix = Fix {
                description: format!(
                    "Change first heading level from {} to {}",
                    heading_level, self.level
                ),
                replacement: Some(fixed_line),
                start: Position { line, column: 1 },
                end: Position {
                    line,
                    column: line_content.len() + 1,
                },
            };

            violations.push(self.create_violation_with_fix(
                message,
                line,
                column,
                Severity::Warning,
                fix,
            ));
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::rule::Rule;
    use std::path::PathBuf;

    fn create_test_document(content: &str) -> Document {
        Document::new(content.to_string(), PathBuf::from("test.md")).unwrap()
    }

    #[test]
    fn test_md002_valid_first_heading() {
        let content = "# First heading\n## Second heading";
        let document = create_test_document(content);
        let rule = MD002::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md002_invalid_first_heading() {
        let content = "## This should be h1\n### This is h3";
        let document = create_test_document(content);
        let rule = MD002::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD002");
        assert_eq!(violations[0].line, 1);
        assert!(violations[0].message.contains("should be level 1"));
        assert!(violations[0].message.contains("got level 2"));
    }

    #[test]
    fn test_md002_custom_level() {
        let content = "## Starting with h2\n### Then h3";
        let document = create_test_document(content);
        let rule = MD002::with_level(2);
        let violations = rule.check(&document).unwrap();

        // Should be valid since we configured level 2 as the expected first level
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md002_custom_level_violation() {
        let content = "### Starting with h3\n#### Then h4";
        let document = create_test_document(content);
        let rule = MD002::with_level(2);
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("should be level 2"));
        assert!(violations[0].message.contains("got level 3"));
    }

    #[test]
    fn test_md002_no_headings() {
        let content = "Just some text without headings.";
        let document = create_test_document(content);
        let rule = MD002::new();
        let violations = rule.check(&document).unwrap();

        // No headings means no violations
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md002_setext_heading() {
        let content = "First Heading\n=============\n\nSecond Heading\n--------------";
        let document = create_test_document(content);
        let rule = MD002::new();
        let violations = rule.check(&document).unwrap();

        // Setext heading (=====) is level 1, so should be valid
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md002_setext_heading_violation() {
        let content = "First Heading\n--------------\n\nAnother Heading\n===============";
        let document = create_test_document(content);
        let rule = MD002::new();
        let violations = rule.check(&document).unwrap();

        // Setext heading (-----) is level 2, should trigger violation
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("should be level 1"));
        assert!(violations[0].message.contains("got level 2"));
    }

    #[test]
    fn test_md002_heading_with_text() {
        let content = "### My Third Level Heading\n#### Subheading";
        let document = create_test_document(content);
        let rule = MD002::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("My Third Level Heading"));
    }

    #[test]
    fn test_md002_mixed_content_before_heading() {
        let content = "Some intro text\n\n## First heading\n### Second heading";
        let document = create_test_document(content);
        let rule = MD002::new();
        let violations = rule.check(&document).unwrap();

        // The first *heading* should be h1, regardless of other content
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("should be level 1"));
        assert!(violations[0].message.contains("got level 2"));
    }

    #[test]
    fn test_md002_fix_atx_heading() {
        let content = "## This should be h1\n### This is h3";
        let document = create_test_document(content);
        let rule = MD002::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Change first heading level from 2 to 1");
        assert_eq!(fix.replacement, Some("# This should be h1\n".to_string()));
    }

    #[test]
    fn test_md002_fix_setext_heading() {
        let content = "First Heading\n--------------\n\nSome text";
        let document = create_test_document(content);
        let rule = MD002::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Change first heading level from 2 to 1");
        assert_eq!(fix.replacement, Some("# First Heading\n".to_string()));
    }

    #[test]
    fn test_md002_can_fix() {
        let rule = MD002::new();
        assert!(mdbook_lint_core::AstRule::can_fix(&rule));
    }

    // Edge case tests for issue #301

    #[test]
    fn test_md002_empty_file() {
        let content = "";
        let document = create_test_document(content);
        let rule = MD002::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md002_whitespace_only_file() {
        let content = "   \n\n\t\t\n   \n";
        let document = create_test_document(content);
        let rule = MD002::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md002_unicode_first_heading_valid() {
        let content = "# 日本語タイトル\n## サブセクション";
        let document = create_test_document(content);
        let rule = MD002::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md002_unicode_first_heading_invalid() {
        let content = "## Ελληνικά κεφαλίδα\n### 中文标题";
        let document = create_test_document(content);
        let rule = MD002::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Ελληνικά κεφαλίδα"));
    }

    #[test]
    fn test_md002_emoji_first_heading() {
        let content = "## 🚀 Getting Started\n### Introduction";
        let document = create_test_document(content);
        let rule = MD002::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("should be level 1"));
    }

    #[test]
    fn test_md002_very_long_heading() {
        let long_text = "A".repeat(1000);
        let content = format!("## {}\n### Subheading", long_text);
        let document = create_test_document(&content);
        let rule = MD002::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 1);
    }

    #[test]
    fn test_md002_heading_with_special_characters() {
        let content = "## Title with `code` and **bold**\n### Next section";
        let document = create_test_document(content);
        let rule = MD002::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_md002_heading_in_blockquote() {
        let content = "> ## Quoted heading\n\n# Real first heading";
        let document = create_test_document(content);
        let rule = MD002::new();
        let violations = rule.check(&document).unwrap();

        // The first heading found is h2 in blockquote
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_md002_heading_with_trailing_hashes() {
        let content = "## Title ##\n### Subtitle ###";
        let document = create_test_document(content);
        let rule = MD002::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_md002_all_levels_starting_h6() {
        let content = "###### Starting at h6";
        let document = create_test_document(content);
        let rule = MD002::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("got level 6"));
    }

    #[test]
    fn test_md002_fix_preserves_unicode() {
        let content = "## 中文标题\n### 子标题";
        let document = create_test_document(content);
        let rule = MD002::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert!(fix.replacement.as_ref().unwrap().contains("中文标题"));
        assert!(fix.replacement.as_ref().unwrap().starts_with("# "));
    }

    #[test]
    fn test_md002_custom_level_h3() {
        let content = "### Starting at configured level\n#### Subheading";
        let document = create_test_document(content);
        let rule = MD002::with_level(3);
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md002_frontmatter_like_content() {
        let content = "---\ntitle: Test\n---\n\n## First heading";
        let document = create_test_document(content);
        let rule = MD002::new();
        let violations = rule.check(&document).unwrap();

        // YAML frontmatter is not a heading, so ## is the first heading
        assert_eq!(violations.len(), 1);
    }
}
