//! ADR014: Non-empty sections
//!
//! Validates that required ADR sections are not empty or contain only placeholder text.

use crate::adr::format::{AdrFormat, detect_format, is_adr_document};
use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use regex::Regex;
use std::sync::LazyLock;

/// Markers that do not appear in finished prose, matched anywhere in a section.
///
/// These are unambiguous: a section mentioning "TODO" or "lorem ipsum" is still
/// a stub regardless of what else it contains.
static STUB_MARKER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(?:todo|tbd|to be determined|to be decided|placeholder|write here|xxx|lorem ipsum)\b",
    )
    .expect("Invalid regex")
});

/// Template guidance left in place by an author who did not replace it.
///
/// "describe", "fill in", and "add content" used to be matched as bare words,
/// which reported any section using them in ordinary prose. An ADR context
/// section frequently describes something, so
/// "These three skills describe org-wide procedure" was classified as a
/// placeholder (issue #481). Adding a word boundary had already fixed the
/// inflected "described", but left the base verb.
///
/// Matching the template form instead keeps the original intent at no cost to
/// coverage. MADR wraps its guidance in braces, as in
/// `{Describe the context and problem statement, e.g., in free form...}`, so the
/// braced form is matched generically, along with the unbraced template sentence
/// for authors who removed the braces but not the text.
static TEMPLATE_PLACEHOLDER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?:\{[^{}]*\b(?:describe|fill in|add content)\b[^{}]*\}|\bdescribe the context and problem statement\b)",
    )
    .expect("Invalid regex")
});

/// Literal placeholder patterns that don't need word boundaries.
///
/// "..." was removed: an ellipsis is ordinary punctuation, and matching it with
/// `contains` reported any section using one mid-sentence. A section consisting
/// only of an ellipsis is still caught, by the punctuation-only check below.
static PLACEHOLDER_LITERALS: &[&str] = &["[insert", "<insert"];

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
    /// Create an instance from rule configuration.
    ///
    /// Recognized key:
    /// - `format`: `"auto"`, `"nygard"`, or `"madr"`. Overrides the
    ///   per-document format auto-detection. Unrecognized values are ignored.
    pub fn from_config(config: &toml::Value) -> Self {
        let mut rule = Self::default();
        if let Some(format) = config
            .get("format")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<AdrFormat>().ok())
        {
            rule.format = format;
        }
        rule
    }

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
        let trimmed = content.trim();

        // Empty or very short content
        if trimmed.is_empty() || trimmed.len() < 3 {
            return true;
        }

        // A section made only of punctuation, such as a lone "...", carries no
        // content. This keeps that case covered now that "..." is no longer
        // matched anywhere in a section.
        if !trimmed.chars().any(|c| c.is_alphanumeric()) {
            return true;
        }

        // Unambiguous stub markers, and template guidance that was never replaced
        if STUB_MARKER_REGEX.is_match(trimmed) || TEMPLATE_PLACEHOLDER_REGEX.is_match(trimmed) {
            return true;
        }

        // Check literal placeholder patterns that don't need word boundaries
        let lower = trimmed.to_lowercase();
        PLACEHOLDER_LITERALS.iter().any(|p| lower.contains(p))
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
        // "described" should NOT match the "describe" placeholder pattern
        assert!(!Adr014::is_placeholder_content(
            r#"We will use Architecture Decision Records, as described by Michael Nygard in his article "Documenting Architecture Decisions"."#
        ));
    }

    #[test]
    fn test_ordinary_prose_using_placeholder_verbs_is_not_flagged() {
        // Issue #481: the bare verbs were matched anywhere in a section, so any
        // ADR using them in ordinary prose carried a permanent warning.
        for prose in [
            "These three skills describe org-wide procedure, so a copy in each repository would drift.",
            "The adapters describe their capabilities at startup.",
            "We fill in the remaining fields from the environment.",
            "Editors add content to the page without a rebuild.",
        ] {
            assert!(
                !Adr014::is_placeholder_content(prose),
                "ordinary prose reported as placeholder: {prose}"
            );
        }
    }

    #[test]
    fn test_ellipsis_in_prose_is_not_flagged() {
        // Issue #481: "..." was matched with `contains`, so an ellipsis anywhere
        // in a sentence reported the section.
        assert!(!Adr014::is_placeholder_content(
            "The alternative, a copy per repository, drifts... and drift is the whole problem."
        ));
    }

    #[test]
    fn test_section_of_only_punctuation_is_still_a_placeholder() {
        // Coverage kept after removing "..." from the literal list.
        for content in ["...", ". . .", "---", "?!"] {
            assert!(
                Adr014::is_placeholder_content(content),
                "punctuation-only section should be a placeholder: {content:?}"
            );
        }
    }

    #[test]
    fn test_unreplaced_template_guidance_is_still_a_placeholder() {
        // The MADR template wraps its guidance in braces.
        assert!(Adr014::is_placeholder_content(
            "{Describe the context and problem statement, e.g., in free form using two to three sentences...}"
        ));
        assert!(Adr014::is_placeholder_content(
            "{Fill in the decision drivers here}"
        ));
        assert!(Adr014::is_placeholder_content(
            "{Add content describing the alternative}"
        ));
        // Braces removed but the template sentence left behind.
        assert!(Adr014::is_placeholder_content(
            "Describe the context and problem statement in two to three sentences."
        ));
    }

    #[test]
    fn test_unambiguous_stub_markers_still_match_anywhere() {
        // These do not occur in finished prose, so a section containing one is
        // still a stub even alongside real content.
        assert!(Adr014::is_placeholder_content(
            "We picked Postgres. TODO: write up the alternatives we rejected."
        ));
        assert!(Adr014::is_placeholder_content(
            "The retention window is to be determined."
        ));
    }

    #[test]
    fn test_no_false_positive_on_initial_adr() {
        let content = r#"# 1. Record architecture decisions

Date: 2024-01-15

## Status

Accepted

## Context

We need to record the architectural decisions made on this project.

## Decision

We will use Architecture Decision Records, as described by Michael Nygard in his article "Documenting Architecture Decisions".

## Consequences

See Michael Nygard's article, linked above. For a lightweight ADR toolset, see Nat Pryce's adr-tools.
"#;
        let doc = create_test_document(content);
        let rule = Adr014::default();
        let violations = rule.check(&doc).unwrap();
        assert!(
            violations.is_empty(),
            "False positive on initial ADR #0001: {violations:?}"
        );
    }
}
