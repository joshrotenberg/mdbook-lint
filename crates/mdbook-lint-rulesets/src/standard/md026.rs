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
    /// Punctuation characters to check for (default: ".,;:!")
    /// Note: Question marks are excluded by default as question-style headings are common
    punctuation: String,
}

impl MD026 {
    /// Create a new MD026 rule with default settings
    pub fn new() -> Self {
        Self {
            // Exclude ? by default - question headings like "What Is Ownership?" are common
            punctuation: ".,;:!".to_string(),
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

    /// Check if heading ends with a Rust macro (identifier followed by !)
    /// Examples: panic!, assert!, format!, vec!
    /// We use a conservative approach: only recognize common Rust macros
    fn ends_with_rust_macro(&self, text: &str) -> bool {
        if !text.ends_with('!') {
            return false;
        }

        // Get the word before the !
        let without_bang = &text[..text.len() - 1];
        let last_word = without_bang.split_whitespace().next_back().unwrap_or("");

        if last_word.is_empty() {
            return false;
        }

        // Common Rust macros that are frequently used in documentation
        const KNOWN_MACROS: &[&str] = &[
            "panic",
            "assert",
            "assert_eq",
            "assert_ne",
            "debug_assert",
            "debug_assert_eq",
            "debug_assert_ne",
            "format",
            "print",
            "println",
            "eprint",
            "eprintln",
            "write",
            "writeln",
            "vec",
            "todo",
            "unimplemented",
            "unreachable",
            "matches",
            "cfg",
            "env",
            "include",
            "include_str",
            "include_bytes",
            "concat",
            "stringify",
            "macro_rules",
            "dbg",
            "column",
            "file",
            "line",
            "module_path",
            "option_env",
        ];

        // Check if it's a known macro
        KNOWN_MACROS.contains(&last_word)
    }

    /// Check if heading ends with Rust range operator (..)
    fn ends_with_range_operator(&self, text: &str) -> bool {
        text.ends_with("..")
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
                    // Skip Rust macros (e.g., panic!, assert!, format!)
                    if last_char == '!' && self.ends_with_rust_macro(heading_text) {
                        continue;
                    }

                    // Skip Rust range operator (..)
                    if last_char == '.' && self.ends_with_range_operator(heading_text) {
                        continue;
                    }
                    // Create fix by removing trailing punctuation
                    let line_content = &document.lines[line - 1];
                    let heading_without_punct =
                        heading_text.trim_end_matches(|c| self.is_punctuation(c));

                    let fixed_line = if line_content.trim_start().starts_with('#') {
                        // ATX heading
                        let trimmed = line_content.trim_start();
                        let hashes_end = trimmed.find(|c: char| c != '#').unwrap_or(trimmed.len());
                        let hashes = &trimmed[..hashes_end];
                        // Check if it's a closed ATX heading (ends with hashes)
                        let content_after_hashes = &trimmed[hashes_end..];
                        if content_after_hashes.trim_end().ends_with('#') {
                            // Closed ATX - preserve closing hashes
                            let closing_hashes_start = content_after_hashes
                                .rfind(|c: char| c != '#' && !c.is_whitespace())
                                .map(|i| i + 1)
                                .unwrap_or(0);
                            format!(
                                "{} {} {}\n",
                                hashes,
                                heading_without_punct,
                                content_after_hashes[closing_hashes_start..].trim()
                            )
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
                        end: Position {
                            line,
                            column: line_content.len() + 1,
                        },
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

        // Question marks are excluded by default, so only 5 violations
        assert_eq!(violations.len(), 5);

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
        // Question mark is no longer flagged by default
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

Another setext with exclamation!
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
                .contains("should not end with punctuation '!'")
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
        assert_eq!(
            fix1.replacement,
            Some("# Heading with exclamation\n".to_string())
        );

        // Second heading - remove .
        assert!(violations[1].fix.is_some());
        let fix2 = violations[1].fix.as_ref().unwrap();
        assert_eq!(fix2.description, "Remove trailing punctuation '.'");
        assert_eq!(
            fix2.replacement,
            Some("## Another with period\n".to_string())
        );
    }

    #[test]
    fn test_md026_fix_multiple_punctuation() {
        // Use colons instead of periods since .. is now recognized as range operator
        let content = "# Heading with multiple:::";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD026::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        // Should remove all trailing punctuation
        assert_eq!(
            fix.replacement,
            Some("# Heading with multiple\n".to_string())
        );
    }

    #[test]
    fn test_md026_can_fix() {
        let rule = MD026::new();
        assert!(mdbook_lint_core::AstRule::can_fix(&rule));
    }

    #[test]
    fn test_md026_question_marks_allowed_by_default() {
        let content = r#"# What Is Ownership?
## Where's the -> Operator?
### Why Not An Enum?
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD026::new();
        let violations = rule.check(&document).unwrap();

        // Question marks should not be flagged by default
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md026_question_marks_configurable() {
        let content = r#"# What Is Ownership?
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        // Explicitly include ? in punctuation
        let rule = MD026::with_punctuation(".,;:!?".to_string());
        let violations = rule.check(&document).unwrap();

        // Question mark should be flagged when explicitly configured
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("'?'"));
    }

    #[test]
    fn test_md026_rust_macros_not_flagged() {
        let content = r#"# Unrecoverable Errors with panic!
## Checking Results with assert!
### Testing Equality with assert_eq! and assert_ne!
#### Concatenating with + or format!
##### Using the vec! macro
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD026::new();
        let violations = rule.check(&document).unwrap();

        // Rust macros should not be flagged
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md026_regular_exclamation_still_flagged() {
        let content = r#"# Hello, World!
## Hello, Cargo!
### This is exciting!
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD026::new();
        let violations = rule.check(&document).unwrap();

        // Regular exclamation marks (not Rust macros) should still be flagged
        assert_eq!(violations.len(), 3);
        assert!(violations[0].message.contains("Hello, World!"));
        assert!(violations[1].message.contains("Hello, Cargo!"));
        assert!(violations[2].message.contains("This is exciting!"));
    }

    #[test]
    fn test_md026_range_operator_not_flagged() {
        let content = r#"# Remaining Parts of a Value with ..
## Using the range operator ..
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD026::new();
        let violations = rule.check(&document).unwrap();

        // Range operator (..) should not be flagged
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md026_single_period_still_flagged() {
        let content = r#"# This heading ends with a period.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD026::new();
        let violations = rule.check(&document).unwrap();

        // Single period should still be flagged
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("'.'"));
    }
}
