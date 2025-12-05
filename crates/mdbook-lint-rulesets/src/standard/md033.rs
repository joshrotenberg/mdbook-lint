//! MD033: Inline HTML should be avoided
//!
//! This rule checks for inline HTML elements in markdown content, which should
//! generally be avoided in favor of pure Markdown syntax.

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};

/// Rule to detect inline HTML elements
pub struct MD033;

impl AstRule for MD033 {
    fn id(&self) -> &'static str {
        "MD033"
    }

    fn name(&self) -> &'static str {
        "no-inline-html"
    }

    fn description(&self) -> &'static str {
        "Inline HTML should be avoided"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Content).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_ast<'a>(
        &self,
        document: &Document,
        _ast: &'a comrak::nodes::AstNode<'a>,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let lines = &document.lines;

        let mut in_code_block = false;

        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = line_idx + 1;

            // Track fenced code blocks to ignore HTML inside them
            if line.trim_start().starts_with("```") || line.trim_start().starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }

            // Skip lines inside code blocks
            if in_code_block {
                continue;
            }

            // Simple HTML detection without regex
            violations.extend(self.check_line_for_html(line, line_num));
        }

        Ok(violations)
    }
}

impl MD033 {
    /// Check a single line for HTML tags and comments
    fn check_line_for_html(&self, line: &str, line_num: usize) -> Vec<Violation> {
        let mut violations = Vec::new();
        let mut chars = line.char_indices().peekable();
        let mut in_backticks = false;

        while let Some((i, ch)) = chars.next() {
            match ch {
                '`' => {
                    in_backticks = !in_backticks;
                }
                '<' if !in_backticks => {
                    // Look ahead to see if this looks like an HTML tag or comment
                    let remaining = &line[i..];

                    if remaining.starts_with("<!--") {
                        // HTML comment - skip entirely (don't flag comments)
                        if let Some(end) = remaining.find("-->") {
                            // Skip past the comment without flagging
                            for _ in 0..end + 2 {
                                chars.next();
                            }
                        }
                    } else if remaining.starts_with("</") {
                        // Closing tag - skip (don't flag closing tags separately)
                        if let Some(tag_end) = remaining.find('>') {
                            // Skip past the closing tag without flagging
                            for _ in 0..tag_end {
                                chars.next();
                            }
                        }
                    } else if let Some(tag_end) = remaining.find('>') {
                        let potential_tag = &remaining[..tag_end + 1];
                        if self.is_html_tag(potential_tag) {
                            violations.push(self.create_violation(
                                format!("Inline HTML element found: {potential_tag}"),
                                line_num,
                                i + 1,
                                Severity::Warning,
                            ));
                            // Skip past the tag
                            for _ in 0..tag_end {
                                chars.next();
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        violations
    }

    /// Simple check if a string looks like an HTML tag
    fn is_html_tag(&self, s: &str) -> bool {
        if !s.starts_with('<') || !s.ends_with('>') {
            return false;
        }

        let content = &s[1..s.len() - 1];
        if content.is_empty() {
            return false;
        }

        // Handle closing tags
        let tag_name = if let Some(stripped) = content.strip_prefix('/') {
            stripped
        } else {
            content
        }
        .split_whitespace()
        .next()
        .unwrap_or("");

        // List of common HTML tags
        let html_tags = [
            "a",
            "abbr",
            "b",
            "br",
            "cite",
            "code",
            "em",
            "i",
            "img",
            "kbd",
            "mark",
            "q",
            "s",
            "samp",
            "small",
            "span",
            "strong",
            "sub",
            "sup",
            "time",
            "u",
            "var",
            "wbr",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "p",
            "div",
            "section",
            "article",
            "header",
            "footer",
            "nav",
            "aside",
            "main",
            "figure",
            "figcaption",
            "blockquote",
            "pre",
            "ul",
            "ol",
            "li",
            "dl",
            "dt",
            "dd",
            "table",
            "thead",
            "tbody",
            "tfoot",
            "tr",
            "th",
            "td",
            "form",
            "input",
            "button",
            "select",
            "option",
            "textarea",
            "label",
            "fieldset",
            "legend",
        ];

        html_tags.contains(&tag_name.to_lowercase().as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::Document;
    use mdbook_lint_core::rule::Rule;
    use std::path::PathBuf;

    #[test]
    fn test_md033_no_violations() {
        let content = r#"# Valid Markdown

This document contains only valid Markdown:

**Bold text** and *italic text*.

`code spans` are fine.

```html
<p>HTML in code blocks is fine</p>
<div class="example">
    <span>This is ignored</span>
</div>
```

[Links](https://example.com) are good.

> Blockquotes are fine

- List items
- More items

## Another heading

Regular paragraphs without HTML.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD033;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md033_html_violations() {
        let content = r#"# Document with HTML

This paragraph has <strong>inline HTML</strong>.

<p>This is a paragraph tag.</p>

Some text with <em>emphasis</em> and <code>code</code> tags.

<div class="container">
Block level HTML
</div>

More content with <span class="highlight">spans</span>.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD033;
        let violations = rule.check(&document).unwrap();

        // Only opening tags are flagged, not closing tags
        assert_eq!(violations.len(), 6);
        assert!(violations[0].message.contains("<strong>"));
        assert!(violations[1].message.contains("<p>"));
        assert!(violations[2].message.contains("<em>"));
        assert!(violations[3].message.contains("<code>"));
        assert!(violations[4].message.contains("<div"));
        assert!(violations[5].message.contains("<span"));
    }

    #[test]
    fn test_md033_html_comments_not_flagged() {
        // Issue #280: HTML comments should NOT be flagged
        let content = r#"# Document with HTML Comments

This has <!-- a comment --> in it.

Regular text here.

<!-- Another comment -->

<!-- ignore -->
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD033;
        let violations = rule.check(&document).unwrap();

        // HTML comments should not be flagged
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md033_code_blocks_ignored() {
        let content = r#"# Code Blocks Should Be Ignored

```html
<div class="example">
    <p>This HTML should be ignored</p>
    <span>Even this</span>
</div>
```

But this <strong>should be detected</strong>.

```javascript
const html = '<div>This is in JS code</div>';
```

And this <em>should also be detected</em>.

~~~html
<article>
    <header>More HTML to ignore</header>
</article>
~~~
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD033;
        let violations = rule.check(&document).unwrap();

        // Only opening tags flagged, closing tags skipped
        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("<strong>"));
        assert!(violations[1].message.contains("<em>"));
    }

    #[test]
    fn test_md033_inline_code_ignored() {
        let content = r#"# Inline Code Should Be Ignored

This `<span>HTML in backticks</span>` should be ignored.

But this <div>should be detected</div>.

Use `<strong>` tags for bold text, but don't use <strong>actual tags</strong>.

Multiple `<code>` spans with `<em>emphasis</em>` should be ignored.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD033;
        let violations = rule.check(&document).unwrap();

        // Only opening tags flagged, closing tags skipped
        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("<div>"));
        assert!(violations[1].message.contains("<strong>"));
    }

    #[test]
    fn test_md033_mixed_content() {
        let content = r#"# Mixed Content

Regular text with <b>bold HTML</b> tag.

```html
<p>This should be ignored</p>
```

Back to regular content with <i>italic</i>.

The `<em>` tag is mentioned in code, but <em>this usage</em> is flagged.

More `<span class="test">code examples</span>` that should be ignored.

Final <strong>HTML usage</strong> to detect.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD033;
        let violations = rule.check(&document).unwrap();

        // Only opening tags flagged, closing tags skipped
        assert_eq!(violations.len(), 4);
        assert!(violations[0].message.contains("<b>"));
        assert!(violations[1].message.contains("<i>"));
        assert!(violations[2].message.contains("<em>"));
        assert!(violations[3].message.contains("<strong>"));
    }

    #[test]
    fn test_md033_closing_tags_not_flagged() {
        // Issue #280: Closing tags should NOT be flagged separately
        let content = r#"# Document with Closing Tags

Text with </div> orphan closing tag.

And </span> another one.

Proper usage: <strong>bold</strong> text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD033;
        let violations = rule.check(&document).unwrap();

        // Only the opening <strong> tag should be flagged
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("<strong>"));
    }
}
