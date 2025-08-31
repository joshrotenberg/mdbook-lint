use mdbook_lint_core::Document;
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Fix, Position, Severity, Violation};

/// MD037 - Spaces inside emphasis markers
pub struct MD037;

impl MD037 {
    fn find_emphasis_violations(
        &self,
        document: &Document,
        line: &str,
        line_number: usize,
    ) -> Vec<Violation> {
        let mut violations = Vec::new();
        let chars: Vec<char> = line.chars().collect();

        // Look for patterns like "** text **", "* text *", etc.
        self.check_pattern(document, line, &chars, "**", line_number, &mut violations);
        self.check_pattern(document, line, &chars, "__", line_number, &mut violations);
        self.check_single_pattern(document, line, &chars, '*', line_number, &mut violations);
        self.check_single_pattern(document, line, &chars, '_', line_number, &mut violations);

        violations
    }

    fn check_pattern(
        &self,
        _document: &Document,
        line: &str,
        chars: &[char],
        marker: &str,
        line_number: usize,
        violations: &mut Vec<Violation>,
    ) {
        let marker_chars: Vec<char> = marker.chars().collect();
        let marker_len = marker_chars.len();
        let mut i = 0;

        while i + marker_len < chars.len() {
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

                                let mut replacement = String::new();
                                replacement.push_str(&line[..i]);
                                replacement.push_str(marker);
                                replacement.push_str(fixed_content);
                                replacement.push_str(marker);
                                replacement.push_str(&line[j + marker_len..]);
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

    fn check_single_pattern(
        &self,
        _document: &Document,
        line: &str,
        chars: &[char],
        marker: char,
        line_number: usize,
        violations: &mut Vec<Violation>,
    ) {
        let mut i = 0;

        while i < chars.len() {
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

                                let mut replacement = String::new();
                                replacement.push_str(&line[..i]);
                                replacement.push(marker);
                                replacement.push_str(fixed_content);
                                replacement.push(marker);
                                replacement.push_str(&line[j + 1..]);
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
        let lines = document.content.lines();

        for (line_number, line) in lines.enumerate() {
            let line_number = line_number + 1;
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
}
