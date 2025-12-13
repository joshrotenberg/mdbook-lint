//! CONTENT001: TODO/FIXME/XXX comment detection
//!
//! Detects TODO, FIXME, XXX, and other common work-in-progress markers
//! that shouldn't appear in production documentation.

use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use regex::Regex;
use std::sync::LazyLock;

/// Default markers to detect (these are matched as whole words)
const DEFAULT_MARKERS: &[&str] = &["TODO", "FIXME", "XXX", "HACK", "WIP"];

/// Markers that require comment-style context to avoid false positives in prose
/// e.g., "BUG:" or "BUG(" but not "this bug" or "bug fix"
/// Note: These are handled separately via CONTEXTUAL_MARKER_REGEX
#[allow(dead_code)]
const CONTEXTUAL_MARKERS: &[&str] = &["BUG"];

/// Regex pattern for matching standard markers (case-insensitive, word boundary)
static MARKER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // Match markers as whole words
    Regex::new(r"(?i)\b(TODO|FIXME|XXX|HACK|WIP)\b").unwrap()
});

/// Regex pattern for markers that need comment-style context
/// Matches BUG only when followed by :, (, or at start of line/after comment markers
static CONTEXTUAL_MARKER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // Match BUG only in comment-like contexts:
    // - BUG: or BUG( (followed by colon or paren)
    // - // BUG or /* BUG (after code comment markers)
    // - Start of line with optional whitespace
    Regex::new(r"(?i)(?:^|\s|//|/\*|#)\s*(BUG)\s*[:(\[]|(?i)\bBUG\s*[:(\[]").unwrap()
});

/// CONTENT001: Detects TODO/FIXME/XXX comments
///
/// This rule flags common work-in-progress markers that indicate
/// incomplete documentation. These should be resolved before publishing.
pub struct CONTENT001 {
    /// Custom markers to detect (in addition to or instead of defaults)
    markers: Vec<String>,
    /// Whether to include default markers
    include_defaults: bool,
    /// Whether to check inside code blocks
    check_code_blocks: bool,
}

impl Default for CONTENT001 {
    fn default() -> Self {
        Self {
            markers: Vec::new(),
            include_defaults: true,
            check_code_blocks: false,
        }
    }
}

impl CONTENT001 {
    /// Create with custom markers
    #[allow(dead_code)]
    pub fn with_markers(markers: Vec<String>) -> Self {
        Self {
            markers,
            include_defaults: false,
            check_code_blocks: false,
        }
    }

    /// Set whether to check inside code blocks
    #[allow(dead_code)]
    pub fn check_code_blocks(mut self, check: bool) -> Self {
        self.check_code_blocks = check;
        self
    }

    /// Get all markers to check
    fn get_markers(&self) -> Vec<&str> {
        let mut markers: Vec<&str> = Vec::new();

        if self.include_defaults {
            markers.extend(DEFAULT_MARKERS.iter().copied());
        }

        for marker in &self.markers {
            markers.push(marker.as_str());
        }

        markers
    }

    /// Build regex pattern for current markers
    fn build_pattern(&self) -> Regex {
        let markers = self.get_markers();
        if markers.is_empty() {
            return MARKER_REGEX.clone();
        }

        let pattern = format!(r"(?i)\b({})\b", markers.join("|"));
        Regex::new(&pattern).unwrap_or_else(|_| MARKER_REGEX.clone())
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

    /// Check if a match is inside an HTML comment
    fn is_in_html_comment(&self, line: &str, col: usize) -> bool {
        // Simple check: look for <!-- before and --> after
        let before = &line[..col.min(line.len())];
        let after = &line[col.min(line.len())..];

        before.contains("<!--") && !before.contains("-->") && after.contains("-->")
    }
}

impl Rule for CONTENT001 {
    fn id(&self) -> &'static str {
        "CONTENT001"
    }

    fn name(&self) -> &'static str {
        "no-todo-comments"
    }

    fn description(&self) -> &'static str {
        "TODO/FIXME/XXX comments should be resolved before publishing"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Content).introduced_in("mdbook-lint v0.11.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> mdbook_lint_core::error::Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let pattern = self.build_pattern();

        for (line_idx, line) in document.lines.iter().enumerate() {
            let line_num = line_idx + 1; // 1-based

            // Skip code blocks unless configured to check them
            if !self.check_code_blocks && self.is_in_code_block(&document.lines, line_idx) {
                continue;
            }

            // Find all matches for standard markers in this line
            for mat in pattern.find_iter(line) {
                let col = mat.start() + 1; // 1-based

                // Skip if inside inline code (unless checking code blocks)
                if !self.check_code_blocks && self.is_in_inline_code(line, mat.start()) {
                    continue;
                }

                // Always report HTML comments - they're often used for TODOs
                let in_comment = self.is_in_html_comment(line, mat.start());

                let marker = mat.as_str().to_uppercase();
                let context = if in_comment {
                    format!("{} comment found in HTML comment", marker)
                } else {
                    format!("{} comment found - resolve before publishing", marker)
                };

                violations.push(self.create_violation(context, line_num, col, Severity::Warning));
            }

            // Check for contextual markers (BUG) that need comment-style context
            for cap in CONTEXTUAL_MARKER_REGEX.captures_iter(line) {
                // Get the position of the BUG marker itself
                if let Some(mat) = cap.get(1) {
                    let col = mat.start() + 1; // 1-based

                    // Skip if inside inline code (unless checking code blocks)
                    if !self.check_code_blocks && self.is_in_inline_code(line, mat.start()) {
                        continue;
                    }

                    let in_comment = self.is_in_html_comment(line, mat.start());

                    let marker = mat.as_str().to_uppercase();
                    let context = if in_comment {
                        format!("{} comment found in HTML comment", marker)
                    } else {
                        format!("{} comment found - resolve before publishing", marker)
                    };

                    violations.push(self.create_violation(
                        context,
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
    fn test_no_markers() {
        let content = "# Title\n\nThis is clean documentation.";
        let doc = create_test_document(content);
        let rule = CONTENT001::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_todo_detected() {
        let content = "# Title\n\nTODO: Add more content here.";
        let doc = create_test_document(content);
        let rule = CONTENT001::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("TODO"));
    }

    #[test]
    fn test_fixme_detected() {
        let content = "# Title\n\nFIXME: This section needs work.";
        let doc = create_test_document(content);
        let rule = CONTENT001::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("FIXME"));
    }

    #[test]
    fn test_xxx_detected() {
        let content = "# Title\n\nXXX: Review this section.";
        let doc = create_test_document(content);
        let rule = CONTENT001::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("XXX"));
    }

    #[test]
    fn test_case_insensitive() {
        let content = "# Title\n\ntodo: lowercase\nFixMe: mixed case";
        let doc = create_test_document(content);
        let rule = CONTENT001::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_multiple_markers() {
        let content = "# Title\n\nTODO: First thing\nFIXME: Second thing\nHACK: Third thing";
        let doc = create_test_document(content);
        let rule = CONTENT001::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 3);
    }

    #[test]
    fn test_skip_code_blocks_by_default() {
        let content = "# Title\n\n```rust\n// TODO: This is in code\n```\n\nTODO: This is not";
        let doc = create_test_document(content);
        let rule = CONTENT001::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 7);
    }

    #[test]
    fn test_check_code_blocks_when_enabled() {
        let content = "# Title\n\n```rust\n// TODO: This is in code\n```";
        let doc = create_test_document(content);
        let rule = CONTENT001::default().check_code_blocks(true);
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_skip_inline_code() {
        let content = "# Title\n\nUse `TODO` as a marker.\n\nTODO: Real marker";
        let doc = create_test_document(content);
        let rule = CONTENT001::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 5);
    }

    #[test]
    fn test_html_comment() {
        let content = "# Title\n\n<!-- TODO: Add content -->\n\nParagraph.";
        let doc = create_test_document(content);
        let rule = CONTENT001::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("HTML comment"));
    }

    #[test]
    fn test_word_boundary() {
        let content = "# Title\n\nTODONOT a marker\nMYTODO not a marker";
        let doc = create_test_document(content);
        let rule = CONTENT001::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0, "Should not match partial words");
    }

    #[test]
    fn test_wip_detected() {
        let content = "# Title\n\nWIP: Work in progress section.";
        let doc = create_test_document(content);
        let rule = CONTENT001::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("WIP"));
    }

    #[test]
    fn test_custom_markers() {
        let content = "# Title\n\nNEEDSREVIEW: Check this.";
        let doc = create_test_document(content);
        let rule = CONTENT001::with_markers(vec!["NEEDSREVIEW".to_string()]);
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_marker_with_colon() {
        let content = "# Title\n\nTODO: With colon\nFIXME - With dash";
        let doc = create_test_document(content);
        let rule = CONTENT001::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_marker_in_parentheses() {
        let content = "# Title\n\n(TODO) In parens\n(FIXME) Also in parens";
        let doc = create_test_document(content);
        let rule = CONTENT001::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_bug_in_prose_not_detected() {
        // "bug" in normal prose should NOT be flagged
        let content = r#"# Title

This kind of bug can be difficult to track down.
The bug fix was released yesterday.
We found a bug in the code.
"#;
        let doc = create_test_document(content);
        let rule = CONTENT001::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0, "BUG in prose should not be flagged");
    }

    #[test]
    fn test_bug_comment_style_detected() {
        // BUG with comment-style context SHOULD be flagged
        let content = r#"# Title

BUG: This needs to be fixed.
BUG(123): Tracked issue.
// BUG: In a code comment style
"#;
        let doc = create_test_document(content);
        let rule = CONTENT001::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.len() >= 2,
            "BUG: style comments should be flagged, got {}",
            violations.len()
        );
    }

    #[test]
    fn test_bug_in_html_comment() {
        let content = "# Title\n\n<!-- BUG: Fix this -->\n\nParagraph.";
        let doc = create_test_document(content);
        let rule = CONTENT001::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("BUG"));
    }
}
