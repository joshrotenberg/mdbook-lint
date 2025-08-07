//! MD045: Images should have alternate text
//!
//! This rule checks that all images have non-empty alternate text for accessibility.

use crate::error::Result;
use crate::rule::{AstRule, RuleCategory, RuleMetadata};
use crate::{
    Document,
    violation::{Severity, Violation},
};
use comrak::nodes::{AstNode, NodeValue};

/// Rule to check that images have alternate text
pub struct MD045;

impl MD045 {
    /// Check if an image node has empty or missing alt text
    fn is_empty_alt_text<'a>(&self, node: &'a AstNode<'a>) -> bool {
        // Get all text content from the image's children
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

    /// Walk AST and find all image violations
    fn check_node<'a>(&self, node: &'a AstNode<'a>, violations: &mut Vec<Violation>) {
        if let NodeValue::Image(_) = &node.data.borrow().value
            && self.is_empty_alt_text(node)
        {
            let (line, column) = self.get_position(node);
            violations.push(self.create_violation(
                "Images should have alternate text".to_string(),
                line,
                column,
                Severity::Warning,
            ));
        }

        // Recursively check children
        for child in node.children() {
            self.check_node(child, violations);
        }
    }
}

impl AstRule for MD045 {
    fn id(&self) -> &'static str {
        "MD045"
    }

    fn name(&self) -> &'static str {
        "no-alt-text"
    }

    fn description(&self) -> &'static str {
        "Images should have alternate text"
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
    fn test_md045_images_with_alt_text_valid() {
        let content = r#"Here is an image with alt text: ![Good alt text](image.png).

Another ![descriptive text](image2.jpg) here.

And a reference image: ![alt text][ref]

[ref]: image3.gif
"#;

        let document = create_test_document(content);
        let rule = MD045;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md045_images_without_alt_text_violation() {
        let content = r#"Here is an image without alt text: ![](image.png).

Another ![](image2.jpg) here.
"#;

        let document = create_test_document(content);
        let rule = MD045;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].rule_id, "MD045");
        assert!(
            violations[0]
                .message
                .contains("Images should have alternate text")
        );
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 3);
    }

    #[test]
    fn test_md045_images_with_whitespace_only_alt_text() {
        let content = r#"Image with spaces: ![   ](image.png).

Image with tabs: ![		](image2.jpg).

Image with mixed whitespace: ![  	  ](image3.gif).
"#;

        let document = create_test_document(content);
        let rule = MD045;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 3);
        assert_eq!(violations[2].line, 5);
    }

    #[test]
    fn test_md045_images_with_code_alt_text_valid() {
        let content = r#"Image with code alt text: ![`filename.png`](image.png).

Another with inline code: ![The `main.rs` file](code.png).
"#;

        let document = create_test_document(content);
        let rule = MD045;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md045_images_with_emphasis_alt_text_valid() {
        let content = r#"Image with emphasis: ![*Important* diagram](diagram.png).

Image with strong: ![**Critical** figure](figure.png).

Image with mixed: ![*Very* **important** chart](chart.png).
"#;

        let document = create_test_document(content);
        let rule = MD045;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md045_reference_images() {
        let content = r#"Good reference image: ![Good alt text][good].

Bad reference image: ![][bad].

Another bad one: ![  ][also-bad].

[good]: image1.png
[bad]: image2.png
[also-bad]: image3.png
"#;

        let document = create_test_document(content);
        let rule = MD045;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 5);
    }

    #[test]
    fn test_md045_links_ignored() {
        let content = r#"This is a [link without text]() which should not be flagged.

This is a [](http://example.com) empty link, also not flagged by this rule.

But this ![](image.png) empty image should be flagged.
"#;

        let document = create_test_document(content);
        let rule = MD045;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 5);
    }

    #[test]
    fn test_md045_mixed_images_and_links() {
        let content = r#"Good image: ![Alt text](image.png) and good [link](http://example.com).

Bad image: ![](bad-image.png) and empty [](http://example.com) link.

Another good image: ![Description](good.png) here.
"#;

        let document = create_test_document(content);
        let rule = MD045;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3);
    }

    #[test]
    fn test_md045_nested_formatting_in_alt_text() {
        let content = r#"Complex alt text: ![Figure showing **bold** and *italic* with `code`](complex.png).

Simple alt text: ![Just text](simple.png).

Empty alt text: ![](empty.png).
"#;

        let document = create_test_document(content);
        let rule = MD045;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 5);
    }

    #[test]
    fn test_md045_inline_images() {
        let content = r#"Text with inline ![good alt](inline.png) image.

Text with inline ![](bad-inline.png) empty image.

More text with ![another good](good-inline.png) alt text.
"#;

        let document = create_test_document(content);
        let rule = MD045;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3);
    }

    #[test]
    fn test_md045_multiple_images_per_line() {
        let content = r#"Multiple images: ![Good](img1.png) and ![](img2.png) and ![Also good](img3.png).

All good: ![Alt 1](img4.png) and ![Alt 2](img5.png).

All bad: ![](img6.png) and ![  ](img7.png).
"#;

        let document = create_test_document(content);
        let rule = MD045;
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].line, 1); // ![](img2.png)
        assert_eq!(violations[1].line, 5); // ![](img6.png)
        assert_eq!(violations[2].line, 5); // ![  ](img7.png)
    }
}
