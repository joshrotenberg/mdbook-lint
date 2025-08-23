//! MD010: Hard tabs
//!
//! This rule checks for hard tab characters in the document.

use crate::error::Result;
use crate::rule::{Rule, RuleCategory, RuleMetadata};
use crate::{
    Document,
    violation::{Fix, Position, Severity, Violation},
};

/// Rule to check for hard tab characters
pub struct MD010 {
    /// Number of spaces that a tab character is equivalent to (for reporting)
    spaces_per_tab: usize,
}

impl MD010 {
    /// Create a new MD010 rule with default settings
    pub fn new() -> Self {
        Self { spaces_per_tab: 4 }
    }

    /// Create a new MD010 rule with custom tab size
    #[allow(dead_code)]
    pub fn with_spaces_per_tab(spaces_per_tab: usize) -> Self {
        Self { spaces_per_tab }
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

        for (line_number, line) in document.lines.iter().enumerate() {
            let line_num = line_number + 1; // Convert to 1-based line numbers

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
    use crate::rule::Rule;
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
}
