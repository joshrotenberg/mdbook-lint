//! ADR015: Decision Drivers format
//!
//! Validates that the Decision Drivers section (MADR format) uses a bullet list.

use crate::adr::format::{AdrFormat, detect_format, is_adr_document};
use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};

/// ADR015: Validates that Decision Drivers section uses bullet list format
///
/// In MADR 4.0, the Decision Drivers section should be a bullet list to clearly
/// enumerate the factors influencing the decision.
pub struct Adr015 {
    /// Configured format (default: auto-detect)
    format: AdrFormat,
}

impl Default for Adr015 {
    fn default() -> Self {
        Self {
            format: AdrFormat::Auto,
        }
    }
}

impl Adr015 {
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

impl Rule for Adr015 {
    fn id(&self) -> &'static str {
        "ADR015"
    }

    fn name(&self) -> &'static str {
        "adr-decision-drivers-format"
    }

    fn description(&self) -> &'static str {
        "Decision Drivers section should use a bullet list (MADR)"
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

        // This rule only applies to MADR format
        if format == AdrFormat::Nygard {
            return Ok(Vec::new());
        }

        let mut violations = Vec::new();

        // Parse AST locally
        let arena = comrak::Arena::new();
        let ast_node = document.parse_ast(&arena);

        let mut in_decision_drivers = false;
        let mut decision_drivers_line: usize = 0;
        let mut found_list = false;
        let mut found_content = false;

        for node in ast_node.descendants() {
            let node_data = node.data.borrow();
            let start_line = node_data.sourcepos.start.line;

            match &node_data.value {
                NodeValue::Heading(heading) if heading.level == 2 => {
                    // Check if we were in Decision Drivers and leaving it
                    if in_decision_drivers {
                        if found_content && !found_list {
                            violations.push(self.create_violation(
                                "Decision Drivers section should use a bullet list".to_string(),
                                decision_drivers_line,
                                1,
                                Severity::Info,
                            ));
                        }
                        in_decision_drivers = false;
                        found_list = false;
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
                    if heading_lower == "decision drivers" {
                        in_decision_drivers = true;
                        decision_drivers_line = start_line;
                    }
                }
                NodeValue::List(_) if in_decision_drivers => {
                    found_list = true;
                }
                NodeValue::Paragraph if in_decision_drivers => {
                    found_content = true;
                }
                _ => {}
            }
        }

        // Check if we ended the document while in Decision Drivers
        if in_decision_drivers && found_content && !found_list {
            violations.push(self.create_violation(
                "Decision Drivers section should use a bullet list".to_string(),
                decision_drivers_line,
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
    fn test_valid_decision_drivers_list() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL for persistence

## Context and Problem Statement

We need to select a database.

## Decision Drivers

* Need ACID compliance
* Team familiarity with SQL
* Good tooling ecosystem

## Considered Options

* PostgreSQL
* MySQL

## Decision Outcome

Chosen option: PostgreSQL.
"#;
        let doc = create_test_document(content);
        let rule = Adr015::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.is_empty(),
            "Expected no violations for bullet list"
        );
    }

    #[test]
    fn test_decision_drivers_paragraph_only() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL for persistence

## Context and Problem Statement

We need to select a database.

## Decision Drivers

We need ACID compliance and team familiarity with SQL databases.
Also good tooling is important.

## Considered Options

* PostgreSQL
* MySQL

## Decision Outcome

Chosen option: PostgreSQL.
"#;
        let doc = create_test_document(content);
        let rule = Adr015::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("bullet list"));
    }

    #[test]
    fn test_no_decision_drivers_section() {
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
        let rule = Adr015::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty(), "No violation if section is missing");
    }

    #[test]
    fn test_nygard_format_skipped() {
        // Nygard format doesn't have Decision Drivers, so rule shouldn't apply
        let content = r#"# 1. Use Rust

Date: 2024-01-15

## Status

Accepted

## Context

We need a language.

## Decision

Use Rust.

## Consequences

Team training needed.
"#;
        let doc = create_test_document(content);
        let rule = Adr015::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty(), "Nygard format should be skipped");
    }

    #[test]
    fn test_decision_drivers_with_dash_list() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL

## Context and Problem Statement

Need a database.

## Decision Drivers

- Need ACID compliance
- Team familiarity

## Decision Outcome

PostgreSQL chosen.
"#;
        let doc = create_test_document(content);
        let rule = Adr015::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty(), "Dash lists should also be valid");
    }

    #[test]
    fn test_empty_decision_drivers() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL

## Context and Problem Statement

Need a database.

## Decision Drivers

## Decision Outcome

PostgreSQL chosen.
"#;
        let doc = create_test_document(content);
        let rule = Adr015::default();
        let violations = rule.check(&doc).unwrap();
        // Empty section doesn't trigger this rule (ADR014 handles empty sections)
        assert!(violations.is_empty());
    }
}
