//! ADR013: Valid ADR links
//!
//! Validates that links to other ADR documents point to existing files.
//! This rule analyzes all ADR documents in a collection.

use crate::adr::format::is_adr_document;
use mdbook_lint_core::rule::{CollectionRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::Severity;
use mdbook_lint_core::{Document, Result, Violation};
use regex::Regex;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::LazyLock;

/// Regex to find markdown links that look like ADR references
/// Matches: [text](path/to/adr/file.md) or [text](./adr/file.md) etc.
static ADR_LINK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]*)\]\(([^)]+\.md)\)").expect("Invalid regex"));

/// ADR013: Validates that ADR links point to existing documents
///
/// This collection rule checks all links within ADR documents that appear
/// to reference other ADR files, ensuring the target files exist.
pub struct Adr013;

impl Default for Adr013 {
    fn default() -> Self {
        Self
    }
}

impl Adr013 {
    /// Check if a path looks like an ADR reference
    fn is_adr_path(path: &str) -> bool {
        let path_lower = path.to_lowercase();
        path_lower.contains("adr/") || path_lower.contains("adr\\") || path_lower.contains("adrs/")
    }

    /// Resolve a relative link path from a source document
    fn resolve_link_path(source_path: &std::path::Path, link: &str) -> Option<PathBuf> {
        // Handle absolute paths (unlikely in markdown)
        if let Some(stripped) = link.strip_prefix('/') {
            return Some(PathBuf::from(stripped));
        }

        // Get the directory containing the source file
        let source_dir = source_path.parent()?;

        // Resolve the relative path
        let mut resolved = source_dir.to_path_buf();
        for component in link.split('/') {
            match component {
                "." => {}
                ".." => {
                    resolved.pop();
                }
                other => {
                    resolved.push(other);
                }
            }
        }

        Some(resolved)
    }

    /// Normalize path for comparison (handle case sensitivity, slashes, etc.)
    fn normalize_path(path: &std::path::Path) -> String {
        path.to_string_lossy().to_lowercase().replace('\\', "/")
    }
}

impl CollectionRule for Adr013 {
    fn id(&self) -> &'static str {
        "ADR013"
    }

    fn name(&self) -> &'static str {
        "adr-valid-adr-links"
    }

    fn description(&self) -> &'static str {
        "Links to other ADR documents should point to existing files"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Links).introduced_in("mdbook-lint v0.14.0")
    }

    fn check_collection(&self, documents: &[Document]) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // Build a set of known ADR paths for quick lookup
        let known_paths: HashSet<String> = documents
            .iter()
            .filter(|doc| is_adr_document(&doc.content, Some(&doc.path)))
            .map(|doc| Self::normalize_path(&doc.path))
            .collect();

        // Also include just the filename for relative references within same directory
        let known_filenames: HashSet<String> = documents
            .iter()
            .filter(|doc| is_adr_document(&doc.content, Some(&doc.path)))
            .filter_map(|doc| doc.path.file_name())
            .filter_map(|n| n.to_str())
            .map(|s| s.to_lowercase())
            .collect();

        for doc in documents {
            if !is_adr_document(&doc.content, Some(&doc.path)) {
                continue;
            }

            // Find all markdown links
            for (line_num, line) in doc.lines.iter().enumerate() {
                for caps in ADR_LINK_REGEX.captures_iter(line) {
                    let link_path = caps.get(2).map(|m| m.as_str()).unwrap_or("");

                    // Only check links that look like ADR references
                    if !Self::is_adr_path(link_path) {
                        // But also check for same-directory .md links in ADR directories
                        if !link_path.ends_with(".md") {
                            continue;
                        }
                        // If it's a simple filename in an ADR directory, check it
                        if link_path.contains('/') || link_path.contains('\\') {
                            continue;
                        }
                    }

                    // Skip external links and anchors
                    if link_path.starts_with("http://")
                        || link_path.starts_with("https://")
                        || link_path.starts_with('#')
                    {
                        continue;
                    }

                    // Resolve the link path relative to the source document
                    if let Some(resolved) = Self::resolve_link_path(&doc.path, link_path) {
                        let normalized = Self::normalize_path(&resolved);
                        let filename = resolved
                            .file_name()
                            .and_then(|n| n.to_str())
                            .map(|s| s.to_lowercase())
                            .unwrap_or_default();

                        // Check if the target exists in our collection
                        if !known_paths.contains(&normalized)
                            && !known_filenames.contains(&filename)
                        {
                            // Only report if the link looks like it should be an ADR
                            if Self::is_adr_path(link_path)
                                || (doc.path.to_string_lossy().contains("adr")
                                    && link_path.ends_with(".md"))
                            {
                                violations.push(self.create_violation_for_file(
                                    &doc.path,
                                    format!(
                                        "Link to '{}' references non-existent ADR file",
                                        link_path
                                    ),
                                    line_num + 1,
                                    1,
                                    Severity::Warning,
                                ));
                            }
                        }
                    }
                }
            }
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_adr_with_link(number: u32, link: &str) -> Document {
        let content = format!(
            r#"# {}. Use Rust

Date: 2024-01-15

## Status

Accepted. See [related decision]({}).

## Context

Context here.

## Decision

Decision here.

## Consequences

Consequences here.
"#,
            number, link
        );
        Document::new(
            content,
            PathBuf::from(format!("adr/{:04}-use-rust.md", number)),
        )
        .unwrap()
    }

    fn create_simple_adr(number: u32) -> Document {
        let content = format!(
            r#"# {}. Decision {}

Date: 2024-01-15

## Status

Accepted

## Context

Context here.

## Decision

Decision here.

## Consequences

Consequences here.
"#,
            number, number
        );
        Document::new(
            content,
            PathBuf::from(format!("adr/{:04}-decision-{}.md", number, number)),
        )
        .unwrap()
    }

    #[test]
    fn test_valid_link_to_existing_adr() {
        let docs = vec![
            create_adr_with_link(1, "0002-decision-2.md"),
            create_simple_adr(2),
        ];

        let rule = Adr013;
        let violations = rule.check_collection(&docs).unwrap();
        assert!(
            violations.is_empty(),
            "Link to existing ADR should be valid"
        );
    }

    #[test]
    fn test_invalid_link_to_nonexistent_adr() {
        let docs = vec![create_adr_with_link(1, "0099-nonexistent.md")];

        let rule = Adr013;
        let violations = rule.check_collection(&docs).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("non-existent"));
    }

    #[test]
    fn test_link_with_adr_path() {
        let docs = vec![
            create_adr_with_link(1, "../adr/0002-decision.md"),
            create_simple_adr(2),
        ];

        let rule = Adr013;
        let _violations = rule.check_collection(&docs).unwrap();
        // This test just verifies the rule doesn't panic on relative paths
        // The actual resolution behavior depends on the document paths
    }

    #[test]
    fn test_external_links_ignored() {
        let docs = vec![create_adr_with_link(1, "https://example.com/doc.md")];

        let rule = Adr013;
        let violations = rule.check_collection(&docs).unwrap();
        assert!(violations.is_empty(), "External links should be ignored");
    }

    #[test]
    fn test_anchor_links_ignored() {
        let docs = vec![create_adr_with_link(1, "#section")];

        let rule = Adr013;
        let violations = rule.check_collection(&docs).unwrap();
        assert!(violations.is_empty(), "Anchor links should be ignored");
    }

    #[test]
    fn test_empty_collection() {
        let docs: Vec<Document> = vec![];

        let rule = Adr013;
        let violations = rule.check_collection(&docs).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_is_adr_path() {
        assert!(Adr013::is_adr_path("adr/0001-test.md"));
        assert!(Adr013::is_adr_path("../adr/0001-test.md"));
        assert!(Adr013::is_adr_path("docs/adr/0001-test.md"));
        assert!(Adr013::is_adr_path("adrs/0001-test.md"));
        assert!(!Adr013::is_adr_path("docs/readme.md"));
        assert!(!Adr013::is_adr_path("other-doc.md"));
    }
}
