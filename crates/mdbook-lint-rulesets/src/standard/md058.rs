//! MD058: Tables should be surrounded by blank lines
//!
//! This rule checks that tables are surrounded by blank lines for better readability.

use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Fix, Position, Severity, Violation},
};

/// Rule to check that tables are surrounded by blank lines
pub struct MD058;

impl MD058 {
    /// Check if a line is blank (empty or whitespace only)
    fn is_blank_line(line: &str) -> bool {
        line.trim().is_empty()
    }

    /// Check if a line looks like a table row (header, separator, or data row)
    fn is_table_row(line: &str) -> bool {
        let trimmed = line.trim();
        if !trimmed.contains('|') {
            return false;
        }
        // Separator row: only |, -, :, and whitespace
        if trimmed
            .chars()
            .all(|c| c == '|' || c == '-' || c == ':' || c.is_whitespace())
        {
            return true;
        }
        // Content row: at least 2 pipes
        trimmed.chars().filter(|&c| c == '|').count() >= 2
    }

    /// Find the actual end of a table by scanning forward from start within AST bounds.
    /// Comrak's sourcepos.end may include trailing non-table content when there's no
    /// blank line after the table.
    fn find_actual_table_end(lines: &[&str], start_line: usize, ast_end_line: usize) -> usize {
        let mut actual_end = start_line;
        for line_num in start_line..=ast_end_line.min(lines.len()) {
            let line_idx = line_num - 1; // Convert to 0-based
            if line_idx < lines.len() && Self::is_table_row(lines[line_idx]) {
                actual_end = line_num;
            } else {
                break;
            }
        }
        actual_end
    }

    /// Walk AST and find all table violations
    fn check_node<'a>(
        &self,
        node: &'a AstNode<'a>,
        violations: &mut Vec<Violation>,
        document: &Document,
    ) {
        if let NodeValue::Table(_) = &node.data.borrow().value {
            let data = node.data.borrow();
            let start_line = data.sourcepos.start.line;
            let ast_end_line = data.sourcepos.end.line;
            drop(data); // Release borrow before accessing document

            let lines: Vec<&str> = document.content.lines().collect();

            // Find the actual end of the table (comrak may include following text)
            let end_line = Self::find_actual_table_end(&lines, start_line, ast_end_line);

            // Check line before table (if not at start of document)
            if start_line > 1 {
                let line_before_idx = start_line - 2; // Convert to 0-based and go back one line
                if line_before_idx < lines.len() && !Self::is_blank_line(lines[line_before_idx]) {
                    let fix = Fix {
                        description: "Add blank line before table".to_string(),
                        replacement: Some("\n".to_string()),
                        start: Position {
                            line: start_line - 1,
                            column: document.lines[start_line - 2].len() + 1,
                        },
                        end: Position {
                            line: start_line - 1,
                            column: document.lines[start_line - 2].len() + 1,
                        },
                    };

                    violations.push(self.create_violation_with_fix(
                        "Tables should be preceded by a blank line".to_string(),
                        start_line,
                        1,
                        Severity::Warning,
                        fix,
                    ));
                }
            }

            // Check line after table (if not at end of document)
            if end_line < lines.len() {
                let line_after_idx = end_line; // end_line is 1-based, so this is the 0-based index of next line
                if line_after_idx < lines.len() && !Self::is_blank_line(lines[line_after_idx]) {
                    let fix = Fix {
                        description: "Add blank line after table".to_string(),
                        replacement: Some("\n".to_string()),
                        start: Position {
                            line: end_line,
                            column: document.lines[end_line - 1].len() + 1,
                        },
                        end: Position {
                            line: end_line,
                            column: document.lines[end_line - 1].len() + 1,
                        },
                    };

                    violations.push(self.create_violation_with_fix(
                        "Tables should be followed by a blank line".to_string(),
                        end_line + 1,
                        1,
                        Severity::Warning,
                        fix,
                    ));
                }
            }
        }

        // Continue walking through child nodes
        for child in node.children() {
            self.check_node(child, violations, document);
        }
    }
}

impl AstRule for MD058 {
    fn id(&self) -> &'static str {
        "MD058"
    }

    fn name(&self) -> &'static str {
        "blanks-around-tables"
    }

    fn description(&self) -> &'static str {
        "Tables should be surrounded by blank lines"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Formatting).introduced_in("mdbook-lint v0.1.0")
    }

    fn can_fix(&self) -> bool {
        true
    }

    fn check_ast<'a>(&self, document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        self.check_node(ast, &mut violations, document);
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
    fn test_md058_tables_with_blank_lines_valid() {
        let content = r#"Here is some text.

| Column 1 | Column 2 |
|----------|----------|
| Value 1  | Value 2  |

More text after the table.
"#;

        let document = create_test_document(content);
        let rule = MD058;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md058_table_at_start_of_document() {
        let content = r#"| Column 1 | Column 2 |
|----------|----------|
| Value 1  | Value 2  |

Text after the table.
"#;

        let document = create_test_document(content);
        let rule = MD058;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md058_table_at_end_of_document() {
        let content = r#"Some text before.

| Column 1 | Column 2 |
|----------|----------|
| Value 1  | Value 2  |"#;

        let document = create_test_document(content);
        let rule = MD058;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md058_table_missing_blank_before() {
        let content = r#"Here is some text.
| Column 1 | Column 2 |
|----------|----------|
| Value 1  | Value 2  |

More text after.
"#;

        let document = create_test_document(content);
        let rule = MD058;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD058");
        assert!(violations[0].message.contains("preceded by a blank line"));
        assert_eq!(violations[0].line, 2);
    }

    #[test]
    fn test_md058_table_missing_blank_after() {
        let content = r#"Some text before.

| Column 1 | Column 2 |
|----------|----------|
| Value 1  | Value 2  |
More text after.
"#;

        let document = create_test_document(content);
        let rule = MD058;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("followed by a blank line"));
        assert_eq!(violations[0].line, 6);
    }

    #[test]
    fn test_md058_table_missing_both_blanks() {
        let content = r#"Text before.
| Column 1 | Column 2 |
|----------|----------|
| Value 1  | Value 2  |
Text after.
"#;

        let document = create_test_document(content);
        let rule = MD058;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("preceded by a blank line"));
        assert!(violations[1].message.contains("followed by a blank line"));
    }

    #[test]
    fn test_md058_multiple_tables() {
        let content = r#"First table with proper spacing:

| Table 1  | Column 2 |
|----------|----------|
| Value 1  | Value 2  |

Second table also with proper spacing:

| Table 2  | Column 2 |
|----------|----------|
| Value 3  | Value 4  |

End of document.
"#;

        let document = create_test_document(content);
        let rule = MD058;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md058_multiple_tables_violations() {
        // Two separate tables, each missing blank lines
        let content = r#"First table:
| Table 1  | Column 2 |
|----------|----------|
| Value 1  | Value 2  |

Second table missing blanks:
| Table 2  | Column 2 |
|----------|----------|
| Value 3  | Value 4  |
End text.
"#;

        let document = create_test_document(content);
        let rule = MD058;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 3); // Table 1: missing before; Table 2: missing before + after
    }

    #[test]
    fn test_md058_table_only_document() {
        let content = r#"| Column 1 | Column 2 |
|----------|----------|
| Value 1  | Value 2  |"#;

        let document = create_test_document(content);
        let rule = MD058;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0); // Table at start and end of document is OK
    }

    #[test]
    fn test_md058_tables_with_different_content() {
        let content = r#"# Heading before table
| Column 1 | Column 2 |
|----------|----------|
| Value 1  | Value 2  |

## Heading after table

Some paragraph.

| Another | Table |
|---------|-------|
| More    | Data  |

- List item after table
"#;

        let document = create_test_document(content);
        let rule = MD058;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1); // Only first table missing blank before
        assert!(violations[0].message.contains("preceded by a blank line"));
    }

    #[test]
    fn test_md058_complex_table() {
        let content = r#"Text before.

| Left | Center | Right | Numbers |
|:-----|:------:|------:|--------:|
| L1   | C1     | R1    | 123     |
| L2   | C2     | R2    | 456     |
| L3   | C3     | R3    | 789     |

Text after.
"#;

        let document = create_test_document(content);
        let rule = MD058;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md058_table_with_empty_cells() {
        let content = r#"Before text.

| Col1 | Col2 | Col3 |
|------|------|------|
| A    |      | C    |
|      | B    |      |
| X    | Y    | Z    |

After text.
"#;

        let document = create_test_document(content);
        let rule = MD058;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md058_fix_missing_blank_before() {
        let content = r#"Here is some text.
| Column 1 | Column 2 |
|----------|----------|
| Value 1  | Value 2  |

More text after.
"#;

        let document = create_test_document(content);
        let rule = MD058;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Add blank line before table");
        assert_eq!(fix.replacement, Some("\n".to_string()));
    }

    #[test]
    fn test_md058_fix_missing_blank_after() {
        let content = r#"Some text before.

| Column 1 | Column 2 |
|----------|----------|
| Value 1  | Value 2  |
More text after.
"#;

        let document = create_test_document(content);
        let rule = MD058;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Add blank line after table");
        assert_eq!(fix.replacement, Some("\n".to_string()));
    }

    #[test]
    fn test_md058_fix_missing_both_blanks() {
        let content = r#"Text before.
| Column 1 | Column 2 |
|----------|----------|
| Value 1  | Value 2  |
Text after.
"#;

        let document = create_test_document(content);
        let rule = MD058;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);

        // Both violations should have fixes
        for violation in &violations {
            assert!(violation.fix.is_some());
            let fix = violation.fix.as_ref().unwrap();
            assert!(fix.description.contains("Add blank line"));
            assert_eq!(fix.replacement, Some("\n".to_string()));
        }
    }

    #[test]
    fn test_md058_can_fix() {
        let rule = MD058;
        assert!(AstRule::can_fix(&rule));
    }

    #[test]
    fn test_md058_no_false_positive_for_pipes_in_code_blocks() {
        // Regression test for issue #393
        let content = "# Chapter 1\n\n## First a section with a table\n\n| First | Second |\n|-------|--------|\n| first | second |\n\n## Then a text block\n\n```text\nTheType {\n  with_field: THIS | THAT\n}\n```\n";

        let document = create_test_document(content);
        let rule = MD058;
        let violations = rule.check(&document).unwrap();
        assert_eq!(
            violations.len(),
            0,
            "Should not flag pipes in code blocks as tables: {violations:?}"
        );
    }

    #[test]
    fn test_md058_no_false_positive_for_pipes_in_list_items() {
        // Regression test for issue #393
        let content = "# Heading\n\n| Column 1 | Column 2 |\n|----------|----------|\n| Value 1  | Value 2  |\n\n## List with pipes\n\n- Item with THIS | THAT pattern\n- Another item\n";

        let document = create_test_document(content);
        let rule = MD058;
        let violations = rule.check(&document).unwrap();
        assert_eq!(
            violations.len(),
            0,
            "Should not flag pipes in list items as tables: {violations:?}"
        );
    }
}
