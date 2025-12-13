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

use comrak::nodes::AstNode;
use mdbook_lint_core::error::Result;
use mdbook_lint_core::{
    Document, Violation,
    rule::{Rule, RuleCategory, RuleMetadata},
    violation::Severity,
};
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
            // Default ignores:
            // - " " (space) for unchecked task list checkboxes (- [ ])
            // - "x" for checked task list checkboxes (- [x])
            // - Extended checkbox states (Obsidian, GitHub Projects, etc.)
            // - GitHub-style admonition labels (supported by mdBook 0.5+)
            ignored_labels: vec![
                // Standard task list checkboxes
                " ".to_string(),
                "x".to_string(),
                // Extended checkbox states
                "-".to_string(),  // Cancelled/removed
                ">".to_string(),  // Forwarded/deferred
                "<".to_string(),  // Scheduled
                "?".to_string(),  // Question
                "!".to_string(),  // Important
                "*".to_string(),  // Star/highlight
                "\"".to_string(), // Quote
                "l".to_string(),  // Location
                "b".to_string(),  // Bookmark
                "i".to_string(),  // Information/Idea
                "s".to_string(),  // Savings/Amount
                "p".to_string(),  // Pro
                "c".to_string(),  // Con
                "f".to_string(),  // Fire
                "k".to_string(),  // Key
                "w".to_string(),  // Win
                "u".to_string(),  // Up
                "d".to_string(),  // Down/deleted
                "/".to_string(),  // Half-done/in progress
                // GitHub-style admonitions
                "!note".to_string(),
                "!tip".to_string(),
                "!important".to_string(),
                "!warning".to_string(),
                "!caution".to_string(),
            ],
            shortcut_syntax: false,
        }
    }

    /// Set the list of ignored labels
    #[allow(dead_code)]
    pub fn ignored_labels(mut self, labels: Vec<String>) -> Self {
        self.ignored_labels = labels;
        self
    }

    /// Create MD052 from configuration
    pub fn from_config(config: &toml::Value) -> Self {
        let mut rule = Self::new();

        if let Some(ignored_labels) = config.get("ignored_labels")
            && let Some(labels_array) = ignored_labels.as_array()
        {
            rule.ignored_labels = labels_array
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();
        }

        if let Some(shortcut_syntax) = config.get("shortcut_syntax").and_then(|v| v.as_bool()) {
            rule.shortcut_syntax = shortcut_syntax;
        }

        rule
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
    in_inline_math: bool,
    in_display_math: bool,
    on_heading_line: bool,
}

impl<'a> LinkParser<'a> {
    fn new(input: &'a [u8]) -> Self {
        // Check if first line is a heading
        let first_line_is_heading = {
            let mut pos = 0;
            while pos < input.len() && (input[pos] == b' ' || input[pos] == b'\t') {
                pos += 1;
            }
            pos < input.len() && input[pos] == b'#'
        };

        Self {
            input,
            pos: 0,
            line: 1,
            line_start: 0,
            in_code_block: false,
            in_inline_math: false,
            in_display_math: false,
            on_heading_line: first_line_is_heading,
        }
    }

    /// Check if the current line is a heading (starts with #)
    fn check_heading_line(&self) -> bool {
        // Look at the start of the current line
        let mut pos = self.line_start;
        // Skip leading whitespace
        while pos < self.input.len() && (self.input[pos] == b' ' || self.input[pos] == b'\t') {
            pos += 1;
        }
        // Check if line starts with #
        pos < self.input.len() && self.input[pos] == b'#'
    }

    /// Check if a [text][label] pattern looks like array notation rather than a reference link
    /// Array notation typically has short identifiers like [i], [j], [row], [col], [0], [1], etc.
    fn looks_like_array_notation(&self, text: &str, label: &str) -> bool {
        // If label is empty (collapsed reference), it's not array notation
        if label.is_empty() {
            return false;
        }

        // Check if both parts look like array indices/identifiers
        // Short alphanumeric strings (1-10 chars) that look like variable names or indices
        let is_index_like = |s: &str| {
            if s.is_empty() || s.len() > 10 {
                return false;
            }
            // Must be alphanumeric (possibly with underscores)
            s.chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        };

        is_index_like(text) && is_index_like(label)
    }

    fn next_link(&mut self) -> Option<LinkType> {
        while self.pos < self.input.len() {
            match self.current_byte()? {
                b'`' => {
                    if self.is_code_fence() {
                        self.toggle_code_block();
                    } else if !self.in_code_block {
                        self.skip_code_span();
                    } else {
                        self.pos += 1;
                    }
                }
                b'$' if !self.in_code_block => {
                    // Handle math blocks - skip content inside $...$ or $$...$$
                    self.handle_math_delimiter();
                }
                b'[' if !self.in_code_block && !self.in_inline_math && !self.in_display_math => {
                    // Check for wiki-link syntax [[text]] - skip if next char is also [
                    if self.peek_byte(1) == Some(b'[') {
                        self.skip_wiki_link();
                    } else if let Some(link) = self.try_parse_link() {
                        return Some(link);
                    } else {
                        self.pos += 1;
                    }
                }
                b'!' if !self.in_code_block && !self.in_inline_math && !self.in_display_math => {
                    // Check for embedded wiki-link syntax ![[text]]
                    if self.peek_byte(1) == Some(b'[') && self.peek_byte(2) == Some(b'[') {
                        self.pos += 1; // Skip '!'
                        self.skip_wiki_link();
                    } else if self.peek_byte(1) == Some(b'[') {
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
                    self.pos += 1;
                    self.line_start = self.pos;
                    // Inline math doesn't span lines
                    self.in_inline_math = false;
                    // Update heading line status for the new line
                    self.on_heading_line = self.check_heading_line();
                }
                _ => self.pos += 1,
            }
        }
        None
    }

    /// Handle $ delimiter for math blocks
    fn handle_math_delimiter(&mut self) {
        // Check if this $ is escaped
        if self.is_escaped() {
            self.pos += 1;
            return;
        }

        // Check for display math ($$)
        if self.peek_byte(1) == Some(b'$') {
            self.in_display_math = !self.in_display_math;
            self.pos += 2;
        } else {
            // Inline math ($)
            self.in_inline_math = !self.in_inline_math;
            self.pos += 1;
        }
    }

    /// Check if the current position is escaped by a backslash
    fn is_escaped(&self) -> bool {
        if self.pos == 0 {
            return false;
        }

        // Count preceding backslashes
        let mut backslash_count = 0;
        let mut check_pos = self.pos - 1;
        while self.input.get(check_pos) == Some(&b'\\') {
            backslash_count += 1;
            if check_pos == 0 {
                break;
            }
            check_pos -= 1;
        }

        // Odd number of backslashes means the $ is escaped
        backslash_count % 2 == 1
    }

    /// Check if the current position is preceded by a specific character
    fn preceded_by(&self, ch: u8) -> bool {
        if self.pos == 0 {
            return false;
        }
        self.input.get(self.pos - 1) == Some(&ch)
    }

    fn try_parse_link(&mut self) -> Option<LinkType> {
        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.pos - self.line_start + 1;

        // Check for inline footnote syntax: ^[content]
        // These should not be treated as reference links
        if self.preceded_by(b'^') {
            self.pos += 1;
            // Skip past the footnote content
            if self.parse_link_text().is_some() && self.current_byte() == Some(b']') {
                self.pos += 1;
            }
            return None;
        }

        // Check for abbreviation definition syntax: *[ABBR]: definition
        // These should not be treated as reference links
        if self.preceded_by(b'*') {
            // Check if this looks like an abbreviation definition (at start of line)
            let at_line_start = {
                let mut pos = self.line_start;
                // Skip leading whitespace
                while pos < self.pos - 1 && (self.input[pos] == b' ' || self.input[pos] == b'\t') {
                    pos += 1;
                }
                // The * should be at the start (after optional whitespace)
                pos == self.pos - 1
            };

            if at_line_start {
                self.pos += 1;
                // Skip past the abbreviation content
                if self.parse_link_text().is_some() && self.current_byte() == Some(b']') {
                    self.pos += 1;
                    // Check for the colon that confirms it's an abbreviation definition
                    if self.current_byte() == Some(b':') {
                        return None;
                    }
                }
                // Reset if not a valid abbreviation definition
                self.pos = start_pos + 1;
                return None;
            }
        }

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
                let final_label = if label.is_empty() {
                    _text.clone()
                } else {
                    label.clone()
                };

                // On heading lines, skip patterns that look like array notation
                // e.g., Matrix[row][col] where both parts are short identifiers
                if self.on_heading_line && self.looks_like_array_notation(&_text, &label) {
                    None
                } else {
                    Some(LinkType::Reference {
                        label: final_label,
                        line: start_line,
                        column: start_col,
                    })
                }
            }
            _ => {
                // Could be shortcut reference [label] but we need to check
                // if it's actually at end of word/sentence
                // Skip shortcut references on heading lines to avoid false positives
                // like "### Array[i] Notation" where [i] looks like a reference
                if self.on_heading_line {
                    None
                } else if self.is_likely_reference() {
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

    /// Skip wiki-link syntax: [[text]] or [[text|display]]
    /// Also handles embedded syntax: ![[text]]
    fn skip_wiki_link(&mut self) {
        // Skip '[['
        self.pos += 2;

        // Skip past the wiki-link content (may contain | for display text, # for headings)
        while self.pos < self.input.len() {
            match self.input[self.pos] {
                b']' => {
                    self.pos += 1;
                    // Check for closing ]
                    if self.current_byte() == Some(b']') {
                        self.pos += 1;
                    }
                    break;
                }
                b'\n' => break, // Newline breaks wiki-link
                _ => self.pos += 1,
            }
        }
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
mod tests {
    use super::*;
    use mdbook_lint_core::test_helpers::*;

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
    fn test_admonitions_ignored() {
        // GitHub-style admonitions (supported by mdBook 0.5+) should not trigger violations
        let content = r#"> [!NOTE]
> This is a note

> [!TIP]
> This is a tip

> [!IMPORTANT]
> This is important

> [!WARNING]
> This is a warning

> [!CAUTION]
> This is a caution
"#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_admonitions_case_insensitive() {
        let content = r#"> [!Note]
> Mixed case should work too
"#;

        assert_no_violations(MD052::new(), content);
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

    #[test]
    fn test_katex_inline_math_not_flagged() {
        // Issue #321: $A[i][j]$ should not be flagged as reference links
        let content = r#"The matrix element $A[i][j]$ is accessed like this."#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_katex_display_math_not_flagged() {
        let content = r#"Display math:
$$
A[i][j] = element
$$
"#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_katex_complex_expressions() {
        let content = r#"Sum notation: $\sum_{i=0}^{n} a[i]$

Product: $\prod_{j=1}^{m} b[j]$

Matrix: $M[row][col]$
"#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_escaped_dollar_signs_ignored() {
        let content = r#"Price is \$50 not a math block, so [undefined] would be flagged.

[defined]: https://example.com
"#;

        // The \$ is escaped, so [undefined] is not in a math block and should be flagged
        let violation = assert_single_violation(MD052::new(), content);
        assert!(violation.message.contains("undefined"));
    }

    #[test]
    fn test_math_with_actual_references_still_work() {
        let content = r#"Math: $x = y$ and [link][ref]

[ref]: https://example.com
"#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_math_followed_by_undefined_reference() {
        let content = r#"Math $x[i]$ and then [undefined][label]
"#;

        let violation = assert_single_violation(MD052::new(), content);
        assert!(violation.message.contains("label"));
    }

    #[test]
    fn test_multiple_math_blocks_on_line() {
        let content = r#"We have $a[i]$ and $b[j]$ in the formula."#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_brackets_in_headings_not_flagged() {
        // Brackets in headings should not be flagged as undefined references
        let content = r#"# Array[i] Notation

## The Matrix[row][col] Access Pattern

### What's New in v2.0

Some regular text here.
"#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_heading_with_actual_reference_link() {
        // Actual reference links in headings should still work
        let content = r#"# See the [documentation][docs]

[docs]: https://example.com
"#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_heading_with_undefined_reference_link() {
        // Actual undefined reference links in headings should still be flagged
        let content = r#"# See the [documentation][undefined-docs]

[docs]: https://example.com
"#;

        let violation = assert_single_violation(MD052::new(), content);
        assert!(violation.message.contains("undefined-docs"));
    }

    #[test]
    fn test_shortcut_reference_after_heading() {
        // Shortcut references in regular text after heading should still be detected
        let content = r#"# Heading

This references [undefined] which should be flagged.
"#;

        let violation = assert_single_violation(MD052::new(), content);
        assert!(violation.message.contains("undefined"));
    }

    #[test]
    fn test_brackets_in_first_line_heading() {
        // First line heading with brackets should not be flagged
        let content = r#"# Array[0] Introduction"#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_task_list_checkboxes_not_flagged() {
        // Task list checkboxes should not be flagged as undefined references
        let content = r#"# Tasks

- [ ] Unchecked task
- [x] Checked task
- [X] Also checked task
* [ ] Unchecked with asterisk
+ [x] Checked with plus
"#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_abbreviation_definitions_not_flagged() {
        // Abbreviation definitions should not be flagged as undefined references
        let content = r#"# Abbreviations

The HTML specification is maintained by W3C.

*[HTML]: Hypertext Markup Language
*[W3C]: World Wide Web Consortium
"#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_inline_footnotes_not_flagged() {
        // Inline footnotes should not be flagged as undefined references
        let content = r#"# Footnotes

This is some text^[with an inline footnote].

Another sentence^[another footnote here] continues.
"#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_mixed_extended_syntax() {
        // Mix of task lists, abbreviations, and footnotes
        let content = r#"# Extended Markdown

- [ ] Learn HTML
- [x] Read about W3C

The HTML spec^[see w3.org] is great.

*[HTML]: Hypertext Markup Language
"#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_wiki_links_not_flagged() {
        // Wiki-link syntax (Obsidian, Foam, etc.) should not be flagged
        let content = r#"# Wiki Links

[[Internal Page]]

[[Page|Display Text]]

[[Page#Heading]]

[[Page#Heading|Custom Text]]

![[Embedded Note]]

![[image.png]]

Press [[Ctrl]] + [[C]] to copy.
"#;

        assert_no_violations(MD052::new(), content);
    }

    #[test]
    fn test_extended_checkbox_states() {
        // Extended checkbox states (Obsidian, GitHub Projects, etc.)
        let content = r#"# Extended Checkboxes

- [-] Cancelled task
- [>] Forwarded task
- [<] Scheduled task
- [?] Question
- [!] Important
- [*] Star
- ["] Quote
- [l] Location
- [b] Bookmark
- [i] Information
- [S] Savings
- [I] Idea
- [p] Pro
- [c] Con
- [/] In progress
"#;

        assert_no_violations(MD052::new(), content);
    }
}
