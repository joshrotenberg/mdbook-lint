//! Tests for batch 2 rule configuration (MD024, MD025, MD026, MD029, MD030)

mod common;

use common::*;
use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_md024_siblings_only_configuration() {
    // Test MD024 with siblings_only configuration
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.md");
    let config_file = temp_dir.path().join(".mdbook-lint.toml");

    let content = r#"# Main Title
## Introduction
### Introduction
## Configuration  
### Configuration
"#;
    fs::write(&test_file, content).unwrap();

    // Test with siblings_only = false (default) - should detect duplicates across levels
    let config = r#"
[rules]
default = false

[rules.enabled]
MD024 = true

[MD024]
siblings_only = false
"#;
    fs::write(&config_file, config).unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--config")
        .arg(&config_file)
        .arg(&test_file)
        .assert();

    assert
        .failure()
        .stdout(contains("MD024"))
        .stdout(contains("Introduction"))
        .stdout(contains("Configuration"));

    // Test with siblings_only = true - should only detect duplicates at same level
    let config = r#"
[rules]
default = false

[rules.enabled]
MD024 = true

[MD024]
siblings_only = true
"#;
    fs::write(&config_file, config).unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--config")
        .arg(&config_file)
        .arg(&test_file)
        .assert();

    // Should succeed as there are no duplicates at same level
    assert.success();
}

#[test]
fn test_md025_level_configuration() {
    // Test MD025 with level configuration
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.md");
    let config_file = temp_dir.path().join(".mdbook-lint.toml");

    let content = r#"# First H1
## H2 heading
# Second H1
## Another H2
## Third H2
"#;
    fs::write(&test_file, content).unwrap();

    // Test with level = 1 (default) - should detect multiple H1s
    let config = r#"
[rules]
default = false

[rules.enabled]
MD025 = true

[MD025]
level = 1
"#;
    fs::write(&config_file, config).unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--config")
        .arg(&config_file)
        .arg(&test_file)
        .assert();

    assert
        .failure()
        .stdout(contains("MD025"))
        .stdout(contains("Multiple top-level headings"));

    // Test with level = 2 - should detect multiple H2s
    let config = r#"
[rules]
default = false

[rules.enabled]
MD025 = true

[MD025]
level = 2
"#;
    fs::write(&config_file, config).unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--config")
        .arg(&config_file)
        .arg(&test_file)
        .assert();

    assert
        .failure()
        .stdout(contains("MD025"))
        .stdout(contains("Multiple top-level headings"));
}

#[test]
fn test_md026_punctuation_configuration() {
    // Test MD026 with punctuation configuration
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.md");
    let config_file = temp_dir.path().join(".mdbook-lint.toml");

    let content = r#"# Heading with period.
## Heading with exclamation!
### Heading with custom @
#### Heading without punctuation
"#;
    fs::write(&test_file, content).unwrap();

    // Test with default punctuation
    let config = r#"
[rules]
default = false

[rules.enabled]
MD026 = true
"#;
    fs::write(&config_file, config).unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--config")
        .arg(&config_file)
        .arg(&test_file)
        .assert();

    assert
        .failure()
        .stdout(contains("MD026"))
        .stdout(contains("period"))
        .stdout(contains("exclamation"));

    // Test with custom punctuation
    let config = r#"
[rules]
default = false

[rules.enabled]
MD026 = true

[MD026]
punctuation = ".@"
"#;
    fs::write(&config_file, config).unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--config")
        .arg(&config_file)
        .arg(&test_file)
        .assert();

    assert
        .failure()
        .stdout(contains("MD026"))
        .stdout(contains("period"))
        .stdout(contains("@"));
}

#[test]
fn test_md029_style_configuration() {
    // Test MD029 with style configuration
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.md");
    let config_file = temp_dir.path().join(".mdbook-lint.toml");

    let content = r#"# Ordered Lists

1. First item
1. Second item
3. Third item
4. Fourth item
"#;
    fs::write(&test_file, content).unwrap();

    // Test with style = "sequential"
    let config = r#"
[rules]
default = false

[rules.enabled]
MD029 = true

[MD029]
style = "sequential"
"#;
    fs::write(&config_file, config).unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--config")
        .arg(&config_file)
        .arg(&test_file)
        .assert();

    assert
        .failure()
        .stdout(contains("MD029"))
        .stdout(contains("expected '2'"));

    // Test with style = "all_ones"
    let config = r#"
[rules]
default = false

[rules.enabled]
MD029 = true

[MD029]
style = "all_ones"
"#;
    fs::write(&config_file, config).unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--config")
        .arg(&config_file)
        .arg(&test_file)
        .assert();

    assert
        .failure()
        .stdout(contains("MD029"))
        .stdout(contains("expected '1'"));
}

#[test]
fn test_md030_spacing_configuration() {
    // Test MD030 with spacing configuration
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.md");
    let config_file = temp_dir.path().join(".mdbook-lint.toml");

    let content = r#"# Lists

- Single space
-  Two spaces
1. Single space
1.  Two spaces
"#;
    fs::write(&test_file, content).unwrap();

    // Test with default (1 space)
    let config = r#"
[rules]
default = false

[rules.enabled]
MD030 = true
"#;
    fs::write(&config_file, config).unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--config")
        .arg(&config_file)
        .arg(&test_file)
        .assert();

    assert
        .failure()
        .stdout(contains("MD030"))
        .stdout(contains("expected 1 space"));

    // Test with custom spacing
    let config = r#"
[rules]
default = false

[rules.enabled]
MD030 = true

[MD030]
ul_single = 2
ol_single = 2
"#;
    fs::write(&config_file, config).unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--config")
        .arg(&config_file)
        .arg(&test_file)
        .assert();

    assert
        .failure()
        .stdout(contains("MD030"))
        .stdout(contains("expected 2 space"));
}

#[test]
fn test_md030_hyphenated_config_keys() {
    // Test MD030 with hyphenated configuration keys
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.md");
    let config_file = temp_dir.path().join(".mdbook-lint.toml");

    let content = r#"# Lists

- Single space
-  Two spaces
"#;
    fs::write(&test_file, content).unwrap();

    let config = r#"
[rules]
default = false

[rules.enabled]
MD030 = true

[MD030]
ul-single = 2
"#;
    fs::write(&config_file, config).unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--config")
        .arg(&config_file)
        .arg(&test_file)
        .assert();

    assert
        .failure()
        .stdout(contains("MD030"))
        .stdout(contains("expected 2 space"));
}

#[test]
fn test_md024_hyphenated_config_key() {
    // Test MD024 with hyphenated configuration key
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.md");
    let config_file = temp_dir.path().join(".mdbook-lint.toml");

    let content = r#"# Main Title
## Introduction
### Introduction
"#;
    fs::write(&test_file, content).unwrap();

    let config = r#"
[rules]
default = false

[rules.enabled]
MD024 = true

[MD024]
siblings-only = true
"#;
    fs::write(&config_file, config).unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--config")
        .arg(&config_file)
        .arg(&test_file)
        .assert();

    // Should succeed as there are no duplicates at same level
    assert.success();
}

#[test]
fn test_batch2_all_rules_with_config() {
    // Test all batch 2 rules together with configuration
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.md");
    let config_file = temp_dir.path().join(".mdbook-lint.toml");

    let content = r#"# Main Title!
## Introduction
### Introduction
## Another Section.

1. First item
1. Second item

- Item with one space
-   Item with three spaces
"#;
    fs::write(&test_file, content).unwrap();

    let config = r#"
[rules]
default = false

[rules.enabled]
MD024 = true
MD025 = true
MD026 = true
MD029 = true
MD030 = true

[MD024]
siblings_only = false

[MD025]
level = 1

[MD026]
punctuation = "!."

[MD029]
style = "sequential"

[MD030]
ul_single = 1
ol_single = 1
"#;
    fs::write(&config_file, config).unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--config")
        .arg(&config_file)
        .arg(&test_file)
        .assert();

    // Should have violations from multiple rules
    assert
        .failure()
        .stdout(contains("MD024")) // "Introduction" duplicated
        .stdout(contains("MD026")) // "!" and "." in headings
        .stdout(contains("MD029")) // Second item should be "2"
        .stdout(contains("MD030")); // Three spaces after "-"
}
