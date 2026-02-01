//! ADR005: Required decision section
//!
//! Validates that the ADR has a decision section.
//!
//! - Nygard format: "## Decision" section
//! - MADR format: "## Decision Outcome" section

use crate::adr::format::{AdrFormat, detect_format, is_adr_document};
use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};

/// ADR005: Validates that ADR has a decision section
///
/// For Nygard format ADRs, there must be a "## Decision" section.
/// For MADR format ADRs, there must be a "## Decision Outcome" section.
pub struct Adr005 {
    /// Configured format (default: auto-detect)
    format: AdrFormat,
}

impl Default for Adr005 {
    fn default() -> Self {
        Self {
            format: AdrFormat::Auto,
        }
    }
}

impl Adr005 {
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

impl Rule for Adr005 {
    fn id(&self) -> &'static str {
        "ADR005"
    }

    fn name(&self) -> &'static str {
        "adr-required-decision"
    }

    fn description(&self) -> &'static str {
        "ADR must have a decision section"
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

        // Parse AST locally
        let arena = comrak::Arena::new();
        let ast_node = document.parse_ast(&arena);

        let mut found_decision = false;

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

                let heading_lower = heading_text.trim().to_lowercase();

                match format {
                    AdrFormat::Nygard | AdrFormat::Auto => {
                        if heading_lower == "decision" {
                            found_decision = true;
                            break;
                        }
                    }
                    AdrFormat::Madr4 => {
                        if heading_lower == "decision outcome" {
                            found_decision = true;
                            break;
                        }
                    }
                }
            }
        }

        if !found_decision {
            let expected = match format {
                AdrFormat::Madr4 => "## Decision Outcome",
                _ => "## Decision",
            };
            violations.push(self.create_violation(
                format!("ADR is missing '{}' section", expected),
                1,
                1,
                Severity::Error,
            ));
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
    fn test_valid_nygard_decision() {
        let content = r#"# 1. Use Rust for implementation

Date: 2024-01-15

## Status

Accepted

## Context

We need to choose a programming language.

## Decision

We will use Rust.

## Consequences

Team needs Rust training.
"#;
        let doc = create_test_document(content);
        let rule = Adr005::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_missing_nygard_decision() {
        let content = r#"# 1. Use Rust for implementation

Date: 2024-01-15

## Status

Accepted

## Context

We need to choose a programming language.

## Consequences

Team needs Rust training.
"#;
        let doc = create_test_document(content);
        let rule = Adr005::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("## Decision"));
    }

    #[test]
    fn test_valid_madr_decision() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL for persistence

## Context and Problem Statement

We need to select a database.

## Decision Outcome

Chosen option: PostgreSQL.
"#;
        let doc = create_test_document(content);
        let rule = Adr005::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_missing_madr_decision() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL for persistence

## Context and Problem Statement

We need to select a database.
"#;
        let doc = create_test_document(content);
        let rule = Adr005::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Decision Outcome"));
    }
}
