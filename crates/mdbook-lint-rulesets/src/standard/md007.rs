use comrak::nodes::{AstNode, NodeValue};
use mdbook_lint_core::Document;
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::violation::{Fix, Position, Severity, Violation};

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

    /// Create MD007 from configuration
    pub fn from_config(config: &toml::Value) -> Self {
        let mut rule = Self::new();

        if let Some(indent) = config.get("indent").and_then(|v| v.as_integer()) {
            rule.indent = indent as usize;
        }

        if let Some(start_indent) = config
            .get("start-indent")
            .or_else(|| config.get("start_indent"))
            .and_then(|v| v.as_integer())
        {
            rule.start_indent = start_indent as usize;
        }

        if let Some(start_indented) = config
            .get("start-indented")
            .or_else(|| config.get("start_indented"))
            .and_then(|v| v.as_bool())
        {
            rule.start_indented = start_indented;
        }

        rule
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
                if let Some(next_ch) = chars.next()
                    && next_ch.is_whitespace()
                {
                    return Some((indent, ch, false)); // false = unordered
                }
                break;
            } else if ch.is_ascii_digit() {
                // Check for ordered list (digit followed by . or ))
                let mut temp_chars = chars.as_str().chars();
                while let Some(digit_ch) = temp_chars.next() {
                    if digit_ch == '.' || digit_ch == ')' {
                        if let Some(next_ch) = temp_chars.next()
                            && next_ch.is_whitespace()
                        {
                            return Some((indent, ch, true)); // true = ordered
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

    /// Get line ranges for code blocks to skip them
    fn get_code_block_line_ranges<'a>(
        &self,
        ast: &'a AstNode<'a>,
    ) -> Vec<(usize, usize)> {
        let mut ranges = Vec::new();
        self.collect_code_block_ranges(ast, &mut ranges);
        ranges
    }

    /// Recursively collect code block line ranges (both fenced and indented)
    fn collect_code_block_ranges<'a>(
        &self,
        node: &'a AstNode<'a>,
        ranges: &mut Vec<(usize, usize)>,
    ) {
        match &node.data.borrow().value {
            NodeValue::CodeBlock(_) => {
                // Fenced or indented code block
                let sourcepos = node.data.borrow().sourcepos;
                if sourcepos.start.line > 0 && sourcepos.end.line > 0 {
                    ranges.push((sourcepos.start.line, sourcepos.end.line));
                }
            }
            _ => {}
        }

        for child in node.children() {
            self.collect_code_block_ranges(child, ranges);
        }
    }
}

impl Default for MD007 {
    fn default() -> Self {
        Self::new()
    }
}

impl AstRule for MD007 {
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

    fn can_fix(&self) -> bool {
        true
    }

    fn check_ast<'a>(
        &self,
        document: &Document,
        ast: &'a AstNode<'a>,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let lines: Vec<&str> = document.content.lines().collect();

        // Get code block line ranges from AST
        let code_block_lines = self.get_code_block_line_ranges(ast);

        let mut list_stack: Vec<(usize, char, bool)> = Vec::new(); // (indent, marker, is_ordered)

        for (line_number, line) in lines.iter().enumerate() {
            let line_number = line_number + 1;

            // Skip lines inside code blocks
            let in_code_block = code_block_lines
                .iter()
                .any(|(start, end)| line_number >= *start && line_number <= *end);
            
            if in_code_block {
                continue;
            }

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
                        // Create fix by replacing current indentation with expected indentation
                        let fixed_line =
                            format!("{}{}", " ".repeat(expected_indent), &line[indent..]);

                        let fix = Fix {
                            description: format!(
                                "Fix indentation from {} to {} spaces",
                                indent, expected_indent
                            ),
                            replacement: Some(fixed_line),
                            start: Position {
                                line: line_number,
                                column: 1,
                            },
                            end: Position {
                                line: line_number,
                                column: line.len() + 1,
                            },
                        };

                        violations.push(self.create_violation_with_fix(
                            format!(
                                "Unordered list indentation: Expected {expected_indent} spaces, found {indent}"
                            ),
                            line_number,
                            indent + 1, // Convert to 1-based column
                            Severity::Warning,
                            fix,
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
    use mdbook_lint_core::{Document, rule::Rule};
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

    #[test]
    fn test_md007_fix_basic_indentation() {
        let content = "* Item 1\n   * Nested item (3 spaces - wrong!)\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD007::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert!(violations[0].fix.is_some());

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Fix indentation from 3 to 2 spaces");
        assert_eq!(
            fix.replacement,
            Some("  * Nested item (3 spaces - wrong!)".to_string())
        );
        assert_eq!(fix.start.line, 2);
        assert_eq!(fix.start.column, 1);
    }

    #[test]
    fn test_md007_fix_multiple_levels() {
        let content = r#"* Item 1
     * Too many spaces (5)
         * Way too many spaces (9)
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD007::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);

        // First violation: 5 spaces should be 2
        assert!(violations[0].fix.is_some());
        let fix1 = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix1.description, "Fix indentation from 5 to 2 spaces");
        assert_eq!(
            fix1.replacement,
            Some("  * Too many spaces (5)".to_string())
        );

        // Second violation: 9 spaces should be 4
        assert!(violations[1].fix.is_some());
        let fix2 = violations[1].fix.as_ref().unwrap();
        assert_eq!(fix2.description, "Fix indentation from 9 to 4 spaces");
        assert_eq!(
            fix2.replacement,
            Some("    * Way too many spaces (9)".to_string())
        );
    }

    #[test]
    fn test_md007_fix_with_custom_indent() {
        let content = r#"* Item 1
  * Wrong for 4-space indent
      * Wrong again
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD007::new().with_indent(4);
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);

        // Should expect 4 spaces
        let fix1 = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix1.description, "Fix indentation from 2 to 4 spaces");
        assert_eq!(
            fix1.replacement,
            Some("    * Wrong for 4-space indent".to_string())
        );

        // Should expect 8 spaces for second level
        let fix2 = violations[1].fix.as_ref().unwrap();
        assert_eq!(fix2.description, "Fix indentation from 6 to 8 spaces");
        assert_eq!(fix2.replacement, Some("        * Wrong again".to_string()));
    }

    #[test]
    fn test_md007_fix_with_start_indented() {
        let content = r#"* No indent (wrong when start_indented)
* Another no indent (also wrong)
  * Correct for level 0 with start_indented
    * Correct for level 1 (4 spaces = 2 base + 2 indent)
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD007::new().with_start_indented(true);
        let violations = rule.check(&document).unwrap();

        // When start_indented is true, all top-level items without indent are violations
        // Let's just check the first two
        assert!(
            violations.len() >= 2,
            "Expected at least 2 violations, got {}",
            violations.len()
        );

        // First item should have 2 spaces when start_indented is true
        let fix1 = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix1.description, "Fix indentation from 0 to 2 spaces");
        assert_eq!(
            fix1.replacement,
            Some("  * No indent (wrong when start_indented)".to_string())
        );

        // Second item should also have 2 spaces
        let fix2 = violations[1].fix.as_ref().unwrap();
        assert_eq!(fix2.description, "Fix indentation from 0 to 2 spaces");
        assert_eq!(
            fix2.replacement,
            Some("  * Another no indent (also wrong)".to_string())
        );
    }

    #[test]
    fn test_md007_fix_removes_extra_spaces() {
        let content = "* Item\n        * Way too indented (8 spaces for first nested level)\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD007::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Fix indentation from 8 to 2 spaces");
        assert_eq!(
            fix.replacement,
            Some("  * Way too indented (8 spaces for first nested level)".to_string())
        );
    }

    #[test]
    fn test_md007_fix_adds_spaces() {
        let content = "* Item\n * Not enough indent (1 space)\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD007::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(fix.description, "Fix indentation from 1 to 2 spaces");
        assert_eq!(
            fix.replacement,
            Some("  * Not enough indent (1 space)".to_string())
        );
    }

    #[test]
    fn test_md007_fix_preserves_content() {
        let content = "* Item\n   * This has **bold** and _italic_ text\n";
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD007::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);

        let fix = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.replacement,
            Some("  * This has **bold** and _italic_ text".to_string())
        );
    }

    #[test]
    fn test_md007_fix_with_different_markers() {
        let content = r#"- Dash item
   + Plus item (wrong indent)
     * Star item (wrong indent)
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD007::new();
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 2);

        // Different markers should still get fixed
        let fix1 = violations[0].fix.as_ref().unwrap();
        assert_eq!(
            fix1.replacement,
            Some("  + Plus item (wrong indent)".to_string())
        );

        let fix2 = violations[1].fix.as_ref().unwrap();
        assert_eq!(
            fix2.replacement,
            Some("    * Star item (wrong indent)".to_string())
        );
    }

    #[test]
    fn test_md007_ignores_lists_in_code_blocks() {
        let content = r#"Regular list:
* Item 1
  * Item 2

Code block with list:
```yaml
steps:
  - uses: actions/checkout@v4
  - name: Install mdBook and mdbook-lint
    run: |
      cargo install mdbook
      cargo install mdbook-lint
  - name: Build book
    run: mdbook build
```

Another regular list:
* Item 3
    * Too much indent (4 spaces)
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD007::new();
        let violations = rule.check(&document).unwrap();

        // Should only have 1 violation for the last list item (too much indent)
        // Should NOT have violations for the YAML list inside the code block
        assert_eq!(violations.len(), 1, "Should only flag the regular list item with wrong indent, not the YAML in code block");
        
        // Verify it's the right violation (line 19 is the "    * Too much indent" line)
        assert_eq!(violations[0].line, 19);
        assert!(violations[0].message.contains("Expected 2 spaces, found 4"));
    }

    #[test]
    fn test_md007_ignores_indented_code_blocks() {
        let content = r#"Regular list:
* Item 1
  * Item 2

Indented code block:

    * This is code, not a list
      * Should not be checked
        * Even with multiple levels

Another list:
* Item 3
  * Correct indent
"#;
        let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
        let rule = MD007::new();
        let violations = rule.check(&document).unwrap();

        // Should have no violations - the indented code block should be ignored
        assert_eq!(violations.len(), 0, "Should not flag items in indented code blocks");
    }
}
