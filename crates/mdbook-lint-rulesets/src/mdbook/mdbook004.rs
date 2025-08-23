//! MDBOOK004: No duplicate chapter titles across the book
//!
//! This rule validates that chapter titles are unique across the entire book.

use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};
use std::collections::HashMap;

/// Type alias for complex document title data structure
type DocumentTitleList = [(String, Vec<(String, usize, usize)>)];

/// MDBOOK004: No duplicate chapter titles across the book
///
/// This rule checks that each chapter has a unique title within the book.
/// Note: This rule is designed to work with individual chapters and will
/// need cross-file coordination to detect duplicates across the entire book.
pub struct MDBOOK004;

impl AstRule for MDBOOK004 {
    fn id(&self) -> &'static str {
        "MDBOOK004"
    }

    fn name(&self) -> &'static str {
        "no-duplicate-chapter-titles"
    }

    fn description(&self) -> &'static str {
        "Chapter titles should be unique across the book"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::MdBook).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_ast<'a>(
        &self,
        document: &Document,
        ast: &'a AstNode<'a>,
    ) -> mdbook_lint_core::error::Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let mut title_positions = HashMap::new();

        // Extract all heading titles and their positions
        for node in ast.descendants() {
            if let NodeValue::Heading(_heading) = &node.data.borrow().value
                && let Some((line, column)) = document.node_position(node)
            {
                let title = document.node_text(node).trim().to_string();

                if !title.is_empty() {
                    // Check for duplicates within the same document
                    if let Some((prev_line, _)) = title_positions.get(&title) {
                        violations.push(self.create_violation(
                            format!(
                                "Duplicate chapter title '{title}' found (also at line {prev_line})"
                            ),
                            line,
                            column,
                            Severity::Error,
                        ));
                    } else {
                        title_positions.insert(title, (line, column));
                    }
                }
            }
        }

        Ok(violations)
    }
}

impl MDBOOK004 {
    /// Extract all heading titles from a document for cross-file analysis
    pub fn extract_chapter_titles(
        document: &Document,
    ) -> mdbook_lint_core::error::Result<Vec<(String, usize, usize)>> {
        use comrak::Arena;

        let arena = Arena::new();
        let ast = document.parse_ast(&arena);
        let mut titles = Vec::new();

        for node in ast.descendants() {
            if let NodeValue::Heading(_) = &node.data.borrow().value
                && let Some((line, column)) = document.node_position(node)
            {
                let title = document.node_text(node).trim().to_string();
                if !title.is_empty() {
                    titles.push((title, line, column));
                }
            }
        }

        Ok(titles)
    }

    /// Check for duplicate titles across multiple documents
    pub fn check_cross_document_duplicates(
        documents_with_titles: &DocumentTitleList,
    ) -> Vec<(String, String, usize, usize, String)> {
        let mut title_to_files = HashMap::new();
        let mut duplicates = Vec::new();

        // Build a map of titles to the files they appear in
        for (file_path, titles) in documents_with_titles {
            for (title, line, column) in titles {
                title_to_files
                    .entry(title.clone())
                    .or_insert_with(Vec::new)
                    .push((file_path.clone(), *line, *column));
            }
        }

        // Find duplicates
        for (title, occurrences) in &title_to_files {
            if occurrences.len() > 1 {
                for (file_path, line, column) in occurrences {
                    let other_files: Vec<String> = title_to_files[title]
                        .iter()
                        .filter(|(f, _, _)| f != file_path)
                        .map(|(f, l, _)| format!("{f}:{l}"))
                        .collect();

                    if !other_files.is_empty() {
                        duplicates.push((
                            file_path.clone(),
                            title.clone(),
                            *line,
                            *column,
                            other_files.join(", "),
                        ));
                    }
                }
            }
        }

        duplicates
    }

    /// Create violations for cross-document duplicates
    pub fn create_cross_document_violations(
        &self,
        duplicates: &[(String, String, usize, usize, String)],
    ) -> Vec<(String, Violation)> {
        duplicates
            .iter()
            .map(|(file_path, title, line, column, other_locations)| {
                let violation = Violation {
                    rule_id: self.id().to_string(),
                    rule_name: self.name().to_string(),
                    message: format!(
                        "Duplicate chapter title '{title}' found in other files: {other_locations}"
                    ),
                    line: *line,
                    column: *column,
                    severity: Severity::Error,
                    fix: None,
                };
                (file_path.clone(), violation)
            })
            .collect()
    }
}

// TODO: Re-enable tests once test_helpers is available or tests are rewritten
// Tests temporarily disabled during migration (Part 1 of #66)
#[cfg(test)]
#[cfg(feature = "test_helpers_available")] // This feature doesn't exist, so tests won't compile
mod tests {
    use super::*;
    // use mdbook_lint_core::test_helpers::*;

    #[test]
    #[ignore = "Test helpers not available during migration"]
    fn test_mdbook004_no_duplicates() {
        // let content = MarkdownBuilder::new()
        //     .heading(1, "Introduction")
        //     .blank_line()
        //     .paragraph("This is the introduction.")
        //     .blank_line()
        //     .heading(2, "Getting Started")
        //     .blank_line()
        //     .paragraph("How to get started.")
        //     .blank_line()
        //     .heading(2, "Advanced Topics")
        //     .blank_line()
        //     .paragraph("Advanced material.")
        //     .build();

        // assert_no_violations(MDBOOK004, &content);
    }

    #[test]
    fn test_mdbook004_within_document_duplicates() {
        let content = MarkdownBuilder::new()
            .heading(1, "Introduction")
            .blank_line()
            .paragraph("First introduction.")
            .blank_line()
            .heading(2, "Getting Started")
            .blank_line()
            .paragraph("How to get started.")
            .blank_line()
            .heading(1, "Introduction")
            .blank_line()
            .paragraph("Second introduction - duplicate!")
            .build();

        let violations = assert_violation_count(MDBOOK004, &content, 1);
        assert_violation_contains_message(&violations, "Duplicate chapter title 'Introduction'");
        assert_violation_contains_message(&violations, "also at line 1");
        assert_violation_at_line(&violations, 9);
    }

    #[test]
    fn test_mdbook004_case_sensitive() {
        let content = MarkdownBuilder::new()
            .heading(1, "Introduction")
            .blank_line()
            .heading(1, "introduction")
            .blank_line()
            .heading(1, "INTRODUCTION")
            .build();

        // These should be treated as different titles (case-sensitive)
        assert_no_violations(MDBOOK004, &content);
    }

    #[test]
    fn test_mdbook004_different_heading_levels() {
        let content = MarkdownBuilder::new()
            .heading(1, "Setup")
            .blank_line()
            .heading(2, "Setup")
            .blank_line()
            .heading(3, "Setup")
            .build();

        // Even different heading levels should be considered duplicates
        let violations = assert_violation_count(MDBOOK004, &content, 2);
        assert_violation_contains_message(&violations, "Duplicate chapter title 'Setup'");
    }

    #[test]
    fn test_mdbook004_extract_titles() {
        let content = MarkdownBuilder::new()
            .heading(1, "Chapter One")
            .blank_line()
            .paragraph("Content.")
            .blank_line()
            .heading(2, "Section A")
            .blank_line()
            .heading(2, "Section B")
            .build();

        let document = create_document(&content);
        let titles = MDBOOK004::extract_chapter_titles(&document).unwrap();

        assert_eq!(titles.len(), 3);
        assert_eq!(titles[0].0, "Chapter One");
        assert_eq!(titles[1].0, "Section A");
        assert_eq!(titles[2].0, "Section B");

        // Check line numbers
        assert_eq!(titles[0].1, 1); // Line 1
        assert_eq!(titles[1].1, 5); // Line 5
        assert_eq!(titles[2].1, 7); // Line 7
    }

    #[test]
    fn test_mdbook004_cross_document_analysis() {
        let documents = vec![
            (
                "chapter1.md".to_string(),
                vec![
                    ("Introduction".to_string(), 1, 1),
                    ("Getting Started".to_string(), 5, 1),
                ],
            ),
            (
                "chapter2.md".to_string(),
                vec![
                    ("Advanced Topics".to_string(), 1, 1),
                    ("Introduction".to_string(), 8, 1), // Duplicate!
                ],
            ),
            (
                "chapter3.md".to_string(),
                vec![
                    ("Conclusion".to_string(), 1, 1),
                    ("Getting Started".to_string(), 3, 1), // Another duplicate!
                ],
            ),
        ];

        let duplicates = MDBOOK004::check_cross_document_duplicates(&documents);

        // Should find 4 violations (2 for "Introduction", 2 for "Getting Started")
        assert_eq!(duplicates.len(), 4);

        // Check that we found duplicates for both titles
        let duplicate_titles: Vec<&String> =
            duplicates.iter().map(|(_, title, _, _, _)| title).collect();
        assert!(duplicate_titles.contains(&&"Introduction".to_string()));
        assert!(duplicate_titles.contains(&&"Getting Started".to_string()));
    }

    #[test]
    fn test_mdbook004_create_cross_document_violations() {
        let rule = MDBOOK004;
        let duplicates = vec![
            (
                "chapter1.md".to_string(),
                "Introduction".to_string(),
                1,
                1,
                "chapter2.md:5".to_string(),
            ),
            (
                "chapter2.md".to_string(),
                "Introduction".to_string(),
                5,
                1,
                "chapter1.md:1".to_string(),
            ),
        ];

        let violations = rule.create_cross_document_violations(&duplicates);

        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].0, "chapter1.md");
        assert_eq!(violations[1].0, "chapter2.md");

        assert!(
            violations[0]
                .1
                .message
                .contains("Duplicate chapter title 'Introduction'")
        );
        assert!(violations[0].1.message.contains("chapter2.md:5"));
        assert!(violations[1].1.message.contains("chapter1.md:1"));
    }

    #[test]
    fn test_mdbook004_empty_headings_ignored() {
        let content = MarkdownBuilder::new()
            .line("# ")
            .blank_line()
            .line("## ")
            .blank_line()
            .heading(1, "Real Title")
            .build();

        // Empty headings should be ignored
        assert_no_violations(MDBOOK004, &content);
    }

    #[test]
    fn test_mdbook004_whitespace_handling() {
        let content = MarkdownBuilder::new()
            .line("# Introduction ")
            .blank_line()
            .line("#  Introduction")
            .blank_line()
            .line("# Introduction  ")
            .build();

        // Whitespace should be trimmed, so these are duplicates
        let violations = assert_violation_count(MDBOOK004, &content, 2);
        assert_violation_contains_message(&violations, "Duplicate chapter title 'Introduction'");
    }
}
