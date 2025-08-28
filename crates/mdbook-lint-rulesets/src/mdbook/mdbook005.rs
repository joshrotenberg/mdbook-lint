//! MDBOOK005: Detect orphaned markdown files not referenced in SUMMARY.md
//!
//! This rule finds markdown files in the book's source directory that are not referenced
//! in SUMMARY.md. Only scans within the directory containing SUMMARY.md, not parent or
//! sibling directories. Orphaned files can indicate incomplete documentation or forgotten content.

use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::{fs, io};

/// MDBOOK005: Detect orphaned markdown files not referenced in SUMMARY.md
///
/// This rule checks for markdown files in the book's source directory that are not
/// referenced in SUMMARY.md. Such files are "orphaned" and won't be included in the
/// generated book, which may indicate:
/// - Incomplete documentation structure
/// - Forgotten content that should be added to the book
/// - Old files that should be removed
///
/// The rule:
/// - Only runs on SUMMARY.md files
/// - Parses all chapter references in SUMMARY.md
/// - Scans for .md and .markdown files ONLY in the book's source directory
/// - Does NOT scan parent directories or sibling directories
/// - Reports files that exist in the source directory but aren't referenced
/// - Ignores common files like README.md by default
/// - Supports configuration for custom ignore patterns
pub struct MDBOOK005 {
    /// Files to ignore when checking for orphans (case-insensitive)
    ignored_files: HashSet<String>,
}

impl Default for MDBOOK005 {
    fn default() -> Self {
        let mut ignored_files = HashSet::new();
        // Common files that are typically not in SUMMARY.md
        ignored_files.insert("readme.md".to_string());
        ignored_files.insert("contributing.md".to_string());
        ignored_files.insert("license.md".to_string());
        ignored_files.insert("changelog.md".to_string());
        ignored_files.insert("summary.md".to_string()); // Don't report SUMMARY.md itself

        Self { ignored_files }
    }
}

// TODO: Re-enable helper methods when tests are restored
// These methods are only used by tests, commenting out to avoid dead code warnings
/*
impl MDBOOK005 {
    /// Create a new instance with custom ignored files (in addition to defaults)
    pub fn with_ignored_files(additional_ignored: Vec<String>) -> Self {
        let mut instance = Self::default();
        for file in additional_ignored {
            instance.ignored_files.insert(file.to_lowercase());
        }
        instance
    }

    /// Add a file to the ignore list
    pub fn ignore_file(&mut self, filename: &str) {
        self.ignored_files.insert(filename.to_lowercase());
    }
}
*/

impl Rule for MDBOOK005 {
    fn id(&self) -> &'static str {
        "MDBOOK005"
    }

    fn name(&self) -> &'static str {
        "orphaned-files"
    }

    fn description(&self) -> &'static str {
        "Detect orphaned markdown files not referenced in SUMMARY.md"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::MdBook).introduced_in("mdbook-lint v0.2.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> mdbook_lint_core::error::Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // Only check SUMMARY.md files
        if !is_summary_file(document) {
            return Ok(violations);
        }

        // Find the book source directory
        // SUMMARY.md should be in the book's src directory
        let book_src_dir = if document.path.is_absolute() {
            document.path.parent().unwrap_or(Path::new("."))
        } else {
            // If path is relative, use current directory
            Path::new(".")
        };

        // Parse referenced files from SUMMARY.md
        let referenced_files = match self.parse_referenced_files(document) {
            Ok(files) => files,
            Err(_) => {
                // If we can't parse SUMMARY.md, we can't check for orphans
                return Ok(violations);
            }
        };

        // Find all markdown files in the book's source directory only
        // This ensures we only check files that are actually part of the book
        let all_markdown_files = match self.find_markdown_files(book_src_dir) {
            Ok(files) => files,
            Err(_) => {
                // If we can't scan the directory, we can't check for orphans
                return Ok(violations);
            }
        };

        // Find orphaned files
        let orphaned_files = self.find_orphaned_files(&referenced_files, &all_markdown_files);

        // Create violations for each orphaned file
        for orphaned_file in orphaned_files {
            let relative_path = orphaned_file
                .strip_prefix(book_src_dir)
                .unwrap_or(orphaned_file.as_path())
                .to_string_lossy()
                .replace('\\', "/") // Ensure consistent forward slashes for cross-platform compatibility
                .to_string();

            violations.push(self.create_violation(
                format!("Orphaned file '{relative_path}' is not referenced in SUMMARY.md"),
                1, // Report on line 1 of SUMMARY.md since it's a structural issue
                1,
                Severity::Warning,
            ));
        }

        Ok(violations)
    }
}

impl MDBOOK005 {
    /// Parse all file paths referenced in SUMMARY.md
    fn parse_referenced_files(
        &self,
        document: &Document,
    ) -> Result<HashSet<PathBuf>, Box<dyn std::error::Error>> {
        let mut referenced = HashSet::new();
        let project_root = document.path.parent().unwrap_or(Path::new("."));

        for line in &document.lines {
            if let Some(path) = self.extract_file_path(line) {
                // Resolve path relative to SUMMARY.md location
                let absolute_path = project_root.join(&path);
                if let Ok(canonical) = absolute_path.canonicalize() {
                    referenced.insert(canonical);
                } else {
                    // If canonicalize fails, use the resolved path
                    referenced.insert(absolute_path);
                }
            }
        }

        Ok(referenced)
    }

    /// Extract file path from a SUMMARY.md line if present
    fn extract_file_path(&self, line: &str) -> Option<String> {
        // Look for markdown link syntax: [title](path)
        if let Some(start) = line.find("](") {
            let after_bracket = &line[start + 2..];
            if let Some(end) = after_bracket.find(')') {
                let path = &after_bracket[..end];

                // Skip empty paths (draft chapters) and external URLs
                if path.is_empty() || path.starts_with("http://") || path.starts_with("https://") {
                    return None;
                }

                // Remove anchor fragments
                let path_without_anchor = path.split('#').next().unwrap_or(path);

                // Only include markdown files
                if path_without_anchor.ends_with(".md")
                    || path_without_anchor.ends_with(".markdown")
                {
                    return Some(path_without_anchor.to_string());
                }
            }
        }

        None
    }

    /// Find all markdown files in the book's source directory
    fn find_markdown_files(&self, book_src_dir: &Path) -> io::Result<HashSet<PathBuf>> {
        let mut markdown_files = HashSet::new();
        // Only scan within the book's source directory
        scan_directory_recursive(book_src_dir, &mut markdown_files)?;
        Ok(markdown_files)
    }

    /// Find files that exist but are not referenced
    fn find_orphaned_files(
        &self,
        referenced: &HashSet<PathBuf>,
        all_files: &HashSet<PathBuf>,
    ) -> Vec<PathBuf> {
        all_files
            .iter()
            .filter(|&file| {
                // Skip if file is referenced in SUMMARY.md
                if referenced.contains(file) {
                    return false;
                }

                // Skip files in our ignore list
                if let Some(filename) = file.file_name().and_then(|n| n.to_str())
                    && self.ignored_files.contains(&filename.to_lowercase())
                {
                    return false;
                }

                true
            })
            .cloned()
            .collect()
    }
}

/// Check if the document represents a SUMMARY.md file
fn is_summary_file(document: &Document) -> bool {
    document
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.eq_ignore_ascii_case("summary.md"))
        .unwrap_or(false)
}

/// Recursively scan directory for markdown files
fn scan_directory_recursive(dir: &Path, markdown_files: &mut HashSet<PathBuf>) -> io::Result<()> {
    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Skip common directories that shouldn't be scanned
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str())
                && matches!(
                    dir_name,
                    "target" | "node_modules" | ".git" | ".svn" | ".hg"
                )
            {
                continue;
            }
            // Recursively scan subdirectories
            scan_directory_recursive(&path, markdown_files)?;
        } else if let Some(extension) = path.extension().and_then(|e| e.to_str())
            && matches!(extension, "md" | "markdown")
        {
            if let Ok(canonical) = path.canonicalize() {
                markdown_files.insert(canonical);
            } else {
                markdown_files.insert(path);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_document(
        content: &str,
        file_path: &Path,
    ) -> mdbook_lint_core::error::Result<Document> {
        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file_path, content)?;
        Document::new(content.to_string(), file_path.to_path_buf())
    }

    #[test]
    fn test_mdbook005_no_orphans() -> mdbook_lint_core::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create SUMMARY.md that references all files
        let summary_content = r#"# Summary

[Introduction](intro.md)
- [Chapter 1](chapter1.md)
- [Chapter 2](chapter2.md)
"#;
        let summary_path = root.join("SUMMARY.md");
        let doc = create_test_document(summary_content, &summary_path)?;

        // Create the referenced files
        create_test_document("# Intro", &root.join("intro.md"))?;
        create_test_document("# Chapter 1", &root.join("chapter1.md"))?;
        create_test_document("# Chapter 2", &root.join("chapter2.md"))?;

        let rule = MDBOOK005::default();
        let violations = rule.check(&doc)?;

        assert_eq!(
            violations.len(),
            0,
            "Should have no violations when all files are referenced"
        );
        Ok(())
    }

    #[test]
    fn test_mdbook005_detect_orphans() -> mdbook_lint_core::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create SUMMARY.md that only references some files
        let summary_content = r#"# Summary

[Introduction](intro.md)
- [Chapter 1](chapter1.md)
"#;
        let summary_path = root.join("SUMMARY.md");
        let doc = create_test_document(summary_content, &summary_path)?;

        // Create referenced files
        create_test_document("# Intro", &root.join("intro.md"))?;
        create_test_document("# Chapter 1", &root.join("chapter1.md"))?;

        // Create orphaned files
        create_test_document("# Orphan", &root.join("orphan.md"))?;
        create_test_document("# Another", &root.join("another.md"))?;

        let rule = MDBOOK005::default();
        let violations = rule.check(&doc)?;

        assert_eq!(violations.len(), 2, "Should detect 2 orphaned files");

        let messages: Vec<_> = violations.iter().map(|v| &v.message).collect();
        assert!(messages.iter().any(|m| m.contains("orphan.md")));
        assert!(messages.iter().any(|m| m.contains("another.md")));

        Ok(())
    }

    #[test]
    fn test_mdbook005_ignore_common_files() -> mdbook_lint_core::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create SUMMARY.md with minimal content
        let summary_content = r#"# Summary

- [Chapter 1](chapter1.md)
"#;
        let summary_path = root.join("SUMMARY.md");
        let doc = create_test_document(summary_content, &summary_path)?;

        create_test_document("# Chapter 1", &root.join("chapter1.md"))?;

        // Create files that should be ignored by default
        create_test_document("# README", &root.join("README.md"))?;
        create_test_document("# Contributing", &root.join("CONTRIBUTING.md"))?;
        create_test_document("# License", &root.join("LICENSE.md"))?;

        let rule = MDBOOK005::default();
        let violations = rule.check(&doc)?;

        assert_eq!(
            violations.len(),
            0,
            "Should ignore common files like README.md"
        );
        Ok(())
    }

    #[test]
    fn test_mdbook005_nested_directories() -> mdbook_lint_core::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create SUMMARY.md that references nested files
        let summary_content = r#"# Summary

- [Chapter 1](guide/chapter1.md)
"#;
        let summary_path = root.join("SUMMARY.md");
        let doc = create_test_document(summary_content, &summary_path)?;

        // Create referenced nested file
        create_test_document("# Chapter 1", &root.join("guide/chapter1.md"))?;

        // Create orphaned nested file
        create_test_document("# Orphan", &root.join("guide/orphan.md"))?;

        let rule = MDBOOK005::default();
        let violations = rule.check(&doc)?;

        assert_eq!(
            violations.len(),
            1,
            "Should detect orphaned files in subdirectories"
        );
        assert!(violations[0].message.contains("guide/orphan.md"));
        Ok(())
    }

    #[test]
    fn test_mdbook005_draft_chapters() -> mdbook_lint_core::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create SUMMARY.md with draft chapters (empty paths)
        let summary_content = r#"# Summary

- [Chapter 1](chapter1.md)
- [Draft Chapter]()
"#;
        let summary_path = root.join("SUMMARY.md");
        let doc = create_test_document(summary_content, &summary_path)?;

        create_test_document("# Chapter 1", &root.join("chapter1.md"))?;
        create_test_document("# Orphan", &root.join("orphan.md"))?;

        let rule = MDBOOK005::default();
        let violations = rule.check(&doc)?;

        // Should still detect the orphan, but not complain about the draft
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("orphan.md"));
        Ok(())
    }

    #[test]
    fn test_mdbook005_non_summary_files() -> mdbook_lint_core::error::Result<()> {
        let temp_dir = TempDir::new()?;

        // Test on a non-SUMMARY.md file
        let content = "# Regular File";
        let doc_path = temp_dir.path().join("README.md");
        let doc = create_test_document(content, &doc_path)?;

        let rule = MDBOOK005::default();
        let violations = rule.check(&doc)?;

        assert_eq!(
            violations.len(),
            0,
            "Should not run on non-SUMMARY.md files"
        );
        Ok(())
    }

    #[test]
    fn test_mdbook005_scope_limited_to_src_dir() -> mdbook_lint_core::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create a book structure with files outside the src directory
        let book_src = root.join("src");
        fs::create_dir_all(&book_src)?;

        // Create SUMMARY.md in src/
        let summary_content = r#"# Summary

- [Chapter 1](chapter1.md)
"#;
        let summary_path = book_src.join("SUMMARY.md");
        let doc = create_test_document(summary_content, &summary_path)?;

        // Create referenced file in src/
        create_test_document("# Chapter 1", &book_src.join("chapter1.md"))?;

        // Create orphaned file in src/
        create_test_document("# Orphan in src", &book_src.join("orphan_in_src.md"))?;

        // Create files outside src/ that should NOT be detected
        create_test_document("# Outside", &root.join("outside.md"))?;
        create_test_document("# Config docs", &root.join("CONFIGURATION.md"))?;

        // Create a sibling directory with markdown files that should NOT be detected
        let docs_dir = root.join("docs");
        fs::create_dir_all(&docs_dir)?;
        create_test_document("# Docs", &docs_dir.join("documentation.md"))?;

        let rule = MDBOOK005::default();
        let violations = rule.check(&doc)?;

        // Should only detect the orphan in src/, not files outside
        assert_eq!(
            violations.len(),
            1,
            "Should only detect orphans within src/"
        );
        assert!(violations[0].message.contains("orphan_in_src.md"));
        assert!(!violations[0].message.contains("outside.md"));
        assert!(!violations[0].message.contains("CONFIGURATION.md"));
        assert!(!violations[0].message.contains("documentation.md"));

        Ok(())
    }

    #[test]
    fn test_extract_file_path() {
        let rule = MDBOOK005::default();

        // Valid paths
        assert_eq!(
            rule.extract_file_path("- [Chapter](chapter.md)"),
            Some("chapter.md".to_string())
        );
        assert_eq!(
            rule.extract_file_path("[Intro](intro.md)"),
            Some("intro.md".to_string())
        );
        assert_eq!(
            rule.extract_file_path("    - [Nested](sub/nested.md)"),
            Some("sub/nested.md".to_string())
        );

        // Paths with anchors
        assert_eq!(
            rule.extract_file_path("- [Link](file.md#section)"),
            Some("file.md".to_string())
        );

        // Invalid or ignored paths
        assert_eq!(rule.extract_file_path("- [Draft]()"), None);
        assert_eq!(
            rule.extract_file_path("- [External](https://example.com)"),
            None
        );
        assert_eq!(rule.extract_file_path("- [Non-MD](image.png)"), None);
        assert_eq!(rule.extract_file_path("Regular text"), None);
    }

    // TODO: Re-enable when helper methods are restored
    /*
    #[test]
    fn test_custom_ignored_files() -> mdbook_lint_core::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        let summary_content = r#"# Summary

- [Chapter 1](chapter1.md)
"#;
        let summary_path = root.join("SUMMARY.md");
        let doc = create_test_document(summary_content, &summary_path)?;

        create_test_document("# Chapter 1", &root.join("chapter1.md"))?;
        create_test_document("# Custom", &root.join("custom.md"))?;
        create_test_document("# Orphan", &root.join("orphan.md"))?;

        // Create rule that ignores custom.md
        let rule = MDBOOK005::with_ignored_files(vec!["custom.md".to_string()]);
        let violations = rule.check(&doc)?;

        // Should only report orphan.md, not custom.md
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("orphan.md"));
        assert!(!violations[0].message.contains("custom.md"));
        Ok(())
    }
    */
}
