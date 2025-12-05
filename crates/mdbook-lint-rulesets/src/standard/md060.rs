//! MD060: Table column style
//!
//! This rule checks that table column delimiter segments use a consistent style.
//! Styles include:
//! - `aligned` - leading/trailing spaces maintain column width alignment
//! - `compact` - no extra spaces around content
//! - `tight` - single-space padding around content

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};

/// Rule to check table column style consistency
pub struct MD060 {
    /// Preferred table column style
    style: ColumnStyle,
}

/// Table column style options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnStyle {
    /// Leading/trailing spaces maintain column width alignment
    Aligned,
    /// No extra spaces around content
    Compact,
    /// Single-space padding around content
    Tight,
    /// Any style is allowed (consistency not enforced)
    Any,
    /// Detect from first usage and enforce consistency
    Consistent,
}

impl MD060 {
    /// Create a new MD060 rule with consistent style detection
    pub fn new() -> Self {
        Self {
            style: ColumnStyle::Consistent,
        }
    }

    /// Create a new MD060 rule with specific style preference
    #[allow(dead_code)]
    pub fn with_style(style: ColumnStyle) -> Self {
        Self { style }
    }

    /// Create MD060 from configuration
    pub fn from_config(config: &toml::Value) -> Self {
        let mut rule = Self::new();

        if let Some(style_str) = config.get("style").and_then(|v| v.as_str()) {
            rule.style = match style_str.to_lowercase().as_str() {
                "aligned" => ColumnStyle::Aligned,
                "compact" => ColumnStyle::Compact,
                "tight" => ColumnStyle::Tight,
                "any" => ColumnStyle::Any,
                "consistent" => ColumnStyle::Consistent,
                _ => ColumnStyle::Consistent, // Default fallback
            };
        }

        rule
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
            }
        }

        in_code_block
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

    /// Find table blocks in the document
    fn find_table_blocks(&self, lines: &[&str], in_code_block: &[bool]) -> Vec<(usize, usize)> {
        let mut table_blocks = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            // Skip code blocks
            if in_code_block[i] {
                i += 1;
                continue;
            }

            // Look for a line with pipes that could be a table header
            let line = lines[i].trim();
            if !line.contains('|') {
                i += 1;
                continue;
            }

            // Check if next line is a separator (indicates valid table)
            if i + 1 < lines.len() && self.is_table_separator(lines[i + 1]) {
                let start = i;
                // Find the end of the table
                i += 2; // Skip header and separator
                while i < lines.len() && !in_code_block[i] {
                    let row = lines[i].trim();
                    if row.is_empty() || !row.contains('|') {
                        break;
                    }
                    // Stop if we hit another separator (would indicate a new table)
                    if self.is_table_separator(row) {
                        break;
                    }
                    i += 1;
                }
                table_blocks.push((start, i - 1));
            } else {
                i += 1;
            }
        }

        table_blocks
    }

    /// Parse a table row into cells
    fn parse_cells(&self, line: &str) -> Vec<String> {
        let trimmed = line.trim();

        // Handle leading/trailing pipes
        let content = if let Some(stripped) = trimmed.strip_prefix('|') {
            if let Some(inner) = stripped.strip_suffix('|') {
                inner
            } else {
                stripped
            }
        } else if let Some(stripped) = trimmed.strip_suffix('|') {
            stripped
        } else {
            trimmed
        };

        content.split('|').map(|s| s.to_string()).collect()
    }

    /// Detect the column style of a cell
    fn detect_cell_style(&self, cell: &str) -> Option<ColumnStyle> {
        if cell.is_empty() {
            return None;
        }

        let trimmed = cell.trim();
        if trimmed.is_empty() {
            // Cell contains only whitespace - could be empty cell
            return None;
        }

        let leading_spaces = cell.len() - cell.trim_start().len();
        let trailing_spaces = cell.len() - cell.trim_end().len();

        if leading_spaces == 0 && trailing_spaces == 0 {
            // No padding at all: |Value|
            Some(ColumnStyle::Compact)
        } else if leading_spaces >= 1 {
            // Has leading space - considered "tight" (readable style)
            // We don't distinguish aligned vs tight at cell level since
            // aligned tables have variable trailing space per column width
            Some(ColumnStyle::Tight)
        } else {
            // Edge case: trailing but no leading space
            Some(ColumnStyle::Tight)
        }
    }

    /// Check if a table uses aligned style (variable trailing spaces for column alignment)
    fn is_table_aligned(&self, lines: &[&str], start: usize, end: usize) -> bool {
        let mut max_trailing_spaces = 0;

        for line in lines.iter().take(end + 1).skip(start) {
            if self.is_table_separator(line) {
                continue;
            }

            let cells = self.parse_cells(line);
            for cell in &cells {
                let trimmed = cell.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let trailing_spaces = cell.len() - cell.trim_end().len();
                if trailing_spaces > max_trailing_spaces {
                    max_trailing_spaces = trailing_spaces;
                }
            }
        }

        // If any cell has more than 1 trailing space, consider it aligned style
        max_trailing_spaces > 1
    }

    /// Analyze a table to detect its column style
    fn analyze_table_style(&self, lines: &[&str], start: usize, end: usize) -> Option<ColumnStyle> {
        // First check if this is an aligned table (has extra trailing spaces)
        if self.is_table_aligned(lines, start, end) {
            return Some(ColumnStyle::Aligned);
        }

        // Otherwise detect from first cell
        for line in lines.iter().take(end + 1).skip(start) {
            // Skip separator rows
            if self.is_table_separator(line) {
                continue;
            }

            let cells = self.parse_cells(line);
            for cell in &cells {
                if let Some(style) = self.detect_cell_style(cell) {
                    return Some(style);
                }
            }
        }

        None
    }

    /// Check a table for style violations
    fn check_table(
        &self,
        lines: &[&str],
        start: usize,
        end: usize,
        expected_style: ColumnStyle,
    ) -> Vec<Violation> {
        let mut violations = Vec::new();

        for (line_idx, line) in lines.iter().enumerate().take(end + 1).skip(start) {
            let line_number = line_idx + 1;

            // Skip separator rows
            if self.is_table_separator(line) {
                continue;
            }

            let cells = self.parse_cells(line);
            for (col_idx, cell) in cells.iter().enumerate() {
                if let Some(cell_style) = self.detect_cell_style(cell) {
                    // Aligned style is compatible with tight (aligned tables have varying padding)
                    // So we only flag when:
                    // - expected is compact but found tight/aligned
                    // - expected is tight but found compact
                    // - expected is aligned but found compact
                    let is_compatible = match (expected_style, cell_style) {
                        (ColumnStyle::Any, _) => true,
                        (ColumnStyle::Consistent, _) => true,
                        (ColumnStyle::Aligned, ColumnStyle::Tight) => true, // Aligned includes tight cells
                        (ColumnStyle::Aligned, ColumnStyle::Aligned) => true,
                        (ColumnStyle::Tight, ColumnStyle::Tight) => true,
                        (ColumnStyle::Compact, ColumnStyle::Compact) => true,
                        _ => false,
                    };

                    if !is_compatible {
                        let expected_desc = match expected_style {
                            ColumnStyle::Aligned => "aligned",
                            ColumnStyle::Compact => "compact",
                            ColumnStyle::Tight => "tight",
                            ColumnStyle::Any | ColumnStyle::Consistent => continue,
                        };
                        let found_desc = match cell_style {
                            ColumnStyle::Aligned => "aligned",
                            ColumnStyle::Compact => "compact",
                            ColumnStyle::Tight => "tight",
                            ColumnStyle::Any | ColumnStyle::Consistent => continue,
                        };

                        violations.push(self.create_violation(
                            format!(
                                "Table column style inconsistent - expected '{expected_desc}' but found '{found_desc}' in column {}",
                                col_idx + 1
                            ),
                            line_number,
                            1,
                            Severity::Warning,
                        ));
                        // Only report one violation per row to avoid noise
                        break;
                    }
                }
            }
        }

        violations
    }
}

impl Default for MD060 {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for MD060 {
    fn id(&self) -> &'static str {
        "MD060"
    }

    fn name(&self) -> &'static str {
        "table-column-style"
    }

    fn description(&self) -> &'static str {
        "Table column style should be consistent"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Formatting).introduced_in("mdbook-lint v0.5.0")
    }

    fn can_fix(&self) -> bool {
        false // Style changes are complex and better done manually
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        // If style is Any, nothing to check
        if self.style == ColumnStyle::Any {
            return Ok(Vec::new());
        }

        let mut violations = Vec::new();
        let lines: Vec<&str> = document.content.lines().collect();
        let in_code_block = self.get_code_block_ranges(&lines);

        // Find all table blocks
        let table_blocks = self.find_table_blocks(&lines, &in_code_block);

        let mut document_style: Option<ColumnStyle> = match self.style {
            ColumnStyle::Aligned => Some(ColumnStyle::Aligned),
            ColumnStyle::Compact => Some(ColumnStyle::Compact),
            ColumnStyle::Tight => Some(ColumnStyle::Tight),
            ColumnStyle::Consistent => None, // Detect from first table
            ColumnStyle::Any => return Ok(Vec::new()),
        };

        // Process each table
        for (start, end) in table_blocks {
            // Detect style from first table if using Consistent mode
            if document_style.is_none() {
                document_style = self.analyze_table_style(&lines, start, end);
            }

            if let Some(expected_style) = document_style {
                let table_violations = self.check_table(&lines, start, end, expected_style);
                violations.extend(table_violations);
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
    fn test_md060_consistent_tight_style() {
        // Tight style: single space on each side of content
        let content = r#"| A | B | C |
|---|---|---|
| 1 | 2 | 3 |
| 4 | 5 | 6 |
"#;

        let document = create_test_document(content);
        let rule = MD060::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md060_consistent_compact_style() {
        // Compact style: no spaces around content
        let content = r#"|A|B|C|
|---|---|---|
|1|2|3|
|4|5|6|
"#;

        let document = create_test_document(content);
        let rule = MD060::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md060_consistent_aligned_style() {
        // Aligned style: extra spaces for column alignment
        let content = r#"| Column 1   | Col 2 | Column 3     |
|------------|-------|--------------|
| Value 1    | V2    | Value 3      |
| V4         | Val 5 | V6           |
"#;

        let document = create_test_document(content);
        let rule = MD060::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md060_mixed_styles_violation() {
        // First row is tight, data row mixes compact and tight
        let content = r#"| A | B |
|---|---|
|1| 2 |
"#;

        let document = create_test_document(content);
        let rule = MD060::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD060");
        assert!(violations[0].message.contains("inconsistent"));
    }

    #[test]
    fn test_md060_enforced_tight_style() {
        // Compact style content should fail when tight is required
        let content = r#"|A|B|
|---|---|
|1|2|
"#;

        let document = create_test_document(content);
        let rule = MD060::with_style(ColumnStyle::Tight);
        let violations = rule.check(&document).unwrap();
        // Compact style violates tight requirement
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_md060_enforced_compact_style() {
        // Tight style content should fail when compact is required
        let content = r#"| A | B |
|---|---|
| 1 | 2 |
"#;

        let document = create_test_document(content);
        let rule = MD060::with_style(ColumnStyle::Compact);
        let violations = rule.check(&document).unwrap();
        // Tight style violates compact requirement
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_md060_any_style_no_violations() {
        // Mixed styles should be OK when Any is set
        let content = r#"| A | B |
|---|---|
|1| 2 |
"#;

        let document = create_test_document(content);
        let rule = MD060::with_style(ColumnStyle::Any);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md060_multiple_tables_consistent() {
        // Both tables use tight style
        let content = r#"| A | B |
|---|---|
| 1 | 2 |

Some text.

| C | D |
|---|---|
| 3 | 4 |
"#;

        let document = create_test_document(content);
        let rule = MD060::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md060_multiple_tables_inconsistent() {
        // First table tight, second table compact
        let content = r#"| A | B |
|---|---|
| 1 | 2 |

Some text.

|C|D|
|---|---|
|3|4|
"#;

        let document = create_test_document(content);
        let rule = MD060::new();
        let violations = rule.check(&document).unwrap();
        // Second table has different style from first
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_md060_code_blocks_ignored() {
        // Table inside code block should be ignored
        let content = r#"| A | B |
|---|---|
| 1 | 2 |

```
|X|Y|
|---|---|
|a|b|
```

| C | D |
|---|---|
| 3 | 4 |
"#;

        let document = create_test_document(content);
        let rule = MD060::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md060_no_tables() {
        let content = r#"# Heading

This is just regular text without any tables.

Some more content.
"#;

        let document = create_test_document(content);
        let rule = MD060::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md060_from_config() {
        let config = toml::toml! {
            style = "compact"
        };

        let rule = MD060::from_config(&config);
        assert_eq!(rule.style, ColumnStyle::Compact);
    }

    #[test]
    fn test_md060_from_config_aligned() {
        let config = toml::toml! {
            style = "aligned"
        };

        let rule = MD060::from_config(&config);
        assert_eq!(rule.style, ColumnStyle::Aligned);
    }

    #[test]
    fn test_md060_from_config_tight() {
        let config = toml::toml! {
            style = "tight"
        };

        let rule = MD060::from_config(&config);
        assert_eq!(rule.style, ColumnStyle::Tight);
    }

    #[test]
    fn test_md060_from_config_any() {
        let config = toml::toml! {
            style = "any"
        };

        let rule = MD060::from_config(&config);
        assert_eq!(rule.style, ColumnStyle::Any);
    }

    #[test]
    fn test_md060_from_config_invalid_defaults_to_consistent() {
        let config = toml::toml! {
            style = "invalid_style"
        };

        let rule = MD060::from_config(&config);
        assert_eq!(rule.style, ColumnStyle::Consistent);
    }

    #[test]
    fn test_md060_separator_with_alignment() {
        // Separator rows with alignment markers should not affect style detection
        let content = r#"| A | B |
|:--|--:|
| 1 | 2 |
"#;

        let document = create_test_document(content);
        let rule = MD060::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md060_can_fix() {
        let rule = MD060::new();
        assert!(!Rule::can_fix(&rule));
    }
}
