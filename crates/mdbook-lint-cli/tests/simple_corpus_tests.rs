//! Simplified corpus testing with property testing for crash resistance
//!
//! This module provides focused correctness testing against real-world files
//! and property-based testing to ensure we never crash on any input.

use mdbook_lint::{Config, Document, LintEngine, PluginRegistry};
use mdbook_lint_rulesets::{MdBookRuleProvider, StandardRuleProvider};
use std::fs;
use std::path::Path;

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

/// Test that we can lint a file without crashing  
fn assert_no_crash(content: &str, description: &str) {
    // For now, just test that we don't crash without catch_unwind
    // The important property is that we handle all inputs gracefully
    let engine = create_lint_engine();

    match Document::new(content.to_string(), "test.md".into()) {
        Ok(document) => {
            let result = engine.lint_document_with_config(&document, &Config::default().core);
            match result {
                Ok(_violations) => {
                    // Success! We got violations (or no violations), both are fine
                }
                Err(e) => {
                    // Linting error is acceptable (e.g., invalid markdown), just don't crash
                    println!("  {} produced error (acceptable): {}", description, e);
                }
            }
        }
        Err(e) => {
            // Document creation error is acceptable for malformed input
            println!(
                "  {} failed document creation (acceptable): {}",
                description, e
            );
        }
    }
}

#[test]
fn test_our_own_documentation() {
    let engine = create_lint_engine();
    let config = Config::default();

    // Test against our own documentation files
    let test_files = ["README.md", "CLAUDE.md", "CONVENTIONS.md", "PROFILING.md"];

    for file_path in &test_files {
        if let Ok(content) = fs::read_to_string(file_path) {
            println!("Testing {}", file_path);

            let document = Document::new(content, file_path.into()).unwrap();
            let result = engine.lint_document_with_config(&document, &config.core);

            assert!(
                result.is_ok(),
                "Should be able to lint our own {} without errors: {:?}",
                file_path,
                result
            );

            let violations = result.unwrap();
            println!("  {} has {} violations", file_path, violations.len());

            // Our own docs shouldn't have major violations
            let error_count = violations
                .iter()
                .filter(|v| v.severity == mdbook_lint::Severity::Error)
                .count();

            assert!(
                error_count == 0,
                "{} should not have error-level violations: {:#?}",
                file_path,
                violations
                    .iter()
                    .filter(|v| v.severity == mdbook_lint::Severity::Error)
                    .collect::<Vec<_>>()
            );
        } else {
            println!("Skipping {} (not found)", file_path);
        }
    }
}

#[test]
fn test_essential_corpus_files() {
    let engine = create_lint_engine();
    let corpus_dir = Path::new("tests/corpus/essential");

    if !corpus_dir.exists() {
        println!("Essential corpus directory not found, skipping");
        return;
    }

    let mut files_tested = 0;

    for entry in fs::read_dir(corpus_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "md") {
            let content = fs::read_to_string(&path).unwrap();
            let document = Document::new(content, path.clone()).unwrap();

            println!("Testing corpus file: {:?}", path.file_name());

            let result = engine.lint_document_with_config(&document, &Config::default().core);
            assert!(
                result.is_ok(),
                "Should be able to lint {:?} without errors: {:?}",
                path,
                result
            );

            files_tested += 1;
        }
    }

    assert!(
        files_tested > 0,
        "Should have tested at least one corpus file"
    );
    println!("Tested {} essential corpus files", files_tested);
}

// Property-based testing for crash resistance
#[test]
fn test_never_crashes_on_random_input() {
    // Test with various types of random/invalid input
    let problematic_inputs = [
        // Empty and whitespace
        "",
        " ",
        "\n",
        "\t",
        "\r\n",
        "\n\n\n\n\n",
        // Invalid UTF-8 sequences (as valid UTF-8 representations)
        "Invalid \u{FFFD} replacement character",
        // Very long lines
        &"x".repeat(100_000),
        // Control characters
        "Control\x00chars\x01here\x02",
        // Unicode edge cases
        "Zero width\u{200B}space",
        "Right\u{202E}to left override",
        // Markdown edge cases
        "[unclosed link",
        "**unclosed emphasis",
        "`unclosed code",
        "```\nunclosed code block",
        // HTML fragments
        "<div>unclosed",
        "&invalid entity;",
        "<!-- unclosed comment",
        // Mixed line endings
        "Line 1\r\nLine 2\nLine 3\rLine 4",
        // Pathological nesting
        &format!("{}deep quote", "> ".repeat(1000)),
        &format!("{}deep list", "* ".repeat(1000)),
        &format!("{} deep heading", "#".repeat(1000)),
        // Binary-ish content
        "\x00\x01\x02\x03\x04\x05\x06\x07",
        // Large amounts of the same character
        &"*".repeat(10000),
        &"[".repeat(5000),
        &"`".repeat(1000),
        // Null bytes
        "Text with\x00null bytes\x00in it",
    ];

    for (i, input) in problematic_inputs.iter().enumerate() {
        assert_no_crash(input, &format!("Problematic input #{}", i + 1));
    }

    println!(
        "✅ Tested {} problematic inputs without crashes",
        problematic_inputs.len()
    );
}

#[test]
fn test_never_crashes_on_generated_fuzz_like_input() {
    // Generate various types of fuzz-like input
    let problematic_sequences = [
        "***", "___", "```", "[[[", "]]]", "(((", ")))", "###", ">>>", "---", "+++", "...", "   ",
        "\n\n", "*_*", "_*_", "`*`", "**_", "_**", "*`*", "`**", "[*]", "*[", "]*", "()",
    ];

    // Test combinations of problematic sequences
    for seq1 in &problematic_sequences {
        for seq2 in &problematic_sequences {
            let combined = format!("{}{}", seq1, seq2);
            assert_no_crash(&combined, &format!("Combined sequence: {}", combined));
        }
    }

    // Test repeated patterns
    for seq in &problematic_sequences {
        let repeated = seq.repeat(100);
        assert_no_crash(&repeated, &format!("Repeated sequence: {}x100", seq));
    }

    println!("✅ Tested fuzz-like generated inputs without crashes");
}

#[test]
fn test_never_crashes_on_malformed_markdown() {
    // Test specifically malformed markdown that might trip up parsers
    let malformed_cases = [
        // Malformed links
        "[text](url",
        "[text](url \"title",
        "[text][ref",
        "[ref]: url",
        "[ref]: url \"title",
        // Malformed emphasis
        "*unclosed emphasis",
        "**unclosed strong",
        "***unclosed strong emphasis",
        "_unclosed emphasis",
        "__unclosed strong",
        "___unclosed strong emphasis",
        // Malformed code
        "`unclosed inline code",
        "```\nunclosed fenced code",
        "~~~\nunclosed fenced code",
        "    indented code without newline",
        // Malformed lists
        "- item 1\n  - nested without parent completion",
        "1. ordered item\n2 malformed numbering",
        "* item\n  * nested\n    * deep\n  back to level 2 but malformed indent",
        // Malformed tables
        "| header |\n| missing |",
        "| header | another |\n| row |",
        // Malformed headings
        "#heading without space",
        "##   heading with multiple spaces",
        "# heading with trailing #s ##",
        // Malformed HTML
        "<div>unclosed tag",
        "<img src=\"unclosed attribute",
        "<!-- unclosed comment",
        "<script>alert('test')",
        // Mixed valid/invalid
        "# Valid Heading\n\n[invalid link\n\n**valid emphasis**\n\n`invalid code",
    ];

    for (i, case) in malformed_cases.iter().enumerate() {
        assert_no_crash(
            case,
            &format!("Malformed markdown #{}: {:.50}...", i + 1, case),
        );
    }

    println!(
        "✅ Tested {} malformed markdown cases without crashes",
        malformed_cases.len()
    );
}

#[test]
fn test_performance_and_correctness_on_known_files() {
    // Test against docs files if they exist
    let docs_dir = Path::new("docs/src");

    if !docs_dir.exists() {
        println!("Docs directory not found, skipping");
        return;
    }

    let _engine = create_lint_engine();
    let mut files_tested = 0;

    for entry in walkdir::WalkDir::new(docs_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
    {
        let path = entry.path();
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => continue,
        };

        let start = std::time::Instant::now();

        assert_no_crash(&content, &format!("Docs file: {:?}", path.file_name()));

        let elapsed = start.elapsed();

        // Ensure reasonable performance (docs files should be fast)
        assert!(
            elapsed < std::time::Duration::from_millis(500),
            "Doc file {:?} took too long: {:?}",
            path.file_name(),
            elapsed
        );

        files_tested += 1;
    }

    if files_tested > 0 {
        println!("✅ Tested {} documentation files", files_tested);
    }
}

// Helper to import walkdir for the docs test
#[cfg(test)]
mod deps {
    // We'll need walkdir for traversing docs
    // This can be added as a dev dependency if not present
}
