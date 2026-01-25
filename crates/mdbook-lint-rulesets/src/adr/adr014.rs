//! ADR014: Non-empty sections
//!
//! Validates that required ADR sections are not empty or contain only placeholder text.

use crate::adr::format::{AdrFormat, detect_format, is_adr_document};
use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use std::sync::LazyLock;

/// Common placeholder patterns that indicate incomplete content
static PLACEHOLDER_PATTERNS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "todo",
        "tbd",
        "to be determined",
        "to be decided",
        "fill in",
        "placeholder",
        "describe",
        "add content",
        "write here",
        "...",
        "xxx",
        "[insert",
        "<insert",
        "lorem ipsum",
    ]
});

/// ADR014: Validates that ADR sections have meaningful content
///
/// Checks that required sections (Context, Decision, etc.) are not empty
/// and don't contain only placeholder text like "TODO" or "TBD".
pub struct Adr014 {
    /// Configured format (default: auto-detect)
    format: AdrFormat,
}

impl Default for Adr014 {
    fn default() -> Self {
        Self {
            format: AdrFormat::Auto,
        }
    }
}

impl Adr014 {
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

    /// Check if content appears to be placeholder text
    fn is_placeholder_content(content: &str) -> bool {
        let content_lower = content.trim().to_lowercase();

        // Empty or very short content
        if content_lower.is_empty() || content_lower.len() < 3 {
            return true;
        }

        // Check for placeholder patterns
        for pattern in PLACEHOLDER_PATTERNS.iter() {
            if content_lower.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Get required section names based on format
    fn required_sections(format: AdrFormat) -> Vec<&'static str> {
        match format {
            AdrFormat::Madr4 => vec!["context and problem statement", "decision outcome"],
            AdrFormat::Nygard | AdrFormat::Auto => vec!["context", "decision", "consequences"],
        }
    }

    /// Recursively collect text content from a node and its descendants
    fn collect_text_content<'a>(node: &'a AstNode<'a>, content: &mut String) {
        let node_data = node.data.borrow();
        match &node_data.value {
            NodeValue::Text(text) => {
                content.push_str(text);
                content.push(' ');
            }
            NodeValue::Code(code) => {
                content.push_str(&code.literal);
                content.push(' ');
            }
            NodeValue::SoftBreak | NodeValue::LineBreak => {
                content.push(' ');
            }
            _ => {
                // Recurse into children
                for child in node.children() {
                    Self::collect_text_content(child, content);
                }
            }
        }
    }
}

impl Rule for Adr014 {
    fn id(&self) -> &'static str {
        "ADR014"
    }

    fn name(&self) -> &'static str {
        "adr-non-empty-sections"
    }

    fn description(&self) -> &'static str {
        "ADR sections should have meaningful content, not placeholders"
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
        let required = Self::required_sections(format);

        // Parse AST locally
        let arena = comrak::Arena::new();
        let ast_node = document.parse_ast(&arena);

        // Collect all H2 sections with their content
        let mut sections: Vec<(String, usize, String)> = Vec::new(); // (name, line, content)
        let mut current_section: Option<(String, usize)> = None;
        let mut section_content = String::new();

        // Iterate over top-level children to find H2 sections
        for child in ast_node.children() {
            let child_data = child.data.borrow();
            let start_line = child_data.sourcepos.start.line;

            if let NodeValue::Heading(heading) = &child_data.value {
                if heading.level == 2 {
                    // Save previous section
                    if let Some((name, line)) = current_section.take() {
                        sections.push((name, line, section_content.clone()));
                    }

                    // Extract heading text
                    let mut heading_text = String::new();
                    for text_child in child.children() {
                        if let NodeValue::Text(text) = &text_child.data.borrow().value {
                            heading_text.push_str(text);
                        }
                    }

                    current_section = Some((heading_text.trim().to_string(), start_line));
                    section_content.clear();
                }
            } else if current_section.is_some() {
                // Collect content from non-heading nodes
                Self::collect_text_content(child, &mut section_content);
            }
        }

        // Save final section
        if let Some((name, line)) = current_section {
            sections.push((name, line, section_content));
        }

        // Check each section
        for (section_name, section_line, content) in sections {
            let section_lower = section_name.to_lowercase();
            if required.iter().any(|r| section_lower.contains(r))
                && Self::is_placeholder_content(&content)
            {
                violations.push(self.create_violation(
                    format!(
                        "Section '## {}' appears to be empty or contains only placeholder text",
                        section_name
                    ),
                    section_line,
                    1,
                    Severity::Warning,
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
        Document::new(content.to_string(), PathBuf::from("adr/0001-test-adr.md")).unwrap()
    }

    #[test]
    fn test_valid_nygard_sections() {
        let content = r#"# 1. Use Rust for implementation

Date: 2024-01-15

## Status

Accepted

## Context

We need to choose a programming language for our new service.
The team has experience with multiple languages.

## Decision

We will use Rust for its memory safety and performance characteristics.

## Consequences

Team members will need Rust training.
Build times may be longer initially.
"#;
        let doc = create_test_document(content);
        let rule = Adr014::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.is_empty(),
            "Expected no violations for valid content"
        );
    }

    #[test]
    fn test_empty_context_section() {
        let content = r#"# 1. Use Rust

Date: 2024-01-15

## Status

Accepted

## Context

## Decision

We will use Rust.

## Consequences

Team needs training.
"#;
        let doc = create_test_document(content);
        let rule = Adr014::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Context"));
    }

    #[test]
    fn test_placeholder_todo() {
        let content = r#"# 1. Use Rust

Date: 2024-01-15

## Status

Accepted

## Context

TODO: Fill in the context

## Decision

We will use Rust.

## Consequences

TBD
"#;
        let doc = create_test_document(content);
        let rule = Adr014::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 2, "Expected violations for TODO and TBD");
    }

    #[test]
    fn test_placeholder_ellipsis() {
        let content = r#"# 1. Use Rust

Date: 2024-01-15

## Status

Accepted

## Context

...

## Decision

We will use Rust.

## Consequences

Good consequences here.
"#;
        let doc = create_test_document(content);
        let rule = Adr014::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Context"));
    }

    #[test]
    fn test_valid_madr_sections() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL for persistence

## Context and Problem Statement

We need to select a database for our application's persistence layer.
The choice will affect performance and scalability.

## Decision Outcome

Chosen option: PostgreSQL, because it provides ACID compliance
and has excellent tooling support.
"#;
        let doc = create_test_document(content);
        let rule = Adr014::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.is_empty(),
            "Expected no violations for valid MADR"
        );
    }

    #[test]
    fn test_empty_madr_context() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL

## Context and Problem Statement

## Decision Outcome

Chosen option: PostgreSQL.
"#;
        let doc = create_test_document(content);
        let rule = Adr014::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("Context and Problem Statement")
        );
    }

    #[test]
    fn test_is_placeholder_content() {
        assert!(Adr014::is_placeholder_content(""));
        assert!(Adr014::is_placeholder_content("   "));
        assert!(Adr014::is_placeholder_content("TODO"));
        assert!(Adr014::is_placeholder_content("TBD"));
        assert!(Adr014::is_placeholder_content("To be determined"));
        assert!(Adr014::is_placeholder_content("..."));
        assert!(Adr014::is_placeholder_content("[Insert context here]"));
        assert!(Adr014::is_placeholder_content("Lorem ipsum dolor sit amet"));
        assert!(!Adr014::is_placeholder_content(
            "We need to choose a database."
        ));
        assert!(!Adr014::is_placeholder_content(
            "The team decided to use Rust."
        ));
    }
}
