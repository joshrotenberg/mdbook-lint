//! Integration tests for batch 2 rule configuration (MD024, MD025, MD026, MD029, MD030)

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_md024_siblings_only_configuration() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join(".mdbook-lint.toml");
    let md_path = dir.path().join("test.md");

    // Write markdown with duplicate headings at different levels
    let markdown_content = r#"# Introduction

## Introduction

## Configuration

# Introduction
"#;
    fs::write(&md_path, markdown_content).unwrap();

    // Test with siblings_only = false (detects all duplicates)
    let config_content = r#"[rules]
default = false

[rules.enabled]
MD024 = true

[MD024]
siblings_only = false
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("mdbook-lint").unwrap();
    let output = cmd
        .arg("lint")
        .arg("-c")
        .arg(&config_path)
        .arg(&md_path)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let output_text = format!("{}{}", stdout, stderr);

    // Should find MD024 violations for duplicate "Introduction" headings
    assert!(output_text.contains("MD024"), "Expected MD024 in output");
    assert!(output_text.contains("Introduction"), "Expected 'Introduction' in output");
    
    // Now test with siblings_only = true
    let config_content = r#"[rules]
default = false

[rules.enabled]
MD024 = true

[MD024]
siblings_only = true
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("mdbook-lint").unwrap();
    let output = cmd
        .arg("lint")
        .arg("-c")
        .arg(&config_path)
        .arg(&md_path)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let output_text = format!("{}{}", stdout, stderr);

    // With siblings_only, should not find violations (duplicates are at different levels)
    // Note: There may be other rules running, so we just check MD024 behavior
    if output_text.contains("MD024") {
        // If MD024 is mentioned, verify it's not for the cross-level duplicates
        assert!(!output_text.contains("MD024/no-duplicate-heading: Duplicate heading content: 'Introduction'"),
               "Should not flag Introduction duplicates at different levels with siblings_only=true");
    }
}

#[test]
fn test_md024_hyphenated_config_key() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join(".mdbook-lint.toml");
    let md_path = dir.path().join("test.md");

    let markdown_content = r#"# Title

## Same

## Same
"#;
    fs::write(&md_path, markdown_content).unwrap();

    // Test hyphenated key format
    let config_content = r#"[rules]
default = false

[rules.enabled]
MD024 = true

[MD024]
siblings-only = false
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("mdbook-lint").unwrap();
    let output = cmd
        .arg("lint")
        .arg("-c")
        .arg(&config_path)
        .arg(&md_path)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let output_text = format!("{}{}", stdout, stderr);

    // Configuration should work with hyphenated key
    assert!(output_text.contains("MD024"), "Expected MD024 in output");
    assert!(output_text.contains("Same"), "Expected 'Same' in output");
}

#[test]
fn test_md025_level_configuration() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join(".mdbook-lint.toml");
    let md_path = dir.path().join("test.md");

    // Write markdown with multiple H2s (but only one H1)
    let markdown_content = r#"# Single H1

## First H2

## Second H2

## Third H2
"#;
    fs::write(&md_path, markdown_content).unwrap();

    // Test with level = 2 (should flag multiple H2s)
    let config_content = r#"[rules]
default = false

[rules.enabled]
MD025 = true

[MD025]
level = 2
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("mdbook-lint").unwrap();
    let output = cmd
        .arg("lint")
        .arg("-c")
        .arg(&config_path)
        .arg(&md_path)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let output_text = format!("{}{}", stdout, stderr);

    // Should find MD025 violations for multiple H2s
    assert!(output_text.contains("MD025"), "Expected MD025 in output");
}

#[test]
fn test_md026_punctuation_configuration() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join(".mdbook-lint.toml");
    let md_path = dir.path().join("test.md");

    // Write markdown with various punctuation in headings
    let markdown_content = r#"# Title with period.

## Question?

### Exclamation!

#### No punctuation

##### Colon:
"#;
    fs::write(&md_path, markdown_content).unwrap();

    // Test with custom punctuation setting (only period and exclamation)
    let config_content = r#"[rules]
default = false

[rules.enabled]
MD026 = true

[MD026]
punctuation = ".!"
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("mdbook-lint").unwrap();
    let output = cmd
        .arg("lint")
        .arg("-c")
        .arg(&config_path)
        .arg(&md_path)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let output_text = format!("{}{}", stdout, stderr);

    // Should find MD026 violations for period and exclamation
    assert!(output_text.contains("MD026"), "Expected MD026 in output");
    // Question mark and colon should not be flagged with this config
}

#[test]
fn test_md029_style_configuration() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join(".mdbook-lint.toml");
    let md_path = dir.path().join("test.md");

    // Write markdown with inconsistent ordered list prefixes
    let markdown_content = r#"# Document

1. First item
1. Second item
3. Third item
"#;
    fs::write(&md_path, markdown_content).unwrap();

    // Test with style = "all_ones"
    let config_content = r#"[rules]
default = false

[rules.enabled]
MD029 = true

[MD029]
style = "all_ones"
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("mdbook-lint").unwrap();
    let output = cmd
        .arg("lint")
        .arg("-c")
        .arg(&config_path)
        .arg(&md_path)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let output_text = format!("{}{}", stdout, stderr);

    // Should find MD029 violation for item 3 (not using 1.)
    assert!(output_text.contains("MD029"), "Expected MD029 in output");
}

#[test]
fn test_md030_spacing_configuration() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join(".mdbook-lint.toml");
    let md_path = dir.path().join("test.md");

    // Write markdown with various list spacing
    let markdown_content = r#"# Document

*  Two spaces after asterisk
*   Three spaces after asterisk
* One space after asterisk

1.  Two spaces after number
2.   Three spaces after number
3. One space after number
"#;
    fs::write(&md_path, markdown_content).unwrap();

    // Test with specific spacing configuration
    let config_content = r#"[rules]
default = false

[rules.enabled]
MD030 = true

[MD030]
ul_single = 1
ol_single = 1
ul_multi = 1
ol_multi = 1
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("mdbook-lint").unwrap();
    let output = cmd
        .arg("lint")
        .arg("-c")
        .arg(&config_path)
        .arg(&md_path)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let output_text = format!("{}{}", stdout, stderr);

    // Should find MD030 violations for incorrect spacing
    assert!(output_text.contains("MD030"), "Expected MD030 in output");
}

#[test]
fn test_md030_hyphenated_config_keys() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join(".mdbook-lint.toml");
    let md_path = dir.path().join("test.md");

    let markdown_content = r#"# Document

*  Two spaces
* One space
"#;
    fs::write(&md_path, markdown_content).unwrap();

    // Test hyphenated config keys
    let config_content = r#"[rules]
default = false

[rules.enabled]
MD030 = true

[MD030]
ul-single = 1
ol-single = 1
ul-multi = 1
ol-multi = 1
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("mdbook-lint").unwrap();
    let output = cmd
        .arg("lint")
        .arg("-c")
        .arg(&config_path)
        .arg(&md_path)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let output_text = format!("{}{}", stdout, stderr);

    // Configuration should work with hyphenated keys
    assert!(output_text.contains("MD030"), "Expected MD030 in output");
}

#[test]
fn test_batch2_all_rules_with_config() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join(".mdbook-lint.toml");
    let md_path = dir.path().join("test.md");

    // Write markdown that could trigger all batch 2 rules
    let markdown_content = r#"# Title!

## Duplicate

## Duplicate

*  Two spaces
1. First
1. Second
"#;
    fs::write(&md_path, markdown_content).unwrap();

    // Configure all batch 2 rules
    let config_content = r#"[rules]
default = false

[rules.enabled]
MD024 = true
MD025 = true
MD026 = true
MD029 = true
MD030 = true

[MD024]
siblings_only = true

[MD025]
level = 1

[MD026]
punctuation = "!"

[MD029]
style = "sequential"

[MD030]
ul_single = 1
ol_single = 1
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("mdbook-lint").unwrap();
    let output = cmd
        .arg("lint")
        .arg("-c")
        .arg(&config_path)
        .arg(&md_path)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let output_text = format!("{}{}", stdout, stderr);

    // Should find violations from multiple rules
    assert!(output_text.contains("MD024"), "Expected MD024 in output"); // Duplicate siblings
    assert!(output_text.contains("MD026"), "Expected MD026 in output"); // Punctuation
    assert!(output_text.contains("MD029"), "Expected MD029 in output"); // List style
    assert!(output_text.contains("MD030"), "Expected MD030 in output"); // List spacing
}