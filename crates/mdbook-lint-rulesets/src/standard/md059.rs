//! MD059 - Link text should be descriptive
//!
//! This rule is triggered when a link has generic text that doesn't describe
//! the purpose of the link.
//!
//! ## Correct
//!
//! ```markdown
//! \[Download the budget document\](document.pdf)
//! \[CommonMark Specification\](https://spec.commonmark.org/)
//! ```
//!
//! ## Incorrect
//!
//! ```markdown
//! \[click here\](document.pdf)
//! \[here\](https://example.com)
//! \[link\](https://example.com)
//! \[more\](https://example.com)
//! ```

use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::error::Result;
use mdbook_lint_core::{
    Document, Violation,
    rule::{Rule, RuleCategory, RuleMetadata},
    violation::Severity,
};

/// MD059 - Link text should be descriptive
pub struct MD059 {
    prohibited_texts: Vec<String>,
}

impl Default for MD059 {
    fn default() -> Self {
        Self::new()
    }
}

impl MD059 {
    /// Create a new MD059 rule instance
    pub fn new() -> Self {
        Self {
            prohibited_texts: vec![
                "click here".to_string(),
                "here".to_string(),
                "link".to_string(),
                "more".to_string(),
            ],
        }
    }

    /// Set the list of prohibited link texts
    #[allow(dead_code)]
    pub fn prohibited_texts(mut self, texts: Vec<String>) -> Self {
        self.prohibited_texts = texts;
        self
    }

    /// Extract text content from a link node
    fn extract_link_text<'a>(node: &'a AstNode<'a>) -> String {
        let mut text = String::new();
        for child in node.children() {
            match &child.data.borrow().value {
                NodeValue::Text(t) => text.push_str(t),
                NodeValue::Code(code) => text.push_str(&code.literal),
                NodeValue::Emph | NodeValue::Strong => {
                    text.push_str(&Self::extract_link_text(child));
                }
                _ => {}
            }
        }
        text.trim().to_string()
    }

    /// Check if link text is prohibited
    fn is_prohibited_text(&self, text: &str) -> bool {
        let normalized_text = text.to_lowercase();
        self.prohibited_texts
            .iter()
            .any(|prohibited| prohibited.to_lowercase() == normalized_text)
    }

    /// Check for non-descriptive link text
    fn check_link_text<'a>(&self, ast: &'a AstNode<'a>) -> Vec<Violation> {
        let mut violations = Vec::new();
        self.traverse_for_links(ast, &mut violations);
        violations
    }

    /// Traverse AST to find links
    fn traverse_for_links<'a>(&self, node: &'a AstNode<'a>, violations: &mut Vec<Violation>) {
        if let NodeValue::Link(link) = &node.data.borrow().value {
            // Skip autolinks and reference definitions
            if !link.url.is_empty() {
                let link_text = Self::extract_link_text(node);

                // Skip empty link text
                if !link_text.is_empty() && self.is_prohibited_text(&link_text) {
                    let pos = node.data.borrow().sourcepos;
                    let line = pos.start.line;
                    let column = pos.start.column;
                    violations.push(self.create_violation(
                        format!(
                            "Link text '{link_text}' is not descriptive. Use descriptive text that explains the purpose of the link"
                        ),
                        line,
                        column,
                        Severity::Warning,
                    ));
                }
            }
        }

        for child in node.children() {
            self.traverse_for_links(child, violations);
        }
    }

    /// Fallback method using manual parsing when no AST is available
    fn check_link_text_fallback(&self, document: &Document) -> Vec<Violation> {
        let mut violations = Vec::new();

        for (line_num, line) in document.content.lines().enumerate() {
            let line_number = line_num + 1;
            let mut chars = line.char_indices().peekable();
            let mut in_backticks = false;

            while let Some((i, ch)) = chars.next() {
                match ch {
                    '`' => {
                        in_backticks = !in_backticks;
                    }
                    '[' if !in_backticks => {
                        // Try to parse any kind of link: [text](url) or [text][ref]
                        if let Some((link_text, text_start, text_end)) =
                            self.parse_any_link_at(&line[i..])
                        {
                            let cleaned_text = Self::strip_emphasis_markers(link_text);
                            let trimmed_text = cleaned_text.trim();

                            if !trimmed_text.is_empty() && self.is_prohibited_text(trimmed_text) {
                                violations.push(self.create_violation(
                                    format!(
                                        "Link text '{trimmed_text}' is not descriptive. Use descriptive text that explains the purpose of the link"
                                    ),
                                    line_number,
                                    i + text_start + 2, // +1 for 1-based indexing, +1 for opening bracket
                                    Severity::Warning,
                                ));
                            }

                            // Skip past the entire link
                            for _ in 0..text_end - 1 {
                                chars.next();
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        violations
    }

    /// Parse any link (inline or reference) starting at the given position
    /// Returns (link_text, text_start_offset, total_length) if found
    fn parse_any_link_at<'a>(&self, text: &'a str) -> Option<(&'a str, usize, usize)> {
        if !text.starts_with('[') {
            return None;
        }

        // Find the closing bracket
        let mut bracket_count = 0;
        let mut closing_bracket_pos = None;

        for (i, ch) in text.char_indices() {
            match ch {
                '[' => bracket_count += 1,
                ']' => {
                    bracket_count -= 1;
                    if bracket_count == 0 {
                        closing_bracket_pos = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }

        let closing_bracket_pos = closing_bracket_pos?;
        let link_text = &text[1..closing_bracket_pos];
        let remaining = &text[closing_bracket_pos + 1..];

        // Check if this is followed by (url) - inline link
        if remaining.starts_with('(') {
            if let Some(closing_paren) = remaining.find(')') {
                let total_length = closing_bracket_pos + 1 + closing_paren + 1;
                return Some((link_text, 0, total_length));
            }
        }
        // Check if this is followed by [ref] - reference link
        else if remaining.starts_with('[')
            && let Some(ref_end) = remaining.find(']')
        {
            let total_length = closing_bracket_pos + 1 + ref_end + 1;
            return Some((link_text, 0, total_length));
        }

        None
    }

    /// Strip emphasis markers from link text (similar to AST extract_link_text)
    fn strip_emphasis_markers(text: &str) -> String {
        let mut result = String::new();
        let mut chars = text.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '*' => {
                    // Check for ** (strong) or * (emphasis)
                    if chars.peek() == Some(&'*') {
                        chars.next(); // consume second *
                        // Find closing **
                        let mut temp = String::new();
                        let mut found_closing = false;
                        while let Some(inner_ch) = chars.next() {
                            if inner_ch == '*' && chars.peek() == Some(&'*') {
                                chars.next(); // consume second *
                                found_closing = true;
                                break;
                            }
                            temp.push(inner_ch);
                        }
                        if found_closing {
                            result.push_str(&Self::strip_emphasis_markers(&temp));
                        } else {
                            result.push_str("**");
                            result.push_str(&temp);
                        }
                    } else {
                        // Find closing *
                        let mut temp = String::new();
                        let mut found_closing = false;
                        for inner_ch in chars.by_ref() {
                            if inner_ch == '*' {
                                found_closing = true;
                                break;
                            }
                            temp.push(inner_ch);
                        }
                        if found_closing {
                            result.push_str(&Self::strip_emphasis_markers(&temp));
                        } else {
                            result.push('*');
                            result.push_str(&temp);
                        }
                    }
                }
                '`' => {
                    // Find closing `
                    let mut temp = String::new();
                    let mut found_closing = false;
                    for inner_ch in chars.by_ref() {
                        if inner_ch == '`' {
                            found_closing = true;
                            break;
                        }
                        temp.push(inner_ch);
                    }
                    if found_closing {
                        result.push_str(&temp); // Code content as-is
                    } else {
                        result.push('`');
                        result.push_str(&temp);
                    }
                }
                _ => result.push(ch),
            }
        }

        result
    }
}

impl Rule for MD059 {
    fn id(&self) -> &'static str {
        "MD059"
    }

    fn name(&self) -> &'static str {
        "descriptive-link-text"
    }

    fn description(&self) -> &'static str {
        "Link text should be descriptive"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Accessibility)
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        ast: Option<&'a AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        if let Some(ast) = ast {
            let violations = self.check_link_text(ast);
            Ok(violations)
        } else {
            // Simplified regex-based fallback when no AST is available
            Ok(self.check_link_text_fallback(document))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::test_helpers::*;

    #[test]
    fn test_descriptive_link_text() {
        let content = r#"[Download the budget document](document.pdf)
[CommonMark Specification](https://spec.commonmark.org/)
[View the installation guide](install.md)
"#;

        assert_no_violations(MD059::new(), content);
    }

    #[test]
    fn test_prohibited_link_text() {
        let content = r#"[click here](document.pdf)
[here](https://example.com)
[link](https://example.com)
[more](info.html)
"#;

        let violations = assert_violation_count(MD059::new(), content, 4);

        assert_eq!(violations[0].line, 1);
        assert!(violations[0].message.contains("click here"));

        assert_eq!(violations[1].line, 2);
        assert!(violations[1].message.contains("here"));

        assert_eq!(violations[2].line, 3);
        assert!(violations[2].message.contains("link"));

        assert_eq!(violations[3].line, 4);
        assert!(violations[3].message.contains("more"));
    }

    #[test]
    fn test_case_insensitive_matching() {
        let content = r#"[CLICK HERE](document.pdf)
[Here](https://example.com)
[Link](https://example.com)
[MORE](info.html)
"#;

        let violations = assert_violation_count(MD059::new(), content, 4);
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 2);
        assert_eq!(violations[2].line, 3);
        assert_eq!(violations[3].line, 4);
    }

    #[test]
    fn test_custom_prohibited_texts() {
        let content = r#"[read more](document.pdf)
[see details](https://example.com)
"#;

        let rule =
            MD059::new().prohibited_texts(vec!["read more".to_string(), "see details".to_string()]);
        let violations = assert_violation_count(rule, content, 2);
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 2);
    }

    #[test]
    fn test_autolinks_ignored() {
        let content = r#"<https://example.com>
<mailto:user@example.com>
"#;

        assert_no_violations(MD059::new(), content);
    }

    #[test]
    fn test_reference_links() {
        let content = r#"[click here][ref]
[descriptive text][ref2]

[ref]: https://example.com
[ref2]: https://example.com
"#;

        let violation = assert_single_violation(MD059::new(), content);
        assert_eq!(violation.line, 1);
        assert!(violation.message.contains("click here"));
    }

    #[test]
    fn test_links_with_emphasis() {
        let content = r#"[**click here**](document.pdf)
[*here*](https://example.com)
[`code link`](https://example.com)
"#;

        let violations = assert_violation_count(MD059::new(), content, 2);

        assert_eq!(violations[0].line, 1);
        assert!(violations[0].message.contains("click here"));

        assert_eq!(violations[1].line, 2);
        assert!(violations[1].message.contains("here"));
    }

    #[test]
    fn test_empty_link_text_ignored() {
        let content = r#"[](https://example.com)
"#;

        assert_no_violations(MD059::new(), content);
    }

    #[test]
    fn test_mixed_content() {
        let content = r#"[Download guide](guide.pdf) contains useful information.
You can [click here](more.html) for additional details.
See the [API documentation](api.md) for technical details.
"#;

        let violation = assert_single_violation(MD059::new(), content);
        assert_eq!(violation.line, 2);
        assert!(violation.message.contains("click here"));
    }
}
