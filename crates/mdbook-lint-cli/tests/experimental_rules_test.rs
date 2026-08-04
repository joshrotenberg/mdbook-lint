//! Integration tests for experimental rule activation
//!
//! Issue #468: experimental rules used to run by default, because the registry's
//! fallback excluded only deprecated rules. They are now opt-in.
//!
//! MDBOOK010 is used as the probe: it is marked experimental and reports an
//! unclosed inline math block for an odd number of `$` characters.

mod common;

use common::cli_command;
use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

/// Content that trips MDBOOK010 (experimental) and MD001 (stable).
const CONTENT: &str = "# Level 1\n\n### Skipped level 2\n\nUnclosed math $x = y here.\n";

/// Write `CONTENT` to a temp file and return the directory and file path.
fn fixture() -> (TempDir, String) {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("doc.md");
    fs::write(&path, CONTENT).unwrap();
    let path_str = path.to_string_lossy().to_string();
    (temp, path_str)
}

/// Write a config file into `dir` and return its path.
fn write_config(dir: &TempDir, body: &str) -> String {
    let path = dir.path().join("config.toml");
    fs::write(&path, body).unwrap();
    path.to_string_lossy().to_string()
}

#[test]
fn test_experimental_rule_is_off_by_default() {
    let (_temp, doc) = fixture();

    let assert = cli_command().arg("lint").arg(&doc).assert();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    assert!(
        !output.contains("MDBOOK010"),
        "experimental rule should not run by default, got:\n{output}"
    );
    assert!(
        output.contains("MD001"),
        "stable rules should still run, got:\n{output}"
    );
}

#[test]
fn test_enable_flag_activates_experimental_rule() {
    // --enable is an explicit selection and can name an experimental rule.
    let (_temp, doc) = fixture();

    cli_command()
        .arg("lint")
        .arg("--enable")
        .arg("MDBOOK010")
        .arg(&doc)
        .assert()
        .stdout(contains("MDBOOK010"));
}

#[test]
fn test_enable_flag_still_means_only_these_rules() {
    // Selecting one rule must not drag in the rest.
    let (_temp, doc) = fixture();

    let assert = cli_command()
        .arg("lint")
        .arg("--enable")
        .arg("MD001")
        .arg(&doc)
        .assert();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    assert!(output.contains("MD001"), "got:\n{output}");
    assert!(
        !output.contains("MDBOOK010"),
        "only the selected rule should run, got:\n{output}"
    );
}

#[test]
fn test_experimental_rules_config_adds_to_defaults() {
    // Unlike enabled-rules, experimental-rules keeps the stable set active.
    let (temp, doc) = fixture();
    let config = write_config(&temp, "experimental-rules = [\"MDBOOK010\"]\n");

    let assert = cli_command()
        .arg("lint")
        .arg("-c")
        .arg(&config)
        .arg(&doc)
        .assert();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    assert!(output.contains("MDBOOK010"), "got:\n{output}");
    assert!(output.contains("MD001"), "got:\n{output}");
}

#[test]
fn test_experimental_rules_wildcard_and_snake_case_alias() {
    for body in [
        "experimental-rules = [\"*\"]\n",
        "experimental_rules = [\"*\"]\n",
    ] {
        let (temp, doc) = fixture();
        let config = write_config(&temp, body);

        cli_command()
            .arg("lint")
            .arg("-c")
            .arg(&config)
            .arg(&doc)
            .assert()
            .stdout(contains("MDBOOK010"));
    }
}

#[test]
fn test_disabled_rules_override_experimental_opt_in() {
    let (temp, doc) = fixture();
    let config = write_config(
        &temp,
        "experimental-rules = [\"*\"]\ndisabled-rules = [\"MDBOOK010\"]\n",
    );

    let assert = cli_command()
        .arg("lint")
        .arg("-c")
        .arg(&config)
        .arg(&doc)
        .assert();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    assert!(
        !output.contains("MDBOOK010"),
        "disabled-rules must win over the opt-in, got:\n{output}"
    );
}

#[test]
fn test_rules_command_marks_experimental_rules() {
    cli_command()
        .arg("rules")
        .assert()
        .success()
        .stdout(contains("MDBOOK010"))
        .stdout(contains("[experimental, off by default]"))
        .stdout(contains("experimental-rules"));
}

#[test]
fn test_rules_detailed_reports_default_activation() {
    let assert = cli_command().arg("rules").arg("--detailed").assert();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    assert!(output.contains("Default"), "expected a Default column");
    assert!(output.contains("Experimental"), "expected stability values");
}

#[test]
fn test_check_reports_unknown_experimental_rule() {
    let temp = TempDir::new().unwrap();
    let config = write_config(&temp, "experimental-rules = [\"MDBOOK999\"]\n");

    cli_command()
        .arg("check")
        .arg(&config)
        .assert()
        .failure()
        .stderr(contains("Unknown rule in experimental-rules"));
}

#[test]
fn test_check_warns_when_rule_is_not_experimental() {
    let temp = TempDir::new().unwrap();
    let config = write_config(&temp, "experimental-rules = [\"MD001\"]\n");

    cli_command()
        .arg("check")
        .arg(&config)
        .assert()
        .success()
        .stderr(contains("is not experimental"));
}
