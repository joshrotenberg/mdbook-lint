//! MD027: Multiple spaces after blockquote symbol
//!
//! This rule checks for multiple spaces after the '>' symbol in blockquotes.
//! Only one space should be used after the blockquote symbol.

use crate::error::Result;
use crate::rule::{Rule, RuleCategory, RuleMetadata};
use crate::{
    Document,
    violation::{Severity, Violation},
};

/// Rule to check for multiple spaces after blockquote symbol
pub struct MD027;

impl Rule for MD027 {
    fn id(&self) -> &'static str {
        "MD027"
    }

    fn name(&self) -> &'static str {
        "no-multiple-space-blockquote"
    }

    fn description(&self) -> &'static str {
        "Multiple spaces after blockquote symbol"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Formatting).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        for (line_number, line) in document.lines.iter().enumerate() {
            let line_num = line_number + 1; // Convert to 1-based line numbers

            // Look for all '>' characters in the line
            let mut pos = 0;
            while let Some(gt_pos) = line[pos..].find('>') {
                let actual_pos = pos + gt_pos;

                // Check what comes after the '>'
                let after_blockquote = &line[actual_pos + 1..];

                // Check for multiple whitespace characters after '>'
                let leading_whitespace_count = after_blockquote
                    .chars()
                    .take_while(|&c| c.is_whitespace())
                    .count();

                // Flag if there are 2+ spaces OR any tabs (since tabs count as multiple spaces)
                let has_tab = after_blockquote
                    .chars()
                    .take_while(|&c| c.is_whitespace())
                    .any(|c| c == '\t');

                if leading_whitespace_count >= 2 || has_tab {
                    violations.push(self.create_violation(
                        format!("Multiple spaces after blockquote symbol: found {leading_whitespace_count} whitespace characters, expected 1"),
                        line_num,
                        actual_pos + 2, // Position after the '>'
                        Severity::Warning,
                    ));
                }

                // Move past this '>' to look for more
                pos = actual_pos + 1;
            }
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use crate::rule::Rule;
    use std::path::PathBuf;

    #[test]
    fn test_md027_no_violations() {
        let content = r#"> Single space after blockquote
> Another line with single space
>
> Empty blockquote line is fine

Regular text here.

> Multi-line blockquote
> with single spaces
> throughout

Nested blockquotes:
> Level 1
> > Level 2 with single space
> > > Level 3 with single space
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md027_multiple_spaces_violation() {
        let content = r#"> Single space is fine
>  Two spaces after blockquote
>   Three spaces after blockquote
>    Four spaces after blockquote

Regular text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);
        assert!(
            violations[0]
                .message
                .contains("found 2 whitespace characters, expected 1")
        );
        assert!(
            violations[1]
                .message
                .contains("found 3 whitespace characters, expected 1")
        );
        assert!(
            violations[2]
                .message
                .contains("found 4 whitespace characters, expected 1")
        );
        assert_eq!(violations[0].line, 2);
        assert_eq!(violations[1].line, 3);
        assert_eq!(violations[2].line, 4);
    }

    #[test]
    fn test_md027_nested_blockquotes() {
        let content = r#"> Level 1 with single space
> > Level 2 with single space
> >  Level 2 with multiple spaces
> > > Level 3 with single space
> > >  Level 3 with multiple spaces

More content.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 5);
    }

    #[test]
    fn test_md027_indented_blockquotes() {
        let content = r#"Regular text.

    > Indented blockquote with single space
    >  Indented blockquote with multiple spaces
    >   Another with even more spaces

Back to regular text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 4);
        assert_eq!(violations[1].line, 5);
    }

    #[test]
    fn test_md027_mixed_valid_invalid() {
        let content = r#"> Valid blockquote
>  Invalid: two spaces
> Another valid line
>   Invalid: three spaces
> Valid again

Regular paragraph.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 2);
        assert_eq!(violations[1].line, 4);
    }

    #[test]
    fn test_md027_no_space_after_gt() {
        let content = r#"> Valid with space
>No space after gt
>Still no space

Some text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        // No space after > is a different rule (not this one)
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md027_tabs_and_mixed_whitespace() {
        let content =
            ">\tTab after blockquote\n>\t\tTwo tabs after blockquote\n> \tSpace then tab\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        // Should detect multiple whitespace characters (all 3 cases have tabs or multiple spaces)
        assert_eq!(violations.len(), 3);
    }

    #[test]
    fn test_md027_empty_blockquote() {
        let content = r#"> Valid content
>
>
>

More content.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        // Empty blockquote lines (just >) should not be flagged - no spaces to check
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md027_complex_nesting() {
        let content = r#"> Level 1
> > Level 2
> >  Level 2 with extra spaces
> > > Level 3
> > >  Level 3 with extra spaces
> Back to level 1
>  Level 1 with extra spaces

Regular text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 5);
        assert_eq!(violations[2].line, 7);
    }
}
