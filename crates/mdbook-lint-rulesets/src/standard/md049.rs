//! MD049: Emphasis style consistency
//!
//! This rule checks that emphasis markers (italics) use a consistent style throughout the document.

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};

/// Rule to check emphasis style consistency
pub struct MD049 {
    /// Preferred emphasis style
    style: EmphasisStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmphasisStyle {
    /// Use asterisk (*text*)
    Asterisk,
    /// Use underscore (_text_)
    Underscore,
    /// Detect from first usage in document
    Consistent,
}

impl MD049 {
    /// Create a new MD049 rule with consistent style detection
    pub fn new() -> Self {
        Self {
            style: EmphasisStyle::Consistent,
        }
    }

    /// Create a new MD049 rule with specific style preference
    #[allow(dead_code)]
    pub fn with_style(style: EmphasisStyle) -> Self {
        Self { style }
    }

    /// Create MD049 from configuration
    pub fn from_config(config: &toml::Value) -> Self {
        let mut rule = Self::new();

        if let Some(style_str) = config.get("style").and_then(|v| v.as_str()) {
            rule.style = match style_str.to_lowercase().as_str() {
                "asterisk" => EmphasisStyle::Asterisk,
                "underscore" => EmphasisStyle::Underscore,
                "consistent" => EmphasisStyle::Consistent,
                _ => EmphasisStyle::Consistent, // Default fallback
            };
        }

        rule
    }

    /// Find emphasis markers in a line and check for style violations
    fn check_line_emphasis(
        &self,
        line: &str,
        line_number: usize,
        expected_style: Option<EmphasisStyle>,
    ) -> (Vec<Violation>, Option<EmphasisStyle>) {
        let mut violations = Vec::new();
        let mut detected_style = expected_style;

        // Get inline code span ranges to exclude from emphasis checking
        let code_span_ranges = self.get_inline_code_spans(line);

        // Find emphasis markers - look for single * or _ that aren't part of strong emphasis
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // Skip if we're inside a code span
            if self.is_inside_code_span(i, &code_span_ranges) {
                i += 1;
                continue;
            }

            if chars[i] == '*' || chars[i] == '_' {
                let marker = chars[i];

                // Skip if this is part of strong emphasis (** or __)
                if i + 1 < chars.len() && chars[i + 1] == marker {
                    i += 2;
                    continue;
                }

                // Skip if preceded by strong emphasis marker
                if i > 0 && chars[i - 1] == marker {
                    i += 1;
                    continue;
                }

                // Look for closing marker
                if let Some(end_pos) =
                    self.find_closing_emphasis_marker(&chars, i + 1, marker, &code_span_ranges)
                {
                    let current_style = if marker == '*' {
                        EmphasisStyle::Asterisk
                    } else {
                        EmphasisStyle::Underscore
                    };

                    // Establish or check style consistency
                    if let Some(ref expected) = detected_style {
                        if *expected != current_style {
                            let expected_marker = if *expected == EmphasisStyle::Asterisk {
                                '*'
                            } else {
                                '_'
                            };
                            violations.push(self.create_violation(
                                format!(
                                    "Emphasis style inconsistent - expected '{expected_marker}' but found '{marker}'"
                                ),
                                line_number,
                                i + 1, // Convert to 1-based column
                                Severity::Warning,
                            ));
                        }
                    } else {
                        // First emphasis found - establish the style
                        detected_style = Some(current_style);
                    }

                    i = end_pos + 1;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        (violations, detected_style)
    }

    /// Find the closing emphasis marker
    fn find_closing_emphasis_marker(
        &self,
        chars: &[char],
        start: usize,
        marker: char,
        code_span_ranges: &[(usize, usize)],
    ) -> Option<usize> {
        let mut i = start;

        while i < chars.len() {
            // Skip if we're inside a code span
            if self.is_inside_code_span(i, code_span_ranges) {
                i += 1;
                continue;
            }

            if chars[i] == marker {
                // Make sure this isn't part of strong emphasis
                if i + 1 < chars.len() && chars[i + 1] == marker {
                    i += 2;
                    continue;
                }
                if i > 0 && chars[i - 1] == marker {
                    i += 1;
                    continue;
                }
                return Some(i);
            }
            i += 1;
        }

        None
    }

    /// Get inline code span ranges (backtick spans) in a line
    fn get_inline_code_spans(&self, line: &str) -> Vec<(usize, usize)> {
        let mut code_spans = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '`' {
                // Count consecutive backticks
                let mut backtick_count = 0;
                let start = i;
                while i < chars.len() && chars[i] == '`' {
                    backtick_count += 1;
                    i += 1;
                }

                // Look for matching closing backticks
                let mut j = i;
                while j < chars.len() {
                    if chars[j] == '`' {
                        // Count consecutive closing backticks
                        let mut closing_count = 0;
                        let _closing_start = j;
                        while j < chars.len() && chars[j] == '`' {
                            closing_count += 1;
                            j += 1;
                        }

                        // If we found matching backticks, record the span
                        if closing_count == backtick_count {
                            code_spans.push((start, j - 1));
                            i = j;
                            break;
                        }
                    } else {
                        j += 1;
                    }
                }

                // If no closing backticks found, move past the opening backticks
                if j >= chars.len() {
                    break;
                }
            } else {
                i += 1;
            }
        }

        code_spans
    }

    /// Check if a character position is inside any code span
    fn is_inside_code_span(&self, pos: usize, code_spans: &[(usize, usize)]) -> bool {
        code_spans
            .iter()
            .any(|&(start, end)| pos >= start && pos <= end)
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

impl Default for MD049 {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for MD049 {
    fn id(&self) -> &'static str {
        "MD049"
    }

    fn name(&self) -> &'static str {
        "emphasis-style"
    }

    fn description(&self) -> &'static str {
        "Emphasis style should be consistent"
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

        let mut expected_style = match self.style {
            EmphasisStyle::Asterisk => Some(EmphasisStyle::Asterisk),
            EmphasisStyle::Underscore => Some(EmphasisStyle::Underscore),
            EmphasisStyle::Consistent => None, // Detect from first usage
        };

        for (line_number, line) in lines.iter().enumerate() {
            let line_number = line_number + 1;

            // Skip lines inside code blocks
            if in_code_block[line_number - 1] {
                continue;
            }

            let (line_violations, detected_style) =
                self.check_line_emphasis(line, line_number, expected_style);
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
    fn test_md049_consistent_asterisk_style() {
        let content = r#"This has *emphasis* and more *italic text* here.

Another paragraph with *more emphasis* text.
"#;

        let document = create_test_document(content);
        let rule = MD049::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md049_consistent_underscore_style() {
        let content = r#"This has _emphasis_ and more _italic text_ here.

Another paragraph with _more emphasis_ text.
"#;

        let document = create_test_document(content);
        let rule = MD049::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md049_mixed_styles_violation() {
        let content = r#"This has *emphasis* and more _italic text_ here.

Another paragraph with *more emphasis* text.
"#;

        let document = create_test_document(content);
        let rule = MD049::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD049");
        assert_eq!(violations[0].line, 1);
        assert!(violations[0].message.contains("expected '*' but found '_'"));
    }

    #[test]
    fn test_md049_preferred_asterisk_style() {
        let content = r#"This has _emphasis_ text.
"#;

        let document = create_test_document(content);
        let rule = MD049::with_style(EmphasisStyle::Asterisk);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("expected '*' but found '_'"));
    }

    #[test]
    fn test_md049_preferred_underscore_style() {
        let content = r#"This has *emphasis* text.
"#;

        let document = create_test_document(content);
        let rule = MD049::with_style(EmphasisStyle::Underscore);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("expected '_' but found '*'"));
    }

    #[test]
    fn test_md049_strong_emphasis_ignored() {
        let content = r#"This has **strong text** and _italic text_.

More **strong** and _italic_ here.
"#;

        let document = create_test_document(content);
        let rule = MD049::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0); // All underscores, should be consistent
    }

    #[test]
    fn test_md049_mixed_strong_and_emphasis() {
        let content = r#"This has **strong** and *italic* and _also italic_.

More text here.
"#;

        let document = create_test_document(content);
        let rule = MD049::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("expected '*' but found '_'"));
    }

    #[test]
    fn test_md049_code_blocks_ignored() {
        let content = r#"This has *italic* text.

```
Code with *asterisks* and _underscores_ should be ignored.
```

This has _different style_ which should trigger violation.
"#;

        let document = create_test_document(content);
        let rule = MD049::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 7);
    }

    #[test]
    fn test_md049_inline_code_spans() {
        let content = r#"This has *italic* and `code with *asterisks*` text.

More *italic* text here.
"#;

        let document = create_test_document(content);
        let rule = MD049::new();
        let violations = rule.check(&document).unwrap();
        // Code spans should be excluded - emphasis inside backticks should be ignored
        // Only the real emphasis outside code spans should be checked
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md049_no_emphasis() {
        let content = r#"This document has no emphasis at all.

Just regular text with **strong** formatting.
"#;

        let document = create_test_document(content);
        let rule = MD049::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md049_multiple_violations() {
        let content = r#"Start with *italic* text.

Then switch to _different style_.

Back to *original style*.

And _different again_.
"#;

        let document = create_test_document(content);
        let rule = MD049::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2); // Line 3 and line 7 violations
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 7);
    }

    #[test]
    fn test_md049_unclosed_emphasis() {
        let content = r#"This has *unclosed emphasis and _closed emphasis_.

More text here.
"#;

        let document = create_test_document(content);
        let rule = MD049::new();
        let violations = rule.check(&document).unwrap();
        // Only the properly closed emphasis should be checked
        assert_eq!(violations.len(), 0); // _closed emphasis_ is the only valid emphasis, so no violation
    }

    #[test]
    fn test_md049_code_spans_with_mixed_markers() {
        let content = r#"Use the `wrapping_*` methods, such as `wrapping_add`.

Return the `None` value if there is overflow with the `checked_*` methods.

Saturate at the value's minimum or maximum values with the `saturating_*` methods.

This has *real emphasis* outside code spans.
"#;

        let document = create_test_document(content);
        let rule = MD049::new();
        let violations = rule.check(&document).unwrap();
        // Should not find any violations - all underscores are in code spans
        // Only the real emphasis should be detected and it's consistent
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md049_mixed_emphasis_and_code_spans() {
        let content = r#"Use the `wrapping_*` methods for *italic text*.

And `checked_*` with _different emphasis style_.
"#;

        let document = create_test_document(content);
        let rule = MD049::new();
        let violations = rule.check(&document).unwrap();
        // Should find one violation - mixed emphasis styles outside code spans
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("expected '*' but found '_'"));
    }
}
