//! CONTENT010: Link text quality
//!
//! Flags generic or unhelpful link text like "click here", "this link", "here",
//! etc. Links should have descriptive text that makes sense out of context.

use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use regex::Regex;
use std::sync::LazyLock;

/// Regex to match markdown links [text](url)
static LINK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]*)\]\([^)]+\)").unwrap());

/// Generic link text patterns to flag
const GENERIC_LINK_TEXT: &[&str] = &[
    "click here",
    "here",
    "this link",
    "this page",
    "this article",
    "this",
    "link",
    "read more",
    "more",
    "learn more",
    "see more",
    "more info",
    "more information",
    "details",
    "info",
];

/// CONTENT010: Detects poor quality link text
///
/// Good link text should be descriptive and make sense out of context.
/// Screen readers often present links as a list, so "click here" or "here"
/// provides no useful information.
#[derive(Default, Clone)]
pub struct CONTENT010;

impl Rule for CONTENT010 {
    fn id(&self) -> &'static str {
        "CONTENT010"
    }

    fn name(&self) -> &'static str {
        "link-text-quality"
    }

    fn description(&self) -> &'static str {
        "Link text should be descriptive, not generic like 'click here' or 'here'"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Content).introduced_in("mdbook-lint v0.14.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> mdbook_lint_core::error::Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let mut in_code_block = false;

        for (line_idx, line) in document.lines.iter().enumerate() {
            let trimmed = line.trim();

            // Track code blocks
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }

            if in_code_block {
                continue;
            }

            // Find all links in the line
            for caps in LINK_REGEX.captures_iter(line) {
                if let Some(text_match) = caps.get(1) {
                    let link_text = text_match.as_str().trim();
                    let link_text_lower = link_text.to_lowercase();

                    // Check against generic patterns
                    for &generic in GENERIC_LINK_TEXT {
                        if link_text_lower == generic {
                            let line_num = line_idx + 1;
                            let col = text_match.start() + 1;

                            violations.push(self.create_violation(
                                format!(
                                    "Generic link text '{}' is not descriptive. \
                                     Use meaningful text that describes the link destination",
                                    link_text
                                ),
                                line_num,
                                col,
                                Severity::Warning,
                            ));
                            break;
                        }
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
    use std::path::PathBuf;

    fn create_test_document(content: &str) -> Document {
        Document::new(content.to_string(), PathBuf::from("test.md")).unwrap()
    }

    #[test]
    fn test_good_link_text() {
        let content = "Check out the [installation guide](./install.md) for details.

See the [API documentation](https://docs.example.com) for more info.";
        let doc = create_test_document(content);
        let rule = CONTENT010;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_click_here() {
        let content = "[Click here](https://example.com) to learn more.";
        let doc = create_test_document(content);
        let rule = CONTENT010;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Click here"));
    }

    #[test]
    fn test_here_alone() {
        let content = "For more information, see [here](./docs.md).";
        let doc = create_test_document(content);
        let rule = CONTENT010;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_this_link() {
        let content = "Follow [this link](https://example.com) for details.";
        let doc = create_test_document(content);
        let rule = CONTENT010;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_read_more() {
        let content = "[Read more](./article.md)";
        let doc = create_test_document(content);
        let rule = CONTENT010;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_learn_more() {
        let content = "[Learn more](https://docs.example.com)";
        let doc = create_test_document(content);
        let rule = CONTENT010;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_multiple_violations() {
        let content = "See [here](./a.md) and [click here](./b.md) for more.";
        let doc = create_test_document(content);
        let rule = CONTENT010;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_case_insensitive() {
        let content = "[HERE](./doc.md) and [CLICK HERE](./other.md)";
        let doc = create_test_document(content);
        let rule = CONTENT010;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_links_in_code_blocks_ignored() {
        let content = "```markdown
[click here](https://example.com)
```";
        let doc = create_test_document(content);
        let rule = CONTENT010;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_partial_match_not_flagged() {
        let content = "Read [more about configuration](./config.md) here.";
        let doc = create_test_document(content);
        let rule = CONTENT010;
        let violations = rule.check(&doc).unwrap();
        // "more about configuration" contains "more" but is not exactly "more"
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_this_alone() {
        let content = "See [this](./example.md) for an example.";
        let doc = create_test_document(content);
        let rule = CONTENT010;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_info() {
        let content = "[Info](./help.md)";
        let doc = create_test_document(content);
        let rule = CONTENT010;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_details() {
        let content = "For [details](./spec.md), see the specification.";
        let doc = create_test_document(content);
        let rule = CONTENT010;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }
}
