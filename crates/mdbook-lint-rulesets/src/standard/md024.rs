//! MD024: Multiple headings with the same content
//!
//! This rule checks that headings with the same content are not duplicated within the document.

use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};
use std::collections::HashMap;

/// Rule to check for duplicate headings
pub struct MD024 {
    /// Only check headings at the same level (default: false)
    siblings_only: bool,
}

impl MD024 {
    /// Create a new MD024 rule with default settings
    pub fn new() -> Self {
        Self {
            siblings_only: false,
        }
    }

    /// Create a new MD024 rule with custom settings
    #[allow(dead_code)]
    pub fn with_siblings_only(siblings_only: bool) -> Self {
        Self { siblings_only }
    }

    /// Create MD024 from configuration
    pub fn from_config(config: &toml::Value) -> Self {
        let mut rule = Self::new();

        if let Some(siblings_only) = config.get("siblings_only").and_then(|v| v.as_bool()) {
            rule.siblings_only = siblings_only;
        } else if let Some(siblings_only) = config.get("siblings-only").and_then(|v| v.as_bool()) {
            rule.siblings_only = siblings_only;
        }

        rule
    }
}

impl Default for MD024 {
    fn default() -> Self {
        Self::new()
    }
}

impl AstRule for MD024 {
    fn id(&self) -> &'static str {
        "MD024"
    }

    fn name(&self) -> &'static str {
        "no-duplicate-heading"
    }

    fn description(&self) -> &'static str {
        "Multiple headings with the same content"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Content).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_ast<'a>(&self, document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        if self.siblings_only {
            // Check for duplicates only at the same heading level
            self.check_siblings_only(document, ast, &mut violations)?;
        } else {
            // Check for duplicates across all heading levels
            self.check_all_levels(document, ast, &mut violations)?;
        }

        Ok(violations)
    }
}

impl MD024 {
    /// Check for duplicate headings across all levels
    fn check_all_levels<'a>(
        &self,
        document: &Document,
        ast: &'a AstNode<'a>,
        violations: &mut Vec<Violation>,
    ) -> Result<()> {
        let mut seen_headings: HashMap<String, (usize, usize)> = HashMap::new();

        for node in ast.descendants() {
            if let NodeValue::Heading(_heading) = &node.data.borrow().value
                && let Some((line, column)) = document.node_position(node)
            {
                let heading_text = document.node_text(node);
                let heading_text = heading_text.trim();

                // Skip empty headings
                if heading_text.is_empty() {
                    continue;
                }

                // Normalize heading text for comparison (case-insensitive, whitespace normalized)
                let normalized_text = self.normalize_heading_text(heading_text);

                if let Some((first_line, _first_column)) = seen_headings.get(&normalized_text) {
                    violations.push(self.create_violation(
                        format!(
                            "Duplicate heading content: '{heading_text}' (first occurrence at line {first_line})"
                        ),
                        line,
                        column,
                        Severity::Warning,
                    ));
                } else {
                    seen_headings.insert(normalized_text, (line, column));
                }
            }
        }

        Ok(())
    }

    /// Check for duplicate headings only at the same level
    fn check_siblings_only<'a>(
        &self,
        document: &Document,
        ast: &'a AstNode<'a>,
        violations: &mut Vec<Violation>,
    ) -> Result<()> {
        // Group headings by level, then check for duplicates within each level
        let mut headings_by_level: HashMap<u8, HashMap<String, (usize, usize)>> = HashMap::new();

        for node in ast.descendants() {
            if let NodeValue::Heading(heading) = &node.data.borrow().value
                && let Some((line, column)) = document.node_position(node)
            {
                let heading_text = document.node_text(node);
                let heading_text = heading_text.trim();

                // Skip empty headings
                if heading_text.is_empty() {
                    continue;
                }

                let level = heading.level;
                let normalized_text = self.normalize_heading_text(heading_text);

                let level_map = headings_by_level.entry(level).or_default();

                if let Some((first_line, _first_column)) = level_map.get(&normalized_text) {
                    violations.push(self.create_violation(
                        format!(
                            "Duplicate heading content at level {level}: '{heading_text}' (first occurrence at line {first_line})"
                        ),
                        line,
                        column,
                        Severity::Warning,
                    ));
                } else {
                    level_map.insert(normalized_text, (line, column));
                }
            }
        }

        Ok(())
    }

    /// Normalize heading text for comparison
    fn normalize_heading_text(&self, text: &str) -> String {
        // Convert to lowercase and normalize whitespace for comparison
        text.to_lowercase()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::Document;
    use mdbook_lint_core::rule::Rule;
    use std::path::PathBuf;

    #[test]
    fn test_md024_no_violations() {
        let content = r#"# Unique First Heading
## Unique Second Heading
### Unique Third Heading
## Another Unique Second Heading
### Another Unique Third Heading
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD024::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md024_duplicate_headings_violation() {
        let content = r#"# Introduction
## Getting Started
### Installation
## Getting Started
### Configuration
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD024::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Duplicate heading content"));
        assert!(violations[0].message.contains("Getting Started"));
        assert!(violations[0].message.contains("first occurrence at line 2"));
        assert_eq!(violations[0].line, 4);
    }

    #[test]
    fn test_md024_case_insensitive_duplicates() {
        let content = r#"# Getting Started
## Configuration
### getting started
## CONFIGURATION
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD024::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("getting started"));
        assert!(violations[1].message.contains("CONFIGURATION"));
    }

    #[test]
    fn test_md024_whitespace_normalization() {
        let content = r#"# Getting   Started
## Multiple    Spaces
### Getting Started
## Multiple Spaces
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD024::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("Getting Started"));
        assert!(violations[1].message.contains("Multiple Spaces"));
    }

    #[test]
    fn test_md024_siblings_only_mode() {
        let content = r#"# Main Heading
## Introduction
### Introduction
## Configuration
### Configuration
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD024::with_siblings_only(true);
        let violations = rule.check(&document).unwrap();

        // Should only detect duplicates at the same level
        // Both "Introduction" headings are at different levels (## vs ###), so no violations
        // Both "Configuration" headings are at different levels (## vs ###), so no violations
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md024_siblings_only_with_same_level_duplicates() {
        let content = r#"# Main Heading
## Introduction
## Configuration
## Introduction
### Different Level Introduction
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD024::with_siblings_only(true);
        let violations = rule.check(&document).unwrap();

        // Should detect the duplicate "Introduction" at level 2, but ignore the level 3 one
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("Duplicate heading content at level 2")
        );
        assert!(violations[0].message.contains("Introduction"));
        assert_eq!(violations[0].line, 4);
    }

    #[test]
    fn test_md024_multiple_duplicates() {
        let content = r#"# Main
## Section A
### Subsection
## Section B
### Subsection
## Section A
### Another Subsection
### Subsection
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD024::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);

        // Check that all duplicates are detected
        let messages: Vec<&str> = violations.iter().map(|v| v.message.as_str()).collect();
        assert!(
            messages
                .iter()
                .any(|m| m.contains("Subsection") && m.contains("line 3"))
        );
        assert!(
            messages
                .iter()
                .any(|m| m.contains("Section A") && m.contains("line 2"))
        );
        assert!(
            messages
                .iter()
                .any(|m| m.contains("Subsection") && m.contains("line 3"))
        );
    }

    #[test]
    fn test_md024_empty_headings_ignored() {
        let content = r#"# Main Heading
##
###
## Valid Heading
###
## Valid Heading
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD024::new();
        let violations = rule.check(&document).unwrap();

        // Should only detect the duplicate "Valid Heading", not the empty ones
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Valid Heading"));
    }

    #[test]
    fn test_md024_mixed_heading_types() {
        let content = r#"# ATX Heading

Setext Heading
==============

## Another Section

ATX Heading
-----------

### Final Section
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD024::new();
        let violations = rule.check(&document).unwrap();

        // Should detect duplicate "ATX Heading" regardless of heading style
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("ATX Heading"));
    }

    #[test]
    fn test_md024_headings_with_formatting() {
        let content = r#"# Introduction to **Markdown**
## Getting Started
### Introduction to Markdown
## *Getting* Started
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD024::new();
        let violations = rule.check(&document).unwrap();

        // Should detect duplicates based on text content, ignoring markdown formatting
        // document.node_text() correctly extracts plain text without formatting markers
        assert_eq!(violations.len(), 2); // Both pairs are duplicates when formatting is ignored
        assert!(violations[0].message.contains("Introduction to Markdown"));
        assert!(violations[1].message.contains("Getting Started"));
    }

    #[test]
    fn test_md024_long_document_with_sections() {
        let content = r#"# User Guide

## Installation
### Prerequisites
### Download
### Setup

## Configuration
### Basic Settings
### Advanced Settings

## Usage
### Getting Started
### Advanced Features

## Troubleshooting
### Common Issues
### Getting Started

## Reference
### API Documentation
### Configuration
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD024::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);

        // Should detect "Getting Started" and "Configuration" duplicates
        let violation_texts: Vec<String> = violations.iter().map(|v| v.message.clone()).collect();
        assert!(
            violation_texts
                .iter()
                .any(|m| m.contains("Getting Started"))
        );
        assert!(violation_texts.iter().any(|m| m.contains("Configuration")));
    }
}
