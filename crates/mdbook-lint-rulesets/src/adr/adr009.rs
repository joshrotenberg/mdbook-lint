//! ADR009: Filename matches ADR number
//!
//! Validates that the filename matches the ADR number in the title.
//! Expected format: NNNN-title-slug.md (e.g., 0001-use-rust.md)

use crate::adr::format::{AdrFormat, detect_format, extract_nygard_number, is_adr_document};
use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use regex::Regex;
use std::sync::LazyLock;

/// Regex to extract number from filename (e.g., "0001-use-rust.md" -> 1)
static FILENAME_NUMBER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+)[-_]").expect("Invalid regex"));

/// ADR009: Validates that filename matches ADR number
pub struct Adr009 {
    /// Configured format (default: auto-detect)
    format: AdrFormat,
}

impl Default for Adr009 {
    fn default() -> Self {
        Self {
            format: AdrFormat::Auto,
        }
    }
}

impl Adr009 {
    /// Get the effective format for the document
    fn effective_format(&self, content: &str) -> AdrFormat {
        match self.format {
            AdrFormat::Auto => detect_format(content),
            other => other,
        }
    }

    /// Extract the number from a filename
    fn extract_filename_number(filename: &str) -> Option<u32> {
        FILENAME_NUMBER_REGEX
            .captures(filename)
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }
}

impl Rule for Adr009 {
    fn id(&self) -> &'static str {
        "ADR009"
    }

    fn name(&self) -> &'static str {
        "adr-filename-matches-number"
    }

    fn description(&self) -> &'static str {
        "ADR filename should match the ADR number in the title"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Structure).introduced_in("mdbook-lint v0.14.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a AstNode<'a>>,
    ) -> mdbook_lint_core::error::Result<Vec<Violation>> {
        // Skip non-ADR documents
        if !is_adr_document(&document.content, Some(&document.path)) {
            return Ok(Vec::new());
        }

        let format = self.effective_format(&document.content);

        // This rule only applies to Nygard format (which has numbered titles)
        if format == AdrFormat::Madr4 {
            return Ok(Vec::new());
        }

        let mut violations = Vec::new();

        // Get filename
        let filename = document
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Extract number from filename
        let filename_number = Self::extract_filename_number(filename);

        // Find H1 heading and extract number from title
        let arena = comrak::Arena::new();
        let ast_node = document.parse_ast(&arena);

        let mut title_number: Option<u32> = None;
        let mut title_line = 1;

        for node in ast_node.descendants() {
            if let NodeValue::Heading(heading) = &node.data.borrow().value
                && heading.level == 1
            {
                title_line = node.data.borrow().sourcepos.start.line;

                // Get the original line to extract number
                if let Some(line) = document.lines.get(title_line.saturating_sub(1)) {
                    title_number = extract_nygard_number(line);
                }
                break;
            }
        }

        // Compare numbers
        match (filename_number, title_number) {
            (Some(fn_num), Some(t_num)) if fn_num != t_num => {
                violations.push(self.create_violation(
                    format!(
                        "Filename number ({:04}) does not match title number ({})",
                        fn_num, t_num
                    ),
                    title_line,
                    1,
                    Severity::Warning,
                ));
            }
            (None, Some(t_num)) => {
                violations.push(self.create_violation(
                    format!(
                        "Filename '{}' does not start with ADR number. Expected format: {:04}-*.md",
                        filename, t_num
                    ),
                    1,
                    1,
                    Severity::Warning,
                ));
            }
            _ => {
                // No title number (handled by ADR001) or numbers match
            }
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_document_with_path(content: &str, path: &str) -> Document {
        Document::new(content.to_string(), PathBuf::from(path)).unwrap()
    }

    #[test]
    fn test_matching_number() {
        let content = r#"# 1. Use Rust for implementation

Date: 2024-01-15

## Status

Accepted
"#;
        let doc = create_test_document_with_path(content, "adr/0001-use-rust.md");
        let rule = Adr009::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_mismatched_number() {
        let content = r#"# 2. Use Rust for implementation

Date: 2024-01-15

## Status

Accepted
"#;
        let doc = create_test_document_with_path(content, "adr/0001-use-rust.md");
        let rule = Adr009::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("does not match"));
    }

    #[test]
    fn test_filename_without_number() {
        let content = r#"# 1. Use Rust for implementation

Date: 2024-01-15

## Status

Accepted
"#;
        let doc = create_test_document_with_path(content, "adr/use-rust.md");
        let rule = Adr009::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("does not start with ADR number")
        );
    }

    #[test]
    fn test_madr_no_check() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL for persistence

## Context and Problem Statement

We need a database.
"#;
        let doc = create_test_document_with_path(content, "adr/use-postgres.md");
        let rule = Adr009::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.is_empty(),
            "MADR format should skip number check"
        );
    }

    #[test]
    fn test_larger_numbers() {
        let content = r#"# 42. Use Kubernetes

Date: 2024-01-15

## Status

Accepted
"#;
        let doc = create_test_document_with_path(content, "adr/0042-use-kubernetes.md");
        let rule = Adr009::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_extract_filename_number() {
        assert_eq!(Adr009::extract_filename_number("0001-use-rust.md"), Some(1));
        assert_eq!(Adr009::extract_filename_number("42-use-rust.md"), Some(42));
        assert_eq!(
            Adr009::extract_filename_number("0042_use_rust.md"),
            Some(42)
        );
        assert_eq!(Adr009::extract_filename_number("use-rust.md"), None);
        assert_eq!(Adr009::extract_filename_number("adr-001.md"), None);
    }

    #[test]
    fn test_underscore_separator() {
        let content = r#"# 1. Use Rust for implementation

Date: 2024-01-15

## Status

Accepted
"#;
        let doc = create_test_document_with_path(content, "adr/0001_use_rust.md");
        let rule = Adr009::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty());
    }
}
