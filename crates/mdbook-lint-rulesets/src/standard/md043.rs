//! MD043: Required heading structure
//!
//! This rule checks that headings follow a required structure/hierarchy pattern.

use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};

/// Rule to check required heading structure
pub struct MD043 {
    /// Required heading patterns
    headings: Vec<String>,
}

impl MD043 {
    /// Create a new MD043 rule with default heading structure
    pub fn new() -> Self {
        Self {
            headings: Vec::new(), // No required structure by default
        }
    }

    /// Create a new MD043 rule with required heading structure
    #[allow(dead_code)]
    pub fn with_headings(headings: Vec<String>) -> Self {
        Self { headings }
    }

    /// Create MD043 from configuration
    pub fn from_config(config: &toml::Value) -> Self {
        let mut rule = Self::new();

        if let Some(headings_value) = config.get("headings")
            && let Some(headings_array) = headings_value.as_array()
        {
            rule.headings = headings_array
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();
        }

        rule
    }

    /// Get line and column position for a node
    fn get_position<'a>(&self, node: &'a AstNode<'a>) -> (usize, usize) {
        let data = node.data.borrow();
        let pos = data.sourcepos;
        (pos.start.line, pos.start.column)
    }

    /// Extract text content from a heading node
    fn extract_heading_text<'a>(&self, node: &'a AstNode<'a>) -> String {
        let mut text = String::new();
        Self::collect_text_content(node, &mut text);
        text
    }

    /// Recursively collect text content from a node and its children
    fn collect_text_content<'a>(node: &'a AstNode<'a>, text: &mut String) {
        match &node.data.borrow().value {
            NodeValue::Text(t) => text.push_str(t),
            NodeValue::Code(code) => text.push_str(&code.literal),
            _ => {}
        }

        for child in node.children() {
            Self::collect_text_content(child, text);
        }
    }

    /// Check if a heading text matches a required pattern
    fn matches_pattern(&self, heading_text: &str, pattern: &str) -> bool {
        // For now, implement exact match (case-insensitive)
        // Could be extended to support regex patterns in the future
        heading_text.trim().to_lowercase() == pattern.trim().to_lowercase()
    }

    /// Walk AST and collect headings, then validate structure
    fn check_node<'a>(&self, node: &'a AstNode<'a>, headings: &mut Vec<(usize, String, usize)>) {
        if let NodeValue::Heading(heading_data) = &node.data.borrow().value {
            let (line, _) = self.get_position(node);
            let text = self.extract_heading_text(node);
            headings.push((line, text, heading_data.level as usize));
        }

        // Recursively check children
        for child in node.children() {
            self.check_node(child, headings);
        }
    }
}

impl Default for MD043 {
    fn default() -> Self {
        Self::new()
    }
}

impl AstRule for MD043 {
    fn id(&self) -> &'static str {
        "MD043"
    }

    fn name(&self) -> &'static str {
        "required-headings"
    }

    fn description(&self) -> &'static str {
        "Required heading structure"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Structure).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_ast<'a>(&self, _document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // If no required structure is configured, skip checking
        if self.headings.is_empty() {
            return Ok(violations);
        }

        let mut document_headings = Vec::new();
        self.check_node(ast, &mut document_headings);

        // Check if document has the required number of headings
        if document_headings.len() < self.headings.len() {
            violations.push(self.create_violation(
                format!(
                    "Document should have at least {} headings but found {}",
                    self.headings.len(),
                    document_headings.len()
                ),
                1,
                1,
                Severity::Warning,
            ));
            return Ok(violations);
        }

        // Check each required heading
        for (i, required_heading) in self.headings.iter().enumerate() {
            if i < document_headings.len() {
                let (line, actual_text, _level) = &document_headings[i];
                if !self.matches_pattern(actual_text, required_heading) {
                    violations.push(self.create_violation(
                        format!("Expected heading '{required_heading}' but found '{actual_text}'"),
                        *line,
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
    use mdbook_lint_core::rule::Rule;
    use std::path::PathBuf;

    fn create_test_document(content: &str) -> Document {
        Document::new(content.to_string(), PathBuf::from("test.md")).unwrap()
    }

    #[test]
    fn test_md043_no_required_structure() {
        let content = r#"# Any Heading

## Any Subheading

### Any Sub-subheading
"#;

        let document = create_test_document(content);
        let rule = MD043::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0); // No requirements, so no violations
    }

    #[test]
    fn test_md043_correct_structure() {
        let content = r#"# Introduction

## Getting Started

## Configuration
"#;

        let required_headings = vec![
            "Introduction".to_string(),
            "Getting Started".to_string(),
            "Configuration".to_string(),
        ];

        let document = create_test_document(content);
        let rule = MD043::with_headings(required_headings);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md043_incorrect_heading_text() {
        let content = r#"# Introduction

## Getting Started

## Setup
"#;

        let required_headings = vec![
            "Introduction".to_string(),
            "Getting Started".to_string(),
            "Configuration".to_string(),
        ];

        let document = create_test_document(content);
        let rule = MD043::with_headings(required_headings);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD043");
        assert!(
            violations[0]
                .message
                .contains("Expected heading 'Configuration' but found 'Setup'")
        );
        assert_eq!(violations[0].line, 5);
    }

    #[test]
    fn test_md043_missing_headings() {
        let content = r#"# Introduction

## Getting Started
"#;

        let required_headings = vec![
            "Introduction".to_string(),
            "Getting Started".to_string(),
            "Configuration".to_string(),
        ];

        let document = create_test_document(content);
        let rule = MD043::with_headings(required_headings);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("should have at least 3 headings but found 2")
        );
    }

    #[test]
    fn test_md043_case_insensitive_matching() {
        let content = r#"# INTRODUCTION

## getting started

## Configuration
"#;

        let required_headings = vec![
            "Introduction".to_string(),
            "Getting Started".to_string(),
            "Configuration".to_string(),
        ];

        let document = create_test_document(content);
        let rule = MD043::with_headings(required_headings);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0); // Case-insensitive matching should work
    }

    #[test]
    fn test_md043_extra_headings_allowed() {
        let content = r#"# Introduction

## Getting Started

## Configuration

## Advanced Topics

### Customization
"#;

        let required_headings = vec![
            "Introduction".to_string(),
            "Getting Started".to_string(),
            "Configuration".to_string(),
        ];

        let document = create_test_document(content);
        let rule = MD043::with_headings(required_headings);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0); // Extra headings are allowed
    }

    #[test]
    fn test_md043_first_heading_wrong() {
        let content = r#"# Overview

## Getting Started

## Configuration
"#;

        let required_headings = vec![
            "Introduction".to_string(),
            "Getting Started".to_string(),
            "Configuration".to_string(),
        ];

        let document = create_test_document(content);
        let rule = MD043::with_headings(required_headings);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("Expected heading 'Introduction' but found 'Overview'")
        );
        assert_eq!(violations[0].line, 1);
    }

    #[test]
    fn test_md043_multiple_violations() {
        let content = r#"# Overview

## Setup

## Deployment
"#;

        let required_headings = vec![
            "Introduction".to_string(),
            "Getting Started".to_string(),
            "Configuration".to_string(),
        ];

        let document = create_test_document(content);
        let rule = MD043::with_headings(required_headings);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 3); // All three headings are wrong
        assert!(
            violations[0]
                .message
                .contains("Expected heading 'Introduction' but found 'Overview'")
        );
        assert!(
            violations[1]
                .message
                .contains("Expected heading 'Getting Started' but found 'Setup'")
        );
        assert!(
            violations[2]
                .message
                .contains("Expected heading 'Configuration' but found 'Deployment'")
        );
    }

    #[test]
    fn test_md043_headings_with_formatting() {
        let content = r#"# **Introduction**

## *Getting Started*

## Configuration
"#;

        let required_headings = vec![
            "Introduction".to_string(),
            "Getting Started".to_string(),
            "Configuration".to_string(),
        ];

        let document = create_test_document(content);
        let rule = MD043::with_headings(required_headings);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0); // Should extract text content ignoring formatting
    }

    #[test]
    fn test_md043_headings_with_code() {
        let content = r#"# Introduction

## Getting Started with `npm`

## Configuration
"#;

        let required_headings = vec![
            "Introduction".to_string(),
            "Getting Started with npm".to_string(),
            "Configuration".to_string(),
        ];

        let document = create_test_document(content);
        let rule = MD043::with_headings(required_headings);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md043_whitespace_handling() {
        let content = r#"#   Introduction

##    Getting Started

##  Configuration
"#;

        let required_headings = vec![
            "Introduction".to_string(),
            "Getting Started".to_string(),
            "Configuration".to_string(),
        ];

        let document = create_test_document(content);
        let rule = MD043::with_headings(required_headings);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0); // Should handle whitespace properly
    }
}
