//! MD022: Headings should be surrounded by blank lines
//!
//! This rule is triggered when headings are not surrounded by blank lines.

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};
use comrak::nodes::{AstNode, NodeValue};

/// MD022: Headings should be surrounded by blank lines
///
/// This rule checks that headings have blank lines before and after them,
/// unless they are at the start or end of the document.
pub struct MD022;

impl AstRule for MD022 {
    fn id(&self) -> &'static str {
        "MD022"
    }

    fn name(&self) -> &'static str {
        "blanks-around-headings"
    }

    fn description(&self) -> &'static str {
        "Headings should be surrounded by blank lines"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Structure).introduced_in("markdownlint v0.1.0")
    }

    fn check_ast<'a>(&self, document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // Find all heading nodes in the AST
        for node in ast.descendants() {
            if let NodeValue::Heading(_) = &node.data.borrow().value
                && let Some((line, column)) = document.node_position(node)
            {
                // Check for blank line before the heading
                if !self.has_blank_line_before(document, line) {
                    violations.push(self.create_violation(
                        "Heading should be preceded by a blank line".to_string(),
                        line,
                        column,
                        Severity::Warning,
                    ));
                }

                // Check for blank line after the heading
                if !self.has_blank_line_after(document, line) {
                    violations.push(self.create_violation(
                        "Heading should be followed by a blank line".to_string(),
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

impl MD022 {
    /// Check if there's a blank line before the given line number
    fn has_blank_line_before(&self, document: &Document, line_num: usize) -> bool {
        // If this is the first line, no blank line needed
        if line_num <= 1 {
            return true;
        }

        // Check if the previous line is blank
        if let Some(prev_line) = document.lines.get(line_num - 2) {
            prev_line.trim().is_empty()
        } else {
            true // Start of document
        }
    }

    /// Check if there's a blank line after the given line number
    fn has_blank_line_after(&self, document: &Document, line_num: usize) -> bool {
        // If this is the last line, no blank line needed
        if line_num >= document.lines.len() {
            return true;
        }

        // Check if the next line is blank
        if let Some(next_line) = document.lines.get(line_num) {
            next_line.trim().is_empty()
        } else {
            true // End of document
        }
    }
}

#[cfg(test)]
// TODO: Tests temporarily disabled during migration (Part 2 of #66)
// Will be re-enabled when test_helpers is made public in Part 3
// mod tests {
    use super::*;
    // TODO: Re-enable when test_helpers is available
    // use mdbook_lint_core::test_helpers::*;

    #[test]
    fn test_md022_valid_headings() {
        let content = MarkdownBuilder::new()
            .heading(1, "Title")
            .blank_line()
            .paragraph("Some content here.")
            .blank_line()
            .heading(2, "Subtitle")
            .blank_line()
            .paragraph("More content.")
            .build();

        assert_no_violations(MD022, &content);
    }

    #[test]
    fn test_md022_missing_blank_before() {
        let content = MarkdownBuilder::new()
            .paragraph("Some text before.")
            .heading(1, "Title")
            .blank_line()
            .paragraph("Content after.")
            .build();

        let violations = assert_violation_count(MD022, &content, 1);
        assert_violation_contains_message(&violations, "preceded by a blank line");
        assert_violation_at_line(&violations, 2);
    }

    #[test]
    fn test_md022_missing_blank_after() {
        let content = MarkdownBuilder::new()
            .heading(1, "Title")
            .paragraph("Content immediately after.")
            .build();

        let violations = assert_violation_count(MD022, &content, 1);
        assert_violation_contains_message(&violations, "followed by a blank line");
        assert_violation_at_line(&violations, 1);
    }

    #[test]
    fn test_md022_missing_both_blanks() {
        let content = MarkdownBuilder::new()
            .paragraph("Text before.")
            .heading(1, "Title")
            .paragraph("Text after.")
            .build();

        let violations = assert_violation_count(MD022, &content, 2);
        assert_violation_contains_message(&violations, "preceded by a blank line");
        assert_violation_contains_message(&violations, "followed by a blank line");
    }

    #[test]
    fn test_md022_start_of_document() {
        let content = MarkdownBuilder::new()
            .heading(1, "Title")
            .blank_line()
            .paragraph("Content after.")
            .build();

        // Should be valid at start of document
        assert_no_violations(MD022, &content);
    }

    #[test]
    fn test_md022_end_of_document() {
        let content = MarkdownBuilder::new()
            .paragraph("Some content.")
            .blank_line()
            .heading(1, "Final Heading")
            .build();

        // Should be valid at end of document
        assert_no_violations(MD022, &content);
    }

    #[test]
    fn test_md022_multiple_headings() {
        let content = MarkdownBuilder::new()
            .heading(1, "Main Title")
            .blank_line()
            .paragraph("Introduction text.")
            .blank_line()
            .heading(2, "Section 1")
            .blank_line()
            .paragraph("Section content.")
            .blank_line()
            .heading(2, "Section 2")
            .blank_line()
            .paragraph("More content.")
            .build();

        assert_no_violations(MD022, &content);
    }

    #[test]
    fn test_md022_consecutive_headings() {
        let content = MarkdownBuilder::new()
            .heading(1, "Main Title")
            .blank_line()
            .heading(2, "Subtitle")
            .blank_line()
            .paragraph("Content.")
            .build();

        assert_no_violations(MD022, &content);
    }

    #[test]
    fn test_md022_mixed_heading_levels() {
        let content = MarkdownBuilder::new()
            .heading(1, "Level 1")
            .blank_line()
            .heading(3, "Level 3")
            .blank_line()
            .heading(2, "Level 2")
            .blank_line()
            .paragraph("Content.")
            .build();

        assert_no_violations(MD022, &content);
    }

    #[test]
    fn test_md022_multiple_violations() {
        let content = MarkdownBuilder::new()
            .paragraph("Text before first heading.")
            .heading(1, "Title")
            .paragraph("No blank lines around this heading.")
            .heading(2, "Subtitle")
            .paragraph("More text.")
            .build();

        let violations = assert_violation_count(MD022, &content, 4);
        // First heading: missing before and after
        // Second heading: missing before and after
        assert_violation_contains_message(&violations, "preceded by a blank line");
        assert_violation_contains_message(&violations, "followed by a blank line");
    }

    #[test]
    fn test_md022_headings_with_other_elements() {
        let content = MarkdownBuilder::new()
            .heading(1, "Document Title")
            .blank_line()
            .blockquote("This is a quote before the next heading.")
            .blank_line()
            .heading(2, "Section with Quote")
            .blank_line()
            .unordered_list(&["Item 1", "Item 2", "Item 3"])
            .blank_line()
            .heading(3, "Section with List")
            .blank_line()
            .code_block("rust", "fn main() {}")
            .build();

        assert_no_violations(MD022, &content);
    }

    #[test]
    fn test_md022_heading_immediately_after_code_block() {
        let content = MarkdownBuilder::new()
            .code_block("rust", "fn main() {}")
            .heading(1, "Heading")
            .blank_line()
            .paragraph("Content.")
            .build();

        let violations = assert_violation_count(MD022, &content, 1);
        assert_violation_contains_message(&violations, "preceded by a blank line");
    }

    #[test]
    fn test_md022_single_heading_document() {
        let content = MarkdownBuilder::new().heading(1, "Only Heading").build();

        // Single heading at start and end of document should be valid
        assert_no_violations(MD022, &content);
    }
// }
