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
pub struct MDBOOK004;

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
                    // Check for duplicates within the same document
                    if let Some((prev_line, _)) = title_positions.get(&title) {
                        violations.push(self.create_violation(
                            format!(
                                "Duplicate chapter title '{title}' found (also at line {prev_line})"
                            ),
                            line,
                            column,
                            Severity::Error,
                        ));
                    } else {
                        title_positions.insert(title, (line, column));
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

        assert_no_violations(MDBOOK004, &content);
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

        let violations = assert_violation_count(MDBOOK004, &content, 1);
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
        assert_no_violations(MDBOOK004, &content);
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
        let violations = assert_violation_count(MDBOOK004, &content, 2);
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
        assert_no_violations(MDBOOK004, &content);
    }

    #[test]
    fn test_mdbook004_whitespace_handling() {
        let content = MarkdownBuilder::new()
            .line("# Introduction ")
            .blank_line()
            .line("#  Introduction")
            .blank_line()
            .line("# Introduction  ")
            .build();

        // Whitespace should be trimmed, so these are duplicates
        let violations = assert_violation_count(MDBOOK004, &content, 2);
        assert_violation_contains_message(&violations, "Duplicate chapter title 'Introduction'");
    }

    #[test]
    fn test_mdbook004_rule_metadata() {
        use mdbook_lint_core::rule::AstRule;
        let rule = MDBOOK004;
        assert_eq!(AstRule::id(&rule), "MDBOOK004");
        assert_eq!(AstRule::name(&rule), "no-duplicate-chapter-titles");
        assert!(AstRule::description(&rule).contains("unique"));
    }
}
