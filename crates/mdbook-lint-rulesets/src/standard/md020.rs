//! MD020: No space inside hashes on closed ATX heading
//!
//! This rule checks for spaces inside hash characters on closed ATX style headings.
//! Closed ATX headings should not have spaces between the content and the closing hashes.

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Fix, Position, Severity, Violation},
};

/// Rule to check for spaces inside hashes on closed ATX style headings
pub struct MD020;

impl Rule for MD020 {
    fn id(&self) -> &'static str {
        "MD020"
    }

    fn name(&self) -> &'static str {
        "no-space-inside-atx"
    }

    fn description(&self) -> &'static str {
        "No space inside hashes on closed atx style heading"
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
                // Check if this is a closed ATX heading (ends with #)
                if trimmed.ends_with('#') {
                    let opening_hash_count = trimmed.chars().take_while(|&c| c == '#').count();
                    let closing_hash_count =
                        trimmed.chars().rev().take_while(|&c| c == '#').count();

                    // Extract the content between opening and closing hashes
                    if trimmed.len() > opening_hash_count + closing_hash_count {
                        let content_with_spaces =
                            &trimmed[opening_hash_count..trimmed.len() - closing_hash_count];

                        // Check for whitespace at the beginning or end of the content
                        if content_with_spaces.starts_with(|c: char| c.is_whitespace())
                            || content_with_spaces.ends_with(|c: char| c.is_whitespace())
                        {
                            // Create fixed line by removing spaces inside hashes
                            let indent = &line[..line.len() - trimmed.len()];
                            let opening_hashes = &trimmed[..opening_hash_count];
                            let closing_hashes = &trimmed[trimmed.len() - closing_hash_count..];
                            let content = content_with_spaces.trim();
                            let fixed_line = format!(
                                "{}{}{}{}\n",
                                indent, opening_hashes, content, closing_hashes
                            );

                            let fix = Fix {
                                description: "Remove spaces inside hashes on closed ATX heading"
                                    .to_string(),
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
                                "Whitespace found inside hashes on closed ATX heading".to_string(),
                                line_num,
                                1,
                                Severity::Warning,
                                fix,
                            ));
                        }
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
    use mdbook_lint_core::Document;
    use mdbook_lint_core::rule::Rule;
    use std::path::PathBuf;

    #[test]
    fn test_md020_no_violations() {
        let content = r#"# Open ATX heading (not checked)

## Another open heading

#No spaces inside#

##No spaces here either##

###Content without spaces###

####Multiple words but no spaces####

#####Another valid closed heading#####

######Level 6 valid######

Regular paragraph text.

Not a heading: # this has text before it #

Shebang line should be ignored:
#!/bin/bash
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD020;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md020_space_at_beginning() {
        let content = r#"# Open heading is fine

## Space at beginning of closed heading ##

### Another violation ###

Regular text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD020;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 5);
        assert!(
            violations[0]
                .message
                .contains("Whitespace found inside hashes")
        );
        assert!(
            violations[1]
                .message
                .contains("Whitespace found inside hashes")
        );
    }

    #[test]
    fn test_md020_space_at_end() {
        let content = r#"# Open heading is fine

##Content with space at end ##

###Another space at end ###

Regular text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD020;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 5);
    }

    #[test]
    fn test_md020_spaces_both_sides() {
        let content = r#"# Open heading is fine

## Spaces on both sides ##

### More spaces on both sides ###

####  Even more spaces  ####

Regular text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD020;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 5);
        assert_eq!(violations[2].line, 7);
    }

    #[test]
    fn test_md020_mixed_valid_invalid() {
        let content = r#"#Valid closed heading#

## Invalid with spaces ##

###Another valid###

#### Another invalid ####

#####Valid again#####

###### Final invalid ######
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD020;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 7);
        assert_eq!(violations[2].line, 11);
    }

    #[test]
    fn test_md020_asymmetric_hashes() {
        let content = r#"# Open heading with one hash

##Content##

###More content####

####Even more#####

Regular text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD020;
        let violations = rule.check(&document).unwrap();

        // Should detect closed headings regardless of hash count symmetry
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md020_empty_closed_heading() {
        let content = r#"# Valid open heading

##Empty closed##

###Another empty###

####Content####

Regular text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD020;
        let violations = rule.check(&document).unwrap();

        // Empty closed headings should not trigger violations (no spaces to check)
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md020_indented_headings() {
        let content = r#"# Valid open heading

    ## Indented with spaces ##

Regular text.

  ### Another indented ###
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD020;
        let violations = rule.check(&document).unwrap();

        // Should detect spaces in indented closed headings
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 7);
    }

    #[test]
    fn test_md020_only_closing_hash() {
        let content = r#"# Valid open heading

This is not a heading #

##This is valid##

Regular text ending with hash #
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD020;
        let violations = rule.check(&document).unwrap();

        // Should only check actual headings (lines starting with #)
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md020_all_heading_levels() {
        let content = r#"# Content with spaces #
## Content with spaces ##
### Content with spaces ###
#### Content with spaces ####
##### Content with spaces #####
###### Content with spaces ######
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD020;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 6);
        for (i, violation) in violations.iter().enumerate() {
            assert_eq!(violation.line, i + 1);
            assert!(violation.message.contains("Whitespace found inside hashes"));
        }
    }

    #[test]
    fn test_md020_tabs_inside_hashes() {
        let content = "#\tContent with tab\t#\n\n##\tAnother tab\t##\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD020;
        let violations = rule.check(&document).unwrap();

        // Should detect tabs as whitespace inside hashes
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_md020_fix_space_at_beginning() {
        let content = "## Space at beginning ##\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD020;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.description,
            "Remove spaces inside hashes on closed ATX heading"
        );
        assert_eq!(
            fix.replacement,
            Some("##Space at beginning##\n".to_string())
        );
        assert_eq!(fix.start.line, 1);
        assert_eq!(fix.start.column, 1);
    }

    #[test]
    fn test_md020_fix_space_at_end() {
        let content = "##Content with space at end ##\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD020;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement,
            Some("##Content with space at end##\n".to_string())
        );
    }

    #[test]
    fn test_md020_fix_spaces_both_sides() {
        let content = "## Spaces on both sides ##\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD020;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement,
            Some("##Spaces on both sides##\n".to_string())
        );
    }

    #[test]
    fn test_md020_fix_multiple_spaces() {
        let content = "###   Multiple spaces here   ###\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD020;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement,
            Some("###Multiple spaces here###\n".to_string())
        );
    }

    #[test]
    fn test_md020_fix_tabs() {
        let content = "#\tContent with tab\t#\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD020;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, Some("#Content with tab#\n".to_string()));
    }

    #[test]
    fn test_md020_fix_preserves_indentation() {
        let content = "    ## Indented with spaces ##\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD020;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement,
            Some("    ##Indented with spaces##\n".to_string())
        );
    }

    #[test]
    fn test_md020_fix_all_levels() {
        let content = r#"# Heading 1 #
## Heading 2 ##
### Heading 3 ###
#### Heading 4 ####
##### Heading 5 #####
###### Heading 6 ######"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD020;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 6);

        // Check fixes for each level
        assert_eq!(
            violations[0].fix.as_ref().unwrap().replacement,
            Some("#Heading 1#\n".to_string())
        );
        assert_eq!(
            violations[1].fix.as_ref().unwrap().replacement,
            Some("##Heading 2##\n".to_string())
        );
        assert_eq!(
            violations[2].fix.as_ref().unwrap().replacement,
            Some("###Heading 3###\n".to_string())
        );
        assert_eq!(
            violations[3].fix.as_ref().unwrap().replacement,
            Some("####Heading 4####\n".to_string())
        );
        assert_eq!(
            violations[4].fix.as_ref().unwrap().replacement,
            Some("#####Heading 5#####\n".to_string())
        );
        assert_eq!(
            violations[5].fix.as_ref().unwrap().replacement,
            Some("######Heading 6######\n".to_string())
        );
    }

    #[test]
    fn test_md020_fix_asymmetric_hashes() {
        let content = "### Content ####\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD020;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        // Should preserve the asymmetric hash count but remove spaces
        assert_eq!(fix.replacement, Some("###Content####\n".to_string()));
    }
}
