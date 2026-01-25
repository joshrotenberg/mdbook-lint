//! ADR016: Considered Options format
//!
//! Validates that the Considered Options section lists at least 2 options.

use crate::adr::format::{AdrFormat, detect_format, is_adr_document};
use comrak::nodes::{AstNode, ListType, NodeValue};
use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};

/// ADR016: Validates that Considered Options section has multiple options
///
/// In MADR 4.0, the Considered Options section should list at least 2 options
/// to demonstrate that alternatives were evaluated.
pub struct Adr016 {
    /// Configured format (default: auto-detect)
    format: AdrFormat,
    /// Minimum number of options required (default: 2)
    min_options: usize,
}

impl Default for Adr016 {
    fn default() -> Self {
        Self {
            format: AdrFormat::Auto,
            min_options: 2,
        }
    }
}

impl Adr016 {
    /// Create a new rule with a specific format
    #[allow(dead_code)]
    pub fn with_format(format: AdrFormat) -> Self {
        Self {
            format,
            min_options: 2,
        }
    }

    /// Create a new rule with minimum options requirement
    #[allow(dead_code)]
    pub fn with_min_options(min_options: usize) -> Self {
        Self {
            format: AdrFormat::Auto,
            min_options,
        }
    }

    /// Get the effective format for the document
    fn effective_format(&self, content: &str) -> AdrFormat {
        match self.format {
            AdrFormat::Auto => detect_format(content),
            other => other,
        }
    }

    /// Count items in a list node
    fn count_list_items<'a>(list_node: &'a AstNode<'a>) -> usize {
        list_node
            .children()
            .filter(|child| matches!(child.data.borrow().value, NodeValue::Item(_)))
            .count()
    }
}

impl Rule for Adr016 {
    fn id(&self) -> &'static str {
        "ADR016"
    }

    fn name(&self) -> &'static str {
        "adr-considered-options-format"
    }

    fn description(&self) -> &'static str {
        "Considered Options section should list at least 2 options"
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

        let mut in_considered_options = false;
        let mut considered_options_line: usize = 0;
        let mut option_count: usize = 0;
        let mut found_section = false;

        for node in ast_node.descendants() {
            let node_data = node.data.borrow();
            let start_line = node_data.sourcepos.start.line;

            match &node_data.value {
                NodeValue::Heading(heading) if heading.level == 2 => {
                    // Check if we were in Considered Options and leaving it
                    if in_considered_options {
                        found_section = true;
                        if option_count < self.min_options {
                            violations.push(self.create_violation(
                                format!(
                                    "Considered Options section should list at least {} options (found {})",
                                    self.min_options, option_count
                                ),
                                considered_options_line,
                                1,
                                Severity::Info,
                            ));
                        }
                        in_considered_options = false;
                        option_count = 0;
                    }

                    // Extract heading text
                    let mut heading_text = String::new();
                    for child in node.children() {
                        if let NodeValue::Text(text) = &child.data.borrow().value {
                            heading_text.push_str(text);
                        }
                    }

                    let heading_lower = heading_text.trim().to_lowercase();
                    if heading_lower == "considered options" {
                        in_considered_options = true;
                        considered_options_line = start_line;
                    }
                }
                NodeValue::List(list) if in_considered_options => {
                    // Only count unordered lists (bullet lists)
                    if matches!(list.list_type, ListType::Bullet) {
                        option_count += Self::count_list_items(node);
                    }
                }
                NodeValue::Heading(heading) if heading.level == 3 && in_considered_options => {
                    // H3 headings under Considered Options also count as options
                    // (MADR allows "### Option 1" format)
                    option_count += 1;
                }
                _ => {}
            }
        }

        // Check if we ended the document while in Considered Options
        if in_considered_options {
            found_section = true;
            if option_count < self.min_options {
                violations.push(self.create_violation(
                    format!(
                        "Considered Options section should list at least {} options (found {})",
                        self.min_options, option_count
                    ),
                    considered_options_line,
                    1,
                    Severity::Info,
                ));
            }
        }

        // If section exists but is empty, that's also a violation
        if found_section && option_count == 0 {
            // Already reported above
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
    fn test_valid_considered_options() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL for persistence

## Context and Problem Statement

We need to select a database.

## Considered Options

* PostgreSQL
* MySQL
* MongoDB

## Decision Outcome

Chosen option: PostgreSQL.
"#;
        let doc = create_test_document(content);
        let rule = Adr016::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.is_empty(),
            "Expected no violations for 3 options"
        );
    }

    #[test]
    fn test_minimum_two_options() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL

## Context and Problem Statement

Need a database.

## Considered Options

* PostgreSQL
* MySQL

## Decision Outcome

PostgreSQL chosen.
"#;
        let doc = create_test_document(content);
        let rule = Adr016::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty(), "2 options should be valid");
    }

    #[test]
    fn test_only_one_option() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL

## Context and Problem Statement

Need a database.

## Considered Options

* PostgreSQL

## Decision Outcome

PostgreSQL chosen.
"#;
        let doc = create_test_document(content);
        let rule = Adr016::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("at least 2 options"));
    }

    #[test]
    fn test_no_options() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL

## Context and Problem Statement

Need a database.

## Considered Options

## Decision Outcome

PostgreSQL chosen.
"#;
        let doc = create_test_document(content);
        let rule = Adr016::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("found 0"));
    }

    #[test]
    fn test_h3_style_options() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL

## Context and Problem Statement

Need a database.

## Considered Options

### PostgreSQL

A relational database.

### MongoDB

A document database.

## Decision Outcome

PostgreSQL chosen.
"#;
        let doc = create_test_document(content);
        let rule = Adr016::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty(), "H3 style options should be counted");
    }

    #[test]
    fn test_no_section_no_violation() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL

## Context and Problem Statement

Need a database.

## Decision Outcome

PostgreSQL chosen.
"#;
        let doc = create_test_document(content);
        let rule = Adr016::default();
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

Training needed.
"#;
        let doc = create_test_document(content);
        let rule = Adr016::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty(), "Nygard format should be skipped");
    }

    #[test]
    fn test_dash_list_options() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL

## Context and Problem Statement

Need a database.

## Considered Options

- PostgreSQL
- MySQL

## Decision Outcome

PostgreSQL chosen.
"#;
        let doc = create_test_document(content);
        let rule = Adr016::default();
        let violations = rule.check(&doc).unwrap();
        assert!(violations.is_empty(), "Dash lists should work");
    }
}
