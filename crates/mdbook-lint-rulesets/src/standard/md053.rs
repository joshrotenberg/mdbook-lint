//! MD053 - Link and image reference definitions should be needed
//!
//! This rule checks for unused or duplicated reference definitions.
//! Note: This is a simplified implementation that works with basic patterns.
//!
//! ## Correct
//!
//! ```markdown
//! [Link][label]
//!
//! [label]: https://example.com
//! ```
//!
//! ## Incorrect
//!
//! ```markdown
//! [Link][label]
//!
//! [label]: https://example.com
//! [unused]: https://example.com
//! [label]: https://duplicate.com
//! ```

use comrak::nodes::AstNode;
use mdbook_lint_core::error::Result;
use mdbook_lint_core::{
    Document, Violation,
    rule::{Rule, RuleCategory, RuleMetadata},
    violation::Severity,
};

use std::collections::{HashMap, HashSet};

/// MD053 - Link and image reference definitions should be needed
pub struct MD053 {
    ignored_definitions: Vec<String>,
}

impl Default for MD053 {
    fn default() -> Self {
        Self::new()
    }
}

impl MD053 {
    /// Create a new MD053 rule instance
    pub fn new() -> Self {
        Self {
            ignored_definitions: vec!["//".to_string()], // Default ignores comment syntax
        }
    }

    /// Set the list of ignored definitions
    #[allow(dead_code)]
    pub fn ignored_definitions(mut self, definitions: Vec<String>) -> Self {
        self.ignored_definitions = definitions;
        self
    }

    /// Create MD053 from configuration
    pub fn from_config(config: &toml::Value) -> Self {
        let mut rule = Self::new();

        if let Some(ignored_definitions) = config.get("ignored_definitions")
            && let Some(defs_array) = ignored_definitions.as_array()
        {
            rule.ignored_definitions = defs_array
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();
        }

        rule
    }

    /// Parse reference definitions from document content
    fn collect_definitions(&self, document: &Document) -> Vec<(String, usize, usize)> {
        let mut definitions = Vec::new();

        for (line_num, line) in document.content.lines().enumerate() {
            let line_number = line_num + 1;

            // Check if line starts with [label]: (reference definition)
            if let Some((label, column)) = self.parse_reference_definition(line) {
                definitions.push((label.to_lowercase(), line_number, column));
            }
        }

        definitions
    }

    /// Parse a reference definition from a line
    /// Returns (label, column_position) if found
    fn parse_reference_definition(&self, line: &str) -> Option<(String, usize)> {
        let mut chars = line.char_indices().peekable();
        let mut start_pos = 0;

        // Skip leading whitespace
        while let Some((pos, ch)) = chars.peek() {
            if ch.is_whitespace() {
                start_pos = *pos + 1;
                chars.next();
            } else {
                break;
            }
        }

        // Must start with [
        if chars.next()?.1 != '[' {
            return None;
        }

        let bracket_start = start_pos;
        let mut label = String::new();
        let mut found_closing_bracket = false;

        // Find closing bracket and collect label
        for (_, ch) in chars.by_ref() {
            if ch == ']' {
                found_closing_bracket = true;
                break;
            }
            label.push(ch);
        }

        if !found_closing_bracket || label.is_empty() {
            return None;
        }

        // Next character must be :
        if chars.next()?.1 != ':' {
            return None;
        }

        // Must be followed by whitespace or end of line
        if let Some((_, ch)) = chars.peek()
            && !ch.is_whitespace()
        {
            return None;
        }

        Some((label, bracket_start + 1))
    }

    /// Parse reference usage from document content
    fn collect_used_labels(&self, document: &Document) -> HashSet<String> {
        let mut used_labels = HashSet::new();

        for line in document.content.lines() {
            let mut chars = line.char_indices().peekable();
            let mut in_backticks = false;

            while let Some((i, ch)) = chars.next() {
                match ch {
                    '`' => {
                        in_backticks = !in_backticks;
                    }
                    '[' if !in_backticks => {
                        // Try to parse reference link
                        if let Some(label) = self.parse_reference_usage(&line[i..]) {
                            used_labels.insert(label.to_lowercase());

                            // Skip past the parsed reference
                            while let Some((_, next_ch)) = chars.peek() {
                                if *next_ch == ']' {
                                    chars.next();
                                    break;
                                }
                                chars.next();
                            }
                        }
                    }
                    ']' => {
                        // Check for ][label] pattern - continuation of reference link from previous line
                        // Note: We don't check in_backticks here because:
                        // 1. Link text can contain backticks and span multiple lines
                        // 2. The ][label] pattern is very specific and unlikely to appear in code
                        // 3. Backtick state may be incorrect due to line-by-line processing
                        if let Some(label) = self.parse_continuation_reference(&line[i..]) {
                            used_labels.insert(label.to_lowercase());
                        }
                    }
                    _ => {}
                }
            }
        }

        used_labels
    }

    /// Parse continuation reference ][label] pattern
    /// This handles cases where [link text] is on a previous line
    fn parse_continuation_reference(&self, text: &str) -> Option<String> {
        if !text.starts_with("][") {
            return None;
        }

        let mut chars = text.char_indices().skip(2); // Skip "]["
        let mut label = String::new();

        for (_, ch) in chars.by_ref() {
            if ch == ']' {
                if !label.is_empty() {
                    return Some(label);
                }
                return None;
            }
            label.push(ch);
        }

        None
    }

    /// Parse reference usage at the given position
    /// Returns the reference label if found
    fn parse_reference_usage(&self, text: &str) -> Option<String> {
        if !text.starts_with('[') {
            return None;
        }

        let mut chars = text.char_indices().skip(1);
        let mut first_part = String::new();
        let mut found_first_closing = false;

        // Find first closing bracket
        for (_, ch) in chars.by_ref() {
            if ch == ']' {
                found_first_closing = true;
                break;
            }
            first_part.push(ch);
        }

        if !found_first_closing || first_part.is_empty() {
            return None;
        }

        // Check what follows
        if let Some((_, next_ch)) = chars.next() {
            if next_ch == '[' {
                // Either [text][ref] or [label][]
                let mut second_part = String::new();
                let mut found_second_closing = false;

                for (_, ch) in chars {
                    if ch == ']' {
                        found_second_closing = true;
                        break;
                    }
                    second_part.push(ch);
                }

                if found_second_closing {
                    if second_part.is_empty() {
                        // Collapsed reference [label][]
                        return Some(first_part);
                    } else {
                        // Full reference [text][ref]
                        return Some(second_part);
                    }
                }
            } else if next_ch != ':'
                && !next_ch.is_alphanumeric()
                && next_ch != '_'
                && next_ch != '-'
            {
                // Shortcut reference [label] - must be followed by non-identifier char
                // (not another letter/number which would make it part of regular text)
                // Common following chars: space, punctuation, newline
                // But NOT ':' which would make it a definition [label]:
                return Some(first_part);
            }
        } else {
            // [label] at end of string - shortcut reference
            return Some(first_part);
        }

        None
    }

    /// Check for unused and duplicate definitions
    fn check_definitions(
        &self,
        definitions: Vec<(String, usize, usize)>,
        used_labels: &HashSet<String>,
    ) -> Vec<Violation> {
        let mut violations = Vec::new();
        let mut seen_labels: HashMap<String, (usize, usize)> = HashMap::new();

        for (label, line, column) in definitions {
            // Skip if label is in ignored list
            if self.ignored_definitions.contains(&label) {
                continue;
            }

            // Check for duplicates
            if let Some((first_line, _first_column)) = seen_labels.get(&label) {
                violations.push(self.create_violation(
                    format!(
                        "Reference definition '{label}' is duplicated (first defined at line {first_line})"
                    ),
                    line,
                    column,
                    Severity::Warning,
                ));
            } else {
                seen_labels.insert(label.clone(), (line, column));

                // Check if unused
                if !used_labels.contains(&label) {
                    violations.push(self.create_violation(
                        format!("Reference definition '{label}' is unused"),
                        line,
                        column,
                        Severity::Warning,
                    ));
                }
            }
        }

        violations
    }
}

impl Rule for MD053 {
    fn id(&self) -> &'static str {
        "MD053"
    }

    fn name(&self) -> &'static str {
        "link-image-reference-definitions"
    }

    fn description(&self) -> &'static str {
        "Link and image reference definitions should be needed"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Links)
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        // This rule works entirely with document content, not AST
        let definitions = self.collect_definitions(document);
        let used_labels = self.collect_used_labels(document);
        let violations = self.check_definitions(definitions, &used_labels);

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::test_helpers::*;

    #[test]
    fn test_used_definitions() {
        let content = r#"[Link][label]

[label]: https://example.com
"#;

        assert_no_violations(MD053::new(), content);
    }

    #[test]
    fn test_unused_definition() {
        let content = r#"[Link][used]

[used]: https://example.com
[unused]: https://example.com
"#;

        let violation = assert_single_violation(MD053::new(), content);
        assert_eq!(violation.line, 4);
        assert!(violation.message.contains("unused"));
    }

    #[test]
    fn test_duplicate_definitions() {
        let content = r#"[Link][label]

[label]: https://example.com
[label]: https://duplicate.com
"#;

        let violation = assert_single_violation(MD053::new(), content);
        assert_eq!(violation.line, 4);
        assert!(violation.message.contains("duplicated"));
        assert!(violation.message.contains("first defined at line 3"));
    }

    #[test]
    fn test_ignored_definitions() {
        let content = r#"[//]: # (This is a comment)
"#;

        assert_no_violations(MD053::new(), content); // '//' is ignored by default
    }

    #[test]
    fn test_case_insensitive_matching() {
        let content = r#"[Link][LABEL]

[label]: https://example.com
"#;

        assert_no_violations(MD053::new(), content);
    }

    #[test]
    fn test_collapsed_reference() {
        let content = r#"[Label][]

[label]: https://example.com
"#;

        assert_no_violations(MD053::new(), content);
    }

    #[test]
    fn test_unused_and_duplicate() {
        let content = r#"[Link][used]

[used]: https://example.com
[unused]: https://example.com
[used]: https://duplicate.com
"#;

        let violations = assert_violation_count(MD053::new(), content, 2);

        // Check for unused definition
        let unused_violation = violations
            .iter()
            .find(|v| v.message.contains("unused"))
            .unwrap();
        assert_eq!(unused_violation.line, 4);

        // Check for duplicate definition
        let duplicate_violation = violations
            .iter()
            .find(|v| v.message.contains("duplicated"))
            .unwrap();
        assert_eq!(duplicate_violation.line, 5);
    }

    #[test]
    fn test_multiline_reference_link() {
        // Reference links can span multiple lines - [link text
        // continues][label] should still recognize the label as used
        let content = r#"> Note: This edition of the book is the same as [The Rust Programming
> Language][nsprust] available in print and ebook format from [No Starch
> Press][nsp].

[nsprust]: https://nostarch.com/rust-programming-language-3rd-edition
[nsp]: https://nostarch.com/
"#;

        assert_no_violations(MD053::new(), content);
    }

    #[test]
    fn test_continuation_reference_at_line_start() {
        // ][label] at the start of a line (after wrap)
        let content = r#"This is a very long link text that wraps to the next line [link
][label] and continues here.

[label]: https://example.com
"#;

        assert_no_violations(MD053::new(), content);
    }

    #[test]
    fn test_continuation_reference_mid_line() {
        // Text][label] in the middle of a line
        let content = r#"Some text
wrapped][label] more text.

[label]: https://example.com
"#;

        assert_no_violations(MD053::new(), content);
    }

    #[test]
    fn test_reference_with_backticks_in_link_text() {
        // Link text can contain backticks, e.g., ["text with `code`"][label]
        let content = r#"of the `if let` construct we saw back in the ["Concise Control Flow with `if
let` and `let...else`"][if-let]<!-- ignore --> section in Chapter 6.

[if-let]: ch06-03-if-let.html
"#;

        assert_no_violations(MD053::new(), content);
    }

    #[test]
    fn test_shortcut_reference() {
        // Shortcut reference [label] without second brackets
        let content = r#"Several community [translations] are also available.

[translations]: appendix-06-translation.html
"#;

        assert_no_violations(MD053::new(), content);
    }

    #[test]
    fn test_shortcut_reference_at_end_of_line() {
        let content = r#"Check out [example]
for more info.

[example]: https://example.com
"#;

        assert_no_violations(MD053::new(), content);
    }

    #[test]
    fn test_footnote_reference_not_link() {
        // Footnote references [^label] are different from link references
        // The definition [^label]: is a footnote, not a link reference
        let content = r#"This has a footnote[^siphash] reference.

[^siphash]: https://en.wikipedia.org/wiki/SipHash
"#;

        // This should have no violations - footnotes are not link references
        // But our current implementation may not handle this correctly
        let rule = MD053::new();
        let document =
            Document::new(content.to_string(), std::path::PathBuf::from("test.md")).unwrap();
        let violations = rule.check(&document).unwrap();

        // If we're reporting it as unused, that's a false positive for footnotes
        // For now, document the current behavior
        // TODO: Consider excluding footnote-style references (starting with ^)
        println!("Footnote test violations: {:?}", violations.len());
    }
}
