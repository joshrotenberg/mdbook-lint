//! MD021: Multiple spaces inside hashes on closed ATX heading
//!
//! This rule checks for multiple spaces inside hash characters on closed ATX style headings.
//! Only one space should be used between the content and the closing hashes.

use crate::error::Result;
use crate::rule::{Rule, RuleCategory, RuleMetadata};
use crate::{
    Document,
    violation::{Severity, Violation},
};

/// Rule to check for multiple spaces inside hashes on closed ATX style headings
pub struct MD021;

impl Rule for MD021 {
    fn id(&self) -> &'static str {
        "MD021"
    }

    fn name(&self) -> &'static str {
        "no-multiple-space-closed-atx"
    }

    fn description(&self) -> &'static str {
        "Multiple spaces inside hashes on closed atx style heading"
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

                        // Check for multiple whitespace at the beginning
                        let leading_whitespace_count = content_with_spaces
                            .chars()
                            .take_while(|c| c.is_whitespace())
                            .count();
                        if leading_whitespace_count > 1 {
                            violations.push(self.create_violation(
                                format!("Multiple spaces after opening hashes in closed ATX heading: found {leading_whitespace_count} whitespace characters, expected 1"),
                                line_num,
                                opening_hash_count + 1,
                                Severity::Warning,
                            ));
                        }

                        // Check for multiple whitespace at the end
                        let trailing_whitespace_count = content_with_spaces
                            .chars()
                            .rev()
                            .take_while(|c| c.is_whitespace())
                            .count();
                        if trailing_whitespace_count > 1 {
                            violations.push(self.create_violation(
                                format!("Multiple spaces before closing hashes in closed ATX heading: found {trailing_whitespace_count} whitespace characters, expected 1"),
                                line_num,
                                trimmed.len() - closing_hash_count - trailing_whitespace_count + 1,
                                Severity::Warning,
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
    use crate::Document;
    use crate::rule::Rule;
    use std::path::PathBuf;

    #[test]
    fn test_md021_no_violations() {
        let content = r#"# Open ATX heading (not checked)

## Another open heading

# Single space inside #

## Single space here ##

### Valid closed heading ###

#### Multiple words single space ####

##### Another valid closed heading #####

###### Level 6 valid ######

Regular paragraph text.

Not a heading: # this has text before it #

Also not a heading:
# this is indented #

Shebang line should be ignored:
#!/bin/bash
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD021;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md021_multiple_spaces_at_beginning() {
        let content = r#"# Open heading is fine

##  Two spaces after opening ##

###   Three spaces after opening ###

####    Four spaces after opening ####

Regular text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD021;
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
    fn test_md021_multiple_spaces_at_end() {
        let content = r#"# Open heading is fine

## Content with two spaces  ##

### Content with three spaces   ###

#### Content with four spaces    ####

Regular text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD021;
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
    fn test_md021_multiple_spaces_both_sides() {
        let content = r#"# Open heading is fine

##  Two spaces both sides  ##

###   Three spaces both sides   ###

####    Four spaces both sides    ####

Regular text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD021;
        let violations = rule.check(&document).unwrap();

        // Should detect violations on both sides
        assert_eq!(violations.len(), 6);
        // Each heading should generate 2 violations (beginning and end)
        assert_eq!(violations[0].line, 3); // Two spaces after opening
        assert_eq!(violations[1].line, 3); // Two spaces before closing
        assert_eq!(violations[2].line, 5); // Three spaces after opening
        assert_eq!(violations[3].line, 5); // Three spaces before closing
        assert_eq!(violations[4].line, 7); // Four spaces after opening
        assert_eq!(violations[5].line, 7); // Four spaces before closing
    }

    #[test]
    fn test_md021_mixed_valid_invalid() {
        let content = r#"# Valid closed heading #

##  Invalid: two spaces after ##

### Valid closed heading ###

####  Invalid: two spaces both sides  ####

##### Valid closed heading #####

######   Invalid: three spaces after ######
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD021;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 4);
        assert_eq!(violations[0].line, 3); // Two spaces after opening
        assert_eq!(violations[1].line, 7); // Two spaces after opening
        assert_eq!(violations[2].line, 7); // Two spaces before closing
        assert_eq!(violations[3].line, 11); // Three spaces after opening
    }

    #[test]
    fn test_md021_tabs_and_mixed_whitespace() {
        let content = "#\t\tTwo tabs after opening##\n\n##Content with tab at end\t\t##\n\n###\t Content tab space mix \t###\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD021;
        let violations = rule.check(&document).unwrap();

        // Should detect multiple whitespace characters (spaces and tabs)
        assert_eq!(violations.len(), 4);
    }

    #[test]
    fn test_md021_empty_closed_heading() {
        let content = r#"# Valid open heading

## ##

### ###

#### ####

Regular text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD021;
        let violations = rule.check(&document).unwrap();

        // Empty closed headings with single space should be valid
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md021_no_space_inside() {
        let content = r#"# Valid open heading

##No space inside##

###Content###

####Text####

Regular text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD021;
        let violations = rule.check(&document).unwrap();

        // No spaces inside is handled by MD020, not this rule
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md021_indented_headings() {
        let content = r#"# Valid open heading

    ##  Indented with multiple spaces  ##

Regular text.

  ###   Another indented with multiple spaces   ###
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD021;
        let violations = rule.check(&document).unwrap();

        // Should detect multiple spaces in indented closed headings
        assert_eq!(violations.len(), 4);
        assert_eq!(violations[0].line, 3); // Two spaces after opening
        assert_eq!(violations[1].line, 3); // Two spaces before closing
        assert_eq!(violations[2].line, 7); // Three spaces after opening
        assert_eq!(violations[3].line, 7); // Three spaces before closing
    }

    #[test]
    fn test_md021_asymmetric_hashes() {
        let content = r#"# Open heading with one hash

##  Content with multiple spaces  ####

###   More content   #####

####    Even more    ######

Regular text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD021;
        let violations = rule.check(&document).unwrap();

        // Should detect multiple spaces regardless of hash count symmetry
        assert_eq!(violations.len(), 6);
    }

    #[test]
    fn test_md021_all_heading_levels() {
        let content = r#"#  Content with multiple spaces  #
##  Content with multiple spaces  ##
###  Content with multiple spaces  ###
####  Content with multiple spaces  ####
#####  Content with multiple spaces  #####
######  Content with multiple spaces  ######
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD021;
        let violations = rule.check(&document).unwrap();

        // Each heading should generate 2 violations (beginning and end)
        assert_eq!(violations.len(), 12);
        for (i, violation) in violations.iter().enumerate() {
            let line_num = (i / 2) + 1; // Two violations per line
            assert_eq!(violation.line, line_num);
            assert!(
                violation
                    .message
                    .contains("found 2 whitespace characters, expected 1")
            );
        }
    }

    #[test]
    fn test_md021_single_space_valid() {
        let content = r#"# Content with single space #
## Content with single space ##
### Content with single space ###
#### Content with single space ####
##### Content with single space #####
###### Content with single space ######
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD021;
        let violations = rule.check(&document).unwrap();

        // Single spaces should be valid
        assert_eq!(violations.len(), 0);
    }
}
