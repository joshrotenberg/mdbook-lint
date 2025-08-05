//! Corpus testing framework for mdbook-lint compatibility validation
//!
//! This module provides comprehensive testing against various markdown corpora
//! to validate rule compatibility with markdownlint and ensure performance targets.

use mdbook_lint::{
    Config, Document, LintEngine, Severity, Violation, create_engine_with_all_rules,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use walkdir::WalkDir;

/// Configuration for corpus testing
#[derive(Debug, Clone)]
pub struct CorpusTestConfig {
    /// Path to markdownlint executable (optional)
    pub markdownlint_path: Option<PathBuf>,
    /// Timeout for external markdownlint calls
    #[allow(dead_code)]
    pub timeout: Duration,
    /// Whether to run performance benchmarks
    pub run_benchmarks: bool,
    /// Whether to generate detailed reports
    pub detailed_reports: bool,
}

impl Default for CorpusTestConfig {
    fn default() -> Self {
        Self {
            markdownlint_path: None,
            timeout: Duration::from_secs(30),
            run_benchmarks: true,
            detailed_reports: true,
        }
    }
}

/// A single test case in the corpus
#[derive(Debug, Clone)]
pub struct CorpusTest {
    /// Test name for identification
    pub name: String,
    /// Path to the markdown file
    pub source_path: PathBuf,
    /// Expected violations (if known)
    #[allow(dead_code)]
    pub expected_violations: Option<Vec<ExpectedViolation>>,
    /// Test category
    pub category: TestCategory,
}

/// Expected violation for comparison testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExpectedViolation {
    pub rule_id: String,
    pub line: u32,
    pub column: u32,
    pub severity: Severity,
    pub message: String,
}

impl From<&Violation> for ExpectedViolation {
    fn from(violation: &Violation) -> Self {
        Self {
            rule_id: violation.rule_id.clone(),
            line: violation.line as u32,
            column: violation.column as u32,
            severity: violation.severity,
            message: violation.message.clone(),
        }
    }
}

/// Category of test for organization
#[derive(Debug, Clone, PartialEq)]
pub enum TestCategory {
    /// Official markdownlint test suite
    MarkdownlintOfficial,
    /// Real mdBook projects
    RealProject,
    /// Synthetic edge cases
    EdgeCase,
    /// Performance benchmarks
    Performance,
    /// CommonMark specification
    #[allow(dead_code)]
    CommonMark,
}

/// Result of comparing our implementation vs markdownlint
#[derive(Debug, Clone)]
pub struct ComparisonResult {
    /// Test that was run
    #[allow(dead_code)]
    pub test: CorpusTest,
    /// Violations found by mdbook-lint
    pub our_violations: Vec<Violation>,
    /// Violations found by markdownlint (if available)
    #[allow(dead_code)]
    pub markdownlint_violations: Option<Vec<ExpectedViolation>>,
    /// Time taken by mdbook-lint
    #[allow(dead_code)]
    pub our_time: Duration,
    /// Time taken by markdownlint (if available)
    #[allow(dead_code)]
    pub markdownlint_time: Option<Duration>,
    /// Compatibility status
    pub compatibility: CompatibilityStatus,
}

/// Compatibility status between implementations
#[derive(Debug, Clone, PartialEq)]
pub enum CompatibilityStatus {
    /// Identical results
    Identical,
    /// Compatible differences (expected)
    Compatible,
    /// Minor differences (investigate)
    MinorDifferences,
    /// Major differences (incompatible)
    Incompatible,
    /// Unable to compare (markdownlint failed)
    UnableToCompare,
}

/// Comprehensive compatibility report
#[derive(Debug, Clone, Serialize)]
pub struct CompatibilityReport {
    /// Total number of files tested
    pub total_files: usize,
    /// Number with identical results
    pub identical_results: usize,
    /// Number with compatible differences
    pub compatible_differences: usize,
    /// Number with minor differences
    pub minor_differences: usize,
    /// Number with incompatible differences
    pub incompatible_differences: usize,
    /// Number unable to compare
    pub unable_to_compare: usize,
    /// Per-rule compatibility breakdown
    pub rule_breakdown: HashMap<String, RuleCompatibility>,
    /// Performance metrics
    pub performance: Option<PerformanceReport>,
    /// Test execution time
    pub total_time: Duration,
}

impl CompatibilityReport {
    /// Calculate overall compatibility percentage
    pub fn compatibility_percentage(&self) -> f64 {
        if self.total_files == 0 {
            return 100.0;
        }

        let compatible_count = self.identical_results + self.compatible_differences;
        (compatible_count as f64 / self.total_files as f64) * 100.0
    }

    /// Get success percentage (including minor differences)
    pub fn success_percentage(&self) -> f64 {
        if self.total_files == 0 {
            return 100.0;
        }

        let success_count = self.total_files - self.incompatible_differences;
        (success_count as f64 / self.total_files as f64) * 100.0
    }
}

/// Per-rule compatibility statistics
#[derive(Debug, Clone, Serialize)]
pub struct RuleCompatibility {
    /// Rule identifier
    pub rule_id: String,
    /// Number of files where rule was tested
    pub files_tested: usize,
    /// Number with identical results
    pub identical: usize,
    /// Number with compatible differences
    pub compatible: usize,
    /// Number with problems
    pub problematic: usize,
}

/// Performance comparison report
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceReport {
    /// Total processing time for mdbook-lint
    pub our_total_time: Duration,
    /// Total processing time for markdownlint
    pub markdownlint_total_time: Duration,
    /// Speed improvement factor
    pub speed_improvement: f64,
    /// Average time per file for mdbook-lint
    pub our_avg_time_per_file: Duration,
    /// Average time per file for markdownlint
    pub markdownlint_avg_time_per_file: Duration,
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize)]
pub struct MemoryStats {
    /// Peak memory usage in bytes
    pub peak_memory_bytes: u64,
    /// Average memory usage in bytes
    pub avg_memory_bytes: u64,
}

/// Main corpus testing runner
pub struct CorpusRunner {
    /// Configuration for testing
    config: CorpusTestConfig,
    /// mdbook-lint engine
    engine: LintEngine,
    /// Test cases to run
    test_cases: Vec<CorpusTest>,
}

impl CorpusRunner {
    /// Create a new corpus runner with default configuration
    pub fn new() -> Self {
        Self::with_config(CorpusTestConfig::default())
    }

    /// Create a new corpus runner with custom configuration
    pub fn with_config(config: CorpusTestConfig) -> Self {
        Self {
            config,
            engine: create_engine_with_all_rules(),
            test_cases: Vec::new(),
        }
    }

    /// Add a directory of markdown files to test
    pub fn add_directory<P: AsRef<Path>>(mut self, path: P, category: TestCategory) -> Self {
        let path = path.as_ref();

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                e.path()
                    .extension()
                    .is_some_and(|ext| ext == "md" || ext == "markdown")
            })
        {
            let test_name = entry
                .path()
                .strip_prefix(path)
                .unwrap_or(entry.path())
                .to_string_lossy()
                .to_string();

            self.test_cases.push(CorpusTest {
                name: test_name,
                source_path: entry.path().to_path_buf(),
                expected_violations: None,
                category: category.clone(),
            });
        }

        self
    }

    /// Add a single test file
    pub fn add_file<P: AsRef<Path>>(
        mut self,
        path: P,
        name: String,
        category: TestCategory,
    ) -> Self {
        self.test_cases.push(CorpusTest {
            name,
            source_path: path.as_ref().to_path_buf(),
            expected_violations: None,
            category,
        });
        self
    }

    /// Run compatibility tests against markdownlint
    pub fn run_compatibility_test(&self) -> CompatibilityReport {
        let start_time = Instant::now();
        let mut results = Vec::new();

        println!(
            "Running corpus compatibility test on {} files...",
            self.test_cases.len()
        );

        for (i, test) in self.test_cases.iter().enumerate() {
            if i % 50 == 0 {
                println!("Progress: {}/{}", i + 1, self.test_cases.len());
            }

            let result = self.compare_file(test);
            results.push(result);
        }

        self.generate_compatibility_report(results, start_time.elapsed())
    }

    /// Run performance benchmarks
    pub fn run_performance_benchmark(&self) -> PerformanceReport {
        println!(
            "Running performance benchmark on {} files...",
            self.test_cases.len()
        );

        let mut our_total_time = Duration::ZERO;
        let mut markdownlint_total_time = Duration::ZERO;
        let mut files_processed = 0;

        for test in &self.test_cases {
            // Benchmark our implementation
            let our_start = Instant::now();
            let _ = self.run_mdbook_lint(&test.source_path);
            our_total_time += our_start.elapsed();

            // Benchmark markdownlint if available
            if let Some(markdownlint_time) = self.benchmark_markdownlint(&test.source_path) {
                markdownlint_total_time += markdownlint_time;
                files_processed += 1;
            }
        }

        let speed_improvement = if markdownlint_total_time.as_nanos() > 0 {
            markdownlint_total_time.as_secs_f64() / our_total_time.as_secs_f64()
        } else {
            0.0
        };

        PerformanceReport {
            our_total_time,
            markdownlint_total_time,
            speed_improvement,
            our_avg_time_per_file: our_total_time / (self.test_cases.len() as u32),
            markdownlint_avg_time_per_file: if files_processed > 0 {
                markdownlint_total_time / (files_processed as u32)
            } else {
                Duration::ZERO
            },
            memory_stats: MemoryStats {
                peak_memory_bytes: 0, // TODO: Implement memory tracking
                avg_memory_bytes: 0,
            },
        }
    }

    /// Compare a single file between implementations
    fn compare_file(&self, test: &CorpusTest) -> ComparisonResult {
        // Run our implementation
        let our_start = Instant::now();
        let our_violations = self.run_mdbook_lint(&test.source_path);
        let our_time = our_start.elapsed();

        // Run markdownlint if available
        let (markdownlint_violations, markdownlint_time) =
            if let Some(result) = self.run_markdownlint(&test.source_path) {
                (Some(result.0), Some(result.1))
            } else {
                (None, None)
            };

        // Determine compatibility
        let compatibility = self.assess_compatibility(&our_violations, &markdownlint_violations);

        ComparisonResult {
            test: test.clone(),
            our_violations,
            markdownlint_violations,
            our_time,
            markdownlint_time,
            compatibility,
        }
    }

    /// Run mdbook-lint on a file
    fn run_mdbook_lint(&self, path: &Path) -> Vec<Violation> {
        match fs::read_to_string(path) {
            Ok(content) => match Document::new(content, path.to_path_buf()) {
                Ok(document) => {
                    let config = Config::default();
                    self.engine
                        .lint_document_with_config(&document, &config)
                        .unwrap_or_default()
                }
                Err(_) => Vec::new(),
            },
            Err(_) => Vec::new(),
        }
    }

    /// Run markdownlint on a file and return violations with timing
    fn run_markdownlint(&self, path: &Path) -> Option<(Vec<ExpectedViolation>, Duration)> {
        let markdownlint_cmd = self.find_markdownlint()?;

        let start = Instant::now();
        let output = Command::new(&markdownlint_cmd)
            .arg("--json")
            .arg(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .ok()?;
        let duration = start.elapsed();

        // markdownlint outputs JSON to stderr when there are violations, stdout when clean
        let json_output = if !output.stderr.is_empty() {
            String::from_utf8(output.stderr).ok()?
        } else if !output.stdout.is_empty() {
            String::from_utf8(output.stdout).ok()?
        } else {
            // No violations found - return empty list
            return Some((Vec::new(), duration));
        };

        let violations = self.parse_markdownlint_output(&json_output)?;
        Some((violations, duration))
    }

    /// Benchmark markdownlint execution time only
    fn benchmark_markdownlint(&self, path: &Path) -> Option<Duration> {
        let markdownlint_cmd = self.find_markdownlint()?;

        let start = Instant::now();
        let _output = Command::new(&markdownlint_cmd)
            .arg("--json")
            .arg(path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .ok()?;

        Some(start.elapsed())
    }

    /// Find markdownlint executable
    fn find_markdownlint(&self) -> Option<PathBuf> {
        if let Some(path) = &self.config.markdownlint_path {
            return Some(path.clone());
        }

        // Try common locations
        for cmd in &["markdownlint", "markdownlint-cli", "npx markdownlint"] {
            if Command::new(cmd)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok_and(|s| s.success())
            {
                return Some(PathBuf::from(cmd));
            }
        }

        None
    }

    /// Parse markdownlint JSON output into violations
    fn parse_markdownlint_output(&self, output: &str) -> Option<Vec<ExpectedViolation>> {
        // markdownlint outputs an array of objects with file paths as keys
        let parsed: serde_json::Value = serde_json::from_str(output).ok()?;

        let mut violations = Vec::new();

        if let Some(obj) = parsed.as_object() {
            for (_file_path, file_violations) in obj {
                if let Some(violation_array) = file_violations.as_array() {
                    for violation in violation_array {
                        if let Some(v) = self.parse_single_violation(violation) {
                            violations.push(v);
                        }
                    }
                }
            }
        }

        Some(violations)
    }

    /// Parse a single violation from markdownlint JSON
    fn parse_single_violation(&self, violation: &serde_json::Value) -> Option<ExpectedViolation> {
        let obj = violation.as_object()?;

        let rule_names = obj.get("ruleNames")?.as_array()?;
        let rule_id = rule_names.first()?.as_str()?.to_string();
        let line = obj.get("lineNumber")?.as_u64()? as u32;
        let column = obj.get("columnNumber")?.as_u64().unwrap_or(1) as u32;
        let description = obj.get("ruleDescription")?.as_str()?.to_string();

        // markdownlint doesn't have severity levels like we do,
        // so we'll default to Warning for compatibility
        let severity = Severity::Warning;

        Some(ExpectedViolation {
            rule_id,
            line,
            column,
            severity,
            message: description,
        })
    }

    /// Assess compatibility between our results and markdownlint results
    fn assess_compatibility(
        &self,
        our_violations: &[Violation],
        markdownlint_violations: &Option<Vec<ExpectedViolation>>,
    ) -> CompatibilityStatus {
        let Some(markdownlint_violations) = markdownlint_violations else {
            return CompatibilityStatus::UnableToCompare;
        };

        // Convert our violations to expected format
        let our_expected: Vec<ExpectedViolation> =
            our_violations.iter().map(ExpectedViolation::from).collect();

        // Simple comparison - in reality this would be more sophisticated
        if our_expected.len() == markdownlint_violations.len() {
            let mut matches = 0;
            for our_violation in &our_expected {
                if markdownlint_violations
                    .iter()
                    .any(|mv| mv.rule_id == our_violation.rule_id && mv.line == our_violation.line)
                {
                    matches += 1;
                }
            }

            let match_percentage = matches as f64 / our_expected.len() as f64;
            match match_percentage {
                p if p >= 0.95 => CompatibilityStatus::Identical,
                p if p >= 0.85 => CompatibilityStatus::Compatible,
                p if p >= 0.70 => CompatibilityStatus::MinorDifferences,
                _ => CompatibilityStatus::Incompatible,
            }
        } else {
            let diff_ratio = ((our_expected.len() as i32 - markdownlint_violations.len() as i32)
                .abs() as f64)
                / (markdownlint_violations.len().max(1) as f64);

            match diff_ratio {
                r if r <= 0.1 => CompatibilityStatus::Compatible,
                r if r <= 0.3 => CompatibilityStatus::MinorDifferences,
                _ => CompatibilityStatus::Incompatible,
            }
        }
    }

    /// Generate comprehensive compatibility report
    fn generate_compatibility_report(
        &self,
        results: Vec<ComparisonResult>,
        total_time: Duration,
    ) -> CompatibilityReport {
        let mut identical_results = 0;
        let mut compatible_differences = 0;
        let mut minor_differences = 0;
        let mut incompatible_differences = 0;
        let mut unable_to_compare = 0;
        let mut rule_breakdown: HashMap<String, RuleCompatibility> = HashMap::new();

        for result in &results {
            match result.compatibility {
                CompatibilityStatus::Identical => identical_results += 1,
                CompatibilityStatus::Compatible => compatible_differences += 1,
                CompatibilityStatus::MinorDifferences => minor_differences += 1,
                CompatibilityStatus::Incompatible => incompatible_differences += 1,
                CompatibilityStatus::UnableToCompare => unable_to_compare += 1,
            }

            // Update rule breakdown
            for violation in &result.our_violations {
                rule_breakdown
                    .entry(violation.rule_id.clone())
                    .or_insert_with(|| RuleCompatibility {
                        rule_id: violation.rule_id.clone(),
                        files_tested: 0,
                        identical: 0,
                        compatible: 0,
                        problematic: 0,
                    })
                    .files_tested += 1;

                let rule_stats = rule_breakdown.get_mut(&violation.rule_id).unwrap();
                match result.compatibility {
                    CompatibilityStatus::Identical => rule_stats.identical += 1,
                    CompatibilityStatus::Compatible => rule_stats.compatible += 1,
                    _ => rule_stats.problematic += 1,
                }
            }
        }

        // Generate performance report if benchmarking was enabled
        let performance = if self.config.run_benchmarks {
            Some(self.run_performance_benchmark())
        } else {
            None
        };

        CompatibilityReport {
            total_files: results.len(),
            identical_results,
            compatible_differences,
            minor_differences,
            incompatible_differences,
            unable_to_compare,
            rule_breakdown,
            performance,
            total_time,
        }
    }

    /// Print detailed compatibility report
    pub fn print_report(&self, report: &CompatibilityReport) {
        println!("\n📊 Corpus Testing Results");
        println!("========================");
        println!("Total files tested: {}", report.total_files);
        println!("✅ Identical results: {}", report.identical_results);
        println!(
            "🟢 Compatible differences: {}",
            report.compatible_differences
        );
        println!("🟡 Minor differences: {}", report.minor_differences);
        println!(
            "🔴 Incompatible differences: {}",
            report.incompatible_differences
        );
        println!("⚪ Unable to compare: {}", report.unable_to_compare);
        println!();
        println!(
            "📈 Compatibility: {:.1}%",
            report.compatibility_percentage()
        );
        println!("📈 Success rate: {:.1}%", report.success_percentage());
        println!("⏱️  Total time: {:.2}s", report.total_time.as_secs_f64());

        if let Some(perf) = &report.performance {
            println!("\n🚀 Performance Results");
            println!("======================");
            println!(
                "mdbook-lint time: {:.2}s",
                perf.our_total_time.as_secs_f64()
            );
            println!(
                "markdownlint time: {:.2}s",
                perf.markdownlint_total_time.as_secs_f64()
            );
            println!("Speed improvement: {:.1}x", perf.speed_improvement);
        }

        if self.config.detailed_reports && !report.rule_breakdown.is_empty() {
            println!("\n📋 Rule Breakdown");
            println!("=================");
            let mut rules: Vec<_> = report.rule_breakdown.values().collect();
            rules.sort_by(|a, b| a.rule_id.cmp(&b.rule_id));

            for rule in rules {
                println!(
                    "{}: {}/{} files compatible ({:.1}%)",
                    rule.rule_id,
                    rule.identical + rule.compatible,
                    rule.files_tested,
                    ((rule.identical + rule.compatible) as f64 / rule.files_tested as f64) * 100.0
                );
            }
        }
    }
}

impl Default for CorpusRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Edge case generator for corpus testing
///
/// This module generates synthetic markdown files with various edge cases
/// to test the robustness of mdbook-lint rules.
pub struct EdgeCaseGenerator {
    /// Output directory for generated files
    output_dir: std::path::PathBuf,
}

impl EdgeCaseGenerator {
    /// Create a new edge case generator
    pub fn new<P: AsRef<Path>>(output_dir: P) -> Self {
        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
        }
    }

    /// Generate all edge case files
    pub fn generate_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(&self.output_dir)?;

        // Generate different categories of edge cases
        self.generate_empty_files()?;
        self.generate_large_files()?;
        self.generate_deeply_nested()?;
        self.generate_pathological_cases()?;
        self.generate_unicode_cases()?;
        self.generate_line_ending_variants()?;
        self.generate_rule_specific_cases()?;

        Ok(())
    }

    /// Generate empty and whitespace-only files
    fn generate_empty_files(&self) -> Result<(), Box<dyn std::error::Error>> {
        let empty_dir = self.output_dir.join("empty");
        fs::create_dir_all(&empty_dir)?;

        // Completely empty file
        fs::write(empty_dir.join("completely_empty.md"), "")?;

        // Whitespace only
        fs::write(empty_dir.join("whitespace_only.md"), "   \n\t\n  \n")?;

        // Single newline
        fs::write(empty_dir.join("single_newline.md"), "\n")?;

        // Multiple blank lines
        fs::write(empty_dir.join("multiple_blanks.md"), "\n\n\n\n\n")?;

        // Only spaces and tabs
        fs::write(empty_dir.join("spaces_tabs.md"), "   \t  \t   ")?;

        Ok(())
    }

    /// Generate large files for performance testing
    fn generate_large_files(&self) -> Result<(), Box<dyn std::error::Error>> {
        let large_dir = self.output_dir.join("large");
        fs::create_dir_all(&large_dir)?;

        // File with many headings
        let mut many_headings = String::new();
        for i in 1..=1000 {
            many_headings.push_str(&format!("# Heading {i}\n\nContent for heading {i}.\n\n"));
        }
        fs::write(large_dir.join("many_headings.md"), many_headings)?;

        // File with very long lines
        let long_line = "a".repeat(5000);
        let long_lines_content = format!("# Test\n\n{long_line}\n\n{long_line}\n");
        fs::write(large_dir.join("very_long_lines.md"), long_lines_content)?;

        // File with many list items
        let mut many_lists = String::new();
        many_lists.push_str("# Lists\n\n");
        for i in 1..=500 {
            many_lists.push_str(&format!("- List item {i}\n"));
        }
        fs::write(large_dir.join("many_lists.md"), many_lists)?;

        // File with many code blocks
        let mut many_code_blocks = String::new();
        many_code_blocks.push_str("# Code Blocks\n\n");
        for i in 1..=200 {
            many_code_blocks.push_str(&format!(
                "```rust\nfn function_{i}() {{\n    println!(\"Function {i}\");\n}}\n```\n\n"
            ));
        }
        fs::write(large_dir.join("many_code_blocks.md"), many_code_blocks)?;

        Ok(())
    }

    /// Generate deeply nested structures
    fn generate_deeply_nested(&self) -> Result<(), Box<dyn std::error::Error>> {
        let nested_dir = self.output_dir.join("nested");
        fs::create_dir_all(&nested_dir)?;

        // Deep heading nesting
        let mut deep_headings = String::new();
        for level in 1..=6 {
            deep_headings.push_str(&format!(
                "{} Level {} Heading\n\n",
                "#".repeat(level),
                level
            ));
            deep_headings.push_str(&format!("Content at level {level}.\n\n"));
        }
        fs::write(nested_dir.join("deep_headings.md"), deep_headings)?;

        // Deep list nesting
        let mut deep_lists = String::new();
        deep_lists.push_str("# Deep Lists\n\n");
        for level in 0..20 {
            let indent = "  ".repeat(level);
            deep_lists.push_str(&format!("{}* Level {} item\n", indent, level + 1));
        }
        fs::write(nested_dir.join("deep_lists.md"), deep_lists)?;

        // Deep blockquote nesting
        let mut deep_quotes = String::new();
        deep_quotes.push_str("# Deep Blockquotes\n\n");
        for level in 1..=10 {
            let prefix = "> ".repeat(level);
            deep_quotes.push_str(&format!("{prefix}This is level {level} quote\n"));
        }
        fs::write(nested_dir.join("deep_quotes.md"), deep_quotes)?;

        Ok(())
    }

    /// Generate pathological edge cases
    fn generate_pathological_cases(&self) -> Result<(), Box<dyn std::error::Error>> {
        let pathological_dir = self.output_dir.join("pathological");
        fs::create_dir_all(&pathological_dir)?;

        // Mixed line endings
        let mixed_endings = "# Mixed Line Endings\r\n\nWindows line ending above.\n\nUnix line ending above.\r\nAnother Windows ending.\n";
        fs::write(
            pathological_dir.join("mixed_line_endings.md"),
            mixed_endings,
        )?;

        // Malformed markdown
        let malformed = r#"# Malformed Markdown

[Broken link with missing closing](

```unclosed code block
This code block is never closed

* List item 1
    * Nested item
        * Deeply nested
      * Wrong indentation
  * Another wrong indent

| Table | With |
| Missing | Rows

> Blockquote
Not properly continued

![Image with broken syntax](

# Heading with trailing spaces
## Another heading	with tab

---
Horizontal rule above, but here's a malformed one:
- - -not properly spaced

*Emphasis not closed
**Bold not closed

~~Strikethrough not closed

[Reference link][nonexistent]

[Another reference]: https://example.com "Title with unmatched quote

html <div>tag not closed

`inline code not closed

![Alt text](image.png "title not closed
"#;
        fs::write(pathological_dir.join("malformed.md"), malformed)?;

        // Extreme whitespace variations
        let extreme_whitespace = "   \t  #  \t  Heading with mixed whitespace  \t  \n\n\t\t\tContent with tabs\n   Content with spaces\n\t \t Content with mixed tabs and spaces\t \t\n\n```  \t\nCode block with trailing whitespace  \t\n```\n\n*  \t List item with trailing whitespace\n-\t\tAnother list with tabs\n";
        fs::write(
            pathological_dir.join("extreme_whitespace.md"),
            extreme_whitespace,
        )?;

        Ok(())
    }

    /// Generate Unicode and encoding edge cases
    fn generate_unicode_cases(&self) -> Result<(), Box<dyn std::error::Error>> {
        let unicode_dir = self.output_dir.join("unicode");
        fs::create_dir_all(&unicode_dir)?;

        // Various Unicode characters
        let unicode_content = r#"# Unicode Test Cases

## Emoji in headings 🚀

This document contains various Unicode characters for testing.

* List item with emoji 📝
* Chinese characters: 中文测试
* Arabic text: اختبار العربية
* Hebrew text: בדיקת עברית
* Russian text: Русский тест
* Mathematical symbols: ∑ ∫ ∂ ∆ ∞ ≠ ≤ ≥
* Currency symbols: € £ ¥ ₹ ₽

## Zero-width characters

This line contains zero-width‌characters\u{200B}that‍might\u{200B}cause‌issues.

## Combining characters

e̊xa̧m̄p̈l̊e̊ with combining diacritics

## Right-to-left text

This is English, but here is Arabic: هذا نص عربي

## Unusual whitespace

En space: ' '
Em space: ' '
Thin space: ' '
Non-breaking space: ' '

## Special quotes

"Smart quotes" vs "regular quotes"
'Single smart quotes' vs 'regular'

## Line separator characters

Line separator:
Paragraph separator:

## Various dashes

Hyphen: -
En dash: –
Em dash: —
Minus sign: −
"#;
        fs::write(unicode_dir.join("unicode_various.md"), unicode_content)?;

        // Non-UTF8 sequences (represented as escaped)
        let encoding_issues = "# Encoding Issues\n\nThis file tests various encoding edge cases.\n\nSome potentially problematic sequences:\n- Byte order marks\n- Invalid UTF-8 sequences\n- Mixed encodings\n";
        fs::write(unicode_dir.join("encoding_issues.md"), encoding_issues)?;

        Ok(())
    }

    /// Generate files with different line ending styles
    fn generate_line_ending_variants(&self) -> Result<(), Box<dyn std::error::Error>> {
        let line_endings_dir = self.output_dir.join("line_endings");
        fs::create_dir_all(&line_endings_dir)?;

        let content = "# Line Ending Test\n\nThis is paragraph one.\n\nThis is paragraph two.\n\n* List item 1\n* List item 2\n\nEnd of file.";

        // Unix line endings (LF)
        fs::write(line_endings_dir.join("unix_lf.md"), content)?;

        // Windows line endings (CRLF)
        let windows_content = content.replace('\n', "\r\n");
        fs::write(line_endings_dir.join("windows_crlf.md"), windows_content)?;

        // Old Mac line endings (CR only)
        let mac_content = content.replace('\n', "\r");
        fs::write(line_endings_dir.join("mac_cr.md"), mac_content)?;

        // Mixed line endings
        let mixed_content = "# Mixed Line Endings\r\n\nFirst paragraph (CRLF above, LF above).\r\nSecond line of first paragraph (CRLF above).\n\nSecond paragraph (LF above).\r\nLine with CRLF ending.\rLine with CR ending.\nLine with LF ending.";
        fs::write(line_endings_dir.join("mixed.md"), mixed_content)?;

        Ok(())
    }

    /// Generate rule-specific edge cases
    fn generate_rule_specific_cases(&self) -> Result<(), Box<dyn std::error::Error>> {
        let rules_dir = self.output_dir.join("rule_specific");
        fs::create_dir_all(&rules_dir)?;

        // MD001 - Heading increment edge cases
        let md001_cases = r#"# MD001 Test Cases

## Valid progression
### Level 3 after level 2

## Another level 2
### Valid level 3

# Back to level 1
##### Skipped to level 5 (should trigger MD001)

## Level 2
#### Skipped level 3 (should trigger MD001)

# Multiple violations
#### Skip straight to 4
###### And then to 6
"#;
        fs::write(rules_dir.join("md001_heading_increment.md"), md001_cases)?;

        // MD013 - Line length edge cases
        let md013_cases = format!(
            r#"# MD013 Line Length Test Cases

This line is exactly 80 characters long - should be at the limit: {}

This line is 81 characters long and should trigger MD013 line length rule: {}

```
This code block line is over 80 characters but might be ignored depending on config: {}
```

| This | table | line | is | over | 80 | characters | and | might | be | ignored | depending | on | config |

# This heading is over 80 characters long and might be ignored based on configuration

https://this-is-a-very-long-url-that-exceeds-80-characters-and-might-be-ignored-by-md013.example.com/path

Normal line.
Short line.

This line has exactly eighty characters including spaces and punctuation!!!

This line exceeds the limit by just one character making it eighty-one chars
"#,
            "x".repeat(44), // Makes exactly 80 chars with the prefix
            "x".repeat(45), // Makes exactly 81 chars with the prefix
            "x".repeat(55)  // Long line in code block
        );
        fs::write(rules_dir.join("md013_line_length.md"), md013_cases)?;

        // MD040 - Fenced code language edge cases
        let md040_cases = r#"# MD040 Fenced Code Language Test Cases

```
Code block without language (should trigger MD040)
```

```rust
Code block with language (should be fine)
```

```javascript
Another code block with language
```

```
Another code block without language
```

~~~
Triple tilde without language (should trigger MD040)
~~~

~~~python
Triple tilde with language (should be fine)
~~~

Indented code block (should not trigger MD040):

    def indented_code():
        return "This is indented, not fenced"

```text
Text language specified
```

``
Two backticks (not a code fence)
``

````
Four backticks without language (should trigger MD040)
````

```unknown-language-name
Code with unknown language name
```
"#;
        fs::write(rules_dir.join("md040_fenced_code_language.md"), md040_cases)?;

        // MDBOOK001 - mdBook specific code language cases
        let mdbook001_cases = r#"# MDBOOK001 Code Block Language Test Cases

This tests mdBook-specific code block language requirements.

```
Plain code block without language (should trigger MDBOOK001)
println!("Hello, world!");
```

```rust
Rust code with proper language tag
fn main() {
    println!("Hello, world!");
}
```

```javascript
JavaScript with language tag
console.log("Hello, world!");
```

```
HTML without language tag (should trigger MDBOOK001)
<div>Hello, world!</div>
```

```html
HTML with proper language tag
<div>Hello, world!</div>
```

```bash
Shell script with language
echo "Hello, world!"
```

```
Shell script without language (should trigger MDBOOK001)
echo "Hello, world!"
```

```python
Python with language
print("Hello, world!")
```

```
# This is a comment but no language specified (should trigger MDBOOK001)
def function():
    pass
```

~~~
Triple tilde without language (should trigger MDBOOK001)
some code here
~~~

~~~rust
Triple tilde with language (should be fine)
fn main() {}
~~~
"#;
        fs::write(
            rules_dir.join("mdbook001_code_language.md"),
            mdbook001_cases,
        )?;

        // MD004 - Unordered list marker consistency
        let md004_cases = r#"# MD004 Unordered List Marker Test Cases

Consistent asterisk markers (should be fine):
* Item 1
* Item 2
* Item 3

Consistent dash markers (should be fine):
- Item 1
- Item 2
- Item 3

Consistent plus markers (should be fine):
+ Item 1
+ Item 2
+ Item 3

Mixed markers (should trigger MD004):
* Item 1
- Item 2
+ Item 3
* Item 4

Nested lists with consistent markers:
* Top level
  * Nested level
  * Another nested
* Back to top level

Nested lists with mixed markers (should trigger MD004):
* Top level
  - Mixed nested (different marker)
  * Back to asterisk
- Mixed top level (different marker)

Multiple separate lists:

First list (asterisk):
* Item A
* Item B

Second list (dash, should trigger MD004 if we consider document-wide consistency):
- Item X
- Item Y

Ordered lists mixed with unordered (ordered lists should be ignored):
1. Ordered item
2. Another ordered
* Unordered item
* Another unordered
3. Back to ordered
"#;
        fs::write(
            rules_dir.join("md004_list_marker_consistency.md"),
            md004_cases,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_corpus_runner_creation() {
        let runner = CorpusRunner::new();
        assert!(runner.test_cases.is_empty());
    }

    #[test]
    fn test_add_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.md");
        fs::write(&test_file, "# Test\nHello world").unwrap();

        let runner =
            CorpusRunner::new().add_file(&test_file, "test.md".to_string(), TestCategory::EdgeCase);

        assert_eq!(runner.test_cases.len(), 1);
        assert_eq!(runner.test_cases[0].name, "test.md");
        assert_eq!(runner.test_cases[0].category, TestCategory::EdgeCase);
    }

    #[test]
    fn test_run_mdbook_lint() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.md");
        fs::write(&test_file, "# Test\n### Skipped level").unwrap();

        let runner = CorpusRunner::new();
        let violations = runner.run_mdbook_lint(&test_file);

        // Should find MD001 violation for skipped heading level
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.rule_id == "MD001"));
    }

    #[test]
    fn test_compatibility_report_percentages() {
        let report = CompatibilityReport {
            total_files: 100,
            identical_results: 80,
            compatible_differences: 15,
            minor_differences: 3,
            incompatible_differences: 2,
            unable_to_compare: 0,
            rule_breakdown: HashMap::new(),
            performance: None,
            total_time: Duration::from_secs(10),
        };

        assert_eq!(report.compatibility_percentage(), 95.0);
        assert_eq!(report.success_percentage(), 98.0);
    }
}
