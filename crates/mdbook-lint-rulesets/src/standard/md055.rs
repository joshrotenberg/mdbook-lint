//! MD055: Table pipe style
//!
//! This rule checks that table pipes are used consistently throughout the document.

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};

/// Rule to check table pipe style consistency
pub struct MD055 {
    /// Preferred table pipe style
    style: PipeStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PipeStyle {
    /// No leading or trailing pipes
    NoLeadingOrTrailing,
    /// Leading and trailing pipes
    LeadingAndTrailing,
    /// Detect from first usage in document
    Consistent,
}

impl MD055 {
    /// Create a new MD055 rule with consistent style detection
    pub fn new() -> Self {
        Self {
            style: PipeStyle::Consistent,
        }
    }

    /// Create a new MD055 rule with specific style preference
    #[allow(dead_code)]
    pub fn with_style(style: PipeStyle) -> Self {
        Self { style }
    }

    /// Find table blocks in the document (sequences of table-like lines)
    fn find_table_blocks(&self, lines: &[&str]) -> Vec<(usize, usize)> {
        let mut table_blocks = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            if let Some(block_end) = self.find_table_block_starting_at(lines, i) {
                table_blocks.push((i, block_end));
                i = block_end + 1;
            } else {
                i += 1;
            }
        }

        table_blocks
    }

    /// Try to find a table block starting at the given line index
    fn find_table_block_starting_at(&self, lines: &[&str], start: usize) -> Option<usize> {
        if start >= lines.len() {
            return None;
        }

        let first_line = lines[start].trim();

        // Must start with a line that has pipes
        if !first_line.contains('|') {
            return None;
        }

        // Look for table patterns:
        // 1. Lines with leading/trailing pipes
        // 2. A header row followed by a separator row
        let has_leading_trailing = first_line.starts_with('|') && first_line.ends_with('|');

        if has_leading_trailing {
            // Find consecutive table lines (including separators and mixed styles)
            let mut end = start;
            while end < lines.len() {
                let line = lines[end].trim();
                if line.is_empty() {
                    break;
                }
                // Accept lines with pipes (including separators and mixed styles)
                if !line.contains('|') {
                    break;
                }
                end += 1;
            }

            if end > start {
                return Some(end - 1);
            }
        } else {
            // Look for header + separator pattern for tables without leading/trailing pipes
            if start + 1 < lines.len() {
                let second_line = lines[start + 1].trim();
                if self.is_table_separator(second_line) {
                    // Find consecutive table rows
                    let mut end = start + 1; // Include separator
                    end += 1; // Move past separator

                    while end < lines.len() {
                        let line = lines[end].trim();
                        if line.is_empty() {
                            break;
                        }
                        // Check if this looks like a table row without leading/trailing pipes
                        let pipe_count = line.chars().filter(|&c| c == '|').count();
                        if pipe_count == 0 || self.is_table_separator(line) {
                            break;
                        }
                        // Make sure it has the same number of columns as the header
                        let header_pipes = first_line.chars().filter(|&c| c == '|').count();
                        let row_pipes = line.chars().filter(|&c| c == '|').count();
                        if row_pipes != header_pipes {
                            break;
                        }
                        end += 1;
                    }

                    if end > start + 2 {
                        // At least header + separator + one data row
                        return Some(end - 1);
                    }
                }
            }
        }

        None
    }

    /// Check if a line looks like a table row within a known table context
    fn is_table_row_in_context(&self, line: &str) -> bool {
        let trimmed = line.trim();
        let pipe_count = trimmed.chars().filter(|&c| c == '|').count();
        pipe_count >= 1 && !self.is_table_separator(trimmed)
    }

    /// Check if a line is a table separator (like |---|---|)
    fn is_table_separator(&self, line: &str) -> bool {
        let trimmed = line.trim();
        if !trimmed.contains('|') {
            return false;
        }

        // Remove pipes and check if remaining chars are only - : and whitespace
        let without_pipes = trimmed.replace('|', "");
        without_pipes
            .chars()
            .all(|c| c == '-' || c == ':' || c.is_whitespace())
    }

    /// Determine the pipe style of a table row
    fn get_pipe_style(&self, line: &str) -> Option<PipeStyle> {
        let trimmed = line.trim();

        if !self.is_table_row_in_context(line) {
            return None;
        }

        let starts_with_pipe = trimmed.starts_with('|');
        let ends_with_pipe = trimmed.ends_with('|');

        if starts_with_pipe && ends_with_pipe {
            Some(PipeStyle::LeadingAndTrailing)
        } else if !starts_with_pipe && !ends_with_pipe {
            Some(PipeStyle::NoLeadingOrTrailing)
        } else {
            // Mixed style (leading but not trailing, or trailing but not leading)
            // We'll treat this as inconsistent and flag it
            None
        }
    }

    /// Check a line for table pipe style violations
    fn check_line_pipes(
        &self,
        line: &str,
        line_number: usize,
        expected_style: Option<PipeStyle>,
    ) -> (Vec<Violation>, Option<PipeStyle>) {
        let mut violations = Vec::new();
        let mut detected_style = expected_style;

        if let Some(current_style) = self.get_pipe_style(line) {
            if let Some(expected) = expected_style {
                // Check consistency with established style
                if expected != current_style {
                    let expected_desc = match expected {
                        PipeStyle::LeadingAndTrailing => "leading and trailing pipes",
                        PipeStyle::NoLeadingOrTrailing => "no leading or trailing pipes",
                        PipeStyle::Consistent => "consistent", // shouldn't happen
                    };
                    let found_desc = match current_style {
                        PipeStyle::LeadingAndTrailing => "leading and trailing pipes",
                        PipeStyle::NoLeadingOrTrailing => "no leading or trailing pipes",
                        PipeStyle::Consistent => "consistent", // shouldn't happen
                    };

                    violations.push(self.create_violation(
                        format!(
                            "Table pipe style inconsistent - expected {expected_desc} but found {found_desc}"
                        ),
                        line_number,
                        1,
                        Severity::Warning,
                    ));
                }
            } else {
                // First table found - establish the style
                detected_style = Some(current_style);
            }
        } else if self.is_table_row_in_context(line) {
            // This is a table row but with mixed pipe style
            violations.push(self.create_violation(
                "Table row has inconsistent pipe style (mixed leading/trailing)".to_string(),
                line_number,
                1,
                Severity::Warning,
            ));
        }

        (violations, detected_style)
    }

    /// Get code block ranges to exclude from checking
    fn get_code_block_ranges(&self, lines: &[&str]) -> Vec<bool> {
        let mut in_code_block = vec![false; lines.len()];
        let mut in_fenced_block = false;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Check for fenced code blocks
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_fenced_block = !in_fenced_block;
                in_code_block[i] = true;
                continue;
            }

            if in_fenced_block {
                in_code_block[i] = true;
                continue;
            }
        }

        in_code_block
    }
}

impl Default for MD055 {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for MD055 {
    fn id(&self) -> &'static str {
        "MD055"
    }

    fn name(&self) -> &'static str {
        "table-pipe-style"
    }

    fn description(&self) -> &'static str {
        "Table pipe style should be consistent"
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
        let lines: Vec<&str> = document.content.lines().collect();
        let in_code_block = self.get_code_block_ranges(&lines);

        // Find all table blocks first
        let table_blocks = self.find_table_blocks(&lines);

        let mut expected_style = match self.style {
            PipeStyle::LeadingAndTrailing => Some(PipeStyle::LeadingAndTrailing),
            PipeStyle::NoLeadingOrTrailing => Some(PipeStyle::NoLeadingOrTrailing),
            PipeStyle::Consistent => None, // Detect from first usage
        };

        // Process each table block
        for (start, end) in table_blocks {
            for line_idx in start..=end {
                let line_number = line_idx + 1;
                let line = lines[line_idx];

                // Skip lines inside code blocks
                if in_code_block[line_idx] {
                    continue;
                }

                // Only check actual table rows (not separators)
                if self.is_table_row_in_context(line) {
                    let (line_violations, detected_style) =
                        self.check_line_pipes(line, line_number, expected_style);
                    violations.extend(line_violations);

                    // Update expected style if we detected one
                    if expected_style.is_none() && detected_style.is_some() {
                        expected_style = detected_style;
                    }
                }
            }
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
    fn test_md055_consistent_leading_trailing_pipes() {
        let content = r#"| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Value 1  | Value 2  | Value 3  |
| Value 4  | Value 5  | Value 6  |
"#;

        let document = create_test_document(content);
        let rule = MD055::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md055_consistent_no_leading_trailing_pipes() {
        let content = r#"Column 1 | Column 2 | Column 3
---------|----------|----------
Value 1  | Value 2  | Value 3
Value 4  | Value 5  | Value 6
"#;

        let document = create_test_document(content);
        let rule = MD055::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md055_mixed_styles_violation() {
        let content = r#"| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
Value 1  | Value 2  | Value 3
| Value 4  | Value 5  | Value 6  |
"#;

        let document = create_test_document(content);
        let rule = MD055::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD055");
        assert_eq!(violations[0].line, 3);
        assert!(
            violations[0]
                .message
                .contains("expected leading and trailing pipes")
        );
    }

    #[test]
    fn test_md055_preferred_leading_trailing_style() {
        let content = r#"Column 1 | Column 2 | Column 3
---------|----------|----------
Value 1  | Value 2  | Value 3
"#;

        let document = create_test_document(content);
        let rule = MD055::with_style(PipeStyle::LeadingAndTrailing);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2); // Header and data rows
        assert!(
            violations[0]
                .message
                .contains("expected leading and trailing pipes")
        );
    }

    #[test]
    fn test_md055_preferred_no_leading_trailing_style() {
        let content = r#"| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Value 1  | Value 2  | Value 3  |
"#;

        let document = create_test_document(content);
        let rule = MD055::with_style(PipeStyle::NoLeadingOrTrailing);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2); // Header and data rows
        assert!(
            violations[0]
                .message
                .contains("expected no leading or trailing pipes")
        );
    }

    #[test]
    fn test_md055_mixed_leading_trailing_on_same_row() {
        let content = r#"| Column 1 | Column 2 | Column 3
|----------|----------|----------|
 Value 1  | Value 2  | Value 3  |
"#;

        let document = create_test_document(content);
        let rule = MD055::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("mixed leading/trailing"));
        assert!(violations[1].message.contains("mixed leading/trailing"));
    }

    #[test]
    fn test_md055_multiple_tables_consistent() {
        let content = r#"| Table 1  | Column 2 |
|----------|----------|
| Value 1  | Value 2  |

Some text between tables.

| Table 2  | Column 2 |
|----------|----------|
| Value 3  | Value 4  |
"#;

        let document = create_test_document(content);
        let rule = MD055::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md055_multiple_tables_inconsistent() {
        let content = r#"| Table 1  | Column 2 |
|----------|----------|
| Value 1  | Value 2  |

Some text between tables.

Table 2  | Column 2
---------|----------
Value 3  | Value 4
"#;

        let document = create_test_document(content);
        let rule = MD055::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2); // Second table has different style
        assert_eq!(violations[0].line, 7);
        assert_eq!(violations[1].line, 9);
    }

    #[test]
    fn test_md055_code_blocks_ignored() {
        let content = r#"| Good table | Column 2 |
|-------------|----------|
| Value 1     | Value 2  |

```
Bad table | Column 2
----------|----------
Value 3   | Value 4
```

| Another good | Column 2 |
|--------------|----------|
| Value 5      | Value 6  |
"#;

        let document = create_test_document(content);
        let rule = MD055::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md055_non_table_content_ignored() {
        let content = r#"This is regular text with | pipes | in it.

| But this | is a table |
|----------|------------|
| Value 1  | Value 2    |

And this is more text with | random | pipes |.
"#;

        let document = create_test_document(content);
        let rule = MD055::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md055_table_separators_ignored() {
        let content = r#"| Column 1 | Column 2 |
|:---------|----------:|
| Value 1  | Value 2   |
"#;

        let document = create_test_document(content);
        let rule = MD055::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md055_complex_table_separators() {
        let content = r#"| Left | Center | Right |
|:-----|:------:|------:|
| L1   | C1     | R1    |
| L2   | C2     | R2    |
"#;

        let document = create_test_document(content);
        let rule = MD055::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }
}
