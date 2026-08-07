//! Shared matching for `ignore-paths` / `ignore_patterns` style configuration.
//!
//! The CLI's `ignore-paths`, the mdBook preprocessor's `ignore-paths`, and the
//! MDBOOK005 rule's `ignore_patterns` all need to agree on what a glob pattern
//! matches. This module is the single implementation so the three cannot
//! drift apart.

use std::path::Path;

/// Return true if `path` matches any of the given ignore glob patterns.
///
/// Matching is intentionally forgiving so the patterns behave the way users
/// expect from `.gitignore`-style configuration:
/// - a trailing `/` marks a directory prefix (`target/` behaves like `target/**`),
/// - a pattern without any `/` also matches anywhere in the tree
///   (`*.backup.md` matches `sub/dir/file.backup.md`),
/// - `*` does not cross path separators, but `**` does.
///
/// Paths are normalized (leading `./` stripped, backslashes converted to `/`)
/// before matching so results are consistent across platforms.
pub fn path_is_ignored(path: &Path, patterns: &[String]) -> bool {
    use glob::{MatchOptions, Pattern};

    if patterns.is_empty() {
        return false;
    }

    let normalized = path
        .to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string();

    let options = MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: false,
    };

    for pattern in patterns {
        let mut pat = pattern.replace('\\', "/");
        pat = pat.trim_start_matches("./").to_string();
        // A trailing slash means "everything under this directory".
        if pat.ends_with('/') {
            pat.push_str("**");
        }

        // A bare name or relative pattern should also match deeper in the tree.
        let mut candidates = vec![pat.clone()];
        if !pat.starts_with("**/") {
            candidates.push(format!("**/{pat}"));
        }

        for candidate in candidates {
            if let Ok(compiled) = Pattern::new(&candidate)
                && compiled.matches_with(&normalized, options)
            {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_path_is_ignored() {
        let p = |s: &str| PathBuf::from(s);

        // Directory prefix (trailing slash)
        let pats = vec!["vendor/".to_string()];
        assert!(path_is_ignored(&p("vendor/skip.md"), &pats));
        assert!(path_is_ignored(&p("./vendor/nested/skip.md"), &pats));
        assert!(!path_is_ignored(&p("docs/keep.md"), &pats));

        // Bare suffix glob matches at any depth
        let pats = vec!["*.backup.md".to_string()];
        assert!(path_is_ignored(&p("notes.backup.md"), &pats));
        assert!(path_is_ignored(&p("a/b/notes.backup.md"), &pats));
        assert!(!path_is_ignored(&p("notes.md"), &pats));

        // Bare name matches anywhere
        let pats = vec!["not-found.md".to_string()];
        assert!(path_is_ignored(&p("not-found.md"), &pats));
        assert!(path_is_ignored(&p("src/deep/not-found.md"), &pats));

        // Explicit ** prefix
        let pats = vec!["**/generated.md".to_string()];
        assert!(path_is_ignored(&p("x/y/generated.md"), &pats));

        // Empty patterns never match
        assert!(!path_is_ignored(&p("anything.md"), &[]));
    }

    #[test]
    fn test_path_is_ignored_normalizes_separators() {
        // mdBook reports chapter source_path with native separators, so a
        // Windows path must match a pattern written with forward slashes.
        // Normalization happens on the string, so this holds on every platform.
        let pats = vec!["generated/".to_string()];
        assert!(path_is_ignored(&PathBuf::from(r"generated\api.md"), &pats));
        assert!(path_is_ignored(
            &PathBuf::from(r"generated\deep\nested.md"),
            &pats
        ));
        assert!(!path_is_ignored(&PathBuf::from(r"guide\intro.md"), &pats));
    }

    #[test]
    fn test_path_is_ignored_star_does_not_cross_separator() {
        // `*` is anchored to the directory it's written in, so a deeper path
        // is not matched.
        let pats = vec!["sub/*.md".to_string()];
        assert!(path_is_ignored(&PathBuf::from("sub/chapter1.md"), &pats));
        assert!(!path_is_ignored(
            &PathBuf::from("sub/deep/chapter1.md"),
            &pats
        ));
    }
}
