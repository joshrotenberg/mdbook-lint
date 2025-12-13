//! CONTENT006: No broken internal links
//!
//! Ensures that internal anchor links (e.g., `[link](#section-name)`) point to
//! valid headings within the same document.

use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

/// Regex to match ATX headings and capture the text
static HEADING_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^#{1,6}\s+(.+?)(?:\s*#*)?$").unwrap());

/// Regex to match internal anchor links [text](#anchor)
static INTERNAL_LINK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]*)\]\(#([^)]+)\)").unwrap());

/// CONTENT006: Detects broken internal anchor links
///
/// This rule checks that all internal anchor links (`#anchor-name`) reference
/// valid headings in the document. Broken internal links lead to poor UX.
#[derive(Default, Clone)]
pub struct CONTENT006;

impl CONTENT006 {
    /// Generate an anchor ID from heading text (mdBook-style)
    ///
    /// mdBook's slug generation algorithm:
    /// - Convert to lowercase
    /// - Replace non-alphanumeric characters (except hyphens and underscores) with hyphens
    /// - Preserve underscores as-is (important for code identifiers like `a_title`)
    /// - Do NOT consolidate multiple consecutive hyphens (mdBook preserves them)
    /// - Remove leading/trailing hyphens
    fn generate_anchor(heading_text: &str) -> String {
        let mut anchor = String::new();
        let text = heading_text.trim();

        for ch in text.chars() {
            if ch.is_alphanumeric() {
                anchor.push(ch.to_ascii_lowercase());
            } else if ch == '-' || ch == '_' {
                // Preserve hyphens and underscores as-is
                anchor.push(ch);
            } else {
                // Replace other characters (spaces, punctuation, backticks) with hyphens
                anchor.push('-');
            }
        }

        // Remove leading/trailing hyphens only
        // Do NOT consolidate multiple consecutive hyphens - mdBook preserves them
        while anchor.ends_with('-') {
            anchor.pop();
        }
        while anchor.starts_with('-') {
            anchor.remove(0);
        }

        anchor
    }

    /// Extract all valid anchors from the document (from headings)
    fn extract_anchors(&self, document: &Document) -> HashSet<String> {
        let mut anchors = HashSet::new();
        let mut in_code_block = false;

        for line in &document.lines {
            let trimmed = line.trim();

            // Track code blocks
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }

            if in_code_block {
                continue;
            }

            // Extract heading text and generate anchor
            if let Some(caps) = HEADING_REGEX.captures(trimmed)
                && let Some(heading_text) = caps.get(1)
            {
                let anchor = Self::generate_anchor(heading_text.as_str());
                if !anchor.is_empty() {
                    anchors.insert(anchor);
                }
            }
        }

        anchors
    }

    /// Find suggestions for a broken anchor
    fn find_similar_anchor<'a>(
        &self,
        broken_anchor: &str,
        valid_anchors: &'a HashSet<String>,
    ) -> Option<&'a String> {
        let broken_lower = broken_anchor.to_lowercase();

        // First try exact case-insensitive match
        for anchor in valid_anchors {
            if anchor.to_lowercase() == broken_lower {
                return Some(anchor);
            }
        }

        // Try to find anchors that contain the broken one or vice versa
        valid_anchors
            .iter()
            .find(|anchor| anchor.contains(&broken_lower) || broken_lower.contains(anchor.as_str()))
    }
}

impl Rule for CONTENT006 {
    fn id(&self) -> &'static str {
        "CONTENT006"
    }

    fn name(&self) -> &'static str {
        "no-broken-internal-links"
    }

    fn description(&self) -> &'static str {
        "Internal anchor links should reference valid headings in the document"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Content).introduced_in("mdbook-lint v0.14.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> mdbook_lint_core::error::Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let valid_anchors = self.extract_anchors(document);
        let mut in_code_block = false;

        for (line_idx, line) in document.lines.iter().enumerate() {
            let trimmed = line.trim();

            // Track code blocks
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }

            if in_code_block {
                continue;
            }

            // Find all internal links in the line
            for caps in INTERNAL_LINK_REGEX.captures_iter(line) {
                let anchor = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                let link_text = caps.get(1).map(|m| m.as_str()).unwrap_or("");

                // Check if this anchor exists
                if !valid_anchors.contains(anchor) {
                    let line_num = line_idx + 1;
                    let col = caps.get(0).map(|m| m.start() + 1).unwrap_or(1);

                    let mut message = format!(
                        "Broken internal link: '#{anchor}' does not match any heading in this document"
                    );

                    // Add suggestion if we find a similar anchor
                    if let Some(suggestion) = self.find_similar_anchor(anchor, &valid_anchors) {
                        message.push_str(&format!(". Did you mean '#{suggestion}'?"));
                    }

                    // Add context about what the link text was
                    if !link_text.is_empty() {
                        message.push_str(&format!(" (link text: '{link_text}')"));
                    }

                    violations.push(self.create_violation(
                        message,
                        line_num,
                        col,
                        Severity::Warning,
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
        Document::new(content.to_string(), PathBuf::from("test.md")).unwrap()
    }

    #[test]
    fn test_generate_anchor() {
        assert_eq!(CONTENT006::generate_anchor("Hello World"), "hello-world");
        assert_eq!(
            CONTENT006::generate_anchor("Getting Started"),
            "getting-started"
        );
        // mdBook preserves punctuation as hyphens, doesn't consolidate
        assert_eq!(CONTENT006::generate_anchor("What's New?"), "what-s-new");
        assert_eq!(
            CONTENT006::generate_anchor("Section 1.2.3"),
            "section-1-2-3"
        );
        assert_eq!(
            CONTENT006::generate_anchor("C++ Programming"),
            "c---programming"
        );
        assert_eq!(CONTENT006::generate_anchor("  Spaces  "), "spaces");
        // Underscores are preserved
        assert_eq!(CONTENT006::generate_anchor("a_title"), "a_title");
        // Multiple spaces become multiple hyphens
        assert_eq!(CONTENT006::generate_anchor("a  b"), "a--b");
    }

    #[test]
    fn test_valid_internal_link() {
        let content = "# Getting Started

See [the introduction](#getting-started) for more info.

## Installation

Check [installation](#installation) instructions.";
        let doc = create_test_document(content);
        let rule = CONTENT006;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_broken_internal_link() {
        let content = "# Getting Started

See [broken link](#nonexistent-section) for more info.";
        let doc = create_test_document(content);
        let rule = CONTENT006;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("nonexistent-section"));
    }

    #[test]
    fn test_multiple_broken_links() {
        let content = "# Title

[link1](#bad1) and [link2](#bad2)";
        let doc = create_test_document(content);
        let rule = CONTENT006;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_case_sensitive_anchors() {
        let content = "# Hello World

[link](#Hello-World)";
        let doc = create_test_document(content);
        let rule = CONTENT006;
        let violations = rule.check(&doc).unwrap();
        // Anchor is hello-world (lowercase), but link uses Hello-World
        assert_eq!(violations.len(), 1);
        // Should suggest the correct anchor
        assert!(violations[0].message.contains("hello-world"));
    }

    #[test]
    fn test_links_in_code_blocks_ignored() {
        let content = "# Title

```markdown
[example](#nonexistent)
```";
        let doc = create_test_document(content);
        let rule = CONTENT006;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_external_links_ignored() {
        let content = "# Title

[external](https://example.com)
[relative](./other-file.md)
[with anchor](./other-file.md#section)";
        let doc = create_test_document(content);
        let rule = CONTENT006;
        let violations = rule.check(&doc).unwrap();
        // Only internal anchor links (#...) are checked
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_special_characters_in_heading() {
        // mdBook converts apostrophe and ? to hyphens
        let content = "# What's New in 2024?

[link](#what-s-new-in-2024)";
        let doc = create_test_document(content);
        let rule = CONTENT006;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_heading_with_code() {
        // mdBook converts backticks to hyphens
        let content = "# The `main` Function

[link](#the--main--function)";
        let doc = create_test_document(content);
        let rule = CONTENT006;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_nested_headings() {
        // mdBook converts dots to hyphens
        let content = "# Chapter 1

## Section 1.1

### Subsection 1.1.1

[to chapter](#chapter-1)
[to section](#section-1-1)
[to subsection](#subsection-1-1-1)";
        let doc = create_test_document(content);
        let rule = CONTENT006;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_suggestion_for_partial_match() {
        let content = "# Installation Guide

See [instructions](#installation) for details.";
        let doc = create_test_document(content);
        let rule = CONTENT006;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        // Should suggest the correct anchor (installation is contained in installation-guide)
        assert!(violations[0].message.contains("installation-guide"));
    }

    #[test]
    fn test_empty_link_text() {
        let content = "# Title

[](#nonexistent)";
        let doc = create_test_document(content);
        let rule = CONTENT006;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        // Should not include "link text:" for empty text
        assert!(!violations[0].message.contains("link text:"));
    }

    #[test]
    fn test_headings_in_code_blocks_not_anchors() {
        let content = "# Real Heading

```markdown
# Fake Heading
```

[link](#fake-heading)";
        let doc = create_test_document(content);
        let rule = CONTENT006;
        let violations = rule.check(&doc).unwrap();
        // #fake-heading is not a valid anchor (heading is in code block)
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_issue_323_underscore_in_code() {
        // Issue #323: ## `a_title` should preserve the underscore in slug
        // Backticks become hyphens which get trimmed from start/end
        let content = "## `a_title`

[somewhere](#a_title)";
        let doc = create_test_document(content);
        let rule = CONTENT006;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_underscores_preserved() {
        let content = "# my_function_name

[link](#my_function_name)";
        let doc = create_test_document(content);
        let rule = CONTENT006;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_double_dashes_preserved() {
        // mdBook preserves multiple consecutive hyphens
        let content = "# Title  With  Spaces

[link](#title--with--spaces)";
        let doc = create_test_document(content);
        let rule = CONTENT006;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }
}
