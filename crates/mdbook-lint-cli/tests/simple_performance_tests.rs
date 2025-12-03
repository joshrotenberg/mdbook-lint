//! Simplified performance regression tests for mdbook-lint
//!
//! This module provides focused performance testing to catch regressions
//! without the complexity of the previous corpus testing framework.

use mdbook_lint::{Config, Document, LintEngine, PluginRegistry};
use mdbook_lint_rulesets::{MdBookRuleProvider, StandardRuleProvider};
use std::time::{Duration, Instant};

/// Create a lint engine for testing
fn create_lint_engine() -> LintEngine {
    let mut registry = PluginRegistry::new();
    registry
        .register_provider(Box::new(StandardRuleProvider))
        .unwrap();
    registry
        .register_provider(Box::new(MdBookRuleProvider))
        .unwrap();
    registry.create_engine().unwrap()
}

/// Create a test document from content
fn create_test_document(content: &str) -> Document {
    Document::new(content.to_string(), "test.md".into()).unwrap()
}

/// Assert that a linting operation completes within the given duration
fn assert_completes_quickly(
    engine: &LintEngine,
    document: &Document,
    max_duration: Duration,
    description: &str,
) {
    let start = Instant::now();
    let result = engine.lint_document_with_config(document, &Config::default().core);
    let elapsed = start.elapsed();

    println!("  {} took {:?}", description, elapsed);

    assert!(
        elapsed < max_duration,
        "{} took {:?} but should complete within {:?}",
        description,
        elapsed,
        max_duration
    );

    // Ensure we actually get results (not just quick failures)
    assert!(result.is_ok(), "Linting should not fail: {:?}", result);
}

#[test]
fn test_performance_regression_md051_html_fragments() {
    let engine = create_lint_engine();

    // Test case that previously caused O(n²) behavior in MD051
    let html_content = r##"# Test Document

This document tests MD051 performance with HTML fragments.

<a href="#section1">Link 1</a>
<a href="#section2">Link 2</a>
<a href="#section3">Link 3</a>
<a href="#section4">Link 4</a>
<a href="#section5">Link 5</a>

## Section 1 {#section1}
Content here.

## Section 2 {#section2}
More content.

## Section 3 {#section3}
Even more content.

## Section 4 {#section4}
Content continues.

## Section 5 {#section5}
Final content.
"##;

    let document = create_test_document(html_content);
    assert_completes_quickly(
        &engine,
        &document,
        Duration::from_millis(150),
        "MD051 HTML fragment test",
    );
}

#[test]
fn test_performance_regression_md049_emphasis_patterns() {
    let engine = create_lint_engine();

    // Test case that previously caused infinite loops in MD049
    let emphasis_content = r##"# Test Document

This tests MD049 with patterns that previously caused issues:

- `wrapping_*` function calls in code
- `checked_*` operations in inline code
- `saturating_*` arithmetic in backticks
- Normal *emphasis* outside code should work
- Multiple `code_*` patterns in the same line
- Mixed patterns: `first_*` and *emphasis* and `second_*`

```rust
// Code blocks should not be affected
fn wrapping_add(a: u32, b: u32) -> u32 {
    a.wrapping_add(b)
}
```

More text with `inline_*` patterns and *real emphasis*.
"##;

    let document = create_test_document(emphasis_content);
    assert_completes_quickly(
        &engine,
        &document,
        Duration::from_millis(50),
        "MD049 emphasis pattern test",
    );
}

#[test]
fn test_performance_large_document() {
    let engine = create_lint_engine();

    // Generate a reasonably large document
    let mut large_content = String::with_capacity(50_000);
    large_content.push_str("# Large Document Performance Test\n\n");

    // Add many headings and content
    for i in 1..=200 {
        large_content.push_str(&format!(
            "## Section {}\n\nThis is content for section {}. It contains some *emphasis* and `code` examples.\n\n",
            i, i
        ));

        // Add some lists
        for j in 1..=5 {
            large_content.push_str(&format!("- List item {} in section {}\n", j, i));
        }
        large_content.push('\n');

        // Add some code blocks occasionally
        if i % 10 == 0 {
            large_content.push_str(&format!(
                "```rust\n// Code example for section {}\nfn example_{}() {{\n    println!(\"Section {}\");\n}}\n```\n\n",
                i, i, i
            ));
        }
    }

    let document = create_test_document(&large_content);
    println!("Testing document with {} characters", large_content.len());

    assert_completes_quickly(
        &engine,
        &document,
        Duration::from_millis(500),
        "Large document test",
    );
}

#[test]
fn test_performance_deeply_nested_content() {
    let engine = create_lint_engine();

    // Test deeply nested structures
    let mut nested_content = String::new();
    nested_content.push_str("# Deep Nesting Test\n\n");

    // Deep list nesting
    for level in 0..50 {
        let indent = "  ".repeat(level);
        nested_content.push_str(&format!(
            "{}* Level {} item with *emphasis*\n",
            indent,
            level + 1
        ));
    }

    nested_content.push('\n');

    // Deep blockquote nesting
    for level in 1..=20 {
        let prefix = "> ".repeat(level);
        nested_content.push_str(&format!("{}Quote at level {} with `code`\n", prefix, level));
    }

    let document = create_test_document(&nested_content);
    assert_completes_quickly(
        &engine,
        &document,
        Duration::from_millis(200),
        "Deep nesting test",
    );
}

#[test]
fn test_performance_many_violations() {
    let engine = create_lint_engine();

    // Document designed to trigger many violations but still be fast
    let violations_content = r#"# Test Document

### Skipped H2 (MD001 violation)

Line with trailing spaces
Another line with trailing spaces

```
Code block without language (MD040/MDBOOK001 violation)
some code here
```

This line is intentionally very long to trigger MD013 line length violations and should exceed the typical 80 character limit significantly

#  Multiple spaces after hash (MD018 violation)

##Heading without space (MD018 violation)

- List item 1
* Mixed list markers (MD004 violation)
- List item 3

[Bad link](

**Unclosed emphasis

Multiple blank lines below:



Above were multiple blank lines (MD012 violation)
"#;

    let document = create_test_document(violations_content);
    assert_completes_quickly(
        &engine,
        &document,
        Duration::from_millis(100),
        "Many violations test",
    );
}

#[test]
fn test_performance_pathological_input() {
    let engine = create_lint_engine();

    // Test various pathological inputs that might cause issues
    let pathological_inputs = [
        // Very long lines
        &format!("# Test\n\n{}", "x".repeat(10000)),
        // Many empty lines
        &format!("# Test\n{}", "\n".repeat(1000)),
        // Repeated patterns
        &"[link]".repeat(1000),
        // Mixed line endings (use raw strings to preserve)
        "# Test\r\nContent\nMore\r\nContent\n",
        // Unicode content
        "# Test\n\nこんにちは世界 🌍 مرحبا بالعالم עולם שלום",
        // Many nested emphasis
        &format!("{}text{}", "*".repeat(100), "*".repeat(100)),
    ];

    for (i, input) in pathological_inputs.iter().enumerate() {
        let document = create_test_document(input);
        assert_completes_quickly(
            &engine,
            &document,
            Duration::from_millis(300),
            &format!("Pathological input #{}", i + 1),
        );
    }
}
