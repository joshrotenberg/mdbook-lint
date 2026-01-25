//! ADR010: Superseded ADRs should reference replacement
//!
//! Validates that ADRs with "superseded" status include a reference to the
//! ADR that supersedes them.

use crate::adr::format::{AdrFormat, detect_format, is_adr_document};
use crate::adr::frontmatter::parse_frontmatter;
use mdbook_lint_core::rule::{CollectionRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::Severity;
use mdbook_lint_core::{Document, Result, Violation};
use regex::Regex;
use std::sync::LazyLock;

/// Regex to find ADR references in text (e.g., "ADR-001", "ADR 1", "ADR-0001")
static ADR_REFERENCE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)ADR[-\s]?(\d+)").expect("Invalid regex"));

/// Regex to find markdown links to ADR files
static ADR_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[.*?\]\([^)]*?(?:adr|ADR)[/\\]?\d+[^)]*\.md\)").expect("Invalid regex")
});

/// Regex to extract status from "## Status" section in Nygard format
static STATUS_SECTION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)##\s+Status\s*\n+\s*(\w+)").expect("Invalid regex"));

/// ADR010: Validates that superseded ADRs reference their replacement
///
/// When an ADR has status "superseded", it should contain a reference to
/// the ADR that supersedes it, either as:
/// - A markdown link to another ADR file
/// - A reference like "ADR-001" or "ADR 1"
/// - A "Superseded by" note in the status section
pub struct Adr010;

impl Default for Adr010 {
    fn default() -> Self {
        Self
    }
}

impl Adr010 {
    /// Check if content contains a reference to another ADR
    fn has_adr_reference(content: &str) -> bool {
        ADR_REFERENCE_REGEX.is_match(content) || ADR_LINK_REGEX.is_match(content)
    }

    /// Extract status from document
    fn extract_status(document: &Document) -> Option<String> {
        let format = detect_format(&document.content);

        match format {
            AdrFormat::Madr4 => parse_frontmatter(&document.content)
                .and_then(|r| r.frontmatter)
                .and_then(|fm| fm.status),
            AdrFormat::Nygard | AdrFormat::Auto => STATUS_SECTION_REGEX
                .captures(&document.content)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().to_string()),
        }
    }

    /// Check if status indicates superseded
    fn is_superseded(status: &str) -> bool {
        status.to_lowercase() == "superseded"
    }
}

impl CollectionRule for Adr010 {
    fn id(&self) -> &'static str {
        "ADR010"
    }

    fn name(&self) -> &'static str {
        "adr-superseded-has-replacement"
    }

    fn description(&self) -> &'static str {
        "Superseded ADRs should reference the ADR that replaces them"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Content).introduced_in("mdbook-lint v0.14.0")
    }

    fn check_collection(&self, documents: &[Document]) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        for doc in documents {
            if !is_adr_document(&doc.content, Some(&doc.path)) {
                continue;
            }

            if let Some(status) = Self::extract_status(doc)
                && Self::is_superseded(&status)
                && !Self::has_adr_reference(&doc.content)
            {
                violations.push(self.create_violation_for_file(
                    &doc.path,
                    "Superseded ADR should reference the ADR that replaces it".to_string(),
                    1,
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
    use std::path::PathBuf;

    fn create_nygard_adr(status: &str, extra_content: &str) -> Document {
        let content = format!(
            r#"# 1. Use Rust

Date: 2024-01-15

## Status

{}

## Context

Context here.
{}

## Decision

Decision here.

## Consequences

Consequences here.
"#,
            status, extra_content
        );
        Document::new(content, PathBuf::from("adr/0001-use-rust.md")).unwrap()
    }

    fn create_madr_adr(status: &str, extra_content: &str) -> Document {
        let content = format!(
            r#"---
status: {}
date: 2024-01-15
---

# Use Rust

## Context and Problem Statement

Context here.
{}

## Decision Outcome

Decision here.
"#,
            status, extra_content
        );
        Document::new(content, PathBuf::from("adr/0001-use-rust.md")).unwrap()
    }

    #[test]
    fn test_accepted_adr_no_violation() {
        let docs = vec![create_nygard_adr("Accepted", "")];

        let rule = Adr010;
        let violations = rule.check_collection(&docs).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_superseded_without_reference() {
        let docs = vec![create_nygard_adr("Superseded", "")];

        let rule = Adr010;
        let violations = rule.check_collection(&docs).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("should reference"));
    }

    #[test]
    fn test_superseded_with_adr_reference() {
        let docs = vec![create_nygard_adr("Superseded", "\nSuperseded by ADR-002.")];

        let rule = Adr010;
        let violations = rule.check_collection(&docs).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_superseded_with_markdown_link() {
        let docs = vec![create_nygard_adr(
            "Superseded",
            "\nSuperseded by [ADR-002](adr/0002-new-decision.md).",
        )];

        let rule = Adr010;
        let violations = rule.check_collection(&docs).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_superseded_madr_without_reference() {
        let docs = vec![create_madr_adr("superseded", "")];

        let rule = Adr010;
        let violations = rule.check_collection(&docs).unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_superseded_madr_with_reference() {
        let docs = vec![create_madr_adr("superseded", "\nSuperseded by ADR 5.")];

        let rule = Adr010;
        let violations = rule.check_collection(&docs).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_has_adr_reference() {
        assert!(Adr010::has_adr_reference("See ADR-001 for details"));
        assert!(Adr010::has_adr_reference("Replaced by ADR 42"));
        assert!(Adr010::has_adr_reference("See [link](adr/0001-test.md)"));
        assert!(!Adr010::has_adr_reference("No reference here"));
    }

    #[test]
    fn test_case_insensitive_status() {
        let docs = vec![create_nygard_adr("SUPERSEDED", "")];

        let rule = Adr010;
        let violations = rule.check_collection(&docs).unwrap();
        assert_eq!(violations.len(), 1);
    }
}
