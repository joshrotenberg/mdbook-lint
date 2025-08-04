//! MD018: No space after hash on atx style heading
//!
//! This rule checks for missing space after hash characters in ATX style headings.

use crate::error::Result;
use crate::rule::{Rule, RuleCategory, RuleMetadata};
use crate::{
    Document,
    violation::{Severity, Violation},
};

/// Rule to check for missing space after hash on ATX style headings
pub struct MD018;

impl Rule for MD018 {
    fn id(&self) -> &'static str {
        "MD018"
    }

    fn name(&self) -> &'static str {
        "no-missing-space-atx"
    }

    fn description(&self) -> &'static str {
        "No space after hash on atx style heading"
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
            if trimmed.starts_with('#') && !trimmed.starts_with("#!") {
                // Find where the heading content starts
                let hash_count = trimmed.chars().take_while(|&c| c == '#').count();

                // Check if there's content after the hashes
                if trimmed.len() > hash_count {
                    let after_hashes = &trimmed[hash_count..];

                    // If there's content but no space, it's a violation
                    if !after_hashes.is_empty() && !after_hashes.starts_with(' ') {
                        let column = line.len() - line.trim_start().len() + hash_count + 1;

                        violations.push(self.create_violation(
                            "No space after hash on atx style heading".to_string(),
                            line_num,
                            column,
                            Severity::Warning,
                        ));
                    }
                }
            }
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
    fn test_md018_valid_headings() {
        let content = "# Heading 1\n## Heading 2\n### Heading 3";
        let document = create_test_document(content);
        let rule = MD018;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md018_no_space_after_hash() {
        let content = "#Heading without space";
        let document = create_test_document(content);
        let rule = MD018;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD018");
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[0].column, 2);
        assert!(violations[0].message.contains("No space after hash"));
    }

    #[test]
    fn test_md018_multiple_violations() {
        let content = "#Heading 1\n##Heading 2\n### Valid heading\n####Another violation";
        let document = create_test_document(content);
        let rule = MD018;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 2);
        assert_eq!(violations[2].line, 4);
    }

    #[test]
    fn test_md018_indented_heading() {
        let content = "  #Indented heading without space";
        let document = create_test_document(content);
        let rule = MD018;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].column, 4); // After the hash
    }

    #[test]
    fn test_md018_empty_heading() {
        let content = "#\n##\n###";
        let document = create_test_document(content);
        let rule = MD018;
        let violations = rule.check(&document).unwrap();

        // Empty headings (just hashes) should not trigger violations
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md018_closed_atx_style() {
        let content = "#Heading#\n##Another#Heading##";
        let document = create_test_document(content);
        let rule = MD018;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 2);
    }

    #[test]
    fn test_md018_setext_headings_ignored() {
        let content = "Setext Heading\n==============\n\nAnother Setext\n--------------";
        let document = create_test_document(content);
        let rule = MD018;
        let violations = rule.check(&document).unwrap();

        // Setext headings should not trigger this rule
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md018_mixed_valid_invalid() {
        let content = "# Valid heading\n#Invalid heading\n## Another valid\n###Invalid again";
        let document = create_test_document(content);
        let rule = MD018;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 2);
        assert_eq!(violations[1].line, 4);
    }

    #[test]
    fn test_md018_shebang_lines_ignored() {
        let content = "#!/bin/bash\n#This should trigger\n#!/usr/bin/env python3\n# This is valid";
        let document = create_test_document(content);
        let rule = MD018;
        let violations = rule.check(&document).unwrap();

        // Only the actual malformed heading should trigger, not shebangs
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 2);
        assert!(violations[0].message.contains("No space after hash"));
    }
}
