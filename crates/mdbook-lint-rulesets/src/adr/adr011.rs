//! ADR011: Sequential ADR numbering
//!
//! Validates that ADR numbers are sequential with no gaps.
//! This rule analyzes all ADR documents in a collection.

use crate::adr::format::{AdrFormat, detect_format, extract_nygard_number, is_adr_document};
use mdbook_lint_core::rule::{CollectionRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::Severity;
use mdbook_lint_core::{Document, Result, Violation};
use std::collections::BTreeMap;

/// ADR011: Validates that ADR numbers are sequential
///
/// This collection rule checks all ADR documents to ensure that:
/// - ADR numbers form a continuous sequence starting from 1
/// - No gaps exist in the numbering
pub struct Adr011;

impl Default for Adr011 {
    fn default() -> Self {
        Self
    }
}

impl Adr011 {
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

impl CollectionRule for Adr011 {
    fn id(&self) -> &'static str {
        "ADR011"
    }

    fn name(&self) -> &'static str {
        "adr-sequential-numbering"
    }

    fn description(&self) -> &'static str {
        "ADR numbers should be sequential with no gaps"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Structure).introduced_in("mdbook-lint v0.14.0")
    }

    fn check_collection(&self, documents: &[Document]) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // Collect ADR numbers with their source documents
        let mut adr_numbers: BTreeMap<u32, &Document> = BTreeMap::new();

        for doc in documents {
            if !is_adr_document(&doc.content, Some(&doc.path)) {
                continue;
            }

            if let Some(num) = Self::extract_adr_number(doc) {
                adr_numbers.insert(num, doc);
            }
        }

        if adr_numbers.is_empty() {
            return Ok(violations);
        }

        // Check for gaps in numbering
        let numbers: Vec<u32> = adr_numbers.keys().copied().collect();

        // Check if first ADR is 1 (or 0)
        let first_num = *numbers.first().unwrap();
        if first_num != 0 && first_num != 1 {
            violations.push(self.create_violation(
                format!(
                    "ADR numbering should start at 1 (or 0), but first ADR is {}",
                    first_num
                ),
                1,
                1,
                Severity::Warning,
            ));
        }

        // Find gaps - check from start to the maximum number found
        let start = if first_num == 0 { 0 } else { 1 };
        let max_num = *numbers.last().unwrap();
        for expected in start..max_num {
            if !adr_numbers.contains_key(&expected) {
                // Find the ADR that comes after the gap to report the violation there
                let next_adr = numbers.iter().find(|&&n| n > expected);
                if let Some(&next_num) = next_adr
                    && let Some(doc) = adr_numbers.get(&next_num)
                {
                    violations.push(self.create_violation_for_file(
                        &doc.path,
                        format!(
                            "Missing ADR number {} (gap before ADR {})",
                            expected, next_num
                        ),
                        1,
                        1,
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

    fn create_nygard_adr(number: u32, title: &str) -> Document {
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
        Document::new(
            content,
            PathBuf::from(format!(
                "adr/{:04}-{}.md",
                number,
                title.to_lowercase().replace(' ', "-")
            )),
        )
        .unwrap()
    }

    #[test]
    fn test_sequential_numbers() {
        let docs = vec![
            create_nygard_adr(1, "First Decision"),
            create_nygard_adr(2, "Second Decision"),
            create_nygard_adr(3, "Third Decision"),
        ];

        let rule = Adr011;
        let violations = rule.check_collection(&docs).unwrap();
        assert!(
            violations.is_empty(),
            "Sequential numbers should not produce violations"
        );
    }

    #[test]
    fn test_gap_in_numbers() {
        let docs = vec![
            create_nygard_adr(1, "First Decision"),
            create_nygard_adr(3, "Third Decision"), // Gap: missing 2
        ];

        let rule = Adr011;
        let violations = rule.check_collection(&docs).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Missing ADR number 2"));
    }

    #[test]
    fn test_multiple_gaps() {
        let docs = vec![
            create_nygard_adr(1, "First Decision"),
            create_nygard_adr(4, "Fourth Decision"), // Gaps: missing 2, 3
        ];

        let rule = Adr011;
        let violations = rule.check_collection(&docs).unwrap();
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_starting_from_zero() {
        let docs = vec![
            create_nygard_adr(0, "Record Architecture Decisions"),
            create_nygard_adr(1, "First Decision"),
            create_nygard_adr(2, "Second Decision"),
        ];

        let rule = Adr011;
        let violations = rule.check_collection(&docs).unwrap();
        assert!(violations.is_empty(), "Starting from 0 should be valid");
    }

    #[test]
    fn test_not_starting_from_one() {
        let docs = vec![
            create_nygard_adr(5, "Fifth Decision"),
            create_nygard_adr(6, "Sixth Decision"),
        ];

        let rule = Adr011;
        let violations = rule.check_collection(&docs).unwrap();
        assert!(
            !violations.is_empty(),
            "Should warn when not starting from 1"
        );
    }

    #[test]
    fn test_empty_collection() {
        let docs: Vec<Document> = vec![];

        let rule = Adr011;
        let violations = rule.check_collection(&docs).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn test_extract_number_from_nygard() {
        let doc = create_nygard_adr(42, "Test Decision");
        assert_eq!(Adr011::extract_adr_number(&doc), Some(42));
    }
}
