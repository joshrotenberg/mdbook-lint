//! MDBOOK002: Internal link validation
//!
//! This rule validates that internal links (relative paths) resolve to existing files.

use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};
use comrak::nodes::NodeValue;
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

    // Resolve the target path relative to the current document
    let target_path = resolve_link_path(&document.path, path_part);

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

/// Resolve a link path relative to the current document
fn resolve_link_path(current_doc_path: &Path, link_path: &str) -> PathBuf {
    let current_dir = current_doc_path.parent().unwrap_or(Path::new("."));

    // Handle different path formats
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

        assert_eq!(
            resolve_link_path(&current_path, "./other.md"),
            PathBuf::from("/project/src/other.md")
        );

        assert_eq!(
            resolve_link_path(&current_path, "../README.md"),
            PathBuf::from("/project/src/../README.md")
        );

        assert_eq!(
            resolve_link_path(&current_path, "other.md"),
            PathBuf::from("/project/src/other.md")
        );

        assert_eq!(
            resolve_link_path(&current_path, "subdir/file.md"),
            PathBuf::from("/project/src/subdir/file.md")
        );
    }
}
