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

    fn can_fix(&self) -> bool {
        true
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let mut in_fenced_code_block = false;

        for (line_number, line) in document.lines.iter().enumerate() {
            let line_num = line_number + 1; // Convert to 1-based line numbers

            // Only check lines that are blockquotes (start with optional whitespace then >)
            // This avoids false positives on table rows with | characters or > in content
            let trimmed = line.trim_start();
            if !trimmed.starts_with('>') {
                // Check for fenced code block markers outside blockquotes
                if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                    in_fenced_code_block = !in_fenced_code_block;
                }
                continue;
            }

            // Get content after blockquote marker(s) for code fence detection
            let blockquote_content = trimmed.trim_start_matches('>').trim_start_matches(' ');
            let blockquote_content_trimmed = blockquote_content.trim_start();

            // Check for fenced code block markers inside blockquotes
            if blockquote_content_trimmed.starts_with("```")
                || blockquote_content_trimmed.starts_with("~~~")
            {
                in_fenced_code_block = !in_fenced_code_block;
            }

            // Skip checking lines inside fenced code blocks - indentation is for code formatting
            if in_fenced_code_block {
                continue;
            }

            // Find blockquote markers at the start of the line
            // A blockquote line can have: optional leading spaces, then one or more "> " patterns
            let mut pos = 0;
            let chars: Vec<char> = line.chars().collect();

            // Skip leading whitespace
            while pos < chars.len() && chars[pos].is_whitespace() {
                pos += 1;
            }

            // Now we should be at blockquote markers
            while pos < chars.len() && chars[pos] == '>' {
                let gt_pos = pos;
                pos += 1; // Move past the '>'

                // Count whitespace after this '>'
                let ws_start = pos;
                let mut whitespace_count = 0;
                let mut has_tab = false;

                while pos < chars.len() && chars[pos].is_whitespace() {
                    if chars[pos] == '\t' {
                        has_tab = true;
                    }
                    whitespace_count += 1;
                    pos += 1;
                }

                // Check if next char is another '>' (nested blockquote) or content
                let is_nested = pos < chars.len() && chars[pos] == '>';

                // Flag if there are 2+ spaces OR any tabs (since tabs count as multiple spaces)
                // This includes lines with only whitespace after > (empty blockquote with extra spaces)
                if whitespace_count >= 2 || has_tab {
                    // Create fixed line with single space after blockquote symbol
                    let before: String = chars[..gt_pos + 1].iter().collect();
                    let after: String = chars[ws_start + whitespace_count..].iter().collect();

                    let mut fixed_line = before;
                    if !after.trim().is_empty() || is_nested {
                        fixed_line.push(' ');
                    }
                    fixed_line.push_str(&after);
                    if !fixed_line.ends_with('\n') {
                        fixed_line.push('\n');
                    }

                    let fix = Fix {
                        description: format!(
                            "Replace {} spaces with 1 space after blockquote symbol",
                            whitespace_count
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
                        format!("Multiple spaces after blockquote symbol: found {whitespace_count} whitespace characters, expected 1"),
                        line_num,
                        gt_pos + 2, // Position after the '>'
                        Severity::Warning,
                        fix,
                    ));
                }

                // If not a nested blockquote marker, we're done with this line's blockquote prefix
                if !is_nested {
                    break;
                }
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
        assert_eq!(
            fix.description,
            "Replace 2 spaces with 1 space after blockquote symbol"
        );
        assert_eq!(
            fix.replacement,
            Some("> Two spaces after blockquote\n".to_string())
        );
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
        assert_eq!(
            fix.description,
            "Replace 5 spaces with 1 space after blockquote symbol"
        );
        assert_eq!(
            fix.replacement,
            Some("> Five spaces after blockquote\n".to_string())
        );
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
        assert_eq!(
            fix.description,
            "Replace 2 spaces with 1 space after blockquote symbol"
        );
        assert_eq!(
            fix.replacement,
            Some("> Two tabs after blockquote\n".to_string())
        );
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
        assert_eq!(
            fix.replacement,
            Some("> > Extra space in nested blockquote\n".to_string())
        );
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
        assert_eq!(
            fix.replacement,
            Some("> > > Extra spaces in triple nested\n".to_string())
        );
    }

    #[test]
    fn test_md027_fix_multiple_violations_in_line() {
        // This tests a line with `> >` in the content (not at start)
        // Only the leading blockquote marker should be checked
        let content = ">  First level > >  nested with extra spaces\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        // Should only detect the leading blockquote marker violation
        // The `> >` in the middle is content, not a blockquote marker
        assert_eq!(violations.len(), 1);

        assert!(violations[0].fix.is_some());
        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement,
            Some("> First level > >  nested with extra spaces\n".to_string())
        );
    }

    #[test]
    fn test_md027_fix_actual_nested_blockquotes() {
        // Proper nested blockquote: markers at the start of the line
        let content = "> >  Nested blockquote with extra space\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        // Should detect the extra space after the second >
        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement,
            Some("> > Nested blockquote with extra space\n".to_string())
        );
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
        assert_eq!(
            fix.replacement,
            Some("> Some important content here\n".to_string())
        );
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

    #[test]
    fn test_md027_no_false_positive_on_tables() {
        // Table rows with | should not be flagged as blockquotes
        let content = r#"| Column 1       | Column 2                             | Column 3 |
|----------------|--------------------------------------|----------|
| <code>&vert;</code>       | <code>pat &vert; pat</code>                             | Pattern alternatives |
| `a > b`        | Greater than comparison              | Operators |
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        // Tables should not trigger MD027 - they don't start with >
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md027_no_false_positive_on_inline_content() {
        // > characters in the middle of content should not be flagged
        let content = r#"This line has a > character in the middle.
The expression `a > b` is a comparison.
Use `->` for return types in Rust.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md027_blockquote_with_code_containing_gt() {
        // Blockquote lines with > in code spans should only check the blockquote marker
        let content = r#"> The expression `a > b` returns true if a is greater.
>  Extra space here is a violation though.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        // Only the second line should be flagged (extra space after >)
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 2);
    }

    #[test]
    fn test_md027_no_false_positive_in_fenced_code_blocks() {
        // Code blocks inside blockquotes should not trigger MD027
        // The extra spaces are code indentation, not blockquote formatting
        let content = r#"> Here is some code:
>
> ```rust
>     fn main() {
>         println!("Hello");
>     }
> ```
>
> After the code block.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        // No violations - the indented lines are inside a code block
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md027_detects_violations_outside_code_blocks() {
        // Violations before/after code blocks should still be detected
        let content = r#">  Before code block (violation)
> ```rust
>     indented code (no violation)
> ```
>  After code block (violation)
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD027;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 1);
        assert_eq!(violations[1].line, 5);
    }
}
