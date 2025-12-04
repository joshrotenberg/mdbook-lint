//! CONTENT003: Short chapter detection
//!
//! Flags chapters that are too short and might be stubs or incomplete.
//! Short chapters can indicate work-in-progress content that needs expansion.

use mdbook_lint_core::Document;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Severity, Violation};

/// Default minimum word count for a chapter
const DEFAULT_MIN_WORDS: usize = 50;

/// CONTENT003: Detects overly short chapters
///
/// This rule flags chapters that have fewer than a configurable minimum
/// number of words (default: 50). Short chapters often indicate:
/// - Stub content that needs expansion
/// - Placeholder sections
/// - Incomplete documentation
#[derive(Clone)]
pub struct CONTENT003 {
    /// Minimum word count threshold
    min_words: usize,
    /// Whether to count words in code blocks
    include_code_blocks: bool,
}

impl Default for CONTENT003 {
    fn default() -> Self {
        Self {
            min_words: DEFAULT_MIN_WORDS,
            include_code_blocks: false,
        }
    }
}

impl CONTENT003 {
    /// Create with a custom minimum word count
    #[allow(dead_code)]
    pub fn with_min_words(min_words: usize) -> Self {
        Self {
            min_words,
            include_code_blocks: false,
        }
    }

    /// Set whether to include code blocks in word count
    #[allow(dead_code)]
    pub fn include_code_blocks(mut self, include: bool) -> Self {
        self.include_code_blocks = include;
        self
    }

    /// Count words in content, optionally excluding code blocks
    fn count_words(&self, lines: &[String]) -> usize {
        let mut word_count = 0;
        let mut in_code_block = false;

        for line in lines {
            let trimmed = line.trim();

            // Track fenced code blocks
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }

            // Skip code block content unless configured to include
            if in_code_block && !self.include_code_blocks {
                continue;
            }

            // Skip headings from word count (they're structural, not content)
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

            // Count words in this line
            word_count += trimmed.split_whitespace().count();
        }

        word_count
    }
}

impl Rule for CONTENT003 {
    fn id(&self) -> &'static str {
        "CONTENT003"
    }

    fn name(&self) -> &'static str {
        "no-short-chapters"
    }

    fn description(&self) -> &'static str {
        "Chapters should have sufficient content (minimum word count)"
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

        let word_count = self.count_words(&document.lines);

        if word_count < self.min_words {
            violations.push(self.create_violation(
                format!(
                    "Chapter has only {} words (minimum: {}). Consider expanding the content or marking as draft",
                    word_count, self.min_words
                ),
                1,
                1,
                Severity::Warning,
            ));
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
    fn test_sufficient_content() {
        // 50+ words of content
        let content = "# Chapter Title

This is a paragraph with enough content to pass the minimum word count threshold.
We need to write several sentences here to make sure we have at least fifty words
in total. Let me add some more text to ensure we definitely pass the check.
Here is another sentence. And another one. Plus a few more words to be safe.";
        let doc = create_test_document(content);
        let rule = CONTENT003::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_short_chapter() {
        let content = "# Short Chapter

This is too short.";
        let doc = create_test_document(content);
        let rule = CONTENT003::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("words"));
    }

    #[test]
    fn test_empty_chapter() {
        let content = "# Empty Chapter";
        let doc = create_test_document(content);
        let rule = CONTENT003::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("0 words"));
    }

    #[test]
    fn test_custom_threshold() {
        let content = "# Chapter

This has about ten words of content here.";
        let doc = create_test_document(content);

        // Should fail with default (50 words)
        let rule = CONTENT003::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);

        // Should pass with lower threshold
        let rule = CONTENT003::with_min_words(5);
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_code_blocks_excluded_by_default() {
        let content = "# Chapter

Short intro.

```rust
fn main() {
    // This is a lot of code that adds many words
    // but should not count toward the word total
    // because code blocks are excluded by default
    let x = 1;
    let y = 2;
    let z = x + y;
    println!(\"Result: {}\", z);
}
```";
        let doc = create_test_document(content);
        let rule = CONTENT003::default();
        let violations = rule.check(&doc).unwrap();
        // Should be short because code block words don't count
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_code_blocks_included_when_configured() {
        let content = "# Chapter

Short intro but lots of code comments.

```rust
// This is a very long comment that contains many words
// and should be counted when we configure the rule to
// include code blocks in the word count which brings
// our total word count up significantly higher than
// the minimum threshold of fifty words required
fn main() {
    println!(\"Hello\");
}
```";
        let doc = create_test_document(content);
        let rule = CONTENT003::default().include_code_blocks(true);
        let violations = rule.check(&doc).unwrap();
        // Should pass because code block words count
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_headings_not_counted() {
        let content = "# This Heading Has Many Words In It

## Another Long Heading With Words

### Yet Another Heading

#### And One More

Short body.";
        let doc = create_test_document(content);
        let rule = CONTENT003::default();
        let violations = rule.check(&doc).unwrap();
        // Should be short because headings don't count
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_mdbook_directives_not_counted() {
        let content = "# Chapter

{{#include file.rs}}
{{#playground example.rs}}
{{#title Some Title}}

Short content.";
        let doc = create_test_document(content);
        let rule = CONTENT003::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_just_at_threshold() {
        // Exactly 50 words
        let words: Vec<&str> = (0..50).map(|_| "word").collect();
        let content = format!("# Chapter\n\n{}", words.join(" "));
        let doc = create_test_document(&content);
        let rule = CONTENT003::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_just_below_threshold() {
        // 49 words
        let words: Vec<&str> = (0..49).map(|_| "word").collect();
        let content = format!("# Chapter\n\n{}", words.join(" "));
        let doc = create_test_document(&content);
        let rule = CONTENT003::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_html_comments_not_counted() {
        let content = "# Chapter

<!-- This is a long HTML comment with many words that should not be counted -->

Short visible content.";
        let doc = create_test_document(content);
        let rule = CONTENT003::default();
        let violations = rule.check(&doc).unwrap();
        assert_eq!(violations.len(), 1);
    }
}
