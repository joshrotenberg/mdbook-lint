//! MDBOOK021: Single title directive per chapter
//!
//! Ensures that `{{#title}}` appears only once per chapter file.
//! Multiple title directives can cause unexpected behavior in mdBook.

use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use regex::Regex;
use std::sync::LazyLock;

/// Regex pattern for matching {{#title ...}} directives
static TITLE_DIRECTIVE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{#title\s+[^}]+\}\}").unwrap());

/// MDBOOK021: Ensures {{#title}} appears only once per chapter
///
/// The `{{#title}}` directive in mdBook sets the page title in the browser.
/// Having multiple title directives can cause unexpected behavior where
/// only one is applied (usually the last one), leading to confusion.
pub struct MDBOOK021;

impl Rule for MDBOOK021 {
    fn id(&self) -> &'static str {
        "MDBOOK021"
    }

    fn name(&self) -> &'static str {
        "single-title-directive"
    }

    fn description(&self) -> &'static str {
        "{{#title}} directive should appear only once per chapter"
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
        let mut title_occurrences: Vec<(usize, usize)> = Vec::new(); // (line, col)

        for (line_idx, line) in document.lines.iter().enumerate() {
            let line_num = line_idx + 1; // 1-based

            // Find all {{#title}} directives in this line
            for mat in TITLE_DIRECTIVE_REGEX.find_iter(line) {
                let col = mat.start() + 1; // 1-based
                title_occurrences.push((line_num, col));
            }
        }

        // If more than one title directive found, flag all but the first
        if title_occurrences.len() > 1 {
            let first = &title_occurrences[0];
            for (line, col) in title_occurrences.iter().skip(1) {
                violations.push(self.create_violation(
                    format!(
                        "Duplicate {{{{#title}}}} directive - first occurrence at line {}. \
                         Only one title directive should be used per chapter",
                        first.0
                    ),
                    *line,
                    *col,
                    Severity::Warning,
                ));
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
    fn test_no_title_directive() {
        let content = "# Chapter Title\n\nSome content here.";
        let doc = create_test_document(content);
        let rule = MDBOOK021;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_single_title_directive() {
        let content = "{{#title My Page Title}}\n\n# Chapter Title\n\nContent.";
        let doc = create_test_document(content);
        let rule = MDBOOK021;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_duplicate_title_directives() {
        let content = "{{#title First Title}}\n\n# Chapter\n\n{{#title Second Title}}\n\nContent.";
        let doc = create_test_document(content);
        let rule = MDBOOK021;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Duplicate"));
        assert!(violations[0].message.contains("line 1"));
        assert_eq!(violations[0].line, 5);
    }

    #[test]
    fn test_multiple_title_directives() {
        let content = "{{#title First}}\n{{#title Second}}\n{{#title Third}}";
        let doc = create_test_document(content);
        let rule = MDBOOK021;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 2);
        assert_eq!(violations[1].line, 3);
    }

    #[test]
    fn test_title_in_code_block_still_detected() {
        // Note: This rule doesn't skip code blocks because {{#title}}
        // in a code block would still be processed by mdBook preprocessor
        // (unless escaped). This is intentional.
        let content = "{{#title Real Title}}\n\n```\n{{#title In Code}}\n```";
        let doc = create_test_document(content);
        let rule = MDBOOK021;
        let violations = rule.check(&doc).unwrap();
        // Code blocks might contain example title directives, but they'd still
        // be processed unless properly escaped. We detect them for awareness.
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_title_with_special_characters() {
        let content = "{{#title My Book: A Guide to Rust}}\n\n# Chapter";
        let doc = create_test_document(content);
        let rule = MDBOOK021;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_multiple_titles_on_same_line() {
        let content = "{{#title First}} {{#title Second}}\n\n# Chapter";
        let doc = create_test_document(content);
        let rule = MDBOOK021;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_similar_but_not_title_directive() {
        let content = "{{#include file.rs}}\n\n# Chapter\n\nThe `{{#title}}` directive is useful.";
        let doc = create_test_document(content);
        let rule = MDBOOK021;
        let violations = rule.check(&doc).unwrap();
        // The inline code mention doesn't match the pattern (no actual title after it)
        assert_eq!(violations.len(), 0);
    }
}
