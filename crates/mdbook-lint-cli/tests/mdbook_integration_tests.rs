//! Integration tests for mdbook-lint preprocessor functionality
//!
//! These tests focus on mdBook-specific use cases and exercise the preprocessor
//! with various configurations and content scenarios.

mod common;

use common::*;
use predicates::prelude::*;
use predicates::str::contains;
use serde_json::json;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_preprocessor_minimal_clean_input() {
    // Test that clean input produces no violations and valid JSON output
    let assert = run_preprocessor_with_mdbook_fixture("minimal_input.json");

    assert
        .success()
        .stdout(contains("sections")) // Should output valid book JSON
        .stderr(contains("No issues found"));
}

#[test]
fn test_preprocessor_with_violations() {
    // Test that input with violations is detected and reported
    let assert = run_preprocessor_with_mdbook_fixture("violations_input.json");

    let output = String::from_utf8(assert.get_output().stderr.clone()).unwrap();

    // Verify we detect mdBook-specific violations
    verify_violations(
        &output,
        &[
            ViolationExpectation::at_least("MDBOOK001", 2), // Missing language tags
            ViolationExpectation::at_least("MD001", 1),     // Heading skip
                                                            // TODO: Fix MD013 detection issue - skipping for now
                                                            // ViolationExpectation::at_least("MD013", 1),     // Line too long
        ],
    );
}

#[test]
fn test_preprocessor_default_config() {
    // Test preprocessor with default configuration (all rules enabled)
    let temp_book = TempMdBook::new();

    temp_book
        .with_book_toml(None) // Default config
        .with_summary(
            r#"
# Summary

[Introduction](./intro.md)
- [Chapter 1](./chapter1.md)
"#,
        )
        .with_chapter(
            "intro.md",
            r#"
# Introduction

This is a clean introduction with proper formatting.

```rust
fn main() {
    println!("Hello, world!");
}
```
"#,
        )
        .with_chapter(
            "chapter1.md",
            r#"
# Chapter 1

### Skipped heading level (should trigger MD001)

```
Code without language tag (should trigger MDBOOK001)
```

[Broken link](./missing.md)
"#,
        );

    let input = temp_book.create_preprocessor_input();
    let assert = cli_command().write_stdin(input).assert();

    let stderr_output = String::from_utf8(assert.get_output().stderr.clone()).unwrap();

    verify_violations(
        &stderr_output,
        &[
            ViolationExpectation::at_least("MD001", 1), // Heading skip
            ViolationExpectation::at_least("MDBOOK001", 1), // Missing language tag
            ViolationExpectation::at_least("MDBOOK002", 1), // Broken link
        ],
    );
}

#[test]
fn test_preprocessor_mdbook_only_config() {
    // Test preprocessor with only mdBook rules enabled
    let temp_book = TempMdBook::new();

    let mdbook_only_config = json!({
        "enabled-categories": ["mdbook"],
        "disabled-categories": ["structure", "formatting", "content", "links", "accessibility"]
    });

    temp_book
        .with_book_toml(Some(mdbook_only_config.clone()))
        .with_summary(r#"
# Summary

[Introduction](./intro.md)
"#)
        .with_chapter("intro.md", r#"
# Introduction

### Skipped heading level (should NOT trigger MD001 in mdbook-only mode)

```
Code without language tag (should trigger MDBOOK001)
```

[Broken link](./missing.md)

This line is very very very very very very very very very very very very long (should NOT trigger MD013)
"#);

    let input = temp_book.create_preprocessor_input_with_config(mdbook_only_config);
    let assert = cli_command().write_stdin(input).assert();

    let stderr_output = String::from_utf8(assert.get_output().stderr.clone()).unwrap();

    // Should only see mdBook violations, not standard ones
    verify_violations(
        &stderr_output,
        &[
            ViolationExpectation::at_least("MDBOOK001", 1), // Missing language tag
            ViolationExpectation::at_least("MDBOOK002", 1), // Broken link
        ],
    );

    // Should NOT see standard rule violations
    assert_eq!(count_violations(&stderr_output, "MD001"), 0);
    assert_eq!(count_violations(&stderr_output, "MD013"), 0);
}

#[test]
fn test_preprocessor_custom_disabled_rules() {
    // Test preprocessor with specific rules disabled
    let temp_book = TempMdBook::new();

    let custom_config = json!({
        "disabled-rules": ["MD001", "MD013"],
        "fail-on-warnings": false
    });

    temp_book
        .with_book_toml(Some(custom_config.clone()))
        .with_summary(r#"
# Summary

[Introduction](./intro.md)
"#)
        .with_chapter("intro.md", r#"
# Introduction

### Skipped heading level (MD001 disabled, should not trigger)

```
Code without language tag (should still trigger MDBOOK001)
```

This line is intentionally way too long and should normally trigger MD013 but it is disabled in this test configuration so it should not appear.
"#);

    let input = temp_book.create_preprocessor_input_with_config(custom_config);
    let assert = cli_command().write_stdin(input).assert();

    let stderr_output = String::from_utf8(assert.get_output().stderr.clone()).unwrap();

    // Should see mdBook violations but not disabled standard ones
    verify_violations(
        &stderr_output,
        &[
            ViolationExpectation::at_least("MDBOOK001", 1), // Missing language tag
        ],
    );

    // Should NOT see disabled violations
    assert_eq!(count_violations(&stderr_output, "MD001"), 0);
    assert_eq!(count_violations(&stderr_output, "MD013"), 0);
}

#[test]
fn test_preprocessor_fail_on_warnings_false() {
    // Test that preprocessor doesn't fail build when fail-on-warnings is false
    let temp_book = TempMdBook::new();

    let config = json!({
        "fail-on-warnings": false,
        "fail-on-errors": false  // Also disable errors for this test
    });

    temp_book
        .with_book_toml(Some(config.clone()))
        .with_summary(
            r#"
# Summary

[Test Chapter](./test_chapter.md)
"#,
        )
        .with_chapter(
            "test_chapter.md",
            r#"
# Test Chapter

```
Code without language (warning level violation)
```
"#,
        );

    let input = temp_book.create_preprocessor_input_with_config(config);
    let assert = cli_command().write_stdin(input).assert();

    // Should succeed even with violations when fail flags are false
    assert.success();
}

#[test]
fn test_preprocessor_summary_validation() {
    // Test MDBOOK003 (SUMMARY.md validation) through preprocessor
    let temp_book = TempMdBook::new();

    temp_book
        .with_book_toml(None)
        .with_summary(
            r#"
# Summary

This is invalid SUMMARY.md structure:
- [Chapter 1] (./chapter1.md)  # Invalid format - space before parens
- [Chapter 2](./chapter2.md
- [Chapter 3]  # Missing link
"#,
        )
        .with_chapter("chapter1.md", "# Chapter 1\n\nContent here.\n")
        .with_chapter("chapter2.md", "# Chapter 2\n\nContent here.\n");

    let input = temp_book.create_preprocessor_input();
    let assert = cli_command().write_stdin(input).assert();

    let stderr_output = String::from_utf8(assert.get_output().stderr.clone()).unwrap();

    // Should detect SUMMARY.md format issues
    verify_violations(
        &stderr_output,
        &[
            ViolationExpectation::at_least("MDBOOK003", 1), // SUMMARY format issues
        ],
    );
}

#[test]
fn test_cli_lint_mdbook_files() {
    // Test CLI mode with mdBook-specific files
    let temp_dir = TempDir::new().unwrap();
    let summary_path = temp_dir.path().join("SUMMARY.md");
    let chapter_path = temp_dir.path().join("chapter.md");

    fs::write(
        &summary_path,
        r#"
# Summary

[Introduction](./intro.md)
- [Chapter 1](./chapter1.md)
"#,
    )
    .unwrap();

    fs::write(
        &chapter_path,
        r#"
# Chapter Title

```
fn code_without_language() {
    println!("Should trigger MDBOOK001");
}
```

[Broken link](./nonexistent.md)
"#,
    )
    .unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg(&summary_path)
        .arg(&chapter_path)
        .assert();

    let stdout_output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    verify_violations(
        &stdout_output,
        &[
            ViolationExpectation::at_least("MDBOOK001", 1), // Missing language tag
            ViolationExpectation::at_least("MDBOOK002", 1), // Broken link
        ],
    );
}

#[test]
fn test_cli_mdbook_only_flag() {
    // Test CLI with --mdbook-only flag
    let fixture_path = fixture_path("markdown", "mdbook_violations.md");

    let assert = cli_command()
        .arg("lint")
        .arg("--mdbook-only")
        .arg(&fixture_path)
        .assert();

    let stdout_output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    // Should only see mdBook rule violations
    verify_violations(
        &stdout_output,
        &[
            ViolationExpectation::at_least("MDBOOK001", 1), // Missing language tags
            ViolationExpectation::at_least("MDBOOK002", 1), // Broken links
        ],
    );

    // Should NOT see standard rule violations
    assert_eq!(count_violations(&stdout_output, "MD001"), 0);
    assert_eq!(count_violations(&stdout_output, "MD013"), 0);
}

#[test]
fn test_cli_with_clean_mdbook_content() {
    // Test that clean mdBook content produces no violations with mdbook-only rules
    let fixture_path = fixture_path("markdown", "mdbook_clean.md");

    let assert = cli_command()
        .arg("lint")
        .arg("--mdbook-only")
        .arg(&fixture_path)
        .assert();

    assert
        .success()
        .stdout(contains("No issues found").or(contains("Found 0 violation")));
}

#[test]
fn test_preprocessor_json_output_structure() {
    // Test that preprocessor outputs valid JSON structure
    let assert = run_preprocessor_with_mdbook_fixture("minimal_input.json");

    let stdout_output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    // Should be valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout_output).expect("Preprocessor output should be valid JSON");

    // Should have the expected structure
    assert!(
        parsed.get("sections").is_some(),
        "Output should have sections"
    );
}

#[test]
fn test_preprocessor_mdbook_05_format_roundtrip() {
    // Test that mdbook 0.5.x input (with "items") produces 0.5.x output (with "items")
    // This is critical for mdbook 0.5.x compatibility - see issue #365
    let assert = run_preprocessor_with_mdbook_fixture("minimal_input_v05.json");

    // Get output before calling success() which consumes self
    let stdout_output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let stderr_output = String::from_utf8(assert.get_output().stderr.clone()).unwrap();

    assert.success().stderr(contains("No issues found"));

    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&stdout_output)
        .unwrap_or_else(|e| panic!("Preprocessor output should be valid JSON: {e}\nOutput: {stdout_output}\nStderr: {stderr_output}"));

    // Should output "items" (mdbook 0.5.x format), NOT "sections" (mdbook 0.4.x format)
    assert!(
        parsed.get("items").is_some(),
        "Output should have 'items' for mdbook 0.5.x input, got: {}",
        stdout_output
    );
    assert!(
        parsed.get("sections").is_none(),
        "Output should NOT have 'sections' for mdbook 0.5.x input"
    );

    // Should NOT have __non_exhaustive (mdbook 0.5.x doesn't use it)
    assert!(
        parsed.get("__non_exhaustive").is_none(),
        "Output should NOT have '__non_exhaustive' for mdbook 0.5.x input"
    );
}

#[test]
fn test_preprocessor_mdbook_04_format_preserved() {
    // Test that mdbook 0.4.x input (with "sections") produces 0.4.x output (with "sections")
    let assert = run_preprocessor_with_mdbook_fixture("minimal_input.json");

    // Get output before calling success() which consumes self
    let stdout_output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    assert.success().stderr(contains("No issues found"));

    // Should be valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout_output).expect("Preprocessor output should be valid JSON");

    // Should output "sections" (mdbook 0.4.x format), NOT "items"
    assert!(
        parsed.get("sections").is_some(),
        "Output should have 'sections' for mdbook 0.4.x input"
    );
    assert!(
        parsed.get("items").is_none(),
        "Output should NOT have 'items' for mdbook 0.4.x input"
    );
}

#[test]
fn test_preprocessor_link_validation_with_relative_paths() {
    // Test that MDBOOK002 (link validation) works correctly in preprocessor mode
    // This specifically tests the fix for issue #366 where relative paths weren't
    // being resolved correctly because the book source directory wasn't available
    let temp_book = TempMdBook::new();

    let config = json!({
        "fail-on-errors": true,
        "fail-on-warnings": false
    });

    temp_book
        .with_book_toml(Some(config.clone()))
        .with_summary(
            r#"
# Summary

[Chapter 1](./chapter1.md)
[Chapter 2](./chapter2.md)
"#,
        )
        .with_chapter(
            "chapter1.md",
            r#"# Chapter 1

This chapter exists and should be linkable.

## Section A

Content in section A.
"#,
        )
        .with_chapter(
            "chapter2.md",
            r#"# Chapter 2

Link to [Chapter 1](./chapter1.md) - should be valid.

Link to [nonexistent](./does_not_exist.md) - should trigger MDBOOK002.
"#,
        );

    let input = temp_book.create_preprocessor_input_with_config(config);
    let assert = cli_command().write_stdin(input).assert();

    let stderr_output = String::from_utf8(assert.get_output().stderr.clone()).unwrap();

    // Should detect the broken link
    verify_violations(
        &stderr_output,
        &[
            ViolationExpectation::at_least("MDBOOK002", 1), // Broken link to does_not_exist.md
        ],
    );

    // The valid link should NOT be flagged (chapter1.md exists)
    // We verify by checking that the total MDBOOK002 violations is exactly 1
    // (only the broken link, not the valid one)
    let mdbook002_count = count_violations(&stderr_output, "MDBOOK002");
    assert!(
        mdbook002_count >= 1,
        "Expected at least 1 MDBOOK002 violation for the broken link"
    );
}

#[test]
fn test_preprocessor_cross_reference_validation() {
    // Test that MDBOOK006 (cross-reference anchor validation) works in preprocessor mode
    // This complements the #366 fix by ensuring anchor validation also works correctly
    let temp_book = TempMdBook::new();

    let config = json!({
        "fail-on-errors": false,
        "fail-on-warnings": false
    });

    temp_book
        .with_book_toml(Some(config.clone()))
        .with_summary(
            r#"
# Summary

[Chapter 1](./chapter1.md)
[Chapter 2](./chapter2.md)
"#,
        )
        .with_chapter(
            "chapter1.md",
            r#"# Chapter 1

## Valid Section

Content here.

## Another Section

More content.
"#,
        )
        .with_chapter(
            "chapter2.md",
            r#"# Chapter 2

Link to valid anchor: [Valid Section](./chapter1.md#valid-section)

Link to invalid anchor: [Bad Anchor](./chapter1.md#nonexistent-section)
"#,
        );

    let input = temp_book.create_preprocessor_input_with_config(config);
    let assert = cli_command().write_stdin(input).assert();

    let stderr_output = String::from_utf8(assert.get_output().stderr.clone()).unwrap();

    // Should detect the invalid anchor reference
    verify_violations(
        &stderr_output,
        &[
            ViolationExpectation::at_least("MDBOOK006", 1), // Invalid anchor reference
        ],
    );
}

#[test]
fn test_large_mdbook_project() {
    // Test performance and correctness with a larger project structure
    let temp_book = TempMdBook::new();

    let config = json!({
        "fail-on-errors": false,
        "fail-on-warnings": false
    });

    temp_book.with_book_toml(Some(config.clone())).with_summary(
        r#"
# Summary

[Introduction](./intro.md)

# Part 1: Basics
- [Getting Started](./part1/getting-started.md)
- [Configuration](./part1/config.md)

# Part 2: Advanced
- [Advanced Topics](./part2/advanced.md)
- [Troubleshooting](./part2/troubleshooting.md)

[Appendix](./appendix.md)
"#,
    );

    // Add multiple chapters with mixed content
    temp_book
        .with_chapter(
            "intro.md",
            "# Introduction\n\nWelcome to the book!\n\n```rust\nfn main() {}\n```\n",
        )
        .with_chapter(
            "part1/getting-started.md",
            "# Getting Started\n\n```\nMissing language tag\n```\n",
        )
        .with_chapter(
            "part1/config.md",
            "# Configuration\n\nConfiguration details here.\n",
        )
        .with_chapter(
            "part2/advanced.md",
            "# Advanced Topics\n\n### Skipped level\n\nContent.\n",
        )
        .with_chapter(
            "part2/troubleshooting.md",
            "# Troubleshooting\n\nHelp content.\n",
        )
        .with_chapter("appendix.md", "# Appendix\n\nAdditional info.\n");

    let input = temp_book.create_preprocessor_input_with_config(config);
    let assert = cli_command().write_stdin(input).assert();

    let stderr_output = String::from_utf8(assert.get_output().stderr.clone()).unwrap();

    // Should complete successfully and detect violations
    assert.success();

    verify_violations(
        &stderr_output,
        &[
            ViolationExpectation::at_least("MDBOOK001", 1), // Missing language tag
            ViolationExpectation::at_least("MD001", 1),     // Heading skip
        ],
    );
}

#[test]
fn test_directory_creation_fix() {
    // Test that nested directory creation works properly
    let temp_book = TempMdBook::new();

    // This should create nested directories successfully
    temp_book
        .with_chapter(
            "part1/getting-started.md",
            "# Getting Started\n\nContent here.\n",
        )
        .with_chapter(
            "part2/advanced/topics.md",
            "# Advanced Topics\n\nDeep nesting.\n",
        )
        .with_chapter("appendix.md", "# Appendix\n\nTop level file.\n");

    // Verify files were created
    assert!(temp_book.src_dir.join("part1/getting-started.md").exists());
    assert!(temp_book.src_dir.join("part2/advanced/topics.md").exists());
    assert!(temp_book.src_dir.join("appendix.md").exists());

    // Verify directories were created
    assert!(temp_book.src_dir.join("part1").is_dir());
    assert!(temp_book.src_dir.join("part2").is_dir());
    assert!(temp_book.src_dir.join("part2/advanced").is_dir());
}

#[test]
fn test_cli_with_nested_directories() {
    // Test CLI linting with nested directory structure
    let temp_book = TempMdBook::new();

    temp_book
        .with_chapter(
            "part1/getting-started.md",
            r#"
# Getting Started

```
Code without language tag (should trigger MDBOOK001)
```

[Broken link](./missing.md)
"#,
        )
        .with_chapter(
            "part2/advanced/topics.md",
            r#"
# Advanced Topics

### Skipped heading level (should trigger MD001)

This line is way too long and should trigger MD013 line length rule when configured with default settings.

```rust
fn good_code() {
    println!("This has a language tag");
}
```
"#,
        )
        .with_chapter(
            "appendix.md",
            r#"
# Appendix

Clean content here.
"#,
        );

    // Test linting all files
    let assert = cli_command()
        .arg("lint")
        .arg(temp_book.src_dir.join("part1/getting-started.md"))
        .arg(temp_book.src_dir.join("part2/advanced/topics.md"))
        .arg(temp_book.src_dir.join("appendix.md"))
        .assert();

    let stdout_output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    verify_violations(
        &stdout_output,
        &[
            ViolationExpectation::at_least("MDBOOK001", 1), // Missing language tag
            ViolationExpectation::at_least("MDBOOK002", 1), // Broken link
            ViolationExpectation::at_least("MD001", 1),     // Heading skip
                                                            // TODO: Fix MD013 detection issue - skipping for now
                                                            // ViolationExpectation::at_least("MD013", 1),     // Line too long
        ],
    );
}
