use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};

/// Line length calculation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LengthMode {
    /// Strict mode: count all characters
    #[default]
    Strict,
    /// Visual mode: exclude URLs from length calculation
    Visual,
}

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
    /// Line length calculation mode
    pub length_mode: LengthMode,
}

impl MD013 {
    /// Create a new MD013 rule with default settings
    pub fn new() -> Self {
        Self {
            line_length: 80,
            ignore_code_blocks: true,
            ignore_tables: true,
            ignore_headings: true,
            length_mode: LengthMode::default(),
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
            length_mode: LengthMode::default(),
        }
    }

    /// Create MD013 from configuration
    pub fn from_config(config: &toml::Value) -> Self {
        let mut rule = Self::new();

        if let Some(line_length) = config
            .get("line-length")
            .or_else(|| config.get("line_length"))
            .and_then(|v| v.as_integer())
        {
            rule.line_length = line_length as usize;
        }

        if let Some(ignore_code) = config
            .get("ignore-code-blocks")
            .or_else(|| config.get("ignore_code_blocks"))
            .and_then(|v| v.as_bool())
        {
            rule.ignore_code_blocks = ignore_code;
        }

        if let Some(ignore_tables) = config
            .get("ignore-tables")
            .or_else(|| config.get("ignore_tables"))
            .and_then(|v| v.as_bool())
        {
            rule.ignore_tables = ignore_tables;
        }

        if let Some(ignore_headings) = config
            .get("ignore-headings")
            .or_else(|| config.get("ignore_headings"))
            .and_then(|v| v.as_bool())
        {
            rule.ignore_headings = ignore_headings;
        }

        if let Some(mode) = config
            .get("length-mode")
            .or_else(|| config.get("length_mode"))
            .and_then(|v| v.as_str())
        {
            rule.length_mode = match mode.to_lowercase().as_str() {
                "visual" => LengthMode::Visual,
                _ => LengthMode::Strict,
            };
        }

        rule
    }

    /// Calculate line length based on the configured mode
    fn calculate_length(&self, line: &str) -> usize {
        match self.length_mode {
            LengthMode::Strict => line.len(),
            LengthMode::Visual => self.visual_length(line),
        }
    }

    /// Calculate visual line length by excluding URLs
    fn visual_length(&self, line: &str) -> usize {
        // Regex pattern for URLs (both bare and in markdown links)
        let mut length = line.len();
        let mut search_start = 0;

        // Find and subtract URL lengths
        while let Some(url_start) = line[search_start..]
            .find("http://")
            .or_else(|| line[search_start..].find("https://"))
        {
            let abs_start = search_start + url_start;
            // Find the end of the URL (whitespace, ), ], or end of line)
            let url_end = line[abs_start..]
                .find(|c: char| {
                    c.is_whitespace() || c == ')' || c == ']' || c == '>' || c == '"' || c == '\''
                })
                .map(|pos| abs_start + pos)
                .unwrap_or(line.len());

            let url_len = url_end - abs_start;
            length = length.saturating_sub(url_len);
            search_start = url_end;
        }

        length
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
            let effective_length = self.calculate_length(line);
            if effective_length > self.line_length {
                let message = match self.length_mode {
                    LengthMode::Strict => format!(
                        "Line length is {} characters, expected no more than {}",
                        effective_length, self.line_length
                    ),
                    LengthMode::Visual => format!(
                        "Line length is {} characters (visual), expected no more than {}",
                        effective_length, self.line_length
                    ),
                };

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

    #[test]
    fn test_md013_visual_mode_excludes_urls() {
        // Line with URL that would be too long in strict mode but OK in visual mode
        // "See docs at " (12) + URL (60) + " for details" (12) = 84 chars total
        // Visual length = 84 - 60 = 24 chars (under 80)
        let content = "See docs at https://example.com/very/long/path/to/documentation for details";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();

        // Strict mode should flag it if over limit
        let strict_rule = MD013::new();
        let strict_violations = strict_rule.check(&document).unwrap();
        // This line is 75 chars, so no violation in strict mode either
        assert_eq!(strict_violations.len(), 0);

        // Now test with a longer line
        let long_content = "See the documentation at https://example.com/very/long/path/to/documentation/that/goes/on/and/on for more details about this feature";
        let long_doc = Document::new(long_content.to_string(), PathBuf::from("test.md")).unwrap();

        // Strict mode should flag it (136 chars)
        let strict_violations = strict_rule.check(&long_doc).unwrap();
        assert_eq!(strict_violations.len(), 1);

        // Visual mode should not flag it (URL is ~90 chars, remaining text is ~46 chars)
        let mut visual_rule = MD013::new();
        visual_rule.length_mode = LengthMode::Visual;
        let visual_violations = visual_rule.check(&long_doc).unwrap();
        assert_eq!(visual_violations.len(), 0);
    }

    #[test]
    fn test_md013_visual_mode_multiple_urls() {
        // Line with multiple URLs
        let content = "Check https://example.com/path1 and https://example.com/path2 for info";

        let mut rule = MD013::new();
        rule.length_mode = LengthMode::Visual;

        // Visual length should exclude both URLs
        let visual_len = rule.visual_length(content);
        // Total: 70 chars, URL1: 27 chars, URL2: 27 chars
        // Visual: 70 - 27 - 27 = 16 chars (just "Check ", " and ", " for info")
        assert!(
            visual_len < 25,
            "Visual length should be small: {}",
            visual_len
        );
    }

    #[test]
    fn test_md013_visual_mode_markdown_link() {
        // Markdown link with long URL
        let content =
            "See [the docs](https://example.com/very/long/path/to/documentation) for details";

        let mut rule = MD013::new();
        rule.length_mode = LengthMode::Visual;

        let visual_len = rule.visual_length(content);
        // URL should be excluded from count
        assert!(
            visual_len < 40,
            "Visual length should exclude URL: {}",
            visual_len
        );
    }

    #[test]
    fn test_md013_from_config_length_mode() {
        let config: toml::Value = toml::from_str(
            r#"
            line-length = 100
            length-mode = "visual"
        "#,
        )
        .unwrap();

        let rule = MD013::from_config(&config);
        assert_eq!(rule.line_length, 100);
        assert_eq!(rule.length_mode, LengthMode::Visual);
    }

    #[test]
    fn test_md013_from_config_length_mode_strict() {
        let config: toml::Value = toml::from_str(
            r#"
            length-mode = "strict"
        "#,
        )
        .unwrap();

        let rule = MD013::from_config(&config);
        assert_eq!(rule.length_mode, LengthMode::Strict);
    }

    #[test]
    fn test_md013_visual_length_no_urls() {
        let rule = MD013::new();
        let line = "This is a normal line without any URLs in it.";
        assert_eq!(rule.visual_length(line), line.len());
    }
}
