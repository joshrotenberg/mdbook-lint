//! ADR007: Valid status values
//!
//! Validates that the ADR status is one of the recognized values.
//!
//! - proposed, accepted, deprecated, superseded (standard)
//! - rejected, draft (common additions)

use crate::adr::format::{AdrFormat, detect_format, is_adr_document};
use crate::adr::frontmatter::parse_frontmatter;
use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use regex::Regex;
use std::sync::LazyLock;

/// Default valid status values
const DEFAULT_VALID_STATUSES: &[&str] = &[
    "proposed",
    "accepted",
    "deprecated",
    "superseded",
    "rejected",
    "draft",
];

/// Regex to extract status value from a line following "## Status" heading
static STATUS_VALUE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(\w+(?:\s+\w+)*)\s*$").expect("Invalid regex"));

/// ADR007: Validates that ADR has a valid status value
pub struct Adr007 {
    /// Configured format (default: auto-detect)
    format: AdrFormat,
    /// Valid status values (lowercase for comparison)
    valid_statuses: Vec<String>,
}

impl Default for Adr007 {
    fn default() -> Self {
        Self {
            format: AdrFormat::Auto,
            valid_statuses: DEFAULT_VALID_STATUSES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

impl Adr007 {
    /// Create a new rule with custom valid statuses
    #[allow(dead_code)]
    pub fn with_statuses(statuses: Vec<String>) -> Self {
        Self {
            format: AdrFormat::Auto,
            valid_statuses: statuses.into_iter().map(|s| s.to_lowercase()).collect(),
        }
    }

    /// Get the effective format for the document
    fn effective_format(&self, content: &str) -> AdrFormat {
        match self.format {
            AdrFormat::Auto => detect_format(content),
            other => other,
        }
    }

    /// Check if a status value is valid
    fn is_valid_status(&self, status: &str) -> bool {
        let status_lower = status.to_lowercase();
        self.valid_statuses
            .iter()
            .any(|v| v.eq_ignore_ascii_case(&status_lower))
    }
}

impl Rule for Adr007 {
    fn id(&self) -> &'static str {
        "ADR007"
    }

    fn name(&self) -> &'static str {
        "adr-valid-status"
    }

    fn description(&self) -> &'static str {
        "ADR status must be a recognized value"
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
                // Check status in frontmatter
                if let Some(result) = parse_frontmatter(&document.content)
                    && let Some(ref fm) = result.frontmatter
                    && let Some(ref status) = fm.status
                    && !self.is_valid_status(status)
                {
                    violations.push(self.create_violation(
                        format!(
                            "Invalid status '{}'. Valid values: {}",
                            status,
                            self.valid_statuses.join(", ")
                        ),
                        result.start_line,
                        1,
                        Severity::Warning,
                    ));
                }
            }
            AdrFormat::Nygard | AdrFormat::Auto => {
                // Find ## Status section and extract the value
                let arena = comrak::Arena::new();
                let ast_node = document.parse_ast(&arena);

                let mut in_status_section = false;

                for node in ast_node.descendants() {
                    if let NodeValue::Heading(heading) = &node.data.borrow().value {
                        if heading.level == 2 {
                            // Extract heading text
                            let mut heading_text = String::new();
                            for child in node.children() {
                                if let NodeValue::Text(text) = &child.data.borrow().value {
                                    heading_text.push_str(text);
                                }
                            }

                            if heading_text.trim().eq_ignore_ascii_case("status") {
                                in_status_section = true;
                            } else if in_status_section {
                                // Hit another H2, stop looking
                                break;
                            }
                        }
                    } else if in_status_section
                        && let NodeValue::Paragraph = &node.data.borrow().value
                    {
                        // Extract paragraph text
                        let mut para_text = String::new();
                        for child in node.children() {
                            if let NodeValue::Text(text) = &child.data.borrow().value {
                                para_text.push_str(text);
                            }
                        }

                        if let Some(caps) = STATUS_VALUE_REGEX.captures(para_text.trim())
                            && let Some(status_match) = caps.get(1)
                        {
                            let status = status_match.as_str();
                            if !self.is_valid_status(status) {
                                violations.push(self.create_violation(
                                    format!(
                                        "Invalid status '{}'. Valid values: {}",
                                        status,
                                        self.valid_statuses.join(", ")
                                    ),
                                    node.data.borrow().sourcepos.start.line,
                                    1,
                                    Severity::Warning,
                                ));
                            }
                        }
                        break;
                    }
                }

                // Handle case where status section exists but no value found
                // This is handled by ADR002, so we don't report here
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
    fn test_valid_nygard_status_accepted() {
        let content = r#"# 1. Use Rust

Date: 2024-01-15

## Status

Accepted

## Context

We need a language.
"#;
        let doc = create_test_document(content);
        let rule = Adr007::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_valid_nygard_status_proposed() {
        let content = r#"# 1. Use Rust

Date: 2024-01-15

## Status

Proposed

## Context

We need a language.
"#;
        let doc = create_test_document(content);
        let rule = Adr007::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_invalid_nygard_status() {
        let content = r#"# 1. Use Rust

Date: 2024-01-15

## Status

WIP

## Context

We need a language.
"#;
        let doc = create_test_document(content);
        let rule = Adr007::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Invalid status"));
        assert!(violations[0].message.contains("WIP"));
    }

    #[test]
    fn test_valid_madr_status() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL

## Context and Problem Statement

We need a database.
"#;
        let doc = create_test_document(content);
        let rule = Adr007::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_invalid_madr_status() {
        let content = r#"---
status: approved
date: 2024-01-15
---

# Use PostgreSQL

## Context and Problem Statement

We need a database.
"#;
        let doc = create_test_document(content);
        let rule = Adr007::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Invalid status"));
        assert!(violations[0].message.contains("approved"));
    }

    #[test]
    fn test_status_case_insensitive() {
        let content = r#"# 1. Use Rust

Date: 2024-01-15

## Status

ACCEPTED

## Context

We need a language.
"#;
        let doc = create_test_document(content);
        let rule = Adr007::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty(), "Status should be case-insensitive");
    }

    #[test]
    fn test_custom_valid_statuses() {
        let content = r#"# 1. Use Rust

Date: 2024-01-15

## Status

approved

## Context

We need a language.
"#;
        let doc = create_test_document(content);
        let rule = Adr007::with_statuses(vec!["approved".to_string(), "denied".to_string()]);
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_deprecated_status() {
        let content = r#"# 1. Use Rust

Date: 2024-01-15

## Status

Deprecated

## Context

We need a language.
"#;
        let doc = create_test_document(content);
        let rule = Adr007::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_superseded_status() {
        let content = r#"# 1. Use Rust

Date: 2024-01-15

## Status

Superseded

## Context

We need a language.
"#;
        let doc = create_test_document(content);
        let rule = Adr007::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty());
    }
}
