//! MD042: No empty links
//!
//! This rule checks for links that have no text content.

use crate::error::Result;
use crate::rule::{AstRule, RuleCategory, RuleMetadata};
use crate::{
    Document,
    violation::{Severity, Violation},
};
use comrak::nodes::{AstNode, NodeValue};

/// Rule to check for empty links
pub struct MD042;

impl MD042 {
    /// Check if a link node is empty (has no text content)
    fn is_empty_link<'a>(&self, node: &'a AstNode<'a>) -> bool {
        // Get all text content from the link's children
        let text_content = Self::extract_text_content(node);
        text_content.trim().is_empty()
    }

    /// Extract all text content from a node and its children
    fn extract_text_content<'a>(node: &'a AstNode<'a>) -> String {
        let mut content = String::new();

        match &node.data.borrow().value {
            NodeValue::Text(text) => {
                content.push_str(text);
            }
            NodeValue::Code(code) => {
                content.push_str(&code.literal);
            }
            _ => {}
        }

        // Recursively extract text from children
        for child in node.children() {
            content.push_str(&Self::extract_text_content(child));
        }

        content
    }

    /// Get line and column position for a node
    fn get_position<'a>(&self, node: &'a AstNode<'a>) -> (usize, usize) {
        let data = node.data.borrow();
        let pos = data.sourcepos;
        (pos.start.line, pos.start.column)
    }

    /// Walk AST and find all link violations
    fn check_node<'a>(&self, node: &'a AstNode<'a>, violations: &mut Vec<Violation>) {
        match &node.data.borrow().value {
            NodeValue::Link(_) => {
                if self.is_empty_link(node) {
                    let (line, column) = self.get_position(node);
                    violations.push(self.create_violation(
                        "Found empty link".to_string(),
                        line,
                        column,
                        Severity::Warning,
                    ));
                }
            }
            NodeValue::Image(_) => {
                // Also check images for empty alt text
                if self.is_empty_link(node) {
                    let (line, column) = self.get_position(node);
                    violations.push(self.create_violation(
                        "Found image with empty alt text".to_string(),
                        line,
                        column,
                        Severity::Warning,
                    ));
                }
            }
            _ => {}
        }

        // Recursively check children
        for child in node.children() {
            self.check_node(child, violations);
        }
    }
}

impl AstRule for MD042 {
    fn id(&self) -> &'static str {
        "MD042"
    }

    fn name(&self) -> &'static str {
        "no-empty-links"
    }

    fn description(&self) -> &'static str {
        "No empty links"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Content).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_ast<'a>(&self, _document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        self.check_node(ast, &mut violations);
        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::Rule;
    use std::path::PathBuf;

    fn create_test_document(content: &str) -> Document {
        Document::new(content.to_string(), PathBuf::from("test.md")).unwrap()
    }

    #[test]
    fn test_md042_normal_links_valid() {
        let content = r#"Here is a [normal link](http://example.com).

Another [link with text](http://example.com) works fine.

Reference link [with text][ref] is also okay.

[ref]: http://example.com
"#;

        let document = create_test_document(content);
        let rule = MD042;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md042_empty_inline_link() {
        let content = r#"Here is an [](http://example.com) empty link.

This is normal text with a problem [](http://bad.com) link.
"#;

        let document = create_test_document(content);
        let rule = MD042;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].rule_id, "MD042");
        assert!(violations[0].message.contains("Found empty link"));
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 3);
    }

    #[test]
    fn test_md042_empty_reference_link() {
        let content = r#"Here is an [][ref] empty reference link.

[ref]: http://example.com
"#;

        let document = create_test_document(content);
        let rule = MD042;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 1);
    }

    #[test]
    fn test_md042_whitespace_only_link() {
        let content = r#"Here is a [   ](http://example.com) whitespace-only link.

Another [	](http://example.com) tab-only link.
"#;

        let document = create_test_document(content);
        let rule = MD042;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 3);
    }

    #[test]
    fn test_md042_link_with_code_valid() {
        let content = r#"Here is a [`code`](http://example.com) link with code.

Another [normal text](http://example.com) link.
"#;

        let document = create_test_document(content);
        let rule = MD042;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md042_link_with_emphasis_valid() {
        let content = r#"Here is a [*emphasized*](http://example.com) link.

Another [**strong**](http://example.com) link.

And [_underlined_](http://example.com) text.
"#;

        let document = create_test_document(content);
        let rule = MD042;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md042_empty_image_alt_text() {
        let content = r#"Here is an ![](image.png) image with no alt text.

This ![good alt text](image.png) is fine.

But this ![](bad.png) is not.
"#;

        let document = create_test_document(content);
        let rule = MD042;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("empty alt text"));
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 5);
    }

    #[test]
    fn test_md042_mixed_valid_and_invalid() {
        let content = r#"Good [link](http://example.com) here.

Bad [](http://example.com) link here.

Another good [link text](http://example.com).

Another bad [](http://bad.com) link.

![good alt](image.png) image.

![](bad-image.png) bad image.
"#;

        let document = create_test_document(content);
        let rule = MD042;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 3); // 2 empty links + 1 empty alt text

        // Check that we get both link violations and image violation
        let link_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.message.contains("Found empty link"))
            .collect();
        let image_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.message.contains("empty alt text"))
            .collect();

        assert_eq!(link_violations.len(), 2);
        assert_eq!(image_violations.len(), 1);
    }

    #[test]
    fn test_md042_autolinks_valid() {
        let content = r#"Autolinks like <http://example.com> are fine.

Email autolinks <user@example.com> are also okay.

Regular [text links](http://example.com) work too.
"#;

        let document = create_test_document(content);
        let rule = MD042;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md042_nested_formatting_valid() {
        let content = r#"Complex [**bold _and italic_**](http://example.com) link.

With [`code` and *emphasis*](http://example.com) mixed.
"#;

        let document = create_test_document(content);
        let rule = MD042;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md042_reference_style_links() {
        let content = r#"Good [reference link][good] here.

Bad [][bad] reference link.

[good]: http://example.com
[bad]: http://example.com
"#;

        let document = create_test_document(content);
        let rule = MD042;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3);
    }
}
