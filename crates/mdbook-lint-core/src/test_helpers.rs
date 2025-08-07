//! Test helper utilities for rule testing
//!
//! This module provides common utilities to reduce boilerplate in rule tests.

use crate::error::Result;
use crate::{Document, Violation, rule::Rule};
use std::path::PathBuf;

/// Helper to create a test document from content
pub fn create_test_document(content: &str, filename: &str) -> Document {
    Document::new(content.to_string(), PathBuf::from(filename))
        .expect("Failed to create test document")
}

/// Helper to create a test document with default filename
pub fn create_document(content: &str) -> Document {
    create_test_document(content, "test.md")
}

/// Helper to run a rule on content and return violations
pub fn check_rule<T: Rule>(rule: T, content: &str) -> Result<Vec<Violation>> {
    let document = create_document(content);
    rule.check(&document)
}

/// Helper to assert that a rule has no violations
pub fn assert_no_violations<T: Rule>(rule: T, content: &str) {
    let violations = check_rule(rule, content).expect("Rule check failed");
    assert_eq!(
        violations.len(),
        0,
        "Expected no violations but found: {violations:#?}"
    );
}

/// Helper to assert that a rule has exactly one violation
pub fn assert_single_violation<T: Rule>(rule: T, content: &str) -> Violation {
    let violations = check_rule(rule, content).expect("Rule check failed");
    assert_eq!(
        violations.len(),
        1,
        "Expected exactly one violation but found: {violations:#?}"
    );
    violations.into_iter().next().unwrap()
}

/// Helper to assert that a rule has a specific number of violations
pub fn assert_violation_count<T: Rule>(
    rule: T,
    content: &str,
    expected_count: usize,
) -> Vec<Violation> {
    let violations = check_rule(rule, content).expect("Rule check failed");
    assert_eq!(
        violations.len(),
        expected_count,
        "Expected {} violations but found {}: {:#?}",
        expected_count,
        violations.len(),
        violations
    );
    violations
}

/// Helper to check if violations contain a specific message
pub fn assert_violation_contains_message(violations: &[Violation], message: &str) {
    let found = violations.iter().any(|v| v.message.contains(message));
    assert!(
        found,
        "Expected to find violation containing '{message}' but found: {violations:#?}"
    );
}

/// Helper to assert a violation at a specific line
pub fn assert_violation_at_line(violations: &[Violation], line: usize) {
    let found = violations.iter().any(|v| v.line == line);
    assert!(
        found,
        "Expected to find violation at line {} but found violations at lines: {:?}",
        line,
        violations.iter().map(|v| v.line).collect::<Vec<_>>()
    );
}

/// Helper to assert a violation with specific rule ID
pub fn assert_violation_rule_id(violations: &[Violation], rule_id: &str) {
    let found = violations.iter().any(|v| v.rule_id == rule_id);
    assert!(
        found,
        "Expected to find violation with rule ID '{}' but found rule IDs: {:?}",
        rule_id,
        violations.iter().map(|v| &v.rule_id).collect::<Vec<_>>()
    );
}

/// Helper to assert a violation with specific severity
pub fn assert_violation_severity(violations: &[Violation], severity: crate::violation::Severity) {
    let found = violations.iter().any(|v| v.severity == severity);
    assert!(
        found,
        "Expected to find violation with severity {:?} but found severities: {:?}",
        severity,
        violations.iter().map(|v| v.severity).collect::<Vec<_>>()
    );
}

/// Builder pattern for creating test content with common markdown patterns
pub struct MarkdownBuilder {
    content: Vec<String>,
}

impl MarkdownBuilder {
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
        }
    }

    pub fn heading(mut self, level: usize, text: &str) -> Self {
        let prefix = "#".repeat(level);
        self.content.push(format!("{prefix} {text}"));
        self
    }

    pub fn paragraph(mut self, text: &str) -> Self {
        self.content.push(text.to_string());
        self
    }

    pub fn blank_line(mut self) -> Self {
        self.content.push(String::new());
        self
    }

    pub fn code_block(mut self, language: &str, code: &str) -> Self {
        self.content.push(format!("```{language}"));
        for line in code.lines() {
            self.content.push(line.to_string());
        }
        self.content.push("```".to_string());
        self
    }

    pub fn unordered_list(mut self, items: &[&str]) -> Self {
        for item in items {
            self.content.push(format!("- {item}"));
        }
        self
    }

    pub fn ordered_list(mut self, items: &[&str]) -> Self {
        for (i, item) in items.iter().enumerate() {
            self.content.push(format!("{}. {}", i + 1, item));
        }
        self
    }

    pub fn line(mut self, text: &str) -> Self {
        self.content.push(text.to_string());
        self
    }

    pub fn blockquote(mut self, text: &str) -> Self {
        for line in text.lines() {
            self.content.push(format!("> {line}"));
        }
        self
    }

    pub fn table(mut self, headers: &[&str], rows: &[Vec<&str>]) -> Self {
        // Header row
        let header_line = format!("| {} |", headers.join(" | "));
        self.content.push(header_line);

        // Separator row
        let separator = format!(
            "|{}|",
            headers.iter().map(|_| "---").collect::<Vec<_>>().join("|")
        );
        self.content.push(separator);

        // Data rows
        for row in rows {
            let row_line = format!("| {} |", row.join(" | "));
            self.content.push(row_line);
        }
        self
    }

    pub fn link(mut self, text: &str, url: &str) -> Self {
        self.content.push(format!("[{text}]({url})"));
        self
    }

    pub fn image(mut self, alt_text: &str, url: &str) -> Self {
        self.content.push(format!("![{alt_text}]({url})"));
        self
    }

    pub fn horizontal_rule(mut self) -> Self {
        self.content.push("---".to_string());
        self
    }

    pub fn inline_code(mut self, text: &str, code: &str) -> Self {
        self.content.push(format!("{text} `{code}`"));
        self
    }

    pub fn emphasis(mut self, text: &str) -> Self {
        self.content.push(format!("*{text}*"));
        self
    }

    pub fn strong(mut self, text: &str) -> Self {
        self.content.push(format!("**{text}**"));
        self
    }

    pub fn strikethrough(mut self, text: &str) -> Self {
        self.content.push(format!("~~{text}~~"));
        self
    }

    pub fn footnote_definition(mut self, label: &str, content: &str) -> Self {
        self.content.push(format!("[^{label}]: {content}"));
        self
    }

    pub fn footnote_reference(mut self, text: &str, label: &str) -> Self {
        self.content.push(format!("{text}[^{label}]"));
        self
    }

    pub fn task_list(mut self, items: &[(&str, bool)]) -> Self {
        for (item, checked) in items {
            let checkbox = if *checked { "[x]" } else { "[ ]" };
            self.content.push(format!("- {checkbox} {item}"));
        }
        self
    }

    pub fn nested_list(mut self, items: &[(&str, Option<Vec<&str>>)]) -> Self {
        for (item, sub_items) in items {
            self.content.push(format!("- {item}"));
            if let Some(sub_list) = sub_items {
                for sub_item in sub_list {
                    self.content.push(format!("  - {sub_item}"));
                }
            }
        }
        self
    }

    pub fn definition_list(mut self, definitions: &[(&str, &str)]) -> Self {
        for (term, definition) in definitions {
            self.content.push(term.to_string());
            self.content.push(format!(": {definition}"));
        }
        self
    }

    pub fn math_block(mut self, formula: &str) -> Self {
        self.content.push("$$".to_string());
        self.content.push(formula.to_string());
        self.content.push("$$".to_string());
        self
    }

    pub fn inline_math(mut self, text: &str, formula: &str) -> Self {
        self.content.push(format!("{text} ${formula}$"));
        self
    }

    pub fn build(self) -> String {
        self.content.join("\n")
    }
}

impl Default for MarkdownBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        rule::{Rule, RuleCategory, RuleMetadata},
        violation::Severity,
    };

    // Mock rule for testing
    struct TestRule;

    impl Rule for TestRule {
        fn id(&self) -> &'static str {
            "TEST001"
        }

        fn name(&self) -> &'static str {
            "test-rule"
        }

        fn description(&self) -> &'static str {
            "A test rule for testing helpers"
        }

        fn metadata(&self) -> RuleMetadata {
            RuleMetadata::stable(RuleCategory::Structure)
        }

        fn check_with_ast<'a>(
            &self,
            _document: &Document,
            _ast: Option<&'a comrak::nodes::AstNode<'a>>,
        ) -> Result<Vec<Violation>> {
            Ok(vec![self.create_violation(
                "Test violation".to_string(),
                1,
                1,
                Severity::Warning,
            )])
        }
    }

    #[test]
    fn test_create_document() {
        let doc = create_document("# Test");
        assert_eq!(doc.content, "# Test");
        assert_eq!(doc.path, PathBuf::from("test.md"));
    }

    #[test]
    fn test_check_rule() {
        let rule = TestRule;
        let violations = check_rule(rule, "# Test").unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].message, "Test violation");
    }

    #[test]
    fn test_assert_single_violation() {
        let rule = TestRule;
        let violation = assert_single_violation(rule, "# Test");
        assert_eq!(violation.rule_id, "TEST001");
        assert_eq!(violation.message, "Test violation");
    }

    #[test]
    fn test_assert_violation_contains_message() {
        let violations = vec![Violation {
            rule_id: "TEST001".to_string(),
            rule_name: "test-rule".to_string(),
            message: "This is a test violation".to_string(),
            line: 1,
            column: 1,
            severity: Severity::Warning,
        }];

        assert_violation_contains_message(&violations, "test violation");
    }

    #[test]
    fn test_markdown_builder() {
        let content = MarkdownBuilder::new()
            .heading(1, "Title")
            .blank_line()
            .paragraph("Some text")
            .blank_line()
            .code_block("rust", "fn main() {}")
            .blank_line()
            .unordered_list(&["Item 1", "Item 2"])
            .build();

        let expected = "# Title\n\nSome text\n\n```rust\nfn main() {}\n```\n\n- Item 1\n- Item 2";
        assert_eq!(content, expected);
    }

    #[test]
    fn test_ordered_list_builder() {
        let content = MarkdownBuilder::new()
            .ordered_list(&["First", "Second", "Third"])
            .build();

        let expected = "1. First\n2. Second\n3. Third";
        assert_eq!(content, expected);
    }

    #[test]
    fn test_table_builder() {
        let content = MarkdownBuilder::new()
            .table(
                &["Name", "Age", "City"],
                &[
                    vec!["Alice", "30", "New York"],
                    vec!["Bob", "25", "San Francisco"],
                ],
            )
            .build();

        let expected = "| Name | Age | City |\n|---|---|---|\n| Alice | 30 | New York |\n| Bob | 25 | San Francisco |";
        assert_eq!(content, expected);
    }

    #[test]
    fn test_complex_markdown_builder() {
        let content = MarkdownBuilder::new()
            .heading(1, "Test Document")
            .blank_line()
            .paragraph("This is a test document with various elements.")
            .blank_line()
            .blockquote("This is an important quote that spans\nmultiple lines.")
            .blank_line()
            .task_list(&[("Complete tests", true), ("Write docs", false)])
            .blank_line()
            .link("Visit our site", "https://example.com")
            .blank_line()
            .horizontal_rule()
            .build();

        assert!(content.contains("# Test Document"));
        assert!(content.contains("> This is an important quote"));
        assert!(content.contains("- [x] Complete tests"));
        assert!(content.contains("- [ ] Write docs"));
        assert!(content.contains("[Visit our site](https://example.com)"));
        assert!(content.contains("---"));
    }

    #[test]
    fn test_nested_list_builder() {
        let content = MarkdownBuilder::new()
            .nested_list(&[
                ("Item 1", Some(vec!["Sub-item A", "Sub-item B"])),
                ("Item 2", None),
                ("Item 3", Some(vec!["Sub-item C"])),
            ])
            .build();

        let expected =
            "- Item 1\n  - Sub-item A\n  - Sub-item B\n- Item 2\n- Item 3\n  - Sub-item C";
        assert_eq!(content, expected);
    }

    #[test]
    fn test_assert_violation_count() {
        let rule = TestRule;
        let violations = assert_violation_count(rule, "# Test", 1);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "TEST001");
    }

    #[test]
    fn test_assert_violation_at_line() {
        let violations = vec![
            Violation {
                rule_id: "TEST001".to_string(),
                rule_name: "test-rule".to_string(),
                message: "Test violation".to_string(),
                line: 5,
                column: 1,
                severity: Severity::Warning,
            },
            Violation {
                rule_id: "TEST002".to_string(),
                rule_name: "test-rule-2".to_string(),
                message: "Another test violation".to_string(),
                line: 10,
                column: 1,
                severity: Severity::Error,
            },
        ];

        assert_violation_at_line(&violations, 5);
        assert_violation_at_line(&violations, 10);
    }

    #[test]
    fn test_assert_violation_rule_id() {
        let violations = vec![
            Violation {
                rule_id: "MD001".to_string(),
                rule_name: "heading-increment".to_string(),
                message: "Test violation".to_string(),
                line: 1,
                column: 1,
                severity: Severity::Warning,
            },
            Violation {
                rule_id: "MD013".to_string(),
                rule_name: "line-length".to_string(),
                message: "Line too long".to_string(),
                line: 2,
                column: 1,
                severity: Severity::Error,
            },
        ];

        assert_violation_rule_id(&violations, "MD001");
        assert_violation_rule_id(&violations, "MD013");
    }

    #[test]
    fn test_assert_violation_severity() {
        let violations = vec![
            Violation {
                rule_id: "TEST001".to_string(),
                rule_name: "test-rule".to_string(),
                message: "Warning violation".to_string(),
                line: 1,
                column: 1,
                severity: Severity::Warning,
            },
            Violation {
                rule_id: "TEST002".to_string(),
                rule_name: "test-rule-2".to_string(),
                message: "Error violation".to_string(),
                line: 2,
                column: 1,
                severity: Severity::Error,
            },
        ];

        assert_violation_severity(&violations, Severity::Warning);
        assert_violation_severity(&violations, Severity::Error);
    }

    #[test]
    fn test_markdown_builder_all_methods() {
        let content = MarkdownBuilder::new()
            .heading(1, "Main Title")
            .blank_line()
            .paragraph("Introduction paragraph")
            .blank_line()
            .heading(2, "Section")
            .code_block("rust", "fn main() {\n    println!(\"Hello\");\n}")
            .blank_line()
            .unordered_list(&["First item", "Second item", "Third item"])
            .blank_line()
            .ordered_list(&["Step 1", "Step 2", "Step 3"])
            .blank_line()
            .line("Custom line of text")
            .blockquote("Important quote\nSpanning multiple lines")
            .blank_line()
            .link("Example", "https://example.com")
            .blank_line()
            .image("Alt text", "image.png")
            .blank_line()
            .horizontal_rule()
            .blank_line()
            .inline_code("Here is", "some_code")
            .blank_line()
            .emphasis("emphasized text")
            .blank_line()
            .strong("strong text")
            .blank_line()
            .strikethrough("crossed out")
            .blank_line()
            .footnote_definition("note1", "This is a footnote")
            .footnote_reference("Text with footnote", "note1")
            .blank_line()
            .task_list(&[("Completed task", true), ("Pending task", false)])
            .blank_line()
            .definition_list(&[("Term 1", "Definition 1"), ("Term 2", "Definition 2")])
            .blank_line()
            .math_block("x = y + z")
            .blank_line()
            .inline_math("The equation", "E = mc^2")
            .build();

        // Verify various components are present
        assert!(content.contains("# Main Title"));
        assert!(content.contains("Introduction paragraph"));
        assert!(content.contains("```rust"));
        assert!(content.contains("- First item"));
        assert!(content.contains("1. Step 1"));
        assert!(content.contains("Custom line of text"));
        assert!(content.contains("> Important quote"));
        assert!(content.contains("[Example](https://example.com)"));
        assert!(content.contains("![Alt text](image.png)"));
        assert!(content.contains("---"));
        assert!(content.contains("Here is `some_code`"));
        assert!(content.contains("*emphasized text*"));
        assert!(content.contains("**strong text**"));
        assert!(content.contains("~~crossed out~~"));
        assert!(content.contains("[^note1]: This is a footnote"));
        assert!(content.contains("Text with footnote[^note1]"));
        assert!(content.contains("- [x] Completed task"));
        assert!(content.contains("- [ ] Pending task"));
        assert!(content.contains("Term 1"));
        assert!(content.contains(": Definition 1"));
        assert!(content.contains("$$"));
        assert!(content.contains("$E = mc^2$"));
    }

    #[test]
    fn test_markdown_builder_default() {
        let builder = MarkdownBuilder::default();
        let content = builder.heading(1, "Test").build();
        assert_eq!(content, "# Test");
    }

    #[test]
    fn test_create_test_document_with_filename() {
        let doc = create_test_document("# Content", "custom.md");
        assert_eq!(doc.content, "# Content");
        assert_eq!(doc.path, PathBuf::from("custom.md"));
    }

    #[test]
    fn test_all_markdown_builder_edge_cases() {
        // Test empty lists
        let content = MarkdownBuilder::new()
            .unordered_list(&[])
            .ordered_list(&[])
            .build();
        assert_eq!(content, "");

        // Test single items
        let content = MarkdownBuilder::new()
            .unordered_list(&["Single"])
            .blank_line()
            .ordered_list(&["One"])
            .build();
        assert_eq!(content, "- Single\n\n1. One");

        // Test nested list with empty sub-items
        let content = MarkdownBuilder::new()
            .nested_list(&[("Item", None)])
            .build();
        assert_eq!(content, "- Item");

        // Test table with empty rows
        let content = MarkdownBuilder::new().table(&["Header"], &[]).build();
        assert_eq!(content, "| Header |\n|---|");

        // Test definition list edge cases
        let content = MarkdownBuilder::new().definition_list(&[]).build();
        assert_eq!(content, "");
    }

    // Error path testing for test helpers - targeting uncovered assertion failures

    // Mock rule that produces no violations for testing assert_no_violations error path
    struct NoViolationRule;
    impl Rule for NoViolationRule {
        fn id(&self) -> &'static str {
            "NO_VIO"
        }
        fn name(&self) -> &'static str {
            "no-violation"
        }
        fn description(&self) -> &'static str {
            "Never produces violations"
        }
        fn metadata(&self) -> RuleMetadata {
            RuleMetadata::stable(RuleCategory::Structure)
        }
        fn check_with_ast<'a>(
            &self,
            _document: &Document,
            _ast: Option<&'a comrak::nodes::AstNode<'a>>,
        ) -> Result<Vec<Violation>> {
            Ok(vec![])
        }
    }

    // Mock rule that produces multiple violations for testing single violation error path
    struct MultiViolationRule;
    impl Rule for MultiViolationRule {
        fn id(&self) -> &'static str {
            "MULTI"
        }
        fn name(&self) -> &'static str {
            "multi-violation"
        }
        fn description(&self) -> &'static str {
            "Produces multiple violations"
        }
        fn metadata(&self) -> RuleMetadata {
            RuleMetadata::stable(RuleCategory::Structure)
        }
        fn check_with_ast<'a>(
            &self,
            _document: &Document,
            _ast: Option<&'a comrak::nodes::AstNode<'a>>,
        ) -> Result<Vec<Violation>> {
            Ok(vec![
                self.create_violation("First violation".to_string(), 1, 1, Severity::Warning),
                self.create_violation("Second violation".to_string(), 2, 1, Severity::Error),
            ])
        }
    }

    #[test]
    #[should_panic(expected = "Expected no violations but found")]
    fn test_assert_no_violations_error_path() {
        // This should panic because TestRule produces violations
        assert_no_violations(TestRule, "# Test content");
    }

    #[test]
    #[should_panic(expected = "Expected exactly one violation but found")]
    fn test_assert_single_violation_multiple_violations_error() {
        // This should panic because MultiViolationRule produces 2 violations
        assert_single_violation(MultiViolationRule, "# Test content");
    }

    #[test]
    #[should_panic(expected = "Expected exactly one violation but found")]
    fn test_assert_single_violation_no_violations_error() {
        // This should panic because NoViolationRule produces 0 violations
        assert_single_violation(NoViolationRule, "# Test content");
    }

    #[test]
    #[should_panic(expected = "Expected 3 violations but found")]
    fn test_assert_violation_count_wrong_count_error() {
        // This should panic because TestRule only produces 1 violation, not 3
        assert_violation_count(TestRule, "# Test content", 3);
    }

    #[test]
    #[should_panic(expected = "Expected to find violation containing 'nonexistent message'")]
    fn test_assert_violation_contains_message_not_found() {
        let violations = vec![Violation {
            rule_id: "TEST".to_string(),
            rule_name: "test".to_string(),
            message: "Test violation".to_string(),
            line: 1,
            column: 1,
            severity: Severity::Warning,
        }];
        assert_violation_contains_message(&violations, "nonexistent message");
    }

    #[test]
    #[should_panic(expected = "Expected to find violation at line 999")]
    fn test_assert_violation_at_line_not_found() {
        let violations = vec![Violation {
            rule_id: "TEST".to_string(),
            rule_name: "test".to_string(),
            message: "Test violation".to_string(),
            line: 1,
            column: 1,
            severity: Severity::Warning,
        }];
        assert_violation_at_line(&violations, 999);
    }

    #[test]
    #[should_panic(expected = "Expected to find violation with rule ID 'NONEXISTENT'")]
    fn test_assert_violation_rule_id_not_found() {
        let violations = vec![Violation {
            rule_id: "TEST".to_string(),
            rule_name: "test".to_string(),
            message: "Test violation".to_string(),
            line: 1,
            column: 1,
            severity: Severity::Warning,
        }];
        assert_violation_rule_id(&violations, "NONEXISTENT");
    }

    #[test]
    #[should_panic(expected = "Expected to find violation with severity")]
    fn test_assert_violation_severity_not_found() {
        let violations = vec![Violation {
            rule_id: "TEST".to_string(),
            rule_name: "test".to_string(),
            message: "Test violation".to_string(),
            line: 1,
            column: 1,
            severity: Severity::Warning,
        }];
        assert_violation_severity(&violations, Severity::Error);
    }

    #[test]
    fn test_successful_helper_paths() {
        // Test successful paths to ensure they work correctly
        assert_no_violations(NoViolationRule, "# Test content");

        let violation = assert_single_violation(TestRule, "# Test content");
        assert_eq!(violation.message, "Test violation");

        let violations = assert_violation_count(MultiViolationRule, "# Test content", 2);
        assert_eq!(violations.len(), 2);

        // Test successful assertion helpers
        let test_violations = vec![Violation {
            rule_id: "TEST123".to_string(),
            rule_name: "test".to_string(),
            message: "Contains specific text".to_string(),
            line: 42,
            column: 1,
            severity: Severity::Error,
        }];

        assert_violation_contains_message(&test_violations, "specific text");
        assert_violation_at_line(&test_violations, 42);
        assert_violation_rule_id(&test_violations, "TEST123");
        assert_violation_severity(&test_violations, Severity::Error);
    }
}
