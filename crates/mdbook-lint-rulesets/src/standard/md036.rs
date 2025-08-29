use mdbook_lint_core::Document;
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};

/// MD036 - Emphasis used instead of a heading
pub struct MD036 {
    /// Punctuation characters that prevent treating emphasis as heading
    pub punctuation: String,
}

impl MD036 {
    pub fn new() -> Self {
        Self {
            punctuation: ".,;:!?。，；：！？".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn with_punctuation(mut self, punctuation: &str) -> Self {
        self.punctuation = punctuation.to_string();
        self
    }

    /// Create MD036 from configuration
    pub fn from_config(config: &toml::Value) -> Self {
        let mut rule = Self::new();

        if let Some(punctuation) = config.get("punctuation").and_then(|v| v.as_str()) {
            rule.punctuation = punctuation.to_string();
        }

        rule
    }

    fn is_emphasis_as_heading(&self, line: &str) -> bool {
        let trimmed = line.trim();

        // Must be a single line paragraph
        if trimmed.is_empty() {
            return false;
        }

        // Check for bold emphasis (**text** or __text__)
        let is_bold = (trimmed.starts_with("**") && trimmed.ends_with("**") && trimmed.len() > 4)
            || (trimmed.starts_with("__") && trimmed.ends_with("__") && trimmed.len() > 4);

        // Check for italic emphasis (*text* or _text_)
        let is_italic = (trimmed.starts_with('*')
            && trimmed.ends_with('*')
            && trimmed.len() > 2
            && !trimmed.starts_with("**"))
            || (trimmed.starts_with('_')
                && trimmed.ends_with('_')
                && trimmed.len() > 2
                && !trimmed.starts_with("__"));

        if !is_bold && !is_italic {
            return false;
        }

        // Extract the inner text
        let inner_text = if is_bold {
            &trimmed[2..trimmed.len() - 2]
        } else {
            &trimmed[1..trimmed.len() - 1]
        };

        // Must not end with punctuation
        if let Some(last_char) = inner_text.chars().last()
            && self.punctuation.contains(last_char)
        {
            return false;
        }

        // Must not be empty after removing emphasis markers
        if inner_text.trim().is_empty() {
            return false;
        }

        // Must not contain line breaks (already handled by single line check)
        // Must be the entire content of the line (already handled by starts_with/ends_with)

        true
    }

    fn is_paragraph_context(&self, lines: &[&str], line_index: usize) -> bool {
        // Check if this line is surrounded by blank lines (paragraph context)
        let has_blank_before = line_index == 0 || lines[line_index - 1].trim().is_empty();
        let has_blank_after =
            line_index == lines.len() - 1 || lines[line_index + 1].trim().is_empty();

        has_blank_before && has_blank_after
    }
}

impl Default for MD036 {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for MD036 {
    fn id(&self) -> &'static str {
        "MD036"
    }

    fn name(&self) -> &'static str {
        "no-emphasis-as-heading"
    }

    fn description(&self) -> &'static str {
        "Emphasis used instead of a heading"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Structure)
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let lines: Vec<&str> = document.content.lines().collect();

        for (line_index, line) in lines.iter().enumerate() {
            let line_number = line_index + 1;

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Check if this line uses emphasis as a heading
            if self.is_emphasis_as_heading(line) && self.is_paragraph_context(&lines, line_index) {
                violations.push(self.create_violation(
                    "Emphasis used instead of a heading".to_string(),
                    line_number,
                    1,
                    Severity::Warning,
                ));
            }
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::Document;
    use std::path::PathBuf;

    #[test]
    fn test_md036_no_violations() {
        let content = r#"# Proper heading

Some normal text with **bold** and *italic* within the paragraph.

## Another heading

Regular paragraph with emphasis.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD036::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md036_bold_as_heading() {
        let content = r#"Some text

**My document**

Lorem ipsum dolor sit amet...
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD036::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3);
        assert!(
            violations[0]
                .message
                .contains("Emphasis used instead of a heading")
        );
    }

    #[test]
    fn test_md036_italic_as_heading() {
        let content = r#"Some text

_Another section_

Consectetur adipiscing elit, sed do eiusmod.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD036::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3);
        assert!(
            violations[0]
                .message
                .contains("Emphasis used instead of a heading")
        );
    }

    #[test]
    fn test_md036_underscore_bold_as_heading() {
        let content = r#"Introduction

__Important Section__

This is the content of the section.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD036::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3);
    }

    #[test]
    fn test_md036_with_punctuation_allowed() {
        let content = r#"Some text

**Section with period.**

More content here.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD036::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0); // No violation because of punctuation
    }

    #[test]
    fn test_md036_custom_punctuation() {
        let content = r#"Some text

**Section with period.**

More content here.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD036::new().with_punctuation("!?"); // Allow periods
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1); // Now triggers because period is allowed
        assert_eq!(violations[0].line, 3);
    }

    #[test]
    fn test_md036_inline_emphasis_ignored() {
        let content = r#"This is a paragraph with **bold text** in the middle and *italic text* as well.

Another paragraph with normal content.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD036::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md036_no_surrounding_blank_lines() {
        let content = r#"Some text
**Not a heading because no blank line above**
More text
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD036::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md036_multiple_violations() {
        let content = r#"Introduction

**First Section**

Some content here.

_Second Section_

More content here.

__Third Section__

Final content.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD036::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 7);
        assert_eq!(violations[2].line, 11);
    }

    #[test]
    fn test_md036_empty_emphasis() {
        let content = r#"Some text

****

**  **

More text.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD036::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0); // Empty emphasis should not trigger
    }

    #[test]
    fn test_md036_mixed_punctuation() {
        let content = r#"Some text

**Question?**

**Exclamation!**

**Normal heading**

More content.
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD036::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1); // Only the one without punctuation
        assert_eq!(violations[0].line, 7);
    }
}
