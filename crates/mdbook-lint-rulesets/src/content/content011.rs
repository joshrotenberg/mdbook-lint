//! CONTENT011: No future tense in documentation
//!
//! Documentation should use present tense ("This function returns...")
//! instead of future tense ("This function will return...").

use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use regex::Regex;
use std::sync::LazyLock;

/// Patterns that indicate future tense usage in documentation
/// These are common "will + verb" patterns
static FUTURE_TENSE_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // Common documentation patterns with "will"
        Regex::new(r"\bwill\s+(?:be|have|return|throw|create|generate|produce|output|display|show|print|log|emit|trigger|fire|call|invoke|execute|run|start|stop|open|close|read|write|load|save|send|receive|get|set|add|remove|delete|update|change|modify|process|handle|validate|check|verify|parse|convert|transform|format|render|build|compile|install|download|upload|fetch|request|respond)\b").unwrap(),
        // "is going to" patterns
        Regex::new(r"\bis\s+going\s+to\s+\w+").unwrap(),
        // "are going to" patterns
        Regex::new(r"\bare\s+going\s+to\s+\w+").unwrap(),
    ]
});

/// CONTENT011: Detects future tense in documentation
///
/// Technical documentation should describe what things DO, not what they
/// WILL DO. Present tense is clearer and more direct.
#[derive(Default, Clone)]
pub struct CONTENT011;

impl Rule for CONTENT011 {
    fn id(&self) -> &'static str {
        "CONTENT011"
    }

    fn name(&self) -> &'static str {
        "no-future-tense"
    }

    fn description(&self) -> &'static str {
        "Documentation should use present tense instead of future tense"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Content).introduced_in("mdbook-lint v0.14.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> mdbook_lint_core::error::Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let mut in_code_block = false;

        for (line_idx, line) in document.lines.iter().enumerate() {
            let trimmed = line.trim();

            // Track code blocks
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }

            if in_code_block {
                continue;
            }

            // Skip lines that are likely not documentation prose
            // (headings are OK, we check the content)
            if trimmed.starts_with("<!--") || trimmed.starts_with("{{#") {
                continue;
            }

            let line_lower = line.to_lowercase();

            // Check for future tense patterns
            for pattern in FUTURE_TENSE_PATTERNS.iter() {
                if let Some(m) = pattern.find(&line_lower) {
                    // Find the actual position in the original line
                    let line_num = line_idx + 1;
                    let col = m.start() + 1;
                    let matched_text = &line[m.start()..m.end()];

                    // Suggest present tense alternative
                    let suggestion = matched_text
                        .to_lowercase()
                        .replace("will be", "is")
                        .replace("will have", "has")
                        .replace("will ", "")
                        .replace("is going to ", "")
                        .replace("are going to ", "");

                    violations.push(self.create_violation(
                        format!(
                            "Future tense '{}' found. Consider using present tense: '{}'",
                            matched_text.trim(),
                            suggestion.trim()
                        ),
                        line_num,
                        col,
                        Severity::Info,
                    ));
                    // Only report one violation per line to avoid noise
                    break;
                }
            }
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_document(content: &str) -> Document {
        Document::new(content.to_string(), PathBuf::from("test.md")).unwrap()
    }

    #[test]
    fn test_present_tense() {
        let content = "# Function Reference

This function returns an integer.
The method creates a new instance.
It throws an error if invalid.";
        let doc = create_test_document(content);
        let rule = CONTENT011;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_will_return() {
        let content = "This function will return an integer.";
        let doc = create_test_document(content);
        let rule = CONTENT011;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("will return"));
    }

    #[test]
    fn test_will_be() {
        let content = "The value will be updated.";
        let doc = create_test_document(content);
        let rule = CONTENT011;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("is"));
    }

    #[test]
    fn test_will_throw() {
        let content = "This will throw an exception if the input is invalid.";
        let doc = create_test_document(content);
        let rule = CONTENT011;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_is_going_to() {
        let content = "This method is going to create a new file.";
        let doc = create_test_document(content);
        let rule = CONTENT011;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_are_going_to() {
        let content = "These functions are going to process the data.";
        let doc = create_test_document(content);
        let rule = CONTENT011;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_code_blocks_ignored() {
        let content = "```rust
// This function will return a value
fn example() -> i32 { 42 }
```";
        let doc = create_test_document(content);
        let rule = CONTENT011;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_multiple_violations() {
        let content = "This will return a value.

That will throw an error.";
        let doc = create_test_document(content);
        let rule = CONTENT011;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_will_in_other_context() {
        let content = "The user's free will is respected.";
        let doc = create_test_document(content);
        let rule = CONTENT011;
        let violations = rule.check(&doc).unwrap();
        // "free will" should not trigger
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_case_insensitive() {
        let content = "This WILL RETURN a value.";
        let doc = create_test_document(content);
        let rule = CONTENT011;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_various_verbs() {
        let content = "It will create, will generate, will produce output.";
        let doc = create_test_document(content);
        let rule = CONTENT011;
        let violations = rule.check(&doc).unwrap();
        // Should only report once per line
        assert_eq!(violations.len(), 1);
    }
}
