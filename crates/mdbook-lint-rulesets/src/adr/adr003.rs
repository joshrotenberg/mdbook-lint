//! ADR003: Required date
//!
//! Validates that the ADR has a date defined:
//! - Nygard format: "Date:" line after the title
//! - MADR format: `date` field in YAML frontmatter

use crate::adr::format::{AdrFormat, detect_format, is_adr_document};
use crate::adr::frontmatter::parse_frontmatter;
use comrak::nodes::AstNode;
use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use regex::Regex;
use std::sync::LazyLock;

/// Regex to match a "Date:" line (case-insensitive)
static DATE_LINE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^date\s*:\s*(.+)$").expect("Invalid regex"));

/// ADR003: Validates that ADR has a date
///
/// For Nygard format ADRs, there must be a "Date:" line.
/// For MADR format ADRs, there must be a `date` field in the frontmatter.
pub struct Adr003 {
    /// Configured format (default: auto-detect)
    format: AdrFormat,
}

impl Default for Adr003 {
    fn default() -> Self {
        Self {
            format: AdrFormat::Auto,
        }
    }
}

impl Adr003 {
    /// Create a new rule with a specific format
    #[allow(dead_code)]
    pub fn with_format(format: AdrFormat) -> Self {
        Self { format }
    }

    /// Get the effective format for the document
    fn effective_format(&self, content: &str) -> AdrFormat {
        match self.format {
            AdrFormat::Auto => detect_format(content),
            other => other,
        }
    }
}

impl Rule for Adr003 {
    fn id(&self) -> &'static str {
        "ADR003"
    }

    fn name(&self) -> &'static str {
        "adr-required-date"
    }

    fn description(&self) -> &'static str {
        "ADR must have a date defined"
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

        let mut violations = Vec::new();

        let format = self.effective_format(&document.content);

        match format {
            AdrFormat::Madr4 => {
                // Check for date in frontmatter
                match parse_frontmatter(&document.content) {
                    Some(result) => {
                        if let Some(ref error) = result.error {
                            // Frontmatter parsing failed
                            violations.push(self.create_violation(
                                format!("Cannot check date: {}", error),
                                result.start_line,
                                1,
                                Severity::Warning,
                            ));
                        } else if let Some(ref fm) = result.frontmatter {
                            if fm.date.is_none() {
                                violations.push(
                                    self.create_violation(
                                        "MADR format ADR is missing 'date' field in frontmatter"
                                            .to_string(),
                                        result.start_line,
                                        1,
                                        Severity::Error,
                                    ),
                                );
                            }
                        } else {
                            // Frontmatter exists but couldn't be parsed
                            violations.push(
                                self.create_violation(
                                    "MADR format ADR is missing 'date' field in frontmatter"
                                        .to_string(),
                                    result.start_line,
                                    1,
                                    Severity::Error,
                                ),
                            );
                        }
                    }
                    None => {
                        // No frontmatter at all, but we detected MADR format
                        violations.push(self.create_violation(
                            "MADR format ADR is missing frontmatter with 'date' field".to_string(),
                            1,
                            1,
                            Severity::Error,
                        ));
                    }
                }
            }
            AdrFormat::Nygard | AdrFormat::Auto => {
                // Check for Date: line in the document body (not in frontmatter)
                let mut found_date = false;

                // Skip frontmatter lines if present
                let skip_lines = parse_frontmatter(&document.content)
                    .map(|r| r.end_line)
                    .unwrap_or(0);

                for line in document.lines.iter().skip(skip_lines) {
                    if DATE_LINE_REGEX.is_match(line) {
                        found_date = true;
                        break;
                    }
                }

                if !found_date {
                    violations.push(self.create_violation(
                        "Nygard format ADR is missing 'Date:' line".to_string(),
                        1,
                        1,
                        Severity::Error,
                    ));
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
        // Use a path that matches ADR directory detection
        Document::new(content.to_string(), PathBuf::from("adr/0001-test-adr.md")).unwrap()
    }

    #[test]
    fn test_valid_nygard_date() {
        let content = r#"# 1. Use Rust for implementation

Date: 2024-01-15

## Status

Accepted
"#;
        let doc = create_test_document(content);
        let rule = Adr003::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.is_empty(),
            "Expected no violations for valid Nygard date"
        );
    }

    #[test]
    fn test_missing_nygard_date() {
        let content = r#"# 1. Use Rust for implementation

## Status

Accepted

## Context

We need a language.
"#;
        let doc = create_test_document(content);
        let rule = Adr003::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("missing 'Date:' line"));
    }

    #[test]
    fn test_valid_madr_date() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL for persistence

## Context and Problem Statement

We need a database.
"#;
        let doc = create_test_document(content);
        let rule = Adr003::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.is_empty(),
            "Expected no violations for valid MADR date"
        );
    }

    #[test]
    fn test_missing_madr_date() {
        let content = r#"---
status: accepted
---

# Use PostgreSQL for persistence

## Context and Problem Statement

We need a database.
"#;
        let doc = create_test_document(content);
        let rule = Adr003::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("missing 'date' field"));
    }

    #[test]
    fn test_date_case_insensitive() {
        let content = r#"# 1. Use Rust

DATE: 2024-01-15

## Status

Accepted
"#;
        let doc = create_test_document(content);
        let rule = Adr003::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.is_empty(),
            "Date line should be case-insensitive"
        );
    }

    #[test]
    fn test_date_without_space() {
        let content = r#"# 1. Use Rust

Date:2024-01-15

## Status

Accepted
"#;
        let doc = create_test_document(content);
        let rule = Adr003::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.is_empty(),
            "Date line should work without space after colon"
        );
    }

    #[test]
    fn test_date_with_extra_whitespace() {
        let content = r#"# 1. Use Rust

Date:    2024-01-15

## Status

Accepted
"#;
        let doc = create_test_document(content);
        let rule = Adr003::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.is_empty(),
            "Date line should work with extra whitespace"
        );
    }

    #[test]
    fn test_empty_frontmatter_missing_date() {
        let content = r#"---
status: accepted
---

# Title
"#;
        let doc = create_test_document(content);
        let rule = Adr003::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("missing 'date' field"));
    }

    #[test]
    fn test_force_nygard_format() {
        // Even with frontmatter, if we force Nygard format,
        // it should check for Date: line
        let content = r#"---
date: 2024-01-15
---

# Title

## Status

Accepted
"#;
        let doc = create_test_document(content);
        let rule = Adr003::with_format(AdrFormat::Nygard);
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("missing 'Date:' line"));
    }

    #[test]
    fn test_date_in_different_formats() {
        // Various date formats should be accepted
        let dates = vec![
            "2024-01-15",
            "January 15, 2024",
            "2024/01/15",
            "15-01-2024",
            "2024.01.15",
        ];

        for date in dates {
            let content = format!(
                r#"# 1. Use Rust

Date: {}

## Status

Accepted
"#,
                date
            );
            let doc = create_test_document(&content);
            let rule = Adr003::default();
            let violations = rule.check(&doc).unwrap();
            assert!(
                violations.is_empty(),
                "Date format '{}' should be accepted",
                date
            );
        }
    }
}
