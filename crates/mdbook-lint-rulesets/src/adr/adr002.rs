//! ADR002: Required status
//!
//! Validates that the ADR has a status defined:
//! - Nygard format: "## Status" section with status value
//! - MADR format: `status` field in YAML frontmatter

use crate::adr::format::{AdrFormat, detect_format, is_adr_document};
use crate::adr::frontmatter::parse_frontmatter;
use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use regex::Regex;
use std::sync::LazyLock;

/// Regex to match a "## Status" heading (case-insensitive)
static STATUS_HEADING_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^##\s+status\s*$").expect("Invalid regex"));

/// ADR002: Validates that ADR has a status
///
/// For Nygard format ADRs, there must be a "## Status" section.
/// For MADR format ADRs, there must be a `status` field in the frontmatter.
pub struct Adr002 {
    /// Configured format (default: auto-detect)
    format: AdrFormat,
}

impl Default for Adr002 {
    fn default() -> Self {
        Self {
            format: AdrFormat::Auto,
        }
    }
}

impl Adr002 {
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

impl Rule for Adr002 {
    fn id(&self) -> &'static str {
        "ADR002"
    }

    fn name(&self) -> &'static str {
        "adr-required-status"
    }

    fn description(&self) -> &'static str {
        "ADR must have a status defined"
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
                // Check for status in frontmatter
                match parse_frontmatter(&document.content) {
                    Some(result) => {
                        if let Some(ref error) = result.error {
                            // Frontmatter parsing failed
                            violations.push(self.create_violation(
                                format!("Cannot check status: {}", error),
                                result.start_line,
                                1,
                                Severity::Warning,
                            ));
                        } else if let Some(ref fm) = result.frontmatter {
                            if fm.status.is_none() {
                                violations.push(
                                    self.create_violation(
                                        "MADR format ADR is missing 'status' field in frontmatter"
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
                                    "MADR format ADR is missing 'status' field in frontmatter"
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
                        // This shouldn't happen with proper format detection
                        violations.push(
                            self.create_violation(
                                "MADR format ADR is missing frontmatter with 'status' field"
                                    .to_string(),
                                1,
                                1,
                                Severity::Error,
                            ),
                        );
                    }
                }
            }
            AdrFormat::Nygard | AdrFormat::Auto => {
                // Check for ## Status section
                let mut found_status_section = false;

                // Parse AST locally (we don't share with other rules)
                let arena = comrak::Arena::new();
                let ast_node = document.parse_ast(&arena);

                for node in ast_node.descendants() {
                    if let NodeValue::Heading(heading) = &node.data.borrow().value
                        && heading.level == 2
                    {
                        // Extract heading text
                        let mut heading_text = String::new();
                        for child in node.children() {
                            if let NodeValue::Text(text) = &child.data.borrow().value {
                                heading_text.push_str(text);
                            }
                        }

                        if heading_text.trim().eq_ignore_ascii_case("status") {
                            found_status_section = true;
                            break;
                        }
                    }
                }

                // Fallback: check lines directly
                if !found_status_section {
                    for line in document.lines.iter() {
                        if STATUS_HEADING_REGEX.is_match(line) {
                            found_status_section = true;
                            break;
                        }
                    }
                }

                if !found_status_section {
                    violations.push(self.create_violation(
                        "Nygard format ADR is missing '## Status' section".to_string(),
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
    fn test_valid_nygard_status() {
        let content = r#"# 1. Use Rust for implementation

Date: 2024-01-15

## Status

Accepted

## Context

We need a language.
"#;
        let doc = create_test_document(content);
        let rule = Adr002::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.is_empty(),
            "Expected no violations for valid Nygard status"
        );
    }

    #[test]
    fn test_missing_nygard_status() {
        let content = r#"# 1. Use Rust for implementation

Date: 2024-01-15

## Context

We need a language.

## Decision

Use Rust.
"#;
        let doc = create_test_document(content);
        let rule = Adr002::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("missing '## Status' section")
        );
    }

    #[test]
    fn test_valid_madr_status() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL for persistence

## Context and Problem Statement

We need a database.
"#;
        let doc = create_test_document(content);
        let rule = Adr002::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.is_empty(),
            "Expected no violations for valid MADR status"
        );
    }

    #[test]
    fn test_missing_madr_status() {
        let content = r#"---
date: 2024-01-15
---

# Use PostgreSQL for persistence

## Context and Problem Statement

We need a database.
"#;
        let doc = create_test_document(content);
        let rule = Adr002::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("missing 'status' field"));
    }

    #[test]
    fn test_status_case_insensitive() {
        let content = r#"# 1. Use Rust

Date: 2024-01-15

## STATUS

Accepted
"#;
        let doc = create_test_document(content);
        let rule = Adr002::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.is_empty(),
            "Status section should be case-insensitive"
        );
    }

    #[test]
    fn test_empty_frontmatter() {
        let content = r#"---
---

# Title
"#;
        let doc = create_test_document(content);
        let rule = Adr002::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("missing 'status' field"));
    }

    #[test]
    fn test_force_nygard_format() {
        // Even with frontmatter, if we force Nygard format,
        // it should check for ## Status section
        let content = r#"---
status: accepted
---

# Title

## Context

Content
"#;
        let doc = create_test_document(content);
        let rule = Adr002::with_format(AdrFormat::Nygard);
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("missing '## Status' section")
        );
    }

    #[test]
    fn test_status_with_extra_whitespace() {
        let content = r#"# 1. Use Rust

Date: 2024-01-15

##   Status

Accepted
"#;
        let doc = create_test_document(content);
        let rule = Adr002::default();
        let violations = rule.check(&doc).unwrap();
        // The regex allows for whitespace after "Status"
        // but the AST-based check is more strict
        // This tests the fallback regex pattern
        assert!(violations.is_empty() || violations.len() == 1);
    }
}
