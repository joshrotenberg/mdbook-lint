use crate::error::{MdBookLintError, Result};
use comrak::nodes::{AstNode, NodeValue};
use comrak::{parse_document, Arena, ComrakOptions};
use std::path::PathBuf;

/// Represents a parsed markdown document with position information
#[derive(Debug)]
pub struct Document {
    /// The original markdown content
    pub content: String,
    /// Path to the source file
    pub path: PathBuf,
    /// Lines split for line-based rule processing
    pub lines: Vec<String>,
}

impl Document {
    /// Parse a markdown document from content and path
    pub fn new(content: String, path: PathBuf) -> Result<Self> {
        // Allow empty documents for edge case handling
        // Some rules need to handle empty files correctly

        // Split content into lines for line-based rules
        let lines: Vec<String> = content.lines().map(|s| s.to_owned()).collect();

        Ok(Document {
            content,
            path,
            lines,
        })
    }

    /// Parse the content into a comrak AST
    pub fn parse_ast<'a>(&self, arena: &'a Arena<AstNode<'a>>) -> &'a AstNode<'a> {
        // Configure comrak options for position tracking and compatibility
        let mut options = ComrakOptions::default();
        options.extension.strikethrough = true;
        options.extension.tagfilter = false;
        options.extension.table = true;
        options.extension.autolink = true;
        options.extension.tasklist = true;
        options.extension.superscript = false;
        options.extension.header_ids = None;
        options.extension.footnotes = true;
        options.extension.description_lists = true;
        options.extension.front_matter_delimiter = Some("---".to_owned());
        options.parse.smart = false;
        options.parse.default_info_string = None;
        options.parse.relaxed_tasklist_matching = false;
        options.parse.relaxed_autolinks = false;

        parse_document(arena, &self.content, &options)
    }

    /// Parse AST with error context
    pub fn parse_ast_with_context<'a>(
        &self,
        arena: &'a Arena<AstNode<'a>>,
    ) -> Result<&'a AstNode<'a>> {
        // For now, comrak parsing doesn't typically fail, but we can add validation
        let ast = self.parse_ast(arena);

        // Basic validation that we got a valid AST
        if ast.children().count() == 0 && !self.content.trim().is_empty() {
            return Err(MdBookLintError::Document(
                "Failed to parse document AST - no content nodes found".to_string(),
            ));
        }

        Ok(ast)
    }

    /// Get the line number (1-based) for a given byte offset
    pub fn line_number_at_offset(&self, offset: usize) -> usize {
        let mut current_offset = 0;
        for (line_idx, line) in self.lines.iter().enumerate() {
            if current_offset + line.len() >= offset {
                return line_idx + 1; // 1-based line numbers
            }
            current_offset += line.len() + 1; // +1 for newline
        }
        self.lines.len() // Return last line if offset is at end
    }

    /// Get the column number (1-based) for a given byte offset
    pub fn column_number_at_offset(&self, offset: usize) -> usize {
        let mut current_offset = 0;
        for line in &self.lines {
            if current_offset + line.len() >= offset {
                return offset - current_offset + 1; // 1-based column numbers
            }
            current_offset += line.len() + 1; // +1 for newline
        }
        1 // Default to column 1
    }

    /// Get all heading nodes from the AST
    pub fn headings<'a>(&self, ast: &'a AstNode<'a>) -> Vec<&'a AstNode<'a>> {
        let mut headings = Vec::new();
        self.collect_headings(ast, &mut headings);
        headings
    }

    /// Get all heading nodes with error context
    pub fn headings_with_context<'a>(&self, ast: &'a AstNode<'a>) -> Result<Vec<&'a AstNode<'a>>> {
        let headings = self.headings(ast);
        Ok(headings)
    }

    /// Get all code block nodes from the AST
    pub fn code_blocks<'a>(&self, ast: &'a AstNode<'a>) -> Vec<&'a AstNode<'a>> {
        let mut code_blocks = Vec::new();
        self.collect_code_blocks(ast, &mut code_blocks);
        code_blocks
    }

    /// Get all code block nodes with error context
    pub fn code_blocks_with_context<'a>(
        &self,
        ast: &'a AstNode<'a>,
    ) -> Result<Vec<&'a AstNode<'a>>> {
        let code_blocks = self.code_blocks(ast);
        Ok(code_blocks)
    }

    /// Recursively collect heading nodes
    #[allow(clippy::only_used_in_recursion)]
    fn collect_headings<'a>(&self, node: &'a AstNode<'a>, result: &mut Vec<&'a AstNode<'a>>) {
        if let NodeValue::Heading(..) = &node.data.borrow().value {
            result.push(node)
        }

        // Recursively check children
        for child in node.children() {
            self.collect_headings(child, result);
        }
    }

    /// Recursively collect code block nodes
    #[allow(clippy::only_used_in_recursion)]
    fn collect_code_blocks<'a>(&self, node: &'a AstNode<'a>, result: &mut Vec<&'a AstNode<'a>>) {
        if let NodeValue::CodeBlock(..) = &node.data.borrow().value {
            result.push(node)
        }

        // Recursively check children
        for child in node.children() {
            self.collect_code_blocks(child, result);
        }
    }

    /// Get the heading level for a heading node
    pub fn heading_level<'a>(node: &'a AstNode<'a>) -> Option<u32> {
        match &node.data.borrow().value {
            NodeValue::Heading(heading) => Some(heading.level.into()),
            _ => None,
        }
    }

    /// Get the text content of a node
    pub fn node_text<'a>(&self, node: &'a AstNode<'a>) -> String {
        let mut text = String::new();
        self.collect_text(node, &mut text);
        text
    }

    /// Recursively collect text from a node and its children
    #[allow(clippy::only_used_in_recursion)]
    fn collect_text<'a>(&self, node: &'a AstNode<'a>, text: &mut String) {
        match &node.data.borrow().value {
            NodeValue::Text(t) => text.push_str(t),
            NodeValue::Code(code) => text.push_str(&code.literal),
            _ => {
                for child in node.children() {
                    self.collect_text(child, text);
                }
            }
        }
    }

    /// Get the source position of a node
    pub fn node_position<'a>(&self, node: &'a AstNode<'a>) -> Option<(usize, usize)> {
        let sourcepos = node.data.borrow().sourcepos;
        if sourcepos.start.line > 0 {
            Some((sourcepos.start.line, sourcepos.start.column))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use comrak::Arena;
    use std::path::PathBuf;

    #[test]
    fn test_document_creation() {
        let content = "# Test\n\nThis is a test.".to_string();
        let path = PathBuf::from("test.md");

        let doc = Document::new(content, path).expect("Failed to create document");

        assert_eq!(doc.lines.len(), 3);
        assert_eq!(doc.lines[0], "# Test");
        assert_eq!(doc.lines[1], "");
        assert_eq!(doc.lines[2], "This is a test.");
    }

    #[test]
    fn test_empty_document_allowed() {
        let content = "".to_string();
        let path = PathBuf::from("empty.md");

        let result = Document::new(content, path);
        assert!(result.is_ok());

        let document = result.unwrap();
        assert_eq!(document.content, "");
        assert_eq!(document.lines.len(), 0);
        assert_eq!(document.path, PathBuf::from("empty.md"));
    }

    #[test]
    fn test_whitespace_only_document_allowed() {
        let content = "   \n  \n  ".to_string();
        let path = PathBuf::from("whitespace.md");

        let result = Document::new(content, path);
        assert!(result.is_ok());

        let document = result.unwrap();
        assert_eq!(document.content, "   \n  \n  ");
        assert_eq!(document.lines.len(), 3);
        assert_eq!(document.path, PathBuf::from("whitespace.md"));
    }

    #[test]
    fn test_line_number_calculation() {
        let content = "Line 1\nLine 2\nLine 3".to_string();
        let path = PathBuf::from("test.md");

        let doc = Document::new(content, path).expect("Failed to create document");

        assert_eq!(doc.line_number_at_offset(0), 1); // Start of line 1
        assert_eq!(doc.line_number_at_offset(7), 2); // Start of line 2
        assert_eq!(doc.line_number_at_offset(14), 3); // Start of line 3
    }

    #[test]
    fn test_heading_extraction() {
        let content = "# H1\n## H2\n### H3\nText".to_string();
        let path = PathBuf::from("test.md");

        let doc = Document::new(content, path).expect("Failed to create document");
        let arena = Arena::new();
        let ast = doc.parse_ast(&arena);
        let headings = doc.headings(ast);

        assert_eq!(headings.len(), 3);

        assert_eq!(Document::heading_level(headings[0]), Some(1));
        assert_eq!(Document::heading_level(headings[1]), Some(2));
        assert_eq!(Document::heading_level(headings[2]), Some(3));
    }

    #[test]
    fn test_document_with_unicode() {
        let content = "# 标题\n\n这是一个测试。\n\n```rust\nfn main() {\n    println!(\"Hello, 世界!\");\n}\n```".to_string();
        let path = PathBuf::from("unicode.md");

        let doc = Document::new(content, path).expect("Failed to create document");
        assert!(doc.content.contains("标题"));
        assert!(doc.content.contains("世界"));
        assert_eq!(doc.lines.len(), 9);
    }

    #[test]
    fn test_document_with_very_long_lines() {
        let long_line = "a".repeat(10000);
        let content = format!("# Test\n\n{}\n\nEnd", long_line);
        let path = PathBuf::from("long.md");

        let doc = Document::new(content, path).expect("Failed to create document");
        assert_eq!(doc.lines.len(), 5);
        assert_eq!(doc.lines[2].len(), 10000);
    }

    #[test]
    fn test_document_with_mixed_line_endings() {
        let content = "Line 1\r\nLine 2\nLine 3\r\nLine 4".to_string();
        let path = PathBuf::from("mixed.md");

        let doc = Document::new(content, path).expect("Failed to create document");
        // The lines() method normalizes line endings
        assert_eq!(doc.lines.len(), 4);
        assert_eq!(doc.lines[0], "Line 1");
        assert_eq!(doc.lines[1], "Line 2");
        assert_eq!(doc.lines[2], "Line 3");
        assert_eq!(doc.lines[3], "Line 4");
    }

    #[test]
    fn test_document_with_only_newlines() {
        let content = "\n\n\n\n".to_string();
        let path = PathBuf::from("newlines.md");

        let doc = Document::new(content, path).expect("Failed to create document");
        assert_eq!(doc.lines.len(), 4);
        for line in &doc.lines {
            assert_eq!(line, "");
        }
    }

    #[test]
    fn test_node_position_edge_cases() {
        let content = "# Test\n\n```rust\ncode\n```\n\n- Item".to_string();
        let path = PathBuf::from("test.md");

        let doc = Document::new(content, path).expect("Failed to create document");
        let arena = Arena::new();
        let ast = doc.parse_ast(&arena);

        // Test position for first node
        let position = doc.node_position(ast);
        assert!(position.is_some());
        let (line, col) = position.unwrap();
        assert!(line >= 1);
        assert!(col >= 1);
    }

    #[test]
    fn test_code_blocks_extraction() {
        let content = r#"# Test

```rust
fn main() {}
```

Some text.

```bash
echo "hello"
```

    // Indented code block
    let x = 5;

```
No language
```
"#
        .to_string();
        let path = PathBuf::from("test.md");

        let doc = Document::new(content, path).expect("Failed to create document");
        let arena = Arena::new();
        let ast = doc.parse_ast(&arena);
        let code_blocks = doc.code_blocks(ast);

        // Should find both fenced and indented code blocks
        assert!(code_blocks.len() >= 3);
    }

    #[test]
    fn test_links_extraction() {
        let content = r#"# Test

[Link 1](http://example.com)

[Link 2](./relative.md)

<https://autolink.com>

[Reference link][ref]

[ref]: http://reference.com
"#
        .to_string();
        let path = PathBuf::from("test.md");

        let doc = Document::new(content, path).expect("Failed to create document");
        let arena = Arena::new();
        let ast = doc.parse_ast(&arena);

        // Test that document parsing succeeds and produces valid AST
        assert!(!doc.content.is_empty());
        assert!(doc.lines.len() > 0);

        // Verify AST structure contains expected elements
        let mut has_heading = false;
        for node in ast.descendants() {
            if matches!(node.data.borrow().value, comrak::nodes::NodeValue::Heading(_)) {
                has_heading = true;
                break;
            }
        }
        assert!(has_heading, "Expected to find heading in parsed AST");
    }

    #[test]
    fn test_line_number_at_offset_edge_cases() {
        let content = "a\nb\nc".to_string();
        let path = PathBuf::from("test.md");

        let doc = Document::new(content, path).expect("Failed to create document");

        // Test offset beyond content
        let line = doc.line_number_at_offset(100);
        assert!(line >= 1);

        // Test offset at exact line boundaries
        assert_eq!(doc.line_number_at_offset(0), 1); // 'a'
        assert_eq!(doc.line_number_at_offset(2), 2); // 'b'
        assert_eq!(doc.line_number_at_offset(4), 3); // 'c'
    }

    #[test]
    fn test_ast_parsing_with_extensions() {
        let content = r#"# Test

| Table | Header |
|-------|--------|
| Cell  | Data   |

~~Strikethrough~~

- [x] Task done
- [ ] Task pending

^Super^script

[^footnote]: Footnote content

Front matter:
---
title: Test
---
"#
        .to_string();
        let path = PathBuf::from("test.md");

        let doc = Document::new(content, path).expect("Failed to create document");
        let arena = Arena::new();
        let ast = doc.parse_ast(&arena);

        // Verify document parses and contains expected markdown extensions
        assert!(doc.content.contains("~~Strikethrough~~"));
        assert!(doc.content.contains("| Table | Header |"));
        assert!(doc.content.contains("- [x] Task done"));
        assert!(doc.content.contains("title: Test"));

        // Verify AST parsing produces nodes (basic structure validation)
        let node_count = ast.descendants().count();
        assert!(node_count > 5, "Expected AST to contain multiple nodes, got {}", node_count);
    }

    #[test]
    fn test_empty_path() {
        let content = "# Test".to_string();
        let path = PathBuf::new();

        let doc = Document::new(content, path).expect("Failed to create document");
        assert_eq!(doc.path, PathBuf::new());
    }

    #[test]
    fn test_complex_nested_structure() {
        let content = r#"# Main Title

## Section 1

### Subsection

Some text with **bold** and *italic*.

> Blockquote with `code` inside.
>
> > Nested blockquote.

1. Ordered list
   - Nested unordered
   - Another item
2. Second ordered item

```rust
// Code with comments
fn complex_function() {
    println!("Complex: {}", "test");
}
```

Final paragraph.
"#
        .to_string();
        let path = PathBuf::from("complex.md");

        let doc = Document::new(content, path).expect("Failed to create document");
        let arena = Arena::new();
        let ast = doc.parse_ast(&arena);

        // Test various extractions work
        let headings = doc.headings(ast);
        assert!(headings.len() >= 3);

        let code_blocks = doc.code_blocks(ast);
        assert!(code_blocks.len() >= 1);

        // Verify the document contains expected content structure
        assert!(doc.content.contains("# Main Title"));
        assert!(doc.content.contains("## Section 1"));
        assert!(doc.content.contains("### Subsection"));
        assert!(doc.content.contains("```rust"));
        assert!(doc.content.contains("Final paragraph"));

        // Verify line structure is correct
        assert!(doc.lines.len() > 10, "Expected multiple lines in complex document");
    }
}
