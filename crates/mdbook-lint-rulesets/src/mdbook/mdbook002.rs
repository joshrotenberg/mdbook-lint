//! MDBOOK002: Internal link validation
//!
//! This rule validates that internal links (relative paths) resolve to existing files.

use comrak::nodes::NodeValue;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};
use std::path::{Path, PathBuf};

/// Rule to check that internal links resolve to existing files
pub struct MDBOOK002;

impl AstRule for MDBOOK002 {
    fn id(&self) -> &'static str {
        "MDBOOK002"
    }

    fn name(&self) -> &'static str {
        "internal-link-validation"
    }

    fn description(&self) -> &'static str {
        "Internal links must resolve to existing files"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::MdBook).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_ast<'a>(
        &self,
        document: &Document,
        ast: &'a comrak::nodes::AstNode<'a>,
    ) -> mdbook_lint_core::error::Result<Vec<Violation>> {
        self.check_ast_nodes(document, ast)
    }
}

impl MDBOOK002 {
    /// Check AST nodes for internal link violations
    fn check_ast_nodes<'a>(
        &self,
        document: &Document,
        ast: &'a comrak::nodes::AstNode<'a>,
    ) -> mdbook_lint_core::error::Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // Walk through all nodes in the AST
        for node in ast.descendants() {
            if let NodeValue::Link(link) = &node.data.borrow().value {
                let url = &link.url;

                // Skip external links (http/https/mailto/etc)
                if is_external_link(url) {
                    continue;
                }

                // Skip anchor-only links (same document)
                if url.starts_with('#') {
                    continue;
                }

                // Check if the internal link resolves
                if let Some(violation) = validate_internal_link(document, node, url)? {
                    violations.push(violation);
                }
            }
        }

        Ok(violations)
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

/// Validate an internal link and return a violation if it doesn't resolve
fn validate_internal_link<'a>(
    document: &Document,
    node: &'a comrak::nodes::AstNode<'a>,
    url: &str,
) -> mdbook_lint_core::error::Result<Option<Violation>> {
    // Remove anchor fragment if present (e.g., "file.md#section" -> "file.md")
    let path_part = url.split('#').next().unwrap_or(url);

    // Skip empty paths
    if path_part.is_empty() {
        return Ok(None);
    }

    // Find the book's source directory (parent of SUMMARY.md)
    let book_src_dir = find_book_src_directory(&document.path);

    // Resolve the target path relative to the current document
    let target_path = resolve_link_path(&document.path, path_part, book_src_dir.as_deref());

    // Check if the target file exists
    if !target_path.exists() {
        let (line, column) = document.node_position(node).unwrap_or((1, 1));

        return Ok(Some(MDBOOK002.create_violation(
            format!("Internal link '{url}' does not resolve to an existing file"),
            line,
            column,
            Severity::Error,
        )));
    }

    Ok(None)
}

/// Find the book's source directory by looking for SUMMARY.md
fn find_book_src_directory(current_doc_path: &Path) -> Option<PathBuf> {
    let mut current = current_doc_path.parent();

    while let Some(dir) = current {
        // Check if SUMMARY.md exists in this directory
        if dir.join("SUMMARY.md").exists() {
            return Some(dir.to_path_buf());
        }
        // Move up one directory
        current = dir.parent();
    }

    None
}

/// Resolve a link path relative to the current document
fn resolve_link_path(
    current_doc_path: &Path,
    link_path: &str,
    book_src_dir: Option<&Path>,
) -> PathBuf {
    let current_dir = current_doc_path.parent().unwrap_or(Path::new("."));

    // Handle different path formats
    if let Some(stripped) = link_path.strip_prefix("./") {
        // Explicit relative path: ./file.md
        current_dir.join(stripped)
    } else if link_path.starts_with("../") {
        // Parent directory path: ../file.md
        current_dir.join(link_path)
    } else if let Some(stripped) = link_path.strip_prefix('/') {
        // Absolute path (relative to book source directory)
        if let Some(src_dir) = book_src_dir {
            src_dir.join(stripped)
        } else {
            // Fallback: if we can't find the book source, resolve relative to current dir
            current_dir.join(stripped)
        }
    } else {
        // Implicit relative path: file.md
        current_dir.join(link_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::rule::Rule;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_document(
        content: &str,
        file_name: &str,
        temp_dir: &TempDir,
    ) -> mdbook_lint_core::error::Result<Document> {
        let file_path = temp_dir.path().join(file_name);
        fs::write(&file_path, content)?;
        Document::new(content.to_string(), file_path)
    }

    #[test]
    fn test_mdbook002_valid_links() -> mdbook_lint_core::error::Result<()> {
        let temp_dir = TempDir::new()?;

        // Create target files
        fs::write(temp_dir.path().join("target.md"), "# Target")?;
        fs::create_dir_all(temp_dir.path().join("subdir"))?;
        fs::write(temp_dir.path().join("subdir/other.md"), "# Other")?;

        let content = r#"# Test Document

[Valid relative link](./target.md)
[Valid implicit link](target.md)
[Valid subdirectory link](subdir/other.md)
[Valid external link](https://example.com)
[Valid anchor link](#section)
"#;

        let document = create_test_document(content, "test.md", &temp_dir)?;
        let rule = MDBOOK002;
        let violations = rule.check(&document)?;

        assert_eq!(violations.len(), 0);
        Ok(())
    }

    #[test]
    fn test_mdbook002_invalid_links() -> mdbook_lint_core::error::Result<()> {
        let temp_dir = TempDir::new()?;

        let content = r#"# Test Document

[Invalid link](./nonexistent.md)
[Another invalid link](missing/file.md)
[Valid external link](https://example.com)
"#;

        let document = create_test_document(content, "test.md", &temp_dir)?;
        let rule = MDBOOK002;
        let violations = rule.check(&document)?;

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].rule_id, "MDBOOK002");
        assert!(violations[0].message.contains("nonexistent.md"));
        assert_eq!(violations[1].rule_id, "MDBOOK002");
        assert!(violations[1].message.contains("missing/file.md"));
        Ok(())
    }

    #[test]
    fn test_mdbook002_links_with_anchors() -> mdbook_lint_core::error::Result<()> {
        let temp_dir = TempDir::new()?;

        // Create target file
        fs::write(temp_dir.path().join("target.md"), "# Target")?;

        let content = r#"# Test Document

[Valid link with anchor](./target.md#section)
[Invalid link with anchor](./nonexistent.md#section)
"#;

        let document = create_test_document(content, "test.md", &temp_dir)?;
        let rule = MDBOOK002;
        let violations = rule.check(&document)?;

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("nonexistent.md#section"));
        Ok(())
    }

    #[test]
    fn test_is_external_link() {
        assert!(is_external_link("https://example.com"));
        assert!(is_external_link("http://example.com"));
        assert!(is_external_link("mailto:test@example.com"));
        assert!(is_external_link("ftp://files.example.com"));
        assert!(is_external_link("tel:+1234567890"));

        assert!(!is_external_link("./local.md"));
        assert!(!is_external_link("../parent.md"));
        assert!(!is_external_link("file.md"));
        assert!(!is_external_link("#anchor"));
    }

    #[test]
    fn test_resolve_link_path() {
        let current_path = PathBuf::from("/project/src/chapter.md");
        let book_src = PathBuf::from("/project/src");

        assert_eq!(
            resolve_link_path(&current_path, "./other.md", None),
            PathBuf::from("/project/src/other.md")
        );

        assert_eq!(
            resolve_link_path(&current_path, "../README.md", None),
            PathBuf::from("/project/src/../README.md")
        );

        assert_eq!(
            resolve_link_path(&current_path, "other.md", None),
            PathBuf::from("/project/src/other.md")
        );

        assert_eq!(
            resolve_link_path(&current_path, "subdir/file.md", None),
            PathBuf::from("/project/src/subdir/file.md")
        );

        // Test absolute paths with book source directory
        assert_eq!(
            resolve_link_path(&current_path, "/README.md", Some(&book_src)),
            PathBuf::from("/project/src/README.md")
        );

        assert_eq!(
            resolve_link_path(&current_path, "/subdir/file.md", Some(&book_src)),
            PathBuf::from("/project/src/subdir/file.md")
        );

        // Test absolute paths without book source (fallback)
        assert_eq!(
            resolve_link_path(&current_path, "/file.md", None),
            PathBuf::from("/project/src/file.md")
        );
    }

    #[test]
    fn test_find_book_src_directory() -> mdbook_lint_core::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir)?;

        // Create SUMMARY.md in src directory
        fs::write(src_dir.join("SUMMARY.md"), "# Summary")?;

        // Create a chapter file
        let chapter_path = src_dir.join("chapter1").join("section.md");
        fs::create_dir_all(chapter_path.parent().unwrap())?;
        fs::write(&chapter_path, "# Section")?;

        // Test finding book source from nested chapter
        let found = find_book_src_directory(&chapter_path);
        assert_eq!(found, Some(src_dir.clone()));

        // Test from file in same directory as SUMMARY.md
        let root_file = src_dir.join("README.md");
        fs::write(&root_file, "# README")?;
        let found = find_book_src_directory(&root_file);
        assert_eq!(found, Some(src_dir.clone()));

        // Test when no SUMMARY.md exists
        let other_dir = temp_dir.path().join("other");
        fs::create_dir_all(&other_dir)?;
        let other_file = other_dir.join("file.md");
        fs::write(&other_file, "# File")?;
        let found = find_book_src_directory(&other_file);
        assert_eq!(found, None);

        Ok(())
    }

    #[test]
    fn test_mdbook002_absolute_paths_with_book_structure() -> mdbook_lint_core::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir)?;

        // Create book structure
        fs::write(
            src_dir.join("SUMMARY.md"),
            "# Summary\n- [Chapter 1](chapter1/intro.md)",
        )?;
        fs::write(src_dir.join("README.md"), "# Book")?;

        // Create subdirectory structure
        let chapter_dir = src_dir.join("chapter1");
        fs::create_dir_all(&chapter_dir)?;
        fs::write(chapter_dir.join("intro.md"), "# Intro")?;
        fs::write(chapter_dir.join("section.md"), "# Section")?;

        // Create a document with absolute links
        let content = r#"# Test Document

[Link to README](/README.md)
[Link to intro](/chapter1/intro.md)
[Link to section](/chapter1/section.md)
[Invalid absolute link](/nonexistent.md)
"#;

        let doc_path = chapter_dir.join("test.md");
        fs::write(&doc_path, content)?;
        let document = Document::new(content.to_string(), doc_path)?;

        let rule = MDBOOK002;
        let violations = rule.check(&document)?;

        // Should only have one violation for the nonexistent file
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("/nonexistent.md"));

        Ok(())
    }

    #[test]
    fn test_mdbook002_relative_paths_from_nested_chapter() -> mdbook_lint_core::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir)?;

        // Create book structure
        fs::write(src_dir.join("SUMMARY.md"), "# Summary")?;
        let chapter1_dir = src_dir.join("chapter1");
        let chapter2_dir = src_dir.join("chapter2");
        fs::create_dir_all(&chapter1_dir)?;
        fs::create_dir_all(&chapter2_dir)?;

        // Create target files
        fs::write(chapter1_dir.join("intro.md"), "# Intro")?;
        fs::write(chapter2_dir.join("other.md"), "# Other")?;
        fs::write(src_dir.join("README.md"), "# README")?;

        // Test from a nested file
        let content = r#"# Nested Document

[Same directory](./intro.md)
[Parent directory](../README.md)
[Sibling directory](../chapter2/other.md)
[Implicit relative](intro.md)
[Invalid relative](./missing.md)
[Invalid parent](../../outside.md)
"#;

        let doc_path = chapter1_dir.join("test.md");
        fs::write(&doc_path, content)?;
        let document = Document::new(content.to_string(), doc_path)?;

        let rule = MDBOOK002;
        let violations = rule.check(&document)?;

        // Should have violations for missing files only
        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("./missing.md"));
        assert!(violations[1].message.contains("../../outside.md"));

        Ok(())
    }

    #[test]
    fn test_mdbook002_mixed_link_types() -> mdbook_lint_core::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir)?;

        // Create book structure
        fs::write(src_dir.join("SUMMARY.md"), "# Summary")?;
        fs::write(src_dir.join("README.md"), "# README")?;
        fs::write(src_dir.join("glossary.md"), "# Glossary")?;

        let content = r#"# Mixed Links

[External HTTP](https://example.com)
[External HTTPS](https://example.com)
[Mailto](mailto:test@example.com)
[Anchor only](#section)
[Absolute path](/README.md)
[Relative with anchor](./glossary.md#term)
[Invalid absolute](/missing.md#section)
[Invalid relative](./nonexistent.md)
"#;

        let doc_path = src_dir.join("test.md");
        fs::write(&doc_path, content)?;
        let document = Document::new(content.to_string(), doc_path)?;

        let rule = MDBOOK002;
        let violations = rule.check(&document)?;

        // Should only report invalid internal links
        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("/missing.md#section"));
        assert!(violations[1].message.contains("./nonexistent.md"));

        Ok(())
    }
}
