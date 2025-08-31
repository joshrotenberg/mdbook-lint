//! MD025: Single H1 per document
//!
//! This rule checks that a document has only one top-level heading (H1).

use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Fix, Position, Severity, Violation},
};

/// Rule to check that documents have only one H1 heading
pub struct MD025 {
    /// The heading level to check (default: 1)
    level: u8,
}

impl MD025 {
    /// Create a new MD025 rule with default settings (level 1)
    pub fn new() -> Self {
        Self { level: 1 }
    }

    /// Create a new MD025 rule with custom level
    #[allow(dead_code)]
    pub fn with_level(level: u8) -> Self {
        Self { level }
    }

    /// Create MD025 from configuration
    pub fn from_config(config: &toml::Value) -> Self {
        let mut rule = Self::new();

        if let Some(level) = config.get("level").and_then(|v| v.as_integer()) {
            rule.level = level as u8;
        }

        rule
    }
}

impl Default for MD025 {
    fn default() -> Self {
        Self::new()
    }
}

impl AstRule for MD025 {
    fn id(&self) -> &'static str {
        "MD025"
    }

    fn name(&self) -> &'static str {
        "single-title"
    }

    fn description(&self) -> &'static str {
        "Multiple top-level headings in the same document"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Structure).introduced_in("mdbook-lint v0.1.0")
    }

    fn can_fix(&self) -> bool {
        true
    }

    fn check_ast<'a>(&self, document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let mut h1_headings = Vec::new();

        // Find all headings at the specified level
        for node in ast.descendants() {
            if let NodeValue::Heading(heading) = &node.data.borrow().value
                && heading.level == self.level
                && let Some((line, column)) = document.node_position(node)
            {
                let heading_text = document.node_text(node);
                let heading_text = heading_text.trim();
                h1_headings.push((line, column, heading_text.to_string()));
            }
        }

        // If we have more than one H1, create violations for all but the first
        if h1_headings.len() > 1 {
            for (_i, (line, column, heading_text)) in h1_headings.iter().enumerate().skip(1) {
                // Create fix by demoting to level 2
                let line_content = &document.lines[*line - 1];
                let new_level = self.level + 1;

                let fixed_line = if line_content.trim_start().starts_with('#') {
                    // ATX heading - add one more hash
                    let trimmed = line_content.trim_start();
                    let content_start = trimmed.find(|c: char| c != '#').unwrap_or(trimmed.len());
                    let heading_content = if content_start < trimmed.len() {
                        &trimmed[content_start..]
                    } else {
                        ""
                    };
                    format!("{}{}\n", "#".repeat(new_level as usize), heading_content)
                } else {
                    // Setext heading - convert to ATX at level 2
                    format!("{} {}\n", "#".repeat(new_level as usize), heading_text)
                };

                let fix = Fix {
                    description: format!("Demote heading to level {}", new_level),
                    replacement: Some(fixed_line),
                    start: Position {
                        line: *line,
                        column: 1,
                    },
                    end: Position {
                        line: *line,
                        column: line_content.len() + 1,
                    },
                };

                violations.push(self.create_violation_with_fix(
                    format!(
                        "Multiple top-level headings in the same document (first at line {}): {}",
                        h1_headings[0].0, heading_text
                    ),
                    *line,
                    *column,
                    Severity::Error,
                    fix,
                ));
            }
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::Document;
    use mdbook_lint_core::rule::Rule;
    use std::path::PathBuf;

    #[test]
    fn test_md025_single_h1() {
        let content = r#"# Single H1 heading
## H2 heading
### H3 heading
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD025::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md025_multiple_h1_violation() {
        let content = r#"# First H1 heading
Some content here.

# Second H1 heading
More content.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD025::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("Multiple top-level headings")
        );
        assert!(violations[0].message.contains("first at line 1"));
        assert!(violations[0].message.contains("Second H1 heading"));
        assert_eq!(violations[0].line, 4);
    }

    #[test]
    fn test_md025_three_h1_violations() {
        let content = r#"# First H1
Content here.

# Second H1
More content.

# Third H1
Even more content.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD025::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);

        // Both violations should reference the first H1 at line 1
        assert!(violations[0].message.contains("first at line 1"));
        assert!(violations[1].message.contains("first at line 1"));

        // Check violation lines
        assert_eq!(violations[0].line, 4); // Second H1
        assert_eq!(violations[1].line, 7); // Third H1
    }

    #[test]
    fn test_md025_no_h1_headings() {
        let content = r#"## H2 heading
### H3 heading
#### H4 heading
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD025::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md025_setext_headings() {
        let content = r#"First H1 Setext
===============

Second H1 Setext
================
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD025::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("first at line 1"));
        assert!(violations[0].message.contains("Second H1 Setext"));
        assert_eq!(violations[0].line, 4);
    }

    #[test]
    fn test_md025_mixed_atx_setext() {
        let content = r#"# ATX H1 heading

Setext H1 heading
=================
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD025::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("first at line 1"));
        assert!(violations[0].message.contains("Setext H1 heading"));
        assert_eq!(violations[0].line, 3);
    }

    #[test]
    fn test_md025_custom_level() {
        let content = r#"# H1 heading
## First H2 heading
### H3 heading
## Second H2 heading
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD025::with_level(2);
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("first at line 2"));
        assert!(violations[0].message.contains("Second H2 heading"));
        assert_eq!(violations[0].line, 4);
    }

    #[test]
    fn test_md025_h1_with_other_levels() {
        let content = r#"# Main heading
## Introduction
### Details
## Conclusion
### More details
#### Sub-details
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD025::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md025_empty_h1_headings() {
        let content = r#"#
Content here.

#
More content.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD025::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 4);
    }

    #[test]
    fn test_md025_h1_in_code_blocks() {
        let content = r#"# Real H1 heading

```markdown
# Fake H1 in code block
```

Some content.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD025::new();
        let violations = rule.check(&document).unwrap();

        // Should not detect the H1 in the code block
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md025_regular_file_still_triggers() {
        let content = r#"# First H1 heading
Some content here.

# Second H1 heading
More content.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("chapter.md")).unwrap();
        let rule = MD025::new();
        let violations = rule.check(&document).unwrap();

        // Regular files should still trigger MD025 violations
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0]
                .message
                .contains("Multiple top-level headings")
        );
    }

    #[test]
    fn test_md025_fix_multiple_h1() {
        let content = r#"# First Title
## Section
# Second Title
# Third Title"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD025::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2); // Second and third H1s

        // First extra H1 gets demoted to H2
        assert!(violations[0].fix.is_some());
        let fix1 = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix1.description, "Demote heading to level 2");
        assert_eq!(fix1.replacement, Some("## Second Title\n".to_string()));

        // Second extra H1 also gets demoted to H2
        assert!(violations[1].fix.is_some());
        let fix2 = violations[1].fix.as_ref().unwrap();
        assert_eq!(fix2.description, "Demote heading to level 2");
        assert_eq!(fix2.replacement, Some("## Third Title\n".to_string()));
    }

    #[test]
    fn test_md025_can_fix() {
        let rule = MD025::new();
        assert!(mdbook_lint_core::AstRule::can_fix(&rule));
    }
}
