//! CONTENT002: Placeholder text detection
//!
//! Detects common placeholder text patterns like "Lorem ipsum", "TBD",
//! "coming soon", etc. that shouldn't appear in production documentation.

use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use regex::Regex;
use std::sync::LazyLock;

/// Regex patterns for placeholder text (case-insensitive)
static PLACEHOLDER_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        // Lorem ipsum variants
        (
            Regex::new(r"(?i)\blorem\s+ipsum\b").unwrap(),
            "Lorem ipsum placeholder text",
        ),
        (
            Regex::new(r"(?i)\bipsum\s+dolor\b").unwrap(),
            "Lorem ipsum placeholder text",
        ),
        // Status placeholders
        (
            Regex::new(r"(?i)\bTBD\b").unwrap(),
            "TBD (To Be Determined) placeholder",
        ),
        (
            Regex::new(r"(?i)\bTBA\b").unwrap(),
            "TBA (To Be Announced) placeholder",
        ),
        (
            Regex::new(r"(?i)\bTBC\b").unwrap(),
            "TBC (To Be Confirmed) placeholder",
        ),
        // Coming soon variants
        (
            Regex::new(r"(?i)\bcoming\s+soon\b").unwrap(),
            "Coming soon placeholder",
        ),
        (
            Regex::new(r"(?i)\bunder\s+construction\b").unwrap(),
            "Under construction placeholder",
        ),
        (
            Regex::new(r"(?i)\bwork\s+in\s+progress\b").unwrap(),
            "Work in progress placeholder",
        ),
        // Insert/add placeholders
        (
            Regex::new(r"(?i)\binsert\s+\w+\s+here\b").unwrap(),
            "Insert here placeholder",
        ),
        (
            Regex::new(r"(?i)\badd\s+\w+\s+here\b").unwrap(),
            "Add here placeholder",
        ),
        (
            Regex::new(r"(?i)\bput\s+\w+\s+here\b").unwrap(),
            "Put here placeholder",
        ),
        // N/A and placeholder
        (
            Regex::new(r"(?i)\bN/A\b").unwrap(),
            "N/A placeholder - consider removing or providing actual content",
        ),
        (
            Regex::new(r"(?i)\bplaceholder\b").unwrap(),
            "Placeholder text detected",
        ),
        // Draft/pending
        (Regex::new(r"(?i)\[draft\]").unwrap(), "Draft marker found"),
        (
            Regex::new(r"(?i)\[pending\]").unwrap(),
            "Pending marker found",
        ),
        // Example placeholders
        (
            Regex::new(r"(?i)\bexample\.com\b").unwrap(),
            "Example.com placeholder URL",
        ),
        (
            Regex::new(r"(?i)\bfoo\s*bar\s*baz\b").unwrap(),
            "Foo bar baz placeholder",
        ),
        // Empty section markers
        (
            Regex::new(r"(?i)\bthis\s+section\s+(is\s+)?(empty|blank|incomplete)\b").unwrap(),
            "Empty section marker",
        ),
        (
            Regex::new(r"(?i)\bcontent\s+goes\s+here\b").unwrap(),
            "Content placeholder",
        ),
        // Your/my name placeholders
        (
            Regex::new(r"(?i)\byour\s+name\s+here\b").unwrap(),
            "Name placeholder",
        ),
        (
            Regex::new(r"(?i)\b<your[_\s]name>\b").unwrap(),
            "Name placeholder",
        ),
        // XXX as content (not code comment - that's CONTENT001)
        (
            Regex::new(r"(?i)^XXX+$").unwrap(),
            "XXX placeholder content",
        ),
        // Ellipsis as placeholder content (standalone)
        (
            Regex::new(r"^\s*\.\.\.\s*$").unwrap(),
            "Ellipsis placeholder - provide actual content",
        ),
    ]
});

/// CONTENT002: Detects placeholder text
///
/// This rule flags common placeholder patterns that indicate
/// incomplete documentation. These should be replaced with
/// actual content before publishing.
pub struct CONTENT002 {
    /// Whether to check inside code blocks
    check_code_blocks: bool,
    /// Whether to allow example.com in examples
    allow_example_urls: bool,
}

impl Default for CONTENT002 {
    fn default() -> Self {
        Self {
            check_code_blocks: false,
            allow_example_urls: true, // example.com is acceptable in code examples
        }
    }
}

impl CONTENT002 {
    /// Set whether to check inside code blocks
    #[allow(dead_code)]
    pub fn check_code_blocks(mut self, check: bool) -> Self {
        self.check_code_blocks = check;
        self
    }

    /// Set whether to allow example.com URLs
    #[allow(dead_code)]
    pub fn allow_example_urls(mut self, allow: bool) -> Self {
        self.allow_example_urls = allow;
        self
    }

    /// Check if a position is inside a code block
    fn is_in_code_block(&self, lines: &[String], line_idx: usize) -> bool {
        let mut in_fenced_block = false;

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Check for fenced code block markers
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_fenced_block = !in_fenced_block;
            }

            if idx == line_idx {
                return in_fenced_block;
            }
        }

        false
    }

    /// Check if a position is inside inline code
    fn is_in_inline_code(&self, line: &str, col: usize) -> bool {
        let before = &line[..col.min(line.len())];

        // Count backticks before the position
        let backtick_count = before.chars().filter(|&c| c == '`').count();

        // Odd number of backticks means we're inside inline code
        backtick_count % 2 == 1
    }
}

impl Rule for CONTENT002 {
    fn id(&self) -> &'static str {
        "CONTENT002"
    }

    fn name(&self) -> &'static str {
        "no-placeholder-text"
    }

    fn description(&self) -> &'static str {
        "Placeholder text should be replaced with actual content"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Content).introduced_in("mdbook-lint v0.12.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> mdbook_lint_core::error::Result<Vec<Violation>> {
        let mut violations = Vec::new();

        for (line_idx, line) in document.lines.iter().enumerate() {
            let line_num = line_idx + 1; // 1-based

            // Skip code blocks unless configured to check them
            if !self.check_code_blocks && self.is_in_code_block(&document.lines, line_idx) {
                continue;
            }

            // Check each pattern
            for (pattern, description) in PLACEHOLDER_PATTERNS.iter() {
                // Skip example.com check if allowed
                if self.allow_example_urls && *description == "Example.com placeholder URL" {
                    // Only flag if not in a code context
                    if self.is_in_code_block(&document.lines, line_idx) {
                        continue;
                    }
                }

                for mat in pattern.find_iter(line) {
                    let col = mat.start() + 1; // 1-based

                    // Skip if inside inline code (unless checking code blocks)
                    if !self.check_code_blocks && self.is_in_inline_code(line, mat.start()) {
                        continue;
                    }

                    violations.push(self.create_violation(
                        format!("{} - replace with actual content", description),
                        line_num,
                        col,
                        Severity::Warning,
                    ));
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
    fn test_no_placeholders() {
        let content = "# Title\n\nThis is real, complete documentation.";
        let doc = create_test_document(content);
        let rule = CONTENT002::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_lorem_ipsum_detected() {
        let content = "# Title\n\nLorem ipsum dolor sit amet.";
        let doc = create_test_document(content);
        let rule = CONTENT002::default();
        let violations = rule.check(&doc).unwrap();
        assert!(!violations.is_empty());
        assert!(violations[0].message.contains("Lorem ipsum"));
    }

    #[test]
    fn test_tbd_detected() {
        let content = "# Title\n\nThis feature is TBD.";
        let doc = create_test_document(content);
        let rule = CONTENT002::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("TBD"));
    }

    #[test]
    fn test_coming_soon_detected() {
        let content = "# Title\n\nThis section is coming soon.";
        let doc = create_test_document(content);
        let rule = CONTENT002::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Coming soon"));
    }

    #[test]
    fn test_under_construction_detected() {
        let content = "# Title\n\nThis page is under construction.";
        let doc = create_test_document(content);
        let rule = CONTENT002::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Under construction"));
    }

    #[test]
    fn test_insert_here_detected() {
        let content = "# Title\n\nInsert content here.";
        let doc = create_test_document(content);
        let rule = CONTENT002::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Insert here"));
    }

    #[test]
    fn test_placeholder_word_detected() {
        let content = "# Title\n\nThis is placeholder text.";
        let doc = create_test_document(content);
        let rule = CONTENT002::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Placeholder"));
    }

    #[test]
    fn test_draft_marker_detected() {
        let content = "# Title [draft]\n\nContent here.";
        let doc = create_test_document(content);
        let rule = CONTENT002::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Draft"));
    }

    #[test]
    fn test_case_insensitive() {
        let content = "# Title\n\nCOMING SOON\ncoming Soon\nComing soon";
        let doc = create_test_document(content);
        let rule = CONTENT002::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 3);
    }

    #[test]
    fn test_skip_code_blocks() {
        let content = "# Title\n\n```\nLorem ipsum in code\n```\n\nLorem ipsum outside";
        let doc = create_test_document(content);
        let rule = CONTENT002::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 7);
    }

    #[test]
    fn test_skip_inline_code() {
        let content = "# Title\n\nUse `TBD` as status.\n\nActual TBD here";
        let doc = create_test_document(content);
        let rule = CONTENT002::default();
        let violations = rule.check(&doc).unwrap();
        // Only line 5 should be flagged - line 3 has TBD in inline code
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 5);
        assert!(violations[0].message.contains("TBD"));
    }

    #[test]
    fn test_ellipsis_placeholder() {
        let content = "# Title\n\n...\n\nActual content.";
        let doc = create_test_document(content);
        let rule = CONTENT002::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Ellipsis"));
    }

    #[test]
    fn test_na_detected() {
        let content = "# Title\n\nDescription: N/A";
        let doc = create_test_document(content);
        let rule = CONTENT002::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("N/A"));
    }

    #[test]
    fn test_content_goes_here() {
        let content = "# Title\n\nContent goes here.";
        let doc = create_test_document(content);
        let rule = CONTENT002::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_example_com_allowed_by_default() {
        let content = "# Title\n\nVisit https://example.com for more.";
        let doc = create_test_document(content);
        let rule = CONTENT002::default();
        let violations = rule.check(&doc).unwrap();
        // Should still detect since it's not in code block
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_your_name_here() {
        let content = "# Title\n\nAuthor: Your name here";
        let doc = create_test_document(content);
        let rule = CONTENT002::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Name placeholder"));
    }

    #[test]
    fn test_work_in_progress() {
        let content = "# Title\n\nThis section is a work in progress.";
        let doc = create_test_document(content);
        let rule = CONTENT002::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Work in progress"));
    }

    #[test]
    fn test_multiple_placeholders() {
        let content = "# Title\n\nTBD\n\nLorem ipsum dolor\n\nComing soon";
        let doc = create_test_document(content);
        let rule = CONTENT002::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.len() >= 3);
    }

    #[test]
    fn test_empty_section_marker() {
        let content = "# Title\n\nThis section is empty.";
        let doc = create_test_document(content);
        let rule = CONTENT002::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Empty section"));
    }
}
