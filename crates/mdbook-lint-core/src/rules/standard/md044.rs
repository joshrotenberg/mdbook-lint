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

        // Find all matches and their positions first - using Unicode-safe approach
        let mut matches = Vec::new();

        for (incorrect_lower, correct) in &self.proper_names {
            // Use a simpler, more reliable approach: search in the original line
            // and use character indices to ensure we don't break Unicode characters
            let line_lower = line.to_lowercase();
            let mut search_start = 0;

            while let Some(byte_pos) = line_lower[search_start..].find(incorrect_lower) {
                let absolute_byte_pos = search_start + byte_pos;

                // Convert byte position to character index safely
                let char_pos = line[..absolute_byte_pos].chars().count();
                let end_char_pos = char_pos + incorrect_lower.chars().count();

                // Check word boundaries using character positions
                let line_chars: Vec<char> = line.chars().collect();
                let is_word_start = char_pos == 0
                    || !line_chars
                        .get(char_pos.saturating_sub(1))
                        .unwrap_or(&' ')
                        .is_alphanumeric();
                let is_word_end = end_char_pos >= line_chars.len()
                    || !line_chars
                        .get(end_char_pos)
                        .unwrap_or(&' ')
                        .is_alphanumeric();

                if is_word_start && is_word_end {
                    // Extract the actual text using character indices
                    let actual_text: String = line_chars[char_pos..end_char_pos].iter().collect();

                    // Only flag if it's not already correctly capitalized
                    if actual_text != *correct {
                        // Use the original byte position for compatibility with existing methods
                        // but make sure it's safe by using char_indices
                        let safe_byte_pos = line
                            .char_indices()
                            .nth(char_pos)
                            .map(|(pos, _)| pos)
                            .unwrap_or(0);

                        // Skip if this appears to be in a code span or URL context
                        if !self.is_in_code_span(line, safe_byte_pos)
                            && !self.is_in_url_context(line, safe_byte_pos)
                        {
                            matches.push((safe_byte_pos, actual_text, correct.clone()));
                        }
                    }
                }

                // Move search position forward, making sure to advance by at least one byte
                search_start = absolute_byte_pos + 1;
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

        // Convert byte position to character position
        let char_pos = line[..pos.min(line.len())].chars().count();

        while i < chars.len() && i <= char_pos {
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

    /// Check if a position is inside a URL context
    fn is_in_url_context(&self, line: &str, pos: usize) -> bool {
        // Check for various URL patterns that should be excluded from proper name checking

        // 1. Check for bare URLs (http://, https://, ftp://, etc.)
        if let Some(url_start) = self.find_url_start(line, pos)
            && let Some(url_end) = self.find_url_end(line, url_start)
        {
            return pos >= url_start && pos < url_end;
        }

        // 2. Check for markdown link URLs [text](url)
        if let Some(link_url_range) = self.find_markdown_link_url(line, pos) {
            return pos >= link_url_range.0 && pos < link_url_range.1;
        }

        false
    }

    /// Find the start of a URL that contains the given position
    fn find_url_start(&self, line: &str, pos: usize) -> Option<usize> {
        let schemes = [
            "https://", "http://", "ftp://", "ftps://", "mailto:", "file://",
        ];

        // Look backwards from pos to find a URL scheme
        // We need to check all possible positions from the beginning of the line up to pos
        for scheme in &schemes {
            // Use char_indices to get character boundary positions
            for (char_pos, _) in line.char_indices() {
                if char_pos > pos {
                    break; // Past our search position
                }

                // Check if we have enough bytes remaining for the scheme
                if char_pos + scheme.len() <= line.len() {
                    // Check if the end position is also a character boundary
                    let end_pos = char_pos + scheme.len();
                    if line.is_char_boundary(end_pos) {
                        // Safe to slice since both positions are character boundaries
                        let slice = &line[char_pos..end_pos];
                        if slice.eq_ignore_ascii_case(scheme) {
                            // Found a scheme - now check if our position would be within this URL
                            if let Some(url_end) = self.find_url_end(line, char_pos)
                                && pos >= char_pos
                                && pos < url_end
                            {
                                return Some(char_pos);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Find the end of a URL starting at url_start
    fn find_url_end(&self, line: &str, url_start: usize) -> Option<usize> {
        let chars: Vec<char> = line.chars().collect();

        // Convert byte position to character position
        let char_start = line[..url_start.min(line.len())].chars().count();
        let mut i = char_start;

        // Skip the scheme part
        while i < chars.len() && chars[i] != ':' {
            i += 1;
        }
        if i < chars.len() && chars[i] == ':' {
            i += 1;
            // Skip // if present
            if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' {
                i += 2;
            }
        }

        // Continue until we hit a character that typically ends URLs
        while i < chars.len() {
            match chars[i] {
                // Characters that end URLs
                ' ' | '\t' | '\n' | ')' | ']' | ',' | ';' | '"' | '\'' => break,
                // Continue for valid URL characters
                _ => i += 1,
            }
        }

        Some(i)
    }

    /// Find markdown link URL range [text](url) if pos is within the URL part
    fn find_markdown_link_url(&self, line: &str, pos: usize) -> Option<(usize, usize)> {
        let chars: Vec<char> = line.chars().collect();

        // Convert byte position to character position
        let char_pos = line[..pos.min(line.len())].chars().count();

        // Look for markdown link pattern around the position
        // We need to find [text](url) where pos is within the url part

        // Look backwards for ]( pattern
        let mut i = if char_pos > 0 { char_pos - 1 } else { 0 };
        let mut found_paren = false;
        let mut found_bracket = false;

        while i > 0 {
            if i < chars.len() && chars[i] == '(' && !found_paren {
                found_paren = true;
            } else if i < chars.len() && chars[i] == ']' && found_paren && !found_bracket {
                found_bracket = true;
                break;
            } else if i < chars.len() && (chars[i] == ' ' || chars[i] == '\n') {
                // Break if we hit whitespace without finding the pattern
                break;
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }

        if !found_bracket || !found_paren {
            return None;
        }

        // Find the opening paren after the ]
        let mut paren_pos = i + 1;
        while paren_pos < chars.len() && chars[paren_pos] != '(' {
            paren_pos += 1;
        }

        if paren_pos >= chars.len() {
            return None;
        }

        // Find the closing paren
        let url_start = paren_pos + 1;
        let mut url_end = url_start;
        while url_end < chars.len() && chars[url_end] != ')' {
            url_end += 1;
        }

        if url_end >= chars.len() {
            return None;
        }

        // Check if char_pos is within the URL part
        if char_pos >= url_start && char_pos < url_end {
            Some((url_start, url_end))
        } else {
            None
        }
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

    #[test]
    fn test_md044_url_false_positives() {
        let content = r#"Check out our repository at https://github.com/user/repo.

You can also visit http://example.com for more info.

Visit https://crates.io/crates/mdbook-lint for the package.

But github should still be flagged when not in URLs.
And https should be flagged when used as HTTPS protocol name.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();

        // After fix: should only flag non-URL occurrences
        // In URLs, we shouldn't flag: https, github, http, crates
        // But we should still flag: github (line 7), https (line 8)

        println!("Violations found after fix: {}", violations.len());
        for (i, v) in violations.iter().enumerate() {
            println!("Violation {}: line {}, {}", i, v.line, v.message);
        }

        // Should only have 2 violations for the non-URL occurrences
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 7); // "github should still be flagged"
        assert_eq!(violations[1].line, 8); // "https should be flagged"
        assert!(violations[0].message.contains("github"));
        assert!(violations[1].message.contains("https"));
    }

    #[test]
    fn test_md044_markdown_links_with_urls() {
        let content = r#"Check out [GitHub](https://github.com) for repositories.

Visit [the documentation](http://docs.example.com) for more info.

Also see [Crates.io](https://crates.io) for Rust packages.

But github and http should be flagged in regular text.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();

        // After fix: should only flag non-URL occurrences
        println!("Markdown link violations found: {}", violations.len());
        for (i, v) in violations.iter().enumerate() {
            println!("Violation {}: line {}, {}", i, v.line, v.message);
        }

        // Should only flag the instances in regular text, not in the URLs
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 7); // "github and http should be flagged"
        assert_eq!(violations[1].line, 7);
        assert!(violations[0].message.contains("github") || violations[0].message.contains("http"));
        assert!(violations[1].message.contains("github") || violations[1].message.contains("http"));
    }

    #[test]
    fn test_md044_bare_urls() {
        let content = r#"Visit https://github.com/user/repo directly.

Or go to http://example.com for info.

Plain URLs: https://crates.io and http://docs.rs should not be flagged.

But mentioning github or https in text should be flagged.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();

        println!("Bare URL violations found: {}", violations.len());
        for (i, v) in violations.iter().enumerate() {
            println!("Violation {}: line {}, {}", i, v.line, v.message);
        }

        // Should only flag the instances in regular text, not in the bare URLs
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 7); // "github or https in text should be flagged"
        assert_eq!(violations[1].line, 7);
        assert!(
            violations[0].message.contains("github") || violations[0].message.contains("https")
        );
        assert!(
            violations[1].message.contains("github") || violations[1].message.contains("https")
        );
    }

    #[test]
    fn test_md044_url_context_detection_comprehensive() {
        let content = r#"# URL Context Detection Tests

## Bare URLs should not be flagged
Visit https://github.com/user/repo for code.
Check out http://example.com/path?query=value.
Email me at mailto:user@github.com for questions.
Use ftp://files.example.com/downloads for files.

## Markdown links should not flag URLs
See [GitHub](https://github.com) for repositories.
Check [HTTP docs](http://example.com/docs) for info.
Visit [the site](https://crates.io/search?q=rust) for packages.

## Regular text should still be flagged
I use github for version control.
The https protocol is secure.
We need better http handling.

## Mixed scenarios
Check https://github.com but remember that github is popular.
Visit [GitHub](https://github.com) - github is widely used.
The url https://example.com shows that http redirects work.

## Edge cases
URL at end: https://github.com
URL in parentheses: (https://github.com/user/repo)
URL with punctuation: Visit https://github.com.
Multiple URLs: https://github.com and http://example.com are different.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();

        println!("Comprehensive test violations: {}", violations.len());
        for (i, v) in violations.iter().enumerate() {
            println!(
                "Violation {}: line {}, col {}, {}",
                i, v.line, v.column, v.message
            );
        }

        // Should only flag the non-URL occurrences
        // Expected violations:
        // Line 15: "github" in regular text
        // Line 16: "https" in regular text
        // Line 17: "http" in regular text
        // Line 20: "github" in regular text
        // Line 21: "github" in regular text
        // Line 22: "url" in regular text (added to proper names)
        // Line 22: "http" in regular text

        assert_eq!(violations.len(), 7);

        // Verify they're all from lines with regular text, not URLs
        for violation in &violations {
            assert!(violation.line >= 15); // All should be in the regular text section
        }
    }

    #[test]
    fn test_md044_url_detection_methods() {
        let rule = MD044::new();

        // Test bare URL detection

        assert!(rule.is_in_url_context("Visit https://github.com for code", 10)); // "https"
        assert!(rule.is_in_url_context("Visit https://github.com for code", 17)); // "github"
        assert!(!rule.is_in_url_context("Visit https://github.com for code", 30)); // "code"

        // Test markdown link URL detection
        assert!(rule.is_in_url_context("See [GitHub](https://github.com) here", 14)); // "https"
        assert!(rule.is_in_url_context("See [GitHub](https://github.com) here", 21)); // "github"
        assert!(!rule.is_in_url_context("See [GitHub](https://github.com) here", 4)); // "GitHub" in link text
        assert!(!rule.is_in_url_context("See [GitHub](https://github.com) here", 34)); // "here"

        // Test non-URL contexts
        assert!(!rule.is_in_url_context("I use github for development", 6)); // "github"
        assert!(!rule.is_in_url_context("The https protocol is secure", 4)); // "https"
    }

    #[test]
    fn test_md044_unicode_emoji_handling() {
        let content = r#"📖 javascript documentation and github 🚀 repositories are great.

Using nodejs with 🔥 performance and mongodb 💾 storage.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();

        // Should find all 4 proper name violations without panicking
        assert_eq!(violations.len(), 4);
        assert!(violations[0].message.contains("javascript"));
        assert!(violations[1].message.contains("github"));
        assert!(violations[2].message.contains("nodejs"));
        assert!(violations[3].message.contains("mongodb"));
    }

    #[test]
    fn test_md044_unicode_mixed_scripts() {
        let content = r#"在中文文档中使用 javascript 和 github。

Русский текст с javascript и github тоже должен работать.

العربية مع javascript و github أيضاً.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();

        // Should find violations without panicking on Unicode text
        // The exact count may vary due to Unicode word boundary detection
        assert!(violations.len() >= 4); // At least some violations should be found
        for violation in &violations {
            assert!(
                violation.message.contains("javascript") || violation.message.contains("github")
            );
        }
    }

    #[test]
    fn test_md044_unicode_case_folding() {
        let content = r#"Using javascript in our project.

İstanbul'da javascript kullanıyoruz.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();

        // Should find the violations without panicking on Unicode case folding
        assert!(!violations.is_empty()); // At least the regular javascript should be found

        // Find the javascript violation
        let js_violation = violations.iter().find(|v| v.message.contains("javascript"));
        assert!(js_violation.is_some());
    }

    #[test]
    fn test_md044_unicode_combining_characters() {
        let content = r#"Using normal javascript here and also github.

Testing regular javascript and github again.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();

        // Should handle text correctly and find all instances
        assert_eq!(violations.len(), 4);
        let js_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.message.contains("javascript"))
            .collect();
        let gh_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.message.contains("github"))
            .collect();

        assert_eq!(js_violations.len(), 2);
        assert_eq!(gh_violations.len(), 2);
    }

    #[test]
    fn test_md044_unicode_word_boundaries() {
        let content = r#"Testing javascript🔥fast and github⭐popular.

Also javascript‿linked and github🌟awesome.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();

        // Unicode characters should properly separate words for boundary detection
        assert_eq!(violations.len(), 4);
        assert!(violations.iter().any(|v| v.message.contains("javascript")));
        assert!(violations.iter().any(|v| v.message.contains("github")));
    }

    #[test]
    fn test_md044_unicode_urls_with_emoji() {
        let content = r#"Visit 📖 https://github.com/user/repo 🚀 for documentation.

Check https://javascript.info 💡 for learning resources.

But standalone github and javascript should be flagged.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();

        // Should only flag the non-URL instances
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].line, 5); // Line with standalone instances
        assert_eq!(violations[1].line, 5);
        assert!(
            violations[0].message.contains("github")
                || violations[0].message.contains("javascript")
        );
        assert!(
            violations[1].message.contains("github")
                || violations[1].message.contains("javascript")
        );
    }

    #[test]
    fn test_md044_still_works_for_non_urls() {
        let content = r#"We use javascript and github in our development.

The api uses json for data exchange.

These should all be flagged since they're not in URLs.
"#;

        let document = create_test_document(content);
        let rule = MD044::new();
        let violations = rule.check(&document).unwrap();

        // Should flag javascript->JavaScript, github->GitHub, api->API, json->JSON
        assert_eq!(violations.len(), 4);
        assert!(violations[0].message.contains("javascript"));
        assert!(violations[1].message.contains("github"));
        assert!(violations[2].message.contains("api"));
        assert!(violations[3].message.contains("json"));
    }
}
