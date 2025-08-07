//! MDBOOK007: Validate include file paths and existence
//!
//! This rule validates that all include directives point to existing files with correct
//! syntax, preventing build failures and broken includes in mdBook projects.

use crate::rule::{AstRule, RuleCategory, RuleMetadata};
use crate::{
    Document,
    violation::{Severity, Violation},
};
use comrak::nodes::AstNode;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::{fs, io};

/// MDBOOK007: Validate include file paths and existence
///
/// This rule validates that all include directives in markdown files point to existing
/// files with correct syntax. It prevents build failures and broken includes by checking:
///
/// The rule:
/// - Finds all include directive patterns in markdown content
/// - Resolves include paths relative to the source file
/// - Validates target files exist and are readable
/// - Checks line range syntax and bounds where applicable
/// - Verifies anchor references exist in target files
/// - Detects circular include dependencies
/// - Provides clear error messages for debugging
///
/// Include Directive Formats Supported:
/// - Basic file includes: `{{#include file.txt}}`
/// - Line ranges: `{{#include file.rs:10:20}}`
/// - Named anchors: `{{#include file.rs:anchor_name}}`
/// - Relative paths: `{{#include ../other/file.md}}`
/// - Rust-specific: `{{#rustdoc_include file.rs}}`
#[derive(Default)]
pub struct MDBOOK007 {
    /// Cache of file existence and content to avoid repeated filesystem access
    file_cache: Arc<RwLock<HashMap<PathBuf, Option<String>>>>,
    /// Track processed files to detect circular dependencies
    processing_stack: Arc<RwLock<Vec<PathBuf>>>,
}

impl AstRule for MDBOOK007 {
    fn id(&self) -> &'static str {
        "MDBOOK007"
    }

    fn name(&self) -> &'static str {
        "include-validation"
    }

    fn description(&self) -> &'static str {
        "Include directives must point to existing files with valid syntax"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::stable(RuleCategory::MdBook).introduced_in("mdbook-lint v0.2.0")
    }

    fn check_ast<'a>(
        &self,
        document: &Document,
        _ast: &'a AstNode<'a>,
    ) -> crate::error::Result<Vec<Violation>> {
        let mut violations = Vec::new();

        // Clear processing stack for this document
        {
            if let Ok(mut stack) = self.processing_stack.write() {
                stack.clear();
                stack.push(document.path.clone());
            }
        }

        // Find all include directives in the document content
        let include_directives = self.find_include_directives(&document.content);

        for directive in include_directives {
            if let Some(violation) = self.validate_include_directive(document, &directive)? {
                violations.push(violation);
            }
        }

        Ok(violations)
    }
}

/// Represents an include directive found in markdown content
#[derive(Debug, Clone)]
struct IncludeDirective {
    /// The full matched directive text
    #[allow(dead_code)]
    full_match: String,
    /// The type of include (include, rustdoc_include, etc.)
    #[allow(dead_code)]
    directive_type: String,
    /// The file path specified in the directive
    file_path: String,
    /// Optional line range (start:end) or anchor name
    range_or_anchor: Option<String>,
    /// Line number where the directive was found
    line_number: usize,
    /// Column position in the line
    column: usize,
}

impl MDBOOK007 {
    /// Find all include directives in markdown content
    fn find_include_directives(&self, content: &str) -> Vec<IncludeDirective> {
        let mut directives = Vec::new();

        for (line_number, line) in content.lines().enumerate() {
            // Look for include directive patterns
            // Pattern: {{#include file.txt}} or {{#include file.rs:10:20}} or {{#include file.rs:anchor}}
            if let Some(directive) = self.parse_include_directive(line, line_number + 1) {
                directives.push(directive);
            }
        }

        directives
    }

    /// Parse a single include directive from a line
    fn parse_include_directive(&self, line: &str, line_number: usize) -> Option<IncludeDirective> {
        // Look for patterns like {{#include ...}} or {{#rustdoc_include ...}}
        let trimmed = line.trim();

        // Find the start of a directive
        if let Some(start) = trimmed.find("{{#")
            && let Some(end) = trimmed[start..].find("}}") {
            let directive_content = &trimmed[start + 3..start + end];
            let parts: Vec<&str> = directive_content.split_whitespace().collect();

            if parts.len() >= 2 {
                let directive_type = parts[0];

                // Only process include-type directives
                if directive_type == "include" || directive_type == "rustdoc_include" {
                    let file_spec = parts[1];
                    let (file_path, range_or_anchor) = self.parse_file_spec(file_spec);

                    return Some(IncludeDirective {
                        full_match: trimmed[start..start + end + 2].to_string(),
                        directive_type: directive_type.to_string(),
                        file_path: file_path.to_string(),
                        range_or_anchor,
                        line_number,
                        column: start + 1,
                    });
                }
            }
        }

        None
    }

    /// Parse file specification to extract path and range/anchor
    fn parse_file_spec<'a>(&self, file_spec: &'a str) -> (&'a str, Option<String>) {
        // Handle formats like:
        // - file.txt
        // - file.rs:10:20
        // - file.rs:anchor_name
        // - file.rs:10  (single line)

        if let Some(colon_pos) = file_spec.find(':') {
            let file_path = &file_spec[..colon_pos];
            let range_spec = &file_spec[colon_pos + 1..];
            (file_path, Some(range_spec.to_string()))
        } else {
            (file_spec, None)
        }
    }

    /// Validate a single include directive
    fn validate_include_directive(
        &self,
        document: &Document,
        directive: &IncludeDirective,
    ) -> crate::error::Result<Option<Violation>> {
        // Resolve the target file path relative to current document
        let target_path = self.resolve_include_path(&document.path, &directive.file_path);

        // Check if file exists and is readable
        match self.get_file_content(&target_path)? {
            Some(content) => {
                // File exists, now validate the range/anchor if specified
                if let Some(range_or_anchor) = &directive.range_or_anchor
                    && let Some(violation) = self.validate_range_or_anchor(
                        directive,
                        &target_path,
                        &content,
                        range_or_anchor,
                    )? {
                    return Ok(Some(violation));
                }

                // Check for circular dependencies
                if let Some(violation) = self.check_circular_dependency(&target_path, directive)? {
                    return Ok(Some(violation));
                }

                Ok(None)
            }
            None => {
                // File doesn't exist
                let message = format!(
                    "Include file '{}' not found. Resolved path: {}",
                    directive.file_path,
                    target_path.display()
                );

                Ok(Some(self.create_violation(
                    message,
                    directive.line_number,
                    directive.column,
                    Severity::Error,
                )))
            }
        }
    }

    /// Resolve include file path relative to current document
    fn resolve_include_path(&self, current_doc_path: &Path, include_path: &str) -> PathBuf {
        let current_dir = current_doc_path.parent().unwrap_or(Path::new("."));

        if let Some(stripped) = include_path.strip_prefix('/') {
            // Absolute path (relative to project root)
            PathBuf::from(stripped)
        } else {
            // Relative path
            current_dir.join(include_path)
        }
    }

    /// Get file content with caching
    fn get_file_content(&self, file_path: &Path) -> io::Result<Option<String>> {
        let canonical_path = match file_path.canonicalize() {
            Ok(path) => path,
            Err(_) => file_path.to_path_buf(),
        };

        // Check cache first
        {
            if let Ok(cache) = self.file_cache.read()
                && let Some(cached_content) = cache.get(&canonical_path) {
                return Ok(cached_content.clone());
            }
        }

        // Read file content
        let content = fs::read_to_string(file_path).ok();

        // Cache the result
        {
            if let Ok(mut cache) = self.file_cache.write() {
                cache.insert(canonical_path, content.clone());
            }
        }

        Ok(content)
    }

    /// Validate line range or anchor specification
    fn validate_range_or_anchor(
        &self,
        directive: &IncludeDirective,
        target_path: &Path,
        content: &str,
        range_or_anchor: &str,
    ) -> crate::error::Result<Option<Violation>> {
        // Try to parse as line range first (e.g., "10:20" or "10")
        if self.is_line_range(range_or_anchor) {
            return self.validate_line_range(directive, target_path, content, range_or_anchor);
        }

        // Check if it looks like it was intended to be a line range but is malformed
        if self.looks_like_malformed_line_range(range_or_anchor) {
            return Ok(Some(self.create_violation(
                format!("Invalid line number format '{range_or_anchor}'. Expected number or number:number format."),
                directive.line_number,
                directive.column,
                Severity::Error,
            )));
        }

        // Otherwise treat as anchor name
        self.validate_anchor(directive, target_path, content, range_or_anchor)
    }

    /// Check if the specification looks like a line range
    fn is_line_range(&self, spec: &str) -> bool {
        // Check if it's all digits, or digits:digits
        spec.chars().all(|c| c.is_ascii_digit() || c == ':') && !spec.is_empty()
    }

    /// Check if the specification looks like it was intended to be a line range but is malformed
    fn looks_like_malformed_line_range(&self, spec: &str) -> bool {
        // Check for patterns that suggest line range intent but are invalid
        // Like mixing letters and digits, or having colons in wrong places
        if spec.is_empty() {
            return false;
        }

        let has_digits = spec.chars().any(|c| c.is_ascii_digit());
        let has_colon = spec.contains(':');

        // Pattern 1: Has digits mixed with letters (like "10abc" or "abc10")
        // This suggests someone tried to write a line number but made a typo
        if has_digits {
            let has_letters = spec.chars().any(|c| c.is_ascii_alphabetic());
            if has_letters {
                return true;
            }
        }

        // Pattern 2: Malformed colon usage (like ":10", "10:", "10:abc")
        if has_colon && (spec.starts_with(':') || spec.ends_with(':')) {
            return true;
        }

        // Pattern 3: Short strings that are just letters (likely intended as line numbers, not anchors)
        // Only flag very short strings (3 chars or less) that are pure alphabetic
        // Longer strings with underscores/hyphens are clearly anchor names
        if spec.len() <= 3
            && spec.chars().all(|c| c.is_ascii_alphabetic())
            && !spec.contains('_')
            && !spec.contains('-')
        {
            return true;
        }

        false
    }

    /// Validate line range specification
    fn validate_line_range(
        &self,
        directive: &IncludeDirective,
        _target_path: &Path,
        content: &str,
        range_spec: &str,
    ) -> crate::error::Result<Option<Violation>> {
        let line_count = content.lines().count();

        let (start_line, end_line) = if let Some(colon_pos) = range_spec.find(':') {
            // Range format "start:end"
            let start_str = &range_spec[..colon_pos];
            let end_str = &range_spec[colon_pos + 1..];

            let start = match start_str.parse::<usize>() {
                Ok(n) if n > 0 => n,
                _ => {
                    return Ok(Some(self.create_violation(
                        format!("Invalid start line number '{start_str}' in range specification"),
                        directive.line_number,
                        directive.column,
                        Severity::Error,
                    )));
                }
            };

            let end = match end_str.parse::<usize>() {
                Ok(n) if n > 0 => n,
                _ => {
                    return Ok(Some(self.create_violation(
                        format!("Invalid end line number '{end_str}' in range specification"),
                        directive.line_number,
                        directive.column,
                        Severity::Error,
                    )));
                }
            };

            if start > end {
                return Ok(Some(self.create_violation(
                    format!("Start line {start} cannot be greater than end line {end}"),
                    directive.line_number,
                    directive.column,
                    Severity::Error,
                )));
            }

            (start, end)
        } else {
            // Single line format "N"
            let line_num = match range_spec.parse::<usize>() {
                Ok(n) if n > 0 => n,
                _ => {
                    return Ok(Some(self.create_violation(
                        format!("Invalid line number '{range_spec}'"),
                        directive.line_number,
                        directive.column,
                        Severity::Error,
                    )));
                }
            };
            (line_num, line_num)
        };

        // Check if line range is within file bounds
        if start_line > line_count || end_line > line_count {
            let message = if start_line == end_line {
                format!("Line {start_line} does not exist in file (file has {line_count} lines)")
            } else {
                format!(
                    "Line range {start_line}:{end_line} exceeds file length (file has {line_count} lines)"
                )
            };

            return Ok(Some(self.create_violation(
                message,
                directive.line_number,
                directive.column,
                Severity::Error,
            )));
        }

        Ok(None)
    }

    /// Validate anchor specification
    fn validate_anchor(
        &self,
        directive: &IncludeDirective,
        _target_path: &Path,
        content: &str,
        anchor: &str,
    ) -> crate::error::Result<Option<Violation>> {
        // Look for the anchor in the file content
        // Anchors are typically comments like "// ANCHOR: anchor_name" or "<!-- ANCHOR: anchor_name -->"
        let anchor_patterns = [
            format!("// ANCHOR: {anchor}"),
            format!("# ANCHOR: {anchor}"),
            format!("<!-- ANCHOR: {anchor} -->"),
            format!("<!-- anchor: {anchor} -->"),
        ];

        let mut found = false;
        for line in content.lines() {
            for pattern in &anchor_patterns {
                if line.contains(pattern) {
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
        }

        if !found {
            return Ok(Some(self.create_violation(
                format!(
                    "Anchor '{}' not found in included file. Expected patterns: {}",
                    anchor,
                    anchor_patterns.join(", ")
                ),
                directive.line_number,
                directive.column,
                Severity::Error,
            )));
        }

        Ok(None)
    }

    /// Check for circular include dependencies
    fn check_circular_dependency(
        &self,
        target_path: &Path,
        directive: &IncludeDirective,
    ) -> crate::error::Result<Option<Violation>> {
        {
            if let Ok(stack) = self.processing_stack.read()
                && stack.contains(&target_path.to_path_buf()) {
                return Ok(Some(self.create_violation(
                    format!(
                        "Circular include dependency detected: {} -> {}",
                        stack.last().unwrap().display(),
                        target_path.display()
                    ),
                    directive.line_number,
                    directive.column,
                    Severity::Error,
                )));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::Rule;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_document(content: &str, file_path: &Path) -> crate::error::Result<Document> {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file_path, content)?;
        Document::new(content.to_string(), file_path.to_path_buf())
    }

    #[test]
    fn test_mdbook007_valid_basic_include() -> crate::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create target file
        create_test_document("Hello, included content!", &root.join("included.txt"))?;

        // Create source file with include
        let source_content = r#"# Chapter 1

{{#include included.txt}}

More content here."#;
        let source_path = root.join("chapter.md");
        let doc = create_test_document(source_content, &source_path)?;

        let rule = MDBOOK007::default();
        let violations = rule.check(&doc)?;

        assert_eq!(
            violations.len(),
            0,
            "Valid include should have no violations"
        );
        Ok(())
    }

    #[test]
    fn test_mdbook007_missing_file() -> crate::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create source file with missing include
        let source_content = r#"# Chapter 1

{{#include nonexistent.txt}}

More content here."#;
        let source_path = root.join("chapter.md");
        let doc = create_test_document(source_content, &source_path)?;

        let rule = MDBOOK007::default();
        let violations = rule.check(&doc)?;

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MDBOOK007");
        assert!(violations[0].message.contains("not found"));
        assert!(violations[0].message.contains("nonexistent.txt"));
        Ok(())
    }

    #[test]
    fn test_mdbook007_valid_line_range() -> crate::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create target file with multiple lines
        let target_content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\n";
        create_test_document(target_content, &root.join("lines.txt"))?;

        // Create source file with line range include
        let source_content = r#"# Chapter 1

{{#include lines.txt:2:4}}

More content here."#;
        let source_path = root.join("chapter.md");
        let doc = create_test_document(source_content, &source_path)?;

        let rule = MDBOOK007::default();
        let violations = rule.check(&doc)?;

        assert_eq!(
            violations.len(),
            0,
            "Valid line range should have no violations"
        );
        Ok(())
    }

    #[test]
    fn test_mdbook007_invalid_line_range() -> crate::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create target file with 3 lines
        let target_content = "Line 1\nLine 2\nLine 3\n";
        create_test_document(target_content, &root.join("lines.txt"))?;

        // Create source file with out-of-bounds line range
        let source_content = r#"# Chapter 1

{{#include lines.txt:2:10}}

More content here."#;
        let source_path = root.join("chapter.md");
        let doc = create_test_document(source_content, &source_path)?;

        let rule = MDBOOK007::default();
        let violations = rule.check(&doc)?;

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MDBOOK007");
        assert!(violations[0].message.contains("exceeds file length"));
        Ok(())
    }

    #[test]
    fn test_mdbook007_single_line_include() -> crate::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create target file
        let target_content = "Line 1\nLine 2\nLine 3\n";
        create_test_document(target_content, &root.join("lines.txt"))?;

        // Create source file with single line include
        let source_content = r#"# Chapter 1

{{#include lines.txt:2}}

More content here."#;
        let source_path = root.join("chapter.md");
        let doc = create_test_document(source_content, &source_path)?;

        let rule = MDBOOK007::default();
        let violations = rule.check(&doc)?;

        assert_eq!(
            violations.len(),
            0,
            "Valid single line include should have no violations"
        );
        Ok(())
    }

    #[test]
    fn test_mdbook007_valid_anchor() -> crate::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create target file with anchor
        let target_content = r#"fn main() {
    // ANCHOR: example
    println!("Hello, world!");
    // ANCHOR_END: example
}"#;
        create_test_document(target_content, &root.join("example.rs"))?;

        // Create source file with anchor include
        let source_content = r#"# Chapter 1

{{#include example.rs:example}}

More content here."#;
        let source_path = root.join("chapter.md");
        let doc = create_test_document(source_content, &source_path)?;

        let rule = MDBOOK007::default();
        let violations = rule.check(&doc)?;

        assert_eq!(
            violations.len(),
            0,
            "Valid anchor include should have no violations"
        );
        Ok(())
    }

    #[test]
    fn test_mdbook007_missing_anchor() -> crate::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create target file without the anchor
        let target_content = r#"fn main() {
    println!("Hello, world!");
}"#;
        create_test_document(target_content, &root.join("example.rs"))?;

        // Create source file with missing anchor include
        let source_content = r#"# Chapter 1

{{#include example.rs:missing_anchor}}

More content here."#;
        let source_path = root.join("chapter.md");
        let doc = create_test_document(source_content, &source_path)?;

        let rule = MDBOOK007::default();
        let violations = rule.check(&doc)?;

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MDBOOK007");
        assert!(
            violations[0]
                .message
                .contains("Anchor 'missing_anchor' not found")
        );
        Ok(())
    }

    #[test]
    fn test_mdbook007_rustdoc_include() -> crate::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create target rust file
        create_test_document("fn example() {}", &root.join("lib.rs"))?;

        // Create source file with rustdoc_include
        let source_content = r#"# Chapter 1

{{#rustdoc_include lib.rs}}

More content here."#;
        let source_path = root.join("chapter.md");
        let doc = create_test_document(source_content, &source_path)?;

        let rule = MDBOOK007::default();
        let violations = rule.check(&doc)?;

        assert_eq!(
            violations.len(),
            0,
            "Valid rustdoc_include should have no violations"
        );
        Ok(())
    }

    #[test]
    fn test_mdbook007_invalid_line_number_format() -> crate::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create target file
        create_test_document("Line 1\nLine 2\n", &root.join("lines.txt"))?;

        // Create source file with invalid line number
        let source_content = r#"# Chapter 1

{{#include lines.txt:abc}}

More content here."#;
        let source_path = root.join("chapter.md");
        let doc = create_test_document(source_content, &source_path)?;

        let rule = MDBOOK007::default();
        let violations = rule.check(&doc)?;

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MDBOOK007");
        assert!(violations[0].message.contains("Invalid line number format"));
        Ok(())
    }

    #[test]
    fn test_mdbook007_nested_includes() -> crate::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create nested directory structure
        fs::create_dir_all(root.join("nested"))?;
        create_test_document("Nested content", &root.join("nested/file.txt"))?;

        // Create source file with nested path
        let source_content = r#"# Chapter 1

{{#include nested/file.txt}}

More content here."#;
        let source_path = root.join("chapter.md");
        let doc = create_test_document(source_content, &source_path)?;

        let rule = MDBOOK007::default();
        let violations = rule.check(&doc)?;

        assert_eq!(
            violations.len(),
            0,
            "Nested include should have no violations"
        );
        Ok(())
    }

    #[test]
    fn test_parse_file_spec() {
        let rule = MDBOOK007::default();

        assert_eq!(rule.parse_file_spec("file.txt"), ("file.txt", None));
        assert_eq!(
            rule.parse_file_spec("file.rs:10:20"),
            ("file.rs", Some("10:20".to_string()))
        );
        assert_eq!(
            rule.parse_file_spec("file.rs:anchor"),
            ("file.rs", Some("anchor".to_string()))
        );
        assert_eq!(
            rule.parse_file_spec("path/to/file.txt:5"),
            ("path/to/file.txt", Some("5".to_string()))
        );
    }

    #[test]
    fn test_is_line_range() {
        let rule = MDBOOK007::default();

        assert!(rule.is_line_range("10"));
        assert!(rule.is_line_range("10:20"));
        assert!(rule.is_line_range("1:1"));
        assert!(!rule.is_line_range("anchor_name"));
        assert!(!rule.is_line_range("10:anchor"));
        assert!(!rule.is_line_range("abc:123"));
    }

    #[test]
    fn test_looks_like_malformed_line_range() {
        let rule = MDBOOK007::default();

        // Should detect malformed line ranges
        assert!(rule.looks_like_malformed_line_range("10abc"));
        assert!(rule.looks_like_malformed_line_range("abc10"));
        assert!(rule.looks_like_malformed_line_range(":10"));
        assert!(rule.looks_like_malformed_line_range("10:"));
        assert!(rule.looks_like_malformed_line_range("10:abc"));
        assert!(rule.looks_like_malformed_line_range("abc:123"));

        // Should not detect valid anchors as malformed line ranges
        assert!(!rule.looks_like_malformed_line_range("anchor_name"));
        assert!(!rule.looks_like_malformed_line_range("valid-anchor"));
        assert!(!rule.looks_like_malformed_line_range(""));

        // Short strings that are just letters are likely intended as line numbers
        assert!(rule.looks_like_malformed_line_range("abc"));

        // But longer strings are likely anchor names
        assert!(!rule.looks_like_malformed_line_range("anchor_name"));
        assert!(!rule.looks_like_malformed_line_range("valid-anchor"));
    }
}
