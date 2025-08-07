use crate::Document;
use crate::error::Result;
use crate::rule::{Rule, RuleCategory, RuleMetadata};
use crate::violation::{Severity, Violation};
use comrak::nodes::AstNode;

/// MD007 - Unordered list indentation
pub struct MD007 {
    /// Number of spaces for indent (default: 2)
    pub indent: usize,
    /// Spaces for first level indent when start_indented is set (default: 2)
    pub start_indent: usize,
    /// Whether to indent the first level of the list (default: false)
    pub start_indented: bool,
}

impl MD007 {
    pub fn new() -> Self {
        Self {
            indent: 2,
            start_indent: 2,
            start_indented: false,
        }
    }

    #[allow(dead_code)]
    pub fn with_indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }

    #[allow(dead_code)]
    pub fn with_start_indent(mut self, start_indent: usize) -> Self {
        self.start_indent = start_indent;
        self
    }

    #[allow(dead_code)]
    pub fn with_start_indented(mut self, start_indented: bool) -> Self {
        self.start_indented = start_indented;
        self
    }

    fn calculate_expected_indent(&self, depth: usize) -> usize {
        if depth == 0 {
            if self.start_indented {
                self.start_indent
            } else {
                0
            }
        } else {
            let base = if self.start_indented {
                self.start_indent
            } else {
                0
            };
            base + depth * self.indent
        }
    }

    fn parse_list_item(&self, line: &str) -> Option<(usize, char, bool)> {
        let mut indent = 0;
        let mut chars = line.chars();

        // Count leading spaces
        while let Some(ch) = chars.next() {
            if ch == ' ' {
                indent += 1;
            } else if ch == '\t' {
                indent += 4; // Treat tab as 4 spaces
            } else if matches!(ch, '*' | '+' | '-') {
                // Check if followed by whitespace (valid list marker)
                if let Some(next_ch) = chars.next() {
                    if next_ch.is_whitespace() {
                        return Some((indent, ch, false)); // false = unordered
                    }
                }
                break;
            } else if ch.is_ascii_digit() {
                // Check for ordered list (digit followed by . or ))
                let mut temp_chars = chars.as_str().chars();
                while let Some(digit_ch) = temp_chars.next() {
                    if digit_ch == '.' || digit_ch == ')' {
                        if let Some(next_ch) = temp_chars.next() {
                            if next_ch.is_whitespace() {
                                return Some((indent, ch, true)); // true = ordered
                            }
                        }
                        break;
                    } else if !digit_ch.is_ascii_digit() {
                        break;
                    }
                }
                break;
            } else {
                break;
            }
        }

        None
    }

    fn calculate_depth(&self, list_stack: &[(usize, char, bool)], current_indent: usize) -> usize {
        // Find the depth based on indentation level
        for (i, &(stack_indent, _, _)) in list_stack.iter().enumerate() {
            if current_indent <= stack_indent {
                return i;
            }
        }
        list_stack.len()
    }

    fn update_list_stack(
        &self,
        list_stack: &mut Vec<(usize, char, bool)>,
        indent: usize,
        marker: char,
        is_ordered: bool,
    ) {
        // Remove items with greater or equal indentation
        list_stack.retain(|&(stack_indent, _, _)| stack_indent < indent);

        // Add current item
        list_stack.push((indent, marker, is_ordered));
    }

    fn has_ordered_ancestors(&self, list_stack: &[(usize, char, bool)]) -> bool {
        list_stack.iter().any(|&(_, _, is_ordered)| is_ordered)
    }
}

impl Default for MD007 {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for MD007 {
    fn id(&self) -> &'static str {
        "MD007"
    }

    fn name(&self) -> &'static str {
        "ul-indent"
    }

    fn description(&self) -> &'static str {
        "Unordered list indentation"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Formatting)
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let lines: Vec<&str> = document.content.lines().collect();

        let mut list_stack: Vec<(usize, char, bool)> = Vec::new(); // (indent, marker, is_ordered)

        for (line_number, line) in lines.iter().enumerate() {
            let line_number = line_number + 1;

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Check if this line is a list item
            if let Some((indent, marker, is_ordered)) = self.parse_list_item(line) {
                // Only check unordered lists and only if all ancestors are unordered
                if !is_ordered && !self.has_ordered_ancestors(&list_stack) {
                    // Calculate expected indentation
                    let current_depth = self.calculate_depth(&list_stack, indent);
                    let expected_indent = self.calculate_expected_indent(current_depth);

                    if indent != expected_indent {
                        violations.push(self.create_violation(
                            format!(
                                "Unordered list indentation: Expected {expected_indent} spaces, found {indent}"
                            ),
                            line_number,
                            indent + 1, // Convert to 1-based column
                            Severity::Warning,
                        ));
                    }
                }

                // Update the list stack
                self.update_list_stack(&mut list_stack, indent, marker, is_ordered);
            } else {
                // Non-list line resets the stack
                list_stack.clear();
            }
        }

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use std::path::PathBuf;

    #[test]
    fn test_md007_correct_indentation() {
        let content = r#"* Item 1
  * Nested item (2 spaces)
    * Deep nested item (4 spaces)
* Item 2
  * Another nested item
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD007::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md007_incorrect_indentation() {
        let content = r#"* Item 1
   * Nested item (3 spaces - wrong!)
     * Deep nested item (5 spaces - wrong!)
* Item 2
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD007::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 2);
        assert_eq!(violations[1].line, 3);
        assert!(violations[0].message.contains("Expected 2 spaces, found 3"));
        assert!(violations[1].message.contains("Expected 4 spaces, found 5"));
    }

    #[test]
    fn test_md007_custom_indent() {
        let content = r#"* Item 1
    * Nested item (4 spaces)
        * Deep nested item (8 spaces)
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD007::new().with_indent(4);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md007_start_indented() {
        let content = r#"  * Item 1 (2 spaces start)
    * Nested item (4 spaces total)
      * Deep nested item (6 spaces total)
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD007::new().with_start_indented(true);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md007_start_indented_custom() {
        let content = r#"    * Item 1 (4 spaces start)
        * Nested item (8 spaces total)
            * Deep nested item (12 spaces total)
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD007::new()
            .with_start_indented(true)
            .with_start_indent(4)
            .with_indent(4);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md007_mixed_list_types() {
        let content = r#"1. Ordered item
   * Unordered nested (should be ignored due to ordered parent)
     * Deep nested (should be ignored)
* Unordered item
  * Unordered nested (should be checked)
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD007::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0); // Mixed list types are ignored
    }

    #[test]
    fn test_md007_only_unordered_lists() {
        let content = r#"1. Ordered item
   2. Another ordered item (wrong indentation but ignored)
      3. Deep ordered item (also ignored)
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD007::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md007_no_indentation_needed() {
        let content = r#"* Item 1
* Item 2
* Item 3
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD007::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md007_zero_indentation_with_start_indented() {
        let content = r#"* Item 1 (should be indented)
* Item 2 (should be indented)
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD007::new().with_start_indented(true);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("Expected 2 spaces, found 0"));
        assert!(violations[1].message.contains("Expected 2 spaces, found 0"));
    }

    #[test]
    fn test_md007_complex_nesting() {
        let content = r#"* Level 1
  * Level 2 correct
    * Level 3 correct
      * Level 4 correct
   * Level 2 wrong (3 spaces)
     * Level 3 wrong (5 spaces)
"#;

        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD007::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 5);
        assert_eq!(violations[1].line, 6);
    }
}
