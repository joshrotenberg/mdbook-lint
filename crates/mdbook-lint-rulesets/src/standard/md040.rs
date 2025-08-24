//! MD040: Fenced code blocks should have a language specified
//!
//! This rule checks that fenced code blocks have a language specified for syntax highlighting.

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};
use comrak::nodes::{AstNode, NodeValue};

/// Rule to check that fenced code blocks have a language specified
pub struct MD040;

impl AstRule for MD040 {
    fn id(&self) -> &'static str {
        "MD040"
    }

    fn name(&self) -> &'static str {
        "fenced-code-language"
    }

    fn description(&self) -> &'static str {
        "Fenced code blocks should have a language specified"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Content).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_ast<'a>(&self, document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // Find all code block nodes
        for node in ast.descendants() {
            if let NodeValue::CodeBlock(code_block) = &node.data.borrow().value {
                // Only check fenced code blocks (ignore indented code blocks)
                if code_block.fenced {
                    let info = code_block.info.trim();

                    // Check if language is missing or empty
                    if info.is_empty()
                        && let Some((line, column)) = document.node_position(node)
                    {
                        violations.push(self.create_violation(
                            "Fenced code block is missing language specification".to_string(),
                            line,
                            column,
                            Severity::Warning,
                        ));
                    }
                }
            }
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::Document;
    use mdbook_lint_core::rule::Rule;
    use std::path::PathBuf;

    #[test]
    fn test_md040_no_violations() {
        let content = r#"# Valid Code Blocks

These code blocks have language tags and should not trigger violations:

```rust
fn main() {
    println!("Hello, world!");
}
```

```python
def hello():
    print("Hello, world!")
```

```markdown
# This is markdown
```

```json
{
    "key": "value"
}
```

Some text here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD040;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md040_missing_language_violation() {
        let content = r#"# Document with Missing Language

This code block is missing a language specification:

```
function hello() {
    console.log("Hello, world!");
}
```

Some content here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD040;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("Fenced code block is missing language specification")
        );
        assert_eq!(violations[0].line, 5);
    }

    #[test]
    fn test_md040_multiple_missing_languages() {
        let content = r#"# Multiple Missing Languages

First code block without language:

```
console.log("First block");
```

Some text in between.

```rust
fn main() {
    println!("This one has language");
}
```

Second code block without language:

```
print("Second block")
```

More text.

```
# Third block without language
echo "hello"
```
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD040;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].line, 5);
        assert_eq!(violations[1].line, 19);
        assert_eq!(violations[2].line, 25);
    }

    #[test]
    fn test_md040_indented_code_blocks_ignored() {
        let content = r#"# Indented Code Blocks

This is an indented code block that should be ignored:

    function hello() {
        console.log("This is indented, not fenced");
    }

But this fenced block without language should be detected:

```
function hello() {
    console.log("This is fenced without language");
}
```

And this indented one should still be ignored:

    def hello():
        print("Still indented")
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD040;
        let violations = rule.check(&document).unwrap();

        // Should only detect the fenced code block, not the indented ones
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 11);
    }

    #[test]
    fn test_md040_whitespace_only_info() {
        let content = r#"# Code Block with Whitespace

This code block has only whitespace in the info string:

```
function hello() {
    console.log("Whitespace only info");
}
```

This should also be detected as missing language.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD040;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 5);
    }

    #[test]
    fn test_md040_mixed_fenced_styles() {
        let content = r#"# Mixed Fenced Styles

Backtick fenced block without language:

```
console.log("backticks");
```

Tilde fenced block without language:

~~~
console.log("tildes");
~~~

Tilde fenced block with language:

~~~javascript
console.log("tildes with language");
~~~
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD040;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 5);
        assert_eq!(violations[1].line, 11);
    }

    #[test]
    fn test_md040_empty_code_blocks() {
        let content = r#"# Empty Code Blocks

Empty fenced block without language:

```
```

Empty fenced block with language:

```bash
```

Another empty block without language:

```

```
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD040;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 5);
        assert_eq!(violations[1].line, 15);
    }

    #[test]
    fn test_md040_language_with_attributes() {
        let content = r#"# Code Blocks with Attributes

Code block with language and attributes should be fine:

```rust,no_run
fn main() {
    println!("Hello, world!");
}
```

Code block with just attributes but no language should be detected:

```
function hello() {
    console.log("Hello, world!");
}
```
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD040;
        let violations = rule.check(&document).unwrap();

        // Should only detect the one without a proper language
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 13);
    }
}
