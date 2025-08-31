//! MD031: Fenced code blocks should be surrounded by blank lines
//!
//! This rule is triggered when fenced code blocks are not surrounded by blank lines.

use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Fix, Position, Severity, Violation},
};

/// MD031: Fenced code blocks should be surrounded by blank lines
///
/// This rule checks that fenced code blocks (```) have blank lines before and after them,
/// unless they are at the start or end of the document.
pub struct MD031;

impl AstRule for MD031 {
    fn id(&self) -> &'static str {
        "MD031"
    }

    fn name(&self) -> &'static str {
        "blanks-around-fences"
    }

    fn description(&self) -> &'static str {
        "Fenced code blocks should be surrounded by blank lines"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Formatting).introduced_in("markdownlint v0.1.0")
    }

    fn can_fix(&self) -> bool {
        true
    }

    fn check_ast<'a>(&self, document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let code_blocks = document.code_blocks(ast);

        for code_block in code_blocks {
            // Only check fenced code blocks, not indented ones
            if let NodeValue::CodeBlock(code_block_data) = &code_block.data.borrow().value
                && code_block_data.fenced
                && let Some((line, column)) = document.node_position(code_block)
            {
                // Check for blank line before the code block
                if !self.has_blank_line_before(document, line) {
                    // Create fix by inserting a blank line before the code block
                    let fix = Fix {
                        description: "Add blank line before fenced code block".to_string(),
                        replacement: Some("\n".to_string()),
                        start: Position {
                            line: line - 1,
                            column: if line > 1 {
                                document.lines.get(line - 2).map_or(1, |l| l.len() + 1)
                            } else {
                                1
                            },
                        },
                        end: Position {
                            line: line - 1,
                            column: if line > 1 {
                                document.lines.get(line - 2).map_or(1, |l| l.len() + 1)
                            } else {
                                1
                            },
                        },
                    };

                    violations.push(self.create_violation_with_fix(
                        "Fenced code block should be preceded by a blank line".to_string(),
                        line,
                        column,
                        Severity::Warning,
                        fix,
                    ));
                }

                // Check for blank line after the code block
                let end_line = self.find_code_block_end_line(document, line);
                if !self.has_blank_line_after(document, end_line) {
                    // Create fix by inserting a blank line after the code block
                    let fix = Fix {
                        description: "Add blank line after fenced code block".to_string(),
                        replacement: Some("\n".to_string()),
                        start: Position {
                            line: end_line,
                            column: document.lines.get(end_line - 1).map_or(1, |l| l.len() + 1),
                        },
                        end: Position {
                            line: end_line,
                            column: document.lines.get(end_line - 1).map_or(1, |l| l.len() + 1),
                        },
                    };

                    violations.push(self.create_violation_with_fix(
                        "Fenced code block should be followed by a blank line".to_string(),
                        end_line,
                        1,
                        Severity::Warning,
                        fix,
                    ));
                }
            }
        }

        Ok(violations)
    }
}

impl MD031 {
    /// Check if there's a blank line before the given line number
    fn has_blank_line_before(&self, document: &Document, line_num: usize) -> bool {
        // If this is the first line, no blank line needed
        if line_num <= 1 {
            return true;
        }

        // Check if the previous line is blank
        if let Some(prev_line) = document.lines.get(line_num - 2) {
            prev_line.trim().is_empty()
        } else {
            true // Start of document
        }
    }

    /// Check if there's a blank line after the given line number
    fn has_blank_line_after(&self, document: &Document, line_num: usize) -> bool {
        // If this is the last line, no blank line needed
        if line_num >= document.lines.len() {
            return true;
        }

        // Check if the next line is blank
        if let Some(next_line) = document.lines.get(line_num) {
            next_line.trim().is_empty()
        } else {
            true // End of document
        }
    }

    /// Find the end line of a code block starting at the given line
    fn find_code_block_end_line(&self, document: &Document, start_line: usize) -> usize {
        let start_idx = start_line - 1; // Convert to 0-based

        // Look for the opening fence
        if let Some(start_line_content) = document.lines.get(start_idx) {
            let trimmed = start_line_content.trim_start();
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                let fence_chars = if trimmed.starts_with("```") {
                    "```"
                } else {
                    "~~~"
                };
                let fence_length = trimmed
                    .chars()
                    .take_while(|&c| c == fence_chars.chars().next().unwrap())
                    .count();

                // Find the closing fence
                for (idx, line) in document.lines.iter().enumerate().skip(start_idx + 1) {
                    let line_trimmed = line.trim();
                    if line_trimmed.starts_with(fence_chars) {
                        let closing_fence_length = line_trimmed
                            .chars()
                            .take_while(|&c| c == fence_chars.chars().next().unwrap())
                            .count();
                        if closing_fence_length >= fence_length
                            && line_trimmed.len() == closing_fence_length
                        {
                            return idx + 1; // Convert back to 1-based
                        }
                    }
                }
            }
        }

        // If we can't find the end, assume it's the start line
        start_line
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::rule::Rule;
    use std::path::PathBuf;

    #[test]
    fn test_md031_valid_fenced_blocks() {
        let content = r#"# Title

```rust
fn main() {
    println!("Hello, world!");
}
```

Some text after.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD031;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md031_missing_blank_before() {
        let content = r#"# Title
```rust
fn main() {
    println!("Hello, world!");
}
```

Some text after.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD031;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD031");
        assert!(violations[0].message.contains("preceded by a blank line"));
        assert_eq!(violations[0].line, 2);
        assert_eq!(violations[0].severity, Severity::Warning);
    }

    #[test]
    fn test_md031_missing_blank_after() {
        let content = r#"# Title

```rust
fn main() {
    println!("Hello, world!");
}
```
Some text after.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD031;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD031");
        assert!(violations[0].message.contains("followed by a blank line"));
        assert_eq!(violations[0].severity, Severity::Warning);
    }

    #[test]
    fn test_md031_missing_both_blanks() {
        let content = r#"# Title
```rust
fn main() {
    println!("Hello, world!");
}
```
Some text after.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD031;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("preceded by a blank line"));
        assert!(violations[1].message.contains("followed by a blank line"));
    }

    #[test]
    fn test_md031_start_of_document() {
        let content = r#"```rust
fn main() {
    println!("Hello, world!");
}
```

Some text after.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD031;
        let violations = rule.check(&document).unwrap();

        // Should be valid at start of document
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md031_end_of_document() {
        let content = r#"# Title

```rust
fn main() {
    println!("Hello, world!");
}
```"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD031;
        let violations = rule.check(&document).unwrap();

        // Should be valid at end of document
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md031_multiple_code_blocks() {
        let content = r#"# Title

```rust
fn main() {
    println!("Hello, world!");
}
```
Some text.
```bash
echo "test"
```

End.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD031;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        // First block missing blank after
        assert!(violations[0].message.contains("followed by a blank line"));
        // Second block missing blank before
        assert!(violations[1].message.contains("preceded by a blank line"));
    }

    #[test]
    fn test_md031_tildes_fenced_blocks() {
        let content = r#"# Title

~~~rust
fn main() {
    println!("Hello, world!");
}
~~~

Some text after.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD031;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md031_indented_code_blocks_ignored() {
        let content = r#"# Title
Here is some code:

    def hello():
        print("Hello, world!")

Some text after.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD031;
        let violations = rule.check(&document).unwrap();

        // Indented code blocks should be ignored
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md031_different_fence_lengths() {
        let content = r#"# Title

````rust
fn main() {
    println!("```");
}
````

Some text after.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD031;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md031_fix_missing_blank_before() {
        let content = r#"# Title
Some text here
```rust
fn main() {}
```

Another line"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD031;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Add blank line before fenced code block");
        assert_eq!(fix.replacement, Some("\n".to_string()));
    }

    #[test]
    fn test_md031_fix_missing_blank_after() {
        let content = r#"# Title

```rust
fn main() {}
```
Some text here"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD031;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Add blank line after fenced code block");
        assert_eq!(fix.replacement, Some("\n".to_string()));
    }

    #[test]
    fn test_md031_fix_missing_both() {
        let content = r#"# Title
Some text before
```rust
fn main() {}
```
Some text after"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD031;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);

        // First violation - missing blank before
        assert!(violations[0].fix.is_some());
        let fix1 = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix1.description, "Add blank line before fenced code block");

        // Second violation - missing blank after
        assert!(violations[1].fix.is_some());
        let fix2 = violations[1].fix.as_ref().unwrap();
        assert_eq!(fix2.description, "Add blank line after fenced code block");
    }

    #[test]
    fn test_md031_fix_tilde_fence() {
        let content = r#"# Title
Some text
~~~python
print("hello")
~~~
More text"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD031;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);

        // Both violations should have fixes
        assert!(violations[0].fix.is_some());
        assert!(violations[1].fix.is_some());
    }

    #[test]
    fn test_md031_fix_multiple_blocks() {
        let content = r#"# Title
First block:
```rust
code1
```
Second block:
```python
code2
```
End"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD031;
        let violations = rule.check(&document).unwrap();

        // Should have 4 violations (before and after each block)
        assert_eq!(violations.len(), 4);

        // All should have fixes
        for violation in &violations {
            assert!(violation.fix.is_some());
        }
    }

    #[test]
    fn test_md031_can_fix() {
        let rule = MD031;
        assert!(mdbook_lint_core::AstRule::can_fix(&rule));
    }
}
