//! MD039: Spaces inside link text
//!
//! This rule checks for unnecessary spaces at the beginning or end of link text.

use crate::error::Result;
use crate::rule::{Rule, RuleCategory, RuleMetadata};
use crate::{
    Document,
    violation::{Severity, Violation},
};

/// Rule to check for spaces inside link text
pub struct MD039;

impl MD039 {
    /// Find link violations in a line
    fn check_line_links(&self, line: &str, line_number: usize) -> Vec<Violation> {
        let mut violations = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '[' {
                // Skip if this is an image (preceded by !)
                if i > 0 && chars[i - 1] == '!' {
                    i += 1;
                    continue;
                }

                // Look for closing bracket
                if let Some(end_bracket) = self.find_closing_bracket(&chars, i + 1) {
                    let link_text = &chars[i + 1..end_bracket];

                    // Check if this is followed by a link URL or reference
                    let is_link = if end_bracket + 1 < chars.len() {
                        chars[end_bracket + 1] == '(' || chars[end_bracket + 1] == '['
                    } else {
                        false
                    };

                    if is_link && self.has_unnecessary_spaces(link_text) {
                        violations.push(self.create_violation(
                            "Spaces inside link text".to_string(),
                            line_number,
                            i + 1, // Convert to 1-based column
                            Severity::Warning,
                        ));
                    }

                    i = end_bracket + 1;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        violations
    }

    /// Find the closing bracket for a link
    fn find_closing_bracket(&self, chars: &[char], start: usize) -> Option<usize> {
        let mut bracket_count = 1;
        let mut i = start;

        while i < chars.len() && bracket_count > 0 {
            match chars[i] {
                '[' => bracket_count += 1,
                ']' => bracket_count -= 1,
                '\\' => {
                    // Skip escaped character
                    i += 1;
                }
                _ => {}
            }

            if bracket_count == 0 {
                return Some(i);
            }

            i += 1;
        }

        None
    }

    /// Check if link text has unnecessary leading or trailing spaces
    fn has_unnecessary_spaces(&self, link_text: &[char]) -> bool {
        if link_text.is_empty() {
            return false;
        }

        // Check for leading space
        let has_leading_space = link_text[0].is_whitespace();

        // Check for trailing space
        let has_trailing_space = link_text[link_text.len() - 1].is_whitespace();

        has_leading_space || has_trailing_space
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

impl Rule for MD039 {
    fn id(&self) -> &'static str {
        "MD039"
    }

    fn name(&self) -> &'static str {
        "no-space-in-links"
    }

    fn description(&self) -> &'static str {
        "Spaces inside link text"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Content).introduced_in("mdbook-lint v0.1.0")
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

            violations.extend(self.check_line_links(line, line_number));
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::Rule;
    use std::path::PathBuf;

    fn create_test_document(content: &str) -> Document {
        Document::new(content.to_string(), PathBuf::from("test.md")).unwrap()
    }

    #[test]
    fn test_md039_normal_links_valid() {
        let content = r#"Here is a [normal link](http://example.com).

Another [link with text](http://example.com) works fine.

Reference link [with text][ref] is also okay.

[ref]: http://example.com
"#;

        let document = create_test_document(content);
        let rule = MD039;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md039_leading_space_violation() {
        let content = r#"Here is a [ leading space](http://example.com) link.

Another [ spaced link](http://example.com) here.
"#;

        let document = create_test_document(content);
        let rule = MD039;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].rule_id, "MD039");
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 3);
    }

    #[test]
    fn test_md039_trailing_space_violation() {
        let content = r#"Here is a [trailing space ](http://example.com) link.

Another [spaced link ](http://example.com) here.
"#;

        let document = create_test_document(content);
        let rule = MD039;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 3);
    }

    #[test]
    fn test_md039_both_spaces_violation() {
        let content = r#"Here is a [ both spaces ](http://example.com) link.

Multiple [ spaced   ](http://example.com) spaces.
"#;

        let document = create_test_document(content);
        let rule = MD039;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 3);
    }

    #[test]
    fn test_md039_reference_links() {
        let content = r#"Good [reference link][good] here.

Bad [ spaced reference][bad] link.

Another [reference with space ][also-bad] here.

[good]: http://example.com
[bad]: http://example.com
[also-bad]: http://example.com
"#;

        let document = create_test_document(content);
        let rule = MD039;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 5);
    }

    #[test]
    fn test_md039_nested_brackets() {
        let content = r#"This has [link with [nested] brackets](http://example.com).

This has [ link with [nested] and space](http://example.com).
"#;

        let document = create_test_document(content);
        let rule = MD039;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3);
    }

    #[test]
    fn test_md039_not_links() {
        let content = r#"This has [brackets] but no link.

This has [ spaced brackets] but no link.

This has [reference] but no definition.
"#;

        let document = create_test_document(content);
        let rule = MD039;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0); // Not links, so no violations
    }

    #[test]
    fn test_md039_images_ignored() {
        let content = r#"This has ![ spaced alt text](image.png) which is an image.

And ![normal alt](image.png) text.
"#;

        let document = create_test_document(content);
        let rule = MD039;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0); // Images are not checked by this rule
    }

    #[test]
    fn test_md039_code_blocks_ignored() {
        let content = r#"This has [normal link](http://example.com).

```
This has [ spaced link](http://example.com) in code.
```

This has [ spaced link](http://example.com) that should be flagged.
"#;

        let document = create_test_document(content);
        let rule = MD039;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 7);
    }

    #[test]
    fn test_md039_escaped_brackets() {
        let content = r#"This has [link with \] escaped bracket](http://example.com).

This has [ link with \] and space](http://example.com).
"#;

        let document = create_test_document(content);
        let rule = MD039;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3);
    }

    #[test]
    fn test_md039_autolinks() {
        let content = r#"Autolinks like <http://example.com> are not checked.

Email autolinks <user@example.com> are also not checked.

Regular [normal link](http://example.com) is fine.

Bad [ spaced link](http://example.com) is flagged.
"#;

        let document = create_test_document(content);
        let rule = MD039;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 7);
    }

    #[test]
    fn test_md039_empty_link_text() {
        let content = r#"Empty link [](http://example.com) is not flagged for spaces.

Link with just space [ ](http://example.com) is flagged.
"#;

        let document = create_test_document(content);
        let rule = MD039;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3);
    }

    #[test]
    fn test_md039_multiple_links_per_line() {
        let content = r#"Multiple [good link](http://example.com) and [ bad link](http://example.com) on same line.

More [good](http://example.com) and [also good](http://example.com) links.
"#;

        let document = create_test_document(content);
        let rule = MD039;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 1);
    }
}
