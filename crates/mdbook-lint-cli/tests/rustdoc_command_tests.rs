//! Integration tests for the `rustdoc` subcommand
//!
//! Tests for linting module-level documentation in Rust source files.

mod common;

use common::cli_command;
use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

/// Create a temporary Rust file with the given content
fn create_temp_rust_file(content: &str) -> (TempDir, std::path::PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.rs");
    fs::write(&file_path, content).expect("Failed to write test file");
    (temp_dir, file_path)
}

#[test]
fn test_rustdoc_command_no_issues() {
    let content = r#"//! # My Crate
//!
//! This is a well-formatted module documentation that provides a comprehensive
//! introduction to the crate's functionality and purpose.
//!
//! ## Features
//!
//! - Feature one
//! - Feature two

fn main() {}
"#;

    let (_temp_dir, file_path) = create_temp_rust_file(content);

    cli_command()
        .arg("rustdoc")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(contains("No issues found"));
}

#[test]
fn test_rustdoc_command_finds_violations() {
    let content = r#"//! #Missing space after hash
//!
//! Some content.

fn main() {}
"#;

    let (_temp_dir, file_path) = create_temp_rust_file(content);

    cli_command()
        .arg("rustdoc")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(contains("MD018"))
        .stdout(contains("No space after hash"));
}

#[test]
fn test_rustdoc_command_correct_line_numbers() {
    // The violation is on line 5 of the source file
    let content = r#"// Regular comment
// Another comment
//! # Title
//!
//! ##Bad heading here

fn main() {}
"#;

    let (_temp_dir, file_path) = create_temp_rust_file(content);

    let assert = cli_command().arg("rustdoc").arg(&file_path).assert();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    // Line 5 should be reported (where ##Bad heading is)
    assert!(
        stdout.contains(":5:"),
        "Expected line 5 in output, got: {}",
        stdout
    );
}

#[test]
fn test_rustdoc_command_preserves_indentation() {
    let content = r#"//! # Lists
//!
//! - Item 1
//!   - Nested item
//!     - Deeply nested

fn main() {}
"#;

    let (_temp_dir, file_path) = create_temp_rust_file(content);

    cli_command()
        .arg("rustdoc")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(contains("No issues found"));
}

#[test]
fn test_rustdoc_command_handles_code_blocks() {
    let content = r#"//! # Example
//!
//! ```rust
//! fn example() {
//!     println!("Hello");
//! }
//! ```

fn main() {}
"#;

    let (_temp_dir, file_path) = create_temp_rust_file(content);

    cli_command()
        .arg("rustdoc")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(contains("No issues found"));
}

#[test]
fn test_rustdoc_command_no_module_docs() {
    let content = r#"/// This is an item doc, not module doc
fn foo() {}

/// Another item doc
struct Bar;
"#;

    let (_temp_dir, file_path) = create_temp_rust_file(content);

    cli_command()
        .arg("rustdoc")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(contains("No issues found"));
}

#[test]
fn test_rustdoc_command_directory_recursive() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).expect("Failed to create src directory");

    // Create multiple Rust files
    fs::write(
        src_dir.join("lib.rs"),
        r#"//! # Library
//!
//! ##Bad heading

pub mod foo;
"#,
    )
    .unwrap();

    fs::write(
        src_dir.join("foo.rs"),
        r#"//! # Foo Module
//!
//! ##Another bad heading
"#,
    )
    .unwrap();

    let assert = cli_command()
        .arg("rustdoc")
        .arg(temp_dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    // Should find violations in both files
    assert!(stdout.contains("lib.rs"), "Expected lib.rs in output");
    assert!(stdout.contains("foo.rs"), "Expected foo.rs in output");
    assert!(
        stdout.contains("MD018"),
        "Expected MD018 violations in output"
    );
}

#[test]
fn test_rustdoc_command_skips_target_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let src_dir = temp_dir.path().join("src");
    let target_dir = temp_dir.path().join("target");

    fs::create_dir_all(&src_dir).expect("Failed to create src directory");
    fs::create_dir_all(&target_dir).expect("Failed to create target directory");

    // Create a good file in src
    fs::write(
        src_dir.join("lib.rs"),
        r#"//! # Library
//!
//! Good documentation.
"#,
    )
    .unwrap();

    // Create a bad file in target (should be skipped)
    fs::write(
        target_dir.join("generated.rs"),
        r#"//! ##Bad heading in target
"#,
    )
    .unwrap();

    cli_command()
        .arg("rustdoc")
        .arg(temp_dir.path())
        .assert()
        .success()
        .stdout(contains("No issues found"));
}

#[test]
fn test_rustdoc_command_with_disable_flag() {
    let content = r#"//! #Missing space
//!
//! Some content.

fn main() {}
"#;

    let (_temp_dir, file_path) = create_temp_rust_file(content);

    cli_command()
        .arg("rustdoc")
        .arg(&file_path)
        .arg("--disable")
        .arg("MD018")
        .assert()
        .success()
        .stdout(contains("No issues found"));
}

#[test]
fn test_rustdoc_command_json_output() {
    let content = r#"//! #Bad heading

fn main() {}
"#;

    let (_temp_dir, file_path) = create_temp_rust_file(content);

    let assert = cli_command()
        .arg("rustdoc")
        .arg(&file_path)
        .arg("--output")
        .arg("json")
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    // Should be valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    assert!(parsed.get("total_violations").is_some());
    assert!(parsed.get("files").is_some());
}

#[test]
fn test_rustdoc_default_disabled_rules() {
    // MD041 (first-line-heading) should be disabled by default for rustdoc
    // because module docs often start with a description, not a heading
    let content = r#"//! This module provides utilities for processing data.
//!
//! It includes various helper functions.

fn main() {}
"#;

    let (_temp_dir, file_path) = create_temp_rust_file(content);

    let assert = cli_command().arg("rustdoc").arg(&file_path).assert();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    // Should NOT have MD041 violation (first line should be heading)
    assert!(
        !stdout.contains("MD041"),
        "MD041 should be disabled by default for rustdoc"
    );
}

#[test]
fn test_rustdoc_stops_at_regular_comment() {
    let content = r#"//! First doc block
//!
//! Some content.

// This regular comment should stop extraction

//! This should NOT be linted (second block after regular comment)
//! ##Bad heading that should be ignored

fn main() {}
"#;

    let (_temp_dir, file_path) = create_temp_rust_file(content);

    let assert = cli_command().arg("rustdoc").arg(&file_path).assert();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    // Should NOT find the MD018 violation in the second block
    assert!(
        !stdout.contains("MD018"),
        "Should not lint content after regular comment break"
    );
}
