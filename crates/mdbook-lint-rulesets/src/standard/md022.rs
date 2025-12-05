//! MD022: Headings should be surrounded by blank lines
//!
//! This rule is triggered when headings are not surrounded by blank lines.

use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Fix, Position, Severity, Violation},
};

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

    fn can_fix(&self) -> bool {
        true
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
                    // Create fix to add blank line before heading
                    let fix = if line > 1 {
                        // Get the previous line to determine where to insert the blank line
                        let prev_line_idx = line - 2; // Convert to 0-based index
                        let prev_line = &document.lines[prev_line_idx];

                        Fix {
                            description: "Add blank line before heading".to_string(),
                            replacement: Some(format!("{}\n", prev_line)),
                            start: Position {
                                line: line - 1,
                                column: 1,
                            },
                            end: Position {
                                line: line - 1,
                                column: prev_line.len() + 1,
                            },
                        }
                    } else {
                        // Can't add blank line before first line
                        Fix {
                            description: "Cannot add blank line before first heading".to_string(),
                            replacement: None,
                            start: Position { line, column },
                            end: Position { line, column },
                        }
                    };

                    violations.push(self.create_violation_with_fix(
                        "Heading should be preceded by a blank line".to_string(),
                        line,
                        column,
                        Severity::Warning,
                        fix,
                    ));
                }

                // Check for blank line after the heading
                if !self.has_blank_line_after(document, line) {
                    // Create fix to add blank line after heading
                    let current_line = &document.lines[line - 1]; // Convert to 0-based

                    let fix = Fix {
                        description: "Add blank line after heading".to_string(),
                        replacement: Some(format!("{}\n", current_line)),
                        start: Position { line, column: 1 },
                        end: Position {
                            line,
                            column: current_line.len() + 1,
                        },
                    };

                    violations.push(self.create_violation_with_fix(
                        "Heading should be followed by a blank line".to_string(),
                        line,
                        column,
                        Severity::Warning,
                        fix,
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
            is_blank_line(prev_line)
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
            is_blank_line(next_line)
        } else {
            true // End of document
        }
    }
}

/// Check if a line is considered "blank" for the purposes of spacing rules.
/// A line is blank if:
/// - It's empty or contains only whitespace
/// - It's a blockquote line with no content after the `>` marker (e.g., `>`, `> `)
fn is_blank_line(line: &str) -> bool {
    let trimmed = line.trim();

    // Empty line
    if trimmed.is_empty() {
        return true;
    }

    // Blockquote blank line: just `>` followed by nothing or whitespace
    // This handles nested blockquotes too (e.g., `> >`, `>> >`)
    let mut chars = trimmed.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '>' {
            // Continue checking for more `>` or whitespace
            while let Some(&next_ch) = chars.peek() {
                if next_ch == ' ' {
                    chars.next();
                } else {
                    break;
                }
            }
        } else {
            // Found non-blockquote content
            return false;
        }
    }

    // If we consumed all characters and they were all `>` and whitespace, it's blank
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::rule::Rule;
    use mdbook_lint_core::test_helpers::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_blank_line() {
        // Regular blank lines
        assert!(is_blank_line(""));
        assert!(is_blank_line("   "));
        assert!(is_blank_line("\t"));

        // Blockquote blank lines
        assert!(is_blank_line(">"));
        assert!(is_blank_line("> "));
        assert!(is_blank_line(">  "));
        assert!(is_blank_line(" > "));
        assert!(is_blank_line("  >"));

        // Nested blockquote blank lines
        assert!(is_blank_line("> >"));
        assert!(is_blank_line(">> "));
        assert!(is_blank_line("> > "));

        // Non-blank lines
        assert!(!is_blank_line("text"));
        assert!(!is_blank_line("> text"));
        assert!(!is_blank_line("> > text"));
        assert!(!is_blank_line(">text")); // No space after >, but has content
    }

    #[test]
    fn test_md022_heading_in_blockquote_with_blank_lines() {
        // Issue #275: Headings inside blockquotes with blank lines (>) should be valid
        let content = r#"> ### Command Line Notation
>
> In this chapter and throughout the book, we'll show some commands used in the
> terminal.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD022;
        let violations = rule.check(&document).unwrap();

        // The `>` line IS a blank line within the blockquote context
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md022_heading_in_blockquote_missing_blank() {
        // Heading in blockquote without proper blank line
        let content = r#"> ### Command Line Notation
> In this chapter immediately after.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD022;
        let violations = rule.check(&document).unwrap();

        // Should have 1 violation - missing blank after
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("followed by a blank line"));
    }

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

    #[test]
    fn test_md022_fix_missing_blank_before() {
        let content = r#"Some text before.
# Title

Content after."#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD022;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Add blank line before heading");
        assert_eq!(fix.replacement, Some("Some text before.\n".to_string()));
        assert_eq!(fix.start.line, 1);
        assert_eq!(fix.start.column, 1);
    }

    #[test]
    fn test_md022_fix_missing_blank_after() {
        let content = r#"# Title
Content immediately after."#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD022;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Add blank line after heading");
        assert_eq!(fix.replacement, Some("# Title\n".to_string()));
        assert_eq!(fix.start.line, 1);
        assert_eq!(fix.start.column, 1);
    }

    #[test]
    fn test_md022_fix_both_missing() {
        let content = r#"Text before.
## Heading
Text after."#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD022;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);

        // Check fix for missing blank before
        let before_fix = violations
            .iter()
            .find(|v| v.message.contains("preceded"))
            .unwrap();
        assert!(before_fix.fix.is_some());
        let fix = before_fix.fix.as_ref().unwrap();
        assert_eq!(fix.description, "Add blank line before heading");

        // Check fix for missing blank after
        let after_fix = violations
            .iter()
            .find(|v| v.message.contains("followed"))
            .unwrap();
        assert!(after_fix.fix.is_some());
        let fix = after_fix.fix.as_ref().unwrap();
        assert_eq!(fix.description, "Add blank line after heading");
    }

    #[test]
    fn test_md022_fix_at_document_start() {
        let content = r#"# First Line Heading
No blank line after."#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD022;
        let violations = rule.check(&document).unwrap();

        // Should only need blank line after, not before (at start of doc)
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("followed"));
        assert!(violations[0].fix.is_some());
    }

    #[test]
    fn test_md022_fix_multiple_headings() {
        let content = r#"# First
Content
## Second
More content"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD022;
        let violations = rule.check(&document).unwrap();

        // Both headings should have violations
        assert!(violations.len() >= 2);

        // All violations should have fixes
        for violation in &violations {
            assert!(violation.fix.is_some());
            let fix = violation.fix.as_ref().unwrap();
            assert!(fix.description.contains("blank line"));
        }
    }
}
