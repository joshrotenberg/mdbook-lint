//! MD034: Bare URL without angle brackets
//!
//! This rule checks for bare URLs that should be enclosed in angle brackets.

use comrak::nodes::{AstNode, NodeValue};
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

    fn can_fix(&self) -> bool {
        true
    }

    fn check_ast<'a>(&self, document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // Get code block line ranges from AST
        let code_block_ranges = self.get_code_block_line_ranges(ast);

        for (line_number, line) in document.lines.iter().enumerate() {
            let line_num = line_number + 1; // 1-based line numbers

            // Skip lines inside code blocks
            if code_block_ranges
                .iter()
                .any(|(start, end)| line_num >= *start && line_num <= *end)
            {
                continue;
            }

            // Skip reference link definitions [ref]: url
            // These are valid markdown and URLs in them should not be flagged
            let trimmed = line.trim_start();
            if trimmed.starts_with('[') {
                // Check if this looks like a reference definition
                if let Some(colon_pos) = trimmed.find("]:") {
                    // Check if the bracket content before ]: doesn't contain other brackets
                    let bracket_content = &trimmed[1..colon_pos];
                    if !bracket_content.contains('[') && !bracket_content.contains(']') {
                        // This is a reference link definition, skip it
                        continue;
                    }
                }
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
    /// Get all code block line ranges from the AST
    fn get_code_block_line_ranges<'a>(&self, ast: &'a AstNode<'a>) -> Vec<(usize, usize)> {
        let mut ranges = Vec::new();
        self.collect_code_block_ranges(ast, &mut ranges);
        ranges
    }

    /// Recursively collect code block ranges from AST nodes
    #[allow(clippy::only_used_in_recursion)]
    fn collect_code_block_ranges<'a>(
        &self,
        node: &'a AstNode<'a>,
        ranges: &mut Vec<(usize, usize)>,
    ) {
        if let NodeValue::CodeBlock(_) = &node.data.borrow().value {
            let sourcepos = node.data.borrow().sourcepos;
            if sourcepos.start.line > 0 && sourcepos.end.line > 0 {
                ranges.push((sourcepos.start.line, sourcepos.end.line));
            }
        }
        for child in node.children() {
            self.collect_code_block_ranges(child, ranges);
        }
    }

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
This https://code-example.com should be ignored in fenced block.
```

`This https://inline-code.com should be ignored.`

Another https://bare-url2.com should be detected.

    This https://indented-code.com should be ignored in indented block.
    Another line with https://another-indented.com in code block.

Final https://final-url.com should be detected.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD034;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 11);
        assert_eq!(violations[2].line, 16);
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

    #[test]
    fn test_md034_fix_simple_url() {
        let content = "Visit https://example.com for more info.";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD034;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement.as_ref().unwrap(), "<https://example.com>");
        assert_eq!(fix.description, "Wrap URL in angle brackets");
    }

    #[test]
    fn test_md034_fix_multiple_urls() {
        let content = "Check https://first.com and http://second.com for details.";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD034;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);

        // First URL
        let fix1 = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix1.replacement.as_ref().unwrap(), "<https://first.com>");

        // Second URL
        let fix2 = violations[1].fix.as_ref().unwrap();
        assert_eq!(fix2.replacement.as_ref().unwrap(), "<http://second.com>");
    }

    #[test]
    fn test_md034_fix_complex_url() {
        let content =
            "API docs: https://api.example.com/v1/users?limit=10&offset=0#pagination here.";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD034;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement.as_ref().unwrap(),
            "<https://api.example.com/v1/users?limit=10&offset=0#pagination>"
        );
    }

    #[test]
    fn test_md034_fix_ftp_url() {
        let content = "Download from ftp://files.example.com/path/file.txt today.";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD034;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement.as_ref().unwrap(),
            "<ftp://files.example.com/path/file.txt>"
        );
    }

    #[test]
    fn test_md034_fix_mailto() {
        let content = "Contact us at mailto:support@example.com for help.";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD034;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement.as_ref().unwrap(),
            "<mailto:support@example.com>"
        );
    }

    #[test]
    fn test_md034_fix_url_with_trailing_punctuation() {
        let content = "Visit https://example.com. Also check https://test.com, please.";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD034;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        // URLs should not include trailing punctuation
        assert_eq!(
            violations[0]
                .fix
                .as_ref()
                .unwrap()
                .replacement
                .as_ref()
                .unwrap(),
            "<https://example.com>"
        );
        assert_eq!(
            violations[1]
                .fix
                .as_ref()
                .unwrap()
                .replacement
                .as_ref()
                .unwrap(),
            "<https://test.com>"
        );
    }

    #[test]
    fn test_md034_fix_position_accuracy() {
        let content = "Text before https://example.com text after.";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD034;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.start.line, 1);
        assert_eq!(fix.start.column, 13); // Start of URL (1-based)
        assert_eq!(fix.end.line, 1);
        assert_eq!(fix.end.column, 32); // End of URL (1-based, exclusive)
    }

    #[test]
    fn test_md034_fix_multiple_lines() {
        let content = "First line with https://first.com\nSecond line with http://second.com\nThird line with ftp://third.com";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD034;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].line, 1);
        assert_eq!(
            violations[0]
                .fix
                .as_ref()
                .unwrap()
                .replacement
                .as_ref()
                .unwrap(),
            "<https://first.com>"
        );
        assert_eq!(violations[1].line, 2);
        assert_eq!(
            violations[1]
                .fix
                .as_ref()
                .unwrap()
                .replacement
                .as_ref()
                .unwrap(),
            "<http://second.com>"
        );
        assert_eq!(violations[2].line, 3);
        assert_eq!(
            violations[2]
                .fix
                .as_ref()
                .unwrap()
                .replacement
                .as_ref()
                .unwrap(),
            "<ftp://third.com>"
        );
    }

    #[test]
    fn test_md034_fix_url_at_start_of_line() {
        let content = "https://example.com is a great site.";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD034;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement.as_ref().unwrap(), "<https://example.com>");
        assert_eq!(fix.start.column, 1);
    }

    #[test]
    fn test_md034_fix_url_at_end_of_line() {
        let content = "Check out this site: https://example.com";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD034;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement.as_ref().unwrap(), "<https://example.com>");
    }

    #[test]
    fn test_md034_reference_link_definitions_ignored() {
        let content = r#"# Reference Links

Text with [reference link][ref] here.

[ref]: https://example.com "Title"
[another]: https://test.com
[spaced]:   https://spaced.com   "With spaces"

But this bare URL https://bare.com should be detected.

[nested-brackets]: https://should-not-work[].com
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD034;
        let violations = rule.check(&document).unwrap();

        // Should only detect the bare URL, not the reference definitions
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("https://bare.com"));
        assert_eq!(violations[0].line, 9);
    }
}
