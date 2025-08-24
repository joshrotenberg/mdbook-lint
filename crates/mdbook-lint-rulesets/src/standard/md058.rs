//! MD058: Tables should be surrounded by blank lines
//!
//! This rule checks that tables are surrounded by blank lines for better readability.

use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};

/// Rule to check that tables are surrounded by blank lines
pub struct MD058;

impl MD058 {
    /// Get line and column position for a node
    fn get_position<'a>(&self, node: &'a AstNode<'a>) -> (usize, usize) {
        let data = node.data.borrow();
        let pos = data.sourcepos;
        (pos.start.line, pos.start.column)
    }

    /// Check if a line is blank (empty or whitespace only)
    fn is_blank_line(&self, line: &str) -> bool {
        line.trim().is_empty()
    }

    /// Walk AST and find all table violations
    fn check_node<'a>(
        &self,
        node: &'a AstNode<'a>,
        violations: &mut Vec<Violation>,
        document: &Document,
    ) {
        if let NodeValue::Table(_) = &node.data.borrow().value {
            let (start_line, _) = self.get_position(node);
            let lines: Vec<&str> = document.content.lines().collect();

            // Find all table segments within this AST node
            let table_segments = self.find_table_segments(start_line, &lines);

            for (segment_start, segment_end) in table_segments {
                // Check line before table segment (if not at start of document)
                if segment_start > 1 {
                    let line_before_idx = segment_start - 2; // Convert to 0-based and go back one line
                    if line_before_idx < lines.len() && !self.is_blank_line(lines[line_before_idx])
                    {
                        violations.push(self.create_violation(
                            "Tables should be preceded by a blank line".to_string(),
                            segment_start,
                            1,
                            Severity::Warning,
                        ));
                    }
                }

                // Check line after table segment (if not at end of document)
                if segment_end < lines.len() {
                    let line_after_idx = segment_end; // segment_end is 1-based, so this gets the line after
                    if line_after_idx < lines.len() {
                        let line_after = lines[line_after_idx];
                        if !self.is_blank_line(line_after) {
                            violations.push(self.create_violation(
                                "Tables should be followed by a blank line".to_string(),
                                segment_end + 1, // Report on the line after the table
                                1,
                                Severity::Warning,
                            ));
                        }
                    }
                }
            }
        }

        // Recursively check children
        // Continue walking through child nodes
        for child in node.children() {
            self.check_node(child, violations, document);
        }
    }

    /// Find all table segments within a potentially combined table structure
    fn find_table_segments(&self, start_line: usize, lines: &[&str]) -> Vec<(usize, usize)> {
        let mut segments = Vec::new();
        let mut current_line = start_line - 1; // Convert to 0-based

        while current_line < lines.len() {
            let line = lines[current_line].trim();

            // Skip until we find a table-like line
            if !line.contains('|') {
                current_line += 1;
                continue;
            }

            // Found start of a table segment
            let segment_start = current_line + 1; // Convert back to 1-based

            // Find end of this table segment
            while current_line < lines.len() {
                let line = lines[current_line].trim();

                if line.contains('|') {
                    // Check if it's a table separator
                    if line
                        .chars()
                        .all(|c| c == '|' || c == '-' || c == ':' || c.is_whitespace())
                    {
                        current_line += 1;
                        continue;
                    }

                    // Check if it looks like a table row
                    let pipe_count = line.chars().filter(|&c| c == '|').count();
                    if pipe_count >= 1 {
                        current_line += 1;
                        continue;
                    }
                }

                // This line is not part of the table
                break;
            }

            let segment_end = current_line; // This is 1-based line number after the table
            segments.push((segment_start, segment_end));

            // Look for more table segments after non-table content
            while current_line < lines.len() {
                let line = lines[current_line].trim();
                if line.contains('|') {
                    break; // Found another potential table segment
                }
                if line.is_empty() {
                    break; // Blank line likely separates table segments
                }
                current_line += 1;
            }
        }

        segments
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
        let content = r#"First table:
| Table 1  | Column 2 |
|----------|----------|
| Value 1  | Value 2  |
Second table immediately after:
| Table 2  | Column 2 |
|----------|----------|
| Value 3  | Value 4  |
End text.
"#;

        let document = create_test_document(content);
        let rule = MD058;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 4); // Both tables missing before and after blanks
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
}
