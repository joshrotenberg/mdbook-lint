//! MDBOOK016: Validate Rust code block attributes
//!
//! Validates that Rust code blocks use valid mdBook/rustdoc attributes
//! like `ignore`, `should_panic`, `no_run`, `compile_fail`, etc.

use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use regex::Regex;
use std::sync::LazyLock;

/// Valid Rust code block attributes recognized by mdBook and rustdoc
const VALID_RUST_ATTRIBUTES: &[&str] = &[
    // mdBook attributes
    "ignore",
    "noplayground",
    "noplaypen",
    "mdbook-runnable",
    "editable",
    "hidelines",
    // rustdoc attributes
    "should_panic",
    "no_run",
    "compile_fail",
    "edition2015",
    "edition2018",
    "edition2021",
    "edition2024",
    // Common valid identifiers
    "rust",
    "text",
    "plain",
];

/// Regex to match code block opening with language tag
static CODE_BLOCK_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^```(\S+)?").unwrap());

/// MDBOOK016: Validates Rust code block attributes
///
/// This rule checks that Rust code blocks use valid attributes.
/// Invalid attributes can cause unexpected behavior in mdBook builds
/// or rustdoc testing.
pub struct MDBOOK016;

impl MDBOOK016 {
    /// Parse attributes from a language tag like "rust,ignore,should_panic"
    fn parse_attributes<'a>(&self, lang_tag: &'a str) -> Vec<&'a str> {
        lang_tag.split(',').map(|s| s.trim()).collect()
    }

    /// Check if a tag indicates this is a Rust code block
    fn is_rust_code_block(&self, lang_tag: &str) -> bool {
        let first_part = lang_tag.split(',').next().unwrap_or("");
        first_part == "rust" || first_part == "rs"
    }

    /// Validate a single attribute
    fn validate_attribute(&self, attr: &str) -> Option<String> {
        // Skip empty attributes
        if attr.is_empty() {
            return None;
        }

        // Skip the language identifier itself
        if attr == "rust" || attr == "rs" {
            return None;
        }

        // Check for hidelines=X pattern
        if attr.starts_with("hidelines=") {
            return None;
        }

        // Check if it's a valid attribute
        if VALID_RUST_ATTRIBUTES.contains(&attr) {
            return None;
        }

        // Check for common typos and suggest corrections
        let suggestion = self.get_typo_suggestion(attr);
        if let Some(suggested) = suggestion {
            Some(format!(
                "Unknown Rust code block attribute '{}'. Did you mean '{}'?",
                attr, suggested
            ))
        } else {
            Some(format!(
                "Unknown Rust code block attribute '{}'. Valid attributes include: ignore, \
                 should_panic, no_run, compile_fail, noplayground, editable",
                attr
            ))
        }
    }

    /// Get suggestion for common typos
    fn get_typo_suggestion(&self, attr: &str) -> Option<&'static str> {
        match attr.to_lowercase().as_str() {
            "shouldpanic" | "should-panic" | "shouldPanic" => Some("should_panic"),
            "norun" | "no-run" | "noRun" => Some("no_run"),
            "compilefail" | "compile-fail" | "compileFail" => Some("compile_fail"),
            "ignored" | "ignor" => Some("ignore"),
            "noplaypen" => Some("noplayground"),
            "editible" | "edittable" => Some("editable"),
            _ => None,
        }
    }
}

impl Rule for MDBOOK016 {
    fn id(&self) -> &'static str {
        "MDBOOK016"
    }

    fn name(&self) -> &'static str {
        "rust-code-block-attributes"
    }

    fn description(&self) -> &'static str {
        "Rust code blocks should use valid mdBook/rustdoc attributes"
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
        let mut in_code_block = false;

        for (line_idx, line) in document.lines.iter().enumerate() {
            let line_num = line_idx + 1;
            let trimmed = line.trim();

            // Check for code block start
            if let Some(caps) = CODE_BLOCK_REGEX.captures(trimmed) {
                if in_code_block {
                    // This is actually closing a code block
                    in_code_block = false;
                    continue;
                }

                in_code_block = true;

                // Check if there's a language tag
                if let Some(lang_match) = caps.get(1) {
                    let lang_tag = lang_match.as_str();

                    // Only validate Rust code blocks
                    if self.is_rust_code_block(lang_tag) {
                        let attrs = self.parse_attributes(lang_tag);

                        for attr in attrs {
                            if let Some(error_msg) = self.validate_attribute(attr) {
                                violations.push(self.create_violation(
                                    error_msg,
                                    line_num,
                                    1,
                                    Severity::Warning,
                                ));
                            }
                        }
                    }
                }
            } else if trimmed == "```" || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
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
    fn test_valid_rust_attributes() {
        let content = "# Code Examples

```rust,ignore
fn main() {}
```

```rust,should_panic
fn main() { panic!(); }
```

```rust,no_run
fn main() {}
```

```rust,compile_fail
fn main() { invalid syntax }
```
";
        let doc = create_test_document(content);
        let rule = MDBOOK016;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_invalid_attribute() {
        let content = "# Code

```rust,invalid_attr
fn main() {}
```
";
        let doc = create_test_document(content);
        let rule = MDBOOK016;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("invalid_attr"));
    }

    #[test]
    fn test_typo_suggestion() {
        let content = "# Code

```rust,shouldpanic
fn main() { panic!(); }
```
";
        let doc = create_test_document(content);
        let rule = MDBOOK016;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("should_panic"));
        assert!(violations[0].message.contains("Did you mean"));
    }

    #[test]
    fn test_multiple_valid_attributes() {
        let content = "# Code

```rust,ignore,editable
fn main() {}
```
";
        let doc = create_test_document(content);
        let rule = MDBOOK016;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_non_rust_block_ignored() {
        let content = "# Code

```python,invalid_attr
def main():
    pass
```

```javascript,also_invalid
console.log('hi');
```
";
        let doc = create_test_document(content);
        let rule = MDBOOK016;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_plain_rust_block() {
        let content = "# Code

```rust
fn main() {}
```
";
        let doc = create_test_document(content);
        let rule = MDBOOK016;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_noplayground_attribute() {
        let content = "# Code

```rust,noplayground
fn main() {}
```
";
        let doc = create_test_document(content);
        let rule = MDBOOK016;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_edition_attributes() {
        let content = "# Code

```rust,edition2021
fn main() {}
```

```rust,edition2018,ignore
fn main() {}
```
";
        let doc = create_test_document(content);
        let rule = MDBOOK016;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_hidelines_attribute() {
        let content = "# Code

```rust,hidelines=#
fn main() {}
```
";
        let doc = create_test_document(content);
        let rule = MDBOOK016;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_mixed_valid_invalid() {
        let content = "# Code

```rust,ignore,badattr,should_panic
fn main() {}
```
";
        let doc = create_test_document(content);
        let rule = MDBOOK016;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("badattr"));
    }

    #[test]
    fn test_rs_alias() {
        let content = "# Code

```rs,ignore
fn main() {}
```
";
        let doc = create_test_document(content);
        let rule = MDBOOK016;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_typo_no_run() {
        let content = "# Code

```rust,norun
fn main() {}
```
";
        let doc = create_test_document(content);
        let rule = MDBOOK016;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("no_run"));
    }
}
