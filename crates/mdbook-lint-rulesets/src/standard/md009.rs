//! MD009: No trailing spaces
//!
//! This rule checks for trailing spaces at the end of lines.
//!
//! ## Why This Rule Exists
//!
//! Trailing spaces are usually unintentional and can cause issues:
//! - They're invisible in most editors, making them hard to spot
//! - They can cause unexpected behavior in version control systems
//! - They may render differently across different markdown processors
//! - They increase file size unnecessarily
//!
//! ## Examples
//!
//! ### ❌ Incorrect (violates rule)
//!
//! ```text
//! This line has trailing spaces   ← spaces
//! This one has a tab at the end    ← tab
//! Multiple spaces here    ← spaces
//! ```
//!
//! (Where arrows indicate invisible whitespace characters)
//!
//! ### ✅ Correct
//!
//! ```markdown
//! This line has no trailing spaces
//! This one is clean too
//! Two spaces for line break are allowed  
//! when configured (br_spaces = 2)
//! ```
//!
//! ## Configuration
//!
//! ```toml
//! [rules.MD009]
//! br_spaces = 2  # Number of trailing spaces allowed for line breaks (default: 2)
//! strict = false # If true, disallow even configured line break spaces (default: false)
//! ```
//!
//! ## Automatic Fix
//!
//! This rule supports automatic fixing. The fix will:
//! - Remove all trailing whitespace from lines
//! - Preserve configured line break spaces (typically 2 spaces)
//! - Maintain the line's content and structure
//!
//! ## When to Disable
//!
//! Consider disabling this rule if:
//! - Your project intentionally uses trailing spaces for formatting
//! - You're working with generated content that includes trailing spaces
//! - You need to preserve exact whitespace for technical documentation

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Fix, Position, Severity, Violation},
};

/// Rule to check for trailing spaces at the end of lines
pub struct MD009 {
    /// Whether to allow trailing spaces in code blocks
    br_spaces: usize,
    /// Whether to allow trailing spaces at the end of list items
    list_item_empty_lines: bool,
    /// Whether to ignore trailing spaces in strict mode
    strict: bool,
}

impl MD009 {
    /// Create a new MD009 rule with default settings
    pub fn new() -> Self {
        Self {
            br_spaces: 2, // Allow 2 trailing spaces for line breaks
            list_item_empty_lines: false,
            strict: false,
        }
    }

    /// Create a new MD009 rule with custom settings
    #[allow(dead_code)]
    pub fn with_config(br_spaces: usize, list_item_empty_lines: bool, strict: bool) -> Self {
        Self {
            br_spaces,
            list_item_empty_lines,
            strict,
        }
    }

    /// Create MD009 from configuration
    pub fn from_config(config: &toml::Value) -> Self {
        let mut rule = Self::new();

        if let Some(br_spaces) = config
            .get("br-spaces")
            .or_else(|| config.get("br_spaces"))
            .and_then(|v| v.as_integer())
        {
            rule.br_spaces = br_spaces as usize;
        }

        if let Some(list_item) = config
            .get("list-item-empty-lines")
            .or_else(|| config.get("list_item_empty_lines"))
            .and_then(|v| v.as_bool())
        {
            rule.list_item_empty_lines = list_item;
        }

        if let Some(strict) = config.get("strict").and_then(|v| v.as_bool()) {
            rule.strict = strict;
        }

        rule
    }
}

impl Default for MD009 {
    fn default() -> Self {
        Self::new()
    }
}

impl AstRule for MD009 {
    fn id(&self) -> &'static str {
        "MD009"
    }

    fn name(&self) -> &'static str {
        "no-trailing-spaces"
    }

    fn description(&self) -> &'static str {
        "Trailing spaces are not allowed"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Formatting).introduced_in("markdownlint v0.1.0")
    }

    fn check_ast<'a>(
        &self,
        document: &Document,
        ast: &'a comrak::nodes::AstNode<'a>,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // Get code block line ranges from provided AST
        let code_block_lines = self.get_code_block_line_ranges(ast);
        let list_item_lines = if self.list_item_empty_lines {
            self.get_list_item_empty_lines(ast)
        } else {
            Vec::new()
        };

        for (line_number, line) in document.lines.iter().enumerate() {
            let line_num = line_number + 1; // Convert to 1-based line numbers

            // Skip if line has no trailing spaces
            if !line.ends_with(' ') && !line.ends_with('\t') {
                continue;
            }

            // Count trailing whitespace
            let trailing_spaces = line.chars().rev().take_while(|c| c.is_whitespace()).count();

            // Check if this line is in a code block
            let in_code_block = code_block_lines
                .iter()
                .any(|(start, end)| line_num >= *start && line_num <= *end);

            // Skip code blocks unless in strict mode
            if in_code_block && !self.strict {
                continue;
            }

            // Check if this is a list item empty line that we should ignore
            if self.list_item_empty_lines && list_item_lines.contains(&line_num) {
                continue;
            }

            // Allow exactly br_spaces trailing spaces for line breaks (markdown soft breaks)
            if !self.strict && trailing_spaces == self.br_spaces {
                continue;
            }

            // Create violation with fix
            let column = line.len() - trailing_spaces + 1;

            // Create the fixed line by removing trailing whitespace
            let fixed_line = line.trim_end().to_string() + "\n";

            let fix = Fix {
                description: format!(
                    "Remove {} trailing space{}",
                    trailing_spaces,
                    if trailing_spaces == 1 { "" } else { "s" }
                ),
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
                    "Trailing spaces detected (found {} trailing space{})",
                    trailing_spaces,
                    if trailing_spaces == 1 { "" } else { "s" }
                ),
                line_num,
                column,
                Severity::Warning,
                fix,
            ));
        }

        Ok(violations)
    }
}

impl MD009 {
    /// Get line ranges for code blocks to potentially skip them
    fn get_code_block_line_ranges<'a>(
        &self,
        ast: &'a comrak::nodes::AstNode<'a>,
    ) -> Vec<(usize, usize)> {
        let mut ranges = Vec::new();
        self.collect_code_block_ranges(ast, &mut ranges);
        ranges
    }

    /// Recursively collect code block line ranges
    #[allow(clippy::only_used_in_recursion)]
    fn collect_code_block_ranges<'a>(
        &self,
        node: &'a comrak::nodes::AstNode<'a>,
        ranges: &mut Vec<(usize, usize)>,
    ) {
        use comrak::nodes::NodeValue;

        if let NodeValue::CodeBlock(_) = &node.data.borrow().value {
            let sourcepos = node.data.borrow().sourcepos;
            if sourcepos.start.line > 0 && sourcepos.end.line > 0 {
                ranges.push((sourcepos.start.line, sourcepos.end.line));
            }
        }

        for child in node.children() {
            self.collect_code_block_ranges(child, ranges);
        }
    }

    /// Get empty lines within list items (if list_item_empty_lines is enabled)
    fn get_list_item_empty_lines<'a>(&self, ast: &'a comrak::nodes::AstNode<'a>) -> Vec<usize> {
        let mut lines = Vec::new();
        self.collect_list_item_empty_lines(ast, &mut lines);
        lines
    }

    /// Recursively collect empty lines within list items
    /// Collect empty lines within list items
    #[allow(clippy::only_used_in_recursion)]
    fn collect_list_item_empty_lines<'a>(
        &self,
        node: &'a comrak::nodes::AstNode<'a>,
        lines: &mut Vec<usize>,
    ) {
        use comrak::nodes::NodeValue;

        if let NodeValue::Item(_) = &node.data.borrow().value {
            // For now, we don't implement the complex logic to identify empty lines within list items
            // This would require more sophisticated AST analysis
        }

        for child in node.children() {
            self.collect_list_item_empty_lines(child, lines);
        }
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
    fn test_md009_no_trailing_spaces() {
        let content = "# Heading\n\nNo trailing spaces here.\nAnother clean line.";
        let document = create_test_document(content);
        let rule = MD009::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md009_single_trailing_space() {
        let content = "# Heading\n\nLine with single trailing space. \nClean line.";
        let document = create_test_document(content);
        let rule = MD009::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD009");
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[0].column, 33);
        assert!(violations[0].message.contains("1 trailing space"));
    }

    #[test]
    fn test_md009_multiple_trailing_spaces() {
        let content = "# Heading\n\nLine with spaces.   \nAnother line.    ";
        let document = create_test_document(content);
        let rule = MD009::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 3);
        assert!(violations[0].message.contains("3 trailing spaces"));
        assert_eq!(violations[1].line, 4);
        assert!(violations[1].message.contains("4 trailing spaces"));
    }

    #[test]
    fn test_md009_trailing_tabs() {
        let content = "# Heading\n\nLine with trailing tab.\t\nClean line.";
        let document = create_test_document(content);
        let rule = MD009::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3);
        assert!(violations[0].message.contains("1 trailing space"));
    }

    #[test]
    fn test_md009_line_break_spaces() {
        let content = "# Heading\n\nLine with two spaces for break.  \nNext line.";
        let document = create_test_document(content);
        let rule = MD009::new();
        let violations = rule.check(&document).unwrap();

        // Should allow exactly 2 trailing spaces for line breaks
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md009_strict_mode() {
        let content = "# Heading\n\nLine with two spaces.  \nThree spaces.   ";
        let document = create_test_document(content);
        let rule = MD009::with_config(2, false, true);
        let violations = rule.check(&document).unwrap();

        // In strict mode, no trailing spaces are allowed
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_md009_code_block_ignored() {
        let content = "# Heading\n\n```rust\nlet x = 1;  \n```\n\nRegular line.   ";
        let document = create_test_document(content);
        let rule = MD009::new();
        let violations = rule.check(&document).unwrap();

        // Should ignore trailing spaces in code blocks but catch them in regular text
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 7);
    }

    #[test]
    fn test_md009_code_block_strict() {
        let content = "# Heading\n\n```rust\nlet x = 1;  \n```\n\nRegular line.   ";
        let document = create_test_document(content);
        let rule = MD009::with_config(2, false, true);
        let violations = rule.check(&document).unwrap();

        // In strict mode, should catch trailing spaces everywhere
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_md009_fix_single_trailing_space() {
        let content = "Line with trailing space ";
        let document = create_test_document(content);
        let rule = MD009::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement.as_ref().unwrap(),
            "Line with trailing space\n"
        );
        assert_eq!(fix.description, "Remove 1 trailing space");
    }

    #[test]
    fn test_md009_fix_multiple_trailing_spaces() {
        let content = "Line with spaces    ";
        let document = create_test_document(content);
        let rule = MD009::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement.as_ref().unwrap(), "Line with spaces\n");
        assert_eq!(fix.description, "Remove 4 trailing spaces");
    }

    #[test]
    fn test_md009_fix_trailing_tabs() {
        let content = "Line with tab\t";
        let document = create_test_document(content);
        let rule = MD009::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement.as_ref().unwrap(), "Line with tab\n");
        assert_eq!(fix.description, "Remove 1 trailing space");
    }

    #[test]
    fn test_md009_fix_mixed_trailing_whitespace() {
        let content = "Line with mixed \t  \t";
        let document = create_test_document(content);
        let rule = MD009::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement.as_ref().unwrap(), "Line with mixed\n");
        assert_eq!(fix.description, "Remove 5 trailing spaces");
    }

    #[test]
    fn test_md009_fix_preserves_line_content() {
        let content = "Important content with spaces   "; // 3 spaces to trigger violation
        let document = create_test_document(content);
        let rule = MD009::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert!(
            fix.replacement
                .as_ref()
                .unwrap()
                .starts_with("Important content with spaces")
        );
        assert!(!fix.replacement.as_ref().unwrap().contains("   \n"));
    }

    #[test]
    fn test_md009_fix_position_accuracy() {
        let content = "Line with trailing spaces   ";
        let document = create_test_document(content);
        let rule = MD009::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.start.line, 1);
        assert_eq!(fix.start.column, 1);
        assert_eq!(fix.end.line, 1);
        assert_eq!(fix.end.column, content.len() + 1);
    }

    #[test]
    fn test_md009_no_fix_for_allowed_line_breaks() {
        let content = "Line with two spaces for break  ";
        let document = create_test_document(content);
        let rule = MD009::new(); // Default allows 2 spaces for line breaks
        let violations = rule.check(&document).unwrap();

        // Should not create violations for exactly 2 trailing spaces
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md009_fix_multiple_lines() {
        let content = "First line with space \nSecond line with tabs\t\nThird line with many     ";
        let document = create_test_document(content);
        let rule = MD009::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);

        // Check first line fix
        assert_eq!(violations[0].line, 1);
        assert_eq!(
            violations[0]
                .fix
                .as_ref()
                .unwrap()
                .replacement
                .as_ref()
                .unwrap(),
            "First line with space\n"
        );

        // Check second line fix
        assert_eq!(violations[1].line, 2);
        assert_eq!(
            violations[1]
                .fix
                .as_ref()
                .unwrap()
                .replacement
                .as_ref()
                .unwrap(),
            "Second line with tabs\n"
        );

        // Check third line fix
        assert_eq!(violations[2].line, 3);
        assert_eq!(
            violations[2]
                .fix
                .as_ref()
                .unwrap()
                .replacement
                .as_ref()
                .unwrap(),
            "Third line with many\n"
        );
    }
}
