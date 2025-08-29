//! Integration tests for auto-fix functionality
//!
//! These tests verify the --fix, --fix-unsafe, and --dry-run flags work correctly
//! including backup creation, fix application, and proper exit codes.

mod common;

use common::*;
use predicates::prelude::*;
use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_fix_flag_applies_fixes() {
    // Test that --fix applies fixable violations
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.md");

    fs::write(
        &test_file,
        "# Test Document  \n\nThis has trailing spaces.   \nMore content here.\n",
    )
    .unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--fix")
        .arg(&test_file)
        .assert();

    // Should succeed (exit 0 when all fixable issues are resolved)
    assert
        .success()
        .stdout(contains("Fixed"))
        .stdout(contains("Applied"));

    // Verify trailing spaces were fixed
    let fixed_content = fs::read_to_string(&test_file).unwrap();
    assert!(
        !fixed_content.contains("spaces.   "),
        "Trailing spaces should be removed"
    );
    assert!(
        fixed_content.contains("spaces."),
        "Content should remain intact"
    );

    // Verify backup was created
    let backup_file = test_file.with_extension("md.bak");
    assert!(backup_file.exists(), "Backup file should be created");

    let backup_content = fs::read_to_string(&backup_file).unwrap();
    assert!(
        backup_content.contains("spaces.   "),
        "Backup should contain original content"
    );
}

#[test]
fn test_fix_with_remaining_violations() {
    // Test fix behavior when some violations cannot be fixed
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.md");

    fs::write(
        &test_file,
        "# Test Document  \n\n\n\nThis has trailing spaces.   \n\n\n",
    )
    .unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--fix")
        .arg(&test_file)
        .assert();

    // Should succeed but show remaining violations
    assert
        .success()
        .stdout(contains("Fixed"))
        .stdout(contains("Applied"))
        .stdout(contains("Found"))
        .stdout(contains("violation"));

    // Verify trailing spaces were fixed but multiple blank lines remain
    let fixed_content = fs::read_to_string(&test_file).unwrap();
    assert!(
        !fixed_content.contains("spaces.   "),
        "Trailing spaces should be removed"
    );
    assert!(
        fixed_content.contains("\n\n\n"),
        "Multiple blank lines should remain (not fixable)"
    );
}

#[test]
fn test_fix_with_fail_on_warnings() {
    // Test that --fix with --fail-on-warnings exits with code 1 when issues remain
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.md");

    fs::write(
        &test_file,
        "# Test Document  \n\n\n\nThis has trailing spaces.   \n",
    )
    .unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--fix")
        .arg("--fail-on-warnings")
        .arg(&test_file)
        .assert();

    // Should exit with code 1 due to remaining violations
    assert
        .code(1)
        .stdout(contains("Fixed"))
        .stdout(contains("Found"))
        .stdout(contains("violation"));
}

#[test]
fn test_dry_run_flag() {
    // Test that --dry-run shows what would be fixed without applying changes
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.md");

    let original_content = "# Test Document  \n\nThis has trailing spaces.   \n";
    fs::write(&test_file, original_content).unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--fix")
        .arg("--dry-run")
        .arg(&test_file)
        .assert();

    // Should succeed and show what would be fixed
    assert.success().stdout(contains("Would fix"));

    // Verify file content is unchanged
    let content_after = fs::read_to_string(&test_file).unwrap();
    assert_eq!(
        content_after, original_content,
        "File should not be modified in dry-run mode"
    );

    // Verify no backup was created
    let backup_file = test_file.with_extension("md.bak");
    assert!(
        !backup_file.exists(),
        "No backup should be created in dry-run mode"
    );
}

#[test]
fn test_fix_unsafe_flag() {
    // Test that --fix-unsafe applies all fixes (including potentially unsafe ones)
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.md");

    fs::write(
        &test_file,
        "# Test Document  \n\nThis has trailing spaces.   \nMore content.\n",
    )
    .unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--fix-unsafe")
        .arg(&test_file)
        .assert();

    // Should succeed
    assert
        .success()
        .stdout(contains("Fixed"))
        .stdout(contains("Applied"));

    // Verify fixes were applied
    let fixed_content = fs::read_to_string(&test_file).unwrap();
    assert!(
        !fixed_content.contains("spaces.   "),
        "Trailing spaces should be removed"
    );
}

#[test]
fn test_dry_run_with_fix_unsafe() {
    // Test --dry-run with --fix-unsafe
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.md");

    let original_content = "# Test Document  \n\nThis has trailing spaces.   \n";
    fs::write(&test_file, original_content).unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--fix-unsafe")
        .arg("--dry-run")
        .arg(&test_file)
        .assert();

    // Should succeed
    assert.success().stdout(contains("Would fix"));

    // Verify file content is unchanged
    let content_after = fs::read_to_string(&test_file).unwrap();
    assert_eq!(
        content_after, original_content,
        "File should not be modified in dry-run mode"
    );
}

#[test]
fn test_backup_flag_disabled() {
    // Test --fix with --no-backup
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.md");

    fs::write(
        &test_file,
        "# Test Document  \n\nThis has trailing spaces.   \n",
    )
    .unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--fix")
        .arg("--no-backup")
        .arg(&test_file)
        .assert();

    // Should succeed
    assert.success().stdout(contains("Fixed"));

    // Verify no backup was created
    let backup_file = test_file.with_extension("md.bak");
    assert!(
        !backup_file.exists(),
        "No backup should be created when --no-backup"
    );

    // Verify fixes were still applied
    let fixed_content = fs::read_to_string(&test_file).unwrap();
    assert!(
        !fixed_content.contains("spaces.   "),
        "Trailing spaces should be removed"
    );
}

#[test]
fn test_fix_multiple_files() {
    // Test fixing multiple files at once
    let temp_dir = TempDir::new().unwrap();
    let file1 = temp_dir.path().join("file1.md");
    let file2 = temp_dir.path().join("file2.md");

    fs::write(&file1, "# File One  \n\nTrailing spaces here.   \n").unwrap();

    fs::write(&file2, "# File Two  \n\nMore trailing spaces.    \n").unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--fix")
        .arg(&file1)
        .arg(&file2)
        .assert();

    // Should succeed and show fixes for both files
    assert
        .success()
        .stdout(contains("Fixed"))
        .stdout(contains("Applied"))
        .stdout(contains("2 file(s)"));

    // Verify both files were fixed
    let content1 = fs::read_to_string(&file1).unwrap();
    let content2 = fs::read_to_string(&file2).unwrap();

    assert!(
        !content1.contains("here.   "),
        "File 1 trailing spaces should be removed"
    );
    assert!(
        !content2.contains("spaces.    "),
        "File 2 trailing spaces should be removed"
    );

    // Verify backups were created
    assert!(file1.with_extension("md.bak").exists());
    assert!(file2.with_extension("md.bak").exists());
}

#[test]
fn test_fix_directory_recursively() {
    // Test fixing all markdown files in a directory
    let temp_dir = TempDir::new().unwrap();
    let sub_dir = temp_dir.path().join("subdir");
    fs::create_dir_all(&sub_dir).unwrap();

    let file1 = temp_dir.path().join("file1.md");
    let file2 = sub_dir.join("file2.md");

    fs::write(&file1, "# File One  \n\nTrailing spaces.   \n").unwrap();

    fs::write(&file2, "# File Two  \n\nMore trailing spaces.    \n").unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--fix")
        .arg(temp_dir.path())
        .assert();

    // Should succeed
    assert
        .success()
        .stdout(contains("Fixed"))
        .stdout(contains("Applied"));

    // Verify both files were fixed
    let content1 = fs::read_to_string(&file1).unwrap();
    let content2 = fs::read_to_string(&file2).unwrap();

    assert!(!content1.contains("spaces.   "));
    assert!(!content2.contains("spaces.    "));
}

#[test]
fn test_fix_no_fixable_violations() {
    // Test fix behavior when no violations have fixes available
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.md");

    // Create content that has violations but no fixable ones (MD033 - inline HTML has no fix)
    fs::write(
        &test_file,
        "# Test Document\n\nThis has <b>inline HTML</b> which violates MD033.\n",
    )
    .unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--fix")
        .arg(&test_file)
        .assert();

    // Get output before consuming the assert
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    // Should succeed but show no fixes were applied
    assert
        .success()
        .stdout(contains("Found"))
        .stdout(contains("violation"));

    // Should not show "Fixed" or "Applied" messages since no fixes were available
    assert!(
        !stdout.contains("Fixed"),
        "Should not show 'Fixed' when no fixes applied"
    );
    assert!(
        !stdout.contains("Applied"),
        "Should not show 'Applied' when no fixes applied"
    );

    // Verify file content is unchanged
    let content = fs::read_to_string(&test_file).unwrap();
    assert!(
        content.contains("<b>inline HTML</b>"),
        "Content should be unchanged when no fixes applied"
    );

    // Verify no backup was created since no changes were made
    let backup_file = test_file.with_extension("md.bak");
    assert!(
        !backup_file.exists(),
        "No backup should be created when no fixes applied"
    );
}

#[test]
fn test_fix_error_validation() {
    // Test error cases for fix flags

    // Test --dry-run without --fix or --fix-unsafe should fail
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.md");
    fs::write(&test_file, "# Test\nContent\n").unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--dry-run")
        .arg(&test_file)
        .assert();

    // Should fail with validation error
    assert
        .code(1)
        .stderr(contains("--dry-run requires either --fix or --fix-unsafe"));
}

#[test]
fn test_fix_clean_file() {
    // Test fixing a file that has no violations
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("clean.md");

    fs::write(
        &test_file,
        "# Clean Document\n\nThis file has no violations.\n\n```rust\nfn main() {}\n```\n",
    )
    .unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--fix")
        .arg(&test_file)
        .assert();

    // Get output before consuming the assert
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    // Should succeed with no issues found
    assert
        .success()
        .stdout(contains("✅ No issues found").or(contains("Found 0 violation")));

    // Should not show any fix-related output
    assert!(
        !stdout.contains("Fixed"),
        "Should not show 'Fixed' for clean files"
    );
    assert!(
        !stdout.contains("Applied"),
        "Should not show 'Applied' for clean files"
    );

    // Verify no backup was created
    let backup_file = test_file.with_extension("md.bak");
    assert!(
        !backup_file.exists(),
        "No backup should be created for clean files"
    );
}

#[test]
fn test_fix_exit_codes() {
    // Test various exit code scenarios with fixes
    let temp_dir = TempDir::new().unwrap();

    // Test 1: All violations fixed - should exit 0
    let test_file1 = temp_dir.path().join("all_fixable.md");
    fs::write(
        &test_file1,
        "# Test\nTrailing spaces.   \nMore trailing.  \n",
    )
    .unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--fix")
        .arg(&test_file1)
        .assert();

    // Should exit 0 when all issues can be fixed
    assert.success();

    // Test 2: Some violations remain after fix - should exit 0 (warnings don't fail by default)
    let test_file2 = temp_dir.path().join("mixed.md");
    fs::write(&test_file2, "# Test\n\n\n\nTrailing spaces.   \n").unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--fix")
        .arg(&test_file2)
        .assert();

    // Should exit 0 even with remaining warnings
    assert.success();

    // Test 3: Remaining violations with --fail-on-warnings - should exit 1
    let test_file3 = temp_dir.path().join("mixed2.md");
    fs::write(&test_file3, "# Test\n\n\n\nTrailing spaces.   \n").unwrap();

    let assert = cli_command()
        .arg("lint")
        .arg("--fix")
        .arg("--fail-on-warnings")
        .arg(&test_file3)
        .assert();

    // Should exit 1 with --fail-on-warnings
    assert.code(1);
}
