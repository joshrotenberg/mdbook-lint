//! Integration tests for batch 3 rule configuration (MD035, MD036, MD043, MD044, MD046)

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_md035_horizontal_rule_style_configuration() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join(".mdbook-lint.toml");
    let md_path = dir.path().join("test.md");

    // Write markdown with various horizontal rule styles
    let markdown_content = r#"# Document

---

Some content

***

More content

___

Final content
"#;
    fs::write(&md_path, markdown_content).unwrap();

    // Test with "---" style configuration
    let config_content = r#"[rules]
default = false

[rules.enabled]
MD035 = true

[MD035]
style = "---"
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

    // Should find violations for *** and ___ but not ---
    assert!(
        output_text.contains("MD035"),
        "Expected MD035 in output: {}",
        output_text
    );
    assert!(output_text.contains(":7:") || output_text.contains("line 7")); // *** line
    assert!(output_text.contains(":11:") || output_text.contains("line 11")); // ___ line
    assert!(!output_text.contains(":3:") && !output_text.contains("line 3")); // --- line should be OK
}

#[test]
fn test_md036_emphasis_punctuation_configuration() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join(".mdbook-lint.toml");
    let md_path = dir.path().join("test.md");

    // Write markdown with emphasis that could be headings
    let markdown_content = r#"Some intro text

**Important Section**

Some content

**Note:**

More content

*Another Section*

Final content
"#;
    fs::write(&md_path, markdown_content).unwrap();

    // Test with custom punctuation configuration (only : and !)
    let config_content = r#"[rules]
default = false

[rules.enabled]
MD036 = true

[MD036]
punctuation = ":!"
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

    // Should find violations for lines without punctuation at the end
    assert!(
        output_text.contains("MD036"),
        "Expected MD036 in output: {}",
        output_text
    );
    assert!(output_text.contains(":3:") || output_text.contains("line 3")); // **Important Section**
    assert!(output_text.contains(":11:") || output_text.contains("line 11")); // *Another Section*
    assert!(!output_text.contains(":7:") && !output_text.contains("line 7")); // **Note:** ends with allowed punctuation
}

#[test]
fn test_md043_required_headings_configuration() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join(".mdbook-lint.toml");
    let md_path = dir.path().join("test.md");

    // Write markdown with specific headings
    let markdown_content = r#"# Overview

## Getting Started

## Configuration

## Advanced Topics
"#;
    fs::write(&md_path, markdown_content).unwrap();

    // Test with required heading structure
    let config_content = r#"[rules]
default = false

[rules.enabled]
MD043 = true

[MD043]
headings = ["Introduction", "Getting Started", "Configuration"]
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

    // Should find violation for wrong first heading
    assert!(
        output_text.contains("MD043"),
        "Expected MD043 in output: {}",
        output_text
    );
    assert!(output_text.contains("Expected heading 'Introduction' but found 'Overview'"));
}

#[test]
fn test_md044_proper_names_configuration() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join(".mdbook-lint.toml");
    let md_path = dir.path().join("test.md");

    // Write markdown with various capitalizations
    let markdown_content = r#"# Working with github

Use Javascript and typescript in your projects.

GitHub Actions and JavaScript are powerful tools.

Don't forget about rust and python.
"#;
    fs::write(&md_path, markdown_content).unwrap();

    // Test with proper names configuration
    let config_content = r#"[rules]
default = false

[rules.enabled]
MD044 = true

[MD044.names]
GitHub = "GitHub"
JavaScript = "JavaScript"
TypeScript = "TypeScript"
Rust = "Rust"
Python = "Python"
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

    // Should find violations for incorrect capitalizations
    assert!(
        output_text.contains("MD044"),
        "Expected MD044 in output: {}",
        output_text
    );
    assert!(output_text.contains("github")); // Line 1
    assert!(output_text.contains("Javascript")); // Line 3
    assert!(output_text.contains("typescript")); // Line 3
    assert!(output_text.contains("rust")); // Line 7
    assert!(output_text.contains("python")); // Line 7
}

#[test]
fn test_md046_code_block_style_configuration() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join(".mdbook-lint.toml");
    let md_path = dir.path().join("test.md");

    // Write markdown with mixed code block styles
    let markdown_content = r#"# Document

Here's some fenced code:

```rust
fn main() {
    println!("Hello");
}
```

And here's indented code:

    def hello():
        print("Hello")

More fenced code:

```python
print("World")
```
"#;
    fs::write(&md_path, markdown_content).unwrap();

    // Test with fenced style requirement
    let config_content = r#"[rules]
default = false

[rules.enabled]
MD046 = true

[MD046]
style = "fenced"
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

    // Should find violation for indented code block
    assert!(
        output_text.contains("MD046"),
        "Expected MD046 in output: {}",
        output_text
    );
    assert!(output_text.contains("expected fenced but found indented"));
}

#[test]
fn test_md046_consistent_style_detection() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join(".mdbook-lint.toml");
    let md_path = dir.path().join("test.md");

    // Write markdown starting with indented style
    let markdown_content = r#"# Document

First code block is indented:

    fn main() {
        println!("Hello");
    }

Then a fenced block:

```python
print("World")
```
"#;
    fs::write(&md_path, markdown_content).unwrap();

    // Test with consistent style (detects from first usage)
    let config_content = r#"[rules]
default = false

[rules.enabled]
MD046 = true

[MD046]
style = "consistent"
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

    // Should find violation for fenced block (first was indented)
    assert!(
        output_text.contains("MD046"),
        "Expected MD046 in output: {}",
        output_text
    );
    assert!(output_text.contains("expected indented but found fenced"));
}

#[test]
fn test_batch3_all_rules_together() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join(".mdbook-lint.toml");
    let md_path = dir.path().join("test.md");

    // Write markdown that triggers all batch 3 rules
    let markdown_content = r#"# Wrong Title

**Section One**

---

Use github and javascript here.

```rust
fn main() {}
```

***

    print("indented")
"#;
    fs::write(&md_path, markdown_content).unwrap();

    // Configure all batch 3 rules
    let config_content = r#"[rules]
default = false

[rules.enabled]
MD035 = true
MD036 = true
MD043 = true
MD044 = true
MD046 = true

[MD035]
style = "---"

[MD036]
punctuation = ".,;:!?"

[MD043]
headings = ["Introduction", "Getting Started"]

[MD044.names]
GitHub = "GitHub"
JavaScript = "JavaScript"

[MD046]
style = "fenced"
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
    assert!(
        output_text.contains("MD035"),
        "Expected MD035 in output: {}",
        output_text
    ); // *** on line 13
    assert!(
        output_text.contains("MD036"),
        "Expected MD036 in output: {}",
        output_text
    ); // **Section One** on line 3
    assert!(
        output_text.contains("MD043"),
        "Expected MD043 in output: {}",
        output_text
    ); // Wrong title structure
    assert!(
        output_text.contains("MD044"),
        "Expected MD044 in output: {}",
        output_text
    ); // github and javascript
    assert!(
        output_text.contains("MD046"),
        "Expected MD046 in output: {}",
        output_text
    ); // indented code block
}
