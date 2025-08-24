//! MD016: Gap in rule numbering
//!
//! This rule number never existed in markdownlint - it was a gap in the numbering sequence.
//! It exists as a placeholder to maintain complete rule numbering.

use comrak::nodes::AstNode;
use mdbook_lint_core::error::Result;
use mdbook_lint_core::rule::{Rule, RuleMetadata};
use mdbook_lint_core::{Document, violation::Violation};

/// Placeholder for non-existent rule MD016
pub struct MD016;

impl Rule for MD016 {
    fn id(&self) -> &'static str {
        "MD016"
    }

    fn name(&self) -> &'static str {
        "gap"
    }

    fn description(&self) -> &'static str {
        "Gap in rule numbering (never existed)"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::reserved("This rule number never existed in markdownlint numbering")
            .introduced_in("mdbook-lint v0.1.0")
    }

    fn check_with_ast<'a>(
        &self,
        _document: &Document,
        _ast: Option<&'a AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        // Gap rules never produce violations
        Ok(vec![])
    }
}
