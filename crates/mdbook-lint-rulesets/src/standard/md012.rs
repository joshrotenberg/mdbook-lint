//! MD012: Multiple consecutive blank lines
//!
//! This rule checks for multiple consecutive blank lines in the document.

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Fix, Position, Severity, Violation},
};

/// Rule to check for multiple consecutive blank lines
pub struct MD012 {
    /// Maximum number of consecutive blank lines allowed
    maximum: usize,
}

impl MD012 {
    /// Create a new MD012 rule with default settings (max 1 blank line)
    pub fn new() -> Self {
        Self { maximum: 1 }
    }

    /// Create a new MD012 rule with custom maximum consecutive blank lines
    #[allow(dead_code)]
    pub fn with_maximum(maximum: usize) -> Self {
        Self { maximum }
    }
}

impl Default for MD012 {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for MD012 {
    fn id(&self) -> &'static str {
        "MD012"
    }

    fn name(&self) -> &'static str {
        "no-multiple-blanks"
    }

    fn description(&self) -> &'static str {
        "Multiple consecutive blank lines are not allowed"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Formatting).introduced_in("markdownlint v0.1.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let mut consecutive_blank_lines = 0;
        let mut blank_sequence_start = 0;

        for (line_number, line) in document.lines.iter().enumerate() {
            let line_num = line_number + 1; // Convert to 1-based line numbers

            if line.trim().is_empty() {
                if consecutive_blank_lines == 0 {
                    blank_sequence_start = line_num;
                }
                consecutive_blank_lines += 1;
            } else {
                // Non-blank line encountered, check if we had too many blank lines
                if consecutive_blank_lines > self.maximum {
                    // Calculate fix: keep only maximum allowed blank lines
                    let extra_lines = consecutive_blank_lines - self.maximum;
                    let fix_start_line = blank_sequence_start + self.maximum;
                    let fix_end_line = blank_sequence_start + consecutive_blank_lines - 1;

                    let fix = Fix {
                        description: format!(
                            "Remove {} extra blank line{}",
                            extra_lines,
                            if extra_lines == 1 { "" } else { "s" }
                        ),
                        replacement: Some(String::new()), // Delete the extra blank lines
                        start: Position {
                            line: fix_start_line,
                            column: 1,
                        },
                        end: Position {
                            line: fix_end_line + 1, // +1 to include the whole line
                            column: 1,
                        },
                    };

                    violations.push(self.create_violation_with_fix(
                        format!(
                            "Multiple consecutive blank lines ({} found, {} allowed)",
                            consecutive_blank_lines, self.maximum
                        ),
                        blank_sequence_start + self.maximum, // Report at the first violating line
                        1,
                        Severity::Warning,
                        fix,
                    ));
                }
                consecutive_blank_lines = 0;
            }
        }

        // Check if the document ends with too many blank lines
        if consecutive_blank_lines > self.maximum {
            let extra_lines = consecutive_blank_lines - self.maximum;
            let fix_start_line = blank_sequence_start + self.maximum;
            let fix_end_line = blank_sequence_start + consecutive_blank_lines - 1;

            let fix = Fix {
                description: format!(
                    "Remove {} extra blank line{} at end of file",
                    extra_lines,
                    if extra_lines == 1 { "" } else { "s" }
                ),
                replacement: Some(String::new()), // Delete the extra blank lines
                start: Position {
                    line: fix_start_line,
                    column: 1,
                },
                end: Position {
                    line: fix_end_line + 1,
                    column: 1,
                },
            };

            violations.push(self.create_violation_with_fix(
                format!(
                    "Multiple consecutive blank lines at end of file ({} found, {} allowed)",
                    consecutive_blank_lines, self.maximum
                ),
                blank_sequence_start + self.maximum,
                1,
                Severity::Warning,
                fix,
            ));
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::rule::Rule;
    use std::path::PathBuf;

    fn create_test_document(content: &str) -> Document {
        Document::new(content.to_string(), PathBuf::from("test.md")).unwrap()
    }

    #[test]
    fn test_md012_no_consecutive_blank_lines() {
        let content = "# Heading\n\nParagraph one.\n\nParagraph two.";
        let document = create_test_document(content);
        let rule = MD012::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md012_two_consecutive_blank_lines() {
        let content = "# Heading\n\n\nParagraph.";
        let document = create_test_document(content);
        let rule = MD012::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD012");
        assert_eq!(violations[0].line, 3); // The second blank line
        assert!(violations[0].message.contains("2 found, 1 allowed"));
    }

    #[test]
    fn test_md012_three_consecutive_blank_lines() {
        let content = "# Heading\n\n\n\nParagraph.";
        let document = create_test_document(content);
        let rule = MD012::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3); // First violating line
        assert!(violations[0].message.contains("3 found, 1 allowed"));
    }

    #[test]
    fn test_md012_multiple_violations() {
        let content = "# Heading\n\n\nParagraph.\n\n\n\nAnother paragraph.";
        let document = create_test_document(content);
        let rule = MD012::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 6);
    }

    #[test]
    fn test_md012_custom_maximum() {
        let content = "# Heading\n\n\nParagraph.";
        let document = create_test_document(content);
        let rule = MD012::with_maximum(2);
        let violations = rule.check(&document).unwrap();

        // Should allow 2 consecutive blank lines
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md012_custom_maximum_violation() {
        let content = "# Heading\n\n\n\nParagraph.";
        let document = create_test_document(content);
        let rule = MD012::with_maximum(2);
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("3 found, 2 allowed"));
    }

    #[test]
    fn test_md012_blank_lines_at_end() {
        let content = "# Heading\n\nParagraph.\n\n\n";
        let document = create_test_document(content);
        let rule = MD012::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("at end of file"));
    }

    #[test]
    fn test_md012_zero_maximum() {
        let content = "# Heading\n\nParagraph.";
        let document = create_test_document(content);
        let rule = MD012::with_maximum(0);
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("1 found, 0 allowed"));
    }

    #[test]
    fn test_md012_only_blank_lines() {
        let content = "\n\n\n";
        let document = create_test_document(content);
        let rule = MD012::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("at end of file"));
    }
}
