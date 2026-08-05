//! End-to-end tests for `--output sarif` (issue #471).
//!
//! The CI guide and the companion GitHub Action advertised SARIF before the CLI
//! could emit it. These tests exercise the real binary and assert the structure
//! that code-scanning consumers depend on.
//!
//! The emitted report was additionally validated against the OASIS SARIF 2.1.0
//! JSON schema during development; that check needs network access and a JSON
//! Schema implementation, so it is not run here.

mod common;

use common::cli_command;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

/// Trips MD001 (error) and MD009 (warning).
const CONTENT: &str = "# Level 1\n\n### Skipped level 2\n\nTrailing spaces.   \n";

/// Run the linter over a fixture and return the parsed SARIF report.
fn sarif_for(content: &str, extra_args: &[&str]) -> Value {
    let temp = TempDir::new().unwrap();
    let doc = temp.path().join("doc.md");
    fs::write(&doc, content).unwrap();

    let mut cmd = cli_command();
    cmd.arg("lint").arg("--output").arg("sarif");
    for arg in extra_args {
        cmd.arg(arg);
    }
    let assert = cmd.arg(&doc).assert();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout was not valid JSON: {e}\n{stdout}"))
}

#[test]
fn test_sarif_envelope_is_well_formed() {
    let report = sarif_for(CONTENT, &[]);

    assert_eq!(report["version"], "2.1.0");
    assert!(
        report["$schema"].as_str().unwrap().contains("sarif"),
        "expected a SARIF schema URL"
    );

    let driver = &report["runs"][0]["tool"]["driver"];
    assert_eq!(driver["name"], "mdbook-lint");
    assert!(
        driver["informationUri"]
            .as_str()
            .unwrap()
            .starts_with("https://")
    );
    assert!(!driver["version"].as_str().unwrap().is_empty());
}

#[test]
fn test_results_reference_rules_and_locations() {
    let report = sarif_for(CONTENT, &[]);

    let results = report["runs"][0]["results"].as_array().unwrap();
    assert!(!results.is_empty(), "expected violations to be reported");

    let rules = report["runs"][0]["tool"]["driver"]["rules"]
        .as_array()
        .unwrap();

    for result in results {
        // Every result carries a level SARIF recognizes.
        let level = result["level"].as_str().unwrap();
        assert!(
            matches!(level, "error" | "warning" | "note"),
            "unexpected SARIF level: {level}"
        );

        assert!(!result["message"]["text"].as_str().unwrap().is_empty());

        let region = &result["locations"][0]["physicalLocation"]["region"];
        assert!(
            region["startLine"].as_u64().unwrap() >= 1,
            "SARIF regions are 1-based"
        );
        if let Some(column) = region["startColumn"].as_u64() {
            assert!(column >= 1, "SARIF columns are 1-based");
        }

        let uri = result["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
            .as_str()
            .unwrap();
        assert!(!uri.is_empty());
        assert!(!uri.contains('\\'), "URIs use forward slashes: {uri}");

        // ruleIndex must resolve to the descriptor for this result's rule.
        let index = result["ruleIndex"].as_u64().unwrap() as usize;
        assert_eq!(rules[index]["id"], result["ruleId"]);
    }
}

#[test]
fn test_rule_descriptors_carry_metadata() {
    let report = sarif_for(CONTENT, &[]);
    let rules = report["runs"][0]["tool"]["driver"]["rules"]
        .as_array()
        .unwrap();

    let md001 = rules
        .iter()
        .find(|r| r["id"] == "MD001")
        .expect("MD001 should have a descriptor");

    assert_eq!(md001["name"], "heading-increment");
    assert!(
        !md001["shortDescription"]["text"]
            .as_str()
            .unwrap()
            .is_empty()
    );
    assert!(
        md001["helpUri"]
            .as_str()
            .unwrap()
            .starts_with("https://joshrotenberg.github.io/mdbook-lint/rules/"),
        "descriptors should link to the published rule reference"
    );
    assert_eq!(md001["properties"]["stability"], "Stable");
}

#[test]
fn test_clean_run_emits_a_valid_empty_report() {
    // Code scanning still needs a report to upload when nothing was found.
    let report = sarif_for(
        "# Title\n\nA clean paragraph of prose.\n",
        &["--enable", "MD001"],
    );

    assert_eq!(report["version"], "2.1.0");
    assert_eq!(report["runs"][0]["results"].as_array().unwrap().len(), 0);
    assert_eq!(
        report["runs"][0]["tool"]["driver"]["rules"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn test_output_file_writes_report_and_keeps_stdout_clean() {
    let temp = TempDir::new().unwrap();
    let doc = temp.path().join("doc.md");
    fs::write(&doc, CONTENT).unwrap();
    // A nested path exercises parent-directory creation.
    let report_path = temp.path().join("reports/results.sarif");

    let assert = cli_command()
        .arg("lint")
        .arg("--output")
        .arg("sarif")
        .arg("--output-file")
        .arg(&report_path)
        .arg(&doc)
        .assert();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        !stdout.contains("\"$schema\""),
        "report should go to the file, not stdout"
    );

    let written = fs::read_to_string(&report_path).expect("report file should exist");
    let report: Value = serde_json::from_str(&written).expect("report file should be valid JSON");
    assert_eq!(report["version"], "2.1.0");
    assert!(!report["runs"][0]["results"].as_array().unwrap().is_empty());
}

#[test]
fn test_exit_status_still_reflects_findings() {
    let temp = TempDir::new().unwrap();
    let doc = temp.path().join("doc.md");
    fs::write(&doc, CONTENT).unwrap();
    let report_path = temp.path().join("results.sarif");

    // Errors present: non-zero exit, report still written.
    cli_command()
        .arg("lint")
        .arg("--output")
        .arg("sarif")
        .arg("--output-file")
        .arg(&report_path)
        .arg(&doc)
        .assert()
        .failure();
    assert!(report_path.exists(), "report is written even when failing");

    // Clean input: success.
    let clean = temp.path().join("clean.md");
    fs::write(&clean, "# Title\n\nA clean paragraph of prose.\n").unwrap();
    cli_command()
        .arg("lint")
        .arg("--output")
        .arg("sarif")
        .arg("--enable")
        .arg("MD001")
        .arg(&clean)
        .assert()
        .success();
}

#[test]
fn test_report_is_byte_identical_across_runs() {
    // Deterministic output keeps diffs and caching meaningful.
    let temp = TempDir::new().unwrap();
    let doc = temp.path().join("doc.md");
    fs::write(&doc, CONTENT).unwrap();

    let run = || {
        let assert = cli_command()
            .arg("lint")
            .arg("--output")
            .arg("sarif")
            .arg(&doc)
            .assert();
        String::from_utf8(assert.get_output().stdout.clone()).unwrap()
    };

    assert_eq!(run(), run());
}
