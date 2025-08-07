//! MDBOOK006: Validate internal cross-reference links between chapters
//!
//! This rule validates anchor fragments in internal links, ensuring they point to valid headings
//! in target files. It complements MDBOOK002 by focusing on the anchor validation that MDBOOK002 skips.

use crate::rule::{AstRule, RuleCategory, RuleMetadata};
use crate::{
    Document,
    violation::{Severity, Violation},
};
use comrak::nodes::{AstNode, NodeValue};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::{fs, io};

/// MDBOOK006: Validate internal cross-reference links between chapters
///
/// This rule validates that internal links with anchor fragments point to valid headings
/// in the target files. It focuses specifically on cross-reference validation between
/// chapters, ensuring that `[text](file.md#heading)` links work correctly.
///
/// The rule:
/// - Only processes internal links with anchor fragments (e.g., `file.md#section`)
/// - Resolves target files relative to the current document
/// - Parses target files to extract heading anchors
/// - Validates that the anchor fragment exists in the target file
/// - Supports configurable anchor ID generation strategies
/// - Caches parsed files to improve performance on large books
///
/// Anchor ID Generation:
/// - Converts heading text to lowercase
/// - Replaces spaces and non-alphanumeric characters with hyphens
/// - Removes leading/trailing hyphens and consecutive hyphens
/// - Handles Unicode characters appropriately
#[derive(Default)]
pub struct MDBOOK006 {
    /// Cache of parsed heading anchors by file path to avoid re-parsing
    anchor_cache: Arc<RwLock<HashMap<PathBuf, Vec<String>>>>,
}

impl AstRule for MDBOOK006 {
    fn id(&self) -> &'static str {
        "MDBOOK006"
    }

    fn name(&self) -> &'static str {
        "internal-cross-references"
    }

    fn description(&self) -> &'static str {
        "Internal cross-reference links must point to valid headings in target files"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::MdBook).introduced_in("mdbook-lint v0.2.0")
    }

    fn check_ast<'a>(
        &self,
        document: &Document,
        ast: &'a AstNode<'a>,
    ) -> crate::error::Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // Walk through all nodes in the AST
        for node in ast.descendants() {
            if let NodeValue::Link(link) = &node.data.borrow().value {
                let url = &link.url;

                // Skip external links
                if is_external_link(url) {
                    continue;
                }

                // Only process links with anchor fragments
                if !url.contains('#') {
                    continue;
                }

                // Skip same-document anchors (start with #)
                if url.starts_with('#') {
                    continue;
                }

                // Validate the cross-reference link
                if let Some(violation) = self.validate_cross_reference(document, node, url)? {
                    violations.push(violation);
                }
            }
        }

        Ok(violations)
    }
}

impl MDBOOK006 {
    /// Validate a cross-reference link with anchor fragment
    fn validate_cross_reference<'a>(
        &self,
        document: &Document,
        node: &'a AstNode<'a>,
        url: &str,
    ) -> crate::error::Result<Option<Violation>> {
        // Split URL into file path and anchor
        let parts: Vec<&str> = url.splitn(2, '#').collect();
        if parts.len() != 2 {
            return Ok(None); // No anchor fragment
        }

        let file_path = parts[0];
        let anchor = parts[1];

        // Skip empty file paths or anchors
        if file_path.is_empty() || anchor.is_empty() {
            return Ok(None);
        }

        // Resolve the target file path relative to current document
        let target_path = self.resolve_target_path(&document.path, file_path);

        // Check if target file exists
        if !target_path.exists() {
            // File doesn't exist - this should be caught by MDBOOK002, so we skip it
            return Ok(None);
        }

        // Get anchors from the target file
        let anchors = match self.get_file_anchors(&target_path)? {
            Some(anchors) => anchors,
            None => return Ok(None), // Couldn't parse file
        };

        // Check if the anchor exists in the target file
        if !anchors.contains(&anchor.to_string()) {
            let (line, column) = document.node_position(node).unwrap_or((1, 1));

            // Create helpful suggestion
            let suggestion = self.suggest_similar_anchor(anchor, &anchors);
            let message = if let Some(suggestion) = suggestion {
                format!(
                    "Cross-reference anchor '{anchor}' not found in '{file_path}'. Did you mean '{suggestion}'?"
                )
            } else {
                format!(
                    "Cross-reference anchor '{}' not found in '{}'. Available anchors: {}",
                    anchor,
                    file_path,
                    if anchors.is_empty() {
                        "none".to_string()
                    } else {
                        anchors
                            .iter()
                            .take(5)
                            .map(|s| format!("'{s}'"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                )
            };

            return Ok(Some(self.create_violation(
                message,
                line,
                column,
                Severity::Error,
            )));
        }

        Ok(None)
    }

    /// Resolve target file path relative to current document
    fn resolve_target_path(&self, current_doc_path: &Path, link_path: &str) -> PathBuf {
        let current_dir = current_doc_path.parent().unwrap_or(Path::new("."));

        if let Some(stripped) = link_path.strip_prefix("./") {
            // Explicit relative path: ./file.md
            current_dir.join(stripped)
        } else if link_path.starts_with("../") {
            // Parent directory path: ../file.md
            current_dir.join(link_path)
        } else if let Some(stripped) = link_path.strip_prefix('/') {
            // Absolute path (relative to project root)
            PathBuf::from(stripped)
        } else {
            // Implicit relative path: file.md
            current_dir.join(link_path)
        }
    }

    /// Get all heading anchors from a markdown file (with caching)
    fn get_file_anchors(&self, file_path: &Path) -> io::Result<Option<Vec<String>>> {
        let canonical_path = match file_path.canonicalize() {
            Ok(path) => path,
            Err(_) => file_path.to_path_buf(),
        };

        // Check cache first
        {
            if let Ok(cache) = self.anchor_cache.read()
                && let Some(anchors) = cache.get(&canonical_path) {
                return Ok(Some(anchors.clone()));
            }
        }

        // Read and parse the file
        let content = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(_) => return Ok(None), // File couldn't be read
        };

        let anchors = self.extract_heading_anchors(&content);

        // Cache the result
        {
            if let Ok(mut cache) = self.anchor_cache.write() {
                cache.insert(canonical_path, anchors.clone());
            }
        }

        Ok(Some(anchors))
    }

    /// Extract heading anchors from markdown content
    fn extract_heading_anchors(&self, content: &str) -> Vec<String> {
        let mut anchors = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            // Match ATX headings (# ## ### etc)
            if let Some(heading_text) = self.extract_atx_heading(line) {
                let anchor = self.generate_anchor_id(&heading_text);
                if !anchor.is_empty() {
                    anchors.push(anchor);
                }
            }
        }

        // TODO: Handle Setext headings (underlined with = or -)
        // This is less common in mdBook but could be added for completeness

        anchors
    }

    /// Extract heading text from ATX heading line
    fn extract_atx_heading(&self, line: &str) -> Option<String> {
        if !line.starts_with('#') {
            return None;
        }

        // Count leading hashes
        let hash_count = line.chars().take_while(|&c| c == '#').count();
        if hash_count == 0 || hash_count > 6 {
            return None; // Invalid heading level
        }

        // Extract text after hashes
        let rest = &line[hash_count..];
        let text = if let Some(stripped) = rest.strip_prefix(' ') {
            stripped
        } else {
            rest
        };

        // Remove trailing hashes if present (closed ATX style)
        let text = text.trim_end_matches(['#', ' ']);

        if text.is_empty() {
            return None;
        }

        Some(text.to_string())
    }

    /// Generate anchor ID from heading text (following common markdown conventions)
    fn generate_anchor_id(&self, heading_text: &str) -> String {
        heading_text
            .to_lowercase()
            // Replace whitespace and non-alphanumeric with hyphens
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            // Remove consecutive hyphens
            .split('-')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Suggest similar anchor that might be what the user intended
    fn suggest_similar_anchor(&self, target: &str, available: &[String]) -> Option<String> {
        if available.is_empty() {
            return None;
        }

        // Simple similarity: find anchor that contains target or vice versa
        for anchor in available {
            if anchor.contains(target) || target.contains(anchor) {
                return Some(anchor.clone());
            }
        }

        // If no substring match, return the first available anchor as a suggestion
        Some(available[0].clone())
    }
}

/// Check if a URL is an external link
fn is_external_link(url: &str) -> bool {
    url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("mailto:")
        || url.starts_with("ftp://")
        || url.starts_with("tel:")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::Rule;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_document(content: &str, file_path: &Path) -> crate::error::Result<Document> {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file_path, content)?;
        Document::new(content.to_string(), file_path.to_path_buf())
    }

    #[test]
    fn test_mdbook006_valid_cross_references() -> crate::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create target file with headings
        let target_content = r#"# Chapter 2

## Overview

Some content here.

### Implementation Details

More details.
"#;
        create_test_document(target_content, &root.join("chapter2.md"))?;

        // Create source file with links to target
        let source_content = r#"# Chapter 1

See [Chapter 2](chapter2.md#chapter-2) for more info.

Check out the [overview](chapter2.md#overview) section.

The [implementation](chapter2.md#implementation-details) is complex.
"#;
        let source_path = root.join("chapter1.md");
        let doc = create_test_document(source_content, &source_path)?;

        let rule = MDBOOK006::default();
        let violations = rule.check(&doc)?;

        assert_eq!(
            violations.len(),
            0,
            "Valid cross-references should have no violations"
        );
        Ok(())
    }

    #[test]
    fn test_mdbook006_invalid_anchor() -> crate::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create target file with headings
        let target_content = r#"# Chapter 2

## Overview

Some content.
"#;
        create_test_document(target_content, &root.join("chapter2.md"))?;

        // Create source file with invalid anchor
        let source_content = r#"# Chapter 1

See [nonexistent section](chapter2.md#nonexistent).
"#;
        let source_path = root.join("chapter1.md");
        let doc = create_test_document(source_content, &source_path)?;

        let rule = MDBOOK006::default();
        let violations = rule.check(&doc)?;

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MDBOOK006");
        assert!(
            violations[0]
                .message
                .contains("anchor 'nonexistent' not found")
        );
        assert!(violations[0].message.contains("chapter2.md"));
        Ok(())
    }

    #[test]
    fn test_mdbook006_missing_target_file() -> crate::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create source file linking to nonexistent file
        let source_content = r#"# Chapter 1

See [missing](nonexistent.md#section).
"#;
        let source_path = root.join("chapter1.md");
        let doc = create_test_document(source_content, &source_path)?;

        let rule = MDBOOK006::default();
        let violations = rule.check(&doc)?;

        // Should not report violations for missing files (MDBOOK002's job)
        assert_eq!(violations.len(), 0);
        Ok(())
    }

    #[test]
    fn test_mdbook006_same_document_anchors() -> crate::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create file with internal anchor link
        let content = r#"# Chapter 1

## Section A

See [Section B](#section-b) below.

## Section B

Content here.
"#;
        let file_path = root.join("chapter1.md");
        let doc = create_test_document(content, &file_path)?;

        let rule = MDBOOK006::default();
        let violations = rule.check(&doc)?;

        // Should not process same-document anchors
        assert_eq!(violations.len(), 0);
        Ok(())
    }

    #[test]
    fn test_mdbook006_external_links() -> crate::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create file with external links
        let content = r#"# Chapter 1

See [external](https://example.com#section).
"#;
        let file_path = root.join("chapter1.md");
        let doc = create_test_document(content, &file_path)?;

        let rule = MDBOOK006::default();
        let violations = rule.check(&doc)?;

        // Should ignore external links
        assert_eq!(violations.len(), 0);
        Ok(())
    }

    #[test]
    fn test_mdbook006_no_anchor_links() -> crate::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create target file
        create_test_document("# Target", &root.join("target.md"))?;

        // Create file with links without anchors
        let content = r#"# Chapter 1

See [target](target.md) for more.
"#;
        let file_path = root.join("chapter1.md");
        let doc = create_test_document(content, &file_path)?;

        let rule = MDBOOK006::default();
        let violations = rule.check(&doc)?;

        // Should ignore links without anchors
        assert_eq!(violations.len(), 0);
        Ok(())
    }

    #[test]
    fn test_extract_atx_heading() {
        let rule = MDBOOK006::default();

        assert_eq!(
            rule.extract_atx_heading("# Heading"),
            Some("Heading".to_string())
        );
        assert_eq!(
            rule.extract_atx_heading("## Sub Heading"),
            Some("Sub Heading".to_string())
        );
        assert_eq!(
            rule.extract_atx_heading("### Deep Heading ###"),
            Some("Deep Heading".to_string())
        );
        assert_eq!(
            rule.extract_atx_heading("#No Space"),
            Some("No Space".to_string())
        );

        // Invalid cases
        assert_eq!(rule.extract_atx_heading("Not a heading"), None);
        assert_eq!(rule.extract_atx_heading(""), None);
        assert_eq!(rule.extract_atx_heading("#"), None);
        assert_eq!(rule.extract_atx_heading("# "), None);
    }

    #[test]
    fn test_generate_anchor_id() {
        let rule = MDBOOK006::default();

        assert_eq!(rule.generate_anchor_id("Simple Heading"), "simple-heading");
        assert_eq!(
            rule.generate_anchor_id("Complex: Heading with! Punctuation?"),
            "complex-heading-with-punctuation"
        );
        assert_eq!(
            rule.generate_anchor_id("Multiple   Spaces"),
            "multiple-spaces"
        );
        assert_eq!(rule.generate_anchor_id("UPPER case"), "upper-case");
        assert_eq!(rule.generate_anchor_id("123 Numbers"), "123-numbers");
        assert_eq!(rule.generate_anchor_id(""), "");
    }

    #[test]
    fn test_mdbook006_nested_directories() -> crate::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create nested target file
        let target_content = r#"# Deep Chapter

## Nested Section

Content here.
"#;
        create_test_document(target_content, &root.join("guide/deep.md"))?;

        // Create source file with relative link
        let source_content = r#"# Main Chapter

See [nested section](guide/deep.md#nested-section).
"#;
        let source_path = root.join("chapter.md");
        let doc = create_test_document(source_content, &source_path)?;

        let rule = MDBOOK006::default();
        let violations = rule.check(&doc)?;

        assert_eq!(
            violations.len(),
            0,
            "Nested directory cross-references should work"
        );
        Ok(())
    }

    #[test]
    fn test_mdbook006_helpful_suggestions() -> crate::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create target file with similar heading
        let target_content = r#"# Target

## Implementation Details

Content here.
"#;
        create_test_document(target_content, &root.join("target.md"))?;

        // Create source file with similar but wrong anchor
        let source_content = r#"# Source

See [details](target.md#implementation).
"#;
        let source_path = root.join("source.md");
        let doc = create_test_document(source_content, &source_path)?;

        let rule = MDBOOK006::default();
        let violations = rule.check(&doc)?;

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Did you mean"));
        assert!(violations[0].message.contains("implementation-details"));
        Ok(())
    }
}
