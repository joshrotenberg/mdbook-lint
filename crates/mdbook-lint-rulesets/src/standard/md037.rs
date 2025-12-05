use mdbook_lint_core::Document;
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Fix, Position, Severity, Violation};

/// MD037 - Spaces inside emphasis markers
pub struct MD037;

impl MD037 {
    /// Find positions of code spans (backtick regions) that should be skipped
    fn find_code_span_positions(&self, chars: &[char]) -> Vec<(usize, usize)> {
        let mut code_spans = Vec::new();
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

                // Find matching closing backticks
                let mut j = i;
                while j < chars.len() {
                    if chars[j] == '`' {
                        let mut closing_count = 0;
                        while j < chars.len() && chars[j] == '`' {
                            closing_count += 1;
                            j += 1;
                        }
                        if closing_count == backtick_count {
                            code_spans.push((start, j));
                            i = j;
                            break;
                        }
                    } else {
                        j += 1;
                    }
                }
            } else {
                i += 1;
            }
        }

        code_spans
    }

    /// Check if a position is inside a code span
    fn is_in_code_span(&self, pos: usize, code_spans: &[(usize, usize)]) -> bool {
        code_spans
            .iter()
            .any(|(start, end)| pos >= *start && pos < *end)
    }

    fn find_emphasis_violations(
        &self,
        document: &Document,
        line: &str,
        line_number: usize,
    ) -> Vec<Violation> {
        let mut violations = Vec::new();
        let chars: Vec<char> = line.chars().collect();

        // Find code span positions to skip
        let code_spans = self.find_code_span_positions(&chars);

        // Look for patterns like "** text **", "* text *", etc.
        self.check_pattern(
            document,
            line,
            &chars,
            "**",
            line_number,
            &mut violations,
            &code_spans,
        );
        self.check_pattern(
            document,
            line,
            &chars,
            "__",
            line_number,
            &mut violations,
            &code_spans,
        );
        self.check_single_pattern(
            document,
            line,
            &chars,
            '*',
            line_number,
            &mut violations,
            &code_spans,
        );
        self.check_single_pattern(
            document,
            line,
            &chars,
            '_',
            line_number,
            &mut violations,
            &code_spans,
        );

        violations
    }

    #[allow(clippy::too_many_arguments)]
    fn check_pattern(
        &self,
        _document: &Document,
        line: &str,
        chars: &[char],
        marker: &str,
        line_number: usize,
        violations: &mut Vec<Violation>,
        code_spans: &[(usize, usize)],
    ) {
        let marker_chars: Vec<char> = marker.chars().collect();
        let marker_len = marker_chars.len();
        let mut i = 0;

        while i + marker_len < chars.len() {
            // Skip if we're inside a code span
            if self.is_in_code_span(i, code_spans) {
                i += 1;
                continue;
            }

            // Check if we found the opening marker
            if chars[i..i + marker_len] == marker_chars {
                // Look for closing marker
                let mut j = i + marker_len;
                while j + marker_len <= chars.len() {
                    if chars[j..j + marker_len] == marker_chars {
                        // Found a pair, check for spaces
                        let content_start = i + marker_len;
                        let content_end = j;

                        if content_start < content_end {
                            let has_leading_space = chars[content_start].is_whitespace();
                            let has_trailing_space = chars[content_end - 1].is_whitespace();

                            if has_leading_space || has_trailing_space {
                                // Create fix by removing spaces
                                let content_slice = &chars[content_start..content_end];
                                let content_str: String = content_slice.iter().collect();
                                let fixed_content = content_str.trim();

                                // Convert character indices to byte indices for string slicing
                                let byte_start: usize =
                                    chars[..i].iter().map(|c| c.len_utf8()).sum();
                                let byte_end: usize =
                                    chars[..j + marker_len].iter().map(|c| c.len_utf8()).sum();

                                let mut replacement = String::new();
                                replacement.push_str(&line[..byte_start]);
                                replacement.push_str(marker);
                                replacement.push_str(fixed_content);
                                replacement.push_str(marker);
                                replacement.push_str(&line[byte_end..]);
                                replacement.push('\n');

                                let fix = Fix {
                                    description: "Remove spaces inside emphasis markers"
                                        .to_string(),
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
                                    "Spaces inside emphasis markers".to_string(),
                                    line_number,
                                    i + 1,
                                    Severity::Warning,
                                    fix,
                                ));
                            }
                        }

                        i = j + marker_len;
                        break;
                    }
                    j += 1;
                }

                if j + marker_len > chars.len() {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn check_single_pattern(
        &self,
        _document: &Document,
        line: &str,
        chars: &[char],
        marker: char,
        line_number: usize,
        violations: &mut Vec<Violation>,
        code_spans: &[(usize, usize)],
    ) {
        let mut i = 0;

        while i < chars.len() {
            // Skip if we're inside a code span
            if self.is_in_code_span(i, code_spans) {
                i += 1;
                continue;
            }

            if chars[i] == marker {
                // Make sure this isn't part of a double marker
                if (i > 0 && chars[i - 1] == marker)
                    || (i + 1 < chars.len() && chars[i + 1] == marker)
                {
                    i += 1;
                    continue;
                }

                // Look for closing marker
                let mut j = i + 1;
                while j < chars.len() {
                    if chars[j] == marker {
                        // Make sure this isn't part of a double marker
                        if (j > 0 && chars[j - 1] == marker)
                            || (j + 1 < chars.len() && chars[j + 1] == marker)
                        {
                            j += 1;
                            continue;
                        }

                        // Found a pair, check for spaces
                        let content_start = i + 1;
                        let content_end = j;

                        if content_start < content_end {
                            let has_leading_space = chars[content_start].is_whitespace();
                            let has_trailing_space = chars[content_end - 1].is_whitespace();

                            if has_leading_space || has_trailing_space {
                                // Create fix by removing spaces
                                let content_slice = &chars[content_start..content_end];
                                let content_str: String = content_slice.iter().collect();
                                let fixed_content = content_str.trim();

                                // Convert character indices to byte indices for string slicing
                                let byte_start: usize =
                                    chars[..i].iter().map(|c| c.len_utf8()).sum();
                                let byte_end: usize =
                                    chars[..=j].iter().map(|c| c.len_utf8()).sum();

                                let mut replacement = String::new();
                                replacement.push_str(&line[..byte_start]);
                                replacement.push(marker);
                                replacement.push_str(fixed_content);
                                replacement.push(marker);
                                replacement.push_str(&line[byte_end..]);
                                replacement.push('\n');

                                let fix = Fix {
                                    description: "Remove spaces inside emphasis markers"
                                        .to_string(),
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
                                    "Spaces inside emphasis markers".to_string(),
                                    line_number,
                                    i + 1,
                                    Severity::Warning,
                                    fix,
                                ));
                            }
                        }

                        i = j + 1;
                        break;
                    }
                    j += 1;
                }

                if j >= chars.len() {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
    }
}

impl Rule for MD037 {
    fn id(&self) -> &'static str {
        "MD037"
    }

    fn name(&self) -> &'static str {
        "no-space-in-emphasis"
    }

    fn description(&self) -> &'static str {
        "Spaces inside emphasis markers"
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
        let mut in_fenced_code_block = false;

        for (line_number, line) in lines.iter().enumerate() {
            let line_number = line_number + 1;

            // Track fenced code block state
            let trimmed = line.trim_start();
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_fenced_code_block = !in_fenced_code_block;
                continue;
            }

            // Skip lines inside fenced code blocks
            if in_fenced_code_block {
                continue;
            }

            // Skip indented code blocks (4+ spaces or tab at start)
            if line.starts_with("    ") || line.starts_with('\t') {
                continue;
            }

            violations.extend(self.find_emphasis_violations(document, line, line_number));
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
    fn test_md037_no_violations() {
        let content = r#"Here is some **bold** text.

Here is some *italic* text.

Here is some more __bold__ text.

Here is some more _italic_ text.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD037;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md037_spaces_in_bold() {
        let content = r#"Here is some ** bold ** text.

Here is some __bold __ text.

Here is some __ bold__ text.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD037;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 3);
        assert_eq!(violations[2].line, 5);
    }

    #[test]
    fn test_md037_spaces_in_italic() {
        let content = r#"Here is some * italic * text.

Here is some _italic _ text.

Here is some _ italic_ text.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD037;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 3);
        assert_eq!(violations[2].line, 5);
    }

    #[test]
    fn test_md037_mixed_violations() {
        let content = r#"Here is ** bold ** and * italic * text.

Normal **bold** and *italic* are fine.

But __bold __ and _italic _ are not.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD037;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 4);
        assert_eq!(violations[0].line, 1); // ** bold **
        assert_eq!(violations[1].line, 1); // * italic *
        assert_eq!(violations[2].line, 5); // __bold __
        assert_eq!(violations[3].line, 5); // _italic _
    }

    #[test]
    fn test_md037_no_false_positives() {
        let content = r#"This line has * asterisk but not emphasis.

This line has ** two asterisks but not emphasis.

This has *proper* emphasis.

This has **proper** emphasis too.

Math: 2 times 3 times 4 = 24.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD037;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md037_nested_emphasis() {
        let content = r#"This has ** bold with *italic* inside ** which is wrong.

This has **bold with *italic* inside** which is correct.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD037;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 1);
    }

    #[test]
    fn test_md037_emphasis_at_line_boundaries() {
        let content = r#"** bold at start **

**bold at end **

* italic at start *

*italic at end *
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD037;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 4);
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 3);
        assert_eq!(violations[2].line, 5);
        assert_eq!(violations[3].line, 7);
    }

    #[test]
    fn test_md037_multiple_spaces() {
        let content = r#"Here is some **  bold with multiple spaces  ** text.

Here is some *   italic with multiple spaces   * text.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD037;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 3);
    }

    #[test]
    fn test_md037_fix_leading_space() {
        let content = "Here is some ** bold** text.\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD037;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Remove spaces inside emphasis markers");
        assert_eq!(
            fix.replacement,
            Some("Here is some **bold** text.\n".to_string())
        );
    }

    #[test]
    fn test_md037_fix_trailing_space() {
        let content = "Here is some **bold ** text.\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD037;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Remove spaces inside emphasis markers");
        assert_eq!(
            fix.replacement,
            Some("Here is some **bold** text.\n".to_string())
        );
    }

    #[test]
    fn test_md037_fix_both_spaces() {
        let content = "Here is some ** bold ** text.\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD037;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement,
            Some("Here is some **bold** text.\n".to_string())
        );
    }

    #[test]
    fn test_md037_fix_single_asterisk() {
        let content = "Here is some * italic * text.\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD037;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement,
            Some("Here is some *italic* text.\n".to_string())
        );
    }

    #[test]
    fn test_md037_fix_underscore() {
        let content = "Here is some _ italic _ text.\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD037;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement,
            Some("Here is some _italic_ text.\n".to_string())
        );
    }

    #[test]
    fn test_md037_fix_double_underscore() {
        let content = "Here is some __ bold __ text.\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD037;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement,
            Some("Here is some __bold__ text.\n".to_string())
        );
    }

    #[test]
    fn test_md037_fix_multiple_in_line() {
        let content = "Some ** bold ** and * italic * text.\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD037;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);

        // Both should have fixes
        assert!(violations[0].fix.is_some());
        assert!(violations[1].fix.is_some());
    }

    #[test]
    fn test_md037_can_fix() {
        let rule = MD037;
        assert!(rule.can_fix());
    }

    #[test]
    fn test_md037_code_spans_not_flagged() {
        // Asterisks inside code spans should not be flagged
        let content = r#"Here is some `* not emphasis *` code.

And here is `` ** also not emphasis ** `` with double backticks.

Table with code: `*` should be fine.

Regular `code` with **bold** outside is fine.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD037;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md037_table_with_code_spans() {
        // This is the real-world case from the Rust Book
        let content = r#"| Operator | Example | Description |
|----------|---------|-------------|
| `*`      | `expr * expr` | Arithmetic multiplication |
| `*=`     | `var *= expr` | Multiplication assignment |
| `*`      | `*expr`       | Dereference |
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD037;
        let violations = rule.check(&document).unwrap();
        // All asterisks are inside code spans, should not be flagged
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md037_mixed_code_and_emphasis() {
        let content = r#"Here is `code` with ** bad emphasis ** nearby.

And `more code` with proper **bold** text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD037;
        let violations = rule.check(&document).unwrap();
        // Only the "** bad emphasis **" should be flagged
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 1);
    }

    #[test]
    fn test_md037_find_code_span_positions() {
        let rule = MD037;

        // Single backtick code span
        let chars1: Vec<char> = "text `code` more".chars().collect();
        let spans1 = rule.find_code_span_positions(&chars1);
        assert_eq!(spans1.len(), 1);
        assert_eq!(spans1[0], (5, 11)); // `code`

        // Double backtick code span
        let chars2: Vec<char> = "text ``code`` more".chars().collect();
        let spans2 = rule.find_code_span_positions(&chars2);
        assert_eq!(spans2.len(), 1);
        assert_eq!(spans2[0], (5, 13)); // ``code``

        // Multiple code spans
        let chars3: Vec<char> = "`a` and `b`".chars().collect();
        let spans3 = rule.find_code_span_positions(&chars3);
        assert_eq!(spans3.len(), 2);
    }

    #[test]
    fn test_md037_indented_code_blocks_not_flagged() {
        // Indented code blocks should be skipped entirely
        let content = r#"Here is some regular text.

    println!("the area is {}", x * r * r);

More regular text with **bold**.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD037;
        let violations = rule.check(&document).unwrap();
        // The x * r * r in the indented code block should not be flagged
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md037_fenced_code_blocks_not_flagged() {
        // Fenced code blocks should be skipped entirely
        let content = r#"Here is some regular text.

```rust
println!("the area is {}", x * r * r);
let y = a * b * c;
```

More regular text with **bold**.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD037;
        let violations = rule.check(&document).unwrap();
        // The multiplication in the fenced code block should not be flagged
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md037_ascii_diagrams_in_code_blocks() {
        // ASCII diagrams inside fenced code blocks should not be flagged
        let content = r#"Release schedule:

```text
nightly: * - - * - - *
beta:    +       +
stable:          +
```

End of diagram.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD037;
        let violations = rule.check(&document).unwrap();
        // These are inside a fenced code block, so should not be flagged
        assert_eq!(violations.len(), 0);
    }
}
