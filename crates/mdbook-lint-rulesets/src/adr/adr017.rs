//! ADR017: Consequences structure
//!
//! Validates that the Consequences section (MADR format) distinguishes good/bad outcomes.

use crate::adr::format::{AdrFormat, detect_format, is_adr_document};
use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use regex::Regex;
use std::sync::LazyLock;

/// Regex patterns for good/bad consequence markers
static GOOD_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^\s*\*?\s*good[,:]").expect("Invalid regex"));

static BAD_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^\s*\*?\s*bad[,:]").expect("Invalid regex"));

static NEUTRAL_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^\s*\*?\s*neutral[,:]").expect("Invalid regex"));

/// ADR017: Validates that Consequences section has structured outcomes
///
/// In MADR 4.0, the Consequences section (often under "### Consequences" or
/// "### Positive Consequences" / "### Negative Consequences") should distinguish
/// between good and bad outcomes using "Good, because..." and "Bad, because..." format.
pub struct Adr017 {
    /// Configured format (default: auto-detect)
    format: AdrFormat,
}

impl Default for Adr017 {
    fn default() -> Self {
        Self {
            format: AdrFormat::Auto,
        }
    }
}

impl Adr017 {
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

    /// Check if a line has good/bad/neutral structure
    fn has_outcome_marker(text: &str) -> bool {
        GOOD_PATTERN.is_match(text) || BAD_PATTERN.is_match(text) || NEUTRAL_PATTERN.is_match(text)
    }
}

impl Rule for Adr017 {
    fn id(&self) -> &'static str {
        "ADR017"
    }

    fn name(&self) -> &'static str {
        "adr-consequences-structure"
    }

    fn description(&self) -> &'static str {
        "Consequences should distinguish good/bad outcomes (MADR)"
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

        let format = self.effective_format(&document.content);

        // This rule only applies to MADR format
        if format == AdrFormat::Nygard {
            return Ok(Vec::new());
        }

        let mut violations = Vec::new();

        // Parse AST locally
        let arena = comrak::Arena::new();
        let ast_node = document.parse_ast(&arena);

        // Track if we're in a consequences-related section
        let mut in_consequences = false;
        let mut consequences_line: usize = 0;
        let mut found_structured_outcome = false;
        let mut found_content = false;
        let mut section_depth: u8 = 0;

        for node in ast_node.descendants() {
            let node_data = node.data.borrow();
            let start_line = node_data.sourcepos.start.line;

            match &node_data.value {
                NodeValue::Heading(heading) => {
                    // Check if we were in Consequences and leaving it
                    if in_consequences && heading.level <= section_depth {
                        if found_content && !found_structured_outcome {
                            violations.push(self.create_violation(
                                "Consequences section should use 'Good, because...' and 'Bad, because...' format to distinguish outcomes".to_string(),
                                consequences_line,
                                1,
                                Severity::Info,
                            ));
                        }
                        in_consequences = false;
                        found_structured_outcome = false;
                        found_content = false;
                    }

                    // Extract heading text
                    let mut heading_text = String::new();
                    for child in node.children() {
                        if let NodeValue::Text(text) = &child.data.borrow().value {
                            heading_text.push_str(text);
                        }
                    }

                    let heading_lower = heading_text.trim().to_lowercase();

                    // Check for consequences-related sections
                    // MADR often uses "### Consequences" under "## Decision Outcome"
                    // or separate "### Positive Consequences" / "### Negative Consequences"
                    if heading_lower == "consequences"
                        || heading_lower == "positive consequences"
                        || heading_lower == "negative consequences"
                    {
                        // If we find positive/negative specific sections, that's structured
                        if heading_lower.contains("positive") || heading_lower.contains("negative")
                        {
                            found_structured_outcome = true;
                        } else {
                            in_consequences = true;
                            consequences_line = start_line;
                            section_depth = heading.level;
                        }
                    }
                }
                NodeValue::Item(_) if in_consequences => {
                    found_content = true;
                    // Check item content for good/bad markers
                    let mut item_text = String::new();
                    for child in node.descendants() {
                        if let NodeValue::Text(text) = &child.data.borrow().value {
                            item_text.push_str(text);
                        }
                    }
                    if Self::has_outcome_marker(&item_text) {
                        found_structured_outcome = true;
                    }
                }
                NodeValue::Paragraph if in_consequences => {
                    found_content = true;
                    // Also check paragraph content
                    let mut para_text = String::new();
                    for child in node.descendants() {
                        if let NodeValue::Text(text) = &child.data.borrow().value {
                            para_text.push_str(text);
                        }
                    }
                    if Self::has_outcome_marker(&para_text) {
                        found_structured_outcome = true;
                    }
                }
                _ => {}
            }
        }

        // Check if we ended the document while in Consequences
        if in_consequences && found_content && !found_structured_outcome {
            violations.push(self.create_violation(
                "Consequences section should use 'Good, because...' and 'Bad, because...' format to distinguish outcomes".to_string(),
                consequences_line,
                1,
                Severity::Info,
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
    fn test_valid_structured_consequences() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL for persistence

## Context and Problem Statement

We need to select a database.

## Decision Outcome

Chosen option: PostgreSQL.

### Consequences

* Good, because it provides ACID compliance
* Good, because team has SQL experience
* Bad, because requires more operational overhead
* Neutral, because licensing is similar to alternatives
"#;
        let doc = create_test_document(content);
        let rule = Adr017::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.is_empty(),
            "Expected no violations for structured consequences"
        );
    }

    #[test]
    fn test_unstructured_consequences() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL for persistence

## Context and Problem Statement

We need to select a database.

## Decision Outcome

Chosen option: PostgreSQL.

### Consequences

* Provides ACID compliance
* Team has SQL experience
* Requires more operational overhead
"#;
        let doc = create_test_document(content);
        let rule = Adr017::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Good, because"));
    }

    #[test]
    fn test_positive_negative_sections() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL for persistence

## Context and Problem Statement

We need to select a database.

## Decision Outcome

Chosen option: PostgreSQL.

### Positive Consequences

* ACID compliance
* Good tooling

### Negative Consequences

* Operational overhead
"#;
        let doc = create_test_document(content);
        let rule = Adr017::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.is_empty(),
            "Separate positive/negative sections should be valid"
        );
    }

    #[test]
    fn test_no_consequences_section() {
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
        let rule = Adr017::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty(), "No violation if section missing");
    }

    #[test]
    fn test_nygard_format_skipped() {
        let content = r#"# 1. Use Rust

Date: 2024-01-15

## Status

Accepted

## Context

We need a language.

## Decision

Use Rust.

## Consequences

Team needs training.
Performance will improve.
"#;
        let doc = create_test_document(content);
        let rule = Adr017::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty(), "Nygard format should be skipped");
    }

    #[test]
    fn test_mixed_case_markers() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL

## Context and Problem Statement

Need a database.

## Decision Outcome

PostgreSQL chosen.

### Consequences

* GOOD, because ACID compliance
* BAD, because operational overhead
"#;
        let doc = create_test_document(content);
        let rule = Adr017::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty(), "Case insensitive matching");
    }

    #[test]
    fn test_empty_consequences() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL

## Context and Problem Statement

Need a database.

## Decision Outcome

PostgreSQL chosen.

### Consequences

## Next Steps
"#;
        let doc = create_test_document(content);
        let rule = Adr017::default();
        let violations = rule.check(&doc).unwrap();
        // Empty section doesn't trigger - ADR014 handles that
        assert!(violations.is_empty());
    }

    #[test]
    fn test_has_outcome_marker() {
        assert!(Adr017::has_outcome_marker("Good, because it works"));
        assert!(Adr017::has_outcome_marker("* Good, because it works"));
        assert!(Adr017::has_outcome_marker("Bad, because it's slow"));
        assert!(Adr017::has_outcome_marker("Neutral, because no change"));
        assert!(Adr017::has_outcome_marker("  Good: better performance"));
        assert!(!Adr017::has_outcome_marker("It works well"));
        assert!(!Adr017::has_outcome_marker("This is good for us"));
    }
}
