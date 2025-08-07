//! MD003: Heading style consistency
//!
//! This rule is triggered when different heading styles (ATX, Setext, and ATX closed)
//! are used in the same document.

use crate::Document;
use crate::error::Result;
use crate::rule::{RuleCategory, RuleMetadata};
use crate::violation::{Severity, Violation};
use comrak::nodes::{AstNode, NodeValue};
use serde::{Deserialize, Serialize};

/// Configuration for MD003 heading style consistency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Md003Config {
    /// The heading style to enforce
    /// - "consistent": Auto-detect from first heading and enforce consistency
    /// - "atx": Require ATX style (# Header)
    /// - "atx_closed": Require ATX closed style (# Header #)
    /// - "setext": Require Setext style (Header\n======)
    /// - "setext_with_atx": Allow Setext for levels 1-2, ATX for 3+
    pub style: String,
}

impl Default for Md003Config {
    fn default() -> Self {
        Self {
            style: "consistent".to_string(),
        }
    }
}

/// MD003: Heading style should be consistent throughout the document
pub struct MD003 {
    config: Md003Config,
}

impl MD003 {
    pub fn new() -> Self {
        Self {
            config: Md003Config::default(),
        }
    }

    #[allow(dead_code)]
    pub fn with_config(config: Md003Config) -> Self {
        Self { config }
    }
}

impl Default for MD003 {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::rule::AstRule for MD003 {
    fn id(&self) -> &'static str {
        "MD003"
    }

    fn name(&self) -> &'static str {
        "heading-style"
    }

    fn description(&self) -> &'static str {
        "Heading style should be consistent throughout the document"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Structure).introduced_in("markdownlint v0.1.0")
    }

    fn check_ast<'a>(&self, document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let mut headings = Vec::new();

        // Collect all headings with their styles
        self.collect_headings(ast, document, &mut headings);

        if headings.is_empty() {
            return Ok(violations);
        }

        // Determine the expected style
        let expected_style = self.determine_expected_style(&headings);

        // Check each heading against the expected style
        for heading in &headings {
            if !self.is_valid_style(&heading.style, &expected_style, heading.level) {
                violations.push(self.create_violation(
                    format!(
                        "Expected '{}' style heading but found '{}' style",
                        expected_style, heading.style
                    ),
                    heading.line,
                    heading.column,
                    Severity::Error,
                ));
            }
        }

        Ok(violations)
    }
}

impl MD003 {
    /// Recursively collect all headings from the AST
    fn collect_headings<'a>(
        &self,
        node: &'a AstNode<'a>,
        document: &Document,
        headings: &mut Vec<HeadingInfo>,
    ) {
        if let NodeValue::Heading(heading_data) = &node.data.borrow().value {
            let position = node.data.borrow().sourcepos;
            let style = self.determine_heading_style(node, document, position.start.line);
            headings.push(HeadingInfo {
                level: heading_data.level,
                style,
                line: position.start.line,
                column: position.start.column,
            });
        }

        // Recursively process child nodes
        for child in node.children() {
            self.collect_headings(child, document, headings);
        }
    }

    /// Determine the style of a specific heading
    fn determine_heading_style(
        &self,
        _node: &AstNode,
        document: &Document,
        line_number: usize,
    ) -> HeadingStyle {
        // Get the line content (convert to 0-based indexing)
        let line_index = line_number.saturating_sub(1);
        if line_index >= document.lines.len() {
            return HeadingStyle::Atx;
        }

        let line = &document.lines[line_index];
        let trimmed = line.trim();

        // Check if it's ATX style (starts with #)
        if trimmed.starts_with('#') {
            // Check if it's ATX closed (ends with #)
            if trimmed.ends_with('#') && trimmed.len() > 1 {
                // Make sure it's not just a line of # characters
                let content = trimmed.trim_start_matches('#').trim_end_matches('#').trim();
                if !content.is_empty() {
                    return HeadingStyle::AtxClosed;
                }
            }
            return HeadingStyle::Atx;
        }

        // Check if it's Setext style (next line has === or ---)
        if line_index + 1 < document.lines.len() {
            let next_line = &document.lines[line_index + 1];
            let next_trimmed = next_line.trim();

            if !next_trimmed.is_empty() {
                let first_char = next_trimmed.chars().next().unwrap();
                if (first_char == '=' || first_char == '-')
                    && next_trimmed.chars().all(|c| c == first_char)
                {
                    return HeadingStyle::Setext;
                }
            }
        }

        // Default to ATX if we can't determine (shouldn't happen with valid markdown)
        HeadingStyle::Atx
    }

    /// Determine the expected style for the document
    fn determine_expected_style(&self, headings: &[HeadingInfo]) -> HeadingStyle {
        match self.config.style.as_str() {
            "atx" => HeadingStyle::Atx,
            "atx_closed" => HeadingStyle::AtxClosed,
            "setext" => HeadingStyle::Setext,
            "setext_with_atx" => HeadingStyle::SetextWithAtx,
            "consistent" => {
                // Use the style of the first heading
                headings
                    .first()
                    .map(|h| h.style.clone())
                    .unwrap_or(HeadingStyle::Atx)
            }
            _ => {
                // Use the style of the first heading
                headings
                    .first()
                    .map(|h| h.style.clone())
                    .unwrap_or(HeadingStyle::Atx)
            }
        }
    }

    /// Check if a heading style is valid given the expected style and level
    fn is_valid_style(&self, actual: &HeadingStyle, expected: &HeadingStyle, level: u8) -> bool {
        match expected {
            HeadingStyle::SetextWithAtx => {
                // Setext for levels 1-2, ATX for 3+
                if level <= 2 {
                    matches!(actual, HeadingStyle::Setext)
                } else {
                    matches!(actual, HeadingStyle::Atx)
                }
            }
            _ => actual == expected,
        }
    }
}

/// Information about a heading found in the document
#[derive(Debug, Clone)]
struct HeadingInfo {
    level: u8,
    style: HeadingStyle,
    line: usize,
    column: usize,
}

/// The different heading styles in Markdown
#[derive(Debug, Clone, PartialEq, Eq)]
enum HeadingStyle {
    /// ATX style: # Header
    Atx,
    /// ATX closed style: # Header #
    AtxClosed,
    /// Setext style: Header\n======
    Setext,
    /// Mixed style: Setext for levels 1-2, ATX for 3+
    SetextWithAtx,
}

impl std::fmt::Display for HeadingStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeadingStyle::Atx => write!(f, "atx"),
            HeadingStyle::AtxClosed => write!(f, "atx_closed"),
            HeadingStyle::Setext => write!(f, "setext"),
            HeadingStyle::SetextWithAtx => write!(f, "setext_with_atx"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use crate::rule::Rule;
    use std::path::PathBuf;

    fn create_test_document(content: &str) -> Document {
        Document::new(content.to_string(), PathBuf::from("test.md")).unwrap()
    }

    #[test]
    fn test_md003_consistent_atx_style() {
        let content = r#"# Main Title

## Section A

### Subsection 1

## Section B

### Subsection 2
"#;
        let doc = create_test_document(content);
        let rule = MD003::new();
        let violations = rule.check(&doc).unwrap();

        assert_eq!(
            violations.len(),
            0,
            "Consistent ATX style should not trigger violations"
        );
    }

    #[test]
    fn test_md003_consistent_atx_closed_style() {
        let content = r#"# Main Title #

## Section A ##

### Subsection 1 ###

## Section B ##
"#;
        let doc = create_test_document(content);
        let rule = MD003::new();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(
            violations.len(),
            0,
            "Consistent ATX closed style should not trigger violations"
        );
    }

    #[test]
    fn test_md003_consistent_setext_style() {
        let content = r#"Main Title
==========

Section A
---------

Section B
---------
"#;
        let doc = create_test_document(content);
        let rule = MD003::new();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(
            violations.len(),
            0,
            "Consistent Setext style should not trigger violations"
        );
    }

    #[test]
    fn test_md003_mixed_styles_violation() {
        let content = r#"# Main Title

Section A
---------

## Section B
"#;
        let doc = create_test_document(content);
        let rule = MD003::new();
        let violations = rule.check(&doc).unwrap();

        // Should have violations for inconsistent styles
        assert!(
            !violations.is_empty(),
            "Mixed heading styles should trigger violations"
        );

        let violation_messages: Vec<&str> = violations.iter().map(|v| v.message.as_str()).collect();

        // At least one violation should mention the style inconsistency
        assert!(
            violation_messages
                .iter()
                .any(|msg| msg.contains("Expected 'atx' style"))
        );
    }

    #[test]
    fn test_md003_atx_and_atx_closed_mixed() {
        let content = r#"# Main Title

## Section A ##

### Subsection 1

## Section B ##
"#;
        let doc = create_test_document(content);
        let rule = MD003::new();
        let violations = rule.check(&doc).unwrap();

        // Should have violations for mixing ATX and ATX closed
        assert!(
            !violations.is_empty(),
            "Mixed ATX and ATX closed styles should trigger violations"
        );
    }

    #[test]
    fn test_md003_configured_atx_style() {
        let content = r#"Main Title
==========

Section A
---------
"#;
        let doc = create_test_document(content);
        let config = Md003Config {
            style: "atx".to_string(),
        };
        let rule = MD003::with_config(config);
        let violations = rule.check(&doc).unwrap();

        // Should have violations because we're requiring ATX but document uses Setext
        assert!(
            !violations.is_empty(),
            "Setext headings should violate when ATX is required"
        );
    }

    #[test]
    fn test_md003_configured_setext_style() {
        let content = r#"# Main Title

## Section A
"#;
        let doc = create_test_document(content);
        let config = Md003Config {
            style: "setext".to_string(),
        };
        let rule = MD003::with_config(config);
        let violations = rule.check(&doc).unwrap();

        // Should have violations because we're requiring Setext but document uses ATX
        assert!(
            !violations.is_empty(),
            "ATX headings should violate when Setext is required"
        );
    }

    #[test]
    fn test_md003_setext_with_atx_valid() {
        let content = r#"Main Title
==========

Section A
---------

### Subsection 1

#### Deep Section
"#;
        let doc = create_test_document(content);
        let config = Md003Config {
            style: "setext_with_atx".to_string(),
        };
        let rule = MD003::with_config(config);
        let violations = rule.check(&doc).unwrap();

        assert_eq!(
            violations.len(),
            0,
            "Setext for levels 1-2 and ATX for 3+ should be valid"
        );
    }

    #[test]
    fn test_md003_setext_with_atx_violation() {
        let content = r#"# Main Title

Section A
---------

### Subsection 1
"#;
        let doc = create_test_document(content);
        let config = Md003Config {
            style: "setext_with_atx".to_string(),
        };
        let rule = MD003::with_config(config);
        let violations = rule.check(&doc).unwrap();

        // Should have violation for ATX level 1 when Setext is expected
        assert!(
            !violations.is_empty(),
            "ATX level 1 should violate setext_with_atx style"
        );
    }

    #[test]
    fn test_md003_no_headings() {
        let content = r#"This is a document with no headings.

Just some regular text content.
"#;
        let doc = create_test_document(content);
        let rule = MD003::new();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(
            violations.len(),
            0,
            "Documents with no headings should not trigger violations"
        );
    }

    #[test]
    fn test_md003_single_heading() {
        let content = r#"# Only One Heading

Some content here.
"#;
        let doc = create_test_document(content);
        let rule = MD003::new();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(
            violations.len(),
            0,
            "Documents with single heading should not trigger violations"
        );
    }
}
