//! MD028: Blank line inside blockquote
//!
//! This rule checks for blank lines inside blockquotes that break the blockquote flow.
//! Blank lines should not appear inside blockquotes without proper continuation.

use crate::error::Result;
use crate::rule::{Rule, RuleCategory, RuleMetadata};
use crate::{
    Document,
    violation::{Severity, Violation},
};

/// Rule to check for blank lines inside blockquotes
pub struct MD028;

impl Rule for MD028 {
    fn id(&self) -> &'static str {
        "MD028"
    }

    fn name(&self) -> &'static str {
        "no-blanks-blockquote"
    }

    fn description(&self) -> &'static str {
        "Blank line inside blockquote"
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

            // Check if this is a blank line
            if line.trim().is_empty() {
                // Look backwards to find the last non-blank line
                let mut prev_is_blockquote = false;
                for i in (0..line_num - 1).rev() {
                    if let Some(prev_line) = document.lines.get(i)
                        && !prev_line.trim().is_empty()
                    {
                        prev_is_blockquote = prev_line.trim_start().starts_with('>');
                        break;
                    }
                }

                // Look forwards to find the next non-blank line
                let mut next_is_blockquote = false;
                for i in line_num..document.lines.len() {
                    if let Some(next_line) = document.lines.get(i)
                        && !next_line.trim().is_empty()
                    {
                        next_is_blockquote = next_line.trim_start().starts_with('>');
                        break;
                    }
                }

                // If we have blockquote lines before and after this blank line,
                // then this blank line breaks the blockquote
                if prev_is_blockquote && next_is_blockquote {
                    violations.push(self.create_violation(
                        "Blank line inside blockquote".to_string(),
                        line_num,
                        1,
                        Severity::Warning,
                    ));
                }
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
    fn test_md028_no_violations() {
        let content = r#"> This is a valid blockquote
> with multiple lines
> all properly formatted

Regular paragraph here.

> Another blockquote
> also properly formatted
>
> with empty blockquote line

More regular text.

> Single line blockquote

Final paragraph.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD028;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md028_blank_line_violation() {
        let content = r#"> This is a blockquote
> with proper formatting

> but then it continues
> after a blank line

Regular text here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD028;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3); // The blank line
        assert!(
            violations[0]
                .message
                .contains("Blank line inside blockquote")
        );
    }

    #[test]
    fn test_md028_multiple_blank_lines() {
        let content = r#"> Start of blockquote
> with some content

> continues after blank line


> continues after multiple blank lines
> and ends here

Regular text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD028;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].line, 3); // First blank line
        assert_eq!(violations[1].line, 5); // Second blank line
        assert_eq!(violations[2].line, 6); // Third blank line
    }

    #[test]
    fn test_md028_proper_blockquote_separation() {
        let content = r#"> First blockquote
> ends here

Regular paragraph in between.

> Second blockquote
> starts here

More regular text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD028;
        let violations = rule.check(&document).unwrap();

        // No violations because the blockquotes are properly separated
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md028_nested_blockquotes() {
        let content = r#"> Outer blockquote
> > Inner blockquote
> > continues here

> > but this breaks the flow
> back to outer level

Text here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD028;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 4); // The blank line between nested levels
    }

    #[test]
    fn test_md028_blockquote_at_end() {
        let content = r#"> Blockquote at the end
> of the document

> continues here"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD028;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3);
    }

    #[test]
    fn test_md028_empty_blockquote_lines() {
        let content = r#"> Blockquote with empty lines
>
> is perfectly valid
>
> because empty lines have >

Regular text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD028;
        let violations = rule.check(&document).unwrap();

        // Empty lines with '>' markers are valid
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md028_indented_blockquotes() {
        let content = r#"Regular text.

    > Indented blockquote
    > continues here

    > but breaks here
    > and continues

More text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD028;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 5);
    }

    #[test]
    fn test_md028_complex_document() {
        let content = r#"# Heading

> Valid blockquote
> with multiple lines

Regular paragraph.

> Another blockquote

> that continues improperly

> and has more content

## Another heading

> Final blockquote
> that ends properly

The end.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD028;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 9); // First improper break
        assert_eq!(violations[1].line, 11); // Second improper break
    }
}
