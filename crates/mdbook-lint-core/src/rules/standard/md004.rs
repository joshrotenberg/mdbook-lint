//! MD004: Unordered list style consistency
//!
//! This rule checks that unordered list styles are consistent throughout the document.

use crate::error::Result;
use crate::rule::{AstRule, RuleCategory, RuleMetadata};
use crate::{
    Document,
    violation::{Severity, Violation},
};
use comrak::nodes::{AstNode, NodeValue};

/// List marker styles for unordered lists
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ListStyle {
    Asterisk, // *
    Plus,     // +
    Dash,     // -
}

impl ListStyle {
    fn from_char(c: char) -> Option<Self> {
        match c {
            '*' => Some(ListStyle::Asterisk),
            '+' => Some(ListStyle::Plus),
            '-' => Some(ListStyle::Dash),
            _ => None,
        }
    }

    fn to_char(self) -> char {
        match self {
            ListStyle::Asterisk => '*',
            ListStyle::Plus => '+',
            ListStyle::Dash => '-',
        }
    }
}

/// Configuration for list style checking
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ListStyleConfig {
    Consistent, // Use the first style found
    #[allow(dead_code)]
    Asterisk, // Enforce asterisk style
    #[allow(dead_code)]
    Plus, // Enforce plus style
    #[allow(dead_code)]
    Dash, // Enforce dash style
}

/// Rule to check unordered list style consistency
pub struct MD004 {
    /// The list style configuration
    style: ListStyleConfig,
}

impl MD004 {
    /// Create a new MD004 rule with consistent style (default)
    pub fn new() -> Self {
        Self {
            style: ListStyleConfig::Consistent,
        }
    }

    /// Create a new MD004 rule with a specific style
    #[allow(dead_code)]
    pub fn with_style(style: ListStyleConfig) -> Self {
        Self { style }
    }
}

impl Default for MD004 {
    fn default() -> Self {
        Self::new()
    }
}

impl AstRule for MD004 {
    fn id(&self) -> &'static str {
        "MD004"
    }

    fn name(&self) -> &'static str {
        "ul-style"
    }

    fn description(&self) -> &'static str {
        "Unordered list style"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Formatting).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_ast<'a>(&self, document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let mut expected_style: Option<ListStyle> = None;

        // If we have a configured style, use it immediately
        if let Some(configured_style) = self.get_configured_style() {
            expected_style = Some(configured_style);
        }

        // Find all unordered list items
        for node in ast.descendants() {
            if let NodeValue::List(list_info) = &node.data.borrow().value {
                // Only check unordered lists
                if list_info.list_type == comrak::nodes::ListType::Bullet {
                    // Check each list item in this list
                    for child in node.children() {
                        if let NodeValue::Item(_) = &child.data.borrow().value {
                            if let Some((line, column)) = document.node_position(child) {
                                // Get the list marker style from the source
                                if let Some(detected_style) =
                                    self.detect_list_marker_style(document, line)
                                {
                                    if let Some(expected) = expected_style {
                                        // We have an expected style, check if it matches
                                        if detected_style != expected {
                                            violations.push(self.create_violation(
                                                format!(
                                                    "Inconsistent list style: expected '{}' but found '{}'",
                                                    expected.to_char(),
                                                    detected_style.to_char()
                                                ),
                                                line,
                                                column,
                                                Severity::Warning,
                                            ));
                                        }
                                    } else {
                                        // First list found, set the expected style
                                        expected_style = Some(detected_style);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(violations)
    }
}

impl MD004 {
    /// Get the configured style if one is set
    fn get_configured_style(&self) -> Option<ListStyle> {
        match self.style {
            ListStyleConfig::Consistent => None,
            ListStyleConfig::Asterisk => Some(ListStyle::Asterisk),
            ListStyleConfig::Plus => Some(ListStyle::Plus),
            ListStyleConfig::Dash => Some(ListStyle::Dash),
        }
    }

    /// Detect the list marker style from the source line
    fn detect_list_marker_style(
        &self,
        document: &Document,
        line_number: usize,
    ) -> Option<ListStyle> {
        if line_number == 0 || line_number > document.lines.len() {
            return None;
        }

        let line = &document.lines[line_number - 1]; // Convert to 0-based index

        // Find the first list marker character
        for ch in line.chars() {
            if let Some(style) = ListStyle::from_char(ch) {
                return Some(style);
            }
            // Stop if we hit non-whitespace that isn't a list marker
            if !ch.is_whitespace() {
                break;
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use crate::rule::Rule;
    use std::path::PathBuf;

    #[test]
    fn test_md004_consistent_asterisk_style() {
        let content = r#"# List Test

* Item 1
* Item 2
* Item 3

Some text.

* Another list
* More items
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD004::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md004_inconsistent_styles_violation() {
        let content = r#"# List Test

* Item 1
+ Item 2
- Item 3
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD004::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("Inconsistent list style"));
        assert!(violations[0].message.contains("expected '*' but found '+'"));
        assert!(violations[1].message.contains("expected '*' but found '-'"));
        assert_eq!(violations[0].line, 4);
        assert_eq!(violations[1].line, 5);
    }

    #[test]
    fn test_md004_multiple_lists_consistent() {
        let content = r#"# Multiple Lists

First list:
- Item 1
- Item 2

Second list:
- Item 3
- Item 4

Third list:
- Item 5
- Item 6
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD004::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md004_multiple_lists_inconsistent() {
        let content = r#"# Multiple Lists

First list:
* Item 1
* Item 2

Second list:
+ Item 3
+ Item 4

Third list:
- Item 5
- Item 6
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD004::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 4);
        // Should detect all items in second and third lists as violations
        assert_eq!(violations[0].line, 8); // First + item
        assert_eq!(violations[1].line, 9); // Second + item
        assert_eq!(violations[2].line, 12); // First - item
        assert_eq!(violations[3].line, 13); // Second - item
    }

    #[test]
    fn test_md004_configured_asterisk_style() {
        let content = r#"# List Test

+ Item 1
+ Item 2
* Item 3
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD004::with_style(ListStyleConfig::Asterisk);
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("expected '*' but found '+'"));
        assert!(violations[1].message.contains("expected '*' but found '+'"));
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 4);
    }

    #[test]
    fn test_md004_configured_plus_style() {
        let content = r#"# List Test

* Item 1
+ Item 2
- Item 3
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD004::with_style(ListStyleConfig::Plus);
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("expected '+' but found '*'"));
        assert!(violations[1].message.contains("expected '+' but found '-'"));
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 5);
    }

    #[test]
    fn test_md004_configured_dash_style() {
        let content = r#"# List Test

* Item 1
+ Item 2
- Item 3
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD004::with_style(ListStyleConfig::Dash);
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("expected '-' but found '*'"));
        assert!(violations[1].message.contains("expected '-' but found '+'"));
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 4);
    }

    #[test]
    fn test_md004_nested_lists() {
        let content = r#"# Nested Lists

* Top level item
  + Nested item (different style should be violation)
  + Another nested item
* Another top level item
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD004::new();
        let violations = rule.check(&document).unwrap();

        // Should detect violations for the nested items
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 4);
        assert_eq!(violations[1].line, 5);
    }

    #[test]
    fn test_md004_ordered_lists_ignored() {
        let content = r#"# Mixed Lists

1. Ordered item 1
2. Ordered item 2

* Unordered item 1
* Unordered item 2

3. More ordered items
4. Should be ignored
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD004::new();
        let violations = rule.check(&document).unwrap();

        // Should only check unordered lists, ignore ordered lists
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md004_indented_lists() {
        let content = r#"# Indented Lists

Some paragraph with indented list:

  * Indented item 1
  * Indented item 2
  + Different style (should be violation)

Regular list:
* Regular item
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD004::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 7);
        assert!(violations[0].message.contains("expected '*' but found '+'"));
    }

    #[test]
    fn test_md004_empty_document() {
        let content = "";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD004::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md004_no_lists() {
        let content = r#"# Document Without Lists

This document has no lists, so there should be no violations.

Just paragraphs and headings.

## Another Section

More text without any lists.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD004::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }
}
