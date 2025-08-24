//! MD015: Removed rule
//!
//! This rule was removed from markdownlint and its functionality merged into MD013.
//! It exists as a placeholder to maintain complete rule numbering.

use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{Document, violation::Violation};
use comrak::nodes::AstNode;

/// Placeholder for removed rule MD015
pub struct MD015;

impl Rule for MD015 {
    fn id(&self) -> &'static str {
        "MD015"
    }

    fn name(&self) -> &'static str {
        "removed"
    }

    fn description(&self) -> &'static str {
        "Removed rule (functionality merged into MD013)"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::deprecated(
            RuleCategory::Formatting,
            "Functionality merged into MD013 (line-length)",
            Some("MD013"),
        )
        .introduced_in("mdbook-lint v0.1.0")
    }

    fn check_with_ast<'a>(
        &self,
        _document: &Document,
        _ast: Option<&'a AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        // Removed rules never produce violations
        Ok(vec![])
    }
}
