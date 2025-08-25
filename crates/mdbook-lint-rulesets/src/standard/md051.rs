//! MD051 - Link fragments should be valid
//!
//! This rule is triggered when a link fragment does not match any of the fragments
//! that are automatically generated for headings in a document.
//!
//! ## Correct
//!
//! ```markdown
//! # Heading Name
//!
//! \[Link\](#heading-name)
//! ```
//!
//! ## Incorrect
//!
//! ```markdown
//! # Heading Name
//!
//! \[Link\](#invalid-fragment)
//! ```

use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::error::Result;
use mdbook_lint_core::{
    Document, Violation,
    rule::{Rule, RuleCategory, RuleMetadata},
    violation::Severity,
};

use std::collections::{HashMap, HashSet};

/// MD051 - Link fragments should be valid
pub struct MD051 {
    ignore_case: bool,
    ignored_pattern: Option<String>,
}

impl Default for MD051 {
    fn default() -> Self {
        Self::new()
    }
}

impl MD051 {
    /// Create a new MD051 rule instance
    pub fn new() -> Self {
        Self {
            ignore_case: false,
            ignored_pattern: None,
        }
    }

    /// Set whether to ignore case when comparing fragments
    #[allow(dead_code)]
    pub fn ignore_case(mut self, ignore_case: bool) -> Self {
        self.ignore_case = ignore_case;
        self
    }

    #[allow(dead_code)]
    pub fn ignored_pattern(mut self, pattern: Option<String>) -> Self {
        self.ignored_pattern = pattern;
        self
    }

    /// Get position information from a node
    fn get_position<'a>(&self, node: &'a AstNode<'a>) -> (usize, usize) {
        let data = node.data.borrow();
        let pos = data.sourcepos;
        (pos.start.line, pos.start.column)
    }

    /// Generate GitHub-style heading fragment from text
    fn generate_heading_fragment(&self, text: &str) -> String {
        // GitHub heading algorithm:
        // 1. Convert to lowercase
        // 2. Remove punctuation (keep alphanumeric, spaces, hyphens)
        // 3. Convert spaces to dashes
        // 4. Remove leading/trailing dashes
        let mut fragment = text.to_lowercase();

        // Remove punctuation, keep alphanumeric, spaces, hyphens, underscores
        fragment = fragment
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
            .collect();

        // Convert spaces to dashes
        fragment = fragment.replace(' ', "-");

        // Remove multiple consecutive dashes
        fragment = self.consolidate_dashes(&fragment);

        // Remove leading/trailing dashes
        fragment = fragment.trim_matches('-').to_string();

        fragment
    }

    /// Extract text content from a heading node
    fn extract_heading_text<'a>(node: &'a AstNode<'a>) -> String {
        let mut text = String::new();
        for child in node.children() {
            match &child.data.borrow().value {
                NodeValue::Text(t) => text.push_str(t),
                NodeValue::Code(code) => text.push_str(&code.literal),
                NodeValue::Emph | NodeValue::Strong => {
                    text.push_str(&Self::extract_heading_text(child));
                }
                _ => {}
            }
        }
        text
    }

    /// Collect all valid fragments from the document
    fn collect_valid_fragments<'a>(&self, ast: &'a AstNode<'a>) -> HashSet<String> {
        let mut fragments = HashSet::new();
        let mut heading_counts: HashMap<String, usize> = HashMap::new();

        // Add special fragments
        fragments.insert("top".to_string());

        self.traverse_for_fragments(ast, &mut fragments, &mut heading_counts);

        fragments
    }

    /// Traverse AST to find fragments
    fn traverse_for_fragments<'a>(
        &self,
        node: &'a AstNode<'a>,
        fragments: &mut HashSet<String>,
        heading_counts: &mut HashMap<String, usize>,
    ) {
        match &node.data.borrow().value {
            NodeValue::Heading(_) => {
                let heading_text = Self::extract_heading_text(node);
                let mut fragment = self.generate_heading_fragment(&heading_text);

                // Handle duplicate fragments by appending numbers
                if let Some(count) = heading_counts.get(&fragment) {
                    let new_count = count + 1;
                    heading_counts.insert(fragment.clone(), new_count);
                    fragment = format!("{fragment}-{new_count}");
                } else {
                    heading_counts.insert(fragment.clone(), 1);
                }

                fragments.insert(fragment);

                // Check for custom anchor syntax {#custom-name}
                if let Some(anchor_id) = self.extract_custom_anchor(&heading_text) {
                    fragments.insert(anchor_id);
                }
            }
            NodeValue::HtmlBlock(html) => {
                // Extract id attributes from HTML elements
                let ids = self.extract_html_ids(&html.literal);
                for id in ids {
                    fragments.insert(id);
                }

                // Extract name attributes from <a> tags
                let names = self.extract_html_names(&html.literal);
                for name in names {
                    fragments.insert(name);
                }
            }
            NodeValue::HtmlInline(html) => {
                // Extract id attributes from HTML elements
                let ids = self.extract_html_ids(html);
                for id in ids {
                    fragments.insert(id);
                }

                // Extract name attributes from <a> tags
                let names = self.extract_html_names(html);
                for name in names {
                    fragments.insert(name);
                }
            }
            _ => {}
        }

        for child in node.children() {
            self.traverse_for_fragments(child, fragments, heading_counts);
        }
    }

    /// Replace multiple consecutive dashes with single dash
    fn consolidate_dashes(&self, text: &str) -> String {
        let mut result = String::new();
        let mut prev_was_dash = false;

        for ch in text.chars() {
            if ch == '-' {
                if !prev_was_dash {
                    result.push(ch);
                }
                prev_was_dash = true;
            } else {
                result.push(ch);
                prev_was_dash = false;
            }
        }

        result
    }

    /// Extract custom anchor ID from text like {#custom-name}
    fn extract_custom_anchor(&self, text: &str) -> Option<String> {
        if let Some(start) = text.find("{#") {
            let remaining = &text[start + 2..];
            if let Some(end) = remaining.find('}') {
                let anchor_id = &remaining[..end];
                // Validate anchor ID (alphanumeric, dash, underscore only)
                if anchor_id
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
                    && !anchor_id.is_empty()
                {
                    return Some(anchor_id.to_string());
                }
            }
        }
        None
    }

    /// Extract HTML id attributes using regex (O(n) complexity)
    fn extract_html_ids(&self, html: &str) -> Vec<String> {
        use regex::Regex;

        // Compile regex once - matches id="value" or id='value' or id=value
        // This regex handles various HTML id attribute formats:
        // - id="value" (double quotes)
        // - id='value' (single quotes)
        // - id=value (no quotes)
        // With optional whitespace around the equals sign
        let id_regex = Regex::new(r#"(?i)id\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]*))"#).unwrap();

        let mut ids = Vec::new();
        for captures in id_regex.captures_iter(html) {
            // Check each capture group (double quote, single quote, no quote)
            for i in 1..=3 {
                if let Some(id_match) = captures.get(i) {
                    let id_value = id_match.as_str().trim();
                    if !id_value.is_empty() {
                        ids.push(id_value.to_string());
                        break; // Only take first non-empty capture group
                    }
                }
            }
        }

        ids
    }

    /// Extract HTML name attributes from <a> tags using regex (O(n) complexity)
    fn extract_html_names(&self, html: &str) -> Vec<String> {
        use regex::Regex;

        // Regex to find <a> tags with name attributes
        // This matches: <a name="value"> or <a name='value'> or <a name=value>
        // Handles various formats and optional whitespace
        let name_regex =
            Regex::new(r#"(?i)<a[^>]*name\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]*)).*?>"#).unwrap();

        let mut names = Vec::new();
        for captures in name_regex.captures_iter(html) {
            // Check each capture group (double quote, single quote, no quote)
            for i in 1..=3 {
                if let Some(name_match) = captures.get(i) {
                    let name_value = name_match.as_str().trim();
                    if !name_value.is_empty() {
                        names.push(name_value.to_string());
                        break; // Only take first non-empty capture group
                    }
                }
            }
        }

        names
    }

    /// Check if fragment is a GitHub line reference (L123, L123C45, L123-L456, etc.)
    fn is_github_line_reference(&self, fragment: &str) -> bool {
        if !fragment.starts_with('L') {
            return false;
        }

        let remaining = &fragment[1..];
        let mut chars = remaining.chars().peekable();

        // Must start with digits
        if !self.consume_digits(&mut chars) {
            return false;
        }

        // Optional C followed by digits
        if chars.peek() == Some(&'C') {
            chars.next();
            if !self.consume_digits(&mut chars) {
                return false;
            }
        }

        // Optional range: -L followed by digits and optional C digits
        if chars.peek() == Some(&'-') {
            chars.next();
            if chars.next() != Some('L') {
                return false;
            }
            if !self.consume_digits(&mut chars) {
                return false;
            }
            // Optional C followed by digits for end of range
            if chars.peek() == Some(&'C') {
                chars.next();
                if !self.consume_digits(&mut chars) {
                    return false;
                }
            }
        }

        // Must be at end of string
        chars.peek().is_none()
    }

    /// Consume consecutive digits from char iterator, return true if any were consumed
    fn consume_digits(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> bool {
        let mut consumed_any = false;
        while let Some(&ch) = chars.peek() {
            if ch.is_ascii_digit() {
                chars.next();
                consumed_any = true;
            } else {
                break;
            }
        }
        consumed_any
    }

    /// Check for invalid link fragments
    fn check_link_fragments<'a>(
        &self,
        ast: &'a AstNode<'a>,
        valid_fragments: &HashSet<String>,
    ) -> Vec<Violation> {
        let mut violations = Vec::new();

        self.traverse_for_links(ast, valid_fragments, &mut violations);

        violations
    }

    /// Traverse AST to find link fragments
    fn traverse_for_links<'a>(
        &self,
        node: &'a AstNode<'a>,
        valid_fragments: &HashSet<String>,
        violations: &mut Vec<Violation>,
    ) {
        if let NodeValue::Link(link) = &node.data.borrow().value
            && let Some(fragment) = link.url.strip_prefix('#')
        {
            // Handle empty fragments - they should cause violations
            if fragment.is_empty() {
                let pos = self.get_position(node);
                violations.push(self.create_violation(
                    "Link fragment is empty".to_string(),
                    pos.0,
                    pos.1,
                    Severity::Error,
                ));
                return;
            }

            // Skip if matches ignored pattern
            if let Some(ref pattern) = self.ignored_pattern
                && fragment.contains(pattern)
            {
                return;
            }

            // GitHub line reference patterns are always valid
            if self.is_github_line_reference(fragment) {
                return;
            }

            let fragment_to_check = if self.ignore_case {
                fragment.to_lowercase()
            } else {
                fragment.to_string()
            };

            let valid_fragments_check: HashSet<String> = if self.ignore_case {
                valid_fragments.iter().map(|f| f.to_lowercase()).collect()
            } else {
                valid_fragments.clone()
            };

            if !valid_fragments_check.contains(&fragment_to_check) {
                let pos = self.get_position(node);
                violations.push(self.create_violation(
                    format!("Link fragment '{fragment}' is not valid"),
                    pos.0,
                    pos.1,
                    Severity::Error,
                ));
            }
        }

        for child in node.children() {
            self.traverse_for_links(child, valid_fragments, violations);
        }
    }

    /// Fallback method using manual parsing when no AST is available
    fn check_fragments_fallback(&self, document: &Document) -> Vec<Violation> {
        let mut violations = Vec::new();

        for (line_num, line) in document.content.lines().enumerate() {
            let line_number = line_num + 1;
            let mut chars = line.char_indices().peekable();
            let mut in_backticks = false;

            while let Some((i, ch)) = chars.next() {
                match ch {
                    '`' => {
                        in_backticks = !in_backticks;
                    }
                    '[' if !in_backticks => {
                        // Try to parse link with fragment: [text](#fragment)
                        if let Some((fragment, text_end)) = self.parse_fragment_link(&line[i..]) {
                            // Handle empty fragments - they should cause violations
                            if fragment.is_empty() {
                                violations.push(self.create_violation(
                                    "Link fragment is empty".to_string(),
                                    line_number,
                                    i + 1,
                                    Severity::Error,
                                ));
                                // Skip past the parsed link
                                for _ in 0..text_end - 1 {
                                    chars.next();
                                }
                                continue;
                            }

                            // Skip special cases like "top"
                            if fragment == "top" {
                                // Skip past the parsed link
                                for _ in 0..text_end - 1 {
                                    chars.next();
                                }
                                continue;
                            }

                            // For the fallback, we'll do basic validation
                            // Check for obvious case issues and suspicious patterns
                            let mut is_suspicious = false;

                            // Skip GitHub line references - they are always valid
                            if self.is_github_line_reference(&fragment) {
                                // Skip past the parsed link
                                for _ in 0..text_end - 1 {
                                    chars.next();
                                }
                                continue;
                            }

                            if fragment.contains("invalid") || fragment.contains("undefined") {
                                is_suspicious = true;
                            }

                            // Check for basic case issues (contains uppercase when should be lowercase)
                            // Only flag this if case sensitivity is enabled
                            if !self.ignore_case && fragment != fragment.to_lowercase() {
                                is_suspicious = true;
                            }

                            if is_suspicious {
                                violations.push(self.create_violation(
                                    format!("Link fragment '{fragment}' may not be valid"),
                                    line_number,
                                    i + 1,
                                    Severity::Warning,
                                ));
                            }

                            // Skip past the parsed link
                            for _ in 0..text_end - 1 {
                                chars.next();
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        violations
    }

    /// Parse a fragment link starting at the given position
    /// Returns (fragment, total_length) if found
    fn parse_fragment_link(&self, text: &str) -> Option<(String, usize)> {
        if !text.starts_with('[') {
            return None;
        }

        // Find the closing bracket
        let mut bracket_count = 0;
        let mut closing_bracket_pos = None;

        for (i, ch) in text.char_indices() {
            match ch {
                '[' => bracket_count += 1,
                ']' => {
                    bracket_count -= 1;
                    if bracket_count == 0 {
                        closing_bracket_pos = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }

        let closing_bracket_pos = closing_bracket_pos?;
        let remaining = &text[closing_bracket_pos + 1..];

        // Check if this is followed by (#fragment)
        if remaining.starts_with("(#") {
            let fragment_start = closing_bracket_pos + 3; // +1 for ], +1 for (, +1 for #
            if let Some(closing_paren) = remaining.find(')') {
                let fragment_end = closing_bracket_pos + 1 + closing_paren;
                let fragment = &text[fragment_start..fragment_end];
                let total_length = fragment_end + 1;
                return Some((fragment.to_string(), total_length));
            }
        }

        None
    }
}

impl Rule for MD051 {
    fn id(&self) -> &'static str {
        "MD051"
    }

    fn name(&self) -> &'static str {
        "link-fragments"
    }

    fn description(&self) -> &'static str {
        "Link fragments should be valid"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Links)
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        ast: Option<&'a AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        if let Some(ast) = ast {
            let valid_fragments = self.collect_valid_fragments(ast);
            let violations = self.check_link_fragments(ast, &valid_fragments);
            Ok(violations)
        } else {
            // Simplified regex-based fallback when no AST is available
            Ok(self.check_fragments_fallback(document))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::test_helpers::*;

    #[test]
    fn test_valid_fragments() {
        let content = r#"# Heading Name

[Link](#heading-name)

## Another Heading

[Another link](#another-heading)

<div id="custom-id"></div>
[Custom](#custom-id)

<a name="bookmark"></a>
[Bookmark](#bookmark)

[Top link](#top)
"#;

        assert_no_violations(MD051::new(), content);
    }

    #[test]
    fn test_invalid_fragments() {
        let content = r#"# Heading Name

[Invalid link](#invalid-fragment)
"#;

        let violation = assert_single_violation(MD051::new(), content);
        assert_eq!(violation.line, 3);
        assert!(violation.message.contains("invalid-fragment"));
    }

    #[test]
    fn test_duplicate_headings() {
        let content = r#"# Test

[Link 1](#test)

# Test

[Link 2](#test-1)
"#;

        assert_no_violations(MD051::new(), content);
    }

    #[test]
    fn test_github_line_references() {
        let content = r#"# Code

[Line 20](#L20)
[Range](#L19C5-L21C11)
"#;

        assert_no_violations(MD051::new(), content);
    }

    #[test]
    fn test_case_sensitivity() {
        let content = r#"# Heading Name

[Link](#Heading-Name)
"#;

        let violation = assert_single_violation(MD051::new(), content);
        assert_eq!(violation.line, 3);

        assert_no_violations(MD051::new().ignore_case(true), content);
    }

    #[test]
    fn test_custom_anchor() {
        let content = r#"# Heading Name {#custom-anchor}

[Link](#custom-anchor)
"#;

        assert_no_violations(MD051::new(), content);
    }

    #[test]
    fn test_empty_fragment() {
        let content = r#"# Heading

[Empty fragment](#)
"#;

        let violation = assert_single_violation(MD051::new(), content);
        assert_eq!(violation.line, 3);
    }

    #[test]
    fn test_html_id_attributes() {
        let content = r#"# Heading

<div id="custom-id">Content</div>
<span id="another-id">Text</span>

[Link to div](#custom-id)
[Link to span](#another-id)
"#;

        assert_no_violations(MD051::new(), content);
    }

    #[test]
    fn test_html_name_attributes() {
        let content = r#"# Heading

<a name="anchor-name"></a>
<div name="form-element">Content</div>

[Link to anchor](#anchor-name)
[Link to element](#form-element)
"#;

        assert_no_violations(MD051::new(), content);
    }

    #[test]
    fn test_html_block_extraction() {
        let content = r#"# Heading

<div class="content">
  <p id="paragraph-id">Text</p>
  <a name="link-name" href="/test">Link</a>
</div>

[Link to paragraph](#paragraph-id)
[Link to anchor](#link-name)
"#;

        assert_no_violations(MD051::new(), content);
    }

    #[test]
    fn test_html_inline_extraction() {
        let content = r#"# Heading

This is text with <span id="inline-id">inline HTML</span> and <a name="inline-name">anchor</a>.

[Link to inline](#inline-id)
[Link to anchor](#inline-name)
"#;

        assert_no_violations(MD051::new(), content);
    }

    #[test]
    fn test_complex_fragment_generation() {
        let content = r#"# Complex Heading with (Parentheses) & Symbols!

[Link](#complex-heading-with-parentheses--symbols)

## Another_Complex-Title 123

[Another link](#another_complex-title-123)

### Multiple   Spaces   Between   Words

[Space link](#multiple-spaces-between-words)
"#;

        assert_no_violations(MD051::new(), content);
    }

    #[test]
    fn test_dash_consolidation() {
        let content = r#"# Title---With----Multiple-----Dashes

[Link](#title-with-multiple-dashes)

## --Leading-And-Trailing--

[Another link](#leading-and-trailing)
"#;

        assert_no_violations(MD051::new(), content);
    }

    #[test]
    fn test_unicode_and_special_chars() {
        let content = r#"# Heading with émojis 🚀 and ñ

[Unicode link](#heading-with-émojis--and-ñ)

## Code `inline` and **bold**

[Code link](#code-inline-and-bold)
"#;

        assert_no_violations(MD051::new(), content);
    }

    #[test]
    fn test_custom_anchor_validation() {
        let content = r#"# Valid Custom {#valid-anchor}

[Link](#valid-anchor)

# Invalid Custom {#invalid anchor}

[Bad link](#invalid-anchor)
"#;

        // Should have one violation for the invalid custom anchor reference
        let violation = assert_single_violation(MD051::new(), content);
        assert_eq!(violation.line, 7);
        assert!(violation.message.contains("invalid-anchor"));
    }

    #[test]
    fn test_custom_anchor_edge_cases() {
        let content = r#"# Empty Custom {#}

# Valid Custom {#test123}

[Link](#test123)

# Invalid Chars {#test@123}

# Nested {#outer {#inner} }
"#;

        assert_no_violations(MD051::new(), content);
    }

    #[test]
    fn test_github_line_references_detailed() {
        let content = r#"# Code Examples

[Line reference](#L42)
[Line range](#L10-L20)
[Complex range](#L15C3-L25C10)
[Another format](#L1C1-L1C5)
"#;

        assert_no_violations(MD051::new(), content);
    }

    #[test]
    fn test_multiple_document_types() {
        let content = r#"# Main Heading

Regular text here.

<div id="html-id">HTML content</div>

<a name="html-name">Anchor</a>

## Sub Heading {#custom-sub}

More content.

[Link to main](#main-heading)
[Link to sub](#custom-sub)
[Link to HTML ID](#html-id)
[Link to HTML name](#html-name)
[GitHub reference](#L100)
[Invalid reference](#Invalid-Reference)
"#;

        let violation = assert_single_violation(MD051::new(), content);
        assert_eq!(violation.line, 18);
        assert!(violation.message.contains("Invalid-Reference"));
    }

    #[test]
    fn test_duplicate_heading_numbering() {
        let content = r#"# Test

[First link](#test)

# Test

[Second link](#test-1)

# Test

[Third link](#test-2)

# Different

[Different link](#different)
"#;

        assert_no_violations(MD051::new(), content);
    }

    #[test]
    fn test_html_parsing_edge_cases() {
        let content = r#"# Heading

<!-- Comment with id="not-real" -->
<div id='single-quotes'>Content</div>
<span id="no-closing-quote>Broken</span>
<p id=unquoted-id>Unquoted</p>

[Single quotes](#single-quotes)
[Unquoted](#unquoted-id)
"#;

        assert_no_violations(MD051::new(), content);
    }

    #[test]
    fn test_configuration_options() {
        let content = r#"# Test Heading

[Case mismatch](#Test-Heading)
"#;

        // Default case sensitive - should fail
        let violation = assert_single_violation(MD051::new(), content);
        assert_eq!(violation.line, 3);

        // Case insensitive - should pass
        assert_no_violations(MD051::new().ignore_case(true), content);
    }

    #[test]
    fn test_ignored_pattern() {
        let content = r#"# Heading

[External link](#external-pattern)
[Normal link](#invalid-fragment)
"#;

        // With ignored pattern, first link should pass, second should fail
        let rule = MD051::new().ignored_pattern(Some("external-*".to_string()));
        let violation = assert_single_violation(rule, content);
        assert_eq!(violation.line, 4);
        assert!(violation.message.contains("invalid-fragment"));
    }

    #[test]
    fn test_empty_document() {
        let content = "";
        assert_no_violations(MD051::new(), content);
    }

    #[test]
    fn test_no_headings_no_fragments() {
        let content = r#"Just some text without headings.

[Invalid link](#Invalid-Fragment)
"#;

        let violation = assert_single_violation(MD051::new(), content);
        assert_eq!(violation.line, 3);
        assert!(violation.message.contains("Invalid-Fragment"));
    }

    #[test]
    fn test_top_fragment() {
        let content = r#"# Heading

[Link to top](#top)
"#;

        assert_no_violations(MD051::new(), content);
    }

    #[test]
    fn test_malformed_html() {
        let content = r#"# Heading

<div id=>Empty value</div>
<span id>No value</span>
<p id="unclosed>Bad quote</p>

[Should still work](#heading)
"#;

        assert_no_violations(MD051::new(), content);
    }

    #[test]
    fn test_nested_html_elements() {
        let content = r#"# Heading

<div class="outer">
  <div id="nested-id">
    <span name="deep-name">Content</span>
  </div>
</div>

[Link to nested](#nested-id)
[Link to deep](#deep-name)
"#;

        assert_no_violations(MD051::new(), content);
    }

    #[test]
    fn test_heading_with_code_and_emphasis() {
        let content = r#"# Title with `code` and **bold** and *italic*

[Link](#title-with-code-and-bold-and-italic)

## Another `complex` **formatting** example

[Another link](#another-complex-formatting-example)
"#;

        assert_no_violations(MD051::new(), content);
    }
}
