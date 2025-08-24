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
                    _ => {}
                }
            }
        }

        used_labels
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

        if !found_first_closing {
            return None;
        }

        // Check what follows
        if let Some((_, next_ch)) = chars.next()
            && next_ch == '['
        {
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
}
