//! MD057: Reserved for future use
//!
//! This rule number is reserved in markdownlint for future implementation.
//! It exists as a placeholder to maintain complete rule numbering.

use crate::error::Result;
use crate::rule::{Rule, RuleMetadata};
use crate::{Document, violation::Violation};
use comrak::nodes::AstNode;

/// Placeholder for reserved rule MD057
pub struct MD057;

impl Rule for MD057 {
    fn id(&self) -> &'static str {
        "MD057"
    }

    fn name(&self) -> &'static str {
        "reserved"
    }

    fn description(&self) -> &'static str {
        "Reserved for future use"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::reserved("Reserved for future implementation in markdownlint")
            .introduced_in("mdbook-lint v0.1.0")
    }

    fn check_with_ast<'a>(
        &self,
        _document: &Document,
        _ast: Option<&'a AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        // Reserved rules never produce violations
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::create_document;

    #[test]
    fn test_md057_rule_properties() {
        let rule = MD057;
        assert_eq!(rule.id(), "MD057");
        assert_eq!(rule.name(), "reserved");
        assert_eq!(rule.description(), "Reserved for future use");
    }

    #[test]
    fn test_md057_metadata() {
        let rule = MD057;
        let metadata = rule.metadata();
        assert_eq!(metadata.stability, crate::rule::RuleStability::Reserved);
        assert!(
            metadata
                .introduced_in
                .unwrap_or("")
                .contains("mdbook-lint v0.1.0")
        );
    }

    #[test]
    fn test_md057_never_produces_violations() {
        let rule = MD057;
        let document = create_document("# Heading\n\nSome content with violations");
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md057_with_empty_document() {
        let rule = MD057;
        let document = create_document("");
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md057_with_complex_markdown() {
        let rule = MD057;
        let content = r#"
# Heading 1

Some **bold** and *italic* text.

## Heading 2

- List item 1
- List item 2

```rust
fn main() {
    println!("Hello, world!");
}
```

[Link](https://example.com)

| Table | Header |
|-------|--------|
| Cell  | Value  |
"#;
        let document = create_document(content);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }
}
