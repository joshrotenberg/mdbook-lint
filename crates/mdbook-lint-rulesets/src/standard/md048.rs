//! MD048: Code fence style consistency
//!
//! This rule checks that fenced code blocks use a consistent fence style (backticks vs tildes) throughout the document.

use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Fix, Position, Severity, Violation},
};

/// Rule to check code fence style consistency
pub struct MD048 {
    /// Preferred fence style
    style: FenceStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FenceStyle {
    /// Use backticks (```)
    Backtick,
    /// Use tildes (~~~)
    Tilde,
    /// Detect from first usage in document
    Consistent,
}

impl MD048 {
    /// Create a new MD048 rule with consistent style detection
    pub fn new() -> Self {
        Self {
            style: FenceStyle::Consistent,
        }
    }

    /// Create a new MD048 rule with specific style preference
    #[allow(dead_code)]
    pub fn with_style(style: FenceStyle) -> Self {
        Self { style }
    }

    /// Create MD048 from configuration
    pub fn from_config(config: &toml::Value) -> Self {
        let mut rule = Self::new();

        if let Some(style_str) = config.get("style").and_then(|v| v.as_str()) {
            rule.style = match style_str.to_lowercase().as_str() {
                "backtick" => FenceStyle::Backtick,
                "tilde" => FenceStyle::Tilde,
                "consistent" => FenceStyle::Consistent,
                _ => FenceStyle::Consistent, // Default fallback
            };
        }

        rule
    }

    /// Determine the fence style of a code block
    fn get_fence_style(&self, node: &AstNode) -> Option<FenceStyle> {
        if let NodeValue::CodeBlock(code_block) = &node.data.borrow().value {
            if code_block.fenced {
                // Check the fence character - comrak stores the fence info
                if code_block.fence_char as char == '`' {
                    Some(FenceStyle::Backtick)
                } else if code_block.fence_char as char == '~' {
                    Some(FenceStyle::Tilde)
                } else {
                    None
                }
            } else {
                // Not a fenced code block, ignore
                None
            }
        } else {
            None
        }
    }

    /// Get line and column position for a node
    fn get_position<'a>(&self, node: &'a AstNode<'a>) -> (usize, usize) {
        let data = node.data.borrow();
        let pos = data.sourcepos;
        (pos.start.line, pos.start.column)
    }

    /// Walk AST and find all fence style violations
    fn check_node<'a>(
        &self,
        node: &'a AstNode<'a>,
        document: &Document,
        violations: &mut Vec<Violation>,
        expected_style: &mut Option<FenceStyle>,
    ) {
        if let NodeValue::CodeBlock(_) = &node.data.borrow().value
            && let Some(current_style) = self.get_fence_style(node)
        {
            if let Some(expected) = expected_style {
                // Check consistency with established style
                if *expected != current_style {
                    let (line, column) = self.get_position(node);
                    let expected_char = match expected {
                        FenceStyle::Backtick => "`",
                        FenceStyle::Tilde => "~",
                        FenceStyle::Consistent => "consistent", // shouldn't happen
                    };
                    let found_char = match current_style {
                        FenceStyle::Backtick => "`",
                        FenceStyle::Tilde => "~",
                        FenceStyle::Consistent => "consistent", // shouldn't happen
                    };

                    // Create fix by replacing fence characters
                    let node_data = node.data.borrow();
                    if let NodeValue::CodeBlock(code_block) = &node_data.value {
                        let fence_length = code_block.fence_length;
                        let expected_fence = expected_char.repeat(fence_length);

                        // Get the lines that need to be fixed
                        let start_line = line;
                        let content_lines: Vec<&str> = code_block.literal.lines().collect();
                        let end_line = start_line + content_lines.len() + 1; // +1 for closing fence

                        // Build the fixed code block
                        let mut fixed_lines = Vec::new();

                        // Opening fence with info string
                        let info = &code_block.info;
                        if info.is_empty() {
                            fixed_lines.push(expected_fence.clone());
                        } else {
                            fixed_lines.push(format!("{}{}", expected_fence, info));
                        }

                        // Content lines
                        for content_line in &content_lines {
                            fixed_lines.push(content_line.to_string());
                        }

                        // Closing fence
                        fixed_lines.push(expected_fence.clone());

                        let fix = Fix {
                            description: format!(
                                "Change code fence style from '{}' to '{}'",
                                found_char, expected_char
                            ),
                            replacement: Some(fixed_lines.join("\n") + "\n"),
                            start: Position {
                                line: start_line,
                                column: 1,
                            },
                            end: Position {
                                line: end_line,
                                column: document
                                    .lines
                                    .get(end_line - 1)
                                    .map(|l| l.len() + 1)
                                    .unwrap_or(1),
                            },
                        };

                        violations.push(self.create_violation_with_fix(
                                format!(
                                    "Code fence style inconsistent - expected '{expected_char}' but found '{found_char}'"
                                ),
                                line,
                                column,
                                Severity::Warning,
                                fix,
                            ));
                    }
                }
            } else {
                // First fenced code block found - establish the style
                match self.style {
                    FenceStyle::Backtick => *expected_style = Some(FenceStyle::Backtick),
                    FenceStyle::Tilde => *expected_style = Some(FenceStyle::Tilde),
                    FenceStyle::Consistent => *expected_style = Some(current_style),
                }
            }
        }

        // Recursively check children
        for child in node.children() {
            self.check_node(child, document, violations, expected_style);
        }
    }
}

impl Default for MD048 {
    fn default() -> Self {
        Self::new()
    }
}

impl AstRule for MD048 {
    fn id(&self) -> &'static str {
        "MD048"
    }

    fn name(&self) -> &'static str {
        "code-fence-style"
    }

    fn description(&self) -> &'static str {
        "Code fence style should be consistent"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Formatting).introduced_in("mdbook-lint v0.1.0")
    }

    fn can_fix(&self) -> bool {
        true
    }

    fn check_ast<'a>(&self, document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let mut expected_style = match self.style {
            FenceStyle::Backtick => Some(FenceStyle::Backtick),
            FenceStyle::Tilde => Some(FenceStyle::Tilde),
            FenceStyle::Consistent => None, // Detect from first usage
        };

        self.check_node(ast, document, &mut violations, &mut expected_style);
        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::rule::Rule;
    use std::path::PathBuf;

    fn create_test_document(content: &str) -> Document {
        Document::new(content.to_string(), PathBuf::from("test.md")).unwrap()
    }

    #[test]
    fn test_md048_consistent_backtick_style() {
        let content = r#"Here is some backtick fenced code:

```rust
fn main() {
    println!("Hello");
}
```

And another backtick block:

```python
print("Hello")
```
"#;

        let document = create_test_document(content);
        let rule = MD048::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md048_consistent_tilde_style() {
        let content = r#"Here is some tilde fenced code:

~~~rust
fn main() {
    println!("Hello");
}
~~~

And another tilde block:

~~~python
print("Hello")
~~~
"#;

        let document = create_test_document(content);
        let rule = MD048::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md048_mixed_fence_styles_violation() {
        let content = r#"Here is backtick fenced code:

```rust
fn main() {
    println!("Hello");
}
```

And here is tilde fenced code:

~~~python
print("Hello")
~~~
"#;

        let document = create_test_document(content);
        let rule = MD048::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD048");
        assert!(violations[0].message.contains("expected '`' but found '~'"));
    }

    #[test]
    fn test_md048_preferred_backtick_style() {
        let content = r#"Here is tilde fenced code:

~~~rust
fn main() {}
~~~
"#;

        let document = create_test_document(content);
        let rule = MD048::with_style(FenceStyle::Backtick);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("expected '`' but found '~'"));
    }

    #[test]
    fn test_md048_preferred_tilde_style() {
        let content = r#"Here is backtick fenced code:

```rust
fn main() {}
```
"#;

        let document = create_test_document(content);
        let rule = MD048::with_style(FenceStyle::Tilde);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("expected '~' but found '`'"));
    }

    #[test]
    fn test_md048_indented_code_blocks_ignored() {
        let content = r#"Backtick fenced:

```rust
fn main() {}
```

Indented code (should be ignored):

    print("hello")

Tilde fenced (should be flagged):

~~~python
print("world")
~~~
"#;

        let document = create_test_document(content);
        let rule = MD048::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("expected '`' but found '~'"));
    }

    #[test]
    fn test_md048_multiple_backtick_blocks() {
        let content = r#"First block:

```rust
fn main() {}
```

Second block:

```python
print("hello")
```

Third block:

```javascript
console.log("hello");
```
"#;

        let document = create_test_document(content);
        let rule = MD048::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md048_multiple_tilde_blocks() {
        let content = r#"First block:

~~~rust
fn main() {}
~~~

Second block:

~~~python
print("hello")
~~~

Third block:

~~~javascript
console.log("hello");
~~~
"#;

        let document = create_test_document(content);
        let rule = MD048::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md048_mixed_multiple_violations() {
        let content = r#"Start with backticks:

```rust
fn main() {}
```

Then tildes (violation):

~~~python
print("hello")
~~~

Then backticks again:

```javascript
console.log("hello");
```

And tildes again (violation):

~~~bash
echo "hello"
~~~
"#;

        let document = create_test_document(content);
        let rule = MD048::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("expected '`' but found '~'"));
        assert!(violations[1].message.contains("expected '`' but found '~'"));
    }

    #[test]
    fn test_md048_no_fenced_code_blocks() {
        let content = r#"This document has no fenced code blocks.

Just regular text and paragraphs.

    This is indented code, not fenced.

And maybe some `inline code` but no fenced blocks.
"#;

        let document = create_test_document(content);
        let rule = MD048::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md048_tilde_first_determines_style() {
        let content = r#"Start with tildes:

~~~rust
fn main() {}
~~~

Then backticks should be flagged:

```python
print("hello")
```
"#;

        let document = create_test_document(content);
        let rule = MD048::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("expected '~' but found '`'"));
    }

    #[test]
    fn test_md048_with_languages() {
        let content = r#"Different languages, same fence style:

```rust
fn main() {}
```

```python
def hello():
    pass
```

```javascript
function hello() {}
```
"#;

        let document = create_test_document(content);
        let rule = MD048::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md048_fix_fence_style() {
        let content = r#"First block:

```rust
fn main() {}
```

Second block uses different style:

~~~python
print("hello")
~~~
"#;

        let document = create_test_document(content);
        let rule = MD048::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 9);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Change code fence style from '~' to '`'");
        assert!(fix.replacement.as_ref().unwrap().starts_with("```python"));
        assert!(fix.replacement.as_ref().unwrap().ends_with("```\n"));
    }

    #[test]
    fn test_md048_can_fix() {
        let rule = MD048::new();
        assert!(mdbook_lint_core::AstRule::can_fix(&rule));
    }
}
