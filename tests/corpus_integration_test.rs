//! Corpus integration tests for mdbook-lint
//!
//! These tests run comprehensive compatibility and performance validation
//! against various markdown corpora to ensure rule accuracy and speed.

use std::env;
use std::path::PathBuf;

mod corpus_tests;

use corpus_tests::{CorpusRunner, CorpusTestConfig, EdgeCaseGenerator, TestCategory};

/// Basic corpus runner functionality test (no markdownlint required)
#[test]
fn test_corpus_runner_basic() {
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.md");
    std::fs::write(&test_file, "# Test\n\nBasic test content.").unwrap();

    let runner = CorpusRunner::new().add_file(&test_file, "test.md".to_string(), TestCategory::EdgeCase);
    
    // This should work without markdownlint - just tests our internal functionality
    let report = runner.run_compatibility_test();
    
    assert_eq!(report.total_files, 1);
    assert!(report.total_time.as_nanos() > 0);
    // If markdownlint is not available, unable_to_compare should be 1
    // If it is available, the comparison will run
    if find_markdownlint().is_none() {
        assert_eq!(report.unable_to_compare, 1);
    }
}

/// Test edge case generation (no markdownlint required)
#[test] 
fn test_edge_case_generation_basic() {
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().unwrap();
    let generator = EdgeCaseGenerator::new(temp_dir.path());

    generator.generate_all().unwrap();

    // Verify structure was created
    assert!(temp_dir.path().join("empty").exists());
    assert!(temp_dir.path().join("large").exists());
    assert!(temp_dir.path().join("nested").exists());
    assert!(temp_dir.path().join("pathological").exists());
    assert!(temp_dir.path().join("unicode").exists());
    assert!(temp_dir.path().join("rule_specific").exists());
}

/// Test corpus compatibility with edge cases
#[test]
#[ignore = "Requires markdownlint installation"]
fn test_corpus_edge_cases() {
    let corpus_dir = PathBuf::from("tests/corpus/edge_cases");

    // Generate edge cases if not present
    if !corpus_dir.exists() {
        eprintln!("Generating edge cases directory: {:?}", corpus_dir);
        let generator = EdgeCaseGenerator::new(&corpus_dir);
        generator
            .generate_all()
            .expect("Failed to generate edge cases");
    } else {
        eprintln!("Edge cases directory already exists: {:?}", corpus_dir);
    }

    let runner = CorpusRunner::new()
        .add_directory(&corpus_dir, TestCategory::EdgeCase);

    let report = runner.run_compatibility_test();
    runner.print_report(&report);

    // Edge cases are synthetic and may have intentional differences
    // We expect at least 60% compatibility for basic functionality
    assert!(
        report.success_percentage() >= 60.0,
        "Compatibility too low: {:.1}%",
        report.success_percentage()
    );
}

/// Test corpus with official markdownlint test suite (if available)
#[test]
#[ignore = "Requires markdownlint installation and internet access"]
fn test_corpus_markdownlint_official() {
    let markdownlint_tests = PathBuf::from("tests/corpus/markdownlint");

    // Clone or update markdownlint test suite if needed
    if !markdownlint_tests.exists() {
        eprintln!("Cloning markdownlint test suite...");
        std::process::Command::new("git")
            .args([
                "clone",
                "--depth=1",
                "https://github.com/DavidAnson/markdownlint",
                markdownlint_tests.to_str().unwrap(),
            ])
            .status()
            .expect("Failed to clone markdownlint");
    }

    let test_dir = markdownlint_tests.join("test");
    if test_dir.exists() {
        let runner = CorpusRunner::new()
            .add_directory(&test_dir, TestCategory::MarkdownlintOfficial);

        let report = runner.run_compatibility_test();
        runner.print_report(&report);

        // Official test suite should have very high compatibility
        assert!(
            report.success_percentage() >= 80.0,
            "Official test suite compatibility too low: {:.1}%",
            report.success_percentage()
        );
    }
}

/// Test corpus with real mdBook projects
#[test]
#[ignore = "Extended test - requires download script"]
fn test_extended_corpus() {
    let real_projects_dir = PathBuf::from("tests/corpus/real_projects");

    // Download real projects if script exists and directory is empty
    if !real_projects_dir.join("mdbook").exists() {
        let download_script = PathBuf::from("scripts/download-corpus.sh");
        if download_script.exists() {
            eprintln!("Downloading real project corpus...");
            std::process::Command::new("bash")
                .arg(download_script)
                .status()
                .expect("Failed to download corpus");
        }
    }

    if real_projects_dir.exists() {
        let runner = CorpusRunner::new()
            .add_directory(&real_projects_dir, TestCategory::RealProject);

        let report = runner.run_compatibility_test();
        runner.print_report(&report);

        // Real projects may have various styles
        assert!(
            report.success_percentage() >= 70.0,
            "Real project compatibility too low: {:.1}%",
            report.success_percentage()
        );
    }
}

/// Test with our own project files (dogfooding)
#[test]
#[ignore = "Requires markdownlint installation"]
fn test_project_files_corpus() {
    let config = CorpusTestConfig {
        markdownlint_path: find_markdownlint(),
        run_benchmarks: false,
        detailed_reports: true,
        ..Default::default()
    };

    // Add all our project markdown files
    let runner = CorpusRunner::with_config(config)
        .add_file(&PathBuf::from("README.md"), "README.md".to_string(), TestCategory::RealProject)
        .add_file(&PathBuf::from("CONTRIBUTING.md"), "CONTRIBUTING.md".to_string(), TestCategory::RealProject)
        .add_file(&PathBuf::from("CHANGELOG.md"), "CHANGELOG.md".to_string(), TestCategory::RealProject)
        .add_directory(PathBuf::from("docs/src"), TestCategory::RealProject);

    let report = runner.run_compatibility_test();
    runner.print_report(&report);

    // Our own files should have good compatibility
    assert!(
        report.success_percentage() >= 75.0,
        "Project files compatibility too low: {:.1}%",
        report.success_percentage()
    );
    
    // Check for specific known issues from manual testing
    if let Some(rule_stats) = report.rule_breakdown.get("MD034") {
        let compat_pct = if rule_stats.files_tested > 0 {
            (rule_stats.identical + rule_stats.compatible) as f64 / rule_stats.files_tested as f64 * 100.0
        } else {
            0.0
        };
        eprintln!("MD034 compatibility: {:.1}%", compat_pct);
    }
    if let Some(rule_stats) = report.rule_breakdown.get("MD044") {
        let compat_pct = if rule_stats.files_tested > 0 {
            (rule_stats.identical + rule_stats.compatible) as f64 / rule_stats.files_tested as f64 * 100.0
        } else {
            0.0
        };
        eprintln!("MD044 compatibility: {:.1}%", compat_pct);
    }
}

/// Test robustness with pathological inputs
#[test]
fn test_robustness() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let generator = EdgeCaseGenerator::new(temp_dir.path());

    // Generate all test cases including pathological ones
    generator.generate_all().unwrap();

    let runner = CorpusRunner::new()
        .add_directory(temp_dir.path().join("pathological"), TestCategory::EdgeCase);

    let report = runner.run_compatibility_test();

    // Should handle all files without crashing
    assert_eq!(
        report.unable_to_compare, 0,
        "Some files caused processing failures"
    );
}

/// Test performance with large corpus
#[test]
#[ignore = "Requires large test files and can be slow"]
fn test_performance_benchmark() {
    let corpus_dir = PathBuf::from("tests/corpus/edge_cases");

    // Generate large files for performance testing
    if !corpus_dir.exists() {
        let generator = EdgeCaseGenerator::new(&corpus_dir);
        generator
            .generate_all()
            .expect("Failed to generate test files");
    }

    let config = CorpusTestConfig {
        markdownlint_path: find_markdownlint(),
        run_benchmarks: true,
        detailed_reports: false,
        ..Default::default()
    };

    let runner = CorpusRunner::with_config(config)
        .add_directory(corpus_dir.join("large"), TestCategory::Performance);

    let perf_report = runner.run_performance_benchmark();

    println!("Performance Results:");
    println!(
        "mdbook-lint: {:.2}s",
        perf_report.our_total_time.as_secs_f64()
    );
    println!(
        "markdownlint: {:.2}s",
        perf_report.markdownlint_total_time.as_secs_f64()
    );
    println!("Speed improvement: {:.1}x", perf_report.speed_improvement);

    // Should be at least 2x faster than markdownlint
    if perf_report.markdownlint_total_time.as_nanos() > 0 {
        assert!(
            perf_report.speed_improvement >= 2.0,
            "Performance target not met: {:.1}x (target: 2.0x)",
            perf_report.speed_improvement
        );
    }
}

/// Test specific rules with targeted cases
#[test]
fn test_rule_specific_cases() {
    let rules_dir = PathBuf::from("tests/corpus/edge_cases/rule_specific");

    // Generate rule-specific test cases
    if !rules_dir.exists() {
        let generator = EdgeCaseGenerator::new("tests/corpus/edge_cases");
        generator
            .generate_all()
            .expect("Failed to generate rule cases");
    }

    let runner = CorpusRunner::new().add_directory(&rules_dir, TestCategory::EdgeCase);

    let report = runner.run_compatibility_test();
    runner.print_report(&report);

    // Rule-specific tests should have very high accuracy
    assert!(
        report.success_percentage() >= 90.0,
        "Rule-specific test success rate too low: {:.1}%",
        report.success_percentage()
    );

    // Check that major rules are being tested
    let major_rules = ["MD001", "MD013", "MD040", "MDBOOK001"];
    for rule in &major_rules {
        if let Some(rule_stats) = report.rule_breakdown.get(*rule) {
            assert!(rule_stats.files_tested > 0, "Rule {rule} was not tested");
        }
    }
}

/// Test Unicode and encoding edge cases
#[test]
fn test_unicode_handling() {
    let unicode_dir = PathBuf::from("tests/corpus/edge_cases/unicode");

    // Generate Unicode test cases
    if !unicode_dir.exists() {
        let generator = EdgeCaseGenerator::new("tests/corpus/edge_cases");
        generator
            .generate_all()
            .expect("Failed to generate Unicode cases");
    }

    let runner = CorpusRunner::new().add_directory(&unicode_dir, TestCategory::EdgeCase);

    let report = runner.run_compatibility_test();
    runner.print_report(&report);

    // Unicode should be handled correctly
    assert!(
        report.success_percentage() >= 80.0,
        "Unicode handling compatibility too low: {:.1}%",
        report.success_percentage()
    );
}

/// Module for additional integration tests
#[cfg(test)]
mod integration_tests {
    use super::*;
    use corpus_tests::{CompatibilityStatus};

    #[test]
    fn test_corpus_runner_basic_functionality() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let test_file1 = temp_dir.path().join("test1.md");
        let test_file2 = temp_dir.path().join("test2.md");

        std::fs::write(&test_file1, "# Test 1\n\nContent").unwrap();
        std::fs::write(&test_file2, "# Test 2\n\n* Item").unwrap();

        let runner = CorpusRunner::new()
            .add_file(&test_file1, "test1.md".to_string(), TestCategory::EdgeCase)
            .add_file(&test_file2, "test2.md".to_string(), TestCategory::EdgeCase);

        // Test performance benchmark
        let perf_report = runner.run_performance_benchmark();
        assert!(perf_report.our_total_time.as_nanos() > 0);
        assert_eq!(
            perf_report.our_avg_time_per_file,
            perf_report.our_total_time / 2
        );

        // Test compatibility
        let compat_report = runner.run_compatibility_test();
        assert_eq!(compat_report.total_files, 2);
    }

    #[test]
    fn test_edge_case_generation() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let generator = EdgeCaseGenerator::new(temp_dir.path());

        // Test the main generation method
        generator.generate_all().unwrap();
        
        // Verify all directories were created
        assert!(temp_dir.path().join("empty").exists());
        assert!(temp_dir.path().join("large").exists());
        assert!(temp_dir.path().join("nested").exists());
        assert!(temp_dir.path().join("unicode").exists());
        assert!(temp_dir.path().join("rule_specific").exists());
        assert!(temp_dir.path().join("pathological").exists());
    }

    #[test]
    fn test_compatibility_status() {
        // Test compatibility status ordering
        assert!((CompatibilityStatus::Identical as u8) < (CompatibilityStatus::Compatible as u8));
        assert!(
            (CompatibilityStatus::Compatible as u8) < (CompatibilityStatus::MinorDifferences as u8)
        );
        assert!(
            (CompatibilityStatus::MinorDifferences as u8) < (CompatibilityStatus::Incompatible as u8)
        );
    }

    #[test]
    fn test_config_builder() {
        let config = CorpusTestConfig {
            markdownlint_path: Some(PathBuf::from("/usr/bin/markdownlint")),
            run_benchmarks: false,
            detailed_reports: true,
            ..Default::default()
        };

        assert_eq!(
            config.markdownlint_path,
            Some(PathBuf::from("/usr/bin/markdownlint"))
        );
        assert!(!config.run_benchmarks);
        assert!(config.detailed_reports);
    }
}

/// Find markdownlint executable for comparison testing
fn find_markdownlint() -> Option<PathBuf> {
    // Check environment variable first
    if let Ok(path) = env::var("MARKDOWNLINT_PATH") {
        return Some(PathBuf::from(path));
    }

    // Try common commands
    for cmd in &["markdownlint", "markdownlint-cli", "npx markdownlint"] {
        if which::which(cmd).is_ok() {
            return Some(PathBuf::from(cmd));
        }
    }

    None
}

/// Helper to run a quick corpus test manually
#[allow(dead_code)]
fn quick_corpus_test() {
    let runner = CorpusRunner::new()
        .add_file(
            &PathBuf::from("README.md"),
            "README.md".to_string(),
            TestCategory::RealProject,
        );

    let report = runner.run_compatibility_test();
    runner.print_report(&report);
}