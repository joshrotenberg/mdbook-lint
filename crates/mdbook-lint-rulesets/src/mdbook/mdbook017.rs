//! MDBOOK017: Check for potentially missing hidden line prefixes in Rust code
//!
//! In mdBook, lines starting with `#` in Rust code blocks are hidden from
//! readers but still compiled. This rule detects common patterns that might
//! indicate missing `#` prefixes.

use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use regex::Regex;
use std::sync::LazyLock;

/// Regex to match code block opening with rust language tag
static RUST_CODE_BLOCK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^```(rust|rs)").unwrap());

/// Patterns that typically should be hidden in documentation examples
const BOILERPLATE_PATTERNS: &[&str] = &[
    "use std::",
    "use crate::",
    "extern crate",
    "fn main() {",
    "fn main(){",
    "pub fn main() {",
    "async fn main() {",
    "#![allow(",
    "#![deny(",
    "#![warn(",
    "#![feature(",
];

/// MDBOOK017: Detects potentially missing hidden line prefixes
///
/// In mdBook Rust code blocks, lines prefixed with `#` are hidden from
/// readers but still compiled. This rule flags common boilerplate patterns
/// that are typically hidden but lack the `#` prefix.
pub struct MDBOOK017;

impl MDBOOK017 {
    /// Check if a line looks like boilerplate that should be hidden
    fn is_boilerplate(&self, line: &str) -> bool {
        let trimmed = line.trim();
        BOILERPLATE_PATTERNS
            .iter()
            .any(|pattern| trimmed.starts_with(pattern))
    }

    /// Check if a line is already hidden (starts with #)
    fn is_hidden(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with('#') && !trimmed.starts_with("#[") && !trimmed.starts_with("#!")
    }

    /// Check if a code block appears to use hidden lines
    fn block_uses_hidden_lines(&self, lines: &[&str]) -> bool {
        lines.iter().any(|line| self.is_hidden(line))
    }
}

impl Rule for MDBOOK017 {
    fn id(&self) -> &'static str {
        "MDBOOK017"
    }

    fn name(&self) -> &'static str {
        "hidden-code-prefix"
    }

    fn description(&self) -> &'static str {
        "Rust code blocks should use # prefix to hide boilerplate from readers"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::MdBook).introduced_in("mdbook-lint v0.12.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> mdbook_lint_core::error::Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let mut in_rust_block = false;
        let mut block_lines: Vec<(usize, &str)> = Vec::new();

        for (line_idx, line) in document.lines.iter().enumerate() {
            let line_num = line_idx + 1;
            let trimmed = line.trim();

            // Check for Rust code block start
            if RUST_CODE_BLOCK_REGEX.is_match(trimmed) {
                in_rust_block = true;
                block_lines.clear();
                continue;
            }

            // Check for code block end
            if in_rust_block && (trimmed == "```" || trimmed.starts_with("~~~")) {
                // Analyze the collected block
                let raw_lines: Vec<&str> = block_lines.iter().map(|(_, l)| *l).collect();

                // Only flag if the block doesn't already use hidden lines
                // If they're using hidden lines elsewhere, they probably know about the feature
                if !self.block_uses_hidden_lines(&raw_lines) {
                    for (bl_num, bl_content) in &block_lines {
                        if self.is_boilerplate(bl_content) {
                            let pattern = BOILERPLATE_PATTERNS
                                .iter()
                                .find(|p| bl_content.trim().starts_with(*p))
                                .unwrap_or(&"boilerplate");

                            violations.push(self.create_violation(
                                format!(
                                    "Consider hiding '{}' with # prefix to focus on the example's core logic",
                                    pattern
                                ),
                                *bl_num,
                                1,
                                Severity::Info,
                            ));
                        }
                    }
                }

                in_rust_block = false;
                block_lines.clear();
                continue;
            }

            // Collect lines inside Rust blocks
            if in_rust_block {
                block_lines.push((line_num, line.as_str()));
            }
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_document(content: &str) -> Document {
        Document::new(content.to_string(), PathBuf::from("test.md")).unwrap()
    }

    #[test]
    fn test_no_boilerplate() {
        let content = r#"# Example

```rust
let x = 42;
println!("{}", x);
```
"#;
        let doc = create_test_document(content);
        let rule = MDBOOK017;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_unhidden_fn_main() {
        let content = r#"# Example

```rust
fn main() {
    let x = 42;
    println!("{}", x);
}
```
"#;
        let doc = create_test_document(content);
        let rule = MDBOOK017;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("fn main()"));
    }

    #[test]
    fn test_unhidden_use_statement() {
        let content = r#"# Example

```rust
use std::collections::HashMap;

let mut map = HashMap::new();
```
"#;
        let doc = create_test_document(content);
        let rule = MDBOOK017;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("use std::"));
    }

    #[test]
    fn test_already_hidden() {
        let content = r#"# Example

```rust
# fn main() {
let x = 42;
println!("{}", x);
# }
```
"#;
        let doc = create_test_document(content);
        let rule = MDBOOK017;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_block_with_some_hidden() {
        // If the block already uses hidden lines, don't flag unhidden boilerplate
        // The author is clearly aware of the feature
        let content = r#"# Example

```rust
# use std::io;
fn main() {
    println!("Hello");
}
```
"#;
        let doc = create_test_document(content);
        let rule = MDBOOK017;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_non_rust_block_ignored() {
        let content = r#"# Example

```python
def main():
    print("Hello")
```
"#;
        let doc = create_test_document(content);
        let rule = MDBOOK017;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_extern_crate() {
        let content = r#"# Example

```rust
extern crate serde;

fn example() {}
```
"#;
        let doc = create_test_document(content);
        let rule = MDBOOK017;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("extern crate"));
    }

    #[test]
    fn test_lint_attributes() {
        let content = r#"# Example

```rust
#![allow(dead_code)]
fn unused() {}
```
"#;
        let doc = create_test_document(content);
        let rule = MDBOOK017;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("#![allow("));
    }

    #[test]
    fn test_multiple_boilerplate() {
        let content = r#"# Example

```rust
use std::collections::HashMap;
use std::io::Read;

fn main() {
    let map: HashMap<i32, i32> = HashMap::new();
}
```
"#;
        let doc = create_test_document(content);
        let rule = MDBOOK017;
        let violations = rule.check(&doc).unwrap();
        // Should flag both use statements and fn main
        assert_eq!(violations.len(), 3);
    }

    #[test]
    fn test_attribute_not_hidden_marker() {
        // #[derive(...)] should NOT be considered a hidden line marker
        let content = r#"# Example

```rust
#[derive(Debug)]
struct Foo;

fn main() {
    println!("{:?}", Foo);
}
```
"#;
        let doc = create_test_document(content);
        let rule = MDBOOK017;
        let violations = rule.check(&doc).unwrap();
        // fn main should be flagged because #[derive] is not a hidden marker
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_rs_alias() {
        let content = r#"# Example

```rs
fn main() {
    println!("test");
}
```
"#;
        let doc = create_test_document(content);
        let rule = MDBOOK017;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }
}
