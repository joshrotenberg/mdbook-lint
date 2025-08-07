//! Common test utilities for mdbook-lint integration tests

use assert_cmd::Command;
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Get the path to a test fixture file
pub fn fixture_path(category: &str, filename: &str) -> PathBuf {
    // Try both workspace root and crate-local paths for workspace compatibility
    let crate_path = PathBuf::from(format!(
        "crates/mdbook-lint-cli/tests/fixtures/{category}/{filename}"
    ));
    let local_path = PathBuf::from(format!("tests/fixtures/{category}/{filename}"));

    let path = if crate_path.exists() {
        crate_path
    } else if local_path.exists() {
        local_path
    } else {
        // Default to local path for better error messages
        local_path
    };

    // Return absolute path so CLI commands can find the file regardless of working directory
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(path)
}

/// Read a fixture file as a string
pub fn read_fixture(category: &str, filename: &str) -> String {
    let path = fixture_path(category, filename);
    fs::read_to_string(path).expect("Failed to read fixture file")
}

/// Helper to create a CLI command for testing
pub fn cli_command() -> Command {
    Command::cargo_bin("mdbook-lint").unwrap()
}

/// Helper to run preprocessor with an mdbook fixture
pub fn run_preprocessor_with_mdbook_fixture(filename: &str) -> assert_cmd::assert::Assert {
    let input = read_fixture("mdbook", filename);
    cli_command().write_stdin(input).assert()
}

/// Helper to create a temporary mdBook project structure for testing
pub struct TempMdBook {
    pub _temp_dir: TempDir,
    pub book_dir: PathBuf,
    pub src_dir: PathBuf,
}

impl TempMdBook {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let book_dir = temp_dir.path().to_path_buf();
        let src_dir = book_dir.join("src");

        fs::create_dir_all(&src_dir).expect("Failed to create src directory");

        Self {
            _temp_dir: temp_dir,
            book_dir,
            src_dir,
        }
    }

    /// Create a book.toml file with optional preprocessor config
    pub fn with_book_toml(&self, preprocessor_config: Option<Value>) -> &Self {
        let mut config = json!({
            "book": {
                "title": "Test Book",
                "authors": ["Test Author"]
            },
            "output": {
                "html": {}
            }
        });

        if let Some(pp_config) = preprocessor_config {
            config["preprocessor"]["mdbook-lint"] = pp_config;
        }

        let book_toml = toml::to_string_pretty(&config).expect("Failed to serialize book.toml");
        fs::write(self.book_dir.join("book.toml"), book_toml).expect("Failed to write book.toml");

        self
    }

    /// Create a SUMMARY.md file
    pub fn with_summary(&self, content: &str) -> &Self {
        fs::write(self.src_dir.join("SUMMARY.md"), content).expect("Failed to write SUMMARY.md");
        self
    }

    /// Add a chapter file
    pub fn with_chapter(&self, filename: &str, content: &str) -> &Self {
        let chapter_path = self.src_dir.join(filename);

        // Create parent directories if they don't exist
        if let Some(parent) = chapter_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent directories");
        }

        fs::write(chapter_path, content).expect("Failed to write chapter file");
        self
    }

    /// Create mdBook preprocessor input JSON for testing
    pub fn create_preprocessor_input(&self) -> String {
        let book_config = json!({
            "authors": ["Test Author"],
            "description": "A test mdBook for testing mdbook-lint preprocessor integration",
            "language": "en",
            "src": "src",
            "title": "Test Book"
        });

        let chapters = self.collect_chapters();

        let book = json!({
            "sections": chapters,
            "__non_exhaustive": null
        });

        let input = json!([
            {
                "root": self.book_dir,
                "config": {
                    "book": book_config,
                    "output": {
                        "html": {}
                    },
                    "preprocessor": {
                        "mdbook-lint": {}
                    }
                },
                "renderer": "html",
                "mdbook_version": "0.4.52"
            },
            book
        ]);

        serde_json::to_string(&input).expect("Failed to serialize preprocessor input")
    }

    /// Create preprocessor input with custom config
    pub fn create_preprocessor_input_with_config(&self, preprocessor_config: Value) -> String {
        let book_config = json!({
            "authors": ["Test Author"],
            "description": "A test mdBook for testing mdbook-lint preprocessor integration",
            "language": "en",
            "src": "src",
            "title": "Test Book"
        });

        let chapters = self.collect_chapters();

        let book = json!({
            "sections": chapters,
            "__non_exhaustive": null
        });

        let input = json!([
            {
                "root": self.book_dir,
                "config": {
                    "book": book_config,
                    "output": {
                        "html": {}
                    },
                    "preprocessor": {
                        "mdbook-lint": preprocessor_config
                    }
                },
                "renderer": "html",
                "mdbook_version": "0.4.52"
            },
            book
        ]);

        serde_json::to_string(&input).expect("Failed to serialize preprocessor input")
    }

    fn collect_chapters(&self) -> Vec<Value> {
        let mut chapters = Vec::new();

        // Add SUMMARY.md if it exists
        if self.src_dir.join("SUMMARY.md").exists() {
            let summary_content = fs::read_to_string(self.src_dir.join("SUMMARY.md"))
                .expect("Failed to read SUMMARY.md");

            chapters.push(json!({
                "Chapter": {
                    "name": "Summary",
                    "content": summary_content,
                    "number": null,
                    "sub_items": [],
                    "path": "SUMMARY.md",
                    "source_path": "SUMMARY.md",
                    "parent_names": []
                }
            }));
        }

        // Recursively collect all markdown files from src directory
        self.collect_chapters_recursive(&self.src_dir, &mut chapters, 1);

        chapters
    }

    fn collect_chapters_recursive(
        &self,
        dir: &std::path::Path,
        chapters: &mut Vec<Value>,
        mut chapter_number: usize,
    ) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    // Recursively process subdirectories
                    self.collect_chapters_recursive(&path, chapters, chapter_number);
                } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".md") && name != "SUMMARY.md" {
                        let content = fs::read_to_string(&path).unwrap_or_default();

                        // Get relative path from src directory
                        let relative_path = path
                            .strip_prefix(&self.src_dir)
                            .unwrap_or(&path)
                            .to_string_lossy()
                            .to_string();

                        chapters.push(json!({
                            "Chapter": {
                                "name": name.trim_end_matches(".md"),
                                "content": content,
                                "number": [chapter_number],
                                "sub_items": [],
                                "path": relative_path,
                                "source_path": relative_path,
                                "parent_names": []
                            }
                        }));
                        chapter_number += 1;
                    }
                }
            }
        }
    }
}

/// Count actual violations by counting lines that contain the rule ID
pub fn count_violations(text: &str, rule_id: &str) -> usize {
    text.lines()
        .filter(|line| line.contains(&format!("{rule_id}/")))
        .count()
}

/// Test expectations for violation counts
pub struct ViolationExpectation {
    pub rule_id: &'static str,
    pub count: usize,
    pub min_count: Option<usize>,
}

impl ViolationExpectation {
    pub fn new(rule_id: &'static str, count: usize) -> Self {
        Self {
            rule_id,
            count,
            min_count: None,
        }
    }

    pub fn at_least(rule_id: &'static str, min_count: usize) -> Self {
        Self {
            rule_id,
            count: 0,
            min_count: Some(min_count),
        }
    }

    /// Verify this expectation against output text
    pub fn verify(&self, output: &str) {
        let actual_count = count_violations(output, self.rule_id);

        if let Some(min_count) = self.min_count {
            assert!(
                actual_count >= min_count,
                "Expected at least {} {} violations, found {}",
                min_count,
                self.rule_id,
                actual_count
            );
        } else {
            assert_eq!(
                actual_count, self.count,
                "Expected {} {} violations, found {}",
                self.count, self.rule_id, actual_count
            );
        }
    }
}

/// Verify multiple violation expectations
pub fn verify_violations(output: &str, expectations: &[ViolationExpectation]) {
    for expectation in expectations {
        expectation.verify(output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_violations() {
        let output = "file.md:5:1: MD001/rule: Error\nfile.md:10:1: MD001/rule: Error\nfile.md:15:1: MD013/rule: Warning";
        assert_eq!(count_violations(output, "MD001"), 2);
        assert_eq!(count_violations(output, "MD013"), 1);
        assert_eq!(count_violations(output, "MD999"), 0);
    }

    #[test]
    fn test_violation_expectation() {
        let output = "test.md:5:1: MD001/rule: Error\ntest.md:10:1: MD001/rule: Error";
        let expectation = ViolationExpectation::new("MD001", 2);
        expectation.verify(output);

        let min_expectation = ViolationExpectation::at_least("MD001", 1);
        min_expectation.verify(output);
    }
}
