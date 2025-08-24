//! MD052 - Reference links and images should use a label that is defined
//!
//! This rule checks for reference links and images that use undefined labels.
//! Uses byte-by-byte parsing for accurate context-aware detection.
//!
//! ## Correct
//!
//! ```markdown
//! [Link text][label]
//! [Collapsed][]
//!
//! [label]: https://example.com
//! [collapsed]: https://example.com
//! ```
//!
//! ## Incorrect
//!
//! ```markdown
//! [Link text][undefined-label]
//! [Collapsed][]
//!
//! [defined]: https://example.com
//! ```

use mdbook_lint_core::error::Result;
use mdbook_lint_core::{
    Document, Violation,
    rule::{Rule, RuleCategory, RuleMetadata},
    violation::Severity,
};
use comrak::nodes::AstNode;
use std::collections::HashSet;

/// MD052 - Reference links and images should use a label that is defined
pub struct MD052 {
    ignored_labels: Vec<String>,
    #[allow(dead_code)]
    shortcut_syntax: bool,
}

impl Default for MD052 {
    fn default() -> Self {
        Self::new()
    }
}

impl MD052 {
    /// Create a new MD052 rule instance
    pub fn new() -> Self {
        Self {
            ignored_labels: vec!["x".to_string()], // Default ignores checkbox syntax
            shortcut_syntax: false,
        }
    }

    /// Set the list of ignored labels
    #[allow(dead_code)]
    pub fn ignored_labels(mut self, labels: Vec<String>) -> Self {
        self.ignored_labels = labels;
        self
    }

    /// Set whether to include shortcut syntax
    #[allow(dead_code)]
    pub fn shortcut_syntax(mut self, include: bool) -> Self {
        self.shortcut_syntax = include;
        self
    }

    /// Parse reference definitions from document content
    fn collect_defined_labels(&self, document: &Document) -> HashSet<String> {
        let mut definitions = HashSet::new();
        let mut parser = RefDefParser::new(document.content.as_bytes());

        while let Some(def) = parser.next_definition() {
            definitions.insert(def.label.to_lowercase());
        }

        definitions
    }

    /// Check for undefined reference labels using byte parsing
    fn check_reference_labels(&self, document: &Document) -> Vec<Violation> {
        let mut violations = Vec::new();
        let defined_labels = self.collect_defined_labels(document);
        let mut parser = LinkParser::new(document.content.as_bytes());

        while let Some(link) = parser.next_link() {
            match link {
                LinkType::Reference {
                    label,
                    line,
                    column,
                } => {
                    let label_lower = label.to_lowercase();
                    if !self.ignored_labels.contains(&label_lower)
                        && !defined_labels.contains(&label_lower)
                    {
                        violations.push(self.create_violation(
                            format!("Reference link uses undefined label '{label}'"),
                            line,
                            column,
                            Severity::Error,
                        ));
                    }
                }
                LinkType::Image {
                    label,
                    line,
                    column,
                } => {
                    let label_lower = label.to_lowercase();
                    if !self.ignored_labels.contains(&label_lower)
                        && !defined_labels.contains(&label_lower)
                    {
                        violations.push(self.create_violation(
                            format!("Reference image uses undefined label '{label}'"),
                            line,
                            column,
                            Severity::Error,
                        ));
                    }
                }
                _ => {} // Ignore inline links
            }
        }

        violations
    }
}

impl Rule for MD052 {
    fn id(&self) -> &'static str {
        "MD052"
    }

    fn name(&self) -> &'static str {
        "reference-links-images"
    }

    fn description(&self) -> &'static str {
        "Reference links and images should use a label that is defined"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Links)
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        // This rule doesn't need AST - works entirely with byte parsing
        let violations = self.check_reference_labels(document);
        Ok(violations)
    }
}

/// Reference definition found in the document
#[derive(Debug)]
struct RefDefinition {
    label: String,
}

/// Parser for reference definitions like `[label]: url`
struct RefDefParser<'a> {
    input: &'a [u8],
    pos: usize,
    line: usize,
}

impl<'a> RefDefParser<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            pos: 0,
            line: 1,
        }
    }

    fn next_definition(&mut self) -> Option<RefDefinition> {
        while self.pos < self.input.len() {
            // Skip whitespace at beginning of line
            self.skip_whitespace();

            if self.pos >= self.input.len() {
                break;
            }

            // Check if line starts with '['
            if self.current_byte() == Some(b'[') {
                if let Some(def) = self.try_parse_definition() {
                    return Some(def);
                } else {
                    // Move forward if parsing failed
                    self.pos += 1;
                }
            } else {
                // Move to next line
                self.skip_to_next_line();
            }
        }
        None
    }

    fn try_parse_definition(&mut self) -> Option<RefDefinition> {
        let start_pos = self.pos;

        // Skip '['
        self.pos += 1;

        // Parse label
        let label = self.parse_ref_label()?;

        // Expect ']'
        if self.current_byte() != Some(b']') {
            self.pos = start_pos;
            return None;
        }
        self.pos += 1;

        // Expect ':'
        if self.current_byte() != Some(b':') {
            self.pos = start_pos;
            return None;
        }
        self.pos += 1;

        // Must have whitespace or end of line after ':'
        if let Some(ch) = self.current_byte()
            && ch != b' '
            && ch != b'\t'
            && ch != b'\n'
            && ch != b'\r'
        {
            self.pos = start_pos;
            return None;
        }

        Some(RefDefinition { label })
    }

    fn parse_ref_label(&mut self) -> Option<String> {
        let mut label = String::new();
        let mut has_content = false;

        while let Some(ch) = self.current_byte() {
            match ch {
                b']' => {
                    if has_content {
                        return Some(label);
                    } else {
                        return None; // Empty label
                    }
                }
                b'\n' | b'\r' => return None, // Newline in label
                _ => {
                    label.push(ch as char);
                    has_content = true;
                    self.pos += 1;
                }
            }
        }
        None
    }

    fn skip_to_next_line(&mut self) {
        while let Some(ch) = self.current_byte() {
            self.pos += 1;
            if ch == b'\n' {
                self.line += 1;
                break;
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            match self.input[self.pos] {
                b' ' | b'\t' => self.pos += 1,
                _ => break,
            }
        }
    }

    fn current_byte(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }
}

/// Link types found in the document
#[derive(Debug)]
enum LinkType {
    Reference {
        label: String,
        line: usize,
        column: usize,
    },
    Image {
        label: String,
        line: usize,
        column: usize,
    },
    Inline, // We don't care about inline links for this rule
}

/// Parser for links in markdown content
struct LinkParser<'a> {
    input: &'a [u8],
    pos: usize,
    line: usize,
    line_start: usize,
    in_code_block: bool,
}

impl<'a> LinkParser<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            pos: 0,
            line: 1,
            line_start: 0,
            in_code_block: false,
        }
    }

    fn next_link(&mut self) -> Option<LinkType> {
        while self.pos < self.input.len() {
            match self.current_byte()? {
                b'`' => {
                    if self.is_code_fence() {
                        self.toggle_code_block();
                    } else {
                        self.skip_code_span();
                    }
                }
                b'[' if !self.in_code_block => {
                    if let Some(link) = self.try_parse_link() {
                        return Some(link);
                    } else {
                        self.pos += 1;
                    }
                }
                b'!' if !self.in_code_block => {
                    if self.peek_byte(1) == Some(b'[') {
                        if let Some(image) = self.try_parse_image() {
                            return Some(image);
                        } else {
                            self.pos += 1;
                        }
                    } else {
                        self.pos += 1;
                    }
                }
                b'\n' => {
                    self.line += 1;
                    self.line_start = self.pos + 1;
                    self.pos += 1;
                }
                _ => self.pos += 1,
            }
        }
        None
    }

    fn try_parse_link(&mut self) -> Option<LinkType> {
        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.pos - self.line_start + 1;

        // Skip '['
        self.pos += 1;

        // Parse link text
        let _text = self.parse_link_text()?;

        // Expect ']'
        if self.current_byte() != Some(b']') {
            self.pos = start_pos + 1; // Move forward to avoid infinite loop
            return None;
        }
        self.pos += 1;

        // Check what follows
        match self.current_byte() {
            Some(b'(') => {
                // Inline link [text](url) - skip it
                self.skip_inline_url();
                Some(LinkType::Inline)
            }
            Some(b'[') => {
                // Reference link [text][label] or collapsed [text][]
                self.pos += 1;
                let label = self.parse_reference_label().unwrap_or_default();

                // If label is empty, this is a collapsed reference [text][]
                // Use the text as the label
                let final_label = if label.is_empty() { _text } else { label };

                Some(LinkType::Reference {
                    label: final_label,
                    line: start_line,
                    column: start_col,
                })
            }
            _ => {
                // Could be shortcut reference [label] but we need to check
                // if it's actually at end of word/sentence
                if self.is_likely_reference() {
                    Some(LinkType::Reference {
                        label: _text,
                        line: start_line,
                        column: start_col,
                    })
                } else {
                    None
                }
            }
        }
    }

    fn try_parse_image(&mut self) -> Option<LinkType> {
        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.pos - self.line_start + 1;

        // Skip '!['
        self.pos += 2;

        // Parse alt text
        let _alt_text = self.parse_link_text()?;

        // Expect ']'
        if self.current_byte() != Some(b']') {
            self.pos = start_pos + 1; // Move forward to avoid infinite loop
            return None;
        }
        self.pos += 1;

        // Check what follows
        match self.current_byte() {
            Some(b'(') => {
                // Inline image ![alt](url) - skip it
                self.skip_inline_url();
                Some(LinkType::Inline)
            }
            Some(b'[') => {
                // Reference image ![alt][label] or collapsed ![alt][]
                self.pos += 1;
                let label = self.parse_reference_label().unwrap_or_default();

                // If label is empty, this is a collapsed reference ![alt][]
                // Use the alt text as the label
                let final_label = if label.is_empty() { _alt_text } else { label };

                Some(LinkType::Image {
                    label: final_label,
                    line: start_line,
                    column: start_col,
                })
            }
            _ => {
                // Shortcut reference ![label]
                Some(LinkType::Image {
                    label: _alt_text,
                    line: start_line,
                    column: start_col,
                })
            }
        }
    }

    fn parse_link_text(&mut self) -> Option<String> {
        let mut text = String::new();
        let mut bracket_depth = 0;

        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            match ch {
                b'[' => {
                    bracket_depth += 1;
                    text.push(ch as char);
                    self.pos += 1;
                }
                b']' => {
                    if bracket_depth > 0 {
                        bracket_depth -= 1;
                        text.push(ch as char);
                        self.pos += 1;
                    } else {
                        return Some(text);
                    }
                }
                b'\\' => {
                    // Handle escaped characters
                    self.pos += 1;
                    if self.pos < self.input.len() {
                        let escaped = self.input[self.pos];
                        text.push('\\');
                        text.push(escaped as char);
                        self.pos += 1;
                    }
                }
                b'\n' => return None, // Newline breaks link
                _ => {
                    text.push(ch as char);
                    self.pos += 1;
                }
            }
        }
        None
    }

    fn parse_reference_label(&mut self) -> Option<String> {
        let mut label = String::new();

        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            match ch {
                b']' => {
                    self.pos += 1;
                    return Some(label); // Return even if empty for collapsed refs
                }
                b'\n' => return None, // Newline breaks reference
                _ => {
                    label.push(ch as char);
                    self.pos += 1;
                }
            }
        }
        None
    }

    fn skip_inline_url(&mut self) {
        // Skip '('
        if self.pos < self.input.len() && self.input[self.pos] == b'(' {
            self.pos += 1;
        }

        let mut paren_depth = 1;
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            match ch {
                b'(' => {
                    paren_depth += 1;
                    self.pos += 1;
                }
                b')' => {
                    paren_depth -= 1;
                    self.pos += 1;
                    if paren_depth == 0 {
                        break;
                    }
                }
                b'\\' => {
                    // Skip escaped character
                    self.pos += 1;
                    if self.pos < self.input.len() {
                        self.pos += 1;
                    }
                }
                _ => self.pos += 1,
            }
        }
    }

    fn skip_code_span(&mut self) {
        let start = self.pos;
        self.pos += 1;

        // Count opening backticks
        let mut backticks = 1;
        while self.pos < self.input.len() && self.input[self.pos] == b'`' {
            backticks += 1;
            self.pos += 1;
        }

        // Find matching closing backticks
        let mut found = 0;
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            if ch == b'`' {
                found += 1;
                self.pos += 1;
                if found == backticks {
                    return;
                }
            } else {
                found = 0;
                self.pos += 1;
                if ch == b'\n' {
                    self.line += 1;
                    self.line_start = self.pos;
                }
            }
        }

        // If we didn't find closing backticks, reset
        self.pos = start + 1;
    }

    fn is_code_fence(&mut self) -> bool {
        let _start = self.pos;

        // Check if we're at start of line (possibly with whitespace)
        let mut line_pos = self.line_start;
        while line_pos < self.pos {
            match self.input.get(line_pos) {
                Some(b' ') | Some(b'\t') => line_pos += 1,
                _ => return false, // Non-whitespace before backticks
            }
        }

        // Count consecutive backticks
        let mut count = 0;
        let mut pos = self.pos;
        while pos < self.input.len() && self.input[pos] == b'`' {
            count += 1;
            pos += 1;
        }

        count >= 3
    }

    fn toggle_code_block(&mut self) {
        self.in_code_block = !self.in_code_block;
        // Skip the entire code fence line
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            self.pos += 1;
            if ch == b'\n' {
                self.line += 1;
                self.line_start = self.pos;
                break;
            }
        }
    }

    fn is_likely_reference(&self) -> bool {
        // Simple heuristic: if followed by whitespace, punctuation, or end of line
        if self.pos >= self.input.len() {
            return true; // End of file
        }

        matches!(
            self.input[self.pos],
            b' ' | b'\t' | b'\n' | b'\r' | b'.' | b',' | b';' | b':' | b'!' | b'?'
        )
    }

    fn current_byte(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn peek_byte(&self, offset: usize) -> Option<u8> {
        self.input.get(self.pos + offset).copied()
    }
}

#[cfg(test)]
// TODO: Tests temporarily disabled during migration (Part 2 of #66)
// Will be re-enabled when test_helpers is made public in Part 3
// mod tests {
    use super::*;
    // TODO: Re-enable when test_helpers is available
    // use mdbook_lint_core::test_helpers::{
    //    assert_no_violations, assert_single_violation, assert_violation_count,
    // };

    #[test]
    fn test_valid_references() {
        let content = r#"[Full reference][label]
[Collapsed reference][]

[label]: https://example.com
[collapsed reference]: https://example.com
"#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_undefined_reference() {
        let content = r#"[Link text][undefined-label]

[defined]: https://example.com
"#;

        let violation = assert_single_violation(MD052::new(), content);
        assert_eq!(violation.line, 1);
        assert!(violation.message.contains("undefined-label"));
    }

    #[test]
    fn test_ignored_labels() {
        let content = r#"[Checkbox][x]
"#;

        assert_no_violations(MD052::new(), content); // 'x' is ignored by default
    }

    #[test]
    fn test_case_insensitive_matching() {
        let content = r#"[Link][LABEL]

[label]: https://example.com
"#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_collapsed_reference() {
        let content = r#"[Label][]

[label]: https://example.com
"#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_multiple_undefined_references() {
        let content = r#"[Link 1][undefined1]
[Link 2][undefined2]

[defined]: https://example.com
"#;

        let violations = assert_violation_count(MD052::new(), content, 2);
        assert!(violations[0].message.contains("undefined1"));
        assert!(violations[1].message.contains("undefined2"));
    }

    #[test]
    fn test_reference_images() {
        let content = r#"![Alt text][undefined-image]

[defined]: https://example.com
"#;

        let violation = assert_single_violation(MD052::new(), content);
        assert_eq!(violation.line, 1);
        assert!(violation.message.contains("undefined-image"));
    }

    #[test]
    fn test_inline_links_ignored() {
        let content = r#"[Inline link](https://example.com)
![Inline image](image.png)
"#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_code_spans_ignored() {
        let content = r#"`[not a link][label]`

[label]: https://example.com
"#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_code_blocks_ignored() {
        let content = r#"```
[not a link][undefined]
```

[defined]: https://example.com
"#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_nested_brackets() {
        let content = r#"[Link with [nested] text][label]

[label]: https://example.com
"#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_escaped_brackets() {
        let content = r#"\[Not a link\][label]

[label]: https://example.com
"#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_shortcut_references() {
        let content = r#"[label] is a shortcut reference.

[label]: https://example.com
"#;

        assert_no_violations(MD052::new(), content);
    }
// }
