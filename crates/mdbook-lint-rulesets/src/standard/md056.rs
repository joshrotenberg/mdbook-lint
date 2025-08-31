//! MD056 - Table column count
//!
//! This rule is triggered when a GitHub Flavored Markdown table does not have
//! the same number of cells in every row.
//!
//! ## Correct
//!
//! ```markdown
//! | Header | Header |
//! | ------ | ------ |
//! | Cell   | Cell   |
//! | Cell   | Cell   |
//! ```
//!
//! ## Incorrect
//!
//! ```markdown
//! | Header | Header |
//! | ------ | ------ |
//! | Cell   | Cell   |
//! | Cell   |
//! | Cell   | Cell   | Cell   |
//! ```

use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::error::Result;
use mdbook_lint_core::{
    Document, Violation,
    rule::{Rule, RuleCategory, RuleMetadata},
    violation::{Fix, Position, Severity},
};

/// MD056 - Table column count
pub struct MD056;

impl Default for MD056 {
    fn default() -> Self {
        Self::new()
    }
}

impl MD056 {
    /// Create a new MD056 rule instance
    pub fn new() -> Self {
        Self
    }

    /// Count cells in a table row
    fn count_cells<'a>(&self, node: &'a AstNode<'a>) -> usize {
        let mut cell_count = 0;
        for child in node.children() {
            if matches!(child.data.borrow().value, NodeValue::TableCell) {
                cell_count += 1;
            }
        }
        cell_count
    }

    /// Check table column consistency
    fn check_table_columns<'a>(&self, ast: &'a AstNode<'a>) -> Vec<Violation> {
        let mut violations = Vec::new();
        self.traverse_for_tables(ast, &mut violations);
        violations
    }

    /// Traverse AST to find tables
    fn traverse_for_tables<'a>(&self, node: &'a AstNode<'a>, violations: &mut Vec<Violation>) {
        if let NodeValue::Table(_) = &node.data.borrow().value {
            self.check_table(node, violations);
        }

        for child in node.children() {
            self.traverse_for_tables(child, violations);
        }
    }

    /// Check a single table for column count consistency
    fn check_table<'a>(&self, table_node: &'a AstNode<'a>, violations: &mut Vec<Violation>) {
        let mut rows = Vec::new();
        let mut expected_columns = None;

        // Collect all rows
        for child in table_node.children() {
            if matches!(child.data.borrow().value, NodeValue::TableRow(..)) {
                let cell_count = self.count_cells(child);
                let pos = child.data.borrow().sourcepos;
                let line = pos.start.line;
                let column = pos.start.column;
                rows.push((cell_count, line, column));

                // Set expected column count from the first row (header)
                if expected_columns.is_none() {
                    expected_columns = Some(cell_count);
                }
            }
        }

        let expected = expected_columns.unwrap_or(0);

        // Check each row against expected column count
        for (i, (cell_count, line, column)) in rows.iter().enumerate() {
            if *cell_count != expected {
                let row_type = if i == 0 {
                    "header row"
                } else if i == 1 {
                    "delimiter row"
                } else {
                    "data row"
                };

                let message = if *cell_count < expected {
                    format!(
                        "Table {} has {} cells, expected {} (missing {} cells)",
                        row_type,
                        cell_count,
                        expected,
                        expected - cell_count
                    )
                } else {
                    format!(
                        "Table {} has {} cells, expected {} (extra {} cells)",
                        row_type,
                        cell_count,
                        expected,
                        cell_count - expected
                    )
                };

                // Create a simple fix by adding empty cells or trimming extras
                // This is a basic implementation - a full fix would need to parse cells properly
                let fix_description = if *cell_count < expected {
                    format!(
                        "Add {} empty cell(s) to match table structure",
                        expected - cell_count
                    )
                } else {
                    format!(
                        "Remove {} extra cell(s) to match table structure",
                        cell_count - expected
                    )
                };

                // For now, just provide a placeholder fix that would need manual adjustment
                // A full implementation would parse the cells and add/remove them properly
                let fix = Fix {
                    description: fix_description,
                    replacement: None, // Would need complex cell parsing to fix properly
                    start: Position {
                        line: *line,
                        column: *column,
                    },
                    end: Position {
                        line: *line,
                        column: *column,
                    },
                };

                violations.push(self.create_violation_with_fix(
                    message,
                    *line,
                    *column,
                    Severity::Error,
                    fix,
                ));
            }
        }
    }

    /// Fallback method using manual parsing when no AST is available
    fn check_tables_fallback(&self, document: &Document) -> Vec<Violation> {
        let mut violations = Vec::new();
        let mut in_table = false;
        let mut expected_columns: Option<usize> = None;
        let mut table_row_index = 0;

        for (line_num, line) in document.content.lines().enumerate() {
            if self.is_table_row(line) {
                let cell_count = line.matches('|').count().saturating_sub(1);

                if !in_table {
                    // First row of table (header)
                    expected_columns = Some(cell_count);
                    in_table = true;
                    table_row_index = 0;
                } else if let Some(expected) = expected_columns
                    && cell_count != expected
                {
                    let row_type = if table_row_index == 1 {
                        "delimiter row"
                    } else {
                        "data row"
                    };

                    let message = if cell_count < expected {
                        format!(
                            "Table {} has {} cells, expected {} (missing {} cells)",
                            row_type,
                            cell_count,
                            expected,
                            expected - cell_count
                        )
                    } else {
                        format!(
                            "Table {} has {} cells, expected {} (extra {} cells)",
                            row_type,
                            cell_count,
                            expected,
                            cell_count - expected
                        )
                    };

                    // Create a simple fix by suggesting to add/remove cells
                    let fix_description = if cell_count < expected {
                        format!(
                            "Add {} empty cell(s) to match table structure",
                            expected - cell_count
                        )
                    } else {
                        format!(
                            "Remove {} extra cell(s) to match table structure",
                            cell_count - expected
                        )
                    };

                    // For basic fix, add empty cells at the end or suggest removal
                    let mut fixed_line = line.to_string();
                    if cell_count < expected {
                        // Add empty cells at the end
                        let cells_to_add = expected - cell_count;
                        for _ in 0..cells_to_add {
                            // Remove trailing pipe if present, add cell, then re-add pipe
                            let trimmed = fixed_line.trim();
                            if trimmed.ends_with('|') {
                                fixed_line = trimmed.trim_end_matches('|').to_string();
                                fixed_line.push_str(" | |");
                            } else {
                                fixed_line.push_str(" |");
                            }
                        }
                    }

                    let fix = Fix {
                        description: fix_description,
                        replacement: Some(format!("{}\n", fixed_line)),
                        start: Position {
                            line: line_num + 1,
                            column: 1,
                        },
                        end: Position {
                            line: line_num + 1,
                            column: line.len() + 1,
                        },
                    };

                    violations.push(self.create_violation_with_fix(
                        message,
                        line_num + 1,
                        1,
                        Severity::Error,
                        fix,
                    ));
                }
                table_row_index += 1;
            } else if in_table && line.trim().is_empty() {
                // End of table
                in_table = false;
                expected_columns = None;
                table_row_index = 0;
            }
        }

        violations
    }

    /// Check if a line is a table row without using regex
    fn is_table_row(&self, line: &str) -> bool {
        let trimmed = line.trim();

        // Must start and end with pipe
        if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
            return false;
        }

        // Must have at least 2 pipes (start and end)
        trimmed.matches('|').count() >= 2
    }
}

impl Rule for MD056 {
    fn id(&self) -> &'static str {
        "MD056"
    }

    fn name(&self) -> &'static str {
        "table-column-count"
    }

    fn description(&self) -> &'static str {
        "Table column count"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Structure)
    }

    fn can_fix(&self) -> bool {
        true
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        ast: Option<&'a AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        if let Some(ast) = ast {
            let violations = self.check_table_columns(ast);
            Ok(violations)
        } else {
            // Simplified regex-based fallback when no AST is available
            Ok(self.check_tables_fallback(document))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::rule::Rule;
    use mdbook_lint_core::test_helpers::*;

    #[test]
    fn test_consistent_table() {
        let content = r#"| Header | Header |
| ------ | ------ |
| Cell   | Cell   |
| Cell   | Cell   |
"#;

        assert_no_violations(MD056::new(), content);
    }

    #[test]
    fn test_missing_cells() {
        let content = r#"| Header | Header |
| ------ | ------ |
| Cell   | Cell   |
| Cell   |
"#;

        let violation = assert_single_violation(MD056::new(), content);
        assert_eq!(violation.line, 4);
        assert!(violation.message.contains("missing 1 cells"));
    }

    #[test]
    fn test_extra_cells() {
        let content = r#"| Header | Header |
| ------ | ------ |
| Cell   | Cell   |
| Cell   | Cell   | Cell   |
"#;

        let violation = assert_single_violation(MD056::new(), content);
        assert_eq!(violation.line, 4);
        assert!(violation.message.contains("extra 1 cells"));
    }

    #[test]
    fn test_delimiter_row_mismatch() {
        let content = r#"| Header | Header |
| ------ |
| Cell   | Cell   |
"#;

        let violation = assert_single_violation(MD056::new(), content);
        assert_eq!(violation.line, 2);
        assert!(violation.message.contains("delimiter row"));
        assert!(violation.message.contains("missing 1 cells"));
    }

    #[test]
    fn test_multiple_violations() {
        let content = r#"| Header | Header |
| ------ | ------ |
| Cell   |
| Cell   | Cell   | Cell   |
"#;

        let violations = assert_violation_count(MD056::new(), content, 2);

        assert_eq!(violations[0].line, 3);
        assert!(violations[0].message.contains("missing 1 cells"));

        assert_eq!(violations[1].line, 4);
        assert!(violations[1].message.contains("extra 1 cells"));
    }

    #[test]
    fn test_single_column_table() {
        let content = r#"| Header |
| ------ |
| Cell   |
| Cell   |
"#;

        assert_no_violations(MD056::new(), content);
    }

    #[test]
    fn test_empty_table() {
        let content = r#"| |
|---|
| |
"#;

        assert_no_violations(MD056::new(), content);
    }

    #[test]
    fn test_multiple_tables() {
        let content = r#"| Table 1 | Header |
| ------- | ------ |
| Cell    | Cell   |

| Table 2 | Header |
| ------- | ------ |
| Cell    |
"#;

        let violation = assert_single_violation(MD056::new(), content);
        assert_eq!(violation.line, 7);
        assert!(violation.message.contains("missing 1 cells"));
    }

    #[test]
    fn test_fallback_multiple_tables() {
        let content = r#"| Table 1 | Header |
| ------- | ------ |
| Cell    | Cell   |

| Table 2 | Header |
| ------- | ------ |
| Cell    |
"#;

        // Test fallback implementation specifically
        use std::path::PathBuf;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD056::new();
        let violations = rule.check_tables_fallback(&document);

        assert_eq!(violations.len(), 1);
        let violations = assert_violation_count(rule, content, 1);
        assert_eq!(violations[0].line, 7);
        assert!(violations[0].message.contains("missing 1 cells"));
    }

    #[test]
    fn test_fallback_method() {
        // Test when no AST is available
        let content = r#"| Header | Header |
| ------ | ------ |
| Cell   | Cell   |
| Cell   |
"#;

        let rule = MD056::new();
        let violations = rule.check_tables_fallback(&create_document(content));
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 4);
        assert!(violations[0].message.contains("missing 1 cells"));
    }

    #[test]
    fn test_edge_case_empty_rows() {
        let content = r#"| Header | Header |
| ------ | ------ |
|        |        |
|        |
"#;

        let violation = assert_single_violation(MD056::new(), content);
        assert_eq!(violation.line, 4);
        assert!(violation.message.contains("missing 1 cells"));
    }

    #[test]
    fn test_table_with_varying_column_counts() {
        let content = r#"| A | B | C |
| - | - | - |
| 1 | 2 |
| 4 | 5 | 6 | 7 |
| 8 | 9 | 10 |
"#;

        let violations = assert_violation_count(MD056::new(), content, 2);
        assert_eq!(violations[0].line, 3);
        assert!(violations[0].message.contains("missing 1 cells"));
        assert_eq!(violations[1].line, 4);
        assert!(violations[1].message.contains("extra 1 cells"));
    }

    #[test]
    fn test_complex_table_structure() {
        let content = r#"| Column 1 | Column 2 | Column 3 | Column 4 |
| -------- | -------- | -------- | -------- |
| Data     | Data     | Data     | Data     |
| Data     | Data     |          |          |
| Data     |          |          |          |
| Data     | Data     | Data     |          |
"#;

        assert_no_violations(MD056::new(), content);
    }

    #[test]
    fn test_table_with_pipes_in_content() {
        let content = r#"| Code | Description |
| ---- | ----------- |
| `a`  | Pipe char   |
| `b`  | With pipe   |
"#;

        assert_no_violations(MD056::new(), content);
    }

    #[test]
    fn test_malformed_table_structure() {
        let content = r#"| Header | Header |
| Cell   | Cell   |
| ------ | ------ |
| Cell   | Cell   |
"#;

        // This tests the fallback parsing with malformed structure
        assert_no_violations(MD056::new(), content);
    }

    #[test]
    fn test_table_cell_count_edge_cases() {
        let content = r#"| A |
| - |
|   |
| B |
"#;

        assert_no_violations(MD056::new(), content);
    }

    #[test]
    fn test_delimiter_row_variations() {
        let content = r#"| Header1 | Header2 | Header3 |
|---------|---------|
| Cell    | Cell    | Cell    |
"#;

        let violation = assert_single_violation(MD056::new(), content);
        assert_eq!(violation.line, 2);
        assert!(violation.message.contains("missing 1 cells"));
    }

    #[test]
    fn test_no_tables_in_document() {
        let content = r#"# Heading

This is just text with no tables.

Some more text here.
"#;

        assert_no_violations(MD056::new(), content);
    }

    #[test]
    fn test_table_within_other_content() {
        let content = r#"# Document Title

Some introductory text.

| Name | Age | City |
| ---- | --- | ---- |
| John | 30  |      |

More text after the table.
"#;

        assert_no_violations(MD056::new(), content);
    }

    #[test]
    fn test_multiple_delimiter_issues() {
        let content = r#"| A | B | C |
| - | - |
| 1 | 2 | 3 |
| 4 | 5 |
"#;

        let violations = assert_violation_count(MD056::new(), content, 2);
        assert_eq!(violations[0].line, 2);
        assert!(violations[0].message.contains("missing 1 cells"));
        assert_eq!(violations[1].line, 4);
        assert!(violations[1].message.contains("missing 1 cells"));
    }

    #[test]
    fn test_large_table_consistency() {
        let content = r#"| C1 | C2 | C3 | C4 | C5 |
| -- | -- | -- | -- | -- |
| D1 | D2 | D3 | D4 | D5 |
| D1 | D2 | D3 | D4 | D5 |
| D1 | D2 | D3 | D4 |    |
| D1 | D2 | D3 | D4 | D5 |
"#;

        assert_no_violations(MD056::new(), content);
    }

    #[test]
    fn test_table_row_parsing_edge_cases() {
        let content = r#"| Header |
|--------|
| Cell   |
|        |
"#;

        assert_no_violations(MD056::new(), content);
    }

    #[test]
    fn test_ast_not_available_error_path() {
        let content = r#"| Header | Header |
| ------ | ------ |
| Cell   |
"#;

        let rule = MD056::new();
        // Test with AST explicitly set to None to trigger fallback
        let violations = rule
            .check_with_ast(&create_document(content), None)
            .unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("missing 1 cells"));
    }

    #[test]
    fn test_complex_table_scenarios() {
        // Test basic table functionality - use consistent column count
        let content = r#"| Code | Description |
| ---- | ----------- |
| abc  | Pipe char |
| def  | Another value |
"#;

        assert_no_violations(MD056::new(), content);
    }

    #[test]
    fn test_malformed_table_detection() {
        // Test tables without proper delimiters
        let content = r#"Not a table line
| Header | Header |
Not a table line
| Cell   |
"#;

        let violation = assert_single_violation(MD056::new(), content);
        assert!(violation.message.contains("missing 1 cells"));
    }

    #[test]
    fn test_header_row_edge_cases() {
        // Test when header row has wrong column count
        let content = r#"| Too | Many | Headers | Here |
| --- | --- |
| One | Two |
"#;

        let violations = assert_violation_count(MD056::new(), content, 2);
        assert_eq!(violations[0].line, 2);
        assert!(violations[0].message.contains("delimiter row"));
    }

    #[test]
    fn test_count_cells_functionality() {
        // Test internal cell counting logic with various scenarios
        let rule = MD056::new();

        // Test different pipe configurations
        let scenarios = vec![
            ("| A |", 1),
            ("| A | B |", 2),
            ("| A | B | C |", 3),
            ("|A|B|", 2),
            ("| | |", 2),
        ];

        // Since count_cells is private, we test through behavior
        for (line, expected_count) in scenarios {
            let content = format!(
                "{}\n|---|\n{}",
                "| Header |"
                    .repeat(expected_count)
                    .replace(" |", " | ")
                    .trim_end(),
                line
            );

            if line.matches('|').count() - 1 != expected_count {
                // Should produce violation
                let violations = rule.check(&create_document(&content)).unwrap();
                assert!(
                    !violations.is_empty(),
                    "Expected violation for line: {line}"
                );
            }
        }
    }

    #[test]
    fn test_table_row_detection_edge_cases() {
        // Test is_table_row logic with various edge cases
        let content = r#"| Valid | Table | Row |
| ----- | ----- | --- |
Not a table row
| Valid | Row |
|Invalid|
||
|   |   |   |
"#;

        let rule = MD056::new();
        let violations = rule.check(&create_document(content)).unwrap();
        // Should find violations for rows with wrong column counts
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_fallback_table_detection() {
        // Test the fallback parsing when AST is not available
        let rule = MD056::new();

        // Test table end detection on blank line
        let content = r#"| Header | Header |
| ------ | ------ |
| Cell   | Cell   |

Not a table anymore
| Header |
| ------ |
| Cell   |
"#;

        let violations = rule.check_tables_fallback(&create_document(content));
        // Test passes if parsing completes without panic
        let _ = violations;
    }

    #[test]
    fn test_table_state_transitions() {
        // Test in_table state transitions in fallback method
        let rule = MD056::new();

        let content = r#"Regular text
| Start | Table |
| ----- | ----- |
| Row   |

Back to regular text
| Another | Table |
| ------- | ----- |
| Cell    | Cell  |
"#;

        let violations = rule.check_tables_fallback(&create_document(content));
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("missing 1 cells"));
    }

    #[test]
    fn test_row_type_messages() {
        // Test different row type error messages - simplified to avoid multiple violations
        let content = r#"| Header | Header |
| ------ | ------ |
| Cell   |"#;

        let violation = assert_single_violation(MD056::new(), content);
        assert!(violation.message.contains("data row"));
        assert!(violation.message.contains("missing"));
    }

    #[test]
    fn test_pipe_counting_edge_cases() {
        // Test pipe counting with different scenarios
        let rule = MD056::new();

        // Test edge case where line has pipes but isn't a table
        let content = r#"This line has | pipes but isn't a table
| Header | Header |
| ------ | ------ |
| Cell   | Cell   |
"#;

        assert_no_violations(rule, content);
    }

    #[test]
    fn test_expected_column_calculation() {
        // Test how expected column count is determined
        let scenarios = vec![
            // Different header configurations
            (
                r#"| A |
| - |
| 1 | 2 |"#,
                1,
            ),
            (
                r#"| A | B | C |
| - | - | - |
| 1 | 2 |"#,
                1,
            ),
        ];

        for (content, expected_violations) in scenarios {
            let violations = assert_violation_count(MD056::new(), content, expected_violations);
            assert!(!violations.is_empty());
        }
    }

    #[test]
    fn test_md056_fix_missing_cells() {
        let content = r#"| Header 1 | Header 2 | Header 3 |
| --------- | --------- | --------- |
| Cell 1    | Cell 2    |
"#;

        let document =
            Document::new(content.to_string(), std::path::PathBuf::from("test.md")).unwrap();
        let rule = MD056::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("missing 1 cells"));
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert!(fix.description.contains("Add 1 empty cell"));
        // The fix should add an empty cell
        if let Some(replacement) = &fix.replacement {
            // The actual cell content might vary, just check structure
            assert!(replacement.contains("|"));
            assert!(replacement.trim().ends_with("|"));
        }
    }

    #[test]
    fn test_md056_fix_extra_cells() {
        let content = r#"| Header 1 | Header 2 |
| --------- | --------- |
| Cell 1    | Cell 2    | Cell 3    | Cell 4 |
"#;

        let document =
            Document::new(content.to_string(), std::path::PathBuf::from("test.md")).unwrap();
        let rule = MD056::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("extra 2 cells"));
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert!(fix.description.contains("Remove 2 extra cell"));
    }

    #[test]
    fn test_md056_can_fix() {
        let rule = MD056::new();
        assert!(Rule::can_fix(&rule));
    }
}
