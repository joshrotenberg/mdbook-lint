//! MD027: Multiple spaces after blockquote symbol
//!
//! This rule checks for multiple spaces after the '>' symbol in blockquotes.
//! Only one space should be used after the blockquote symbol.

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Fix, Position, Severity, Violation},
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
                    // Create fixed line with single space after blockquote symbol
                    let mut fixed_line = String::new();
                    fixed_line.push_str(&line[..actual_pos + 1]); // Include up to and including '>'

                    // Add single space if there's content after, otherwise keep empty
                    let content_after = after_blockquote.trim_start();
                    if !content_after.is_empty() {
                        fixed_line.push(' ');
                        fixed_line.push_str(content_after);
                    }
                    fixed_line.push('\n');

                    let fix = Fix {
                        description: format!(
                            "Replace {} spaces with 1 space after blockquote symbol",
                            leading_whitespace_count
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
                        format!("Multiple spaces after blockquote symbol: found {leading_whitespace_count} whitespace characters, expected 1"),
                        line_num,
                        actual_pos + 2, // Position after the '>'
                        Severity::Warning,
                        fix,
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
    use mdbook_lint_core::Document;
    use mdbook_lint_core::rule::Rule;
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

    #[test]
    fn test_md027_fix_two_spaces() {
        let content = ">  Two spaces after blockquote\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());
        
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Replace 2 spaces with 1 space after blockquote symbol");
        assert_eq!(fix.replacement, Some("> Two spaces after blockquote\n".to_string()));
        assert_eq!(fix.start.line, 1);
        assert_eq!(fix.start.column, 1);
    }

    #[test]
    fn test_md027_fix_many_spaces() {
        let content = ">     Five spaces after blockquote\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());
        
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Replace 5 spaces with 1 space after blockquote symbol");
        assert_eq!(fix.replacement, Some("> Five spaces after blockquote\n".to_string()));
    }

    #[test]
    fn test_md027_fix_tabs() {
        let content = ">\t\tTwo tabs after blockquote\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());
        
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Replace 2 spaces with 1 space after blockquote symbol");
        assert_eq!(fix.replacement, Some("> Two tabs after blockquote\n".to_string()));
    }

    #[test]
    fn test_md027_fix_nested_blockquotes() {
        let content = "> >  Extra space in nested blockquote\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());
        
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, Some("> > Extra space in nested blockquote\n".to_string()));
    }

    #[test]
    fn test_md027_fix_deeply_nested() {
        let content = "> > >  Extra spaces in triple nested\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());
        
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, Some("> > > Extra spaces in triple nested\n".to_string()));
    }

    #[test]
    fn test_md027_fix_multiple_violations_in_line() {
        let content = ">  First level > >  nested with extra spaces\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        // Should detect both instances of multiple spaces
        assert_eq!(violations.len(), 2);
        
        // First violation (after first >)
        assert!(violations[0].fix.is_some());
        let fix = violations[0].fix.as_ref().unwrap();
        // The fix should fix all violations in the line
        assert_eq!(fix.replacement, Some("> First level > > nested with extra spaces\n".to_string()));
    }

    #[test]
    fn test_md027_fix_empty_blockquote() {
        let content = ">  \n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());
        
        let fix = violations[0].fix.as_ref().unwrap();
        // Empty blockquote should remove all spaces after >
        assert_eq!(fix.replacement, Some(">\n".to_string()));
    }

    #[test]
    fn test_md027_fix_preserves_content() {
        let content = ">   Some important content here\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());
        
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, Some("> Some important content here\n".to_string()));
    }

    #[test]
    fn test_md027_fix_mixed_whitespace() {
        let content = "> \t Mixed space and tab\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());
        
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, Some("> Mixed space and tab\n".to_string()));
    }
}
