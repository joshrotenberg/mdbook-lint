use mdbook_lint_core::Document;
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Fix, Position, Severity, Violation};

/// MD006 - Consider starting bulleted lists at the beginning of the line
pub struct MD006;

impl Rule for MD006 {
    fn id(&self) -> &'static str {
        "MD006"
    }

    fn name(&self) -> &'static str {
        "ul-start-left"
    }

    fn description(&self) -> &'static str {
        "Consider starting bulleted lists at the beginning of the line"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::deprecated(
            RuleCategory::Formatting,
            "Removed from markdownlint; MD007 covers list indentation more comprehensively",
            Some("MD007"),
        )
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

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Skip lines inside code blocks
            if in_code_block[line_number - 1] {
                continue;
            }

            // Check for unordered list markers (*, +, -) that are indented
            if let Some(first_char_pos) = line.find(|c: char| !c.is_whitespace())
                && first_char_pos > 0
            {
                let remaining = &line[first_char_pos..];

                // Check if this is a list item (starts with *, +, or - followed by space)
                if let Some(first_char) = remaining.chars().next()
                    && matches!(first_char, '*' | '+' | '-')
                    && remaining.len() > 1
                {
                    let second_char = remaining.chars().nth(1).unwrap();
                    if second_char.is_whitespace() {
                        // This is an indented unordered list item
                        // Create fix by removing the indentation
                        let fixed_line = format!("{}\n", &line[first_char_pos..]);
                        let fix = Fix {
                            description: format!("Remove {} spaces of indentation", first_char_pos),
                            replacement: Some(fixed_line),
                            start: Position {
                                line: line_number,
                                column: 1,
                            },
                            end: Position {
                                line: line_number,
                                column: line.len() + 1,
                            },
                        };

                        violations.push(
                            self.create_violation_with_fix(
                                "Consider starting bulleted lists at the beginning of the line"
                                    .to_string(),
                                line_number,
                                1,
                                Severity::Warning,
                                fix,
                            ),
                        );
                    }
                }
            }
        }

        Ok(violations)
    }
}

impl MD006 {
    /// Get code block ranges to exclude from checking
    fn get_code_block_ranges(&self, lines: &[&str]) -> Vec<bool> {
        let mut in_code_block = vec![false; lines.len()];
        let mut in_fenced_block = false;
        let mut in_indented_block = false;

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

            // Check for indented code blocks (4+ spaces at start of line)
            // But not if it's a list item
            if !line.trim().is_empty() && line.starts_with("    ") {
                let trimmed_after_indent = line[4..].trim_start();
                // Check if this is a list item (starts with *, +, or - followed by space)
                let is_list_item = if let Some(first_char) = trimmed_after_indent.chars().next() {
                    matches!(first_char, '*' | '+' | '-')
                        && trimmed_after_indent.len() > 1
                        && trimmed_after_indent
                            .chars()
                            .nth(1)
                            .is_some_and(|c| c.is_whitespace())
                } else {
                    false
                };

                if !is_list_item {
                    in_indented_block = true;
                    in_code_block[i] = true;
                }
            } else if !line.trim().is_empty() {
                in_indented_block = false;
            } else if in_indented_block {
                // Empty lines continue indented code blocks
                in_code_block[i] = true;
            }
        }

        in_code_block
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::Document;
    use std::path::PathBuf;

    #[test]
    fn test_md006_no_violations() {
        let content = r#"# Heading

* Item 1
* Item 2
* Item 3

Some text

+ Item A
+ Item B

More text

- Item X
- Item Y
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD006;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md006_indented_list() {
        let content = r#"# Heading

Some text
 * Indented item 1
 * Indented item 2

More text
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD006;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 4);
        assert_eq!(violations[1].line, 5);
        assert!(
            violations[0]
                .message
                .contains("Consider starting bulleted lists")
        );
    }

    #[test]
    fn test_md006_mixed_indentation() {
        let content = r#"* Good item
 * Bad item
* Good item
  + Another bad item
- Good item
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD006;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 2);
        assert_eq!(violations[1].line, 4);
    }

    #[test]
    fn test_md006_nested_lists_valid() {
        let content = r#"* Item 1
  * Nested item (this triggers the rule - it's indented)
  * Another nested item
* Item 2
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD006;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2); // The nested items are indented
        assert_eq!(violations[0].line, 2);
        assert_eq!(violations[1].line, 3);
    }

    #[test]
    fn test_md006_code_blocks_ignored() {
        let content = r#"# Heading

```
 * This is in a code block
 * Should not trigger the rule
```

 * But this should trigger it
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD006;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 8);
    }

    #[test]
    fn test_md006_blockquotes_ignored() {
        let content = r#"# Heading

> * This is in a blockquote
> * Should not trigger the rule

 * But this should trigger it
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD006;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 6);
    }

    #[test]
    fn test_md006_different_markers() {
        let content = r#" * Asterisk indented
 + Plus indented
 - Dash indented
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD006;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 2);
        assert_eq!(violations[2].line, 3);
    }

    #[test]
    fn test_md006_not_list_markers() {
        let content = r#" * Not followed by space
 *Not followed by space
 - Not followed by space
 -Not followed by space
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD006;
        let violations = rule.check(&document).unwrap();
        // First and third lines have space after marker, so they trigger the rule
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 3);
    }

    #[test]
    fn test_md006_tab_indentation() {
        let content = "\t* Tab indented item\n\t+ Another tab indented";

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD006;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 2);
    }

    #[test]
    fn test_md006_fix_simple_indentation() {
        let content = " * Single space indented\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD006;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Remove 1 spaces of indentation");
        assert_eq!(
            fix.replacement,
            Some("* Single space indented\n".to_string())
        );
    }

    #[test]
    fn test_md006_fix_multiple_spaces() {
        let content = "    * Four spaces indented\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD006;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Remove 4 spaces of indentation");
        assert_eq!(
            fix.replacement,
            Some("* Four spaces indented\n".to_string())
        );
    }

    #[test]
    fn test_md006_fix_tab_indentation() {
        let content = "\t* Tab indented item\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD006;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Remove 1 spaces of indentation");
        assert_eq!(fix.replacement, Some("* Tab indented item\n".to_string()));
    }

    #[test]
    fn test_md006_fix_multiple_items() {
        let content = " * First item\n  + Second item\n   - Third item\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD006;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);

        // First item - 1 space
        assert!(violations[0].fix.is_some());
        let fix1 = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix1.description, "Remove 1 spaces of indentation");
        assert_eq!(fix1.replacement, Some("* First item\n".to_string()));

        // Second item - 2 spaces
        assert!(violations[1].fix.is_some());
        let fix2 = violations[1].fix.as_ref().unwrap();
        assert_eq!(fix2.description, "Remove 2 spaces of indentation");
        assert_eq!(fix2.replacement, Some("+ Second item\n".to_string()));

        // Third item - 3 spaces
        assert!(violations[2].fix.is_some());
        let fix3 = violations[2].fix.as_ref().unwrap();
        assert_eq!(fix3.description, "Remove 3 spaces of indentation");
        assert_eq!(fix3.replacement, Some("- Third item\n".to_string()));
    }

    #[test]
    fn test_md006_fix_preserves_content() {
        let content = "  * Item with **bold** and *italic* text\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD006;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement,
            Some("* Item with **bold** and *italic* text\n".to_string())
        );
    }

    #[test]
    fn test_md006_fix_mixed_indentation() {
        let content = " * Space indented\n\t+ Tab indented\n  - Two space indented\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD006;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);

        // All should have fixes that remove indentation
        for violation in &violations {
            assert!(violation.fix.is_some());
        }

        assert_eq!(
            violations[0].fix.as_ref().unwrap().replacement,
            Some("* Space indented\n".to_string())
        );
        assert_eq!(
            violations[1].fix.as_ref().unwrap().replacement,
            Some("+ Tab indented\n".to_string())
        );
        assert_eq!(
            violations[2].fix.as_ref().unwrap().replacement,
            Some("- Two space indented\n".to_string())
        );
    }

    #[test]
    fn test_md006_fix_different_markers() {
        let content = "  * Asterisk item\n  + Plus item\n  - Dash item\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD006;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);

        // All should preserve their original markers
        assert_eq!(
            violations[0].fix.as_ref().unwrap().replacement,
            Some("* Asterisk item\n".to_string())
        );
        assert_eq!(
            violations[1].fix.as_ref().unwrap().replacement,
            Some("+ Plus item\n".to_string())
        );
        assert_eq!(
            violations[2].fix.as_ref().unwrap().replacement,
            Some("- Dash item\n".to_string())
        );
    }

    #[test]
    fn test_md006_can_fix() {
        let rule = MD006;
        assert!(rule.can_fix());
    }
}
