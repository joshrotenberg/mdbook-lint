//! MDBOOK010: Missing or invalid preprocessor configuration
//!
//! This rule checks for invalid preprocessor directives in mdBook files.
//! Preprocessors like mermaid, katex, and others require specific syntax.

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};
use regex::Regex;

/// Rule to check for invalid preprocessor configuration
pub struct MDBOOK010;

impl Rule for MDBOOK010 {
    fn id(&self) -> &'static str {
        "MDBOOK010"
    }

    fn name(&self) -> &'static str {
        "preprocessor-validation"
    }

    fn description(&self) -> &'static str {
        "Missing or invalid preprocessor configuration"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::experimental(RuleCategory::MdBook).introduced_in("mdbook-lint v0.11.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // Check for common preprocessor patterns
        self.check_mermaid_blocks(document, &mut violations);
        self.check_katex_blocks(document, &mut violations);
        self.check_admonish_blocks(document, &mut violations);

        Ok(violations)
    }
}

impl MDBOOK010 {
    /// Check for invalid mermaid blocks
    fn check_mermaid_blocks(&self, document: &Document, violations: &mut Vec<Violation>) {
        let _mermaid_re = Regex::new(r"```mermaid\s*\n([\s\S]*?)```").unwrap();

        for (line_num, line) in document.lines.iter().enumerate() {
            if line.trim() == "```mermaid" {
                // Check if the block is empty
                if line_num + 1 < document.lines.len() {
                    let next_line = &document.lines[line_num + 1];
                    if next_line.trim() == "```" {
                        violations.push(self.create_violation(
                            "Empty mermaid block detected".to_string(),
                            line_num + 1,
                            1,
                            Severity::Warning,
                        ));
                    }
                }
            }

            // Check for common mermaid syntax errors
            if line.contains("```mermaid") && !line.trim().eq("```mermaid") {
                violations.push(self.create_violation(
                    "Mermaid blocks should start with '```mermaid' on its own line".to_string(),
                    line_num + 1,
                    1,
                    Severity::Error,
                ));
            }
        }
    }

    /// Check for invalid KaTeX blocks
    fn check_katex_blocks(&self, document: &Document, violations: &mut Vec<Violation>) {
        // Check for inline math
        let _inline_math_re = Regex::new(r"\$([^$\n]+)\$").unwrap();
        // Check for display math
        let _display_math_re = Regex::new(r"\$\$([^$]+)\$\$").unwrap();

        for (line_num, line) in document.lines.iter().enumerate() {
            // Check for unclosed inline math
            let dollar_count = line.chars().filter(|&c| c == '$').count();
            if dollar_count % 2 != 0 && !line.contains("$$") {
                violations.push(self.create_violation(
                    "Unclosed inline math block (odd number of $ signs)".to_string(),
                    line_num + 1,
                    1,
                    Severity::Error,
                ));
            }

            // Check for empty math blocks
            if line.contains("$$$$") {
                violations.push(self.create_violation(
                    "Empty display math block detected".to_string(),
                    line_num + 1,
                    line.find("$$$$").unwrap() + 1,
                    Severity::Warning,
                ));
            }

            if line.contains("$ $") {
                violations.push(self.create_violation(
                    "Empty inline math block detected".to_string(),
                    line_num + 1,
                    line.find("$ $").unwrap() + 1,
                    Severity::Warning,
                ));
            }
        }
    }

    /// Check for invalid admonish blocks
    fn check_admonish_blocks(&self, document: &Document, violations: &mut Vec<Violation>) {
        let admonish_re = Regex::new(r"```admonish\s+(\w+)(.*)").unwrap();
        let valid_types = [
            "note",
            "tip",
            "info",
            "warning",
            "danger",
            "important",
            "caution",
            "bug",
            "example",
            "quote",
        ];

        for (line_num, line) in document.lines.iter().enumerate() {
            if line.starts_with("```admonish") {
                if let Some(captures) = admonish_re.captures(line) {
                    if let Some(admonish_type) = captures.get(1) {
                        let type_str = admonish_type.as_str();
                        if !valid_types.contains(&type_str) {
                            violations.push(self.create_violation(
                                format!(
                                    "Invalid admonish type '{}'. Valid types are: {}",
                                    type_str,
                                    valid_types.join(", ")
                                ),
                                line_num + 1,
                                admonish_type.start() + 1,
                                Severity::Error,
                            ));
                        }
                    }
                } else if line.trim() == "```admonish" {
                    violations.push(self.create_violation(
                        "Admonish block missing type. Use format: ```admonish <type>".to_string(),
                        line_num + 1,
                        1,
                        Severity::Error,
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::Document;
    use std::path::PathBuf;

    #[test]
    fn test_valid_preprocessors() {
        let content = r#"# Chapter

Here's a mermaid diagram:

```mermaid
graph TD
    A --> B
```

Some math: $x = y^2$

Display math:
$$
E = mc^2
$$

```admonish note
This is a note.
```
"#;
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK010;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_empty_mermaid_block() {
        let content = r#"```mermaid
```"#;
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK010;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Empty mermaid block"));
    }

    #[test]
    fn test_invalid_mermaid_syntax() {
        let content = "```mermaid with extra text";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK010;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("should start with '```mermaid' on its own line")
        );
    }

    #[test]
    fn test_unclosed_inline_math() {
        let content = "This is $unclosed math";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK010;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Unclosed inline math"));
    }

    #[test]
    fn test_empty_math_blocks() {
        let content = r#"Empty inline: $ $
Empty display: $$$$"#;
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK010;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("Empty inline math"));
        assert!(violations[1].message.contains("Empty display math"));
    }

    #[test]
    fn test_invalid_admonish_type() {
        let content = "```admonish invalid";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK010;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("Invalid admonish type 'invalid'")
        );
    }

    #[test]
    fn test_missing_admonish_type() {
        let content = "```admonish";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK010;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("Admonish block missing type")
        );
    }
}
