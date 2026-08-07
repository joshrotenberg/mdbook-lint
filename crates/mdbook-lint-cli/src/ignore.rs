//! CLI-facing helpers for the global `ignore-paths` configuration.
//!
//! The matching semantics live in `mdbook_lint_core::ignore`, shared with the
//! mdBook preprocessor and the MDBOOK005 rule so the three cannot drift. This
//! module just adapts that shared matcher to the CLI's file-filtering use case.

pub use mdbook_lint_core::ignore::path_is_ignored;

use std::path::PathBuf;

/// Remove files matching any of the configured ignore-paths patterns.
pub fn filter_ignored_paths(files: &mut Vec<PathBuf>, patterns: &[String]) {
    if patterns.is_empty() {
        return;
    }
    files.retain(|path| !path_is_ignored(path, patterns));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_ignored_paths() {
        let mut files = vec![
            PathBuf::from("docs/keep.md"),
            PathBuf::from("vendor/skip.md"),
            PathBuf::from("drafts/wip.md"),
            PathBuf::from("notes.backup.md"),
        ];
        let patterns = vec![
            "vendor/".to_string(),
            "drafts/".to_string(),
            "*.backup.md".to_string(),
        ];
        filter_ignored_paths(&mut files, &patterns);
        assert_eq!(files, vec![PathBuf::from("docs/keep.md")]);
    }
}
