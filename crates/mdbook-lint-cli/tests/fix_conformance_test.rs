//! Conformance tests for automatic fixes (issue #456).
//!
//! Every case here applies fixes through `LintEngine::apply_fixes` and then
//! re-lints the result. The rule-level tests that existed before asserted the
//! shape of `Fix` objects without ever applying them, which is how three
//! corruption bugs shipped:
//!
//! - MD011 left a stray `]` because the end position pointed at the bracket
//!   rather than one past it, and fix ranges are end-exclusive.
//! - MD034 wrapped Liquid syntax into the URL, and rejected multibyte URLs
//!   because the span mixed a char index with a byte length.
//! - MD047 replaced the final newline with itself, reporting a file as fixed
//!   while leaving it unchanged.
//!
//! Applying a fix and re-linting catches all three; inspecting the `Fix` does
//! not.

use mdbook_lint::Config;
use mdbook_lint_core::{Document, PluginRegistry};
use mdbook_lint_rulesets::StandardRuleProvider;
use std::path::PathBuf;

/// Lint `content` with only `rule_id` enabled.
fn lint(content: &str, rule_id: &str) -> (Vec<mdbook_lint_core::Violation>, Config) {
    let mut config = Config::default();
    config.core.enabled_rules = vec![rule_id.to_string()];

    let mut registry = PluginRegistry::new();
    registry
        .register_provider(Box::new(StandardRuleProvider))
        .unwrap();
    let engine = registry.create_engine().unwrap();

    let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
    let violations = engine
        .lint_document_with_config(&document, &config.core)
        .unwrap();
    (violations, config)
}

/// Apply every available fix for `rule_id` once, returning the new content.
fn fix_once(content: &str, rule_id: &str) -> String {
    let (violations, _) = lint(content, rule_id);
    if violations.is_empty() {
        return content.to_string();
    }

    let mut registry = PluginRegistry::new();
    registry
        .register_provider(Box::new(StandardRuleProvider))
        .unwrap();
    let engine = registry.create_engine().unwrap();

    let (fixed, _unfixed) = engine.apply_fixes(content, &violations);
    fixed
}

/// Apply fixes, assert the result matches `expected`, and assert the rule is
/// then silent and a second pass changes nothing.
fn assert_fixes_to(content: &str, rule_id: &str, expected: &str) {
    let fixed = fix_once(content, rule_id);
    assert_eq!(
        fixed, expected,
        "{rule_id} produced unexpected output\n  input:    {content:?}\n  expected: {expected:?}\n  actual:   {fixed:?}"
    );

    let (remaining, _) = lint(&fixed, rule_id);
    assert!(
        remaining.is_empty(),
        "{rule_id} still reports {} violation(s) after fixing: {:?}",
        remaining.len(),
        remaining.iter().map(|v| &v.message).collect::<Vec<_>>()
    );

    let twice = fix_once(&fixed, rule_id);
    assert_eq!(twice, fixed, "{rule_id} fix is not idempotent");
}

#[test]
fn test_md011_fixes_the_complete_reversed_link() {
    // The stray ']' case from the issue.
    assert_fixes_to(
        "(text)[https://example.com]\n",
        "MD011",
        "[text](https://example.com)\n",
    );
}

#[test]
fn test_md011_fixes_reversed_link_mid_sentence() {
    assert_fixes_to(
        "Before (text)[url] after.\n",
        "MD011",
        "Before [text](url) after.\n",
    );
}

#[test]
fn test_md011_handles_multibyte_text_before_the_link() {
    // Columns are char-based, so preceding multibyte text must not shift the span.
    assert_fixes_to(
        "Préface (café)[https://example.com] fin.\n",
        "MD011",
        "Préface [café](https://example.com) fin.\n",
    );
}

#[test]
fn test_md034_does_not_consume_liquid_syntax() {
    assert_fixes_to(
        "See https://example.com{% if foo %}\n",
        "MD034",
        "See <https://example.com>{% if foo %}\n",
    );
}

#[test]
fn test_md034_does_not_consume_handlebars_syntax() {
    assert_fixes_to(
        "See https://example.com{{ foo }}\n",
        "MD034",
        "See <https://example.com>{{ foo }}\n",
    );
}

#[test]
fn test_md034_fixes_multibyte_urls() {
    // Previously rejected: the span mixed a char index with a byte length.
    assert_fixes_to(
        "Préface https://example.com/café\n",
        "MD034",
        "Préface <https://example.com/café>\n",
    );
}

#[test]
fn test_md034_leaves_already_wrapped_urls_alone() {
    let content = "See <https://example.com> here.\n";
    let (violations, _) = lint(content, "MD034");
    assert!(violations.is_empty(), "got: {violations:?}");
}

#[test]
fn test_md047_reduces_trailing_newlines_to_one() {
    for input in [
        "# Title\n\nBody.\n\n",
        "# Title\n\nBody.\n\n\n",
        "# Title\n\nBody.\n\n\n\n\n",
    ] {
        assert_fixes_to(input, "MD047", "# Title\n\nBody.\n");
    }
}

#[test]
fn test_md047_adds_a_missing_trailing_newline() {
    assert_fixes_to("# Title\n\nBody.", "MD047", "# Title\n\nBody.\n");
}

#[test]
fn test_md047_leaves_a_correct_ending_alone() {
    let content = "# Title\n\nBody.\n";
    let (violations, _) = lint(content, "MD047");
    assert!(violations.is_empty(), "got: {violations:?}");
}

#[test]
fn test_md047_handles_multibyte_final_line() {
    assert_fixes_to(
        "# Title\n\nPréface café\n\n",
        "MD047",
        "# Title\n\nPréface café\n",
    );
}

#[test]
fn test_fixes_are_idempotent_across_rules() {
    // Applying each rule's fixes in turn must converge, not oscillate.
    let content = "(text)[https://example.com/café]\n\n\n";

    let mut current = content.to_string();
    for rule in ["MD011", "MD034", "MD047"] {
        current = fix_once(&current, rule);
    }

    // A second full pass must be a no-op.
    let mut second = current.clone();
    for rule in ["MD011", "MD034", "MD047"] {
        second = fix_once(&second, rule);
    }
    assert_eq!(second, current, "combined fixes did not converge");

    assert!(current.ends_with("]\n") || current.ends_with(")\n"));
    assert!(
        !current.ends_with("\n\n"),
        "trailing newlines were not normalized: {current:?}"
    );
}

#[test]
fn test_multiple_violations_on_one_line_all_fixed() {
    assert_fixes_to("(a)[u1] and (b)[u2]\n", "MD011", "[a](u1) and [b](u2)\n");
}
