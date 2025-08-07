//! MD026: Trailing punctuation in headings
//!
//! This rule checks that headings do not end with punctuation characters.

use crate::error::Result;
use crate::rule::{AstRule, RuleCategory, RuleMetadata};
use crate::{
    Document,
    violation::{Severity, Violation},
};
use comrak::nodes::{AstNode, NodeValue};

/// Rule to check that headings do not end with punctuation
pub struct MD026 {
    /// Punctuation characters to check for (default: ".,;:!?")
    punctuation: String,
}

impl MD026 {
    /// Create a new MD026 rule with default settings
    pub fn new() -> Self {
        Self {
            punctuation: ".,;:!?".to_string(),
        }
    }

    /// Create a new MD026 rule with custom punctuation characters
    #[allow(dead_code)]
    pub fn with_punctuation(punctuation: String) -> Self {
        Self { punctuation }
    }

    /// Check if a character is considered punctuation for this rule
    fn is_punctuation(&self, ch: char) -> bool {
        self.punctuation.contains(ch)
    }
}

impl Default for MD026 {
    fn default() -> Self {
        Self::new()
    }
}

impl AstRule for MD026 {
    fn id(&self) -> &'static str {
        "MD026"
    }

    fn name(&self) -> &'static str {
        "no-trailing-punctuation"
    }

    fn description(&self) -> &'static str {
        "Trailing punctuation in heading"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Formatting).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_ast<'a>(&self, document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // Find all heading nodes
        for node in ast.descendants() {
            if let NodeValue::Heading(_heading) = &node.data.borrow().value
                && let Some((line, column)) = document.node_position(node)
            {
                let heading_text = document.node_text(node);
                let heading_text = heading_text.trim();

                // Skip empty headings
                if heading_text.is_empty() {
                    continue;
                }

                // Check if heading ends with punctuation
                if let Some(last_char) = heading_text.chars().last()
                    && self.is_punctuation(last_char)
                {
                    violations.push(self.create_violation(
                        format!(
                            "Heading should not end with punctuation '{last_char}': {heading_text}"
                        ),
                        line,
                        column,
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
    fn test_md026_no_punctuation() {
        let content = r#"# Valid heading
## Another valid heading
### Third level heading
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD026::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md026_period_violation() {
        let content = r#"# Heading with period.
Some content here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD026::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("should not end with punctuation '.'")
        );
        assert!(violations[0].message.contains("Heading with period."));
        assert_eq!(violations[0].line, 1);
    }

    #[test]
    fn test_md026_multiple_punctuation_types() {
        let content = r#"# Heading with period.
## Heading with comma,
### Heading with semicolon;
#### Heading with colon:
##### Heading with exclamation!
###### Heading with question?
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD026::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 6);

        // Check each punctuation type
        assert!(
            violations[0]
                .message
                .contains("should not end with punctuation '.'")
        );
        assert!(
            violations[1]
                .message
                .contains("should not end with punctuation ','")
        );
        assert!(
            violations[2]
                .message
                .contains("should not end with punctuation ';'")
        );
        assert!(
            violations[3]
                .message
                .contains("should not end with punctuation ':'")
        );
        assert!(
            violations[4]
                .message
                .contains("should not end with punctuation '!'")
        );
        assert!(
            violations[5]
                .message
                .contains("should not end with punctuation '?'")
        );
    }

    #[test]
    fn test_md026_custom_punctuation() {
        let content = r#"# Heading with period.
## Heading with custom @
### Heading with allowed!
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD026::with_punctuation(".@".to_string());
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(
            violations[0]
                .message
                .contains("should not end with punctuation '.'")
        );
        assert!(
            violations[1]
                .message
                .contains("should not end with punctuation '@'")
        );
    }

    #[test]
    fn test_md026_setext_headings() {
        let content = r#"Setext heading with period.
===========================

Another setext with question?
-----------------------------
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD026::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(
            violations[0]
                .message
                .contains("should not end with punctuation '.'")
        );
        assert!(
            violations[1]
                .message
                .contains("should not end with punctuation '?'")
        );
    }

    #[test]
    fn test_md026_empty_headings_ignored() {
        let content = r#"#

##

###
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD026::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md026_punctuation_in_middle() {
        let content = r#"# Heading with punctuation, but not at end
## Question? No, this is fine at end!
### Period. In middle is ok
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD026::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("should not end with punctuation '!'")
        );
        assert_eq!(violations[0].line, 2);
    }

    #[test]
    fn test_md026_whitespace_after_punctuation() {
        let content = r#"# Heading with period.
## Heading with spaces after punctuation.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD026::new();
        let violations = rule.check(&document).unwrap();

        // Should detect punctuation even with trailing whitespace
        assert_eq!(violations.len(), 2);
        assert!(
            violations[0]
                .message
                .contains("should not end with punctuation '.'")
        );
        assert!(
            violations[1]
                .message
                .contains("should not end with punctuation '.'")
        );
    }

    #[test]
    fn test_md026_closed_atx_headings() {
        let content = r#"# Closed ATX heading. #
## Another closed heading! ##
### Valid closed heading ###
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD026::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(
            violations[0]
                .message
                .contains("should not end with punctuation '.'")
        );
        assert!(
            violations[1]
                .message
                .contains("should not end with punctuation '!'")
        );
    }

    #[test]
    fn test_md026_headings_in_code_blocks() {
        let content = r#"Some text here.

```markdown
# This heading has punctuation.
## This one too!
```

# But this real heading also has punctuation.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD026::new();
        let violations = rule.check(&document).unwrap();

        // Should only detect the real heading, not the ones in code blocks
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("should not end with punctuation '.'")
        );
        assert_eq!(violations[0].line, 8);
    }
}
