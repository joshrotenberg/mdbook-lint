//! MD046: Code block style consistency
//!
//! This rule checks that code blocks use a consistent style (fenced vs indented) throughout the document.

use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};

/// Rule to check code block style consistency
pub struct MD046 {
    /// Preferred code block style
    style: CodeBlockStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CodeBlockStyle {
    /// Use fenced code blocks (``` or ~~~)
    Fenced,
    /// Use indented code blocks (4 spaces or 1 tab)
    Indented,
    /// Detect from first usage in document
    Consistent,
}

impl MD046 {
    /// Create a new MD046 rule with consistent style detection
    pub fn new() -> Self {
        Self {
            style: CodeBlockStyle::Consistent,
        }
    }

    /// Create a new MD046 rule with specific style preference
    #[allow(dead_code)]
    pub fn with_style(style: CodeBlockStyle) -> Self {
        Self { style }
    }

    /// Determine if a code block is fenced or indented
    fn get_code_block_style(&self, node: &AstNode) -> Option<CodeBlockStyle> {
        if let NodeValue::CodeBlock(code_block) = &node.data.borrow().value {
            // Check if it's a fenced code block by looking for fence markers
            if code_block.fenced {
                Some(CodeBlockStyle::Fenced)
            } else {
                Some(CodeBlockStyle::Indented)
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

    /// Walk AST and find all code block style violations
    fn check_node<'a>(
        &self,
        node: &'a AstNode<'a>,
        violations: &mut Vec<Violation>,
        expected_style: &mut Option<CodeBlockStyle>,
    ) {
        if let NodeValue::CodeBlock(_) = &node.data.borrow().value
            && let Some(current_style) = self.get_code_block_style(node)
        {
            if let Some(expected) = expected_style {
                // Check consistency with established style
                if *expected != current_style {
                    let (line, column) = self.get_position(node);
                    let expected_name = match expected {
                        CodeBlockStyle::Fenced => "fenced",
                        CodeBlockStyle::Indented => "indented",
                        CodeBlockStyle::Consistent => "consistent", // shouldn't happen
                    };
                    let found_name = match current_style {
                        CodeBlockStyle::Fenced => "fenced",
                        CodeBlockStyle::Indented => "indented",
                        CodeBlockStyle::Consistent => "consistent", // shouldn't happen
                    };

                    violations.push(self.create_violation(
                            format!(
                                "Code block style inconsistent - expected {expected_name} but found {found_name}"
                            ),
                            line,
                            column,
                            Severity::Warning,
                        ));
                }
            } else {
                // First code block found - establish the style
                match self.style {
                    CodeBlockStyle::Fenced => *expected_style = Some(CodeBlockStyle::Fenced),
                    CodeBlockStyle::Indented => *expected_style = Some(CodeBlockStyle::Indented),
                    CodeBlockStyle::Consistent => *expected_style = Some(current_style),
                }
            }
        }

        // Recursively check children
        for child in node.children() {
            self.check_node(child, violations, expected_style);
        }
    }
}

impl Default for MD046 {
    fn default() -> Self {
        Self::new()
    }
}

impl AstRule for MD046 {
    fn id(&self) -> &'static str {
        "MD046"
    }

    fn name(&self) -> &'static str {
        "code-block-style"
    }

    fn description(&self) -> &'static str {
        "Code block style should be consistent"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Formatting).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_ast<'a>(&self, _document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let mut expected_style = match self.style {
            CodeBlockStyle::Fenced => Some(CodeBlockStyle::Fenced),
            CodeBlockStyle::Indented => Some(CodeBlockStyle::Indented),
            CodeBlockStyle::Consistent => None, // Detect from first usage
        };

        self.check_node(ast, &mut violations, &mut expected_style);
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
    fn test_md046_consistent_fenced_style() {
        let content = r#"Here is some fenced code:

```rust
fn main() {
    println!("Hello");
}
```

And another fenced block:

```python
print("Hello")
```
"#;

        let document = create_test_document(content);
        let rule = MD046::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md046_consistent_indented_style() {
        let content = r#"Here is some indented code:

    fn main() {
        println!("Hello");
    }

And another indented block:

    print("Hello")
"#;

        let document = create_test_document(content);
        let rule = MD046::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md046_mixed_styles_violation() {
        let content = r#"Here is fenced code:

```rust
fn main() {
    println!("Hello");
}
```

And here is indented code:

    print("Hello")
"#;

        let document = create_test_document(content);
        let rule = MD046::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD046");
        assert!(
            violations[0]
                .message
                .contains("expected fenced but found indented")
        );
    }

    #[test]
    fn test_md046_preferred_fenced_style() {
        let content = r#"Here is indented code:

    print("Hello")
"#;

        let document = create_test_document(content);
        let rule = MD046::with_style(CodeBlockStyle::Fenced);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("expected fenced but found indented")
        );
    }

    #[test]
    fn test_md046_preferred_indented_style() {
        let content = r#"Here is fenced code:

```rust
fn main() {}
```
"#;

        let document = create_test_document(content);
        let rule = MD046::with_style(CodeBlockStyle::Indented);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("expected indented but found fenced")
        );
    }

    #[test]
    fn test_md046_multiple_fenced_blocks() {
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
        let rule = MD046::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md046_multiple_indented_blocks() {
        let content = r#"First block:

    fn main() {}

Second block:

    print("hello")

Third block:

    console.log("hello");
"#;

        let document = create_test_document(content);
        let rule = MD046::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md046_mixed_multiple_violations() {
        let content = r#"Start with fenced:

```rust
fn main() {}
```

Then indented:

    print("hello")

Then fenced again:

```javascript
console.log("hello");
```

And indented again:

    another_function()
"#;

        let document = create_test_document(content);
        let rule = MD046::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2); // Two violations: second and fourth blocks
        assert!(
            violations[0]
                .message
                .contains("expected fenced but found indented")
        );
        assert!(
            violations[1]
                .message
                .contains("expected fenced but found indented")
        );
    }

    #[test]
    fn test_md046_no_code_blocks() {
        let content = r#"This document has no code blocks.

Just regular text and paragraphs.

And maybe some `inline code` but no blocks.
"#;

        let document = create_test_document(content);
        let rule = MD046::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md046_tilde_fenced_blocks() {
        let content = r#"Using tilde fences:

~~~rust
fn main() {}
~~~

And backtick fences:

```python
print("hello")
```
"#;

        let document = create_test_document(content);
        let rule = MD046::new();
        let violations = rule.check(&document).unwrap();
        // Both are fenced style, so should be consistent
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md046_fenced_vs_indented_first_determines() {
        let content = r#"Start with indented:

    fn main() {}

Then fenced should be flagged:

```python
print("hello")
```
"#;

        let document = create_test_document(content);
        let rule = MD046::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("expected indented but found fenced")
        );
    }
}
