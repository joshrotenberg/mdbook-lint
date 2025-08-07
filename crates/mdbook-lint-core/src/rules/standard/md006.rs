use crate::Document;
use crate::error::Result;
use crate::rule::{Rule, RuleCategory, RuleMetadata};
use crate::violation::{Severity, Violation};

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
        RuleMetadata::stable(RuleCategory::Formatting)
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
                        violations.push(
                            self.create_violation(
                                "Consider starting bulleted lists at the beginning of the line"
                                    .to_string(),
                                line_number,
                                1,
                                Severity::Warning,
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
            if !line.trim().is_empty() && line.starts_with("    ") {
                in_indented_block = true;
                in_code_block[i] = true;
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
    use crate::Document;
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
}
