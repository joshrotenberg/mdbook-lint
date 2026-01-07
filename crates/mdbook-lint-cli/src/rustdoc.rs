//! Rustdoc comment extraction for linting
//!
//! This module extracts module-level documentation comments (`//!`) from Rust source files
//! and converts them to markdown for linting.

use std::path::Path;

/// Represents extracted documentation from a Rust source file
#[derive(Debug, Clone)]
pub struct ExtractedDoc {
    /// The markdown content extracted from doc comments
    pub content: String,
    /// The starting line number in the original file (1-based)
    pub start_line: usize,
}

/// Extract module-level documentation (`//!` comments) from Rust source content
///
/// Returns `None` if no module-level documentation is found.
///
/// # Arguments
/// * `content` - The Rust source file content
///
/// # Example
/// ```
/// use mdbook_lint::rustdoc::extract_module_docs;
///
/// let content = r#"//! # My Crate
/// //!
/// //! This is the module documentation.
///
/// fn main() {}
/// "#;
///
/// let doc = extract_module_docs(content).unwrap();
/// assert!(doc.content.contains("# My Crate"));
/// assert_eq!(doc.start_line, 1);
/// ```
pub fn extract_module_docs(content: &str) -> Option<ExtractedDoc> {
    let mut doc_lines = Vec::new();
    let mut start_line = None;
    let mut in_doc_block = false;

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("//!") {
            if start_line.is_none() {
                start_line = Some(i + 1); // 1-based line numbers
                in_doc_block = true;
            }

            // Extract the content after "//!"
            let doc_content = trimmed.strip_prefix("//!").unwrap_or("");
            // Remove at most one leading space (standard rustdoc formatting)
            let doc_content = doc_content.strip_prefix(' ').unwrap_or(doc_content);
            doc_lines.push(doc_content.to_string());
        } else if in_doc_block {
            // We've hit non-doc content after starting a doc block
            // Empty lines between doc comments are okay, but code ends the block
            if trimmed.is_empty() {
                // Could be a blank line within docs, but we're conservative
                // and end on first non-doc, non-empty line
                continue;
            }
            // End of module doc block
            break;
        }
    }

    if doc_lines.is_empty() {
        return None;
    }

    Some(ExtractedDoc {
        content: doc_lines.join("\n"),
        start_line: start_line.unwrap_or(1),
    })
}

/// Recursively find all Rust source files in a directory
pub fn find_rust_files(path: &Path) -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();

    if path.is_file() {
        if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            files.push(path.to_path_buf());
        }
        return Ok(files);
    }

    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            // Skip hidden directories and target
            if let Some(name) = entry_path.file_name().and_then(|n| n.to_str())
                && (name.starts_with('.') || name == "target")
            {
                continue;
            }

            if entry_path.is_dir() {
                files.extend(find_rust_files(&entry_path)?);
            } else if entry_path.extension().and_then(|e| e.to_str()) == Some("rs") {
                files.push(entry_path);
            }
        }
    }

    Ok(files)
}

/// Map a line number from extracted markdown back to the original Rust file
///
/// # Arguments
/// * `markdown_line` - Line number in the extracted markdown (1-based)
/// * `doc_start_line` - Starting line of the doc block in the Rust file (1-based)
///
/// # Returns
/// The corresponding line number in the original Rust file
pub fn map_line_to_source(markdown_line: usize, doc_start_line: usize) -> usize {
    // markdown_line is 1-based within the extracted content
    // doc_start_line is 1-based in the source file
    // Line 1 of markdown corresponds to doc_start_line in source
    doc_start_line + markdown_line - 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_module_docs() {
        let content = r#"//! # My Crate
//!
//! This is the module documentation.

fn main() {}
"#;

        let doc = extract_module_docs(content).unwrap();
        assert_eq!(doc.start_line, 1);
        assert!(doc.content.contains("# My Crate"));
        assert!(doc.content.contains("This is the module documentation."));
    }

    #[test]
    fn test_extract_docs_with_code_blocks() {
        let content = r#"//! # Example
//!
//! ```rust
//! let x = 1;
//! ```

use std::io;
"#;

        let doc = extract_module_docs(content).unwrap();
        assert!(doc.content.contains("```rust"));
        assert!(doc.content.contains("let x = 1;"));
    }

    #[test]
    fn test_no_module_docs() {
        let content = r#"/// This is an item doc, not module doc
fn foo() {}
"#;

        assert!(extract_module_docs(content).is_none());
    }

    #[test]
    fn test_docs_not_at_start() {
        let content = r#"// Regular comment
//! Module doc starts here
//! More docs

fn main() {}
"#;

        let doc = extract_module_docs(content).unwrap();
        assert_eq!(doc.start_line, 2);
        assert!(doc.content.contains("Module doc starts here"));
    }

    #[test]
    fn test_line_mapping() {
        // If doc starts at line 5 in source, and we have a violation on line 3 of markdown,
        // that corresponds to line 7 in the source (5 + 3 - 1 = 7)
        assert_eq!(map_line_to_source(1, 5), 5);
        assert_eq!(map_line_to_source(3, 5), 7);
        assert_eq!(map_line_to_source(1, 1), 1);
    }

    #[test]
    fn test_preserves_indentation() {
        let content = r#"//! # Heading
//!
//! - Item 1
//!   - Nested item
//!     - Deeply nested
"#;

        let doc = extract_module_docs(content).unwrap();
        assert!(doc.content.contains("  - Nested item"));
        assert!(doc.content.contains("    - Deeply nested"));
    }

    #[test]
    fn test_empty_doc_lines() {
        let content = r#"//! # Title
//!
//! Paragraph after blank line.
"#;

        let doc = extract_module_docs(content).unwrap();
        let lines: Vec<_> = doc.content.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "# Title");
        assert_eq!(lines[1], "");
        assert_eq!(lines[2], "Paragraph after blank line.");
    }
}
