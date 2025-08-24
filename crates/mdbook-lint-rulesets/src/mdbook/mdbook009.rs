//! MDBOOK009: Invalid {{#playground}} configuration
//!
//! This rule checks for invalid playground directives in mdBook files.
//! The playground directive allows embedding Rust playground links.

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};
use regex::Regex;

/// Rule to check for invalid {{#playground}} directives
pub struct MDBOOK009;

impl Rule for MDBOOK009 {
    fn id(&self) -> &'static str {
        "MDBOOK009"
    }

    fn name(&self) -> &'static str {
        "playground-validation"
    }

    fn description(&self) -> &'static str {
        "Invalid {{#playground}} configuration"
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

        // Regex to match {{#playground path}} directives
        let playground_re = Regex::new(r"\{\{#playground\s*([^}]*)\}\}").unwrap();

        for (line_num, line) in document.lines.iter().enumerate() {
            for capture in playground_re.captures_iter(line) {
                if let Some(args) = capture.get(1) {
                    let file_path = args.as_str().trim();

                    // Check if file path is empty
                    if file_path.is_empty() {
                        violations.push(self.create_violation(
                            "Empty file path in {{#playground}} directive".to_string(),
                            line_num + 1,
                            args.start() + 1,
                            Severity::Error,
                        ));
                        continue;
                    }

                    // Extract just the file path part (before any colon for line ranges)
                    let base_path = if file_path.contains(':') && !file_path.contains("://") {
                        file_path.split(':').next().unwrap_or(file_path)
                    } else {
                        file_path
                    };

                    // Check if it's a Rust file
                    if !base_path.ends_with(".rs") {
                        violations.push(self.create_violation(
                            format!(
                                "{{#playground}} should reference a Rust file (.rs), found: {}",
                                base_path
                            ),
                            line_num + 1,
                            args.start() + 1,
                            Severity::Error,
                        ));
                    }

                    // Check relative path format (cross-platform)
                    if base_path.starts_with('/')
                        || base_path.starts_with('\\')
                        || (base_path.len() > 1 && base_path.chars().nth(1) == Some(':'))
                    {
                        violations.push(self.create_violation(
                            format!(
                                "{{#playground}} should use relative paths, found absolute: {}",
                                base_path
                            ),
                            line_num + 1,
                            args.start() + 1,
                            Severity::Error,
                        ));
                    }

                    // Check for common mistakes
                    if file_path.contains("{{") || file_path.contains("}}") {
                        violations.push(self.create_violation(
                            "Nested mdBook directives not allowed in {{#playground}}".to_string(),
                            line_num + 1,
                            args.start() + 1,
                            Severity::Error,
                        ));
                    }

                    // Warn about line numbers (not supported)
                    if file_path.contains(':') && !file_path.contains("://") {
                        violations.push(self.create_violation(
                            "{{#playground}} does not support line ranges, use {{#include}} instead for partial content".to_string(),
                            line_num + 1,
                            args.start() + 1,
                            Severity::Warning,
                        ));
                    }
                }
            }

            // Also check for common misspellings
            if (line.contains("{{#play ") || line.contains("{{#play}}"))
                || line.contains("{{#plaground")
                || line.contains("{{#payground")
            {
                violations.push(self.create_violation(
                    "Possible misspelling of {{#playground}} directive".to_string(),
                    line_num + 1,
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
    fn test_valid_playground() {
        let content = r#"# Playground Examples

Here's a playground example:

{{#playground ../examples/hello.rs}}

Another one:

{{#playground examples/demo.rs}}
"#;
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK009;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_empty_file_path() {
        let content = "{{#playground }}";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK009;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Empty file path"));
    }

    #[test]
    fn test_non_rust_file() {
        let content = "{{#playground ../docs/README.md}}";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK009;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("should reference a Rust file")
        );
    }

    #[test]
    fn test_absolute_path() {
        let content = "{{#playground /usr/src/main.rs}}";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK009;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("should use relative paths"));
    }

    #[test]
    fn test_line_ranges_not_supported() {
        let content = "{{#playground ../src/main.rs:10:20}}";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK009;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("does not support line ranges")
        );
    }

    #[test]
    fn test_nested_directives() {
        let content = "{{#playground {{#include ../src/main.rs}}}}";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK009;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Nested mdBook directives"));
    }

    #[test]
    fn test_misspellings() {
        let content = r#"
{{#play ../src/main.rs}}
{{#plaground ../src/main.rs}}
{{#payground ../src/main.rs}}
"#;
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK009;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 3);
        for violation in &violations {
            assert!(violation.message.contains("misspelling"));
        }
    }
}
