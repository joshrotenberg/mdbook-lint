//! MD008: Reserved rule number
//!
//! This rule number was never implemented in the original markdownlint.
//! It exists as a placeholder to maintain complete rule numbering.

use crate::error::Result;
use crate::rule::{Rule, RuleMetadata};
use crate::{Document, violation::Violation};
use comrak::nodes::AstNode;

/// Placeholder for reserved rule MD008
pub struct MD008;

impl Rule for MD008 {
    fn id(&self) -> &'static str {
        "MD008"
    }

    fn name(&self) -> &'static str {
        "reserved"
    }

    fn description(&self) -> &'static str {
        "Reserved rule number (never implemented)"
    }

    fn metadata(&self) -> RuleMetadata {
        RuleMetadata::reserved("This rule number was never implemented in markdownlint")
            .introduced_in("mdbook-lint v0.1.0")
    }

    fn check_with_ast<'a>(
        &self,
        _document: &Document,
        _ast: Option<&'a AstNode<'a>>,
    ) -> Result<Vec<Violation>> {
        // Reserved rules never produce violations
        Ok(vec![])
    }
}
