//! MDBOOK012: Broken {{#include}} line ranges
//!
//! This rule checks for invalid line ranges in {{#include}} directives.
//! Line ranges can be specified with colons or dashes.

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};
use regex::Regex;

/// Rule to check for broken {{#include}} line ranges
pub struct MDBOOK012;

impl Rule for MDBOOK012 {
    fn id(&self) -> &'static str {
        "MDBOOK012"
    }

    fn name(&self) -> &'static str {
        "include-line-range-validation"
    }

    fn description(&self) -> &'static str {
        "Broken {{#include}} line ranges"
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

        // Regex to match {{#include path:line_range}}
        let include_re = Regex::new(r"\{\{#include\s*([^}]*)\}\}").unwrap();

        for (line_num, line) in document.lines.iter().enumerate() {
            for capture in include_re.captures_iter(line) {
                if let Some(args) = capture.get(1) {
                    let args_str = args.as_str().trim();

                    // Check if arguments are empty
                    if args_str.is_empty() {
                        violations.push(self.create_violation(
                            "Empty file path in {{#include}} directive".to_string(),
                            line_num + 1,
                            args.start() + 1,
                            Severity::Error,
                        ));
                        continue;
                    }

                    // Parse the arguments (file path and optional line ranges)
                    let parts: Vec<&str> = args_str.split(':').collect();
                    if parts.is_empty() {
                        continue;
                    }

                    let file_path = parts[0].trim();

                    // Check if file path is empty
                    if file_path.is_empty() {
                        violations.push(self.create_violation(
                            "Empty file path in {{#include}} directive".to_string(),
                            line_num + 1,
                            args.start() + 1,
                            Severity::Error,
                        ));
                        continue;
                    }

                    // Check if path is absolute (cross-platform)
                    if file_path.starts_with('/') || file_path.starts_with('\\') 
                        || (file_path.len() > 1 && file_path.chars().nth(1) == Some(':')) {
                        violations.push(self.create_violation(
                            format!(
                                "{{#include}} should use relative paths, found absolute: {}",
                                file_path
                            ),
                            line_num + 1,
                            args.start() + 1,
                            Severity::Error,
                        ));
                    }

                    // Validate line ranges if present
                    if parts.len() > 1 {
                        // Check if this is an anchor format
                        if parts.len() >= 2
                            && (parts[1].trim() == "ANCHOR" || parts[1].trim() == "ANCHOR_END")
                        {
                            // This is an anchor format like file.rs:ANCHOR:name or file.rs:ANCHOR_END:name
                            if parts.len() < 3 || parts[2].trim().is_empty() {
                                violations.push(self.create_violation(
                                    "Anchor name missing after ANCHOR: or ANCHOR_END:".to_string(),
                                    line_num + 1,
                                    args.start() + 1,
                                    Severity::Error,
                                ));
                            }
                        } else {
                            // Regular line range format
                            for part in parts.iter().skip(1) {
                                let range_part = part.trim();

                                if !Self::is_valid_line_range(range_part) {
                                    violations.push(self.create_violation(
                                        format!("Invalid line range in {{#include}}: '{}'. Use formats like :10, :10:20, :10-20", range_part),
                                        line_num + 1,
                                        args.start() + 1,
                                        Severity::Error,
                                    ));
                                }
                            }
                        }

                        // Check for logical errors in ranges
                        if parts.len() == 3 {
                            if let (Ok(start), Ok(end)) = (
                                parts[1].trim().parse::<usize>(),
                                parts[2].trim().parse::<usize>(),
                            ) {
                                if start > end {
                                    violations.push(self.create_violation(
                                        format!("Invalid line range: start line {} is greater than end line {}", start, end),
                                        line_num + 1,
                                        args.start() + 1,
                                        Severity::Error,
                                    ));
                                }
                                if start == 0 {
                                    violations.push(self.create_violation(
                                        "Line numbers start at 1, not 0".to_string(),
                                        line_num + 1,
                                        args.start() + 1,
                                        Severity::Error,
                                    ));
                                }
                            }
                        } else if parts.len() == 2 {
                            // Check for dash-separated ranges
                            if parts[1].contains('-') {
                                let dash_parts: Vec<&str> = parts[1].split('-').collect();
                                if dash_parts.len() == 2
                                    && let (Ok(start), Ok(end)) = (
                                        dash_parts[0].trim().parse::<usize>(),
                                        dash_parts[1].trim().parse::<usize>(),
                                    )
                                {
                                    if start > end {
                                        violations.push(self.create_violation(
                                            format!("Invalid line range: start line {} is greater than end line {}", start, end),
                                            line_num + 1,
                                            args.start() + 1,
                                            Severity::Error,
                                        ));
                                    }
                                    if start == 0 {
                                        violations.push(self.create_violation(
                                            "Line numbers start at 1, not 0".to_string(),
                                            line_num + 1,
                                            args.start() + 1,
                                            Severity::Error,
                                        ));
                                    }
                                }
                            }
                        }
                    }

                    // Check for nested directives
                    if args_str.contains("{{#") {
                        violations.push(self.create_violation(
                            "Nested mdBook directives not allowed in {{#include}}".to_string(),
                            line_num + 1,
                            args.start() + 1,
                            Severity::Error,
                        ));
                    }
                    // Check for anchor syntax (but not if it's a nested directive)
                    else if args_str.contains('#') && !args_str.contains("ANCHOR") {
                        violations.push(self.create_violation(
                            "Invalid anchor syntax. Use ANCHOR:name or ANCHOR_END:name".to_string(),
                            line_num + 1,
                            args.start() + 1,
                            Severity::Warning,
                        ));
                    }
                }
            }
        }

        Ok(violations)
    }
}

impl MDBOOK012 {
    /// Check if a string is a valid line range specification
    fn is_valid_line_range(spec: &str) -> bool {
        // Valid formats:
        // - Single number: "10"
        // - Range with dash: "10-20"
        // - Empty (for end of range in colon format)
        // - ANCHOR:name or ANCHOR_END:name

        if spec.is_empty() {
            return true; // Empty is valid for end range
        }

        if spec.starts_with("ANCHOR:") || spec.starts_with("ANCHOR_END:") {
            return true;
        }

        // Check if it's a single number
        if spec.parse::<usize>().is_ok() {
            return true;
        }

        // Check if it's a range with dash
        if spec.contains('-') {
            let parts: Vec<&str> = spec.split('-').collect();
            if parts.len() == 2 {
                let start_valid = parts[0].is_empty() || parts[0].parse::<usize>().is_ok();
                let end_valid = parts[1].is_empty() || parts[1].parse::<usize>().is_ok();
                return start_valid && end_valid;
            }
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
    fn test_valid_include() {
        let content = r#"# Chapter

{{#include ../src/main.rs}}

With line range:

{{#include ../src/lib.rs:10:20}}

Another format:

{{#include ../src/lib.rs:10-20}}

With anchors:

{{#include ../src/lib.rs:ANCHOR:example}}
"#;
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK012;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_empty_file_path() {
        let content = "{{#include }}";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK012;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Empty file path"));
    }

    #[test]
    fn test_absolute_path() {
        let content = "{{#include /usr/src/main.rs}}";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK012;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("should use relative paths"));
    }

    #[test]
    fn test_invalid_line_range() {
        let content = r#"
{{#include ../src/main.rs:abc}}
{{#include ../src/main.rs:10:xyz}}
{{#include ../src/main.rs:10-}}
"#;
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK012;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 2); // "10-" is valid (open-ended range)
        assert!(violations[0].message.contains("Invalid line range"));
        assert!(violations[1].message.contains("Invalid line range"));
    }

    #[test]
    fn test_inverted_range() {
        let content = r#"
{{#include ../src/main.rs:20:10}}
{{#include ../src/main.rs:20-10}}
"#;
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK012;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(
            violations[0]
                .message
                .contains("start line 20 is greater than end line 10")
        );
        assert!(
            violations[1]
                .message
                .contains("start line 20 is greater than end line 10")
        );
    }

    #[test]
    fn test_zero_line_number() {
        let content = r#"
{{#include ../src/main.rs:0:10}}
{{#include ../src/main.rs:0-10}}
"#;
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK012;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("Line numbers start at 1"));
        assert!(violations[1].message.contains("Line numbers start at 1"));
    }

    #[test]
    fn test_nested_directives() {
        let content = "{{#include {{#rustdoc_include ../src/main.rs}}}}";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK012;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Nested mdBook directives"));
    }

    #[test]
    fn test_invalid_anchor_syntax() {
        let content = "{{#include ../src/main.rs#invalid}}";
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK012;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Invalid anchor syntax"));
    }

    #[test]
    fn test_valid_open_ended_ranges() {
        let content = r#"
{{#include ../src/main.rs:10:}}
{{#include ../src/main.rs:10-}}
{{#include ../src/main.rs:-20}}
"#;
        let doc = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MDBOOK012;
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 0);
    }
}
