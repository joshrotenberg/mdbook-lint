//! MD026: Trailing punctuation in headings
//!
//! This rule checks that headings do not end with punctuation characters.

use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Fix, Position, Severity, Violation},
};

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

    /// Create MD026 from configuration
    pub fn from_config(config: &toml::Value) -> Self {
        let mut rule = Self::new();

        if let Some(punctuation) = config.get("punctuation").and_then(|v| v.as_str()) {
            rule.punctuation = punctuation.to_string();
        }

        rule
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

    fn can_fix(&self) -> bool {
        true
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
                    // Create fix by removing trailing punctuation
                    let line_content = &document.lines[line - 1];
                    let heading_without_punct = heading_text.trim_end_matches(|c| self.is_punctuation(c));
                    
                    let fixed_line = if line_content.trim_start().starts_with('#') {
                        // ATX heading
                        let trimmed = line_content.trim_start();
                        let hashes_end = trimmed.find(|c: char| c != '#').unwrap_or(trimmed.len());
                        let hashes = &trimmed[..hashes_end];
                        // Check if it's a closed ATX heading (ends with hashes)
                        let content_after_hashes = &trimmed[hashes_end..];
                        if content_after_hashes.trim_end().ends_with('#') {
                            // Closed ATX - preserve closing hashes
                            let closing_hashes_start = content_after_hashes.rfind(|c: char| c != '#' && !c.is_whitespace()).map(|i| i + 1).unwrap_or(0);
                            format!("{} {} {}\n", hashes, heading_without_punct, content_after_hashes[closing_hashes_start..].trim())
                        } else {
                            format!("{} {}\n", hashes, heading_without_punct)
                        }
                    } else {
                        // Setext heading - just replace the text
                        format!("{}\n", heading_without_punct)
                    };
                    
                    let fix = Fix {
                        description: format!("Remove trailing punctuation '{}'", last_char),
                        replacement: Some(fixed_line),
                        start: Position { line, column: 1 },
                        end: Position { line, column: line_content.len() + 1 },
                    };
                    
                    violations.push(self.create_violation_with_fix(
                        format!(
                            "Heading should not end with punctuation '{last_char}': {heading_text}"
                        ),
                        line,
                        column,
                        Severity::Warning,
                        fix,
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
    use mdbook_lint_core::Document;
    use mdbook_lint_core::rule::Rule;
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

    #[test]
    fn test_md026_fix_trailing_punctuation() {
        let content = "# Heading with exclamation!\n## Another with period.";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD026::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        
        // First heading - remove !
        assert!(violations[0].fix.is_some());
        let fix1 = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix1.description, "Remove trailing punctuation '!'");
        assert_eq!(fix1.replacement, Some("# Heading with exclamation\n".to_string()));
        
        // Second heading - remove .
        assert!(violations[1].fix.is_some());
        let fix2 = violations[1].fix.as_ref().unwrap();
        assert_eq!(fix2.description, "Remove trailing punctuation '.'");
        assert_eq!(fix2.replacement, Some("## Another with period\n".to_string()));
    }

    #[test]
    fn test_md026_fix_multiple_punctuation() {
        let content = "# Heading with multiple...";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD026::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());
        
        let fix = violations[0].fix.as_ref().unwrap();
        // Should remove all trailing punctuation
        assert_eq!(fix.replacement, Some("# Heading with multiple\n".to_string()));
    }

    #[test]
    fn test_md026_can_fix() {
        let rule = MD026::new();
        assert!(mdbook_lint_core::AstRule::can_fix(&rule));
    }
}
