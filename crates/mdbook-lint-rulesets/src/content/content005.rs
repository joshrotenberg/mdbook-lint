//! CONTENT005: Introductory paragraph before subheading
//!
//! Ensures chapters have introductory content before the first subheading.
//! Jumping directly to subheadings without context can confuse readers.

use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};
use regex::Regex;
use std::sync::LazyLock;

/// Regex to match ATX headings
static HEADING_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(#{1,6})\s+.+").unwrap());

/// Default minimum words for an introduction
const DEFAULT_MIN_INTRO_WORDS: usize = 10;

/// CONTENT005: Ensures chapters have introductory paragraphs
///
/// This rule checks that there is substantive content between the main
/// heading (H1) and the first subheading (H2+). Chapters that jump
/// directly into subheadings without introduction can be disorienting.
#[derive(Clone)]
pub struct CONTENT005 {
    /// Minimum words required in the introduction
    min_intro_words: usize,
}

impl Default for CONTENT005 {
    fn default() -> Self {
        Self {
            min_intro_words: DEFAULT_MIN_INTRO_WORDS,
        }
    }
}

impl CONTENT005 {
    /// Create with a custom minimum introduction word count
    #[allow(dead_code)]
    pub fn with_min_words(min_words: usize) -> Self {
        Self {
            min_intro_words: min_words,
        }
    }

    /// Get the heading level from a line (1-6, or 0 if not a heading)
    fn get_heading_level(&self, line: &str) -> usize {
        let trimmed = line.trim();
        if let Some(caps) = HEADING_REGEX.captures(trimmed) {
            caps.get(1).map(|m| m.as_str().len()).unwrap_or(0)
        } else {
            0
        }
    }

    /// Count words in a range of lines, excluding special elements
    fn count_content_words(&self, lines: &[String]) -> usize {
        let mut word_count = 0;
        let mut in_code_block = false;

        for line in lines {
            let trimmed = line.trim();

            // Track code blocks
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }

            if in_code_block {
                continue;
            }

            // Skip headings
            if trimmed.starts_with('#') {
                continue;
            }

            // Skip HTML comments
            if trimmed.starts_with("<!--") {
                continue;
            }

            // Skip mdBook directives
            if trimmed.starts_with("{{#") {
                continue;
            }

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            word_count += trimmed.split_whitespace().count();
        }

        word_count
    }
}

impl Rule for CONTENT005 {
    fn id(&self) -> &'static str {
        "CONTENT005"
    }

    fn name(&self) -> &'static str {
        "intro-before-subheading"
    }

    fn description(&self) -> &'static str {
        "Chapters should have introductory content before the first subheading"
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
        let mut h1_line: Option<usize> = None;
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

            let level = self.get_heading_level(trimmed);

            // Track H1
            if level == 1 && h1_line.is_none() {
                h1_line = Some(line_idx);
                continue;
            }

            // Check content before first subheading (H2+)
            if level >= 2 {
                if let Some(h1_idx) = h1_line {
                    // Get content between H1 and this subheading
                    let content_lines = &document.lines[h1_idx + 1..line_idx];
                    let intro_words = self.count_content_words(content_lines);

                    if intro_words < self.min_intro_words {
                        let line_num = line_idx + 1;
                        violations.push(self.create_violation(
                            format!(
                                "Subheading at line {} has insufficient introduction ({} words, minimum: {}). \
                                 Add introductory content after the main heading",
                                line_num, intro_words, self.min_intro_words
                            ),
                            line_num,
                            1,
                            Severity::Warning,
                        ));
                    }
                }
                // Only check the first subheading
                break;
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
    fn test_sufficient_introduction() {
        let content = "# Chapter Title

This chapter covers important topics that you need to understand.
We will explore several key concepts in detail below.

## First Section";
        let doc = create_test_document(content);
        let rule = CONTENT005::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_no_introduction() {
        let content = "# Chapter Title

## First Section";
        let doc = create_test_document(content);
        let rule = CONTENT005::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("0 words"));
    }

    #[test]
    fn test_short_introduction() {
        let content = "# Chapter Title

Brief intro.

## First Section";
        let doc = create_test_document(content);
        let rule = CONTENT005::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("2 words"));
    }

    #[test]
    fn test_custom_min_words() {
        let content = "# Chapter Title

Short intro here.

## First Section";
        let doc = create_test_document(content);

        // Should fail with default (10 words)
        let rule = CONTENT005::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);

        // Should pass with lower threshold
        let rule = CONTENT005::with_min_words(3);
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_no_h1_heading() {
        let content = "## First Section

Some content here.

## Second Section";
        let doc = create_test_document(content);
        let rule = CONTENT005::default();
        let violations = rule.check(&doc).unwrap();
        // No H1, so no check performed
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_no_subheadings() {
        let content = "# Chapter Title

This is a simple chapter with no subheadings.
It just has regular paragraphs of content.";
        let doc = create_test_document(content);
        let rule = CONTENT005::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_code_block_before_subheading() {
        let content = "# Chapter Title

```rust
// This code block should not count as intro
fn main() {}
```

## First Section";
        let doc = create_test_document(content);
        let rule = CONTENT005::default();
        let violations = rule.check(&doc).unwrap();
        // Code blocks don't count toward intro
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_mdbook_directive_doesnt_count() {
        let content = "# Chapter Title

{{#include intro.md}}

## First Section";
        let doc = create_test_document(content);
        let rule = CONTENT005::default();
        let violations = rule.check(&doc).unwrap();
        // Directives don't count as intro content
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_only_first_subheading_checked() {
        let content = "# Chapter Title

This is a proper introduction with enough words to pass.

## First Section

## Second Section";
        let doc = create_test_document(content);
        let rule = CONTENT005::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_h3_as_first_subheading() {
        let content = "# Chapter Title

### Directly to H3";
        let doc = create_test_document(content);
        let rule = CONTENT005::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_heading_in_code_block_ignored() {
        let content = "# Chapter Title

This is the introduction with enough words here to pass the threshold.

```markdown
## This is not a real subheading
```

## Actual Section";
        let doc = create_test_document(content);
        let rule = CONTENT005::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_html_comments_dont_count() {
        let content = "# Chapter Title

<!-- This is a long comment that has many words but should not count -->

## First Section";
        let doc = create_test_document(content);
        let rule = CONTENT005::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_exact_threshold() {
        // Exactly 10 words
        let content = "# Chapter Title

One two three four five six seven eight nine ten.

## First Section";
        let doc = create_test_document(content);
        let rule = CONTENT005::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }
}
