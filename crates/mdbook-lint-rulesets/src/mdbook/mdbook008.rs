//! MDBOOK008: Invalid {{#rustdoc_include}} paths or syntax
//!
//! This rule checks for invalid rustdoc_include directives in mdBook files.
//! The rustdoc_include directive allows including portions of Rust files with rustdoc annotations.

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};
use regex::Regex;

/// Rule to check for invalid {{#rustdoc_include}} directives
pub struct MDBOOK008;

impl Rule for MDBOOK008 {
    fn id(&self) -> &'static str {
        "MDBOOK008"
    }

    fn name(&self) -> &'static str {
        "rustdoc-include-validation"
    }

    fn description(&self) -> &'static str {
        "Invalid {{#rustdoc_include}} paths or syntax"
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

        // Regex to match {{#rustdoc_include path:line_range}}
        // Supports various formats:
        // {{#rustdoc_include file.rs}}
        // {{#rustdoc_include file.rs:10}}
        // {{#rustdoc_include file.rs:10:20}}
        // {{#rustdoc_include file.rs:10-20}}
        let rustdoc_include_re = Regex::new(r"\{\{#rustdoc_include\s*([^}]*)\}\}").unwrap();

        for (line_num, line) in document.lines.iter().enumerate() {
            for capture in rustdoc_include_re.captures_iter(line) {
                if let Some(args) = capture.get(1) {
                    let args_str = args.as_str().trim();

                    // Check for empty directive
                    if args_str.is_empty() {
                        violations.push(self.create_violation(
                            "Empty file path in {{#rustdoc_include}} directive".to_string(),
                            line_num + 1,
                            args.start() + 1,
                            Severity::Error,
                        ));
                        continue;
                    }

                    // Parse the arguments
                    let parts: Vec<&str> = args_str.split(':').collect();
                    if parts.is_empty() {
                        violations.push(self.create_violation(
                            "Missing file path in {{#rustdoc_include}} directive".to_string(),
                            line_num + 1,
                            args.start() + 1,
                            Severity::Error,
                        ));
                        continue;
                    }

                    let file_path = parts[0].trim();

                    // Check if file path is empty
                    if file_path.is_empty() {
                        violations.push(self.create_violation(
                            "Empty file path in {{#rustdoc_include}} directive".to_string(),
                            line_num + 1,
                            args.start() + 1,
                            Severity::Error,
                        ));
                        continue;
                    }

                    // Check if it's a Rust file
                    if !file_path.ends_with(".rs") {
                        violations.push(self.create_violation(
                            format!("{{#rustdoc_include}} should reference a Rust file (.rs), found: {}", file_path),
                            line_num + 1,
                            args.start() + 1,
                            Severity::Warning,
                        ));
                    }

                    // Check relative path format (cross-platform)
                    if file_path.starts_with('/')
                        || file_path.starts_with('\\')
                        || (file_path.len() > 1 && file_path.chars().nth(1) == Some(':'))
                    {
                        violations.push(self.create_violation(
                            format!("{{#rustdoc_include}} should use relative paths, found absolute: {}", file_path),
                            line_num + 1,
                            args.start() + 1,
                            Severity::Error,
                        ));
                    }

                    // Validate line range format if present
                    if parts.len() > 1 {
                        for part in parts.iter().skip(1) {
                            let range_part = part.trim();

                            // Check for valid line number or range
                            if !Self::is_valid_line_spec(range_part) {
                                violations.push(self.create_violation(
                                    format!(
                                        "Invalid line specification in {{#rustdoc_include}}: '{}'",
                                        range_part
                                    ),
                                    line_num + 1,
                                    args.start() + 1,
                                    Severity::Error,
                                ));
                            }
                        }
                    }
                }
            }
        }

        Ok(violations)
    }
}

impl MDBOOK008 {
    /// Check if a string is a valid line specification
    fn is_valid_line_spec(spec: &str) -> bool {
        // Valid formats:
        // - Single number: "10"
        // - Range with dash: "10-20"
        // - Range with colon: "20" (when it's the second part after a start line)

        // Check if it's a single number
        if spec.parse::<usize>().is_ok() {
            return true;
        }

        // Check if it's a range with dash
        if let Some(dash_pos) = spec.find('-') {
            let start = &spec[..dash_pos];
            let end = &spec[dash_pos + 1..];

            return start.parse::<usize>().is_ok() && end.parse::<usize>().is_ok();
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::Document;
    use std::path::PathBuf;

    #[test]
    fn test_valid_rustdoc_include() {
        let content = r#"# Chapter
        
Here's some code:

{{#rustdoc_include ../src/main.rs}}

With line range:

{{#rustdoc_include ../src/lib.rs:10:20}}

Another format:

{{#rustdoc_include ../src/lib.rs:10-20}}
"#;
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK008;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_empty_file_path() {
        let content = "{{#rustdoc_include }}";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK008;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Empty file path"));
    }

    #[test]
    fn test_non_rust_file() {
        let content = "{{#rustdoc_include ../docs/README.md}}";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK008;
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
        let content = "{{#rustdoc_include /usr/src/main.rs}}";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK008;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("should use relative paths"));
    }

    #[test]
    fn test_invalid_line_range() {
        let content = r#"
{{#rustdoc_include ../src/main.rs:abc}}
{{#rustdoc_include ../src/main.rs:10:xyz}}
{{#rustdoc_include ../src/main.rs:10-}}
"#;
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK008;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 3);
        assert!(violations[0].message.contains("Invalid line specification"));
        assert!(violations[1].message.contains("Invalid line specification"));
        assert!(violations[2].message.contains("Invalid line specification"));
    }

    #[test]
    fn test_multiple_rustdoc_includes() {
        let content = r#"
# Examples

{{#rustdoc_include ../src/lib.rs:1:10}}

Some text.

{{#rustdoc_include ../src/main.rs}}

More text.

{{#rustdoc_include ../examples/demo.rs:5-15}}
"#;
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK008;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 0);
    }
}
