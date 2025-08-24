//! MDBOOK011: Invalid {{#template}} syntax
//!
//! This rule checks for invalid template directives in mdBook files.
//! The template directive allows reusing content snippets.

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};
use regex::Regex;
use std::path::Path;

/// Rule to check for invalid {{#template}} directives
pub struct MDBOOK011;

impl MDBOOK011 {
    /// Parse template arguments, handling quoted strings properly
    fn parse_template_args(args: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut quote_char = ' ';

        for ch in args.chars() {
            match ch {
                '"' | '\'' if !in_quotes => {
                    in_quotes = true;
                    quote_char = ch;
                    current.push(ch);
                }
                c if c == quote_char && in_quotes => {
                    in_quotes = false;
                    current.push(c);
                }
                ' ' | '\t' if !in_quotes => {
                    if !current.is_empty() {
                        result.push(current.clone());
                        current.clear();
                    }
                }
                c => current.push(c),
            }
        }

        if !current.is_empty() {
            result.push(current);
        }

        result
    }
}

impl Rule for MDBOOK011 {
    fn id(&self) -> &'static str {
        "MDBOOK011"
    }

    fn name(&self) -> &'static str {
        "template-validation"
    }

    fn description(&self) -> &'static str {
        "Invalid {{#template}} syntax"
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

        // Regex to match {{#template path args...}}
        let template_re = Regex::new(r"\{\{#template\s*([^}]*)\}\}").unwrap();

        for (line_num, line) in document.lines.iter().enumerate() {
            for capture in template_re.captures_iter(line) {
                if let Some(args) = capture.get(1) {
                    let args_str = args.as_str().trim();

                    // Check if arguments are empty
                    if args_str.is_empty() {
                        violations.push(
                            self.create_violation(
                                "Empty {{#template}} directive - must specify template file"
                                    .to_string(),
                                line_num + 1,
                                args.start() + 1,
                                Severity::Error,
                            ),
                        );
                        continue;
                    }

                    // Parse template arguments (handle quoted strings)
                    let parts = Self::parse_template_args(args_str);
                    if parts.is_empty() {
                        violations.push(self.create_violation(
                            "Missing template file path in {{#template}} directive".to_string(),
                            line_num + 1,
                            args.start() + 1,
                            Severity::Error,
                        ));
                        continue;
                    }

                    let template_path = &parts[0];

                    // Check if path is absolute (cross-platform)
                    let path_str = template_path.as_str();
                    if path_str.starts_with('/') || path_str.starts_with('\\') 
                        || (path_str.len() > 1 && path_str.chars().nth(1) == Some(':')) {
                        violations.push(self.create_violation(
                            format!(
                                "{{#template}} should use relative paths, found absolute: {}",
                                template_path
                            ),
                            line_num + 1,
                            args.start() + 1,
                            Severity::Error,
                        ));
                    }

                    // Check for valid file extension (should be .md or .hbs)
                    let path = Path::new(template_path.as_str());
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy();
                        if !matches!(ext_str.as_ref(), "md" | "hbs" | "html") {
                            violations.push(self.create_violation(
                                format!("{{#template}} should reference a template file (.md, .hbs, or .html), found: .{}", ext_str),
                                line_num + 1,
                                args.start() + 1,
                                Severity::Warning,
                            ));
                        }
                    } else {
                        violations.push(self.create_violation(
                            "{{#template}} file path missing extension".to_string(),
                            line_num + 1,
                            args.start() + 1,
                            Severity::Warning,
                        ));
                    }

                    // Check template arguments format (key=value pairs)
                    for arg in parts.iter().skip(1) {
                        if !arg.contains('=') {
                            violations.push(self.create_violation(
                                format!("Invalid template argument '{}' - should be in key=value format", arg),
                                line_num + 1,
                                args.start() + 1,
                                Severity::Error,
                            ));
                        } else {
                            let kv: Vec<&str> = arg.splitn(2, '=').collect();
                            if kv.len() != 2 || kv[0].is_empty() || kv[1].is_empty() {
                                violations.push(self.create_violation(
                                    format!("Invalid template argument '{}' - key and value must not be empty", arg),
                                    line_num + 1,
                                    args.start() + 1,
                                    Severity::Error,
                                ));
                            }
                        }
                    }

                    // Check for nested directives
                    if args_str.contains("{{#") {
                        violations.push(
                            self.create_violation(
                                "Nested mdBook directives not allowed in {{#template}} path"
                                    .to_string(),
                                line_num + 1,
                                args.start() + 1,
                                Severity::Error,
                            ),
                        );
                    }
                }
            }

            // Check for common misspellings
            if line.contains("{{#tempalte")
                || line.contains("{{#tempate")
                || line.contains("{{#templte")
            {
                violations.push(self.create_violation(
                    "Possible misspelling of {{#template}} directive".to_string(),
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
    fn test_valid_template() {
        let content = r#"# Chapter

{{#template ../templates/header.md title="My Title" author="John Doe"}}

Some content.

{{#template templates/footer.hbs year=2024}}
"#;
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK011;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_empty_template() {
        let content = "{{#template }}";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK011;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("Empty {{#template}} directive")
        );
    }

    #[test]
    fn test_absolute_path() {
        let content = "{{#template /usr/templates/header.md}}";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK011;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("should use relative paths"));
    }

    #[test]
    fn test_invalid_extension() {
        let content = "{{#template templates/header.txt}}";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK011;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("should reference a template file")
        );
    }

    #[test]
    fn test_missing_extension() {
        let content = "{{#template templates/header}}";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK011;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("missing extension"));
    }

    #[test]
    fn test_invalid_arguments() {
        let content = r#"{{#template templates/header.md invalid_arg title=Valid}}"#;
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK011;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("should be in key=value format")
        );
    }

    #[test]
    fn test_empty_key_value() {
        let content = "{{#template templates/header.md =value key=}}";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK011;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(
            violations[0]
                .message
                .contains("key and value must not be empty")
        );
        assert!(
            violations[1]
                .message
                .contains("key and value must not be empty")
        );
    }

    #[test]
    fn test_nested_directives() {
        let content = "{{#template {{#include ../path.md}}}}";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK011;
        let violations = rule.check(&doc).unwrap();

        // We expect at least one violation for nested directives (may have other parse errors too)
        assert!(!violations.is_empty());
        assert!(
            violations
                .iter()
                .any(|v| v.message.contains("Nested mdBook directives"))
        );
    }

    #[test]
    fn test_misspellings() {
        let content = r#"
{{#tempalte templates/header.md}}
{{#tempate templates/header.md}}
{{#templte templates/header.md}}
"#;
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK011;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 3);
        for violation in &violations {
            assert!(violation.message.contains("misspelling"));
        }
    }
}
