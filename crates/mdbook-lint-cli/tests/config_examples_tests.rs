mod common;

use common::cli_command;
use std::{fs, path::PathBuf};
use tempfile::TempDir;

const EXAMPLES: &[&str] = &[
    "strict.mdbook-lint.toml",
    "relaxed.mdbook-lint.toml",
    "api-docs.mdbook-lint.toml",
    "tutorial.mdbook-lint.toml",
];

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples")
        .join(name)
}

#[test]
fn example_configurations_are_valid_and_loadable() {
    let temp_dir = TempDir::new().unwrap();
    let markdown = temp_dir.path().join("example.md");
    fs::write(&markdown, "# Example\n").unwrap();

    for name in EXAMPLES {
        let config = example_path(name);

        cli_command()
            .arg("check")
            .arg(&config)
            .assert()
            .success()
            .stderr("");

        // Loading the configuration through lint exercises rule constructors,
        // while the single enabled rule keeps the fixture independent of the
        // policy choices made by each example.
        cli_command()
            .arg("lint")
            .arg("--enable")
            .arg("MD003")
            .arg("--config")
            .arg(&config)
            .arg(&markdown)
            .assert()
            .success();
    }
}
