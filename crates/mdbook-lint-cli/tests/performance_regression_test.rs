//! Performance regression tests for rules that had performance issues
//!
//! These tests ensure that previously identified performance problems don't regress.
//! They use content patterns that previously caused O(n²) complexity or infinite loops.

use mdbook_lint_core::{Document, Rule};
use mdbook_lint_rulesets::standard::{md049::MD049, md051::MD051};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Helper to create a test document
fn create_test_document(content: &str) -> Document {
    Document::new(content.to_string(), PathBuf::from("test.md")).unwrap()
}

/// Helper to assert that a rule completes within a reasonable time (100ms)
fn assert_completes_quickly<R: Rule>(rule: &R, document: &Document, max_duration: Duration) {
    let start = Instant::now();
    let result = rule.check(document);
    let elapsed = start.elapsed();

    assert!(
        elapsed < max_duration,
        "Rule {} took {:?} (max allowed: {:?})",
        rule.id(),
        elapsed,
        max_duration
    );

    // Also ensure the rule didn't error
    assert!(result.is_ok(), "Rule check failed: {:?}", result);
}

#[test]
fn test_md051_large_html_content_performance() {
    // Create a document with lots of HTML that previously caused O(n²) behavior
    let mut content = String::from("# Test Document\n\n");

    // Add many HTML elements with id attributes
    for i in 0..100 {
        let section = format!(
            r##"<div id="section-{}" class="content">
<p id="para-{}-1">Content here</p>
<span id="span-{}-1">More content</span>
<a name="anchor-{}" href="#top">Link</a>
<h2 id="heading-{}">Subsection {}</h2>
<div id="nested-{}" data-value="test">
    <p id="deep-{}-1">Nested paragraph</p>
    <span id="deep-{}-2">Nested span</span>
</div>
</div>

"##,
            i, i, i, i, i, i, i, i, i
        );
        content.push_str(&section);
    }

    // Add some markdown content with fragment links
    content.push_str("\n## Links\n\n");
    for i in 0..50 {
        let link1 = format!("- [Section {}](#section-{})\n", i, i);
        let link2 = format!("- [Heading {}](#heading-{})\n", i, i);
        content.push_str(&link1);
        content.push_str(&link2);
    }

    let document = create_test_document(&content);
    let rule = MD051::new();

    // This should complete in well under a second even with lots of HTML
    // Previously this would timeout due to O(n²) complexity
    assert_completes_quickly(&rule, &document, Duration::from_millis(500));
}

#[test]
fn test_md051_pathological_html_attributes() {
    // Test with HTML that has many attributes and complex patterns
    let mut content = String::from("# Document\n\n");

    // Create HTML with many attributes that the regex needs to parse
    for i in 0..50 {
        let html_block = format!(
            r##"<div class="class-{}" data-test="value" id="element-{}" style="color: red" onclick="handleClick()" data-id="{}" title="Element {}">
    <input type="text" name="field-{}" id="input-{}" value="test" placeholder="Enter text" required>
    <a href="#anchor-{}" name="link-{}" id="anchor-{}" class="link">Link text</a>
</div>
"##,
            i, i, i, i, i, i, i, i, i
        );
        content.push_str(&html_block);
    }

    let document = create_test_document(&content);
    let rule = MD051::new();

    assert_completes_quickly(&rule, &document, Duration::from_millis(200));
}

#[test]
fn test_md049_code_spans_with_underscore_asterisk_patterns() {
    // This pattern previously caused infinite loops in MD049
    let content = r#"# Rust Methods

Use the following methods for overflow handling:

- Wrap in all modes with the `wrapping_*` methods, such as `wrapping_add`.
- Return the `None` value if there is overflow with the `checked_*` methods.
- Return the value and a boolean with the `overflowing_*` methods.
- Saturate at min/max values with the `saturating_*` methods.

You can also use patterns like `impl_*` or `trait_*` in macros.

The `*_mut` and `*_ref` patterns are common in Rust:
- `as_mut`, `as_ref`
- `get_mut`, `get_ref`  
- `iter_mut`, `into_iter`

Some more examples: `Box<dyn Any + '_>`, `&'_ str`, `*const T`, `*mut T`.

And here's some actual *emphasized text* and _also emphasized_ text.
"#;

    let document = create_test_document(content);
    let rule = MD049::new();

    // This should complete instantly, not hang
    assert_completes_quickly(&rule, &document, Duration::from_millis(100));
}

#[test]
fn test_md049_many_code_spans_performance() {
    // Test with many inline code spans to ensure performance stays good
    let mut content = String::from("# Document\n\n");

    for i in 0..100 {
        content.push_str(&format!(
            "Use `function_{}` with `param_*` and `result_{}`. ",
            i, i
        ));
        content.push_str(&format!("The `check_{}` validates `input_*` patterns. ", i));
        content.push_str(&format!("Call `wrapper_{}` for `*_ptr` handling.\n", i));
    }

    // Add some real emphasis to ensure it still works
    content.push_str("\nThis has *real emphasis* and _also this_.\n");

    let document = create_test_document(&content);
    let rule = MD049::new();

    assert_completes_quickly(&rule, &document, Duration::from_millis(100));
}

#[test]
fn test_md049_nested_backticks_and_emphasis() {
    // Test complex nesting scenarios
    let content = r#"# Complex Patterns

Here's a line with `code containing * and _` mixed with *real emphasis*.

Multiple backticks: ``usage: `command_*` `` and then _emphasis_.

Triple backticks inline: ```not a code block but `inline` ``` with *emphasis*.

Edge cases:
- `_underscore_start`
- `asterisk_end*`
- `*both*_mixed_*`
- `_*_*_*_` (pathological but valid)

And normal text with *proper emphasis* and _underscored emphasis_.
"#;

    let document = create_test_document(content);
    let rule = MD049::new();

    assert_completes_quickly(&rule, &document, Duration::from_millis(100));
}

#[test]
fn test_combined_performance_stress_test() {
    // Test both rules on a document that combines problematic patterns
    let mut content = String::from("# Combined Stress Test\n\n");

    // Add HTML content that stressed MD051
    for i in 0..50 {
        let html_section = format!(
            r##"<section id="section-{}" class="main">
<h2 id="header-{}">Section {}</h2>
</section>
"##,
            i, i, i
        );
        content.push_str(&html_section);
    }

    // Add code span patterns that stressed MD049
    content.push_str("\n## Code Patterns\n\n");
    for i in 0..50 {
        content.push_str(&format!(
            "Use `method_{}` with `pattern_*` and `suffix_{}`. ",
            i, i
        ));
    }

    // Add fragment links
    content.push_str("\n## Links\n\n");
    for i in 0..25 {
        let link = format!("- [Link to section {}](#section-{})\n", i, i);
        content.push_str(&link);
    }

    let document = create_test_document(&content);

    // Test MD051
    let md051 = MD051::new();
    assert_completes_quickly(&md051, &document, Duration::from_millis(300));

    // Test MD049
    let md049 = MD049::new();
    assert_completes_quickly(&md049, &document, Duration::from_millis(100));
}
