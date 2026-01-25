//! ADR006: Required consequences section
//!
//! Validates that Nygard format ADRs have a consequences section.
//! MADR format does not require a consequences section.

use crate::adr::format::{AdrFormat, detect_format, is_adr_document};
use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};

/// ADR006: Validates that Nygard format ADR has a consequences section
///
/// For Nygard format ADRs, there must be a "## Consequences" section.
/// MADR format ADRs do not require this section.
pub struct Adr006 {
    /// Configured format (default: auto-detect)
    format: AdrFormat,
}

impl Default for Adr006 {
    fn default() -> Self {
        Self {
            format: AdrFormat::Auto,
        }
    }
}

impl Adr006 {
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

impl Rule for Adr006 {
    fn id(&self) -> &'static str {
        "ADR006"
    }

    fn name(&self) -> &'static str {
        "adr-required-consequences"
    }

    fn description(&self) -> &'static str {
        "Nygard format ADR must have a consequences section"
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

        // MADR format does not require a consequences section
        if format == AdrFormat::Madr4 {
            return Ok(Vec::new());
        }

        let mut violations = Vec::new();

        // Parse AST locally
        let arena = comrak::Arena::new();
        let ast_node = document.parse_ast(&arena);

        let mut found_consequences = false;

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

                if heading_text.trim().eq_ignore_ascii_case("consequences") {
                    found_consequences = true;
                    break;
                }
            }
        }

        if !found_consequences {
            violations.push(self.create_violation(
                "Nygard format ADR is missing '## Consequences' section".to_string(),
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
    fn test_valid_nygard_consequences() {
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
        let rule = Adr006::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_missing_nygard_consequences() {
        let content = r#"# 1. Use Rust for implementation

Date: 2024-01-15

## Status

Accepted

## Context

We need to choose a programming language.

## Decision

We will use Rust.
"#;
        let doc = create_test_document(content);
        let rule = Adr006::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("## Consequences"));
    }

    #[test]
    fn test_madr_no_consequences_required() {
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
        let rule = Adr006::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.is_empty(),
            "MADR format should not require consequences section"
        );
    }

    #[test]
    fn test_consequences_case_insensitive() {
        let content = r#"# 1. Use Rust

Date: 2024-01-15

## Status

Accepted

## Context

We need a language.

## Decision

Use Rust.

## CONSEQUENCES

Training needed.
"#;
        let doc = create_test_document(content);
        let rule = Adr006::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty());
    }
}
