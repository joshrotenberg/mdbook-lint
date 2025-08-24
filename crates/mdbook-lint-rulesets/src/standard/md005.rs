//! MD005: List item indentation consistency
//!
//! This rule checks that list items have consistent indentation throughout the document.

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};
use comrak::nodes::{AstNode, NodeValue};

/// Rule to check for consistent list item indentation
pub struct MD005;

impl AstRule for MD005 {
    fn id(&self) -> &'static str {
        "MD005"
    }

    fn name(&self) -> &'static str {
        "list-indent"
    }

    fn description(&self) -> &'static str {
        "List item indentation should be consistent"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Formatting).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_ast<'a>(&self, document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // Find all list nodes
        for node in ast.descendants() {
            if let NodeValue::List(list_data) = &node.data.borrow().value {
                // Check indentation consistency within this list
                violations.extend(self.check_list_indentation(document, node, list_data)?);
            }
        }

        Ok(violations)
    }
}

impl MD005 {
    /// Check indentation consistency within a single list
    fn check_list_indentation<'a>(
        &self,
        document: &Document,
        list_node: &'a AstNode<'a>,
        _list_data: &comrak::nodes::NodeList,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let mut expected_indent: Option<usize> = None;

        // Iterate through list items
        for child in list_node.children() {
            if let NodeValue::Item(_) = &child.data.borrow().value
                && let Some((line_num, _)) = document.node_position(child)
                && let Some(line) = document.lines.get(line_num - 1)
            {
                let actual_indent = self.get_line_indentation(line);

                // Set expected indentation from first item
                if expected_indent.is_none() {
                    expected_indent = Some(actual_indent);
                } else if let Some(expected) = expected_indent
                    && actual_indent != expected
                {
                    // Check if this item's indentation matches
                    violations.push(self.create_violation(
                        format!(
                            "List item indentation inconsistent: expected {expected} spaces, found {actual_indent}"
                        ),
                        line_num,
                        1,
                        Severity::Warning,
                    ));
                }
            }
        }

        Ok(violations)
    }

    /// Get the indentation level (number of leading spaces/tabs) of a line
    fn get_line_indentation(&self, line: &str) -> usize {
        let mut indent = 0;
        for ch in line.chars() {
            match ch {
                ' ' => indent += 1,
                '\t' => indent += 4, // Count tabs as 4 spaces
                _ => break,
            }
        }
        indent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::Document;
    use mdbook_lint_core::rule::Rule;
    use std::path::PathBuf;

    #[test]
    fn test_md005_no_violations() {
        let content = r#"# Consistent List Indentation

These lists have consistent indentation:

- Item 1
- Item 2
- Item 3

1. First item
2. Second item
3. Third item

Nested lists with consistent indentation:

- Top level
  - Nested item 1
  - Nested item 2
    - Deeply nested 1
    - Deeply nested 2
- Back to top level
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD005;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md005_inconsistent_indentation() {
        let content = r#"# Inconsistent List Indentation

This list has inconsistent indentation at the same level:

- Item 1
- Item 2
 - Item 3 (inconsistent - 1 space instead of 0)
- Item 4
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD005;
        let violations = rule.check(&document).unwrap();

        // Should detect inconsistent indentation in the main list
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("expected 0 spaces, found 1"));
    }

    #[test]
    fn test_md005_ordered_list_inconsistent() {
        let content = r#"# Inconsistent Ordered List

1. First item
 2. Second item (wrong indentation)
1. Third item
  3. Fourth item (wrong again)
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD005;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("expected 0 spaces, found 1"));
        assert!(violations[1].message.contains("expected 0 spaces, found 2"));
    }

    #[test]
    fn test_md005_mixed_spaces_tabs() {
        let content = "# Mixed Spaces and Tabs\n\n- Item 1\n\t- Item 2 (tab indented)\n    - Item 3 (space indented)\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD005;
        let violations = rule.check(&document).unwrap();

        // Should detect inconsistency between tab (4 spaces) and 4 actual spaces
        // Note: tabs are converted to 4 spaces for comparison
        assert_eq!(violations.len(), 0); // Both should be equivalent to 4 spaces
    }

    #[test]
    fn test_md005_separate_lists() {
        let content = r#"# Separate Lists

First list:
- Item A
- Item B

Second list with different indentation (should be OK):
  - Item X
  - Item Y

Third list:
1. Item 1
1. Item 2
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD005;
        let violations = rule.check(&document).unwrap();

        // Each list can have its own indentation style
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md005_nested_lists_independent() {
        let content = r#"# Nested Lists

- Top level item 1
- Top level item 2
  - Nested item A
   - Nested item B (inconsistent with nested level)
  - Nested item C
- Top level item 3
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD005;
        let violations = rule.check(&document).unwrap();

        // Should detect inconsistency in the nested list
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("expected 2 spaces, found 3"));
    }

    #[test]
    fn test_md005_empty_list() {
        let content = r#"# Empty or Single Item Lists

- Single item

1. Another single item

Some text here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD005;
        let violations = rule.check(&document).unwrap();

        // No violations for single-item lists
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md005_complex_nesting() {
        let content = r#"# Complex Nesting

- Level 1 item 1
  - Level 2 item 1
    - Level 3 item 1
    - Level 3 item 2
  - Level 2 item 2
   - Level 2 item 3 (wrong indentation)
- Level 1 item 2
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD005;
        let violations = rule.check(&document).unwrap();

        // Should detect the inconsistent level 2 item
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("expected 2 spaces, found 3"));
    }

    #[test]
    fn test_get_line_indentation() {
        let rule = MD005;

        assert_eq!(rule.get_line_indentation("- No indentation"), 0);
        assert_eq!(rule.get_line_indentation("  - Two spaces"), 2);
        assert_eq!(rule.get_line_indentation("    - Four spaces"), 4);
        assert_eq!(rule.get_line_indentation("\t- One tab"), 4);
        assert_eq!(rule.get_line_indentation("\t  - Tab plus two spaces"), 6);
        assert_eq!(rule.get_line_indentation("      - Six spaces"), 6);
    }
}
