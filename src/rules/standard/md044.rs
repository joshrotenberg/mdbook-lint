//! MD044: Proper names should have correct capitalization
//!
//! This rule checks that proper names (like company names, product names, etc.)
//! are capitalized correctly throughout the document.

use crate::error::Result;
use crate::rule::{Rule, RuleCategory, RuleMetadata};
use crate::{
    Document,
    violation::{Severity, Violation},
};
use std::collections::HashMap;

/// Rule to check proper name capitalization
pub struct MD044 {
    /// Map of lowercase names to their correct capitalization
    proper_names: HashMap<String, String>,
}

impl MD044 {
    /// Create a new MD044 rule with default proper names
    pub fn new() -> Self {
        let mut proper_names = HashMap::new();

        // Add common technology names that are often miscapitalized
        proper_names.insert("javascript".to_string(), "JavaScript".to_string());
        proper_names.insert("typescript".to_string(), "TypeScript".to_string());
        proper_names.insert("github".to_string(), "GitHub".to_string());
        proper_names.insert("gitlab".to_string(), "GitLab".to_string());
        proper_names.insert("bitbucket".to_string(), "Bitbucket".to_string());
        proper_names.insert("nodejs".to_string(), "Node.js".to_string());
        proper_names.insert("mysql".to_string(), "MySQL".to_string());
        proper_names.insert("postgresql".to_string(), "PostgreSQL".to_string());
        proper_names.insert("mongodb".to_string(), "MongoDB".to_string());
        proper_names.insert("redis".to_string(), "Redis".to_string());
        proper_names.insert("docker".to_string(), "Docker".to_string());
        proper_names.insert("kubernetes".to_string(), "Kubernetes".to_string());
        proper_names.insert("aws".to_string(), "AWS".to_string());
        proper_names.insert("azure".to_string(), "Azure".to_string());
        proper_names.insert("google cloud".to_string(), "Google Cloud".to_string());
        proper_names.insert("gcp".to_string(), "GCP".to_string());
        proper_names.insert("react".to_string(), "React".to_string());
        proper_names.insert("vue".to_string(), "Vue".to_string());
        proper_names.insert("angular".to_string(), "Angular".to_string());
        proper_names.insert("webpack".to_string(), "webpack".to_string());
        proper_names.insert("eslint".to_string(), "ESLint".to_string());
        proper_names.insert("prettier".to_string(), "Prettier".to_string());
        proper_names.insert("babel".to_string(), "Babel".to_string());
        proper_names.insert("json".to_string(), "JSON".to_string());
        proper_names.insert("xml".to_string(), "XML".to_string());
        proper_names.insert("html".to_string(), "HTML".to_string());
        proper_names.insert("css".to_string(), "CSS".to_string());
        proper_names.insert("sass".to_string(), "Sass".to_string());
        proper_names.insert("scss".to_string(), "SCSS".to_string());
        proper_names.insert("less".to_string(), "Less".to_string());
        proper_names.insert("api".to_string(), "API".to_string());
        proper_names.insert("rest".to_string(), "REST".to_string());
        proper_names.insert("graphql".to_string(), "GraphQL".to_string());
        proper_names.insert("oauth".to_string(), "OAuth".to_string());
        proper_names.insert("jwt".to_string(), "JWT".to_string());
        proper_names.insert("http".to_string(), "HTTP".to_string());
        proper_names.insert("https".to_string(), "HTTPS".to_string());
        proper_names.insert("tcp".to_string(), "TCP".to_string());
        proper_names.insert("udp".to_string(), "UDP".to_string());
        proper_names.insert("ip".to_string(), "IP".to_string());
        proper_names.insert("dns".to_string(), "DNS".to_string());
        proper_names.insert("url".to_string(), "URL".to_string());
        proper_names.insert("uri".to_string(), "URI".to_string());
        proper_names.insert("uuid".to_string(), "UUID".to_string());

        Self { proper_names }
    }

    /// Create a new MD044 rule with custom proper names
    #[allow(dead_code)]
    pub fn with_names(proper_names: HashMap<String, String>) -> Self {
        Self { proper_names }
    }

    /// Add a proper name to the list
    #[allow(dead_code)]
    pub fn add_name(&mut self, incorrect: String, correct: String) {
        self.proper_names.insert(incorrect.to_lowercase(), correct);
    }

    /// Check a line for proper name violations
    fn check_line_names(&self, line: &str, line_number: usize) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Skip empty lines
        if line.trim().is_empty() {
            return violations;
        }

        // Find all matches and their positions first
        let mut matches = Vec::new();
        let line_lower = line.to_lowercase();

        for (incorrect_lower, correct) in &self.proper_names {
            let mut search_pos = 0;

            while let Some(pos) = line_lower[search_pos..].find(incorrect_lower) {
                let absolute_pos = search_pos + pos;

                // Check if this is a whole word match (not part of another word)
                let is_word_start = absolute_pos == 0
                    || !line_lower
                        .chars()
                        .nth(absolute_pos - 1)
                        .unwrap_or(' ')
                        .is_alphanumeric();
                let end_pos = absolute_pos + incorrect_lower.len();
                let is_word_end = end_pos >= line_lower.len()
                    || !line_lower
                        .chars()
                        .nth(end_pos)
                        .unwrap_or(' ')
                        .is_alphanumeric();

                if is_word_start && is_word_end {
                    // Extract the actual text from the original line to check casing
                    let actual_text = &line[absolute_pos..end_pos];

                    // Only flag if it's not already correctly capitalized
                    if actual_text != correct {
                        // Skip if this appears to be in a code span
                        if !self.is_in_code_span(line, absolute_pos) {
                            matches.push((absolute_pos, actual_text.to_string(), correct.clone()));
                        }
                    }
                }

                search_pos = absolute_pos + 1;
            }
        }

        // Sort matches by position to maintain text order
        matches.sort_by_key(|(pos, _, _)| *pos);

        // Create violations in order
        for (pos, actual_text, correct) in matches {
            violations.push(self.create_violation(
                format!("Proper name '{actual_text}' should be capitalized as '{correct}'"),
                line_number,
                pos + 1, // Convert to 1-based column
                Severity::Warning,
            ));
        }

        violations
    }

    /// Check if a position is inside a code span
    fn is_in_code_span(&self, line: &str, pos: usize) -> bool {
        let chars: Vec<char> = line.chars().collect();
        let mut in_code_span = false;
        let mut i = 0;

        while i < chars.len() && i <= pos {
            if chars[i] == '`' {
                // Count consecutive backticks
                let mut _backtick_count = 0;
                let _start = i;
                while i < chars.len() && chars[i] == '`' {
                    _backtick_count += 1;
                    i += 1;
                }

                if in_code_span {
                    // Check if this closes the code span (same number of backticks)
                    in_code_span = false; // Simplified - just toggle
                } else {
                    in_code_span = true;
                }
            } else {
                i += 1;
            }
        }

        in_code_span
    }

    /// Get code block ranges to exclude from checking
    fn get_code_block_ranges(&self, lines: &[&str]) -> Vec<bool> {
        let mut in_code_block = vec![false; lines.len()];
        let mut in_fenced_block = false;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Check for fenced code blocks
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_fenced_block = !in_fenced_block;
                in_code_block[i] = true;
                continue;
            }

            if in_fenced_block {
                in_code_block[i] = true;
                continue;
            }
        }

        in_code_block
    }
}

impl Default for MD044 {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for MD044 {
    fn id(&self) -> &'static str {
        "MD044"
    }

    fn name(&self) -> &'static str {
        "proper-names"
    }

    fn description(&self) -> &'static str {
        "Proper names should have the correct capitalization"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::Content).introduced_in("mdbook-lint v0.1.0")
    }

    fn check_with_ast<'a>(
        &self,
        document: &Document,
        _ast: Option<&'a comrak::nodes::AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        let lines: Vec<&str> = document.content.lines().collect();
        let in_code_block = self.get_code_block_ranges(&lines);

        for (line_number, line) in lines.iter().enumerate() {
            let line_number = line_number + 1;

            // Skip lines inside code blocks
            if in_code_block[line_number - 1] {
                continue;
            }

            violations.extend(self.check_line_names(line, line_number));
        }

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
    fn test_md044_correct_capitalization_valid() {
        let content = r#"This document uses JavaScript and GitHub correctly.

We also use Node.js and MongoDB in our stack.

The API is built with GraphQL and runs on AWS.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md044_incorrect_capitalization_violation() {
        let content = r#"This document uses javascript and github incorrectly.

We also use nodejs and mongodb in our stack.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 4);
        assert!(violations[0].message.contains("javascript"));
        assert!(violations[0].message.contains("JavaScript"));
        assert!(violations[1].message.contains("github"));
        assert!(violations[1].message.contains("GitHub"));
        assert!(violations[2].message.contains("nodejs"));
        assert!(violations[2].message.contains("Node.js"));
        assert!(violations[3].message.contains("mongodb"));
        assert!(violations[3].message.contains("MongoDB"));
    }

    #[test]
    fn test_md044_mixed_correct_incorrect() {
        let content = r#"We use JavaScript (correct) but also javascript (incorrect).

GitHub is right, but github is wrong.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("javascript"));
        assert!(violations[1].message.contains("github"));
    }

    #[test]
    fn test_md044_code_blocks_ignored() {
        let content = r#"We use JavaScript in our application.

```javascript
// This javascript in code should be ignored
console.log("github");
```

But javascript outside code blocks should be flagged.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 8);
    }

    #[test]
    fn test_md044_code_spans_ignored() {
        let content = r#"We use JavaScript, and in code we write `javascript` or `github.com`.

But javascript outside of `code spans` should be flagged.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 3);
    }

    #[test]
    fn test_md044_custom_names() {
        let content = r#"We use mycompany products and someapi.

This should flag mycompany and someapi.
"#;

        let mut custom_names = HashMap::new();
        custom_names.insert("mycompany".to_string(), "MyCompany".to_string());
        custom_names.insert("someapi".to_string(), "SomeAPI".to_string());

        let document = create_test_document(content);
        let rule = MD044::with_names(custom_names);
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 4); // 2 on each line
        assert!(violations[0].message.contains("MyCompany"));
        assert!(violations[1].message.contains("SomeAPI"));
    }

    #[test]
    fn test_md044_word_boundaries() {
        let content = r#"The word javascript should be flagged.

But javascriptlike should not be flagged (it's a different word).

And notjavascript should also not be flagged.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 1);
    }

    #[test]
    fn test_md044_case_insensitive_matching() {
        let content = r#"We use Javascript, JAVASCRIPT, and JaVaScRiPt.

All variations should be flagged.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 3);
        assert!(violations[0].message.contains("Javascript"));
        assert!(violations[1].message.contains("JAVASCRIPT"));
        assert!(violations[2].message.contains("JaVaScRiPt"));
    }

    #[test]
    fn test_md044_multiple_occurrences_per_line() {
        let content = r#"Using javascript and github and nodejs in the same line.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 3);
        assert!(violations[0].message.contains("javascript"));
        assert!(violations[1].message.contains("github"));
        assert!(violations[2].message.contains("nodejs"));
    }

    #[test]
    fn test_md044_no_proper_names() {
        let content = r#"This document doesn't contain any configured proper names.

Just regular words and sentences here.

Nothing to flag in this content.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 0);
    }

    #[test]
    fn test_md044_acronyms() {
        let content = r#"We use api, rest, and json in our application.

These should be API, REST, and JSON.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 3); // Only line 1 has incorrect capitalization
        assert!(violations[0].message.contains("API"));
        assert!(violations[1].message.contains("REST"));
        assert!(violations[2].message.contains("JSON"));
    }

    #[test]
    fn test_md044_multi_word_names() {
        let content = r#"We deploy to google cloud platform.

Should be Google Cloud not google cloud.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();
        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("google cloud"));
        assert!(violations[1].message.contains("google cloud"));
    }
}
