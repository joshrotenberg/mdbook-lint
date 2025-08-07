//! MD023: Headings must start at the beginning of the line
//!
//! This rule checks that headings are not indented with spaces or tabs.

use crate::error::Result;
use crate::rule::{Rule, RuleCategory, RuleMetadata};
use crate::{
    Document,
    violation::{Severity, Violation},
};

/// Rule to check that headings start at the beginning of the line
pub struct MD023;

impl Rule for MD023 {
    fn id(&self) -> &'static str {
        "MD023"
    }

    fn name(&self) -> &'static str {
        "heading-start-left"
    }

    fn description(&self) -> &'static str {
        "Headings must start at the beginning of the line"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Structure).introduced_in("markdownlint v0.1.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        for (line_number, line) in document.lines.iter().enumerate() {
            let line_num = line_number + 1; // Convert to 1-based line numbers

            // Check if this is an ATX-style heading (starts with #)
            // Skip shebang lines (#!/...)
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') && !trimmed.starts_with("#!") && line != trimmed {
                let leading_whitespace = line.len() - trimmed.len();

                violations.push(self.create_violation(
                    format!(
                        "Heading is indented by {} character{}",
                        leading_whitespace,
                        if leading_whitespace == 1 { "" } else { "s" }
                    ),
                    line_num,
                    1,
                    Severity::Warning,
                ));
            }
            // Note: Setext headings are handled differently as they span multiple lines
            // and the heading text itself might be indented, but we only care about
            // ATX headings for this rule
        }

        Ok(violations)
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
    fn test_md023_valid_headings() {
        let content = "# Heading 1\n## Heading 2\n### Heading 3";
        let document = create_test_document(content);
        let rule = MD023;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md023_single_space_indent() {
        let content = " # Indented heading";
        let document = create_test_document(content);
        let rule = MD023;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD023");
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[0].column, 1);
        assert!(violations[0].message.contains("indented by 1 character"));
    }

    #[test]
    fn test_md023_multiple_spaces_indent() {
        let content = "   ## Heading with 3 spaces";
        let document = create_test_document(content);
        let rule = MD023;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("indented by 3 characters"));
    }

    #[test]
    fn test_md023_tab_indent() {
        let content = "\t# Tab indented heading";
        let document = create_test_document(content);
        let rule = MD023;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("indented by 1 character"));
    }

    #[test]
    fn test_md023_mixed_whitespace_indent() {
        let content = " \t # Mixed whitespace indent";
        let document = create_test_document(content);
        let rule = MD023;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("indented by 3 characters"));
    }

    #[test]
    fn test_md023_multiple_violations() {
        let content = " # Heading 1\n## Valid heading\n  ### Heading 3\n#### Valid heading";
        let document = create_test_document(content);
        let rule = MD023;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 3);
    }

    #[test]
    fn test_md023_setext_headings_ignored() {
        let content = "  Setext Heading\n  ==============\n\n  Another Setext\n  --------------";
        let document = create_test_document(content);
        let rule = MD023;
        let violations = rule.check(&document).unwrap();

        // Setext headings should not trigger this rule (they don't start with #)
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md023_code_blocks_detected() {
        let content = "```\n  # This is in a code block\n  ## Should trigger\n```";
        let document = create_test_document(content);
        let rule = MD023;
        let violations = rule.check(&document).unwrap();

        // Simple line-based approach will detect indented # as violations
        // even in code blocks (more sophisticated parsing would be needed to avoid this)
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 2);
        assert_eq!(violations[1].line, 3);
    }

    #[test]
    fn test_md023_blockquote_headings() {
        let content = "> # Heading in blockquote\n>  ## Indented heading in blockquote";
        let document = create_test_document(content);
        let rule = MD023;
        let violations = rule.check(&document).unwrap();

        // Simple line-based approach doesn't understand blockquote context
        // so it won't detect these as headings since they don't start with #
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md023_closed_atx_headings() {
        let content = "  # Indented closed heading #\n   ## Another indented ##";
        let document = create_test_document(content);
        let rule = MD023;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("indented by 2 characters"));
        assert!(violations[1].message.contains("indented by 3 characters"));
    }

    #[test]
    fn test_md023_shebang_lines_ignored() {
        let content =
            "#!/bin/bash\n  #This should trigger\n  #!/usr/bin/env python3\n# This is valid";
        let document = create_test_document(content);
        let rule = MD023;
        let violations = rule.check(&document).unwrap();

        // Only the actual indented heading should trigger, not shebangs
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 2);
        assert!(violations[0].message.contains("indented by 2 characters"));
    }
}
