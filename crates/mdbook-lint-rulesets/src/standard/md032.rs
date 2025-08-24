//! MD032: Lists should be surrounded by blank lines
//!
//! This rule is triggered when lists are not surrounded by blank lines.

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};
use comrak::nodes::{AstNode, NodeValue};

/// MD032: Lists should be surrounded by blank lines
///
/// This rule checks that lists have blank lines before and after them,
/// unless they are at the start or end of the document, or are nested within other lists.
pub struct MD032;

impl AstRule for MD032 {
    fn id(&self) -> &'static str {
        "MD032"
    }

    fn name(&self) -> &'static str {
        "blanks-around-lists"
    }

    fn description(&self) -> &'static str {
        "Lists should be surrounded by blank lines"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Structure).introduced_in("markdownlint v0.1.0")
    }

    fn check_ast<'a>(&self, document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // Find all list nodes in the AST
        for node in ast.descendants() {
            if let NodeValue::List(_) = &node.data.borrow().value {
                // Skip nested lists - only check top-level lists
                if !self.is_nested_list(node)
                    && let Some((start_line, start_column)) = document.node_position(node)
                {
                    // Check for blank line before the list
                    if !self.has_blank_line_before(document, start_line) {
                        violations.push(self.create_violation(
                            "List should be preceded by a blank line".to_string(),
                            start_line,
                            start_column,
                            Severity::Warning,
                        ));
                    }

                    // Find the end line of the list by checking all its descendants
                    let end_line = self.find_list_end_line(document, node);
                    if !self.has_blank_line_after(document, end_line) {
                        violations.push(self.create_violation(
                            "List should be followed by a blank line".to_string(),
                            end_line,
                            1,
                            Severity::Warning,
                        ));
                    }
                }
            }
        }

        Ok(violations)
    }
}

impl MD032 {
    /// Check if a list is nested within another list
    fn is_nested_list(&self, list_node: &AstNode) -> bool {
        let mut current = list_node.parent();
        while let Some(parent) = current {
            match &parent.data.borrow().value {
                NodeValue::List(_) => return true,
                NodeValue::Item(_) => {
                    // Check if this item's parent is a list
                    if let Some(grandparent) = parent.parent()
                        && let NodeValue::List(_) = &grandparent.data.borrow().value
                    {
                        return true;
                    }
                }
                _ => {}
            }
            current = parent.parent();
        }
        false
    }

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

    /// Find the end line of a list by examining all its descendants
    fn find_list_end_line<'a>(&self, document: &Document, list_node: &'a AstNode<'a>) -> usize {
        let mut max_line = 1;

        // Walk through all descendants to find the maximum line number
        for descendant in list_node.descendants() {
            if let Some((line, _)) = document.node_position(descendant) {
                max_line = max_line.max(line);
            }
        }

        max_line
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
    fn test_md032_valid_unordered_list() {
        let content = MarkdownBuilder::new()
            .heading(1, "Title")
            .blank_line()
            .unordered_list(&["Item 1", "Item 2", "Item 3"])
            .blank_line()
            .paragraph("Some text after.")
            .build();

        assert_no_violations(MD032, &content);
    }

    #[test]
    fn test_md032_valid_ordered_list() {
        let content = MarkdownBuilder::new()
            .heading(1, "Title")
            .blank_line()
            .ordered_list(&["First item", "Second item", "Third item"])
            .blank_line()
            .paragraph("Some text after.")
            .build();

        assert_no_violations(MD032, &content);
    }

    #[test]
    fn test_md032_missing_blank_before() {
        let content = MarkdownBuilder::new()
            .heading(1, "Title")
            .unordered_list(&["Item 1", "Item 2", "Item 3"])
            .blank_line()
            .paragraph("Some text after.")
            .build();

        let violations = assert_violation_count(MD032, &content, 1);
        assert_violation_contains_message(&violations, "preceded by a blank line");
    }

    #[test]
    fn test_md032_missing_blank_after() {
        // When there's no blank line after a list, markdown parsers treat
        // the following text as part of the last list item, so no violation occurs
        let content = MarkdownBuilder::new()
            .heading(1, "Title")
            .blank_line()
            .unordered_list(&["Item 1", "Item 2", "Item 3"])
            .paragraph("Some text after.")
            .build();

        // This is actually valid markdown - no violations expected
        assert_no_violations(MD032, &content);
    }

    #[test]
    fn test_md032_missing_both_blanks() {
        let content = MarkdownBuilder::new()
            .heading(1, "Title")
            .unordered_list(&["Item 1", "Item 2", "Item 3"])
            .paragraph("Some text after.")
            .build();

        // Only the "before" violation is detected since "after" becomes part of the list
        let violations = assert_violation_count(MD032, &content, 1);
        assert_violation_contains_message(&violations, "preceded by a blank line");
    }

    #[test]
    fn test_md032_start_of_document() {
        let content = MarkdownBuilder::new()
            .unordered_list(&["Item 1", "Item 2", "Item 3"])
            .blank_line()
            .paragraph("Some text after.")
            .build();

        // Should be valid at start of document
        assert_no_violations(MD032, &content);
    }

    #[test]
    fn test_md032_end_of_document() {
        let content = MarkdownBuilder::new()
            .heading(1, "Title")
            .blank_line()
            .unordered_list(&["Item 1", "Item 2", "Item 3"])
            .build();

        // Should be valid at end of document
        assert_no_violations(MD032, &content);
    }

    #[test]
    fn test_md032_nested_lists_ignored() {
        let content = r#"# Title

- Item 1
  - Nested item 1
  - Nested item 2
- Item 2
- Item 3

Some text after.
"#;
        // Only the top-level list should be checked, nested lists are ignored
        assert_no_violations(MD032, content);
    }

    #[test]
    fn test_md032_multiple_lists() {
        let content = MarkdownBuilder::new()
            .heading(1, "Title")
            .blank_line()
            .unordered_list(&["First list item 1", "First list item 2"])
            .blank_line()
            .paragraph("Some text in between.")
            .blank_line()
            .ordered_list(&["Second list item 1", "Second list item 2"])
            .blank_line()
            .paragraph("End.")
            .build();

        assert_no_violations(MD032, &content);
    }

    #[test]
    fn test_md032_mixed_list_types() {
        // Different list markers create separate lists in markdown
        let content = r#"# Title

- Unordered item

* Different marker

+ Another marker

Some text.

1. Ordered item
2. Another ordered item

End.
"#;
        assert_no_violations(MD032, content);
    }

    #[test]
    fn test_md032_list_with_multiline_items() {
        let content = r#"# Title

- Item 1 with a very long line that wraps
  to multiple lines
- Item 2 which also has
  multiple lines of content
- Item 3

Some text after.
"#;
        assert_no_violations(MD032, content);
    }

    #[test]
    fn test_md032_numbered_list_variations() {
        let content = MarkdownBuilder::new()
            .heading(1, "Title")
            .blank_line()
            .ordered_list(&["Item one", "Item two", "Item three"])
            .blank_line()
            .paragraph("Text between.")
            .blank_line()
            .line("1) Parenthesis style")
            .line("2) Another item")
            .line("3) Third item")
            .blank_line()
            .paragraph("End.")
            .build();

        assert_no_violations(MD032, &content);
    }

    #[test]
    fn test_md032_markdown_parsing_behavior() {
        // This test documents how markdown parsers handle lists without blank lines
        let content = "# Title\n\n- Item 1\n- Item 2\n- Item 3\nText immediately after.";

        // In markdown, text without a blank line after a list becomes part of the last item
        // So this is actually valid markdown structure - no violations expected
        assert_no_violations(MD032, content);
    }
// }
