//! CONTENT009: No excessive heading nesting
//!
//! Warns when heading nesting goes too deep (e.g., beyond h4), which can
//! indicate overly complex document structure that's hard to navigate.

use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use regex::Regex;
use std::sync::LazyLock;

/// Regex to match ATX headings
static HEADING_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(#{1,6})\s+.+").unwrap());

/// Default maximum heading depth
const DEFAULT_MAX_DEPTH: usize = 4;

/// CONTENT009: Detects excessive heading nesting
///
/// Deep heading hierarchies (h5, h6) often indicate that content should be
/// restructured into separate documents or that the hierarchy is too granular.
#[derive(Clone)]
pub struct CONTENT009 {
    /// Maximum allowed heading depth (1-6)
    max_depth: usize,
}

impl Default for CONTENT009 {
    fn default() -> Self {
        Self {
            max_depth: DEFAULT_MAX_DEPTH,
        }
    }
}

impl CONTENT009 {
    /// Create with a custom maximum depth
    #[allow(dead_code)]
    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            max_depth: max_depth.clamp(1, 6),
        }
    }
}

impl Rule for CONTENT009 {
    fn id(&self) -> &'static str {
        "CONTENT009"
    }

    fn name(&self) -> &'static str {
        "no-excessive-nesting"
    }

    fn description(&self) -> &'static str {
        "Heading nesting should not be too deep (default max: h4)"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Content).introduced_in("mdbook-lint v0.14.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> mdbook_lint_core::error::Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let mut in_code_block = false;

        for (line_idx, line) in document.lines.iter().enumerate() {
            let trimmed = line.trim();

            // Track code blocks
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }

            if in_code_block {
                continue;
            }

            // Check for headings
            if let Some(caps) = HEADING_REGEX.captures(trimmed)
                && let Some(hashes) = caps.get(1)
            {
                let depth = hashes.as_str().len();

                if depth > self.max_depth {
                    let line_num = line_idx + 1;
                    violations.push(self.create_violation(
                        format!(
                            "Heading level h{} exceeds maximum depth of h{}. \
                             Consider restructuring to reduce nesting or splitting into separate documents",
                            depth, self.max_depth
                        ),
                        line_num,
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

    fn create_test_document(content: &str) -> Document {
        Document::new(content.to_string(), PathBuf::from("test.md")).unwrap()
    }

    #[test]
    fn test_normal_nesting() {
        let content = "# Chapter

## Section

### Subsection

#### Details";
        let doc = create_test_document(content);
        let rule = CONTENT009::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_excessive_h5() {
        let content = "# Chapter

## Section

### Subsection

#### Details

##### Too Deep";
        let doc = create_test_document(content);
        let rule = CONTENT009::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("h5"));
    }

    #[test]
    fn test_excessive_h6() {
        let content = "# Chapter

###### Way Too Deep";
        let doc = create_test_document(content);
        let rule = CONTENT009::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("h6"));
    }

    #[test]
    fn test_multiple_violations() {
        let content = "# Chapter

##### Deep 1

###### Deep 2";
        let doc = create_test_document(content);
        let rule = CONTENT009::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_custom_max_depth() {
        let content = "# Chapter

## Section

### Too Deep for Custom";
        let doc = create_test_document(content);
        let rule = CONTENT009::with_max_depth(2);
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("h3"));
    }

    #[test]
    fn test_max_depth_h6() {
        let content = "# Chapter

###### This is fine";
        let doc = create_test_document(content);
        let rule = CONTENT009::with_max_depth(6);
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_headings_in_code_blocks_ignored() {
        let content = "# Chapter

```markdown
##### This is in a code block
###### This too
```";
        let doc = create_test_document(content);
        let rule = CONTENT009::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_no_headings() {
        let content = "Just some text without any headings.

More paragraphs here.";
        let doc = create_test_document(content);
        let rule = CONTENT009::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }
}
