//! MDBOOK022: Title directive should appear near the top of the file
//!
//! The `{{#title}}` directive should appear within the first few lines
//! of a chapter file for consistency and to ensure it's processed early.

use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use regex::Regex;
use std::sync::LazyLock;

/// Default maximum line number for title directive
const DEFAULT_MAX_LINE: usize = 5;

/// Regex pattern for matching {{#title ...}} directives
static TITLE_DIRECTIVE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{#title\s+[^}]+\}\}").unwrap());

/// MDBOOK022: Ensures {{#title}} appears near the top of the file
///
/// The `{{#title}}` directive sets the page title in the browser tab.
/// For consistency and readability, it should appear near the top of
/// the file, typically before or just after the main heading.
#[derive(Clone)]
pub struct MDBOOK022 {
    /// Maximum line number where title directive is acceptable
    max_line: usize,
}

impl Default for MDBOOK022 {
    fn default() -> Self {
        Self {
            max_line: DEFAULT_MAX_LINE,
        }
    }
}

impl MDBOOK022 {
    /// Create with a custom maximum line threshold
    #[allow(dead_code)]
    pub fn with_max_line(max_line: usize) -> Self {
        Self { max_line }
    }
}

impl Rule for MDBOOK022 {
    fn id(&self) -> &'static str {
        "MDBOOK022"
    }

    fn name(&self) -> &'static str {
        "title-near-top"
    }

    fn description(&self) -> &'static str {
        "{{#title}} directive should appear near the top of the file"
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

        for (line_idx, line) in document.lines.iter().enumerate() {
            let line_num = line_idx + 1; // 1-based

            // Find {{#title}} directives in this line
            if let Some(mat) = TITLE_DIRECTIVE_REGEX.find(line) {
                let col = mat.start() + 1; // 1-based

                // Check if it's beyond the acceptable threshold
                if line_num > self.max_line {
                    violations.push(self.create_violation(
                        format!(
                            "{{{{#title}}}} directive at line {} should appear within the first {} lines of the file",
                            line_num, self.max_line
                        ),
                        line_num,
                        col,
                        Severity::Warning,
                    ));
                }

                // Only check the first occurrence (MDBOOK021 handles duplicates)
                break;
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
        let rule = MDBOOK022::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_title_on_first_line() {
        let content = "{{#title My Page Title}}\n\n# Chapter Title\n\nContent.";
        let doc = create_test_document(content);
        let rule = MDBOOK022::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_title_on_line_five() {
        let content = "# Chapter\n\nIntro paragraph.\n\n{{#title My Title}}\n\nMore content.";
        let doc = create_test_document(content);
        let rule = MDBOOK022::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(
            violations.len(),
            0,
            "Line 5 should be within default threshold"
        );
    }

    #[test]
    fn test_title_beyond_threshold() {
        let content =
            "# Chapter\n\nParagraph 1.\n\nParagraph 2.\n\n{{#title Late Title}}\n\nContent.";
        let doc = create_test_document(content);
        let rule = MDBOOK022::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("line 7"));
        assert!(violations[0].message.contains("first 5 lines"));
    }

    #[test]
    fn test_custom_threshold() {
        let content = "# Chapter\n\nParagraph.\n\n{{#title Title on Line 5}}";
        let doc = create_test_document(content);

        // With threshold of 3, line 5 should fail
        let rule = MDBOOK022::with_max_line(3);
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);

        // With threshold of 10, line 5 should pass
        let rule = MDBOOK022::with_max_line(10);
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_title_after_frontmatter() {
        // Common pattern: frontmatter-style content before title
        let content = "---\ndate: 2024-01-01\n---\n\n{{#title My Title}}\n\n# Chapter";
        let doc = create_test_document(content);
        let rule = MDBOOK022::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0, "Title on line 5 should pass");
    }

    #[test]
    fn test_title_way_down_in_file() {
        let content = "# Chapter\n\n\
            Paragraph 1.\n\n\
            Paragraph 2.\n\n\
            Paragraph 3.\n\n\
            Paragraph 4.\n\n\
            Paragraph 5.\n\n\
            {{#title Very Late Title}}\n\n\
            More content.";
        let doc = create_test_document(content);
        let rule = MDBOOK022::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 13);
    }

    #[test]
    fn test_only_first_title_checked() {
        // If there are multiple titles, only check the first one's position
        // (MDBOOK021 handles the duplicate issue)
        let content = "{{#title First}}\n\n# Chapter\n\n\n\n\n\n\n\n{{#title Second}}";
        let doc = create_test_document(content);
        let rule = MDBOOK022::default();
        let violations = rule.check(&doc).unwrap();
        // First title is on line 1, so no violation from this rule
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_title_with_empty_lines_before() {
        let content = "\n\n\n\n\n\n{{#title Late Start}}\n\n# Chapter";
        let doc = create_test_document(content);
        let rule = MDBOOK022::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 7);
    }
}
