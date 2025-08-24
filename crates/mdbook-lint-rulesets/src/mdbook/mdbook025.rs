//! MDBOOK025: Multiple H1 headings allowed in SUMMARY.md
//!
//! This rule overrides MD025 for mdBook projects to allow multiple H1 headings
//! in SUMMARY.md files, which legitimately use them for organizing parts and sections.

use comrak::nodes::AstNode;
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{Document, violation::Violation};

/// Rule that allows multiple H1 headings in SUMMARY.md files
///
/// In mdBook projects, SUMMARY.md files legitimately use multiple H1 headings
/// to organize content into parts and sections. This rule overrides the standard
/// MD025 behavior specifically for SUMMARY.md files.
pub struct MDBOOK025;

impl MDBOOK025 {
    /// Create a new MDBOOK025 rule
    pub fn new() -> Self {
        Self
    }
}

impl Default for MDBOOK025 {
    fn default() -> Self {
        Self::new()
    }
}

impl AstRule for MDBOOK025 {
    fn id(&self) -> &'static str {
        "MDBOOK025"
    }

    fn name(&self) -> &'static str {
        "summary-multiple-h1-allowed"
    }

    fn description(&self) -> &'static str {
        "Multiple H1 headings are allowed in SUMMARY.md files"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Structure)
            .introduced_in("mdbook-lint v0.4.0")
            .overrides("MD025")
    }

    fn check_ast<'a>(&self, document: &Document, _ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        // This rule only applies to SUMMARY.md files
        if let Some(filename) = document.path.file_name()
            && filename == "SUMMARY.md"
        {
            // Always allow multiple H1s in SUMMARY.md - return no violations
            return Ok(Vec::new());
        }

        // For non-SUMMARY.md files, this rule doesn't apply
        // The standard MD025 rule will handle them
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::rule::Rule;
    use std::path::PathBuf;

    #[test]
    fn test_mdbook025_summary_file_multiple_h1s_allowed() {
        let content = r#"# Summary

[Introduction](introduction.md)

# Part I: Getting Started

- [Chapter 1](chapter1.md)

# Part II: Advanced Topics

- [Chapter 2](chapter2.md)
"#;
        let document = Document::new(content.to_string(), PathBuf::from("SUMMARY.md")).unwrap();
        let rule = MDBOOK025::new();
        let violations = rule.check(&document).unwrap();

        // SUMMARY.md should not trigger MDBOOK025 violations despite multiple H1s
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_mdbook025_non_summary_file_ignored() {
        let content = r#"# First H1 heading
Some content here.

# Second H1 heading
More content.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK025::new();
        let violations = rule.check(&document).unwrap();

        // Non-SUMMARY.md files are ignored by this rule (MD025 handles them)
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_mdbook025_summary_with_single_h1() {
        let content = r#"# Summary

- [Chapter 1](chapter1.md)
- [Chapter 2](chapter2.md)
"#;
        let document = Document::new(content.to_string(), PathBuf::from("SUMMARY.md")).unwrap();
        let rule = MDBOOK025::new();
        let violations = rule.check(&document).unwrap();

        // Single H1 in SUMMARY.md is also fine
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_mdbook025_rule_metadata() {
        use mdbook_lint_core::rule::AstRule;
        let rule = MDBOOK025::new();

        assert_eq!(AstRule::id(&rule), "MDBOOK025");
        assert_eq!(AstRule::name(&rule), "summary-multiple-h1-allowed");
        assert!(AstRule::description(&rule).contains("SUMMARY.md"));

        let metadata = AstRule::metadata(&rule);
        assert_eq!(metadata.category, RuleCategory::Structure);
        assert!(metadata.overrides.as_ref().unwrap().contains("MD025"));
    }
}
