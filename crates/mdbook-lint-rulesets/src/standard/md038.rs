use mdbook_lint_core::Document;
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Fix, Position, Severity, Violation};

/// MD038 - Spaces inside code span elements
pub struct MD038;

impl MD038 {
    fn find_code_span_violations(&self, line: &str, line_number: usize) -> Vec<Violation> {
        let mut violations = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();

        let mut i = 0;
        while i < len {
            if chars[i] == '`' {
                // Check if this looks like a closing backtick from a previous line
                // In that case, skip it as it's not a valid opening for a code span on this line
                // A closing backtick is typically preceded by non-whitespace content
                if i > 0 && !chars[i - 1].is_whitespace() && chars[i - 1] != '`' {
                    // This backtick is likely closing a code span that started on a previous line
                    // Skip past all consecutive backticks
                    while i < len && chars[i] == '`' {
                        i += 1;
                    }
                    continue;
                }

                // Count consecutive backticks
                let mut backtick_count = 0;
                let start = i;
                while i < len && chars[i] == '`' {
                    backtick_count += 1;
                    i += 1;
                }

                // Find the closing backticks
                if let Some(end_start) = self.find_closing_backticks(&chars, i, backtick_count) {
                    let content_start = start + backtick_count;
                    let content_end = end_start;

                    if content_start < content_end {
                        let content = &chars[content_start..content_end];

                        // Check for violations
                        if self.has_unnecessary_spaces(content) {
                            // Create fix by removing spaces
                            let fixed_content = self.fix_code_span_content(content);
                            let backticks = "`".repeat(backtick_count);

                            // Convert character indices to byte indices for string slicing
                            let byte_start: usize =
                                chars[..start].iter().map(|c| c.len_utf8()).sum();
                            let byte_end: usize = chars[..end_start + backtick_count]
                                .iter()
                                .map(|c| c.len_utf8())
                                .sum();

                            let mut replacement = String::new();
                            replacement.push_str(&line[..byte_start]);
                            replacement.push_str(&backticks);
                            replacement.push_str(&fixed_content);
                            replacement.push_str(&backticks);
                            replacement.push_str(&line[byte_end..]);
                            replacement.push('\n');

                            let fix = Fix {
                                description: "Remove spaces inside code span".to_string(),
                                replacement: Some(replacement),
                                start: Position {
                                    line: line_number,
                                    column: 1,
                                },
                                end: Position {
                                    line: line_number,
                                    column: line.len() + 1,
                                },
                            };

                            violations.push(self.create_violation_with_fix(
                                "Spaces inside code span elements".to_string(),
                                line_number,
                                start + 1, // Convert to 1-based column
                                Severity::Warning,
                                fix,
                            ));
                        }
                    }

                    i = end_start + backtick_count;
                } else {
                    // No matching closing backticks found, move on
                    break;
                }
            } else {
                i += 1;
            }
        }

        violations
    }

    fn find_closing_backticks(&self, chars: &[char], start: usize, count: usize) -> Option<usize> {
        let mut i = start;
        while i + count <= chars.len() {
            if chars[i] == '`' {
                let mut consecutive = 0;
                let mut j = i;
                while j < chars.len() && chars[j] == '`' {
                    consecutive += 1;
                    j += 1;
                }

                if consecutive == count {
                    return Some(i);
                }

                i = j;
            } else {
                i += 1;
            }
        }
        None
    }

    fn has_unnecessary_spaces(&self, content: &[char]) -> bool {
        if content.is_empty() {
            return false;
        }

        // Check for spaces-only content (this is allowed)
        if content.iter().all(|&c| c.is_whitespace()) {
            return false;
        }

        // Check for special case: content that contains backticks
        // In this case, single leading and trailing spaces are allowed and required
        let content_str: String = content.iter().collect();
        if content_str.contains('`') {
            // For backtick-containing content, spaces are required and allowed
            return false;
        }

        // Check for unnecessary leading space
        let has_leading_space = content[0].is_whitespace();

        // Check for unnecessary trailing space
        let has_trailing_space = content[content.len() - 1].is_whitespace();

        // If there are multiple leading or trailing spaces, that's definitely wrong
        if content.len() >= 2 {
            let has_multiple_leading = has_leading_space && content[1].is_whitespace();
            let has_multiple_trailing =
                has_trailing_space && content[content.len() - 2].is_whitespace();

            if has_multiple_leading || has_multiple_trailing {
                return true;
            }
        }

        // For normal content, any leading or trailing space is unnecessary
        has_leading_space || has_trailing_space
    }

    fn fix_code_span_content(&self, content: &[char]) -> String {
        if content.is_empty() {
            return String::new();
        }

        // Special case: if content contains backticks, preserve spaces as they may be required
        let content_str: String = content.iter().collect();
        if content_str.contains('`') {
            return content_str;
        }

        // Remove leading and trailing spaces
        content_str.trim().to_string()
    }

    /// Get code block ranges to exclude from checking
    fn get_code_block_ranges(&self, lines: &[&str]) -> Vec<bool> {
        let mut in_code_block = vec![false; lines.len()];
        let mut in_fenced_block = false;
        let mut in_html_comment = false;

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

            // Check for HTML comments
            // Handle multi-line HTML comments
            if in_html_comment {
                in_code_block[i] = true;
                if line.contains("-->") {
                    in_html_comment = false;
                }
                continue;
            }

            // Check if line starts an HTML comment
            if line.contains("<!--") {
                // If it also contains -->, it's a single-line comment
                // But we should still skip backticks in that line
                in_code_block[i] = true;
                if !line.contains("-->") || line.find("<!--") > line.find("-->") {
                    // Multi-line comment starting
                    in_html_comment = true;
                }
            }
        }

        in_code_block
    }
}

impl Rule for MD038 {
    fn id(&self) -> &'static str {
        "MD038"
    }

    fn name(&self) -> &'static str {
        "no-space-in-code"
    }

    fn description(&self) -> &'static str {
        "Spaces inside code span elements"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Formatting)
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

        for (line_number, line) in lines.iter().enumerate() {
            let line_number = line_number + 1;

            // Skip lines inside code blocks
            if in_code_block[line_number - 1] {
                continue;
            }

            violations.extend(self.find_code_span_violations(line, line_number));
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::Document;
    use std::path::PathBuf;

    #[test]
    fn test_md038_no_violations() {
        let content = r#"Here is some `code` text.

More text with `another code span` here.

Complex code: `some.method()` works.

Multiple backticks: ``code with `backticks` inside``.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD038;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md038_leading_space() {
        let content = r#"Here is some ` code` with leading space.

Another example: ` another` here.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD038;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 3);
    }

    #[test]
    fn test_md038_trailing_space() {
        let content = r#"Here is some `code ` with trailing space.

Another example: `another ` here.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD038;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 3);
    }

    #[test]
    fn test_md038_both_spaces() {
        let content = r#"Here is some ` code ` with both spaces.

Multiple spaces: `   code   ` is also wrong.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD038;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 3);
    }

    #[test]
    fn test_md038_backtick_escaping_allowed() {
        let content = r#"To show a backtick: `` ` ``.

To show backticks: `` `backticks` ``.

Another way: `` backtick` ``.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD038;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0); // These should be allowed
    }

    #[test]
    fn test_md038_spaces_only_allowed() {
        let content = r#"Single space: ` `.

Multiple spaces: `   `.

Tab character: `	`.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD038;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0); // Spaces-only content is allowed
    }

    #[test]
    fn test_md038_multiple_code_spans() {
        let content = r#"Good: `code1` and `code2` and `code3`.

Bad: ` code1` and `code2 ` and ` code3 `.

Mixed: `good` and ` bad` and `also good`.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD038;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 4);
        assert_eq!(violations[0].line, 3); // ` code1`
        assert_eq!(violations[1].line, 3); // `code2 `
        assert_eq!(violations[2].line, 3); // ` code3 `
        assert_eq!(violations[3].line, 5); // ` bad`
    }

    #[test]
    fn test_md038_triple_backticks_ignored() {
        let content = r#"```
This is a code block, not a code span.
` spaces here` should not be flagged.
```

But this `code span ` should be flagged.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD038;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 6);
    }

    #[test]
    fn test_md038_unmatched_backticks() {
        let content = r#"This line has ` unmatched backtick.

This line has normal `code` and then ` another unmatched.

Normal content here.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD038;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0); // The `code` span has no spaces, so no violations
    }

    #[test]
    fn test_md038_empty_code_spans() {
        let content = r#"Empty code span: ``.

Another empty: ``.

With spaces only: ` `.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD038;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0); // Empty spans are not violations
    }

    #[test]
    fn test_md038_fix_leading_space() {
        let content = "Here is some ` code` with leading space.\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD038;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Remove spaces inside code span");
        assert_eq!(
            fix.replacement,
            Some("Here is some `code` with leading space.\n".to_string())
        );
    }

    #[test]
    fn test_md038_fix_trailing_space() {
        let content = "Here is some `code ` with trailing space.\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD038;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement,
            Some("Here is some `code` with trailing space.\n".to_string())
        );
    }

    #[test]
    fn test_md038_fix_both_spaces() {
        let content = "Here is some ` code ` with both spaces.\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD038;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement,
            Some("Here is some `code` with both spaces.\n".to_string())
        );
    }

    #[test]
    fn test_md038_fix_multiple_spaces() {
        let content = "Multiple: `   code   ` here.\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD038;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement,
            Some("Multiple: `code` here.\n".to_string())
        );
    }

    #[test]
    fn test_md038_fix_multiple_in_line() {
        let content = "Bad: ` code1` and `code2 ` and ` code3 `.\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD038;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);

        // All should have fixes
        for violation in &violations {
            assert!(violation.fix.is_some());
        }
    }

    #[test]
    fn test_md038_fix_preserves_backticks() {
        let content = "To show backticks: `` `backticks` ``.\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD038;
        let violations = rule.check(&document).unwrap();

        // This should not have violations because spaces are required with backticks
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md038_fix_double_backticks() {
        let content = "Double: `` code `` here.\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD038;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement,
            Some("Double: ``code`` here.\n".to_string())
        );
    }

    #[test]
    fn test_md038_can_fix() {
        let rule = MD038;
        assert!(rule.can_fix());
    }

    #[test]
    fn test_md038_multiline_code_span_not_flagged() {
        // Issue #277: Code spans that appear to continue from previous line should not be flagged
        // Line that starts with "build`" is the closing of a code span from the previous line
        let content = r#"Using `cargo run` is more convenient than having to remember to run `cargo
build` and then use the whole path to the binary, so most developers use `cargo
run`.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD038;
        let violations = rule.check(&document).unwrap();

        // Should NOT flag because:
        // - Line 2 starts with "build`" which is the closing of `cargo from line 1
        // - Line 3 starts with "run`" which is the closing of `cargo from line 2
        // All actual code spans (`cargo run`, `cargo build`, `cargo run`) are valid
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md038_backtick_after_word_skipped() {
        // A backtick immediately following a word is likely a closing backtick
        let content = "Some word` should not start a span here `other.\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD038;
        let violations = rule.check(&document).unwrap();

        // The "word`" should be skipped as an opening, so we shouldn't see a violation
        // for " should not start a span here "
        assert_eq!(violations.len(), 0);
    }
}
