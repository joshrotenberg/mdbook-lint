use comrak::nodes::AstNode;
use mdbook_lint_core::error::Result;
use mdbook_lint_core::{
    Document,
    rule::{AstRule, RuleCategory, RuleMetadata},
    violation::{Severity, Violation},
};

/// MD001: Heading levels should only increment by one level at a time
///
/// This rule is triggered when you skip heading levels in a markdown document.
/// For example, a heading level 1 should be followed by level 2, not level 3.
///
/// ## Why This Rule Exists
///
/// Proper heading hierarchy improves document structure, accessibility, and navigation.
/// Screen readers and document outlines rely on sequential heading levels to convey
/// the document's organization to users.
///
/// ## Examples
///
/// ### ❌ Incorrect (violates rule)
///
/// ```markdown
/// # Title
///
/// ### Subsection (skips h2)
///
/// ## Back to h2
///
/// ##### Deep section (skips h3 and h4)
/// ```
///
/// ### ✅ Correct
///
/// ```markdown
/// # Title
///
/// ## Section
///
/// ### Subsection
///
/// #### Subsubsection
///
/// ##### Deep section
/// ```
///
/// ## Configuration
///
/// This rule has no configuration options. It always enforces strict sequential heading levels.
///
/// ## When to Disable
///
/// Consider disabling this rule if:
/// - You're working with generated content that doesn't follow strict hierarchy
/// - You're importing documentation from external sources with different conventions
/// - Your project has specific heading level requirements
pub struct MD001;

impl AstRule for MD001 {
    fn id(&self) -> &'static str {
        "MD001"
    }

    fn name(&self) -> &'static str {
        "heading-increment"
    }

    fn description(&self) -> &'static str {
        "Heading levels should only increment by one level at a time"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Structure).introduced_in("markdownlint v0.1.0")
    }

    fn check_ast<'a>(&self, document: &Document, ast: &'a AstNode<'a>) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let headings = document.headings(ast);

        if headings.is_empty() {
            return Ok(violations);
        }

        let mut previous_level = 0u32;

        for heading in headings {
            if let Some(level) = Document::heading_level(heading) {
                // First heading can be any level
                if previous_level == 0 {
                    previous_level = level;
                    continue;
                }

                // Check if we've skipped levels
                if level > previous_level + 1 {
                    let (line, column) = document.node_position(heading).unwrap_or((1, 1));

                    let heading_text = document.node_text(heading);
                    let message = format!(
                        "Expected heading level {} (max {}) but got level {}{}",
                        previous_level + 1,
                        previous_level + 1,
                        level,
                        if heading_text.is_empty() {
                            String::new()
                        } else {
                            format!(": {}", heading_text.trim())
                        }
                    );

                    violations.push(self.create_violation(message, line, column, Severity::Error));
                }

                previous_level = level;
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

    #[test]
    fn test_md001_valid_sequence() {
        let content = r#"# Level 1
## Level 2
### Level 3
## Level 2 again
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD001;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md001_skip_level() {
        let content = r#"# Level 1
### Level 3 - skipped level 2
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD001;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD001");
        assert_eq!(violations[0].line, 2);
        assert_eq!(violations[0].severity, Severity::Error);
        assert!(violations[0].message.contains("Expected heading level 2"));
        assert!(violations[0].message.contains("got level 3"));
    }

    #[test]
    fn test_md001_multiple_skips() {
        let content = r#"# Level 1
#### Level 4 - skipped levels 2 and 3
## Level 2
##### Level 5 - skipped level 4
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD001;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);

        // First violation: level 1 to level 4
        assert_eq!(violations[0].line, 2);
        assert!(violations[0].message.contains("Expected heading level 2"));
        assert!(violations[0].message.contains("got level 4"));

        // Second violation: level 2 to level 5
        assert_eq!(violations[1].line, 4);
        assert!(violations[1].message.contains("Expected heading level 3"));
        assert!(violations[1].message.contains("got level 5"));
    }

    #[test]
    fn test_md001_decrease_is_ok() {
        let content = r#"# Level 1
## Level 2
### Level 3
# Level 1 again - this is OK
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD001;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md001_no_headings() {
        let content = "Just some text without headings.";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD001;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md001_single_heading() {
        let content = "### Starting with level 3";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD001;
        let violations = rule.check(&document).unwrap();

        // Single heading is always OK, regardless of level
        assert_eq!(violations.len(), 0);
    }
}
