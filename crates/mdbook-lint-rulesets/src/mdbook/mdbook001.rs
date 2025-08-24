use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};

/// MDBOOK001: Code blocks should have language tags
///
/// This rule is triggered when code blocks don't have language tags for syntax highlighting.
/// Proper language tags help with documentation clarity and proper rendering in mdBook.
///
/// ## Why This Rule Exists
///
/// mdBook uses language tags for:
/// - Syntax highlighting in rendered output
/// - Proper code formatting and display
/// - Enabling language-specific features (like line numbers, highlighting specific lines)
/// - Improving accessibility for screen readers
/// - Better SEO and content understanding
///
/// ## Examples
///
/// ### ❌ Incorrect (violates rule)
///
/// ````markdown
/// ```
/// fn main() {
///     println!("Hello, world!");
/// }
/// ```
/// ````
///
/// ### ✅ Correct
///
/// ````markdown
/// ```rust
/// fn main() {
///     println!("Hello, world!");
/// }
/// ```
/// ````
///
/// Other valid examples:
///
/// ````markdown
/// ```bash
/// cargo build --release
/// ```
///
/// ```toml
/// [dependencies]
/// serde = "1.0"
/// ```
///
/// ```json
/// {
///   "name": "example",
///   "version": "1.0.0"
/// }
/// ```
/// ````
///
/// ## Special Language Tags
///
/// mdBook supports special language tags:
/// - `text` or `plain` - for plain text without highlighting
/// - `console` - for command-line output
/// - `diff` - for showing differences
/// - `ignore` - for Rust code that shouldn't be tested
/// - `no_run` - for Rust code that compiles but shouldn't run
/// - `should_panic` - for Rust code expected to panic
///
/// ## Configuration
///
/// This rule has no configuration options. All code blocks should have language tags.
///
/// ## When to Disable
///
/// Consider disabling this rule if:
/// - You have many legacy code blocks without language tags
/// - You're using a custom mdBook renderer that doesn't require language tags
/// - Your documentation intentionally uses generic code blocks
pub struct MDBOOK001;

impl AstRule for MDBOOK001 {
    fn id(&self) -> &'static str {
        "MDBOOK001"
    }

    fn name(&self) -> &'static str {
        "code-block-language"
    }

    fn description(&self) -> &'static str {
        "Code blocks should have language tags for proper syntax highlighting"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::MdBook).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_ast<'a>(
        &self,
        document: &Document,
        ast: &'a AstNode<'a>,
    ) -> mdbook_lint_core::error::Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let code_blocks = document.code_blocks(ast);

        for code_block in code_blocks {
            if let NodeValue::CodeBlock(code_block_data) = &code_block.data.borrow().value {
                // Only check fenced code blocks (skip indented code blocks)
                if code_block_data.fenced {
                    let info = code_block_data.info.trim();

                    // Check if the info string is empty or just whitespace
                    if info.is_empty() {
                        let (line, column) = document.node_position(code_block).unwrap_or((1, 1));

                        let message = "Code block is missing language tag for syntax highlighting"
                            .to_string();

                        violations.push(self.create_violation(
                            message,
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
    use mdbook_lint_core::rule::Rule;
    use std::path::PathBuf;

    #[test]
    fn test_mdbook001_valid_fenced_code_blocks() {
        let content = r#"# Test

```rust
fn main() {
    println!("Hello, world!");
}
```

```bash
echo "Hello from bash"
```

```json
{"key": "value"}
```
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MDBOOK001;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_mdbook001_missing_language_tags() {
        let content = r#"# Test

```
fn main() {
    println!("No language tag");
}
```

Some text.

```
echo "Another block without language"
```
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MDBOOK001;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);

        assert_eq!(violations[0].rule_id, "MDBOOK001");
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[0].severity, Severity::Warning);
        assert!(violations[0].message.contains("missing language tag"));

        assert_eq!(violations[1].rule_id, "MDBOOK001");
        assert_eq!(violations[1].line, 11);
        assert_eq!(violations[1].severity, Severity::Warning);
        assert!(violations[1].message.contains("missing language tag"));
    }

    #[test]
    fn test_mdbook001_indented_code_blocks_ignored() {
        let content = r#"# Test

This is normal text.

    // This is an indented code block
    fn main() {
        println!("This should be ignored");
    }

And some more text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MDBOOK001;
        let violations = rule.check(&document).unwrap();

        // Indented code blocks should be ignored
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_mdbook001_mixed_code_blocks() {
        let content = r#"# Test

```rust
// Good: has language tag
fn main() {}
```

```
// Bad: missing language tag
fn bad() {}
```

    // Indented: should be ignored
    fn indented() {}

```bash
# Good: has language tag
echo "hello"
```
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MDBOOK001;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 8);
        assert!(violations[0].message.contains("missing language tag"));
    }

    #[test]
    fn test_mdbook001_whitespace_only_info() {
        let content = r#"```
// Code block with whitespace-only info string
fn test() {}
```"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MDBOOK001;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("missing language tag"));
    }

    #[test]
    fn test_mdbook001_no_code_blocks() {
        let content = r#"# Test Document

This is just regular text with no code blocks.

## Another Section

Still no code blocks here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MDBOOK001;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }
}
