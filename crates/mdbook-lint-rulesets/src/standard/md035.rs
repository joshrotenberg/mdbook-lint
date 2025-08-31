use mdbook_lint_core::Document;
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Fix, Position, Severity, Violation};

/// MD035 - Horizontal rule style
pub struct MD035 {
    /// Horizontal rule style: "consistent", "---", "***", "___", etc.
    pub style: String,
}

impl MD035 {
    pub fn new() -> Self {
        Self {
            style: "consistent".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = style.to_string();
        self
    }

    /// Create MD035 from configuration
    pub fn from_config(config: &toml::Value) -> Self {
        let mut rule = Self::new();

        if let Some(style) = config.get("style").and_then(|v| v.as_str()) {
            rule.style = style.to_string();
        }

        rule
    }

    fn is_horizontal_rule(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();

        // Must be at least 3 characters
        if trimmed.len() < 3 {
            return None;
        }

        // Check for various horizontal rule patterns
        if self.is_hr_pattern(trimmed, '-') {
            Some(self.normalize_hr_style(trimmed, '-'))
        } else if self.is_hr_pattern(trimmed, '*') {
            Some(self.normalize_hr_style(trimmed, '*'))
        } else if self.is_hr_pattern(trimmed, '_') {
            Some(self.normalize_hr_style(trimmed, '_'))
        } else {
            None
        }
    }

    fn is_hr_pattern(&self, line: &str, char: char) -> bool {
        let mut char_count = 0;
        let mut has_other = false;

        for c in line.chars() {
            if c == char {
                char_count += 1;
            } else if c == ' ' || c == '\t' {
                // Spaces and tabs are allowed
                continue;
            } else {
                has_other = true;
                break;
            }
        }

        char_count >= 3 && !has_other
    }

    fn normalize_hr_style(&self, line: &str, char: char) -> String {
        // Count the character and determine if there are spaces
        let char_count = line.chars().filter(|&c| c == char).count();
        let has_spaces = line.contains(' ') || line.contains('\t');

        if has_spaces {
            // Return the style with spaces (e.g., "* * *")
            let chars: Vec<String> = std::iter::repeat_n(char.to_string(), char_count).collect();
            chars.join(" ")
        } else {
            // Return the style without spaces (e.g., "***")
            std::iter::repeat_n(char, char_count).collect()
        }
    }

    fn get_canonical_style(&self, style: &str) -> String {
        // Normalize common variations to canonical forms
        let first_char = style.chars().next().unwrap_or('-');
        let has_spaces = style.contains(' ');
        let _char_count = style.chars().filter(|&c| c == first_char).count();

        if has_spaces {
            match first_char {
                '-' => "- - -".to_string(),
                '*' => "* * *".to_string(),
                '_' => "_ _ _".to_string(),
                _ => style.to_string(),
            }
        } else {
            match first_char {
                '-' => "---".to_string(),
                '*' => "***".to_string(),
                '_' => "___".to_string(),
                _ => style.to_string(),
            }
        }
    }
}

impl Default for MD035 {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for MD035 {
    fn id(&self) -> &'static str {
        "MD035"
    }

    fn name(&self) -> &'static str {
        "hr-style"
    }

    fn description(&self) -> &'static str {
        "Horizontal rule style"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Formatting)
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
        let lines = document.content.lines();
        let mut horizontal_rules = Vec::new();

        // First pass: collect all horizontal rules
        for (line_number, line) in lines.enumerate() {
            let line_number = line_number + 1;

            if let Some(hr_style) = self.is_horizontal_rule(line) {
                horizontal_rules.push((line_number, hr_style));
            }
        }

        // If no horizontal rules found, no violations
        if horizontal_rules.is_empty() {
            return Ok(violations);
        }

        // Determine expected style
        let expected = if self.style == "consistent" {
            // Use the style of the first horizontal rule
            self.get_canonical_style(&horizontal_rules[0].1)
        } else {
            // Use the configured style
            self.style.clone()
        };

        // Second pass: check for violations
        for (line_number, hr_style) in horizontal_rules {
            let canonical_style = self.get_canonical_style(&hr_style);

            if canonical_style != expected {
                // Create fix by replacing with expected style
                let line_content = &document.lines[line_number - 1];
                let leading_whitespace = line_content.len() - line_content.trim_start().len();
                let indent = &line_content[..leading_whitespace];

                let fix = Fix {
                    description: format!("Change horizontal rule style to '{}'", expected),
                    replacement: Some(format!("{}{}\n", indent, expected)),
                    start: Position {
                        line: line_number,
                        column: 1,
                    },
                    end: Position {
                        line: line_number,
                        column: line_content.len() + 1,
                    },
                };

                violations.push(self.create_violation_with_fix(
                    format!(
                        "Horizontal rule style mismatch: Expected '{expected}', found '{canonical_style}'"
                    ),
                    line_number,
                    1,
                    Severity::Warning,
                    fix,
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
    fn test_md035_consistent_style() {
        let content = r#"# Heading

---

Some content

---

More content

---
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD035::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md035_inconsistent_style() {
        let content = r#"# Heading

---

Some content

***

More content

___
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD035::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 7);
        assert_eq!(violations[1].line, 11);
        assert!(
            violations[0]
                .message
                .contains("Expected '---', found '***'")
        );
        assert!(
            violations[1]
                .message
                .contains("Expected '---', found '___'")
        );
    }

    #[test]
    fn test_md035_spaced_style_consistent() {
        let content = r#"# Heading

* * *

Some content

* * * * *

More content

- - -
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD035::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 11);
        assert!(
            violations[0]
                .message
                .contains("Expected '* * *', found '- - -'")
        );
    }

    #[test]
    fn test_md035_specific_style() {
        let content = r#"# Heading

---

Some content

***

More content
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD035::new().with_style("***");
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3);
        assert!(
            violations[0]
                .message
                .contains("Expected '***', found '---'")
        );
    }

    #[test]
    fn test_md035_various_lengths() {
        let content = r#"---

-----

---------
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD035::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0); // All use dashes, so consistent
    }

    #[test]
    fn test_md035_mixed_spacing() {
        let content = r#"---

- - -

-- --
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD035::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert!(
            violations[0]
                .message
                .contains("Expected '---', found '- - -'")
        );
        assert!(
            violations[1]
                .message
                .contains("Expected '---', found '- - -'")
        ); // Normalized
    }

    #[test]
    fn test_md035_not_horizontal_rules() {
        let content = r#"# Heading

Some text with -- dashes

* List item
* Another item

-- Not enough dashes

Code with ---
    ---

> Block quote with
> ---
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD035::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md035_minimum_length() {
        let content = r#"--

---

----
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD035::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0); // First line is too short, so not an HR
    }

    #[test]
    fn test_md035_with_spaces_around() {
        let content = r#"   ---

  ***

    ___
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD035::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert!(
            violations[0]
                .message
                .contains("Expected '---', found '***'")
        );
        assert!(
            violations[1]
                .message
                .contains("Expected '---', found '___'")
        );
    }

    #[test]
    fn test_md035_underscore_style() {
        let content = r#"___

___

***
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD035::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 5);
        assert!(
            violations[0]
                .message
                .contains("Expected '___', found '***'")
        );
    }

    #[test]
    fn test_md035_no_horizontal_rules() {
        let content = r#"# Heading

Some content

## Another heading

More content
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD035::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md035_fix_inconsistent_style() {
        let content = r#"# Heading

---

Content

***

More content
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD035::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Change horizontal rule style to '---'");
        assert_eq!(fix.replacement, Some("---\n".to_string()));
    }

    #[test]
    fn test_md035_fix_to_configured_style() {
        let content = r#"# Heading

---

Content

___
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD035::new().with_style("***");
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);

        // Both should be fixed to ***
        for violation in &violations {
            assert!(violation.fix.is_some());
            let fix = violation.fix.as_ref().unwrap();
            assert_eq!(fix.description, "Change horizontal rule style to '***'");
            assert_eq!(fix.replacement, Some("***\n".to_string()));
        }
    }

    #[test]
    fn test_md035_can_fix() {
        let rule = MD035::new();
        assert!(Rule::can_fix(&rule));
    }
}
