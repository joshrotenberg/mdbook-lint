//! MDBOOK007: Validate include file paths and existence
//!
//! This rule validates that all include directives point to existing files with correct
//! syntax, preventing build failures and broken includes in mdBook projects.

use comrak::nodes::AstNode;
use mdbook_lint_core::rule::{AstRule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{
    Document,
    violation::{Severity, Violation},
};
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, RwLock};
use std::{fs, io};

/// Anchor start marker, as mdBook matches it.
///
/// mdBook finds `ANCHOR:` anywhere on a line, whatever comment syntax precedes
/// it, and compares the captured name exactly. Matching only a fixed list of
/// comment prefixes rejected valid anchors in SQL, Lua, CSS and similar files,
/// and substring matching let `abc` resolve against `ANCHOR: abcd`.
static ANCHOR_START: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"ANCHOR:\s*(?P<anchor_name>[\w_-]+)").expect("Invalid regex"));

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
    ) -> mdbook_lint_core::error::Result<Vec<Violation>> {
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
            directives.extend(self.parse_include_directives(line, line_number + 1));
        }

        directives
    }

    /// Parse every include directive on a line
    ///
    /// A line can carry more than one directive, and an escaped directive can sit
    /// beside a real one, so every `{{#` on the line is examined rather than only
    /// the first.
    fn parse_include_directives(&self, line: &str, line_number: usize) -> Vec<IncludeDirective> {
        // Look for patterns like {{#include ...}} or {{#rustdoc_include ...}}
        let mut directives = Vec::new();
        let trimmed = line.trim();
        let mut search_from = 0;

        while let Some(relative_start) = trimmed[search_from..].find("{{#") {
            let start = search_from + relative_start;
            let Some(relative_end) = trimmed[start..].find("}}") else {
                break;
            };
            let end = start + relative_end;

            // Resume after this directive, whether or not it produced a result.
            search_from = end + 2;

            // mdBook does not process an escaped directive. It renders the literal
            // text `{{#include ...}}` instead, which books use to show the include
            // syntax to a reader, so the file it names need not exist.
            //
            // mdBook's escape pattern is `\\\{\{#.*\}\}` with a greedy `.*`, so
            // it runs to the last `}}` on the line and swallows any directive
            // after it. Nothing later on this line is processed.
            if trimmed[..start].ends_with('\\') {
                break;
            }

            let directive_content = &trimmed[start + 3..end];
            let parts: Vec<&str> = directive_content.split_whitespace().collect();

            if parts.len() >= 2 {
                let directive_type = parts[0];

                // Only process include-type directives
                if directive_type == "include" || directive_type == "rustdoc_include" {
                    let file_spec = parts[1];
                    let (file_path, range_or_anchor) = self.parse_file_spec(file_spec);

                    directives.push(IncludeDirective {
                        full_match: trimmed[start..end + 2].to_string(),
                        directive_type: directive_type.to_string(),
                        file_path: file_path.to_string(),
                        range_or_anchor,
                        line_number,
                        column: start + 1,
                    });
                }
            }
        }

        directives
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
    ) -> mdbook_lint_core::error::Result<Option<Violation>> {
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
                    )?
                {
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
                && let Some(cached_content) = cache.get(&canonical_path)
            {
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

    /// Validate the range or anchor that follows the include path
    ///
    /// This mirrors mdBook's `parse_range_or_anchor`, which never rejects a spec.
    /// The first `:`-separated segment is a start line when it is a number or
    /// empty. Otherwise the whole spec is an anchor named by that first segment,
    /// so `abc`, `step2`, `10abc` and `abc:123` are all anchors. Guessing at
    /// intent instead flagged valid anchor names as malformed line numbers.
    fn validate_range_or_anchor(
        &self,
        directive: &IncludeDirective,
        target_path: &Path,
        content: &str,
        range_or_anchor: &str,
    ) -> mdbook_lint_core::error::Result<Option<Violation>> {
        let first = range_or_anchor.split(':').next().unwrap_or("");

        if Self::is_range_start(first) {
            self.validate_line_range(directive, target_path, content, range_or_anchor)
        } else {
            self.validate_anchor(directive, target_path, content, first)
        }
    }

    /// Whether a spec's first segment makes it a line range rather than an anchor
    fn is_range_start(first_segment: &str) -> bool {
        first_segment.is_empty() || first_segment.parse::<usize>().is_ok()
    }

    /// Validate a line range specification
    ///
    /// mdBook accepts `N`, `N:M`, `N:` (to end of file), `:M` (from the start)
    /// and `:` (whole file). It never fails on a range, but some ranges include
    /// nothing or the wrong lines, and those are reported: a zero line number,
    /// a start past the end of the file, an end before the start, an end past
    /// the end of the file, and a non-numeric end, which mdBook silently widens
    /// to the end of the file.
    fn validate_line_range(
        &self,
        directive: &IncludeDirective,
        target_path: &Path,
        content: &str,
        range_spec: &str,
    ) -> mdbook_lint_core::error::Result<Option<Violation>> {
        let line_count = content.lines().count();
        let violation = |message: String| {
            Ok(Some(self.create_violation(
                message,
                directive.line_number,
                directive.column,
                Severity::Error,
            )))
        };

        // splitn(3) as mdBook does: a third segment is ignored.
        let mut parts = range_spec.splitn(3, ':');
        let start_str = parts.next().unwrap_or("");
        let end_str = parts.next();

        let start = match start_str.parse::<usize>() {
            _ if start_str.is_empty() => None,
            Ok(0) => {
                return violation(format!(
                    "Invalid start line number '{start_str}' in range specification"
                ));
            }
            Ok(n) => Some(n),
            // validate_range_or_anchor only routes numeric or empty starts here.
            Err(_) => return self.validate_anchor(directive, target_path, content, start_str),
        };

        let end = match end_str {
            // `N` alone includes that single line.
            None => start,
            // `N:` runs to the end of the file.
            Some("") => None,
            Some(end_str) => match end_str.parse::<usize>() {
                Ok(0) => {
                    return violation(format!(
                        "Invalid end line number '{end_str}' in range specification"
                    ));
                }
                Ok(n) => Some(n),
                Err(_) => {
                    let from =
                        start.map_or_else(|| "the start".to_string(), |s| format!("line {s}"));
                    return violation(format!(
                        "End line '{end_str}' is not a number, so mdBook includes from {from} to the end of the file"
                    ));
                }
            },
        };

        if let (Some(start), Some(end)) = (start, end)
            && start > end
        {
            return violation(format!(
                "Start line {start} cannot be greater than end line {end}"
            ));
        }

        if let Some(start) = start
            && start > line_count
        {
            return violation(if end == Some(start) {
                format!("Line {start} does not exist in file (file has {line_count} lines)")
            } else {
                format!(
                    "Line range {range_spec} starts past the end of the file (file has {line_count} lines)"
                )
            });
        }

        if let Some(end) = end
            && end > line_count
        {
            return violation(format!(
                "Line range {range_spec} exceeds file length (file has {line_count} lines)"
            ));
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
    ) -> mdbook_lint_core::error::Result<Option<Violation>> {
        let found = content.lines().any(|line| {
            ANCHOR_START
                .captures_iter(line)
                .any(|cap| &cap["anchor_name"] == anchor)
        });

        if !found {
            return Ok(Some(self.create_violation(
                format!(
                    "Anchor '{anchor}' not found in included file. Expected a line containing 'ANCHOR: {anchor}'"
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
    ) -> mdbook_lint_core::error::Result<Option<Violation>> {
        {
            if let Ok(stack) = self.processing_stack.read()
                && stack.contains(&target_path.to_path_buf())
            {
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
    use mdbook_lint_core::rule::Rule;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_document(
        content: &str,
        file_path: &Path,
    ) -> mdbook_lint_core::error::Result<Document> {
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file_path, content)?;
        Document::new(content.to_string(), file_path.to_path_buf())
    }

    #[test]
    fn test_mdbook007_valid_basic_include() -> mdbook_lint_core::error::Result<()> {
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
    fn test_mdbook007_missing_file() -> mdbook_lint_core::error::Result<()> {
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
    fn test_mdbook007_valid_line_range() -> mdbook_lint_core::error::Result<()> {
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
    fn test_mdbook007_invalid_line_range() -> mdbook_lint_core::error::Result<()> {
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
    fn test_mdbook007_single_line_include() -> mdbook_lint_core::error::Result<()> {
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
    fn test_mdbook007_valid_anchor() -> mdbook_lint_core::error::Result<()> {
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
    fn test_mdbook007_missing_anchor() -> mdbook_lint_core::error::Result<()> {
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
    fn test_mdbook007_rustdoc_include() -> mdbook_lint_core::error::Result<()> {
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
    fn test_mdbook007_invalid_line_number_format() -> mdbook_lint_core::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        // Create target file
        create_test_document("Line 1\nLine 2\n", &root.join("lines.txt"))?;

        // `10abc` does not parse as a line number, so mdBook treats it as an
        // anchor name. A mistyped line number therefore surfaces as a missing
        // anchor, which is what mdBook would actually look for.
        let source_content = r#"# Chapter 1

{{#include lines.txt:10abc}}

More content here."#;
        let source_path = root.join("chapter.md");
        let doc = create_test_document(source_content, &source_path)?;

        let rule = MDBOOK007::default();
        let violations = rule.check(&doc)?;

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "MDBOOK007");
        assert!(
            violations[0].message.contains("Anchor '10abc' not found"),
            "got {:?}",
            violations[0].message
        );
        Ok(())
    }

    #[test]
    fn test_mdbook007_nested_includes() -> mdbook_lint_core::error::Result<()> {
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
    fn test_is_range_start_mirrors_mdbook() {
        // Numeric or empty first segment: a line range.
        for first in ["10", "1", "0", ""] {
            assert!(MDBOOK007::is_range_start(first), "{first:?}");
        }
        // Anything else: an anchor, whatever it contains.
        for first in [
            "abc",
            "a",
            "step2",
            "example1",
            "10abc",
            "abc10",
            "h264",
            "valid-anchor",
        ] {
            assert!(!MDBOOK007::is_range_start(first), "{first:?}");
        }
    }

    /// #498: an anchor name of three characters or less was flagged as a
    /// mistyped line number. mdBook treats anything that is not a valid line
    /// range as an anchor, so these are valid includes.
    #[test]
    fn test_mdbook007_short_anchor_names_are_valid() -> mdbook_lint_core::error::Result<()> {
        for anchor in ["a", "ab", "abc", "abcd"] {
            let temp_dir = TempDir::new()?;
            let root = temp_dir.path();

            let target_content = format!("# ANCHOR: {anchor}\necho x\n# ANCHOR_END: {anchor}\n");
            create_test_document(&target_content, &root.join("inc.sh"))?;

            let source_content = format!("# Chapter 1\n\n{{{{#include inc.sh:{anchor}}}}}\n");
            let doc = create_test_document(&source_content, &root.join("chapter.md"))?;

            let violations = MDBOOK007::default().check(&doc)?;

            assert_eq!(
                violations.len(),
                0,
                "anchor {anchor:?} should be a valid include, got {violations:?}"
            );
        }
        Ok(())
    }

    /// A short anchor that is genuinely absent must still be reported, and as a
    /// missing anchor rather than as a malformed line number.
    #[test]
    fn test_mdbook007_short_anchor_name_missing_is_reported() -> mdbook_lint_core::error::Result<()>
    {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        create_test_document(
            "# ANCHOR: abc\necho x\n# ANCHOR_END: abc\n",
            &root.join("inc.sh"),
        )?;

        let source_content = r#"# Chapter 1

{{#include inc.sh:zzz}}
"#;
        let doc = create_test_document(source_content, &root.join("chapter.md"))?;

        let violations = MDBOOK007::default().check(&doc)?;

        assert_eq!(violations.len(), 1);
        assert!(
            violations[0].message.contains("Anchor 'zzz' not found"),
            "expected a missing-anchor message, got {:?}",
            violations[0].message
        );
        Ok(())
    }

    /// #499: mdBook does not process an escaped directive, it renders the
    /// literal text, so the file it names need not exist.
    #[test]
    fn test_mdbook007_escaped_include_is_ignored() -> mdbook_lint_core::error::Result<()> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        let source_content = r#"# Chapter 1

Write an include like this:

\{{#include ../file.md:name}}
"#;
        let doc = create_test_document(source_content, &root.join("chapter.md"))?;

        let violations = MDBOOK007::default().check(&doc)?;

        assert_eq!(
            violations.len(),
            0,
            "escaped include should not be validated, got {violations:?}"
        );
        Ok(())
    }

    /// mdBook's escape pattern is greedy to the last `}}` on the line, so a
    /// directive after an escaped one is rendered as literal text too.
    #[test]
    fn test_mdbook007_escape_swallows_the_rest_of_the_line() -> mdbook_lint_core::error::Result<()>
    {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        let source_content =
            "# Chapter 1\n\nEscaped \\{{#include escaped.md}} and real {{#include missing.md}}\n";
        let doc = create_test_document(source_content, &root.join("chapter.md"))?;

        let violations = MDBOOK007::default().check(&doc)?;

        assert!(violations.is_empty(), "got {violations:?}");
        Ok(())
    }

    /// A real directive before an escaped one is processed normally.
    #[test]
    fn test_mdbook007_real_include_before_escape_is_checked() -> mdbook_lint_core::error::Result<()>
    {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        let source_content =
            "# Chapter 1\n\nReal {{#include missing.md}} then \\{{#include escaped.md}}\n";
        let doc = create_test_document(source_content, &root.join("chapter.md"))?;

        let violations = MDBOOK007::default().check(&doc)?;

        assert_eq!(violations.len(), 1, "got {violations:?}");
        assert!(
            violations[0].message.contains("missing.md"),
            "got {:?}",
            violations[0].message
        );
        Ok(())
    }

    /// Lint `spec` against a 30-line target containing a few anchors.
    fn check_spec(spec: &str) -> mdbook_lint_core::error::Result<Vec<Violation>> {
        let temp_dir = TempDir::new()?;
        let root = temp_dir.path();

        let mut target = String::new();
        for anchor in ["example1", "step2", "h264", "abc"] {
            target.push_str(&format!(
                "// ANCHOR: {anchor}\nx\n// ANCHOR_END: {anchor}\n"
            ));
        }
        target.push_str("-- ANCHOR: sql_query\nSELECT 1;\n-- ANCHOR_END: sql_query\n");
        target.push_str("/* ANCHOR: css_rule */\nbody {}\n/* ANCHOR_END: css_rule */\n");
        target.push_str("#ANCHOR:tight\nx\n#ANCHOR_END:tight\n");
        while target.lines().count() < 30 {
            target.push_str("filler\n");
        }
        assert_eq!(target.lines().count(), 30, "fixture length drifted");
        create_test_document(&target, &root.join("inc.rs"))?;

        let source = format!("# Chapter\n\n{{{{#include inc.rs:{spec}}}}}\n");
        let doc = create_test_document(&source, &root.join("chapter.md"))?;
        MDBOOK007::default().check(&doc)
    }

    /// Specs mdBook resolves to the intended content must not be reported.
    #[test]
    fn test_mdbook007_accepts_every_spec_mdbook_resolves() -> mdbook_lint_core::error::Result<()> {
        for spec in [
            // Anchor names containing digits.
            "example1",
            "step2",
            "h264",
            // Open and full ranges.
            "10:",
            ":10",
            ":",
            "3:5",
            "7",
            // Only the first segment names the anchor.
            "abc:123",
            // Anchors in other comment syntaxes, and with no space after the colon.
            "sql_query",
            "css_rule",
            "tight",
        ] {
            let violations = check_spec(spec)?;
            assert!(
                violations.is_empty(),
                "{spec:?} should be valid, got {violations:?}"
            );
        }
        Ok(())
    }

    /// Specs that include nothing, or the wrong lines, are still reported.
    #[test]
    fn test_mdbook007_reports_specs_that_include_the_wrong_lines()
    -> mdbook_lint_core::error::Result<()> {
        for (spec, expected) in [
            ("0", "Invalid start line number '0'"),
            ("5:0", "Invalid end line number '0'"),
            ("10:5", "Start line 10 cannot be greater than end line 5"),
            ("31", "Line 31 does not exist"),
            ("31:", "starts past the end of the file"),
            ("25:35", "exceeds file length"),
            (":35", "exceeds file length"),
            ("10:abc", "includes from line 10 to the end of the file"),
            ("nosuch", "Anchor 'nosuch' not found"),
        ] {
            let violations = check_spec(spec)?;
            assert_eq!(violations.len(), 1, "{spec:?}: got {violations:?}");
            assert!(
                violations[0].message.contains(expected),
                "{spec:?}: expected {expected:?}, got {:?}",
                violations[0].message
            );
        }
        Ok(())
    }

    /// Anchor names are compared exactly, not as substrings.
    #[test]
    fn test_mdbook007_anchor_match_is_exact() -> mdbook_lint_core::error::Result<()> {
        // `ab` and `step` are prefixes of anchors that exist, but are not anchors.
        for spec in ["ab", "step"] {
            let violations = check_spec(spec)?;
            assert_eq!(violations.len(), 1, "{spec:?}: got {violations:?}");
            assert!(violations[0].message.contains("not found"), "{spec:?}");
        }
        Ok(())
    }
}
