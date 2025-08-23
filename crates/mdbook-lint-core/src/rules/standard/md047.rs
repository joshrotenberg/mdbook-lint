//! MD047: Files should end with a single newline character
//!
//! This rule checks that files end with exactly one newline character.

use crate::error::Result;
use crate::rule::{Rule, RuleCategory, RuleMetadata};
use crate::{
    Document,
    violation::{Fix, Position, Severity, Violation},
};

/// Rule to check that files end with a single newline
pub struct MD047;

impl MD047 {
    /// Check the ending of the file content
    fn check_file_ending(&self, content: &str) -> Option<String> {
        if content.is_empty() {
            return Some("File should end with a single newline character".to_string());
        }

        let ends_with_newline = content.ends_with('\n');
        let ends_with_multiple_newlines = content.ends_with("\n\n");

        if !ends_with_newline {
            Some("File should end with a single newline character".to_string())
        } else if ends_with_multiple_newlines {
            // Count trailing newlines
            let trailing_newlines = content.chars().rev().take_while(|&c| c == '\n').count();

            if trailing_newlines > 1 {
                Some(format!(
                    "File should end with a single newline character (found {trailing_newlines} trailing newlines)"
                ))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Rule for MD047 {
    fn id(&self) -> &'static str {
        "MD047"
    }

    fn name(&self) -> &'static str {
        "single-trailing-newline"
    }

    fn description(&self) -> &'static str {
        "Files should end with a single newline character"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Formatting).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        if let Some(message) = self.check_file_ending(&document.content) {
            let line_count = document.lines.len();
            let line_number = if line_count == 0 { 1 } else { line_count };

            // Create fix based on the specific issue
            let fix = if document.content.is_empty() {
                // Empty file: add a single newline
                Fix {
                    description: "Add newline at end of file".to_string(),
                    replacement: Some("\n".to_string()),
                    start: Position { line: 1, column: 1 },
                    end: Position { line: 1, column: 1 },
                }
            } else if !document.content.ends_with('\n') {
                // No trailing newline: add one
                let last_line_len = document.lines.last().map(|l| l.len()).unwrap_or(0) + 1;
                Fix {
                    description: "Add newline at end of file".to_string(),
                    replacement: Some("\n".to_string()),
                    start: Position {
                        line: line_number,
                        column: last_line_len,
                    },
                    end: Position {
                        line: line_number,
                        column: last_line_len,
                    },
                }
            } else {
                // Multiple trailing newlines: remove extras
                let trailing_newlines = document
                    .content
                    .chars()
                    .rev()
                    .take_while(|&c| c == '\n')
                    .count();
                let start_line = line_count - trailing_newlines + 2;
                Fix {
                    description: "Remove extra trailing newlines".to_string(),
                    replacement: Some("\n".to_string()),
                    start: Position {
                        line: start_line,
                        column: 1,
                    },
                    end: Position {
                        line: line_count + 1,
                        column: 1,
                    },
                }
            };

            violations.push(self.create_violation_with_fix(
                message,
                line_number,
                1,
                Severity::Warning,
                fix,
            ));
        }

        Ok(violations)
    }

    fn can_fix(&self) -> bool {
        true
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
    fn test_md047_single_newline_valid() {
        let content = "# Heading\n\nSome content here.\n";
        let document = create_test_document(content);
        let rule = MD047;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md047_no_newline_invalid() {
        let content = "# Heading\n\nSome content here.";
        let document = create_test_document(content);
        let rule = MD047;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD047");
        assert!(
            violations[0]
                .message
                .contains("File should end with a single newline character")
        );

        // Check fix is present
        assert!(violations[0].fix.is_some());
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Add newline at end of file");
        assert_eq!(fix.replacement, Some("\n".to_string()));
    }

    #[test]
    fn test_md047_multiple_newlines_invalid() {
        let content = "# Heading\n\nSome content here.\n\n";
        let document = create_test_document(content);
        let rule = MD047;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD047");
        assert!(violations[0].message.contains("found 2 trailing newlines"));
    }

    #[test]
    fn test_md047_three_newlines_invalid() {
        let content = "# Heading\n\nSome content here.\n\n\n";
        let document = create_test_document(content);
        let rule = MD047;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("found 3 trailing newlines"));
    }

    #[test]
    fn test_md047_empty_file_invalid() {
        let content = "";
        let document = create_test_document(content);
        let rule = MD047;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("File should end with a single newline character")
        );
    }

    #[test]
    fn test_md047_only_newline_valid() {
        let content = "\n";
        let document = create_test_document(content);
        let rule = MD047;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md047_only_multiple_newlines_invalid() {
        let content = "\n\n";
        let document = create_test_document(content);
        let rule = MD047;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("found 2 trailing newlines"));
    }

    #[test]
    fn test_md047_content_with_final_newline_valid() {
        let content = "Line 1\nLine 2\nLine 3\n";
        let document = create_test_document(content);
        let rule = MD047;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md047_content_without_final_newline_invalid() {
        let content = "Line 1\nLine 2\nLine 3";
        let document = create_test_document(content);
        let rule = MD047;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3); // Should report on last line
    }

    #[test]
    fn test_md047_mixed_line_endings_with_newline_valid() {
        let content = "# Title\r\n\r\nContent here.\n";
        let document = create_test_document(content);
        let rule = MD047;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md047_single_line_with_newline_valid() {
        let content = "Single line\n";
        let document = create_test_document(content);
        let rule = MD047;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md047_single_line_without_newline_invalid() {
        let content = "Single line";
        let document = create_test_document(content);
        let rule = MD047;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 1);
    }
}
