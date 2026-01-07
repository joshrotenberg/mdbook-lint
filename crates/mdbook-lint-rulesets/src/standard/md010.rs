//! MD010: Hard tabs
//!
//! This rule checks for hard tab characters in the document.
//!
//! ## Why This Rule Exists
//!
//! Hard tabs can cause formatting inconsistencies:
//!
//! - Tab width varies between editors (2, 4, or 8 spaces)
//! - Mixing tabs and spaces leads to misaligned text
//! - Different markdown renderers may handle tabs differently
//! - Code blocks and indentation become unpredictable
//!
//! ## Examples
//!
//! ### ❌ Incorrect (violates rule)
//!
//! ```text
//! →   This line starts with a tab
//! -→  List item with tab after marker
//! ```→   Code block with tab indent
//! ```
//!
//! (Where → represents a tab character)
//!
//! ### ✅ Correct
//!
//! ```markdown
//!     This line uses spaces for indentation
//! -   List item with spaces after marker
//! ```    Code block with space indent
//! ```
//!
//! ## Configuration
//!
//! ```toml
//! [rules.MD010]
//! code_blocks = true  # Check for tabs in code blocks (default: true)
//! spaces_per_tab = 4  # Number of spaces to replace each tab with (default: 4)
//! ```
//!
//! ## Automatic Fix
//!
//! This rule supports automatic fixing. The fix will:
//!
//! - Replace each tab character with the configured number of spaces
//! - Preserve the visual indentation of your content
//! - Handle tabs in all contexts (text, lists, code blocks)
//!
//! ## When to Disable
//!
//! Consider disabling this rule if:
//!
//! - Your project standard requires tabs
//! - You're working with tab-delimited data files
//! - You're documenting makefiles or other tab-sensitive formats

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Fix, Position, Severity, Violation},
};

/// Rule to check for hard tab characters
pub struct MD010 {
    /// Number of spaces that a tab character is equivalent to (for reporting)
    spaces_per_tab: usize,
    /// Whether to check for tabs inside code blocks (default: true)
    code_blocks: bool,
}

impl MD010 {
    /// Create a new MD010 rule with default settings
    pub fn new() -> Self {
        Self {
            spaces_per_tab: 4,
            code_blocks: true,
        }
    }

    /// Create a new MD010 rule with custom tab size
    #[allow(dead_code)]
    pub fn with_spaces_per_tab(spaces_per_tab: usize) -> Self {
        Self {
            spaces_per_tab,
            code_blocks: true,
        }
    }

    /// Set whether to check for tabs inside code blocks
    #[allow(dead_code)]
    pub fn with_code_blocks(mut self, check_code_blocks: bool) -> Self {
        self.code_blocks = check_code_blocks;
        self
    }

    /// Create MD010 from configuration
    pub fn from_config(config: &toml::Value) -> Self {
        let mut rule = Self::new();

        if let Some(spaces) = config
            .get("spaces-per-tab")
            .or_else(|| config.get("spaces_per_tab"))
            .and_then(|v| v.as_integer())
        {
            rule.spaces_per_tab = spaces as usize;
        }

        if let Some(code_blocks) = config
            .get("code-blocks")
            .or_else(|| config.get("code_blocks"))
            .and_then(|v| v.as_bool())
        {
            rule.code_blocks = code_blocks;
        }

        rule
    }
}

impl Default for MD010 {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for MD010 {
    fn id(&self) -> &'static str {
        "MD010"
    }

    fn name(&self) -> &'static str {
        "no-hard-tabs"
    }

    fn description(&self) -> &'static str {
        "Hard tabs are not allowed"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Formatting).introduced_in("markdownlint v0.1.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let mut in_code_block = false;

        for (line_number, line) in document.lines.iter().enumerate() {
            let line_num = line_number + 1; // Convert to 1-based line numbers

            // Track fenced code block boundaries
            let trimmed = line.trim_start();
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                // Still check the fence line itself for tabs
            }

            // Skip code block content if code_blocks is false
            if !self.code_blocks
                && in_code_block
                && !trimmed.starts_with("```")
                && !trimmed.starts_with("~~~")
            {
                continue;
            }

            // Check for tab characters
            if let Some(tab_pos) = line.find('\t') {
                let column = tab_pos + 1; // Convert to 1-based column

                // Create replacement string with spaces
                let replacement = " ".repeat(self.spaces_per_tab);

                // Find all tabs in the line to create a complete fix
                let fixed_line = line.replace('\t', &replacement);

                let fix = Fix {
                    description: format!("Replace tab with {} spaces", self.spaces_per_tab),
                    replacement: Some(fixed_line),
                    start: Position {
                        line: line_num,
                        column: 1,
                    },
                    end: Position {
                        line: line_num,
                        column: line.len() + 1,
                    },
                };

                violations.push(self.create_violation_with_fix(
                    format!(
                        "Hard tab character found (consider using {} spaces)",
                        self.spaces_per_tab
                    ),
                    line_num,
                    column,
                    Severity::Warning,
                    fix,
                ));
            }
        }

        Ok(violations)
    }

    fn can_fix(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::rule::Rule;
    use std::path::PathBuf;

    fn create_test_document(content: &str) -> Document {
        Document::new(content.to_string(), PathBuf::from("test.md")).unwrap()
    }

    #[test]
    fn test_md010_no_tabs() {
        let content = "# Heading\n\nNo tabs here.\nJust spaces.";
        let document = create_test_document(content);
        let rule = MD010::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md010_single_tab() {
        let content = "# Heading\n\nLine with\ttab.\nClean line.";
        let document = create_test_document(content);
        let rule = MD010::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD010");
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[0].column, 10);
        assert!(violations[0].message.contains("Hard tab character"));
        assert!(violations[0].message.contains("4 spaces"));

        // Check fix is present
        assert!(violations[0].fix.is_some());
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Replace tab with 4 spaces");
        assert_eq!(fix.replacement, Some("Line with    tab.".to_string()));
    }

    #[test]
    fn test_md010_multiple_tabs() {
        let content = "# Heading\n\nLine\twith\ttabs.\nAnother\ttab line.";
        let document = create_test_document(content);
        let rule = MD010::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[0].column, 5); // First tab position
        assert_eq!(violations[1].line, 4);
        assert_eq!(violations[1].column, 8); // First tab in second line
    }

    #[test]
    fn test_md010_custom_spaces_per_tab() {
        let content = "Line with\ttab.";
        let document = create_test_document(content);
        let rule = MD010::with_spaces_per_tab(2);
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("2 spaces"));
    }

    #[test]
    fn test_md010_tab_at_beginning() {
        let content = "\tIndented with tab";
        let document = create_test_document(content);
        let rule = MD010::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].column, 1);
    }

    #[test]
    fn test_md010_only_first_tab_reported() {
        let content = "Line\twith\tmultiple\ttabs";
        let document = create_test_document(content);
        let rule = MD010::new();
        let violations = rule.check(&document).unwrap();

        // Should only report the first tab per line
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].column, 5);
    }

    #[test]
    fn test_md010_fix_single_tab() {
        let content = "Line with\ttab here.";
        let document = create_test_document(content);
        let rule = MD010::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Replace tab with 4 spaces");
        assert_eq!(fix.replacement, Some("Line with    tab here.".to_string()));
        assert_eq!(fix.start.line, 1);
        assert_eq!(fix.start.column, 1);
    }

    #[test]
    fn test_md010_fix_multiple_tabs() {
        let content = "Text\twith\tmultiple\ttabs.";
        let document = create_test_document(content);
        let rule = MD010::new();
        let violations = rule.check(&document).unwrap();

        // Only first tab is reported
        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        // Fix should replace ALL tabs on the line
        assert_eq!(
            fix.replacement,
            Some("Text    with    multiple    tabs.".to_string())
        );
    }

    #[test]
    fn test_md010_fix_custom_spaces() {
        let content = "\tIndented line.";
        let document = create_test_document(content);
        let rule = MD010::with_spaces_per_tab(2);
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Replace tab with 2 spaces");
        assert_eq!(fix.replacement, Some("  Indented line.".to_string()));
    }

    #[test]
    fn test_md010_check_code_blocks_by_default() {
        let content = "```\ncode\twith\ttab\n```\nText\twith tab.";
        let document = create_test_document(content);
        let rule = MD010::new();
        let violations = rule.check(&document).unwrap();

        // By default, code_blocks = true, so tabs in code blocks are flagged
        // This finds tabs on line 2 and line 4
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 2);
        assert_eq!(violations[1].line, 4);
        assert!(violations[0].fix.is_some());
        assert!(violations[1].fix.is_some());
    }

    #[test]
    fn test_md010_skip_code_blocks_when_disabled() {
        let content = "```\ncode\twith\ttab\n```\nText\twith tab.";
        let document = create_test_document(content);
        let rule = MD010::new().with_code_blocks(false);
        let violations = rule.check(&document).unwrap();

        // With code_blocks = false, only the tab outside code blocks is flagged
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 4);
    }

    #[test]
    fn test_md010_skip_code_blocks_with_tilde_fence() {
        let content = "~~~rust\ncode\twith\ttab\n~~~\nText\twith tab.";
        let document = create_test_document(content);
        let rule = MD010::new().with_code_blocks(false);
        let violations = rule.check(&document).unwrap();

        // With code_blocks = false, only the tab outside code blocks is flagged
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 4);
    }

    #[test]
    fn test_md010_multiple_code_blocks() {
        let content = "Text\twith tab.\n```\ncode\ttab\n```\nMore\ttext.\n~~~\nanother\tblock\n~~~\nFinal\ttab.";
        let document = create_test_document(content);
        let rule = MD010::new().with_code_blocks(false);
        let violations = rule.check(&document).unwrap();

        // Only tabs outside code blocks: lines 1, 5, 9
        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 5);
        assert_eq!(violations[2].line, 9);
    }

    #[test]
    fn test_md010_code_blocks_config_from_toml() {
        let config: toml::Value = toml::from_str(
            r#"
            code_blocks = false
            spaces_per_tab = 2
            "#,
        )
        .unwrap();

        let rule = MD010::from_config(&config);
        assert!(!rule.code_blocks);
        assert_eq!(rule.spaces_per_tab, 2);
    }

    #[test]
    fn test_md010_code_blocks_config_with_hyphen() {
        let config: toml::Value = toml::from_str(
            r#"
            code-blocks = false
            "#,
        )
        .unwrap();

        let rule = MD010::from_config(&config);
        assert!(!rule.code_blocks);
    }

    #[test]
    fn test_md010_fix_preserves_content() {
        let content = "Before\t\tmiddle\t\tafter.";
        let document = create_test_document(content);
        let rule = MD010::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        // All tabs should be replaced
        assert_eq!(
            fix.replacement,
            Some("Before        middle        after.".to_string())
        );
    }
}
