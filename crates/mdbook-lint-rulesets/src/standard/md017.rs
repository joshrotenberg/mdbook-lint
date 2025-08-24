//! MD017: Removed rule
//!
//! This rule was removed from markdownlint and its functionality covered by MD018-MD021.
//! It exists as a placeholder to maintain complete rule numbering.

use comrak::nodes::AstNode;
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleCategory, RuleMetadata};
use mdbook_lint_core::{Document, violation::Violation};

/// Placeholder for removed rule MD017
pub struct MD017;

impl Rule for MD017 {
    fn id(&self) -> &'static str {
        "MD017"
    }

    fn name(&self) -> &'static str {
        "removed"
    }

    fn description(&self) -> &'static str {
        "Removed rule (functionality covered by MD018-MD021)"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::deprecated(
            RuleCategory::Structure,
            "Functionality covered by MD018, MD019, MD020, and MD021",
            Some("MD018"),
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
