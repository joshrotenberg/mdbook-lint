//! MDBOOK023: Chapter title matching validation
//!
//! Validates that chapter titles in SUMMARY.md match the H1 headers in the linked files.

use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use std::path::PathBuf;

/// MDBOOK023: Validates that chapter titles in SUMMARY.md match H1 headers
///
/// This rule checks that the title used in SUMMARY.md for a chapter matches
/// the H1 header in the linked markdown file. Mismatches can confuse readers
/// when the navigation shows a different title than the page content.
#[derive(Default)]
pub struct MDBOOK023 {
    /// The source directory for the mdBook (usually "src")
    src_dir: Option<PathBuf>,
}

impl MDBOOK023 {
    /// Create a new MDBOOK023 rule with a specific source directory
    #[allow(dead_code)]
    pub fn with_src_dir(src_dir: PathBuf) -> Self {
        Self {
            src_dir: Some(src_dir),
        }
    }

    /// Extract chapter entries from SUMMARY.md content
    fn extract_chapters(&self, content: &str) -> Vec<ChapterEntry> {
        let mut chapters = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num + 1; // 1-based
            let trimmed = line.trim();

            // Skip empty lines, headers, and separators
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.chars().all(|c| c == '-') {
                continue;
            }

            // Parse markdown links: [Title](path.md) or - [Title](path.md)
            if let Some(chapter) = self.parse_chapter_link(trimmed, line_num) {
                chapters.push(chapter);
            }
        }

        chapters
    }

    /// Parse a chapter link from a line
    fn parse_chapter_link(&self, line: &str, line_num: usize) -> Option<ChapterEntry> {
        // Remove list marker if present
        let content = line
            .trim_start_matches(|c: char| c == '-' || c == '*' || c.is_whitespace())
            .trim();

        // Find [title](path) pattern
        let open_bracket = content.find('[')?;
        let close_bracket = content.find("](")?;
        let close_paren = content.find(')')?;

        if open_bracket >= close_bracket || close_bracket >= close_paren {
            return None;
        }

        let title = content[open_bracket + 1..close_bracket].trim().to_string();
        let path = content[close_bracket + 2..close_paren].trim().to_string();

        // Skip draft chapters (empty path) and external links
        if path.is_empty() || path.starts_with("http://") || path.starts_with("https://") {
            return None;
        }

        // Skip anchor-only links
        if path.starts_with('#') {
            return None;
        }

        Some(ChapterEntry {
            title,
            path,
            line: line_num,
        })
    }

    /// Extract the first H1 header from a markdown file
    fn extract_h1_header(&self, content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();

            // ATX-style H1: # Title
            if let Some(title) = trimmed.strip_prefix("# ") {
                // Remove trailing # if present (closed ATX)
                let title = title.trim_end_matches('#').trim();
                if !title.is_empty() {
                    return Some(title.to_string());
                }
            }

            // Also check for # without space (common typo, but still an H1)
            if trimmed.starts_with('#') && !trimmed.starts_with("##") {
                let title = trimmed.trim_start_matches('#').trim();
                // Remove trailing # if present
                let title = title.trim_end_matches('#').trim();
                if !title.is_empty() {
                    return Some(title.to_string());
                }
            }
        }

        None
    }

    /// Normalize a title for comparison (case-insensitive, whitespace-normalized)
    fn normalize_title(&self, title: &str) -> String {
        title
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    }

    /// Check if two titles match (allowing for minor formatting differences)
    fn titles_match(&self, summary_title: &str, h1_title: &str) -> bool {
        self.normalize_title(summary_title) == self.normalize_title(h1_title)
    }
}

impl Rule for MDBOOK023 {
    fn id(&self) -> &'static str {
        "MDBOOK023"
    }

    fn name(&self) -> &'static str {
        "chapter-title-match"
    }

    fn description(&self) -> &'static str {
        "Chapter titles in SUMMARY.md should match H1 headers in linked files"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::MdBook).introduced_in("mdbook-lint v0.11.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> mdbook_lint_core::error::Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // This rule only applies to SUMMARY.md
        if !is_summary_file(document) {
            return Ok(violations);
        }

        // Determine the source directory
        let src_dir = self.src_dir.clone().unwrap_or_else(|| {
            document
                .path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_default()
        });

        // Extract all chapter entries from SUMMARY.md
        let chapters = self.extract_chapters(&document.content);

        // Check each chapter's linked file for H1 header match
        for chapter in chapters {
            // Resolve the path relative to src directory
            let chapter_path = src_dir.join(&chapter.path);

            // Try to read the linked file and check H1 header match
            // If file doesn't exist, that's handled by MDBOOK002
            // If no H1 header found, that's handled by other rules (MD041)
            if let Ok(content) = std::fs::read_to_string(&chapter_path)
                && let Some(h1_title) = self.extract_h1_header(&content)
                && !self.titles_match(&chapter.title, &h1_title)
            {
                violations.push(self.create_violation(
                    format!(
                        "Chapter title '{}' doesn't match H1 header '{}' in {}",
                        chapter.title, h1_title, chapter.path
                    ),
                    chapter.line,
                    1,
                    Severity::Warning,
                ));
            }
        }

        Ok(violations)
    }
}

/// A chapter entry parsed from SUMMARY.md
#[derive(Debug)]
struct ChapterEntry {
    /// The title as shown in SUMMARY.md
    title: String,
    /// The path to the chapter file
    path: String,
    /// The line number in SUMMARY.md
    line: usize,
}

/// Check if the document represents a SUMMARY.md file
fn is_summary_file(document: &Document) -> bool {
    document
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "SUMMARY.md")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_document(content: &str, path: &std::path::Path) -> Document {
        Document::new(content.to_string(), path.to_path_buf()).unwrap()
    }

    #[test]
    fn test_not_summary_file() {
        let content = "# Some File\n\n[Link](other.md)";
        let doc = create_test_document(content, std::path::Path::new("README.md"));
        let rule = MDBOOK023::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(
            violations.len(),
            0,
            "Non-SUMMARY.md files should be ignored"
        );
    }

    #[test]
    fn test_extract_chapters() {
        let rule = MDBOOK023::default();
        let content = r#"# Summary

[Introduction](README.md)

# User Guide

- [Getting Started](guide/start.md)
- [Advanced Usage](guide/advanced.md)
    - [Sub Chapter](guide/sub.md)

---

[Contributors](misc/contributors.md)
"#;
        let chapters = rule.extract_chapters(content);
        assert_eq!(chapters.len(), 5);
        assert_eq!(chapters[0].title, "Introduction");
        assert_eq!(chapters[0].path, "README.md");
        assert_eq!(chapters[1].title, "Getting Started");
        assert_eq!(chapters[2].title, "Advanced Usage");
        assert_eq!(chapters[3].title, "Sub Chapter");
        assert_eq!(chapters[4].title, "Contributors");
    }

    #[test]
    fn test_extract_h1_header() {
        let rule = MDBOOK023::default();

        // Standard ATX header
        assert_eq!(
            rule.extract_h1_header("# My Title\n\nContent"),
            Some("My Title".to_string())
        );

        // Closed ATX header
        assert_eq!(
            rule.extract_h1_header("# My Title #\n\nContent"),
            Some("My Title".to_string())
        );

        // With leading content
        assert_eq!(
            rule.extract_h1_header("Some intro\n\n# The Title\n\nContent"),
            Some("The Title".to_string())
        );

        // No H1
        assert_eq!(rule.extract_h1_header("## Only H2\n\nContent"), None);

        // Empty H1
        assert_eq!(rule.extract_h1_header("#\n\nContent"), None);
    }

    #[test]
    fn test_titles_match() {
        let rule = MDBOOK023::default();

        // Exact match
        assert!(rule.titles_match("Getting Started", "Getting Started"));

        // Case insensitive
        assert!(rule.titles_match("Getting Started", "getting started"));

        // Whitespace normalized
        assert!(rule.titles_match("Getting  Started", "Getting Started"));

        // Different titles
        assert!(!rule.titles_match("Getting Started", "Introduction"));
    }

    #[test]
    fn test_skip_draft_chapters() {
        let rule = MDBOOK023::default();
        let content = r#"# Summary

- [Regular Chapter](chapter.md)
- [Draft Chapter]()
"#;
        let chapters = rule.extract_chapters(content);
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].title, "Regular Chapter");
    }

    #[test]
    fn test_skip_external_links() {
        let rule = MDBOOK023::default();
        let content = r#"# Summary

- [Local Chapter](chapter.md)
- [External Link](https://example.com)
- [HTTP Link](http://example.com)
"#;
        let chapters = rule.extract_chapters(content);
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].title, "Local Chapter");
    }

    #[test]
    fn test_matching_titles() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path();

        // Create a chapter file with matching title
        let chapter_content = "# Getting Started\n\nWelcome to the guide.";
        fs::write(src_dir.join("start.md"), chapter_content).unwrap();

        // Create SUMMARY.md
        let summary_content = r#"# Summary

- [Getting Started](start.md)
"#;
        let summary_path = src_dir.join("SUMMARY.md");
        fs::write(&summary_path, summary_content).unwrap();

        let doc = create_test_document(summary_content, &summary_path);
        let rule = MDBOOK023::with_src_dir(src_dir.to_path_buf());
        let violations = rule.check(&doc).unwrap();

        assert_eq!(
            violations.len(),
            0,
            "Matching titles should have no violations"
        );
    }

    #[test]
    fn test_mismatched_titles() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path();

        // Create a chapter file with different title
        let chapter_content = "# Introduction to the Project\n\nWelcome!";
        fs::write(src_dir.join("intro.md"), chapter_content).unwrap();

        // Create SUMMARY.md with different title
        let summary_content = r#"# Summary

- [Getting Started](intro.md)
"#;
        let summary_path = src_dir.join("SUMMARY.md");
        fs::write(&summary_path, summary_content).unwrap();

        let doc = create_test_document(summary_content, &summary_path);
        let rule = MDBOOK023::with_src_dir(src_dir.to_path_buf());
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("doesn't match"));
        assert!(violations[0].message.contains("Getting Started"));
        assert!(
            violations[0]
                .message
                .contains("Introduction to the Project")
        );
    }

    #[test]
    fn test_case_insensitive_match() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path();

        // Create a chapter file with different case
        let chapter_content = "# getting started\n\nContent here.";
        fs::write(src_dir.join("start.md"), chapter_content).unwrap();

        // Create SUMMARY.md
        let summary_content = r#"# Summary

- [Getting Started](start.md)
"#;
        let summary_path = src_dir.join("SUMMARY.md");
        fs::write(&summary_path, summary_content).unwrap();

        let doc = create_test_document(summary_content, &summary_path);
        let rule = MDBOOK023::with_src_dir(src_dir.to_path_buf());
        let violations = rule.check(&doc).unwrap();

        assert_eq!(violations.len(), 0, "Case differences should be allowed");
    }

    #[test]
    fn test_missing_h1_header() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path();

        // Create a chapter file without H1
        let chapter_content = "## Only H2\n\nNo H1 header here.";
        fs::write(src_dir.join("no-h1.md"), chapter_content).unwrap();

        // Create SUMMARY.md
        let summary_content = r#"# Summary

- [Some Chapter](no-h1.md)
"#;
        let summary_path = src_dir.join("SUMMARY.md");
        fs::write(&summary_path, summary_content).unwrap();

        let doc = create_test_document(summary_content, &summary_path);
        let rule = MDBOOK023::with_src_dir(src_dir.to_path_buf());
        let violations = rule.check(&doc).unwrap();

        // No violation from this rule - missing H1 is handled by MD041
        assert_eq!(violations.len(), 0);
    }
}
