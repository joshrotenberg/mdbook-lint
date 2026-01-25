//! ADR001: ADR title format
//!
//! Validates that the ADR has a properly formatted title:
//! - Nygard format: "# N. Title" or "# N - Title" (H1 with number prefix)
//! - MADR format: Any H1 heading is acceptable

use crate::adr::format::{AdrFormat, detect_format, is_adr_document, is_nygard_title};
use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};

/// ADR001: Validates ADR title format
///
/// For Nygard format ADRs, the title must follow the pattern "# N. Title"
/// where N is the ADR number.
///
/// For MADR format ADRs, any H1 heading is acceptable.
pub struct Adr001 {
    /// Configured format (default: auto-detect)
    format: AdrFormat,
}

impl Default for Adr001 {
    fn default() -> Self {
        Self {
            format: AdrFormat::Auto,
        }
    }
}

impl Adr001 {
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

impl Rule for Adr001 {
    fn id(&self) -> &'static str {
        "ADR001"
    }

    fn name(&self) -> &'static str {
        "adr-title-format"
    }

    fn description(&self) -> &'static str {
        "ADR title should follow the appropriate format for its type"
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

        // Find the first H1 heading
        let mut found_h1 = false;
        let mut h1_line = 0;
        let mut h1_text = String::new();

        // Parse AST locally (we don't share with other rules)
        let arena = comrak::Arena::new();
        let ast_node = document.parse_ast(&arena);

        for node in ast_node.descendants() {
            if let NodeValue::Heading(heading) = &node.data.borrow().value
                && heading.level == 1
            {
                found_h1 = true;
                h1_line = node.data.borrow().sourcepos.start.line;

                // Extract heading text
                for child in node.children() {
                    if let NodeValue::Text(text) = &child.data.borrow().value {
                        h1_text.push_str(text);
                    }
                }
                break;
            }
        }

        if !found_h1 {
            violations.push(self.create_violation(
                format!(
                    "ADR is missing a title (H1 heading). {} format ADRs should have {}",
                    format,
                    if format == AdrFormat::Nygard {
                        "a title like '# 1. Record architecture decisions'"
                    } else {
                        "an H1 heading"
                    }
                ),
                1,
                1,
                Severity::Error,
            ));
            return Ok(violations);
        }

        // For Nygard format, check the title pattern
        if format == AdrFormat::Nygard {
            // Get the original line to check the pattern
            if let Some(line) = document.lines.get(h1_line.saturating_sub(1))
                && !is_nygard_title(line)
            {
                violations.push(self.create_violation(
                    format!(
                        "Nygard format ADR title should follow pattern '# N. Title' (e.g., '# 1. Record architecture decisions'), found: '{}'",
                        line.trim()
                    ),
                    h1_line,
                    1,
                    Severity::Error,
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

    fn create_test_document(content: &str) -> Document {
        // Use a path that matches ADR directory detection
        Document::new(content.to_string(), PathBuf::from("adr/0001-test-adr.md")).unwrap()
    }

    #[test]
    fn test_valid_nygard_title() {
        let content = r#"# 1. Use Rust for implementation

Date: 2024-01-15

## Status

Accepted
"#;
        let doc = create_test_document(content);
        let rule = Adr001::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.is_empty(),
            "Expected no violations for valid Nygard title"
        );
    }

    #[test]
    fn test_valid_nygard_title_with_dash() {
        let content = r#"# 1 - Use Rust for implementation

Date: 2024-01-15

## Status

Accepted
"#;
        let doc = create_test_document(content);
        let rule = Adr001::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.is_empty(),
            "Expected no violations for valid Nygard title with dash"
        );
    }

    #[test]
    fn test_invalid_nygard_title_no_number() {
        let content = r#"# Use Rust for implementation

Date: 2024-01-15

## Status

Accepted
"#;
        let doc = create_test_document(content);
        let rule = Adr001::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("pattern"));
    }

    #[test]
    fn test_missing_title() {
        let content = r#"Date: 2024-01-15

## Status

Accepted
"#;
        let doc = create_test_document(content);
        let rule = Adr001::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("missing a title"));
    }

    #[test]
    fn test_valid_madr_title() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL for persistence

## Context and Problem Statement

We need to select a database.
"#;
        let doc = create_test_document(content);
        let rule = Adr001::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.is_empty(),
            "Expected no violations for valid MADR title"
        );
    }

    #[test]
    fn test_madr_missing_title() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

## Context and Problem Statement

We need to select a database.
"#;
        let doc = create_test_document(content);
        let rule = Adr001::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("missing a title"));
    }

    #[test]
    fn test_force_nygard_format() {
        // Even with frontmatter-like content, if we force Nygard format,
        // it should check for Nygard title pattern
        let content = r#"# Use PostgreSQL

Some content
"#;
        let doc = create_test_document(content);
        let rule = Adr001::with_format(AdrFormat::Nygard);
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("pattern"));
    }

    #[test]
    fn test_large_adr_number() {
        let content = r#"# 9999. Very late decision

Date: 2024-01-15

## Status

Accepted
"#;
        let doc = create_test_document(content);
        let rule = Adr001::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty());
    }
}
