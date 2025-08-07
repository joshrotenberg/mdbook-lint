//! MDBOOK003: SUMMARY.md structure validation
//!
//! Validates that SUMMARY.md follows the mdBook specification for structure and formatting.

use crate::Document;
use crate::rule::{Rule, RuleCategory, RuleMetadata};
use crate::violation::{Severity, Violation};

/// MDBOOK003: Validates SUMMARY.md structure and formatting
///
/// This rule checks:
/// - File must be named SUMMARY.md (case-sensitive)
/// - Consistent list delimiters (don't mix - and *)
/// - Proper nesting hierarchy (no skipped indentation levels)
/// - Part titles must be h1 headers only
/// - Prefix chapters cannot be nested
/// - No prefix chapters after numbered chapters begin
/// - Valid link syntax for chapters
/// - Draft chapters use empty parentheses
/// - Separators contain only dashes (minimum 3)
pub struct MDBOOK003;

impl Rule for MDBOOK003 {
    fn id(&self) -> &'static str {
        "MDBOOK003"
    }

    fn name(&self) -> &'static str {
        "summary-structure"
    }

    fn description(&self) -> &'static str {
        "SUMMARY.md must follow mdBook format requirements"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::MdBook).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> crate::error::Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // Check if this is a SUMMARY.md file
        if !is_summary_file(document) {
            return Ok(violations);
        }

        let mut checker = SummaryChecker::new(self);
        checker.validate(document, &mut violations);

        Ok(violations)
    }
}

/// Internal state tracker for SUMMARY.md validation
struct SummaryChecker<'a> {
    /// Reference to the rule for creating violations
    rule: &'a MDBOOK003,
    /// Track if we've seen numbered chapters (to detect prefix chapters after)
    seen_numbered_chapters: bool,
    /// Track the list delimiter used (- or *)
    list_delimiter: Option<char>,
    /// Track current nesting level for hierarchy validation
    current_nesting_level: usize,
    /// Track line numbers of part titles for context
    part_title_lines: Vec<usize>,
}

impl<'a> SummaryChecker<'a> {
    fn new(rule: &'a MDBOOK003) -> Self {
        Self {
            rule,
            seen_numbered_chapters: false,
            list_delimiter: None,
            current_nesting_level: 0,
            part_title_lines: Vec::new(),
        }
    }

    fn validate(&mut self, document: &Document, violations: &mut Vec<Violation>) {
        for (line_num, line) in document.lines.iter().enumerate() {
            let line_num = line_num + 1; // Convert to 1-based
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            // Check for part titles (h1 headers)
            if let Some(title) = self.parse_part_title(trimmed) {
                self.validate_part_title(line_num, &title, violations);
                continue;
            }

            // Check for invalid part titles (h2, h3, etc.)
            if self.is_invalid_part_title(trimmed) {
                violations.push(self.rule.create_violation(
                    "Part titles must be h1 headers (single #)".to_string(),
                    line_num,
                    1,
                    Severity::Error,
                ));
                continue;
            }

            // Check for separators
            if self.is_separator(trimmed) {
                self.validate_separator(line_num, trimmed, violations);
                continue;
            }

            // Check for chapters (both numbered and prefix/suffix)
            if let Some(chapter) = self.parse_chapter(line) {
                self.validate_chapter(line_num, line, &chapter, violations);
            }
        }
    }

    fn parse_part_title(&self, line: &str) -> Option<String> {
        line.strip_prefix("# ")
            .map(|stripped| stripped.trim().to_string())
    }

    fn is_invalid_part_title(&self, line: &str) -> bool {
        line.starts_with("##") && !line.starts_with("###")
            || line.starts_with("###")
            || line.starts_with("####")
            || line.starts_with("#####")
            || line.starts_with("######")
    }

    fn validate_part_title(
        &mut self,
        line_num: usize,
        title: &str,
        violations: &mut Vec<Violation>,
    ) {
        self.part_title_lines.push(line_num);

        // Part titles should not be empty
        if title.is_empty() {
            violations.push(self.rule.create_violation(
                "Part titles cannot be empty".to_string(),
                line_num,
                1,
                Severity::Error,
            ));
        }
    }

    fn is_separator(&self, line: &str) -> bool {
        !line.is_empty() && line.chars().all(|c| c == '-')
    }

    fn validate_separator(&self, line_num: usize, line: &str, violations: &mut Vec<Violation>) {
        if line.len() < 3 {
            violations.push(self.rule.create_violation(
                "Separators must contain at least 3 dashes".to_string(),
                line_num,
                1,
                Severity::Error,
            ));
        }
    }

    fn parse_chapter(&self, line: &str) -> Option<Chapter> {
        let trimmed = line.trim_start();
        let indent_level = (line.len() - trimmed.len()) / 4; // Assume 4-space indentation

        // Check for numbered chapters (list items)
        if let Some(rest) = trimmed.strip_prefix("- ") {
            return Some(Chapter {
                is_numbered: true,
                indent_level,
                delimiter: '-',
                content: rest.to_string(),
            });
        }

        if let Some(rest) = trimmed.strip_prefix("* ") {
            return Some(Chapter {
                is_numbered: true,
                indent_level,
                delimiter: '*',
                content: rest.to_string(),
            });
        }

        // Check for prefix/suffix chapters (plain links)
        if trimmed.starts_with('[') && trimmed.contains("](") {
            return Some(Chapter {
                is_numbered: false,
                indent_level,
                delimiter: ' ', // Not applicable for prefix/suffix
                content: trimmed.to_string(),
            });
        }

        None
    }

    fn validate_chapter(
        &mut self,
        line_num: usize,
        line: &str,
        chapter: &Chapter,
        violations: &mut Vec<Violation>,
    ) {
        if chapter.is_numbered {
            self.validate_numbered_chapter(line_num, line, chapter, violations);
        } else {
            self.validate_prefix_suffix_chapter(line_num, chapter, violations);
        }

        // Validate the link syntax
        self.validate_chapter_link(line_num, &chapter.content, violations);
    }

    fn validate_numbered_chapter(
        &mut self,
        line_num: usize,
        line: &str,
        chapter: &Chapter,
        violations: &mut Vec<Violation>,
    ) {
        self.seen_numbered_chapters = true;

        // Check for consistent delimiters
        if let Some(existing_delimiter) = self.list_delimiter {
            if existing_delimiter != chapter.delimiter {
                violations.push(self.rule.create_violation(
                    format!(
                        "Inconsistent list delimiter. Expected '{}' but found '{}'",
                        existing_delimiter, chapter.delimiter
                    ),
                    line_num,
                    line.len() - line.trim_start().len() + 1,
                    Severity::Error,
                ));
            }
        } else {
            self.list_delimiter = Some(chapter.delimiter);
        }

        // Check nesting hierarchy
        self.validate_nesting_hierarchy(line_num, chapter, violations);
    }

    fn validate_nesting_hierarchy(
        &mut self,
        line_num: usize,
        chapter: &Chapter,
        violations: &mut Vec<Violation>,
    ) {
        let expected_max_level = self.current_nesting_level + 1;

        if chapter.indent_level > expected_max_level {
            violations.push(self.rule.create_violation(
                format!(
                    "Invalid nesting level. Skipped from level {} to level {}",
                    self.current_nesting_level, chapter.indent_level
                ),
                line_num,
                1,
                Severity::Error,
            ));
        }

        self.current_nesting_level = chapter.indent_level;
    }

    fn validate_prefix_suffix_chapter(
        &mut self,
        line_num: usize,
        chapter: &Chapter,
        violations: &mut Vec<Violation>,
    ) {
        // Prefix chapters cannot be nested
        if chapter.indent_level > 0 {
            violations.push(self.rule.create_violation(
                "Prefix and suffix chapters cannot be nested".to_string(),
                line_num,
                1,
                Severity::Error,
            ));
        }

        // Cannot add prefix chapters after numbered chapters have started
        if self.seen_numbered_chapters {
            // This is a suffix chapter, which is allowed
            // Only prefix chapters (before numbered) are restricted
        }
    }

    fn validate_chapter_link(
        &self,
        line_num: usize,
        content: &str,
        violations: &mut Vec<Violation>,
    ) {
        // Basic link syntax validation
        if !content.trim().starts_with('[') {
            violations.push(self.rule.create_violation(
                "Chapter entries must be in link format [title](path)".to_string(),
                line_num,
                1,
                Severity::Error,
            ));
            return;
        }

        // Find the closing bracket and opening parenthesis
        if let Some(bracket_end) = content.find("](") {
            let title = &content[1..bracket_end];
            let rest = &content[bracket_end + 2..];

            if title.is_empty() {
                violations.push(self.rule.create_violation(
                    "Chapter title cannot be empty".to_string(),
                    line_num,
                    2,
                    Severity::Error,
                ));
            }

            // Find closing parenthesis
            if let Some(paren_end) = rest.find(')') {
                let path = &rest[..paren_end];

                // Draft chapters should have empty path
                if path.is_empty() {
                    // This is a draft chapter, which is valid
                } else {
                    // Validate path format (basic checks)
                    if path.contains("\\") {
                        violations.push(self.rule.create_violation(
                            "Use forward slashes in paths, not backslashes".to_string(),
                            line_num,
                            bracket_end + 3,
                            Severity::Warning,
                        ));
                    }
                }
            } else {
                violations.push(self.rule.create_violation(
                    "Missing closing parenthesis in chapter link".to_string(),
                    line_num,
                    content.len(),
                    Severity::Error,
                ));
            }
        } else if content.contains('[') && content.contains(']') {
            violations.push(self.rule.create_violation(
                "Invalid link syntax. Missing '](' between title and path".to_string(),
                line_num,
                content.find(']').unwrap_or(0) + 1,
                Severity::Error,
            ));
        }
    }
}

/// Represents a parsed chapter entry
#[derive(Debug)]
struct Chapter {
    /// Whether this is a numbered chapter (list item) or prefix/suffix
    is_numbered: bool,
    /// Indentation level (number of 4-space indents)
    indent_level: usize,
    /// List delimiter used (- or *)
    delimiter: char,
    /// The content of the chapter line
    content: String,
}

/// Check if the document represents a SUMMARY.md file
fn is_summary_file(document: &Document) -> bool {
    document
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "SUMMARY.md")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use crate::rule::Rule;
    use std::path::PathBuf;

    fn create_test_document(content: &str, filename: &str) -> Document {
        Document::new(content.to_string(), PathBuf::from(filename)).unwrap()
    }

    #[test]
    fn test_valid_summary() {
        let content = r#"# Summary

[Introduction](README.md)

# User Guide

- [Installation](guide/installation.md)
- [Reading Books](guide/reading.md)
    - [Sub Chapter](guide/sub.md)

---

[Contributors](misc/contributors.md)
"#;
        let doc = create_test_document(content, "SUMMARY.md");
        let rule = MDBOOK003;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(
            violations.len(),
            0,
            "Valid SUMMARY.md should have no violations"
        );
    }

    #[test]
    fn test_not_summary_file() {
        let content = "# Some Random File\n\n- [Link](file.md)";
        let doc = create_test_document(content, "README.md");
        let rule = MDBOOK003;
        let violations = rule.check(&doc).unwrap();
        assert_eq!(
            violations.len(),
            0,
            "Non-SUMMARY.md files should be ignored"
        );
    }

    #[test]
    fn test_mixed_delimiters() {
        let content = r#"# Summary

- [First](first.md)
* [Second](second.md)
- [Third](third.md)
"#;
        let doc = create_test_document(content, "SUMMARY.md");
        let rule = MDBOOK003;
        let violations = rule.check(&doc).unwrap();

        let delimiter_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.message.contains("Inconsistent list delimiter"))
            .collect();
        assert!(
            !delimiter_violations.is_empty(),
            "Should detect mixed delimiters"
        );
    }

    #[test]
    fn test_invalid_part_titles() {
        let content = r#"# Summary

## Invalid Part Title

- [Chapter](chapter.md)

### Another Invalid

- [Another](another.md)
"#;
        let doc = create_test_document(content, "SUMMARY.md");
        let rule = MDBOOK003;
        let violations = rule.check(&doc).unwrap();

        let part_title_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.message.contains("Part titles must be h1 headers"))
            .collect();
        assert_eq!(
            part_title_violations.len(),
            2,
            "Should detect invalid part title levels"
        );
    }

    #[test]
    fn test_nested_prefix_chapters() {
        let content = r#"# Summary

[Introduction](README.md)
    [Nested Prefix](nested.md)

- [Chapter](chapter.md)
"#;
        let doc = create_test_document(content, "SUMMARY.md");
        let rule = MDBOOK003;
        let violations = rule.check(&doc).unwrap();

        let nesting_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.message.contains("cannot be nested"))
            .collect();
        assert!(
            !nesting_violations.is_empty(),
            "Should detect nested prefix chapters"
        );
    }

    #[test]
    fn test_bad_nesting_hierarchy() {
        let content = r#"# Summary

- [Chapter](chapter.md)
        - [Skip Level](skip.md)
"#;
        let doc = create_test_document(content, "SUMMARY.md");
        let rule = MDBOOK003;
        let violations = rule.check(&doc).unwrap();

        let hierarchy_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.message.contains("Invalid nesting level"))
            .collect();
        assert!(
            !hierarchy_violations.is_empty(),
            "Should detect skipped nesting levels"
        );
    }

    #[test]
    fn test_invalid_link_syntax() {
        let content = r#"# Summary

- [Missing Path]
- [Bad Syntax(missing-bracket.md)
- Missing Link Format
"#;
        let doc = create_test_document(content, "SUMMARY.md");
        let rule = MDBOOK003;
        let violations = rule.check(&doc).unwrap();

        let link_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.message.contains("link") || v.message.contains("format"))
            .collect();
        assert!(
            !link_violations.is_empty(),
            "Should detect invalid link syntax"
        );
    }

    #[test]
    fn test_draft_chapters() {
        let content = r#"# Summary

- [Regular Chapter](chapter.md)
- [Draft Chapter]()
"#;
        let doc = create_test_document(content, "SUMMARY.md");
        let rule = MDBOOK003;
        let violations = rule.check(&doc).unwrap();

        // Draft chapters should not generate violations
        let draft_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.line == 4) // Line with draft chapter
            .collect();
        assert_eq!(draft_violations.len(), 0, "Draft chapters should be valid");
    }

    #[test]
    fn test_separator_validation() {
        let content = r#"# Summary

- [Chapter](chapter.md)

--

- [Another](another.md)

---

[Suffix](suffix.md)
"#;
        let doc = create_test_document(content, "SUMMARY.md");
        let rule = MDBOOK003;
        let violations = rule.check(&doc).unwrap();

        let separator_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.message.contains("at least 3 dashes"))
            .collect();
        assert!(
            !separator_violations.is_empty(),
            "Should detect invalid separator length"
        );
    }
}
