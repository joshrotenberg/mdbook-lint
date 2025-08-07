//! MD054 - Link and image style
//!
//! This rule checks for consistent link and image styles within a document.
//! Note: This is a simplified implementation that focuses on basic consistency.
//!
//! ## Correct
//!
//! ```markdown
//! \[Inline link\](https://example.com)
//! \[Another inline link\](https://example.com)
//! ```
//!
//! ## Incorrect
//!
//! ```markdown
//! \[Inline link\](https://example.com)
//! \[Reference link\]\[ref\]
//!
//! [ref]: https://example.com
//! ```

use crate::error::Result;
use crate::{
    Document, Violation,
    rule::{Rule, RuleCategory, RuleMetadata},
    violation::Severity,
};
use comrak::nodes::AstNode;

#[derive(Debug, Clone, PartialEq)]
enum ParsedLinkType {
    Inline,
    Reference,
    UrlInline,
}

/// MD054 - Link and image style
pub struct MD054 {
    autolink: bool,
    inline: bool,
    reference: bool,
    url_inline: bool,
}

impl Default for MD054 {
    fn default() -> Self {
        Self::new()
    }
}

impl MD054 {
    /// Create a new MD054 rule instance
    pub fn new() -> Self {
        Self {
            autolink: true,
            inline: true,
            reference: true,
            url_inline: true,
        }
    }

    /// Set whether to allow autolinks
    #[allow(dead_code)]
    pub fn autolink(mut self, allow: bool) -> Self {
        self.autolink = allow;
        self
    }

    /// Allow inline links
    #[allow(dead_code)]
    pub fn inline(mut self, allow: bool) -> Self {
        self.inline = allow;
        self
    }

    /// Allow reference links
    #[allow(dead_code)]
    pub fn reference(mut self, allow: bool) -> Self {
        self.reference = allow;
        self
    }

    /// Allow URL inline links
    #[allow(dead_code)]
    pub fn url_inline(mut self, allow: bool) -> Self {
        self.url_inline = allow;
        self
    }

    /// Check for style violations using manual parsing
    fn check_link_styles(&self, document: &Document) -> Vec<Violation> {
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
                    '<' if !in_backticks => {
                        // Check for autolinks: <https://...>
                        if let Some(autolink_end) = self.find_autolink_end(&line[i..]) {
                            if !self.autolink {
                                violations.push(self.create_violation(
                                    "Disallowed link style: autolink".to_string(),
                                    line_number,
                                    i + 1,
                                    Severity::Warning,
                                ));
                            }
                            // Skip past the autolink
                            for _ in 0..autolink_end - 1 {
                                chars.next();
                            }
                        }
                    }
                    '[' if !in_backticks => {
                        // Check for inline links: [text](url) or reference links: [text][ref]
                        if let Some((link_type, link_end)) = self.parse_link_at_position(&line[i..])
                        {
                            match link_type {
                                ParsedLinkType::Inline => {
                                    if !self.inline {
                                        violations.push(self.create_violation(
                                            "Disallowed link style: inline".to_string(),
                                            line_number,
                                            i + 1,
                                            Severity::Warning,
                                        ));
                                    }
                                }
                                ParsedLinkType::Reference => {
                                    if !self.reference {
                                        violations.push(self.create_violation(
                                            "Disallowed link style: reference".to_string(),
                                            line_number,
                                            i + 1,
                                            Severity::Warning,
                                        ));
                                    }
                                }
                                ParsedLinkType::UrlInline => {
                                    if !self.url_inline {
                                        violations.push(
                                            self.create_violation(
                                                "URL should use autolink style instead of inline"
                                                    .to_string(),
                                                line_number,
                                                i + 1,
                                                Severity::Warning,
                                            ),
                                        );
                                    }
                                }
                            }
                            // Skip past the link
                            for _ in 0..link_end - 1 {
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

    /// Find the end of an autolink starting with <
    fn find_autolink_end(&self, text: &str) -> Option<usize> {
        if !text.starts_with('<') {
            return None;
        }

        // Look for https:// or http://
        if text.len() < 8 || !text[1..].starts_with("http") {
            return None;
        }

        // Find the closing >
        if let Some(end_pos) = text.find('>') {
            let url = &text[1..end_pos];
            if url.starts_with("http://") || url.starts_with("https://") {
                return Some(end_pos + 1);
            }
        }

        None
    }

    /// Parse a link starting at position and return its type and end position
    fn parse_link_at_position(&self, text: &str) -> Option<(ParsedLinkType, usize)> {
        if !text.starts_with('[') {
            return None;
        }

        // Find the closing ]
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

        if remaining.starts_with('(') {
            // Inline link: [text](url)
            if let Some(closing_paren) = remaining.find(')') {
                let url = &remaining[1..closing_paren];
                let total_length = closing_bracket_pos + 1 + closing_paren + 1;

                // Check if this is a URL inline link (URL as both text and href)
                if (url.starts_with("http://") || url.starts_with("https://")) && link_text == url {
                    return Some((ParsedLinkType::UrlInline, total_length));
                }

                return Some((ParsedLinkType::Inline, total_length));
            }
        } else if remaining.starts_with('[') {
            // Reference link: [text][ref]
            if let Some(ref_end) = remaining.find(']') {
                let total_length = closing_bracket_pos + 1 + ref_end + 1;
                return Some((ParsedLinkType::Reference, total_length));
            }
        }

        None
    }
}

impl Rule for MD054 {
    fn id(&self) -> &'static str {
        "MD054"
    }

    fn name(&self) -> &'static str {
        "link-image-style"
    }

    fn description(&self) -> &'static str {
        "Link and image style"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Links)
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        // This rule works entirely with document content, not AST
        let violations = self.check_link_styles(document);
        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{
        assert_no_violations, assert_single_violation, assert_violation_count,
    };

    #[test]
    fn test_all_styles_allowed_by_default() {
        let content = r#"[Inline link](https://example.com)
[Reference link][ref]
<https://example.com>

[ref]: https://example.com
"#;

        assert_no_violations(MD054::new(), content);
    }

    #[test]
    fn test_disallow_autolinks() {
        let content = r#"<https://example.com>
[Inline link](https://example.com)
"#;

        let violation = assert_single_violation(MD054::new().autolink(false), content);
        assert_eq!(violation.line, 1);
        assert!(violation.message.contains("autolink"));
    }

    #[test]
    fn test_disallow_inline_links() {
        let content = r#"[Inline link](https://example.com)
[Reference link][ref]

[ref]: https://example.com
"#;

        let violation = assert_single_violation(MD054::new().inline(false), content);
        assert_eq!(violation.line, 1);
        assert!(violation.message.contains("inline"));
    }

    #[test]
    fn test_disallow_reference_links() {
        let content = r#"[Inline link](https://example.com)
[Reference link][ref]

[ref]: https://example.com
"#;

        let violation = assert_single_violation(MD054::new().reference(false), content);
        assert_eq!(violation.line, 2);
        assert!(violation.message.contains("reference"));
    }

    #[test]
    fn test_url_inline_detection() {
        let content = r#"[https://example.com](https://example.com)
"#;

        let violation = assert_single_violation(MD054::new().url_inline(false), content);
        assert_eq!(violation.line, 1);
        assert!(violation.message.contains("autolink style instead"));
    }

    #[test]
    fn test_mixed_content_allowed() {
        let content = r#"[Descriptive link](https://example.com)
![Inline image](image.png)
"#;

        assert_no_violations(MD054::new(), content);
    }

    #[test]
    fn test_multiple_violations() {
        let content = r#"<https://example.com>
[Inline link](https://different.com)
"#;

        let violations =
            assert_violation_count(MD054::new().autolink(false).inline(false), content, 2);
        assert!(violations[0].message.contains("autolink"));
        assert!(violations[1].message.contains("inline"));
    }

    #[test]
    fn test_reference_definitions_ignored() {
        let content = r#"[Link][ref]

[ref]: https://example.com
"#;

        // Reference links should still be detected when disabled, but reference definitions should not
        let violation = assert_single_violation(MD054::new().reference(false), content);
        assert_eq!(violation.line, 1);
        assert!(violation.message.contains("reference"));
    }
}
