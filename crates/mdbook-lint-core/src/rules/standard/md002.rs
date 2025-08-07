//! MD002: First heading should be a top-level heading
//!
//! This rule checks that the first heading in a document is a top-level heading (h1).
//! Note: This rule is deprecated in the original markdownlint but included for completeness.

use crate::error::Result;
use crate::rule::{AstRule, RuleCategory, RuleMetadata};
use crate::{
    Document,
    violation::{Severity, Violation},
};
use comrak::nodes::AstNode;

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

    fn check_ast<'a>(&self, document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let headings = document.headings(ast);

        // Find the first heading
        if let Some(first_heading) = headings.first() {
            if let Some(heading_level) = Document::heading_level(first_heading)
                && heading_level != self.level
            {
                if let Some((line, column)) = document.node_position(first_heading) {
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

                    violations.push(self.create_violation(
                        message,
                        line,
                        column,
                        Severity::Warning,
                    ));
                }
            }
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::Rule;
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
}
