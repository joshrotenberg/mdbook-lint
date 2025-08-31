//! MD011: Reversed link syntax
//!
//! This rule checks for reversed link syntax: (text)\[url\] instead of \[text\](url).

use comrak::nodes::AstNode;
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Fix, Position, Severity, Violation},
};

/// Rule to check for reversed link syntax
pub struct MD011;

impl AstRule for MD011 {
    fn id(&self) -> &'static str {
        "MD011"
    }

    fn name(&self) -> &'static str {
        "no-reversed-links"
    }

    fn description(&self) -> &'static str {
        "Reversed link syntax"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Content).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_ast<'a>(&self, document: &Document, _ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let mut in_code_block = false;

        for (line_number, line) in document.lines.iter().enumerate() {
            // Track code block state
            if line.trim_start().starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }

            // Skip lines inside code blocks
            if in_code_block {
                continue;
            }

            // Parse the line character by character looking for (text)[url] pattern
            // but skip content inside inline code spans
            let chars: Vec<char> = line.chars().collect();
            let mut i = 0;

            while i < chars.len() {
                // Skip inline code spans
                if chars[i] == '`' {
                    i += 1;
                    // Find the closing backtick
                    while i < chars.len() && chars[i] != '`' {
                        i += 1;
                    }
                    if i < chars.len() {
                        i += 1; // Skip closing backtick
                    }
                    continue;
                }

                if chars[i] == '(' {
                    // Found opening parenthesis, look for the pattern (text)[url]
                    if let Some((text, url, start_pos, end_pos)) =
                        self.parse_reversed_link(&chars, i)
                    {
                        // Create fix to reverse the link syntax
                        let fix = Fix {
                            description: format!("Fix reversed link: [{text}]({url})"),
                            replacement: Some(format!("[{text}]({url})")),
                            start: Position {
                                line: line_number + 1,
                                column: start_pos + 1,
                            },
                            end: Position {
                                line: line_number + 1,
                                column: end_pos + 1, // +1 because end_pos is 0-based position of ']'
                            },
                        };

                        violations.push(self.create_violation_with_fix(
                            format!(
                                "Reversed link syntax: ({text})[{url}]. Should be: [{text}]({url})"
                            ),
                            line_number + 1, // 1-based line numbers
                            start_pos + 1,   // 1-based column
                            Severity::Error,
                            fix,
                        ));
                        i = end_pos;
                    } else {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }
        }

        Ok(violations)
    }
}

impl MD011 {
    /// Parse a potential reversed link starting at position `start`
    /// Returns (text, url, start_pos, end_pos) if a reversed link is found
    fn parse_reversed_link(
        &self,
        chars: &[char],
        start: usize,
    ) -> Option<(String, String, usize, usize)> {
        if start >= chars.len() || chars[start] != '(' {
            return None;
        }

        let mut i = start + 1;
        let mut text = String::new();

        // Parse text inside parentheses
        while i < chars.len() && chars[i] != ')' {
            text.push(chars[i]);
            i += 1;
        }

        // Must find closing parenthesis
        if i >= chars.len() || chars[i] != ')' {
            return None;
        }
        i += 1; // Skip ')'

        // Must find opening bracket
        if i >= chars.len() || chars[i] != '[' {
            return None;
        }
        i += 1; // Skip '['

        let mut url = String::new();

        // Parse URL inside brackets
        while i < chars.len() && chars[i] != ']' {
            url.push(chars[i]);
            i += 1;
        }

        // Must find closing bracket
        if i >= chars.len() || chars[i] != ']' {
            return None;
        }

        Some((text, url, start, i))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::Document;
    use mdbook_lint_core::rule::Rule;
    use std::path::PathBuf;

    #[test]
    fn test_md011_no_violations() {
        let content = r#"# Valid Links

Here's a [valid link](https://example.com) that works correctly.

Another [good link](./relative/path.md) here.

[Email link](mailto:test@example.com) is also fine.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD011;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md011_reversed_link_violation() {
        let content = r#"# Document with Reversed Link

This has (reversed link)[https://example.com] syntax.

Some content here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD011;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Reversed link syntax"));
        assert!(
            violations[0]
                .message
                .contains("(reversed link)[https://example.com]")
        );
        assert!(
            violations[0]
                .message
                .contains("Should be: [reversed link](https://example.com)")
        );
        assert_eq!(violations[0].line, 3);
    }

    #[test]
    fn test_md011_multiple_reversed_links() {
        let content = r#"# Multiple Issues

First (bad link)[url1] here.

Second (another bad)[url2] there.

And a (third one)[url3] at the end.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD011;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);

        assert_eq!(violations[0].line, 3);
        assert!(violations[0].message.contains("bad link"));
        assert!(violations[0].message.contains("url1"));

        assert_eq!(violations[1].line, 5);
        assert!(violations[1].message.contains("another bad"));
        assert!(violations[1].message.contains("url2"));

        assert_eq!(violations[2].line, 7);
        assert!(violations[2].message.contains("third one"));
        assert!(violations[2].message.contains("url3"));
    }

    #[test]
    fn test_md011_mixed_valid_and_invalid() {
        let content = r#"# Mixed Links

This [valid link](https://good.com) is fine.

But this (bad link)[https://bad.com] is not.

Another [good one](./path.md) here.

And another (problem)[./bad-path.md] there.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD011;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 5);
        assert_eq!(violations[1].line, 9);
    }

    #[test]
    fn test_md011_code_blocks_ignored() {
        let content = r#"# Code Examples

This (bad link)[url] should be detected.

```
This (code example)[url] should be ignored.
```

`This (inline code)[url] should be ignored.`

Another (bad link)[url2] should be detected.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD011;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 11);
    }

    #[test]
    fn test_md011_empty_text_and_url() {
        let content = r#"# Edge Cases

This ()[empty text] has empty parts.

This ()[url] has empty text.

This (text)[] has empty URL.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD011;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);
        assert!(violations[0].message.contains("Should be: [](empty text)"));
        assert!(violations[1].message.contains("Should be: [](url)"));
        assert!(violations[2].message.contains("Should be: [text]()"));
    }

    #[test]
    fn test_md011_complex_urls() {
        let content = r#"# Complex URLs

This (complex link)[https://example.com/path?param=value&other=test#anchor] is wrong.

This (relative link)[../parent/file.md#section] is also wrong.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD011;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("complex link"));
        assert!(
            violations[0]
                .message
                .contains("https://example.com/path?param=value&other=test#anchor")
        );
        assert!(violations[1].message.contains("relative link"));
        assert!(violations[1].message.contains("../parent/file.md#section"));
    }

    #[test]
    fn test_md011_fix_basic_reversed_link() {
        let content = r#"# Test

This is a (reversed link)[https://example.com] here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD011;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());
        
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Fix reversed link: [reversed link](https://example.com)");
        assert_eq!(fix.replacement, Some("[reversed link](https://example.com)".to_string()));
        assert_eq!(fix.start.line, 3);
        assert_eq!(fix.start.column, 11); // Position of opening paren
    }

    #[test]
    fn test_md011_fix_multiple_reversed_links() {
        let content = r#"# Multiple Links

First (link one)[url1] and then (link two)[url2] here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD011;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        
        // First link fix
        assert!(violations[0].fix.is_some());
        let fix1 = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix1.replacement, Some("[link one](url1)".to_string()));
        
        // Second link fix
        assert!(violations[1].fix.is_some());
        let fix2 = violations[1].fix.as_ref().unwrap();
        assert_eq!(fix2.replacement, Some("[link two](url2)".to_string()));
    }

    #[test]
    fn test_md011_fix_empty_text() {
        let content = r#"# Empty Text

This ()[https://example.com] has empty text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD011;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());
        
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, Some("[](https://example.com)".to_string()));
    }

    #[test]
    fn test_md011_fix_complex_url() {
        let content = r#"# Complex URL

Check this (documentation)[https://example.com/path?param=value&other=test#anchor] out.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD011;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());
        
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement, 
            Some("[documentation](https://example.com/path?param=value&other=test#anchor)".to_string())
        );
    }

    #[test]
    fn test_md011_fix_preserves_position() {
        let content = r#"# Position Test

Some text before (reversed)[url] and text after.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD011;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());
        
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.start.line, 3);
        assert_eq!(fix.start.column, 18); // Position of opening paren
        assert_eq!(fix.end.line, 3);
        // The text is: "Some text before (reversed)[url] and text after."
        // 0-based: position 17 is '(', position 31 is ']'
        // 1-based: position 18 is '(', position 32 is ']'
        // parse_reversed_link returns 31 (0-based position of ']')
        // Fix adds +1 to convert to 1-based, so end.column = 32
        assert_eq!(fix.end.column, 32);
    }
}
