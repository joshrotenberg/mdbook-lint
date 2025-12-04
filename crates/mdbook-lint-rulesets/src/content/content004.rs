//! CONTENT004: Heading capitalization consistency
//!
//! Checks that headings use consistent capitalization style throughout
//! the document (e.g., Title Case vs sentence case).

use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use regex::Regex;
use std::sync::LazyLock;

/// Regex to extract heading text from ATX headings
static HEADING_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(#{1,6})\s+(.+?)(?:\s*#*)?$").unwrap());

/// Common lowercase words in Title Case (articles, conjunctions, prepositions)
const TITLE_CASE_EXCEPTIONS: &[&str] = &[
    "a", "an", "the", "and", "but", "or", "nor", "for", "yet", "so", "at", "by", "in", "of", "on",
    "to", "up", "as", "if", "is", "it", "vs", "via", "with",
];

/// Capitalization style for headings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CapitalizationStyle {
    /// Title Case: Most Words Are Capitalized
    TitleCase,
    /// Sentence case: Only first word and proper nouns capitalized
    SentenceCase,
    /// Consistent: Use whatever style the first heading uses
    #[default]
    Consistent,
}

/// CONTENT004: Checks heading capitalization consistency
///
/// This rule ensures headings use a consistent capitalization style.
/// By default, it detects the style from the first heading and expects
/// all subsequent headings to follow the same pattern.
#[derive(Clone, Default)]
pub struct CONTENT004 {
    /// Required capitalization style
    style: CapitalizationStyle,
}

impl CONTENT004 {
    /// Create with a specific capitalization style
    #[allow(dead_code)]
    pub fn with_style(style: CapitalizationStyle) -> Self {
        Self { style }
    }

    /// Extract heading text from a line
    fn extract_heading(&self, line: &str) -> Option<(usize, String)> {
        HEADING_REGEX.captures(line).map(|caps| {
            let level = caps.get(1).unwrap().as_str().len();
            let text = caps.get(2).unwrap().as_str().trim().to_string();
            (level, text)
        })
    }

    /// Check if a word is an acronym (all uppercase letters)
    fn is_acronym(&self, word: &str) -> bool {
        word.len() > 1
            && word.chars().all(|c| c.is_uppercase() || !c.is_alphabetic())
            && word.chars().any(|c| c.is_alphabetic())
    }

    /// Check if a word is a title case exception (article, preposition, etc.)
    fn is_exception(&self, word: &str) -> bool {
        TITLE_CASE_EXCEPTIONS.contains(&word.to_lowercase().as_str())
    }

    /// Get significant words (excluding acronyms and exceptions except first word)
    fn get_significant_words<'a>(&self, text: &'a str) -> Vec<(usize, &'a str)> {
        text.split_whitespace()
            .enumerate()
            .filter(|(i, word)| {
                // First word is always significant
                if *i == 0 {
                    return true;
                }
                // Skip acronyms
                if self.is_acronym(word) {
                    return false;
                }
                // Skip exception words
                if self.is_exception(word) {
                    return false;
                }
                // Skip non-alphabetic words
                if !word.chars().next().is_some_and(|c| c.is_alphabetic()) {
                    return false;
                }
                true
            })
            .collect()
    }

    /// Check if a heading appears to be Title Case
    fn is_title_case(&self, text: &str) -> bool {
        let words = self.get_significant_words(text);
        if words.is_empty() {
            return true;
        }

        let capitalized = words
            .iter()
            .filter(|(_, word)| word.chars().next().is_some_and(|c| c.is_uppercase()))
            .count();

        // Consider title case if >= 60% of significant words are capitalized
        capitalized as f64 / words.len() as f64 >= 0.6
    }

    /// Check if a heading appears to be sentence case
    fn is_sentence_case(&self, text: &str) -> bool {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return true;
        }

        // First word should be capitalized
        if !words[0].chars().next().is_some_and(|c| c.is_uppercase()) {
            return false;
        }

        // Count non-first words that start with uppercase (excluding acronyms)
        let mut uppercase_non_first = 0;
        let mut checkable_non_first = 0;

        for word in words.iter().skip(1) {
            // Skip acronyms
            if self.is_acronym(word) {
                continue;
            }
            // Skip non-alphabetic
            if !word.chars().next().is_some_and(|c| c.is_alphabetic()) {
                continue;
            }

            checkable_non_first += 1;
            if word.chars().next().is_some_and(|c| c.is_uppercase()) {
                uppercase_non_first += 1;
            }
        }

        // Allow some capitalization for proper nouns (~30% of remaining words)
        checkable_non_first == 0 || uppercase_non_first as f64 / checkable_non_first as f64 <= 0.35
    }

    /// Detect the style of a heading
    fn detect_style(&self, text: &str) -> CapitalizationStyle {
        let is_title = self.is_title_case(text);
        let is_sentence = self.is_sentence_case(text);

        // If it looks like title case but not sentence case, it's title case
        if is_title && !is_sentence {
            CapitalizationStyle::TitleCase
        } else {
            // Default to sentence case (includes ambiguous cases)
            CapitalizationStyle::SentenceCase
        }
    }
}

impl Rule for CONTENT004 {
    fn id(&self) -> &'static str {
        "CONTENT004"
    }

    fn name(&self) -> &'static str {
        "heading-capitalization"
    }

    fn description(&self) -> &'static str {
        "Headings should use consistent capitalization (Title Case or sentence case)"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Content).introduced_in("mdbook-lint v0.12.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> mdbook_lint_core::error::Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let mut detected_style: Option<CapitalizationStyle> = None;
        let mut in_code_block = false;

        for (line_idx, line) in document.lines.iter().enumerate() {
            let line_num = line_idx + 1;
            let trimmed = line.trim();

            // Track code blocks
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }

            if in_code_block {
                continue;
            }

            // Extract heading
            if let Some((_level, text)) = self.extract_heading(trimmed) {
                // Skip very short headings (single word)
                if text.split_whitespace().count() < 2 {
                    continue;
                }

                let heading_style = self.detect_style(&text);

                match self.style {
                    CapitalizationStyle::Consistent => {
                        if let Some(expected) = detected_style {
                            if heading_style != expected {
                                let expected_name = match expected {
                                    CapitalizationStyle::TitleCase => "Title Case",
                                    CapitalizationStyle::SentenceCase => "sentence case",
                                    CapitalizationStyle::Consistent => unreachable!(),
                                };
                                violations.push(self.create_violation(
                                    format!(
                                        "Heading '{}' uses inconsistent capitalization (expected {})",
                                        text, expected_name
                                    ),
                                    line_num,
                                    1,
                                    Severity::Warning,
                                ));
                            }
                        } else {
                            detected_style = Some(heading_style);
                        }
                    }
                    CapitalizationStyle::TitleCase => {
                        if !self.is_title_case(&text) {
                            violations.push(self.create_violation(
                                format!("Heading '{}' should use Title Case", text),
                                line_num,
                                1,
                                Severity::Warning,
                            ));
                        }
                    }
                    CapitalizationStyle::SentenceCase => {
                        if !self.is_sentence_case(&text) {
                            violations.push(self.create_violation(
                                format!("Heading '{}' should use sentence case", text),
                                line_num,
                                1,
                                Severity::Warning,
                            ));
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
    use std::path::PathBuf;

    fn create_test_document(content: &str) -> Document {
        Document::new(content.to_string(), PathBuf::from("test.md")).unwrap()
    }

    #[test]
    fn test_consistent_title_case() {
        let content = "# Getting Started Guide

## Installation Steps

### Configuration Options";
        let doc = create_test_document(content);
        let rule = CONTENT004::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_consistent_sentence_case() {
        let content = "# Getting started guide

## Installation steps

### Configuration options";
        let doc = create_test_document(content);
        let rule = CONTENT004::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_inconsistent_capitalization() {
        let content = "# Getting Started Guide

## installation steps

### More Configuration Options";
        let doc = create_test_document(content);
        let rule = CONTENT004::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("installation steps"));
    }

    #[test]
    fn test_enforced_title_case() {
        let content = "# Getting started guide

## Installation Steps";
        let doc = create_test_document(content);
        let rule = CONTENT004::with_style(CapitalizationStyle::TitleCase);
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Getting started guide"));
    }

    #[test]
    fn test_enforced_sentence_case() {
        let content = "# Getting Started Guide

## Installation steps";
        let doc = create_test_document(content);
        let rule = CONTENT004::with_style(CapitalizationStyle::SentenceCase);
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Getting Started Guide"));
    }

    #[test]
    fn test_single_word_headings_ignored() {
        let content = "# Introduction

## Overview

### Details";
        let doc = create_test_document(content);
        let rule = CONTENT004::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_headings_in_code_blocks_ignored() {
        let content = "# Main Title Here

```markdown
# This Is Not a Real Heading
## neither is this
```

## Second Section Here";
        let doc = create_test_document(content);
        let rule = CONTENT004::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_mixed_styles_detected() {
        let content = "# User Guide Introduction

## getting started quickly

### Advanced Configuration";
        let doc = create_test_document(content);
        let rule = CONTENT004::default();
        let violations = rule.check(&doc).unwrap();
        // Should detect that "getting started quickly" doesn't match Title Case
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_is_title_case() {
        let rule = CONTENT004::default();
        assert!(rule.is_title_case("Getting Started Guide"));
        assert!(rule.is_title_case("The Quick Brown Fox"));
        assert!(!rule.is_title_case("getting started guide"));
    }

    #[test]
    fn test_is_sentence_case() {
        let rule = CONTENT004::default();
        assert!(rule.is_sentence_case("Getting started guide"));
        assert!(rule.is_sentence_case("The quick brown fox"));
        assert!(!rule.is_sentence_case("Getting Started Guide"));
    }

    #[test]
    fn test_is_acronym() {
        let rule = CONTENT004::default();
        assert!(rule.is_acronym("API"));
        assert!(rule.is_acronym("HTTP"));
        assert!(rule.is_acronym("REST"));
        assert!(!rule.is_acronym("Api"));
        assert!(!rule.is_acronym("A")); // Single letter not acronym
    }
}
