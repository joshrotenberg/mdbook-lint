//! MD034: Bare URL without angle brackets
//!
//! This rule checks for bare URLs that should be enclosed in angle brackets.

use comrak::nodes::AstNode;
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Fix, Position, Severity, Violation},
};

/// Rule to check for bare URLs without angle brackets
pub struct MD034;

impl AstRule for MD034 {
    fn id(&self) -> &'static str {
        "MD034"
    }

    fn name(&self) -> &'static str {
        "no-bare-urls"
    }

    fn description(&self) -> &'static str {
        "Bare URL used"
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

            // Parse the line character by character looking for bare URLs
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

                // Skip content inside links [text](url) or <url>
                if chars[i] == '[' {
                    // Skip to end of link
                    while i < chars.len() && chars[i] != ']' {
                        i += 1;
                    }
                    if i < chars.len() {
                        i += 1; // Skip ']'
                    }
                    // Skip the (url) part if it exists
                    if i < chars.len() && chars[i] == '(' {
                        while i < chars.len() && chars[i] != ')' {
                            i += 1;
                        }
                        if i < chars.len() {
                            i += 1; // Skip ')'
                        }
                    }
                    continue;
                }

                // Skip URLs already in angle brackets
                if chars[i] == '<' {
                    while i < chars.len() && chars[i] != '>' {
                        i += 1;
                    }
                    if i < chars.len() {
                        i += 1; // Skip '>'
                    }
                    continue;
                }

                // Look for URLs starting with http:// or https://
                if i + 7 < chars.len() && self.starts_with_url_scheme(&chars, i) {
                    let start_pos = i;
                    let url = self.extract_url(&chars, i);

                    if !url.is_empty() {
                        // Create fix to wrap URL in angle brackets
                        let fix = Fix {
                            description: "Wrap URL in angle brackets".to_string(),
                            replacement: Some(format!("<{}>", url)),
                            start: Position {
                                line: line_number + 1,
                                column: start_pos + 1,
                            },
                            end: Position {
                                line: line_number + 1,
                                column: start_pos + url.len() + 1,
                            },
                        };

                        violations.push(self.create_violation_with_fix(
                            format!(
                                "Bare URL used: {url}. Consider wrapping in angle brackets: <{url}>"
                            ),
                            line_number + 1, // 1-based line numbers
                            start_pos + 1,   // 1-based column
                            Severity::Warning,
                            fix,
                        ));
                        i = start_pos + url.len();
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

impl MD034 {
    /// Check if the character sequence starts with a URL scheme
    fn starts_with_url_scheme(&self, chars: &[char], pos: usize) -> bool {
        let schemes = ["http://", "https://", "ftp://", "mailto:"];

        for scheme in &schemes {
            let scheme_chars: Vec<char> = scheme.chars().collect();
            if pos + scheme_chars.len() <= chars.len() {
                let mut matches = true;
                for (j, &expected_char) in scheme_chars.iter().enumerate() {
                    if chars[pos + j] != expected_char {
                        matches = false;
                        break;
                    }
                }
                if matches {
                    return true;
                }
            }
        }
        false
    }

    /// Extract a complete URL starting at the given position
    fn extract_url(&self, chars: &[char], start: usize) -> String {
        let mut url = String::new();
        let mut i = start;

        // Extract until we hit whitespace or certain delimiters
        while i < chars.len() {
            let ch = chars[i];
            if ch.is_whitespace() || ch == ')' || ch == ']' || ch == '>' || ch == '"' || ch == '\''
            {
                break;
            }
            url.push(ch);
            i += 1;
        }

        // Remove common trailing punctuation that's probably sentence punctuation
        while let Some(last_char) = url.chars().last() {
            if last_char == '.'
                || last_char == ','
                || last_char == ';'
                || last_char == ':'
                || last_char == '!'
                || last_char == '?'
            {
                url.pop();
            } else {
                break;
            }
        }

        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::Document;
    use mdbook_lint_core::rule::Rule;
    use std::path::PathBuf;

    #[test]
    fn test_md034_no_violations() {
        let content = r#"# Valid URLs

These URLs are properly formatted and should not trigger violations:

- Link: [Google](https://google.com)
- Angle brackets: <https://example.com>
- Email: <mailto:test@example.com>
- Another link: [Local](./page.md)

Text with <https://wrapped-url.com> in angle brackets.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD034;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md034_bare_url_violation() {
        let content = r#"# Document with Bare URL

This has a bare URL: https://example.com that should be wrapped.

Some content here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD034;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Bare URL used"));
        assert!(violations[0].message.contains("https://example.com"));
        assert!(
            violations[0]
                .message
                .contains("Consider wrapping in angle brackets")
        );
        assert_eq!(violations[0].line, 3);
    }

    #[test]
    fn test_md034_multiple_bare_urls() {
        let content = r#"# Multiple Bare URLs

First URL: https://first.com here.
Second URL: http://second.com there.
And an email: mailto:test@example.com end.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD034;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);
        assert!(violations[0].message.contains("https://first.com"));
        assert!(violations[1].message.contains("http://second.com"));
        assert!(violations[2].message.contains("mailto:test@example.com"));
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 4);
        assert_eq!(violations[2].line, 5);
    }

    #[test]
    fn test_md034_ignores_links_and_wrapped_urls() {
        let content = r#"# Mixed URLs

This [valid link](https://good.com) is fine.
This <https://wrapped.com> is also fine.
But this https://bare.com is not.
Another [link](mailto:test@example.com) is good.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD034;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("https://bare.com"));
        assert_eq!(violations[0].line, 5);
    }

    #[test]
    fn test_md034_code_blocks_ignored() {
        let content = r#"# Code Examples

This https://bare-url.com should be detected.

```
This https://code-example.com should be ignored.
```

`This https://inline-code.com should be ignored.`

Another https://bare-url2.com should be detected.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD034;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 11);
    }

    #[test]
    fn test_md034_url_with_trailing_punctuation() {
        let content = r#"# URLs with Punctuation

Visit https://example.com. for more info.
Check out https://test.com, it's great.
See https://other.com; it has details.
The URL is https://final.com: very useful.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD034;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 4);
        // Check that URLs are detected (don't worry about exact punctuation handling)
        assert!(violations[0].message.contains("https://example.com"));
        assert!(violations[1].message.contains("https://test.com"));
        assert!(violations[2].message.contains("https://other.com"));
        assert!(violations[3].message.contains("https://final.com"));
    }

    #[test]
    fn test_md034_complex_urls() {
        let content = r#"# Complex URLs

This https://example.com/path?param=value&other=test#anchor is complex.
This ftp://files.example.com/path/file.txt is an FTP URL.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD034;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(
            violations[0]
                .message
                .contains("https://example.com/path?param=value&other=test#anchor")
        );
        assert!(
            violations[1]
                .message
                .contains("ftp://files.example.com/path/file.txt")
        );
    }

    #[test]
    fn test_md034_no_false_positives() {
        let content = r#"# No False Positives

This text mentions http but not as a URL: "The HTTP protocol is important."
This talks about https: "HTTPS encryption is secure."
This is not a URL: http:something or https:other

Normal text without URLs should be fine.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD034;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }
}
