//! ADR008: Date format validation
//!
//! Validates that the ADR date follows ISO 8601 format (YYYY-MM-DD).

use crate::adr::format::{AdrFormat, detect_format, is_adr_document};
use crate::adr::frontmatter::parse_frontmatter;
use comrak::nodes::AstNode;
use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use regex::Regex;
use std::sync::LazyLock;

/// Regex for ISO 8601 date format (YYYY-MM-DD)
static ISO_DATE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d{4}-\d{2}-\d{2}$").expect("Invalid regex"));

/// Regex to extract date from "Date:" line
static DATE_LINE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^date\s*:\s*(.+)$").expect("Invalid regex"));

/// ADR008: Validates that ADR date follows ISO 8601 format
pub struct Adr008 {
    /// Configured format (default: auto-detect)
    format: AdrFormat,
}

impl Default for Adr008 {
    fn default() -> Self {
        Self {
            format: AdrFormat::Auto,
        }
    }
}

impl Adr008 {
    /// Get the effective format for the document
    fn effective_format(&self, content: &str) -> AdrFormat {
        match self.format {
            AdrFormat::Auto => detect_format(content),
            other => other,
        }
    }

    /// Check if a date string is in ISO 8601 format
    fn is_iso_date(date: &str) -> bool {
        let date = date.trim();
        if !ISO_DATE_REGEX.is_match(date) {
            return false;
        }

        // Also validate that it's a reasonable date
        let parts: Vec<&str> = date.split('-').collect();
        if parts.len() != 3 {
            return false;
        }

        if let (Ok(year), Ok(month), Ok(day)) = (
            parts[0].parse::<u32>(),
            parts[1].parse::<u32>(),
            parts[2].parse::<u32>(),
        ) {
            // Basic validation
            (1900..=2100).contains(&year) && (1..=12).contains(&month) && (1..=31).contains(&day)
        } else {
            false
        }
    }
}

impl Rule for Adr008 {
    fn id(&self) -> &'static str {
        "ADR008"
    }

    fn name(&self) -> &'static str {
        "adr-date-format"
    }

    fn description(&self) -> &'static str {
        "ADR date should follow ISO 8601 format (YYYY-MM-DD)"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Content).introduced_in("mdbook-lint v0.14.0")
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

        let mut violations = Vec::new();
        let format = self.effective_format(&document.content);

        match format {
            AdrFormat::Madr4 => {
                // Check date in frontmatter
                if let Some(result) = parse_frontmatter(&document.content)
                    && let Some(ref fm) = result.frontmatter
                    && let Some(ref date) = fm.date
                    && !Self::is_iso_date(date)
                {
                    violations.push(self.create_violation(
                        format!("Date '{}' is not in ISO 8601 format (YYYY-MM-DD)", date),
                        result.start_line,
                        1,
                        Severity::Warning,
                    ));
                }
            }
            AdrFormat::Nygard | AdrFormat::Auto => {
                // Skip frontmatter lines if present
                let skip_lines = parse_frontmatter(&document.content)
                    .map(|r| r.end_line)
                    .unwrap_or(0);

                // Find Date: line
                for (idx, line) in document.lines.iter().enumerate().skip(skip_lines) {
                    if let Some(caps) = DATE_LINE_REGEX.captures(line) {
                        if let Some(date_match) = caps.get(1) {
                            let date = date_match.as_str().trim();
                            if !Self::is_iso_date(date) {
                                violations.push(self.create_violation(
                                    format!(
                                        "Date '{}' is not in ISO 8601 format (YYYY-MM-DD)",
                                        date
                                    ),
                                    idx + 1,
                                    1,
                                    Severity::Warning,
                                ));
                            }
                        }
                        break;
                    }
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
        Document::new(content.to_string(), PathBuf::from("adr/0001-test-adr.md")).unwrap()
    }

    #[test]
    fn test_valid_iso_date_nygard() {
        let content = r#"# 1. Use Rust

Date: 2024-01-15

## Status

Accepted
"#;
        let doc = create_test_document(content);
        let rule = Adr008::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_invalid_date_format_us() {
        let content = r#"# 1. Use Rust

Date: 01/15/2024

## Status

Accepted
"#;
        let doc = create_test_document(content);
        let rule = Adr008::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("not in ISO 8601 format"));
    }

    #[test]
    fn test_invalid_date_format_text() {
        let content = r#"# 1. Use Rust

Date: January 15, 2024

## Status

Accepted
"#;
        let doc = create_test_document(content);
        let rule = Adr008::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("not in ISO 8601 format"));
    }

    #[test]
    fn test_valid_iso_date_madr() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL

## Context and Problem Statement

We need a database.
"#;
        let doc = create_test_document(content);
        let rule = Adr008::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_invalid_date_format_madr() {
        let content = r#"---
status: accepted
date: 15-01-2024
---

# Use PostgreSQL

## Context and Problem Statement

We need a database.
"#;
        let doc = create_test_document(content);
        let rule = Adr008::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("not in ISO 8601 format"));
    }

    #[test]
    fn test_is_iso_date() {
        assert!(Adr008::is_iso_date("2024-01-15"));
        assert!(Adr008::is_iso_date("2024-12-31"));
        assert!(Adr008::is_iso_date("1999-01-01"));
        assert!(!Adr008::is_iso_date("01-15-2024"));
        assert!(!Adr008::is_iso_date("2024/01/15"));
        assert!(!Adr008::is_iso_date("January 15, 2024"));
        assert!(!Adr008::is_iso_date("2024-13-01")); // Invalid month
        assert!(!Adr008::is_iso_date("2024-00-01")); // Invalid month
        assert!(!Adr008::is_iso_date("2024-01-32")); // Invalid day
        assert!(!Adr008::is_iso_date("2024-01-00")); // Invalid day
    }

    #[test]
    fn test_date_with_whitespace() {
        let content = r#"# 1. Use Rust

Date:   2024-01-15

## Status

Accepted
"#;
        let doc = create_test_document(content);
        let rule = Adr008::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.is_empty(),
            "Should handle whitespace around date"
        );
    }
}
