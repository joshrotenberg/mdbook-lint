//! MD041: First line in file should be a top level heading
//!
//! This rule checks that the first line of the file is a top-level heading (H1).

use crate::error::Result;
use crate::rule::{Rule, RuleCategory, RuleMetadata};
use crate::{
    Document,
    violation::{Severity, Violation},
};

/// Rule to check that the first line is a top-level heading
pub struct MD041;

impl MD041 {
    /// Check if a line is a top-level heading (H1)
    fn is_top_level_heading(&self, line: &str) -> bool {
        let trimmed = line.trim();

        // ATX style: # Heading
        if trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
            return true;
        }

        // Also accept just # without space if there's content after
        if trimmed.starts_with('#') && !trimmed.starts_with("##") && trimmed.len() > 1 {
            return true;
        }

        false
    }

    /// Check if a line is a setext-style H1 (underlined with =)
    fn is_setext_h1_underline(&self, line: &str) -> bool {
        let trimmed = line.trim();
        !trimmed.is_empty() && trimmed.chars().all(|c| c == '=')
    }

    /// Check if a line is content that could be a setext heading
    fn could_be_setext_heading(&self, line: &str) -> bool {
        let trimmed = line.trim();
        !trimmed.is_empty() && !trimmed.starts_with('#')
    }
}

impl Rule for MD041 {
    fn id(&self) -> &'static str {
        "MD041"
    }

    fn name(&self) -> &'static str {
        "first-line-heading"
    }

    fn description(&self) -> &'static str {
        "First line in file should be a top level heading"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Structure).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        if document.lines.is_empty() {
            return Ok(violations);
        }

        // Find the first non-empty line
        let mut first_content_line_idx = None;
        for (idx, line) in document.lines.iter().enumerate() {
            if !line.trim().is_empty() {
                first_content_line_idx = Some(idx);
                break;
            }
        }

        let Some(first_idx) = first_content_line_idx else {
            // File is empty or only whitespace
            return Ok(violations);
        };

        let first_line = &document.lines[first_idx];

        // Check if first line is an ATX H1
        if self.is_top_level_heading(first_line) {
            return Ok(violations);
        }

        // Check for setext-style H1 (current line + next line with =)
        if first_idx + 1 < document.lines.len() {
            let second_line = &document.lines[first_idx + 1];
            if self.could_be_setext_heading(first_line) && self.is_setext_h1_underline(second_line)
            {
                return Ok(violations);
            }
        }

        // If we get here, the first line is not a top-level heading
        violations.push(self.create_violation(
            "First line in file should be a top level heading".to_string(),
            first_idx + 1, // Convert to 1-based line number
            1,
            Severity::Warning,
        ));

        Ok(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::Rule;
    use std::path::PathBuf;

    fn create_test_document(content: &str) -> Document {
        Document::new(content.to_string(), PathBuf::from("test.md")).unwrap()
    }

    #[test]
    fn test_md041_atx_h1_valid() {
        let content = "# Top Level Heading\n\nSome content here.";
        let document = create_test_document(content);
        let rule = MD041;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md041_atx_h1_no_space_valid() {
        let content = "#Top Level Heading\n\nSome content here.";
        let document = create_test_document(content);
        let rule = MD041;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md041_setext_h1_valid() {
        let content = "Top Level Heading\n=================\n\nSome content here.";
        let document = create_test_document(content);
        let rule = MD041;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md041_h2_invalid() {
        let content = "## Second Level Heading\n\nSome content here.";
        let document = create_test_document(content);
        let rule = MD041;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MD041");
        assert_eq!(violations[0].line, 1);
        assert!(
            violations[0]
                .message
                .contains("First line in file should be a top level heading")
        );
    }

    #[test]
    fn test_md041_paragraph_first_invalid() {
        let content = "This is a paragraph.\n\n# Heading comes later";
        let document = create_test_document(content);
        let rule = MD041;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 1);
    }

    #[test]
    fn test_md041_setext_h2_invalid() {
        let content = "Second Level Heading\n--------------------\n\nSome content here.";
        let document = create_test_document(content);
        let rule = MD041;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 1);
    }

    #[test]
    fn test_md041_empty_file_valid() {
        let content = "";
        let document = create_test_document(content);
        let rule = MD041;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md041_whitespace_only_valid() {
        let content = "   \n\n\t\n   ";
        let document = create_test_document(content);
        let rule = MD041;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md041_leading_whitespace_valid() {
        let content = "\n\n# Top Level Heading\n\nSome content here.";
        let document = create_test_document(content);
        let rule = MD041;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md041_leading_whitespace_invalid() {
        let content = "\n\nSome paragraph first.\n\n# Heading later";
        let document = create_test_document(content);
        let rule = MD041;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3); // Line with "Some paragraph first."
    }

    #[test]
    fn test_md041_bare_hash_invalid() {
        let content = "#\n\nSome content here.";
        let document = create_test_document(content);
        let rule = MD041;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 1);
    }

    #[test]
    fn test_md041_code_block_first_invalid() {
        let content = "```\ncode block\n```\n\n# Heading later";
        let document = create_test_document(content);
        let rule = MD041;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 1);
    }

    #[test]
    fn test_md041_list_first_invalid() {
        let content = "- List item\n- Another item\n\n# Heading later";
        let document = create_test_document(content);
        let rule = MD041;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 1);
    }

    #[test]
    fn test_md041_setext_incomplete_invalid() {
        let content = "Potential heading\n\nBut no underline.";
        let document = create_test_document(content);
        let rule = MD041;
        let violations = rule.check(&document).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 1);
    }
}
