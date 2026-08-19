//! Public contract tests for the baseline preset (issue #466).

mod common;

use common::{cli_command, fixture_path};
use mdbook_lint::{Config, PluginRegistry, RuleStability};
use mdbook_lint_rulesets::{BASELINE_RULE_IDS, RulePreset, StandardRuleProvider};
use predicates::str::contains;
use serde_json::Value;
use std::{collections::BTreeSet, fs, path::PathBuf};
use tempfile::TempDir;

const CONTENT: &str = "# Title\n\n### Skipped heading\n\nThis line is deliberately much longer than eighty characters so the non-baseline MD013 rule reports it.\n";

fn fixture() -> (TempDir, String) {
    let temp = TempDir::new().unwrap();
    let document = temp.path().join("document.md");
    fs::write(&document, CONTENT).unwrap();
    (temp, document.to_string_lossy().into_owned())
}

fn write_config(temp: &TempDir, extension: &str, body: &str) -> String {
    let path = temp.path().join(format!("config.{extension}"));
    fs::write(&path, body).unwrap();
    path.to_string_lossy().into_owned()
}

fn json_stdout(assert: &assert_cmd::assert::Assert) -> Value {
    serde_json::from_slice(&assert.get_output().stdout).expect("stdout should be JSON")
}

fn violation_ids(output: &Value) -> BTreeSet<String> {
    output["files"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|file| file["violations"].as_array().into_iter().flatten())
        .filter_map(|violation| violation["rule_id"].as_str().map(String::from))
        .collect()
}

#[test]
fn baseline_membership_is_explicit_registered_and_stable() {
    assert_eq!(
        BASELINE_RULE_IDS,
        ["MD001", "MD003", "MD009", "MD010", "MD011", "MD014"]
    );
    assert_eq!(
        BASELINE_RULE_IDS
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len(),
        BASELINE_RULE_IDS.len(),
        "preset membership must not contain duplicates"
    );

    let mut providers = PluginRegistry::new();
    providers
        .register_provider(Box::new(StandardRuleProvider))
        .unwrap();
    let engine = providers.create_engine().unwrap();

    for rule_id in BASELINE_RULE_IDS {
        let rule = engine
            .registry()
            .get_rule(rule_id)
            .unwrap_or_else(|| panic!("baseline member {rule_id} is not registered"));
        let metadata = rule.metadata();
        assert_eq!(metadata.stability, RuleStability::Stable, "{rule_id}");
        assert!(!metadata.deprecated, "{rule_id}");
    }
}

#[test]
fn rules_command_explains_exact_baseline_membership() {
    let assert = cli_command()
        .args(["rules", "--preset", "baseline", "--json"])
        .assert()
        .success();
    let output = json_stdout(&assert);

    assert_eq!(output["preset"]["name"], "baseline");
    assert!(
        output["preset"]["description"]
            .as_str()
            .unwrap()
            .contains("Low-noise")
    );
    assert_eq!(output["total_rules"], BASELINE_RULE_IDS.len());

    let listed: Vec<&str> = output["providers"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|provider| provider["rules"].as_array().unwrap())
        .map(|rule| rule["id"].as_str().unwrap())
        .collect();
    assert_eq!(listed, BASELINE_RULE_IDS);

    cli_command()
        .args(["rules", "--preset", "baseline"])
        .assert()
        .success()
        .stdout(contains("Rules in preset 'baseline'"))
        .stdout(contains("MD001"))
        .stdout(contains("MD014"));
}

#[test]
fn cli_preset_selects_baseline_without_changing_default_behavior() {
    let (temp, document) = fixture();
    let default_config = write_config(&temp, "toml", "");

    let default_assert = cli_command()
        .args([
            "lint",
            "--config",
            &default_config,
            "--output",
            "json",
            &document,
        ])
        .assert();
    let default_ids = violation_ids(&json_stdout(&default_assert));
    assert!(default_ids.contains("MD001"));
    assert!(default_ids.contains("MD013"));

    let baseline_assert = cli_command()
        .args([
            "lint",
            "--config",
            &default_config,
            "--preset",
            "baseline",
            "--output",
            "json",
            &document,
        ])
        .assert();
    let baseline_ids = violation_ids(&json_stdout(&baseline_assert));
    assert!(baseline_ids.contains("MD001"));
    assert!(!baseline_ids.contains("MD013"));
    assert!(
        baseline_ids
            .iter()
            .all(|id| BASELINE_RULE_IDS.contains(&id.as_str()))
    );
}

#[test]
fn preset_deserializes_from_every_supported_config_format() {
    for config in [
        Config::from_toml_str("preset = \"baseline\"\n").unwrap(),
        Config::from_yaml_str("preset: baseline\n").unwrap(),
        Config::from_json_str("{\"preset\":\"baseline\"}").unwrap(),
    ] {
        assert_eq!(config.preset, Some(RulePreset::Baseline));
        assert_eq!(
            config.effective_core_config().enabled_rules,
            BASELINE_RULE_IDS
        );
    }
}

#[test]
fn config_preset_and_disabled_rules_use_public_lint_path() {
    let (temp, document) = fixture();
    let config = write_config(
        &temp,
        "toml",
        "preset = \"baseline\"\ndisabled-rules = [\"MD001\"]\n",
    );

    let assert = cli_command()
        .args(["lint", "--config", &config, "--output", "json", &document])
        .assert();
    let ids = violation_ids(&json_stdout(&assert));
    assert!(!ids.contains("MD001"));
    assert!(!ids.contains("MD013"));
}

#[test]
fn explicit_config_rule_list_wins_over_config_preset() {
    let (temp, document) = fixture();
    let config = write_config(
        &temp,
        "toml",
        "preset = \"baseline\"\nenabled-rules = [\"MD013\"]\n",
    );

    let assert = cli_command()
        .args(["lint", "--config", &config, "--output", "json", &document])
        .assert();
    let ids = violation_ids(&json_stdout(&assert));
    assert!(ids.contains("MD013"));
    assert!(!ids.contains("MD001"));

    cli_command()
        .args(["check", &config])
        .assert()
        .success()
        .stderr(contains("Preset 'baseline' is ignored"));
}

#[test]
fn cli_preset_overrides_configured_rule_list_and_accepts_disable() {
    let (temp, document) = fixture();
    let config = write_config(&temp, "toml", "enabled-rules = [\"MD013\"]\n");

    let assert = cli_command()
        .args([
            "lint",
            "--config",
            &config,
            "--preset",
            "baseline",
            "--disable",
            "MD001",
            "--output",
            "json",
            &document,
        ])
        .assert();
    let ids = violation_ids(&json_stdout(&assert));
    assert!(!ids.contains("MD001"));
    assert!(!ids.contains("MD013"));
}

#[test]
fn cli_enable_overrides_config_preset_and_configured_disables() {
    let (temp, document) = fixture();
    let config = write_config(
        &temp,
        "toml",
        "preset = \"baseline\"\ndisabled-rules = [\"MD013\"]\n",
    );

    let assert = cli_command()
        .args([
            "lint", "--config", &config, "--enable", "MD013", "--output", "json", &document,
        ])
        .assert();
    let ids = violation_ids(&json_stdout(&assert));
    assert!(ids.contains("MD013"));
    assert!(!ids.contains("MD001"));
}

#[test]
fn representative_mdbook_fixture_runs_only_baseline_members() {
    let temp = TempDir::new().unwrap();
    let config = write_config(&temp, "toml", "preset = \"baseline\"\n");
    let book_src = fixture_path("real_books/minimal_book", "src");

    let assert = cli_command()
        .arg("lint")
        .arg("--config")
        .arg(config)
        .args(["--output", "json"])
        .arg(book_src)
        .assert();
    let ids = violation_ids(&json_stdout(&assert));
    assert!(
        ids.iter()
            .all(|id| BASELINE_RULE_IDS.contains(&id.as_str()))
    );
}

#[test]
fn essential_corpus_runs_through_baseline_config() {
    let temp = TempDir::new().unwrap();
    let config = write_config(&temp, "toml", "preset = \"baseline\"\n");
    let corpus = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus/essential");

    let assert = cli_command()
        .arg("lint")
        .arg("--config")
        .arg(config)
        .args(["--output", "json"])
        .arg(corpus)
        .assert();
    let ids = violation_ids(&json_stdout(&assert));
    assert!(ids.contains("MD001"));
    assert!(ids.contains("MD009"));
    assert!(
        ids.iter()
            .all(|id| BASELINE_RULE_IDS.contains(&id.as_str()))
    );
}

#[test]
fn incompatible_cli_selectors_are_rejected() {
    cli_command()
        .args([
            "lint",
            "--preset",
            "baseline",
            "--enable",
            "MD001",
            "document.md",
        ])
        .assert()
        .failure()
        .stderr(contains("cannot be used with '--enable"));

    cli_command()
        .args(["rules", "--preset", "baseline", "--mdbook-only"])
        .assert()
        .failure()
        .stderr(contains("cannot be used with '--mdbook-only"));
}

#[test]
fn check_rejects_unknown_preset_and_warns_about_ignored_selectors() {
    let temp = TempDir::new().unwrap();
    let unknown = write_config(&temp, "toml", "preset = \"unknown\"\n");
    cli_command()
        .args(["check", &unknown])
        .assert()
        .failure()
        .stderr(contains("unknown preset 'unknown'"));

    let ineffective = write_config(
        &temp,
        "yaml",
        "preset: baseline\nexperimental-rules: [MDBOOK010]\nenabled-categories: [structure]\nmarkdownlint-compatible: true\n",
    );
    cli_command()
        .args(["check", &ineffective])
        .assert()
        .success()
        .stderr(contains("experimental-rules is ignored"))
        .stderr(contains("rule categories are ignored"))
        .stderr(contains("markdownlint-compatible is ignored"));
}

#[test]
fn linting_subcommands_advertise_preset_support() {
    for subcommand in ["lint", "fix", "rustdoc", "rules"] {
        cli_command()
            .args([subcommand, "--help"])
            .assert()
            .success()
            .stdout(contains("--preset"));
    }
}
