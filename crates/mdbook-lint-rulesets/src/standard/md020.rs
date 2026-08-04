//! MD020: Missing space inside hashes on a closed ATX heading
//!
//! Closed ATX headings should have a single separator between their opening and
//! closing hash sequences and the heading content.

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Fix, Position, Severity, Violation},
};

use super::atx::trailing_hash_sequence;

/// Rule to check for missing spaces inside closed ATX style headings.
pub struct MD020;

impl Rule for MD020 {
    fn id(&self) -> &'static str {
        "MD020"
    }

    fn name(&self) -> &'static str {
        "no-missing-space-closed-atx"
    }

    fn description(&self) -> &'static str {
        "Missing space inside hashes on closed atx style heading"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Formatting).introduced_in("mdbook-lint v0.1.0")
    }

    fn can_fix(&self) -> bool {
        true
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        for (line_number, line) in document.lines.iter().enumerate() {
            let line_num = line_number + 1;

            // Check if this is an ATX-style heading candidate (starts with
            // between one and six hashes). Skip shebang lines (#!/...).
            let trimmed = line.trim_start();
            if !trimmed.starts_with('#') || trimmed.starts_with("#!") {
                continue;
            }

            let opening_hash_count = trimmed.chars().take_while(|&c| c == '#').count();
            if !(1..=6).contains(&opening_hash_count) {
                continue;
            }

            let Some((closing_start, closing_end)) = trailing_hash_sequence(trimmed) else {
                continue;
            };
            if closing_start <= opening_hash_count {
                continue;
            }

            let between = &trimmed[opening_hash_count..closing_start];
            let left_missing = !matches!(between.chars().next(), Some(' ' | '\t'));
            let right_missing = !matches!(between.chars().next_back(), Some(' ' | '\t'));

            // If the opening separator is already present, a content-adjacent
            // trailing hash is ordinary heading text (`### C#`), not evidence
            // of a closing sequence. This intentionally avoids markdownlint's
            // ambiguous C#/F# false positive.
            let unambiguous_closed_candidate = !right_missing || left_missing;
            if !(unambiguous_closed_candidate && (left_missing || right_missing)) {
                continue;
            }

            let content = between.trim_matches([' ', '\t']);
            if content.is_empty() {
                continue;
            }

            let indent = &line[..line.len() - trimmed.len()];
            let opening_hashes = &trimmed[..opening_hash_count];
            let closing_hashes = &trimmed[closing_start..closing_end];
            let fixed_line = format!("{indent}{opening_hashes} {content} {closing_hashes}\n");

            let fix = Fix {
                description: "Add missing spaces inside closed ATX heading".to_string(),
                replacement: Some(fixed_line),
                start: Position {
                    line: line_num,
                    column: 1,
                },
                end: Position {
                    line: line_num,
                    column: line.chars().count() + 1,
                },
            };

            violations.push(self.create_violation_with_fix(
                "Missing space inside hashes on closed ATX heading".to_string(),
                line_num,
                1,
                Severity::Warning,
                fix,
            ));
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::rule::Rule;
    use std::path::PathBuf;

    fn check(content: &str) -> Vec<Violation> {
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        MD020.check(&document).unwrap()
    }

    #[test]
    fn accepts_valid_closed_headings() {
        let violations = check(
            "# Heading #\n## Another heading ##\n### Tab separated\t###\n#### Asymmetric #\n",
        );
        assert!(violations.is_empty());
    }

    #[test]
    fn content_adjacent_hashes_are_heading_text() {
        let violations = check("### C#\n### F#\n### Rust##\n### C\\#\n### Heading###\n");
        assert!(violations.is_empty());
    }

    #[test]
    fn reports_missing_spaces_on_both_sides() {
        let violations = check("##Heading##\n");
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].fix.as_ref().unwrap().replacement,
            Some("## Heading ##\n".to_string())
        );
    }

    #[test]
    fn reports_missing_opening_space_when_closing_sequence_is_clear() {
        let violations = check("###Heading ###\n");
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].fix.as_ref().unwrap().replacement,
            Some("### Heading ###\n".to_string())
        );
    }

    #[test]
    fn preserves_indentation_in_fix() {
        let violations = check("  ##Heading ##\n");
        assert_eq!(
            violations[0].fix.as_ref().unwrap().replacement,
            Some("  ## Heading ##\n".to_string())
        );
    }

    #[test]
    fn allows_multiple_spaces_for_md021() {
        let violations = check("#  Heading  #\n##\tHeading\t##\n");
        assert!(violations.is_empty());
    }

    #[test]
    fn ignores_non_candidates() {
        let violations = check(
            "# Open heading\nRegular text #\n#!/bin/bash\n####### Too many hashes #######\n###\n",
        );
        assert!(violations.is_empty());
    }

    #[test]
    fn handles_trailing_whitespace() {
        let violations = check("##Heading ##  \n");
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].fix.as_ref().unwrap().replacement,
            Some("## Heading ##\n".to_string())
        );
    }

    #[test]
    fn fix_range_uses_character_columns() {
        let violations = check("##Résumé ##\n");
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.end.column, "##Résumé ##".chars().count() + 1);
    }
}
