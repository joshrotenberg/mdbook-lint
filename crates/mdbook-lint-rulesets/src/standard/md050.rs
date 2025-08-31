//! MD050: Strong style consistency
//!
//! This rule checks that strong emphasis markers (bold text) are used consistently throughout the document.

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Fix, Position, Severity, Violation},
};

/// Rule to check strong emphasis style consistency
pub struct MD050 {
    /// Preferred strong emphasis style
    style: StrongStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StrongStyle {
    /// Use double asterisk (**text**)
    Asterisk,
    /// Use double underscore (__text__)
    Underscore,
    /// Detect from first usage in document
    Consistent,
}

impl MD050 {
    /// Create a new MD050 rule with consistent style detection
    pub fn new() -> Self {
        Self {
            style: StrongStyle::Consistent,
        }
    }

    /// Create a new MD050 rule with specific style preference
    #[allow(dead_code)]
    pub fn with_style(style: StrongStyle) -> Self {
        Self { style }
    }

    /// Create MD050 from configuration
    pub fn from_config(config: &toml::Value) -> Self {
        let mut rule = Self::new();

        if let Some(style_str) = config.get("style").and_then(|v| v.as_str()) {
            rule.style = match style_str.to_lowercase().as_str() {
                "asterisk" => StrongStyle::Asterisk,
                "underscore" => StrongStyle::Underscore,
                "consistent" => StrongStyle::Consistent,
                _ => StrongStyle::Consistent, // Default fallback
            };
        }

        rule
    }

    /// Find strong emphasis markers in a line and check for style violations
    fn check_line_strong(
        &self,
        line: &str,
        line_number: usize,
        expected_style: Option<StrongStyle>,
    ) -> (Vec<Violation>, Option<StrongStyle>) {
        let mut violations = Vec::new();
        let mut detected_style = expected_style;

        // Find strong emphasis markers - look for double ** or __
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if (chars[i] == '*' || chars[i] == '_')
                && i + 1 < chars.len()
                && chars[i + 1] == chars[i]
            {
                let marker = chars[i];

                // Look for closing marker pair
                if let Some(end_pos) = self.find_closing_strong_marker(&chars, i + 2, marker) {
                    let current_style = if marker == '*' {
                        StrongStyle::Asterisk
                    } else {
                        StrongStyle::Underscore
                    };

                    // Establish or check style consistency
                    if let Some(ref expected) = detected_style {
                        if *expected != current_style {
                            let expected_marker = if *expected == StrongStyle::Asterisk {
                                "**"
                            } else {
                                "__"
                            };
                            let found_marker = if marker == '*' { "**" } else { "__" };
                            // Get the text between the markers
                            let text_start = i + 2;
                            let text: String = chars[text_start..end_pos].iter().collect();

                            // Create fix by replacing the current strong with expected style
                            let fixed_strong =
                                format!("{}{}{}", expected_marker, text, expected_marker);
                            let original_strong =
                                format!("{}{}{}", found_marker, text, found_marker);

                            // Find the actual position in the line to replace
                            let line_content = line;
                            let mut fixed_line = line_content.to_string();
                            if let Some(pos) = fixed_line.find(&original_strong) {
                                fixed_line
                                    .replace_range(pos..pos + original_strong.len(), &fixed_strong);
                            }

                            let fix = Fix {
                                description: format!(
                                    "Change strong emphasis style from '{}' to '{}'",
                                    found_marker, expected_marker
                                ),
                                replacement: Some(format!("{}\n", fixed_line)),
                                start: Position {
                                    line: line_number,
                                    column: 1,
                                },
                                end: Position {
                                    line: line_number,
                                    column: line_content.len() + 1,
                                },
                            };

                            violations.push(self.create_violation_with_fix(
                                format!(
                                    "Strong emphasis style inconsistent - expected '{expected_marker}' but found '{found_marker}'"
                                ),
                                line_number,
                                i + 1, // Convert to 1-based column
                                Severity::Warning,
                                fix,
                            ));
                        }
                    } else {
                        // First strong emphasis found - establish the style
                        detected_style = Some(current_style);
                    }

                    i = end_pos + 2;
                } else {
                    i += 2;
                }
            } else {
                i += 1;
            }
        }

        (violations, detected_style)
    }

    /// Find the closing strong emphasis marker pair
    fn find_closing_strong_marker(
        &self,
        chars: &[char],
        start: usize,
        marker: char,
    ) -> Option<usize> {
        let mut i = start;

        while i + 1 < chars.len() {
            if chars[i] == marker && chars[i + 1] == marker {
                return Some(i);
            }
            i += 1;
        }

        None
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

impl Default for MD050 {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for MD050 {
    fn id(&self) -> &'static str {
        "MD050"
    }

    fn name(&self) -> &'static str {
        "strong-style"
    }

    fn description(&self) -> &'static str {
        "Strong emphasis style should be consistent"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Formatting).introduced_in("mdbook-lint v0.1.0")
    }

    fn can_fix(&self) -> bool {
        true
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let lines: Vec<&str> = document.content.lines().collect();
        let in_code_block = self.get_code_block_ranges(&lines);

        let mut expected_style = match self.style {
            StrongStyle::Asterisk => Some(StrongStyle::Asterisk),
            StrongStyle::Underscore => Some(StrongStyle::Underscore),
            StrongStyle::Consistent => None, // Detect from first usage
        };

        for (line_number, line) in lines.iter().enumerate() {
            let line_number = line_number + 1;

            // Skip lines inside code blocks
            if in_code_block[line_number - 1] {
                continue;
            }

            let (line_violations, detected_style) =
                self.check_line_strong(line, line_number, expected_style);
            violations.extend(line_violations);

            // Update expected style if we detected one
            if expected_style.is_none() && detected_style.is_some() {
                expected_style = detected_style;
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
    fn test_md050_consistent_asterisk_style() {
        let content = r#"This has **strong** and more **bold text** here.

Another paragraph with **more strong** text.
"#;

        let document = create_test_document(content);
        let rule = MD050::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md050_consistent_underscore_style() {
        let content = r#"This has __strong__ and more __bold text__ here.

Another paragraph with __more strong__ text.
"#;

        let document = create_test_document(content);
        let rule = MD050::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md050_mixed_styles_violation() {
        let content = r#"This has **strong** and more __bold text__ here.

Another paragraph with **more strong** text.
"#;

        let document = create_test_document(content);
        let rule = MD050::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD050");
        assert_eq!(violations[0].line, 1);
        assert!(
            violations[0]
                .message
                .contains("expected '**' but found '__'")
        );
    }

    #[test]
    fn test_md050_preferred_asterisk_style() {
        let content = r#"This has __strong__ text.
"#;

        let document = create_test_document(content);
        let rule = MD050::with_style(StrongStyle::Asterisk);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("expected '**' but found '__'")
        );
    }

    #[test]
    fn test_md050_preferred_underscore_style() {
        let content = r#"This has **strong** text.
"#;

        let document = create_test_document(content);
        let rule = MD050::with_style(StrongStyle::Underscore);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("expected '__' but found '**'")
        );
    }

    #[test]
    fn test_md050_emphasis_ignored() {
        let content = r#"This has *italic text* and __strong text__.

More *italic* and __strong__ here.
"#;

        let document = create_test_document(content);
        let rule = MD050::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0); // All strong uses __, should be consistent
    }

    #[test]
    fn test_md050_mixed_emphasis_and_strong() {
        let content = r#"This has *italic* and **strong** and __also strong__.

More text here.
"#;

        let document = create_test_document(content);
        let rule = MD050::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("expected '**' but found '__'")
        );
    }

    #[test]
    fn test_md050_code_blocks_ignored() {
        let content = r#"This has **strong** text.

```
Code with **asterisks** and __underscores__ should be ignored.
```

This has __different style__ which should trigger violation.
"#;

        let document = create_test_document(content);
        let rule = MD050::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 7);
    }

    #[test]
    fn test_md050_inline_code_spans() {
        let content = r#"This has **strong** and `code with **asterisks**` text.

More **strong** text here.
"#;

        let document = create_test_document(content);
        let rule = MD050::new();
        let violations = rule.check(&document).unwrap();
        // Code spans are not excluded by this rule (they're handled at line level)
        // but the strong emphasis should still be consistent
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md050_no_strong() {
        let content = r#"This document has no strong emphasis at all.

Just regular text with *italic* formatting.
"#;

        let document = create_test_document(content);
        let rule = MD050::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md050_multiple_violations() {
        let content = r#"Start with **strong** text.

Then switch to __different style__.

Back to **original style**.

And __different again__.
"#;

        let document = create_test_document(content);
        let rule = MD050::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2); // Line 3 and line 7 violations
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 7);
    }

    #[test]
    fn test_md050_unclosed_strong() {
        let content = r#"This has **unclosed strong and __closed strong__.

More text here.
"#;

        let document = create_test_document(content);
        let rule = MD050::new();
        let violations = rule.check(&document).unwrap();
        // Only the properly closed strong should be checked
        assert_eq!(violations.len(), 0); // __closed strong__ is the only valid strong, so no violation
    }

    #[test]
    fn test_md050_nested_formatting() {
        let content = r#"This has **strong with *nested italic* text**.

More __strong__ text.
"#;

        let document = create_test_document(content);
        let rule = MD050::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("expected '**' but found '__'")
        );
    }

    #[test]
    fn test_md050_fix_style() {
        let content = r#"This has **strong** text.

Then uses __different__ style.

Back to **original** again.
"#;

        let document = create_test_document(content);
        let rule = MD050::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.description,
            "Change strong emphasis style from '__' to '**'"
        );
        assert_eq!(
            fix.replacement,
            Some("Then uses **different** style.\n".to_string())
        );
    }

    #[test]
    fn test_md050_can_fix() {
        let rule = MD050::new();
        assert!(Rule::can_fix(&rule));
    }
}
