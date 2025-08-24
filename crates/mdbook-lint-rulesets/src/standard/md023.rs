//! MD023: Headings must start at the beginning of the line
//!
//! This rule checks that headings are not indented with spaces or tabs.

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Fix, Position, Severity, Violation},
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

                // Create fixed line by removing the indentation
                let fixed_line = format!("{}\n", trimmed);

                let fix = Fix {
                    description: format!(
                        "Remove {} character{} of indentation",
                        leading_whitespace,
                        if leading_whitespace == 1 { "" } else { "s" }
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
                        "Heading is indented by {} character{}",
                        leading_whitespace,
                        if leading_whitespace == 1 { "" } else { "s" }
                    ),
                    line_num,
                    1,
                    Severity::Warning,
                    fix,
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
    use mdbook_lint_core::rule::Rule;
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

    #[test]
    fn test_md023_fix_single_space_indent() {
        let content = " # Heading";
        let document = create_test_document(content);
        let rule = MD023;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement.as_ref().unwrap(), "# Heading\n");
        assert_eq!(fix.description, "Remove 1 character of indentation");
    }

    #[test]
    fn test_md023_fix_multiple_spaces_indent() {
        let content = "    ## Heading with 4 spaces";
        let document = create_test_document(content);
        let rule = MD023;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement.as_ref().unwrap(),
            "## Heading with 4 spaces\n"
        );
        assert_eq!(fix.description, "Remove 4 characters of indentation");
    }

    #[test]
    fn test_md023_fix_tab_indent() {
        let content = "\t# Tab indented";
        let document = create_test_document(content);
        let rule = MD023;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement.as_ref().unwrap(), "# Tab indented\n");
        assert_eq!(fix.description, "Remove 1 character of indentation");
    }

    #[test]
    fn test_md023_fix_mixed_whitespace() {
        let content = " \t ### Mixed indent";
        let document = create_test_document(content);
        let rule = MD023;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement.as_ref().unwrap(), "### Mixed indent\n");
        assert_eq!(fix.description, "Remove 3 characters of indentation");
    }

    #[test]
    fn test_md023_fix_closed_atx() {
        let content = "  ## Closed heading ##";
        let document = create_test_document(content);
        let rule = MD023;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement.as_ref().unwrap(), "## Closed heading ##\n");
    }

    #[test]
    fn test_md023_fix_multiple_headings() {
        let content = " # First\n  ## Second\n   ### Third";
        let document = create_test_document(content);
        let rule = MD023;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);
        assert_eq!(
            violations[0]
                .fix
                .as_ref()
                .unwrap()
                .replacement
                .as_ref()
                .unwrap(),
            "# First\n"
        );
        assert_eq!(
            violations[1]
                .fix
                .as_ref()
                .unwrap()
                .replacement
                .as_ref()
                .unwrap(),
            "## Second\n"
        );
        assert_eq!(
            violations[2]
                .fix
                .as_ref()
                .unwrap()
                .replacement
                .as_ref()
                .unwrap(),
            "### Third\n"
        );
    }

    #[test]
    fn test_md023_fix_position_accuracy() {
        let content = "  # Indented";
        let document = create_test_document(content);
        let rule = MD023;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.start.line, 1);
        assert_eq!(fix.start.column, 1);
        assert_eq!(fix.end.line, 1);
        assert_eq!(fix.end.column, content.len() + 1);
    }

    #[test]
    fn test_md023_fix_all_heading_levels() {
        let content = " #H1\n  ##H2\n   ###H3\n    ####H4\n     #####H5\n      ######H6";
        let document = create_test_document(content);
        let rule = MD023;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 6);
        for violation in violations.iter() {
            assert!(violation.fix.is_some());
            let fix = violation.fix.as_ref().unwrap();
            assert!(fix.replacement.as_ref().unwrap().starts_with("#"));
            assert!(!fix.replacement.as_ref().unwrap().starts_with(" "));
        }
    }
}
