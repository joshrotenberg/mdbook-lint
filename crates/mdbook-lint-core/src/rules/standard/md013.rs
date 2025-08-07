use crate::error::Result;
use crate::rule::{Rule, RuleCategory, RuleMetadata};
use crate::{
    Document,
    violation::{Severity, Violation},
};

/// MD013: Line length should not exceed a specified limit
///
/// This rule is triggered when lines exceed a specified length.
/// The default line length is 80 characters.
pub struct MD013 {
    /// Maximum allowed line length
    pub line_length: usize,
    /// Whether to ignore code blocks
    pub ignore_code_blocks: bool,
    /// Whether to ignore tables
    pub ignore_tables: bool,
    /// Whether to ignore headings
    pub ignore_headings: bool,
}

impl MD013 {
    /// Create a new MD013 rule with default settings
    pub fn new() -> Self {
        Self {
            line_length: 80,
            ignore_code_blocks: true,
            ignore_tables: true,
            ignore_headings: true,
        }
    }

    /// Create a new MD013 rule with custom line length
    #[allow(dead_code)]
    pub fn with_line_length(line_length: usize) -> Self {
        Self {
            line_length,
            ignore_code_blocks: true,
            ignore_tables: true,
            ignore_headings: true,
        }
    }

    /// Check if a line should be ignored based on rule settings
    fn should_ignore_line(&self, line: &str, in_code_block: bool, in_table: bool) -> bool {
        let trimmed = line.trim_start();

        // Ignore code blocks if configured
        if in_code_block && self.ignore_code_blocks {
            return true;
        }

        // Ignore tables if configured
        if in_table && self.ignore_tables {
            return true;
        }

        // Ignore headings if configured
        if self.ignore_headings && trimmed.starts_with('#') {
            return true;
        }

        // Always ignore lines that are just URLs (common in markdown)
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            return true;
        }

        false
    }
}

impl Default for MD013 {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for MD013 {
    fn id(&self) -> &'static str {
        "MD013"
    }

    fn name(&self) -> &'static str {
        "line-length"
    }

    fn description(&self) -> &'static str {
        "Line length should not exceed a specified limit"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Formatting).introduced_in("markdownlint v0.1.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        // MD013 is line-based and doesn't need AST, so we ignore the ast parameter
        let mut violations = Vec::new();
        let mut in_code_block = false;
        let mut in_table = false;

        for (line_number, line) in document.lines.iter().enumerate() {
            let line_num = line_number + 1; // Convert to 1-based

            // Track code block state
            if line.trim_start().starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }

            // Track table state (simplified - tables have | characters)
            let trimmed = line.trim();
            if !in_code_block && (trimmed.starts_with('|') || trimmed.contains(" | ")) {
                in_table = true;
            } else if in_table && trimmed.is_empty() {
                in_table = false;
            }

            // Check if we should ignore this line
            if self.should_ignore_line(line, in_code_block, in_table) {
                continue;
            }

            // Check line length
            if line.len() > self.line_length {
                let message = format!(
                    "Line length is {} characters, expected no more than {}",
                    line.len(),
                    self.line_length
                );

                violations.push(self.create_violation(
                    message,
                    line_num,
                    self.line_length + 1, // Point to the first character that exceeds limit
                    Severity::Warning,
                ));
            }
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_md013_short_lines() {
        let content = "# Short title\n\nThis is a short line.\nAnother short line.";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD013::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md013_long_line() {
        let long_line = "a".repeat(100);
        let content = format!("# Title\n\n{long_line}");
        let document = Document::new(content, PathBuf::from("test.md")).unwrap();
        let rule = MD013::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD013");
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[0].column, 81);
        assert_eq!(violations[0].severity, Severity::Warning);
        assert!(violations[0].message.contains("100 characters"));
        assert!(violations[0].message.contains("no more than 80"));
    }

    #[test]
    fn test_md013_custom_line_length() {
        let content = "This line is exactly fifty characters long here.";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD013::with_line_length(40);
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("48 characters"));
        assert!(violations[0].message.contains("no more than 40"));
    }

    #[test]
    fn test_md013_ignore_headings() {
        let long_heading = format!("# {}", "a".repeat(100));
        let document = Document::new(long_heading, PathBuf::from("test.md")).unwrap();
        let rule = MD013::new(); // ignore_headings is true by default
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md013_ignore_code_blocks() {
        let content = r#"# Title

```rust
let very_long_line_of_code_that_exceeds_the_normal_line_length_limit_but_should_be_ignored = "value";
```

This is a normal line that should be checked."#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD013::new(); // ignore_code_blocks is true by default
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md013_ignore_urls() {
        let content = "https://example.com/very/long/path/that/exceeds/normal/line/length/limits/but/should/be/ignored";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD013::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md013_ignore_tables() {
        let content = r#"# Title

| Column 1 with very long content | Column 2 with very long content | Column 3 with very long content |
|----------------------------------|----------------------------------|----------------------------------|
| Data 1 with very long content   | Data 2 with very long content   | Data 3 with very long content   |

This is a normal line that should be checked if it's too long."#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD013::new(); // ignore_tables is true by default
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md013_multiple_violations() {
        let long_line = "a".repeat(100);
        let content = format!("Normal line\n{long_line}\nAnother normal line\n{long_line}");
        let document = Document::new(content, PathBuf::from("test.md")).unwrap();
        let rule = MD013::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 2);
        assert_eq!(violations[1].line, 4);
    }
}
