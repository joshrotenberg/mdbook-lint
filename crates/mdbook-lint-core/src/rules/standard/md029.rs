//! MD029: Ordered list item prefix consistency
//!
//! This rule checks for consistent numbering style in ordered lists.
//! Lists can use either sequential numbering (1, 2, 3) or all ones (1, 1, 1).

use crate::error::Result;
use crate::rule::{AstRule, RuleCategory, RuleMetadata};
use crate::{
    Document,
    violation::{Severity, Violation},
};
use comrak::nodes::{AstNode, ListType, NodeValue};

/// Configuration for ordered list prefix style
#[derive(Debug, Clone, PartialEq)]
pub enum OrderedListStyle {
    /// Sequential numbering: 1, 2, 3, 4...
    Sequential,
    /// All ones: 1, 1, 1, 1...
    AllOnes,
    /// Use whatever style is found first in the document
    Consistent,
}

/// Rule to check for ordered list item prefix consistency
pub struct MD029 {
    style: OrderedListStyle,
}

impl MD029 {
    /// Create a new MD029 rule with default settings (consistent style)
    pub fn new() -> Self {
        Self {
            style: OrderedListStyle::Consistent,
        }
    }

    /// Create a new MD029 rule with a specific style
    #[allow(dead_code)]
    pub fn with_style(style: OrderedListStyle) -> Self {
        Self { style }
    }
}

impl Default for MD029 {
    fn default() -> Self {
        Self::new()
    }
}

impl AstRule for MD029 {
    fn id(&self) -> &'static str {
        "MD029"
    }

    fn name(&self) -> &'static str {
        "ol-prefix"
    }

    fn description(&self) -> &'static str {
        "Ordered list item prefix consistency"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Formatting).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_ast<'a>(&self, document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let mut detected_style: Option<OrderedListStyle> = None;

        // Find all ordered list nodes
        for node in ast.descendants() {
            if let NodeValue::List(list_data) = &node.data.borrow().value
                && let ListType::Ordered = list_data.list_type
            {
                violations.extend(self.check_ordered_list(document, node, &mut detected_style)?);
            }
        }

        Ok(violations)
    }
}

impl MD029 {
    /// Check an individual ordered list for prefix consistency
    fn check_ordered_list<'a>(
        &self,
        document: &Document,
        list_node: &'a AstNode<'a>,
        detected_style: &mut Option<OrderedListStyle>,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let mut list_items = Vec::new();

        // Collect all list items with their line numbers and prefixes
        for child in list_node.children() {
            if let NodeValue::Item(_) = &child.data.borrow().value
                && let Some((line_num, _)) = document.node_position(child)
                && let Some(line) = document.lines.get(line_num - 1)
                && let Some(prefix) = self.extract_list_prefix(line)
            {
                list_items.push((line_num, prefix));
            }
        }

        if list_items.len() < 2 {
            // Single item lists don't need consistency checking
            return Ok(violations);
        }

        // Determine the expected style for this list
        let expected_style = match &self.style {
            OrderedListStyle::Sequential => OrderedListStyle::Sequential,
            OrderedListStyle::AllOnes => OrderedListStyle::AllOnes,
            OrderedListStyle::Consistent => {
                if let Some(style) = detected_style {
                    style.clone()
                } else {
                    // Detect style from this list
                    let detected = self.detect_list_style(&list_items);
                    *detected_style = Some(detected.clone());
                    detected
                }
            }
        };

        // Check each item against the expected style
        for (i, (line_num, actual_prefix)) in list_items.iter().enumerate() {
            let expected_prefix = match expected_style {
                OrderedListStyle::Sequential => (i + 1).to_string(),
                OrderedListStyle::AllOnes => "1".to_string(),
                OrderedListStyle::Consistent => {
                    // This case is handled by detecting the style first
                    continue;
                }
            };

            if actual_prefix != &expected_prefix {
                violations.push(self.create_violation(
                    format!(
                        "Ordered list item prefix inconsistent: expected '{expected_prefix}', found '{actual_prefix}'"
                    ),
                    *line_num,
                    1,
                    Severity::Warning,
                ));
            }
        }

        Ok(violations)
    }

    /// Extract the numeric prefix from a list item line
    fn extract_list_prefix(&self, line: &str) -> Option<String> {
        let trimmed = line.trim_start();

        // Look for pattern like "1. " or "42. "
        if let Some(dot_pos) = trimmed.find('.') {
            let prefix = &trimmed[..dot_pos];
            if prefix.chars().all(|c| c.is_ascii_digit()) && !prefix.is_empty() {
                return Some(prefix.to_string());
            }
        }

        None
    }

    /// Detect the style used in a list based on its items
    fn detect_list_style(&self, items: &[(usize, String)]) -> OrderedListStyle {
        if items.len() < 2 {
            return OrderedListStyle::Sequential; // Default for single items
        }

        // Check if all items use "1"
        if items.iter().all(|(_, prefix)| prefix == "1") {
            return OrderedListStyle::AllOnes;
        }

        // Check if items are sequential starting from 1
        for (i, (_, prefix)) in items.iter().enumerate() {
            if prefix != &(i + 1).to_string() {
                // Not sequential, return the style of the first item
                return if items[0].1 == "1" {
                    OrderedListStyle::AllOnes
                } else {
                    OrderedListStyle::Sequential
                };
            }
        }

        OrderedListStyle::Sequential
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use crate::rule::Rule;
    use std::path::PathBuf;

    #[test]
    fn test_md029_no_violations_sequential() {
        let content = r#"# Sequential Lists

1. First item
2. Second item
3. Third item
4. Fourth item

Another list:

1. Item one
2. Item two
3. Item three

Text here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD029::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md029_no_violations_all_ones() {
        let content = r#"# All Ones Lists

1. First item
1. Second item
1. Third item
1. Fourth item

Another list:

1. Item one
1. Item two
1. Item three

Text here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD029::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md029_inconsistent_numbering() {
        let content = r#"# Inconsistent Numbering

1. First item
1. Second item should be 2
3. Third item is correct
1. Fourth item should be 4

Text here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD029::with_style(OrderedListStyle::Sequential);
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("expected '2', found '1'"));
        assert!(violations[1].message.contains("expected '4', found '1'"));
        assert_eq!(violations[0].line, 4);
        assert_eq!(violations[1].line, 6);
    }

    #[test]
    fn test_md029_mixed_styles_in_document() {
        let content = r#"# Mixed Styles

First list (sequential):
1. First item
2. Second item
3. Third item

Second list (all ones):
1. First item
1. Second item
1. Third item

Text here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD029::new(); // Consistent mode
        let violations = rule.check(&document).unwrap();

        // With consistent mode, it should detect inconsistency
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 10); // Second list, second item
        assert_eq!(violations[1].line, 11); // Second list, third item
    }

    #[test]
    fn test_md029_forced_sequential_style() {
        let content = r#"# Forced Sequential Style

1. First item
1. Should be 2
1. Should be 3
1. Should be 4

Text here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD029::with_style(OrderedListStyle::Sequential);
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);
        assert!(violations[0].message.contains("expected '2', found '1'"));
        assert!(violations[1].message.contains("expected '3', found '1'"));
        assert!(violations[2].message.contains("expected '4', found '1'"));
    }

    #[test]
    fn test_md029_forced_all_ones_style() {
        let content = r#"# Forced All Ones Style

1. First item
2. Should be 1
3. Should be 1
4. Should be 1

Text here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD029::with_style(OrderedListStyle::AllOnes);
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);
        assert!(violations[0].message.contains("expected '1', found '2'"));
        assert!(violations[1].message.contains("expected '1', found '3'"));
        assert!(violations[2].message.contains("expected '1', found '4'"));
    }

    #[test]
    fn test_md029_nested_lists() {
        let content = r#"# Nested Lists

1. Top level item
   1. Nested item one
   2. Nested item two
2. Second top level
   1. Another nested item
   1. This should be 2

Text here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD029::with_style(OrderedListStyle::Sequential);
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("expected '2', found '1'"));
        assert_eq!(violations[0].line, 8);
    }

    #[test]
    fn test_md029_single_item_lists() {
        let content = r#"# Single Item Lists

1. Only item in this list

Another single item:
1. Just this one

Text here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD029::new();
        let violations = rule.check(&document).unwrap();

        // Single item lists should not generate violations
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md029_moderately_indented_lists() {
        let content = r#"# Moderately Indented Lists

  1. Moderately indented list item
  2. Second moderately indented item
  1. This should be 3

Text here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD029::with_style(OrderedListStyle::Sequential);
        let violations = rule.check(&document).unwrap();

        // Test with moderately indented list (2 spaces - should still be parsed as list)
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("expected '3', found '1'"));
        assert_eq!(violations[0].line, 5);
    }

    #[test]
    fn test_md029_extract_prefix() {
        let rule = MD029::new();

        assert_eq!(
            rule.extract_list_prefix("1. Item text"),
            Some("1".to_string())
        );
        assert_eq!(
            rule.extract_list_prefix("42. Item text"),
            Some("42".to_string())
        );
        assert_eq!(
            rule.extract_list_prefix("  1. Indented item"),
            Some("1".to_string())
        );
        assert_eq!(
            rule.extract_list_prefix("    42. More indented"),
            Some("42".to_string())
        );

        // Invalid formats
        assert_eq!(rule.extract_list_prefix("- Unordered item"), None);
        assert_eq!(rule.extract_list_prefix("Not a list"), None);
        assert_eq!(rule.extract_list_prefix("1) Wrong delimiter"), None);
        assert_eq!(rule.extract_list_prefix("a. Letter prefix"), None);
    }

    #[test]
    fn test_md029_detect_style() {
        let rule = MD029::new();

        // Sequential style
        let sequential_items = vec![
            (1, "1".to_string()),
            (2, "2".to_string()),
            (3, "3".to_string()),
        ];
        assert_eq!(
            rule.detect_list_style(&sequential_items),
            OrderedListStyle::Sequential
        );

        // All ones style
        let all_ones_items = vec![
            (1, "1".to_string()),
            (2, "1".to_string()),
            (3, "1".to_string()),
        ];
        assert_eq!(
            rule.detect_list_style(&all_ones_items),
            OrderedListStyle::AllOnes
        );

        // Mixed style (defaults to all ones if starts with 1)
        let mixed_items = vec![
            (1, "1".to_string()),
            (2, "3".to_string()),
            (3, "1".to_string()),
        ];
        assert_eq!(
            rule.detect_list_style(&mixed_items),
            OrderedListStyle::AllOnes
        );
    }
}
