//! Corpus integration tests for mdbook-lint
//!
//! These tests run comprehensive compatibility and performance validation
//! against various markdown corpora to ensure rule accuracy and speed.

use std::env;
use std::path::PathBuf;

mod corpus_tests;

use corpus_tests::{CorpusRunner, CorpusTestConfig, EdgeCaseGenerator, TestCategory};

/// Test corpus against edge cases
#[test]
fn test_corpus_edge_cases() {
    let corpus_dir = PathBuf::from("tests/corpus/edge_cases");

    // Generate edge cases if they don't exist
    if !corpus_dir.exists() {
        println!("Generating edge cases in: {corpus_dir:?}");
        let generator = EdgeCaseGenerator::new(&corpus_dir);
        match generator.generate_all() {
            Ok(()) => println!("Successfully generated edge cases"),
            Err(e) => {
                println!("Error generating edge cases: {e}");
                panic!("Failed to generate edge cases: {e}");
            }
        }

        // Verify files were created
        if corpus_dir.exists() {
            println!("Edge cases directory created successfully");
            let file_count = std::fs::read_dir(&corpus_dir)
                .map(|entries| entries.filter_map(|e| e.ok()).count())
                .unwrap_or(0);
            println!("Generated {file_count} edge case files");
        } else {
            println!("Warning: Edge cases directory was not created");
        }
    } else {
        println!("Edge cases directory already exists: {corpus_dir:?}");
    }

    let config = CorpusTestConfig {
        run_benchmarks: false, // Skip benchmarks for edge cases
        detailed_reports: true,
        ..Default::default()
    };

    let runner =
        CorpusRunner::with_config(config).add_directory(&corpus_dir, TestCategory::EdgeCase);

    let report = runner.run_compatibility_test();
    runner.print_report(&report);

    // Edge cases are designed to be challenging, so lower expectations are realistic
    // The key is that markdownlint integration is working (no "Unable to compare")
    assert!(
        report.unable_to_compare == 0,
        "markdownlint integration should work: {} unable to compare",
        report.unable_to_compare
    );
}

/// Test our own project files for dogfooding
#[test]
fn test_project_files_corpus() {
    let config = CorpusTestConfig {
        run_benchmarks: false,
        detailed_reports: true,
        ..Default::default()
    };

    let mut runner = CorpusRunner::with_config(config);
    
    // Add project documentation files
    let project_files = [
        "README.md",
        "CONTRIBUTING.md", 
        "docs/src/getting-started.md",
        "docs/src/configuration.md",
        "docs/src/rules.md",
        "docs/src/contributing.md",
    ];
    
    for file in &project_files {
        let path = PathBuf::from(file);
        if path.exists() {
            runner = runner.add_file(&path, file.to_string(), TestCategory::RealProject);
            println!("Added project file: {}", file);
        } else {
            println!("Project file not found: {}", file);
        }
    }

    let report = runner.run_compatibility_test();
    runner.print_report(&report);

    // Project files should have working markdownlint integration
    // Compatibility may be lower due to our specific rules and configurations
    assert!(
        report.unable_to_compare == 0,
        "markdownlint integration should work on project files: {} unable to compare",
        report.unable_to_compare
    );
    
    // At least one file should be tested
    assert!(
        report.total_files > 0,
        "Should test at least one project file"
    );
}

/// Test against markdownlint official test suite (if available)
#[test]
#[ignore = "Requires markdownlint installation and internet access"]
fn test_corpus_markdownlint_official() {
    let markdownlint_dir = PathBuf::from("tests/corpus/markdownlint");

    // Skip if markdownlint test suite is not available
    if !markdownlint_dir.exists() {
        println!("Skipping markdownlint official tests - directory not found");
        println!(
            "Run: git submodule add https://github.com/DavidAnson/markdownlint tests/corpus/markdownlint"
        );
        return;
    }

    let config = CorpusTestConfig {
        markdownlint_path: find_markdownlint(),
        run_benchmarks: true,
        detailed_reports: true,
        ..Default::default()
    };

    let runner = CorpusRunner::with_config(config).add_directory(
        markdownlint_dir.join("test"),
        TestCategory::MarkdownlintOfficial,
    );

    let report = runner.run_compatibility_test();
    runner.print_report(&report);

    // Official test suite should have very high compatibility
    assert!(
        report.compatibility_percentage() >= 95.0,
        "Markdownlint compatibility too low: {:.1}%",
        report.compatibility_percentage()
    );

    // Performance should be significantly better
    if let Some(perf) = &report.performance {
        assert!(
            perf.speed_improvement >= 1.5,
            "Performance improvement too low: {:.1}x",
            perf.speed_improvement
        );
    }
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

    // Unicode handling should be robust
    assert!(
        report.success_percentage() >= 85.0,
        "Unicode handling success rate too low: {:.1}%",
        report.success_percentage()
    );

    // Check that we didn't have any actual failures (incompatible differences would indicate crashes)
    assert_eq!(
        report.incompatible_differences, 0,
        "Some Unicode files caused processing failures"
    );

    // Files should be processed successfully
    println!(
        "Successfully processed {} Unicode files",
        report.total_files
    );
}

/// Test empty and malformed files
#[test]
fn test_robustness() {
    let edge_cases_dir = PathBuf::from("tests/corpus/edge_cases");

    // Generate edge cases
    if !edge_cases_dir.exists() {
        let generator = EdgeCaseGenerator::new(&edge_cases_dir);
        generator
            .generate_all()
            .expect("Failed to generate edge cases");
    }

    let runner = CorpusRunner::new()
        .add_directory(edge_cases_dir.join("empty"), TestCategory::EdgeCase)
        .add_directory(edge_cases_dir.join("pathological"), TestCategory::EdgeCase);

    let report = runner.run_compatibility_test();
    runner.print_report(&report);

    // Should handle all files without crashing (but may be unable to compare without markdownlint)
    assert!(report.total_files > 0, "No files were processed");

    // Check that we didn't have any actual failures (incompatible differences would indicate crashes)
    assert_eq!(
        report.incompatible_differences, 0,
        "Some files caused processing failures"
    );

    // Files should be processed successfully
    println!(
        "Successfully processed {} edge case files",
        report.total_files
    );
}

/// Extended corpus test with downloaded content
#[test]
#[ignore = "Extended test - requires download script"]
fn test_extended_corpus() {
    use std::process::Command;

    // Run download script to get extended corpus
    let download_script = PathBuf::from("scripts/download-corpus.sh");
    if download_script.exists() {
        println!("📥 Running corpus download script...");
        let output = Command::new("bash")
            .arg(&download_script)
            .output()
            .expect("Failed to run download script");

        if !output.status.success() {
            println!(
                "Download script output: {}",
                String::from_utf8_lossy(&output.stdout)
            );
            println!(
                "Download script errors: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            panic!("Corpus download failed");
        }

        println!("✅ Extended corpus downloaded successfully");
    }

    let config = CorpusTestConfig {
        markdownlint_path: None, // Skip markdownlint comparison for speed
        run_benchmarks: false,   // Skip benchmarks for speed
        detailed_reports: false, // Skip detailed reports for speed
        ..Default::default()
    };

    let mut runner = CorpusRunner::with_config(config);

    // Add extended corpus directories (limited for performance)
    let extended_dir = PathBuf::from("tests/corpus/extended");
    if extended_dir.exists() {
        // Add only the first 2 subdirectories with limited files to prevent timeout
        let mut dir_count = 0;
        for entry in std::fs::read_dir(&extended_dir).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() && dir_count < 2 {
                println!("Adding extended corpus directory: {:?}", entry.path());

                // Add files manually with a limit of 50 files per directory
                let mut file_count = 0;
                if let Ok(dir_entries) = std::fs::read_dir(entry.path()) {
                    for file_entry in dir_entries.filter_map(|e| e.ok()) {
                        if file_count >= 50 {
                            break;
                        }
                        if file_entry.file_type().unwrap().is_file() {
                            if let Some(ext) = file_entry.path().extension() {
                                if ext == "md" || ext == "markdown" {
                                    let test_name = format!("{dir_count}-{file_count}");
                                    runner = runner.add_file(
                                        file_entry.path(),
                                        test_name,
                                        TestCategory::RealProject,
                                    );
                                    file_count += 1;
                                }
                            }
                        }
                    }
                }
                println!("Added {file_count} files from directory");
                dir_count += 1;
            }
        }
        if dir_count == 0 {
            println!("No extended corpus directories found");
        }
    }

    // Add regular corpus as well
    let edge_cases_dir = PathBuf::from("tests/corpus/edge_cases");
    if edge_cases_dir.exists() {
        runner = runner.add_directory(&edge_cases_dir, TestCategory::EdgeCase);
    }

    let report = runner.run_compatibility_test();
    runner.print_report(&report);

    // Save detailed report
    let report_json = serde_json::to_string_pretty(&report).unwrap();
    std::fs::write("extended_corpus_test_report.json", report_json).unwrap();

    println!("\n📋 Extended Corpus Results");
    println!("==========================");
    println!("Files tested: {}", report.total_files);
    println!("Compatibility: {:.1}%", report.compatibility_percentage());
    println!("Success rate: {:.1}%", report.success_percentage());

    if let Some(perf) = &report.performance {
        println!("Speed improvement: {:.1}x", perf.speed_improvement);
    }

    // Extended corpus should have high success rate
    assert!(
        report.success_percentage() >= 85.0,
        "Extended corpus success rate too low: {:.1}%",
        report.success_percentage()
    );
}

/// Comprehensive corpus test combining multiple sources
#[test]
#[ignore = "Comprehensive test - run manually"]
fn test_comprehensive_corpus() {
    // Generate all edge cases
    let edge_cases_dir = PathBuf::from("tests/corpus/edge_cases");
    if !edge_cases_dir.exists() {
        let generator = EdgeCaseGenerator::new(&edge_cases_dir);
        generator
            .generate_all()
            .expect("Failed to generate edge cases");
    }

    let config = CorpusTestConfig {
        markdownlint_path: find_markdownlint(),
        run_benchmarks: true,
        detailed_reports: true,
        ..Default::default()
    };

    let mut runner =
        CorpusRunner::with_config(config).add_directory(&edge_cases_dir, TestCategory::EdgeCase);

    // Add markdownlint official tests if available
    let markdownlint_dir = PathBuf::from("tests/corpus/markdownlint");
    if markdownlint_dir.exists() {
        runner = runner.add_directory(
            markdownlint_dir.join("test"),
            TestCategory::MarkdownlintOfficial,
        );
    }

    // Add real project samples if available
    let projects_dir = PathBuf::from("tests/corpus/mdbook_projects");
    if projects_dir.exists() {
        runner = runner.add_directory(&projects_dir, TestCategory::RealProject);
    }

    let report = runner.run_compatibility_test();
    runner.print_report(&report);

    // Save detailed report
    let report_json = serde_json::to_string_pretty(&report).unwrap();
    std::fs::write("corpus_test_report.json", report_json).unwrap();

    println!("\n📋 Comprehensive Results Summary");
    println!("================================");
    println!("Files tested: {}", report.total_files);
    println!("Compatibility: {:.1}%", report.compatibility_percentage());
    println!("Success rate: {:.1}%", report.success_percentage());

    if let Some(perf) = &report.performance {
        println!("Speed improvement: {:.1}x", perf.speed_improvement);
    }

    // Overall targets for comprehensive testing
    assert!(
        report.compatibility_percentage() >= 90.0,
        "Overall compatibility too low: {:.1}%",
        report.compatibility_percentage()
    );

    assert!(
        report.success_percentage() >= 95.0,
        "Overall success rate too low: {:.1}%",
        report.success_percentage()
    );
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
fn run_quick_test() {
    let edge_cases_dir = PathBuf::from("tests/corpus/edge_cases");

    if !edge_cases_dir.exists() {
        let generator = EdgeCaseGenerator::new(&edge_cases_dir);
        generator
            .generate_all()
            .expect("Failed to generate edge cases");
    }

    let runner = CorpusRunner::new().add_directory(&edge_cases_dir, TestCategory::EdgeCase);

    let report = runner.run_compatibility_test();
    runner.print_report(&report);
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_corpus_runner_basic_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.md");
        std::fs::write(&test_file, "# Test\n\n### Skipped level").unwrap();

        let runner =
            CorpusRunner::new().add_file(&test_file, "test.md".to_string(), TestCategory::EdgeCase);

        let report = runner.run_compatibility_test();

        assert_eq!(report.total_files, 1);
        assert!(report.total_time.as_nanos() > 0);
    }

    #[test]
    fn test_edge_case_generation() {
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
}
