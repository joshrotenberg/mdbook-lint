//! ADR012: No duplicate ADR numbers
//!
//! Validates that no two ADRs share the same number.
//! This rule analyzes all ADR documents in a collection.

use crate::adr::format::{AdrFormat, detect_format, extract_nygard_number, is_adr_document};
use mdbook_lint_core::rule::{CollectionRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::Severity;
use mdbook_lint_core::{Document, Result, Violation};
use std::collections::HashMap;

/// ADR012: Validates that no two ADRs share the same number
///
/// This collection rule checks all ADR documents to ensure that
/// each ADR number is unique across the entire collection.
pub struct Adr012;

impl Default for Adr012 {
    fn default() -> Self {
        Self
    }
}

impl Adr012 {
    /// Extract ADR number from a document
    fn extract_adr_number(document: &Document) -> Option<u32> {
        let format = detect_format(&document.content);

        match format {
            AdrFormat::Nygard | AdrFormat::Auto => {
                // Look for numbered title in first few lines
                for line in document.lines.iter().take(10) {
                    if let Some(num) = extract_nygard_number(line) {
                        return Some(num);
                    }
                }
                None
            }
            AdrFormat::Madr4 => {
                // MADR doesn't require numbered titles, try to extract from filename
                document
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .and_then(|name| {
                        // Match patterns like "0001-title.md" or "1-title.md"
                        name.split(&['-', '_'][..])
                            .next()
                            .and_then(|s| s.parse().ok())
                    })
            }
        }
    }
}

impl CollectionRule for Adr012 {
    fn id(&self) -> &'static str {
        "ADR012"
    }

    fn name(&self) -> &'static str {
        "adr-no-duplicate-numbers"
    }

    fn description(&self) -> &'static str {
        "Each ADR number should be unique"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Structure).introduced_in("mdbook-lint v0.14.0")
    }

    fn check_collection(&self, documents: &[Document]) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // Collect ADR numbers with their source documents
        let mut numbers_seen: HashMap<u32, Vec<&Document>> = HashMap::new();

        for doc in documents {
            if !is_adr_document(&doc.content, Some(&doc.path)) {
                continue;
            }

            if let Some(num) = Self::extract_adr_number(doc) {
                numbers_seen.entry(num).or_default().push(doc);
            }
        }

        // Find duplicates
        for (num, docs) in numbers_seen {
            if docs.len() > 1 {
                // Report violation for each duplicate after the first
                for doc in docs.iter().skip(1) {
                    let other_files: Vec<_> = docs
                        .iter()
                        .filter(|d| d.path != doc.path)
                        .map(|d| d.path.display().to_string())
                        .collect();

                    violations.push(self.create_violation_for_file(
                        &doc.path,
                        format!(
                            "Duplicate ADR number {}. Also used in: {}",
                            num,
                            other_files.join(", ")
                        ),
                        1,
                        1,
                        Severity::Error,
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

    fn create_nygard_adr(number: u32, title: &str, filename: &str) -> Document {
        let content = format!(
            r#"# {}. {}

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
            number, title
        );
        Document::new(content, PathBuf::from(format!("adr/{}", filename))).unwrap()
    }

    #[test]
    fn test_unique_numbers() {
        let docs = vec![
            create_nygard_adr(1, "First Decision", "0001-first.md"),
            create_nygard_adr(2, "Second Decision", "0002-second.md"),
            create_nygard_adr(3, "Third Decision", "0003-third.md"),
        ];

        let rule = Adr012;
        let violations = rule.check_collection(&docs).unwrap();
        assert!(
            violations.is_empty(),
            "Unique numbers should not produce violations"
        );
    }

    #[test]
    fn test_duplicate_numbers() {
        let docs = vec![
            create_nygard_adr(1, "First Decision", "0001-first.md"),
            create_nygard_adr(1, "Also First", "0001-also-first.md"), // Duplicate!
        ];

        let rule = Adr012;
        let violations = rule.check_collection(&docs).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Duplicate ADR number 1"));
    }

    #[test]
    fn test_multiple_duplicates() {
        let docs = vec![
            create_nygard_adr(1, "First", "0001-first.md"),
            create_nygard_adr(1, "Also First", "0001-also-first.md"),
            create_nygard_adr(1, "Third First", "0001-third.md"), // Three with same number
            create_nygard_adr(2, "Second", "0002-second.md"),
        ];

        let rule = Adr012;
        let violations = rule.check_collection(&docs).unwrap();
        // Two violations: one for each duplicate after the first
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_empty_collection() {
        let docs: Vec<Document> = vec![];

        let rule = Adr012;
        let violations = rule.check_collection(&docs).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_violation_severity_is_error() {
        let docs = vec![
            create_nygard_adr(1, "First", "0001-first.md"),
            create_nygard_adr(1, "Duplicate", "0001-duplicate.md"),
        ];

        let rule = Adr012;
        let violations = rule.check_collection(&docs).unwrap();
        assert_eq!(violations[0].severity, Severity::Error);
    }
}
