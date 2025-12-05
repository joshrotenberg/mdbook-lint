//! Integration tests for the `rules` command
//!
//! Tests for rule listing functionality including different output formats,
//! filtering options, and the --json shorthand flag.

mod common;

use common::cli_command;
use predicates::str::contains;

#[test]
fn test_rules_command_default_output() {
    let assert = cli_command().arg("rules").assert();

    assert
        .success()
        .stdout(contains("MD001"))
        .stdout(contains("heading-increment"))
        .stdout(contains("MDBOOK001"));
}

#[test]
fn test_rules_command_detailed_output() {
    let assert = cli_command().arg("rules").arg("--detailed").assert();

    // Detailed output should have table formatting with rounded borders
    assert
        .success()
        .stdout(contains("Rule"))
        .stdout(contains("Name"))
        .stdout(contains("Description"))
        .stdout(contains("Category"))
        .stdout(contains("Status"))
        .stdout(contains("Fix"))
        .stdout(contains("MD001"))
        .stdout(contains("heading-increment"));
}

#[test]
fn test_rules_command_json_shorthand() {
    let assert = cli_command().arg("rules").arg("--json").assert();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    // Should be valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("--json output should be valid JSON");

    // Should have expected structure
    assert!(parsed.get("total_rules").is_some());
    assert!(parsed.get("providers").is_some());

    // Should have rules in providers
    let providers = parsed.get("providers").unwrap().as_array().unwrap();
    assert!(!providers.is_empty());

    // First provider should have rules array
    let first_provider = &providers[0];
    assert!(first_provider.get("rules").is_some());
}

#[test]
fn test_rules_command_format_json() {
    let assert = cli_command()
        .arg("rules")
        .arg("--format")
        .arg("json")
        .assert();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    // Should be valid JSON (same as --json)
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("--format json output should be valid JSON");

    assert!(parsed.get("total_rules").is_some());
}

#[test]
fn test_rules_command_json_and_format_json_equivalent() {
    // Both --json and --format json should produce equivalent output
    let json_flag_output = cli_command().arg("rules").arg("--json").assert();

    let format_json_output = cli_command()
        .arg("rules")
        .arg("--format")
        .arg("json")
        .assert();

    let json_stdout = String::from_utf8(json_flag_output.get_output().stdout.clone()).unwrap();
    let format_stdout = String::from_utf8(format_json_output.get_output().stdout.clone()).unwrap();

    // Parse both as JSON and compare structure (not exact match due to potential ordering)
    let json_parsed: serde_json::Value = serde_json::from_str(&json_stdout).unwrap();
    let format_parsed: serde_json::Value = serde_json::from_str(&format_stdout).unwrap();

    assert_eq!(
        json_parsed.get("total_rules"),
        format_parsed.get("total_rules")
    );
}

#[test]
fn test_rules_command_standard_only() {
    let assert = cli_command().arg("rules").arg("--standard-only").assert();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    // Should have standard rules
    assert!(stdout.contains("MD001"));
    assert!(stdout.contains("MD013"));

    // Should NOT have mdBook rules
    assert!(!stdout.contains("MDBOOK001"));
    assert!(!stdout.contains("MDBOOK002"));
}

#[test]
fn test_rules_command_mdbook_only() {
    let assert = cli_command().arg("rules").arg("--mdbook-only").assert();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    // Should have mdBook rules
    assert!(stdout.contains("MDBOOK001"));
    assert!(stdout.contains("MDBOOK002"));

    // Should NOT have standard rules
    assert!(!stdout.contains("MD001"));
    assert!(!stdout.contains("MD013"));
}

#[test]
fn test_rules_command_category_filter() {
    let assert = cli_command()
        .arg("rules")
        .arg("--category")
        .arg("Structure")
        .assert();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    // Should have structure-related rules
    assert!(stdout.contains("MD001")); // heading-increment is Structure category

    // The output should be filtered (fewer rules than total)
    let line_count = stdout.lines().count();
    assert!(line_count < 80); // Total rules is 78+, filtered should be much less
}

#[test]
fn test_rules_command_detailed_with_json() {
    // --json should take precedence over --detailed
    let assert = cli_command()
        .arg("rules")
        .arg("--detailed")
        .arg("--json")
        .assert();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    // Should be JSON, not table format
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON even with --detailed");

    assert!(parsed.get("total_rules").is_some());
}

#[test]
fn test_rules_command_help() {
    let assert = cli_command().arg("rules").arg("--help").assert();

    assert
        .success()
        .stdout(contains("--detailed"))
        .stdout(contains("--json"))
        .stdout(contains("--format"))
        .stdout(contains("--standard-only"))
        .stdout(contains("--mdbook-only"))
        .stdout(contains("--category"))
        .stdout(contains("--provider"));
}
