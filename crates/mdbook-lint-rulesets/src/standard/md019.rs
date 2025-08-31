//! MD019: Multiple spaces after hash on ATX heading
//!
//! This rule checks for multiple spaces after hash characters in ATX style headings.
//! Only one space should be used after the hash characters.

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Fix, Position, Severity, Violation},
};

/// Rule to check for multiple spaces after hash on ATX style headings
pub struct MD019;

impl Rule for MD019 {
    fn id(&self) -> &'static str {
        "MD019"
    }

    fn name(&self) -> &'static str {
        "no-multiple-space-atx"
    }

    fn description(&self) -> &'static str {
        "Multiple spaces after hash on atx style heading"
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

            // Check if this is an ATX-style heading (starts with #)
            // Skip shebang lines (#!/...)
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') && !trimmed.starts_with("#!") {
                // Find where the heading content starts
                let hash_count = trimmed.chars().take_while(|&c| c == '#').count();

                // Check if there's content after the hashes
                if trimmed.len() > hash_count {
                    let after_hashes = &trimmed[hash_count..];

                    // Check for multiple whitespace characters after the hashes
                    if after_hashes.starts_with("  ")
                        || after_hashes.starts_with("\t")
                        || (after_hashes.starts_with(" ")
                            && after_hashes.chars().nth(1) == Some('\t'))
                    {
                        let whitespace_count = after_hashes
                            .chars()
                            .take_while(|&c| c.is_whitespace())
                            .count();

                        // Create fixed line with single space after hashes
                        let indent = &line[..line.len() - trimmed.len()];
                        let hashes = &trimmed[..hash_count];
                        let content = after_hashes.trim_start();
                        let fixed_line = format!("{}{} {}\n", indent, hashes, content);

                        let fix = Fix {
                            description: format!(
                                "Replace {} spaces with 1 space after hash",
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
                            format!("Multiple spaces after hash on ATX heading: found {whitespace_count} whitespace characters, expected 1"),
                            line_num,
                            hash_count + 1, // Position after the last hash
                            Severity::Warning,
                            fix,
                        ));
                    }
                } else if trimmed.len() == hash_count {
                    // Handle empty headings like "##" - they should have no space after
                    continue;
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
    fn test_md019_no_violations() {
        let content = r#"# Single space heading

## Another single space

### Level 3 with single space

#### Level 4 heading

##### Level 5

###### Level 6

Regular paragraph text.

Not a heading: # this has text before it

Also not a heading:
# this is indented

Shebang line should be ignored:
#!/bin/bash
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD019;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md019_multiple_spaces_violation() {
        let content = r#"# Single space is fine

##  Two spaces after hash

###   Three spaces after hash

####    Four spaces after hash

Regular text here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD019;
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
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 5);
        assert_eq!(violations[2].line, 7);
    }

    #[test]
    fn test_md019_mixed_valid_invalid() {
        let content = r#"# Valid heading

##  Invalid: two spaces

### Valid heading

####  Invalid: two spaces again

##### Valid heading

######   Invalid: three spaces
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD019;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 7);
        assert_eq!(violations[2].line, 11);
    }

    #[test]
    fn test_md019_no_space_after_hash() {
        let content = r#"# Valid heading

##No space after hash (different rule)

### Valid heading

####Multiple spaces after hash

Regular text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD019;
        let violations = rule.check(&document).unwrap();

        // Should only detect the multiple spaces, not the missing space
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md019_tabs_and_mixed_whitespace() {
        let content = "# Valid heading\n\n##\t\tTwo tabs after hash\n\n###  \tSpace then tab\n\n#### \t Space tab space\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD019;
        let violations = rule.check(&document).unwrap();

        // Should detect multiple whitespace characters (spaces and tabs)
        assert_eq!(violations.len(), 3);
        assert!(violations[0].message.contains("whitespace characters"));
        assert!(violations[1].message.contains("whitespace characters"));
        assert!(violations[2].message.contains("whitespace characters"));
    }

    #[test]
    fn test_md019_heading_with_no_content() {
        let content = r#"# Valid heading

##

###

####

Text here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD019;
        let violations = rule.check(&document).unwrap();

        // Empty headings (## with no content) should not be flagged by this rule
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md019_shebang_and_hash_comments() {
        let content = r#"#!/bin/bash

# Valid heading

##  Invalid heading

# This is a comment in some contexts but valid markdown heading

Regular text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD019;
        let violations = rule.check(&document).unwrap();

        // Should ignore shebang but detect the invalid heading
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 5);
    }

    #[test]
    fn test_md019_indented_headings() {
        let content = r#"# Valid heading

    ##  Indented heading with multiple spaces

Regular text.

  ###   Another indented heading
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD019;
        let violations = rule.check(&document).unwrap();

        // Should detect multiple spaces even in indented headings
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 7);
    }

    #[test]
    fn test_md019_all_heading_levels() {
        let content = r#"#  Level 1 with multiple spaces
##  Level 2 with multiple spaces
###  Level 3 with multiple spaces
####  Level 4 with multiple spaces
#####  Level 5 with multiple spaces
######  Level 6 with multiple spaces
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD019;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 6);
        for (i, violation) in violations.iter().enumerate() {
            assert_eq!(violation.line, i + 1);
            assert!(
                violation
                    .message
                    .contains("found 2 whitespace characters, expected 1")
            );
        }
    }

    #[test]
    fn test_md019_fix_multiple_spaces() {
        let content = "##  Two spaces after hash\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD019;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Replace 2 spaces with 1 space after hash");
        assert_eq!(
            fix.replacement,
            Some("## Two spaces after hash\n".to_string())
        );
        assert_eq!(fix.start.line, 1);
        assert_eq!(fix.start.column, 1);
    }

    #[test]
    fn test_md019_fix_many_spaces() {
        let content = "#     Five spaces after hash\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD019;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Replace 5 spaces with 1 space after hash");
        assert_eq!(
            fix.replacement,
            Some("# Five spaces after hash\n".to_string())
        );
    }

    #[test]
    fn test_md019_fix_tabs() {
        let content = "##\t\tTwo tabs after hash\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD019;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Replace 2 spaces with 1 space after hash");
        assert_eq!(
            fix.replacement,
            Some("## Two tabs after hash\n".to_string())
        );
    }

    #[test]
    fn test_md019_fix_mixed_whitespace() {
        let content = "### \t Space then tab\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD019;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        // Space + tab + space = 3 whitespace characters
        assert_eq!(fix.description, "Replace 3 spaces with 1 space after hash");
        assert_eq!(fix.replacement, Some("### Space then tab\n".to_string()));
    }

    #[test]
    fn test_md019_fix_preserves_indentation() {
        let content = "    ##  Indented heading with multiple spaces\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD019;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement,
            Some("    ## Indented heading with multiple spaces\n".to_string())
        );
    }

    #[test]
    fn test_md019_fix_all_levels() {
        let content = r#"#  Level 1
##   Level 2
###    Level 3
####     Level 4
#####      Level 5
######       Level 6"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD019;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 6);

        // Check fixes for each level
        assert_eq!(
            violations[0].fix.as_ref().unwrap().replacement,
            Some("# Level 1\n".to_string())
        );
        assert_eq!(
            violations[1].fix.as_ref().unwrap().replacement,
            Some("## Level 2\n".to_string())
        );
        assert_eq!(
            violations[2].fix.as_ref().unwrap().replacement,
            Some("### Level 3\n".to_string())
        );
        assert_eq!(
            violations[3].fix.as_ref().unwrap().replacement,
            Some("#### Level 4\n".to_string())
        );
        assert_eq!(
            violations[4].fix.as_ref().unwrap().replacement,
            Some("##### Level 5\n".to_string())
        );
        assert_eq!(
            violations[5].fix.as_ref().unwrap().replacement,
            Some("###### Level 6\n".to_string())
        );
    }
}
