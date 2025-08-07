//! MD030: Spaces after list markers
//!
//! This rule checks for consistent spacing after list markers.
//! Unordered lists should have one space after the marker, and ordered lists should have one space after the period.

use crate::error::Result;
use crate::rule::{Rule, RuleCategory, RuleMetadata};
use crate::{
    Document,
    violation::{Severity, Violation},
};

/// Configuration for spaces after list markers
#[derive(Debug, Clone, PartialEq)]
pub struct MD030Config {
    /// Number of spaces after unordered list markers (default: 1)
    pub ul_single: usize,
    /// Number of spaces after ordered list markers (default: 1)
    pub ol_single: usize,
    /// Number of spaces after unordered list markers in multi-item lists (default: 1)
    pub ul_multi: usize,
    /// Number of spaces after ordered list markers in multi-item lists (default: 1)
    pub ol_multi: usize,
}

impl Default for MD030Config {
    fn default() -> Self {
        Self {
            ul_single: 1,
            ol_single: 1,
            ul_multi: 1,
            ol_multi: 1,
        }
    }
}

/// Rule to check for spaces after list markers
pub struct MD030 {
    config: MD030Config,
}

impl MD030 {
    /// Create a new MD030 rule with default settings
    pub fn new() -> Self {
        Self {
            config: MD030Config::default(),
        }
    }

    /// Create a new MD030 rule with custom configuration
    #[allow(dead_code)]
    pub fn with_config(config: MD030Config) -> Self {
        Self { config }
    }
}

impl Default for MD030 {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for MD030 {
    fn id(&self) -> &'static str {
        "MD030"
    }

    fn name(&self) -> &'static str {
        "list-marker-space"
    }

    fn description(&self) -> &'static str {
        "Spaces after list markers"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Formatting).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let mut in_code_block = false;

        for (line_number, line) in document.lines.iter().enumerate() {
            let line_num = line_number + 1; // Convert to 1-based line numbers

            // Track code block state
            if line.trim_start().starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }

            // Skip lines inside code blocks
            if in_code_block {
                continue;
            }

            if let Some(violation) = self.check_list_marker_spacing(line, line_num) {
                violations.push(violation);
            }
        }

        Ok(violations)
    }
}

impl MD030 {
    /// Check spacing after list markers on a single line
    fn check_list_marker_spacing(&self, line: &str, line_num: usize) -> Option<Violation> {
        let trimmed = line.trim_start();
        let indent_count = line.len() - trimmed.len();

        // Skip setext heading underlines (lines that are all = or - characters)
        if self.is_setext_underline(trimmed) {
            return None;
        }

        // Check for unordered list markers
        if let Some(marker_char) = self.get_unordered_marker(trimmed) {
            let after_marker = &trimmed[1..];
            let whitespace_count = after_marker
                .chars()
                .take_while(|&c| c.is_whitespace())
                .count();
            let expected_spaces = self.config.ul_single; // TODO: Determine if multi-item

            // For expected_spaces = 1: accept exactly 1 space OR exactly 1 tab
            let is_valid_spacing = if expected_spaces == 1 {
                whitespace_count == 1
                    && (after_marker.starts_with(' ') || after_marker.starts_with('\t'))
            } else {
                whitespace_count == expected_spaces
            };

            if !is_valid_spacing {
                return Some(self.create_violation(
                    format!(
                        "Unordered list marker spacing: expected {expected_spaces} space(s) after '{marker_char}', found {whitespace_count}"
                    ),
                    line_num,
                    indent_count + 2, // Position after the marker
                    Severity::Warning,
                ));
            }
        }

        // Check for ordered list markers
        if let Some((number, dot_pos)) = self.get_ordered_marker(trimmed) {
            let after_dot = &trimmed[dot_pos + 1..];
            let whitespace_count = after_dot.chars().take_while(|&c| c.is_whitespace()).count();
            let expected_spaces = self.config.ol_single; // TODO: Determine if multi-item

            // For expected_spaces = 1: accept exactly 1 space OR exactly 1 tab
            let is_valid_spacing = if expected_spaces == 1 {
                whitespace_count == 1 && (after_dot.starts_with(' ') || after_dot.starts_with('\t'))
            } else {
                whitespace_count == expected_spaces
            };

            if !is_valid_spacing {
                return Some(self.create_violation(
                    format!(
                        "Ordered list marker spacing: expected {expected_spaces} space(s) after '{number}. ', found {whitespace_count}"
                    ),
                    line_num,
                    indent_count + dot_pos + 2, // Position after the dot
                    Severity::Warning,
                ));
            }
        }

        None
    }

    /// Get unordered list marker character if line starts with one
    fn get_unordered_marker(&self, trimmed: &str) -> Option<char> {
        let first_char = trimmed.chars().next()?;
        match first_char {
            '-' | '*' | '+' => {
                // Check if this is actually emphasis syntax, not a list marker
                if self.is_emphasis_syntax(trimmed, first_char) {
                    return None;
                }
                Some(first_char)
            }
            _ => None,
        }
    }

    /// Check if a line starting with *, -, or + is actually emphasis/bold syntax
    fn is_emphasis_syntax(&self, trimmed: &str, marker: char) -> bool {
        // Check for bold syntax: **text** or __text__
        if marker == '*' && trimmed.starts_with("**") {
            return true;
        }
        if marker == '_' && trimmed.starts_with("__") {
            return true;
        }

        // Check for italic syntax that's not a list: *text* (but allow "* text" as list)
        if marker == '*' {
            // If there's immediately non-whitespace after the *, it's likely emphasis
            if let Some(second_char) = trimmed.chars().nth(1)
                && !second_char.is_whitespace()
                && second_char != '*'
                && let Some(closing_pos) = trimmed[2..].find('*')
            {
                // Make sure it's not just another list item with * in the text
                let text_between = &trimmed[1..closing_pos + 2];
                if !text_between.contains('\n') && closing_pos < 50 {
                    // Likely emphasis if reasonably short and no newlines
                    return true;
                }
            }
        }

        // For - and +, only consider them emphasis in very specific cases
        // Most of the time, these should be treated as potential list markers
        // We'll be conservative here and only exclude obvious non-list cases

        false
    }

    /// Get ordered list marker number and dot position if line starts with one
    fn get_ordered_marker(&self, trimmed: &str) -> Option<(String, usize)> {
        // Look for pattern like "1. " or "42. "
        let dot_pos = trimmed.find('.')?;
        let prefix = &trimmed[..dot_pos];

        // Check if prefix is all digits
        if prefix.chars().all(|c| c.is_ascii_digit()) && !prefix.is_empty() {
            Some((prefix.to_string(), dot_pos))
        } else {
            None
        }
    }

    /// Check if a line is a setext heading underline (all = or - characters)
    fn is_setext_underline(&self, trimmed: &str) -> bool {
        if trimmed.is_empty() {
            return false;
        }

        let first_char = trimmed.chars().next().unwrap();
        (first_char == '=' || first_char == '-') && trimmed.chars().all(|c| c == first_char)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use crate::rule::Rule;
    use std::path::PathBuf;

    #[test]
    fn test_md030_no_violations() {
        let content = r#"# Valid List Spacing

Unordered lists with single space:
- Item 1
* Item 2
+ Item 3

Ordered lists with single space:
1. First item
2. Second item
42. Item with large number

Regular text here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD030::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md030_unordered_multiple_spaces() {
        let content = r#"# Unordered List Spacing Issues

- Single space is fine
-  Two spaces after dash
*   Three spaces after asterisk
+    Four spaces after plus

Regular text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD030::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);
        assert!(
            violations[0]
                .message
                .contains("expected 1 space(s) after '-', found 2")
        );
        assert!(
            violations[1]
                .message
                .contains("expected 1 space(s) after '*', found 3")
        );
        assert!(
            violations[2]
                .message
                .contains("expected 1 space(s) after '+', found 4")
        );
        assert_eq!(violations[0].line, 4);
        assert_eq!(violations[1].line, 5);
        assert_eq!(violations[2].line, 6);
    }

    #[test]
    fn test_md030_ordered_multiple_spaces() {
        let content = r#"# Ordered List Spacing Issues

1. Single space is fine
2.  Two spaces after number
42.   Three spaces after large number
100.    Four spaces after even larger number

Regular text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD030::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);
        assert!(
            violations[0]
                .message
                .contains("expected 1 space(s) after '2. ', found 2")
        );
        assert!(
            violations[1]
                .message
                .contains("expected 1 space(s) after '42. ', found 3")
        );
        assert!(
            violations[2]
                .message
                .contains("expected 1 space(s) after '100. ', found 4")
        );
        assert_eq!(violations[0].line, 4);
        assert_eq!(violations[1].line, 5);
        assert_eq!(violations[2].line, 6);
    }

    #[test]
    fn test_md030_no_spaces_after_marker() {
        let content = r#"# No Spaces After Markers

-No space after dash
*No space after asterisk
+No space after plus
1.No space after number
42.No space after large number

Text here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD030::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 5);
        for violation in &violations {
            assert!(violation.message.contains("expected 1 space(s)"));
            assert!(violation.message.contains("found 0"));
        }
    }

    #[test]
    fn test_md030_custom_config() {
        let content = r#"# Custom Configuration Test

- Single space (should be invalid)
-  Two spaces (should be valid)
1. Single space (should be invalid)
2.  Two spaces (should be valid)

Text here.
"#;
        let config = MD030Config {
            ul_single: 2,
            ol_single: 2,
            ul_multi: 2,
            ol_multi: 2,
        };
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD030::with_config(config);
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);
        assert!(
            violations[0]
                .message
                .contains("expected 2 space(s) after '-', found 1")
        );
        assert!(
            violations[1]
                .message
                .contains("expected 2 space(s) after '1. ', found 1")
        );
        assert_eq!(violations[0].line, 3);
        assert_eq!(violations[1].line, 5);
    }

    #[test]
    fn test_md030_indented_lists() {
        let content = r#"# Moderately Indented Lists

  - Moderately indented item
  -  Too many spaces
  * Another marker type
  *   Too many spaces here too

Regular text here.

1. Regular ordered list
2.  Too many spaces
42. Correct spacing
100.   Too many spaces

Text here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD030::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 4);
        assert_eq!(violations[0].line, 4); // -  Too many spaces
        assert_eq!(violations[1].line, 6); // *   Too many spaces here too
        assert_eq!(violations[2].line, 11); // 2.  Too many spaces
        assert_eq!(violations[3].line, 13); // 100.   Too many spaces
    }

    #[test]
    fn test_md030_nested_lists() {
        let content = r#"# Nested Lists

- Top level item
  - Nested item with correct spacing
  -  Nested item with too many spaces
  * Different marker type
  *   Too many spaces with asterisk
    1. Nested ordered list
    2.  Too many spaces in nested ordered
    3. Correct spacing

More text.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD030::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].line, 5); // -  Nested item with too many spaces
        assert_eq!(violations[1].line, 7); // *   Too many spaces with asterisk
        assert_eq!(violations[2].line, 9); // 2.  Too many spaces in nested ordered
    }

    #[test]
    fn test_md030_mixed_violations() {
        let content = r#"# Mixed Violations

- Correct spacing
-  Too many spaces
* Correct spacing
*No spaces
+ Correct spacing
+   Way too many spaces

1. Correct spacing
2.  Too many spaces
3. Correct spacing
42.No spaces
100.     Many spaces

Text here.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD030::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 6);
        // Unordered violations
        assert_eq!(violations[0].line, 4); // -  Too many spaces
        assert_eq!(violations[1].line, 6); // *No spaces
        assert_eq!(violations[2].line, 8); // +   Way too many spaces
        // Ordered violations
        assert_eq!(violations[3].line, 11); // 2.  Too many spaces
        assert_eq!(violations[4].line, 13); // 42.No spaces
        assert_eq!(violations[5].line, 14); // 100.     Many spaces
    }

    #[test]
    fn test_md030_tabs_after_markers() {
        let content = "- Item with tab\t\n*\tItem starting with tab\n1.\tOrdered with tab\n42.\t\tMultiple tabs\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD030::new();
        let violations = rule.check(&document).unwrap();

        // Single tabs should be treated as valid (equivalent to single space)
        // Only multiple tabs should be flagged as violations
        assert_eq!(violations.len(), 1); // Only the multiple tabs case should be flagged
        assert_eq!(violations[0].line, 4); // 42.\t\tMultiple tabs
    }

    #[test]
    fn test_md030_get_markers() {
        let rule = MD030::new();

        // Unordered markers
        assert_eq!(rule.get_unordered_marker("- Item"), Some('-'));
        assert_eq!(rule.get_unordered_marker("* Item"), Some('*'));
        assert_eq!(rule.get_unordered_marker("+ Item"), Some('+'));
        assert_eq!(rule.get_unordered_marker("Not a marker"), None);
        assert_eq!(rule.get_unordered_marker("1. Ordered"), None);

        // Ordered markers
        assert_eq!(
            rule.get_ordered_marker("1. Item"),
            Some(("1".to_string(), 1))
        );
        assert_eq!(
            rule.get_ordered_marker("42. Item"),
            Some(("42".to_string(), 2))
        );
        assert_eq!(
            rule.get_ordered_marker("100. Item"),
            Some(("100".to_string(), 3))
        );
        assert_eq!(rule.get_ordered_marker("- Unordered"), None);
        assert_eq!(rule.get_ordered_marker("Not a list"), None);
        assert_eq!(rule.get_ordered_marker("a. Letter"), None);
    }

    #[test]
    fn test_md030_setext_headings_ignored() {
        let content = r#"Main Heading
============

Some content here.

Subheading
----------

More content.

- This is a real list
- With proper spacing
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD030::new();
        let violations = rule.check(&document).unwrap();

        // Should have no violations - setext underlines should be ignored
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md030_is_setext_underline() {
        let rule = MD030::new();

        // Valid setext underlines
        assert!(rule.is_setext_underline("============"));
        assert!(rule.is_setext_underline("----------"));
        assert!(rule.is_setext_underline("==="));
        assert!(rule.is_setext_underline("---"));
        assert!(rule.is_setext_underline("="));
        assert!(rule.is_setext_underline("-"));

        // Not setext underlines
        assert!(!rule.is_setext_underline(""));
        assert!(!rule.is_setext_underline("- Item"));
        assert!(!rule.is_setext_underline("=-="));
        assert!(!rule.is_setext_underline("=== Header ==="));
        assert!(!rule.is_setext_underline("-- Comment --"));
        assert!(!rule.is_setext_underline("* Not a setext"));
        assert!(!rule.is_setext_underline("+ Also not"));
    }

    #[test]
    fn test_md030_bold_text_not_flagged() {
        let content = r#"# Bold Text Should Not Be Flagged

**Types**: feat, fix, docs
**Scopes**: cli, preprocessor, rules
**Important**: This is bold text, not a list marker

Regular bold text like **this** should be fine.
Italic text like *this* should also be fine.

But actual lists should still be checked:
- Valid list item
-  Invalid spacing (should be flagged)
* Another valid item
*  Invalid spacing (should be flagged)
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD030::new();
        let violations = rule.check(&document).unwrap();

        // Should only flag the actual list items with bad spacing, not the bold text
        assert_eq!(violations.len(), 2);
        assert!(
            violations[0]
                .message
                .contains("expected 1 space(s) after '-', found 2")
        );
        assert!(
            violations[1]
                .message
                .contains("expected 1 space(s) after '*', found 2")
        );
        assert_eq!(violations[0].line, 12); // -  Invalid spacing (corrected line number)
        assert_eq!(violations[1].line, 14); // *  Invalid spacing (corrected line number)
    }

    #[test]
    fn test_md030_emphasis_syntax_detection() {
        let rule = MD030::new();

        // Bold syntax should be detected as emphasis
        assert!(rule.is_emphasis_syntax("**bold text**", '*'));
        assert!(rule.is_emphasis_syntax("**Types**: something", '*'));
        assert!(rule.is_emphasis_syntax("__bold text__", '_'));

        // Italic syntax should be detected as emphasis
        assert!(rule.is_emphasis_syntax("*italic text*", '*'));
        assert!(rule.is_emphasis_syntax("*word*", '*'));

        // List markers should NOT be detected as emphasis
        assert!(!rule.is_emphasis_syntax("* List item", '*'));
        assert!(!rule.is_emphasis_syntax("- List item", '-'));
        assert!(!rule.is_emphasis_syntax("+ List item", '+'));
        assert!(!rule.is_emphasis_syntax("*  List with extra spaces", '*'));

        // Edge cases
        assert!(!rule.is_emphasis_syntax("* ", '*')); // Just marker and space
        assert!(!rule.is_emphasis_syntax("*", '*')); // Just marker
        assert!(!rule.is_emphasis_syntax("*text with no closing", '*')); // No closing marker
    }

    #[test]
    fn test_md030_mixed_emphasis_and_lists() {
        let content = r#"# Mixed Content

**Bold**: This should not be flagged
*Italic*: This should not be flagged

Valid lists:
- Item one
* Item two  
+ Item three

Invalid lists:
-  Too many spaces after dash
*  Too many spaces after asterisk
+  Too many spaces after plus

More **bold text** that should be ignored.
And some *italic text* that should be ignored.
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD030::new();
        let violations = rule.check(&document).unwrap();

        // Should only flag the 3 invalid list items, not the emphasis text
        assert_eq!(violations.len(), 3);
        for violation in &violations {
            assert!(violation.message.contains("expected 1 space(s)"));
            assert!(violation.message.contains("found 2"));
        }
        assert_eq!(violations[0].line, 12); // -  Too many spaces after dash
        assert_eq!(violations[1].line, 13); // *  Too many spaces after asterisk  
        assert_eq!(violations[2].line, 14); // +  Too many spaces after plus
    }

    #[test]
    fn test_md030_get_unordered_marker_with_emphasis() {
        let rule = MD030::new();

        // Should return marker for actual lists
        assert_eq!(rule.get_unordered_marker("- List item"), Some('-'));
        assert_eq!(rule.get_unordered_marker("* List item"), Some('*'));
        assert_eq!(rule.get_unordered_marker("+ List item"), Some('+'));

        // Should NOT return marker for emphasis syntax
        assert_eq!(rule.get_unordered_marker("**Bold text**"), None);
        assert_eq!(rule.get_unordered_marker("*Italic text*"), None);
        assert_eq!(rule.get_unordered_marker("**Types**: something"), None);

        // Edge cases
        assert_eq!(rule.get_unordered_marker("Not a list"), None);
        assert_eq!(rule.get_unordered_marker("1. Ordered list"), None);
    }

    #[test]
    fn test_md030_code_blocks_ignored() {
        let content = r#"# Test Code Blocks

Valid list:
- Item one

```bash
# Deploy with CLI flags - these should not trigger MD030
rot deploy --admin-password secret123 \
  --database-name myapp \
  --port 6379

# List items that look like markdown but are inside code
- Not a real list item, just text
* Also not a real list item  
1. Not an ordered list either
```

Another list:
- Item two
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD030::new();
        let violations = rule.check(&document).unwrap();

        // Should have no violations - all apparent list markers are inside code blocks
        // except the real list items which are properly formatted
        assert_eq!(violations.len(), 0);
    }
}
