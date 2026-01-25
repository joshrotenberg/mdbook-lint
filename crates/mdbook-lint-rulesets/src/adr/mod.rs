//! ADR (Architecture Decision Record) linting rules
//!
//! This module provides rules for validating Architecture Decision Records (ADRs)
//! against both the Nygard format and MADR 4.0 format.
//!
//! # Supported Formats
//!
//! ## Nygard Format
//!
//! The original ADR format proposed by Michael Nygard in his blog post
//! "Documenting Architecture Decisions". Key characteristics:
//!
//! - Title: `# N. Title` (e.g., `# 1. Record architecture decisions`)
//! - Date: `Date: YYYY-MM-DD` line after the title
//! - Status: `## Status` section with status value
//! - Required sections: Context, Decision, Consequences
//!
//! Example:
//! ```markdown
//! # 1. Record architecture decisions
//!
//! Date: 2024-01-15
//!
//! ## Status
//!
//! Accepted
//!
//! ## Context
//!
//! We need to record the architectural decisions made on this project.
//!
//! ## Decision
//!
//! We will use Architecture Decision Records, as described by Michael Nygard.
//!
//! ## Consequences
//!
//! See Michael Nygard's article for more details.
//! ```
//!
//! ## MADR 4.0 Format
//!
//! Markdown Any Decision Records (MADR) version 4.0 uses YAML frontmatter
//! for metadata and a slightly different structure. Key characteristics:
//!
//! - YAML frontmatter with `status` and `date` fields
//! - Simple H1 title (no number prefix required)
//! - Different section names (Context and Problem Statement, Decision Outcome)
//!
//! Example:
//! ```markdown
//! ---
//! status: accepted
//! date: 2024-01-15
//! decision-makers:
//!   - Alice Smith
//! ---
//!
//! # Use PostgreSQL for persistence
//!
//! ## Context and Problem Statement
//!
//! We need to select a database for the application.
//!
//! ## Decision Outcome
//!
//! Chosen option: PostgreSQL.
//! ```
//!
//! # Available Rules
//!
//! | Rule | Name | Description |
//! |------|------|-------------|
//! | ADR001 | adr-title-format | Title follows appropriate format for ADR type |
//! | ADR002 | adr-required-status | Status is defined (section or frontmatter) |
//! | ADR003 | adr-required-date | Date is defined (line or frontmatter) |
//! | ADR004 | adr-required-context | Context section is present |
//! | ADR005 | adr-required-decision | Decision section is present |
//! | ADR006 | adr-required-consequences | Consequences section is present (Nygard only) |
//! | ADR007 | adr-valid-status | Status value is recognized |
//! | ADR008 | adr-date-format | Date follows ISO 8601 format |
//! | ADR009 | adr-filename-matches-number | Filename matches ADR number (Nygard only) |
//!
//! ## Collection Rules (Multi-Document)
//!
//! These rules analyze multiple ADR documents together:
//!
//! | Rule | Name | Description |
//! |------|------|-------------|
//! | ADR010 | adr-superseded-has-replacement | Superseded ADRs reference replacement |
//! | ADR011 | adr-sequential-numbering | ADR numbers are sequential with no gaps |
//! | ADR012 | adr-no-duplicate-numbers | Each ADR number is unique |
//! | ADR013 | adr-valid-adr-links | Links to other ADRs point to existing files |
//!
//! ## Content Quality Rules
//!
//! | Rule | Name | Description |
//! |------|------|-------------|
//! | ADR014 | adr-non-empty-sections | Required sections should have meaningful content |
//! | ADR015 | adr-decision-drivers-format | Decision Drivers should be a bullet list (MADR) |
//! | ADR016 | adr-considered-options-format | Considered Options should list at least 2 options |
//! | ADR017 | adr-consequences-structure | Consequences should distinguish good/bad outcomes (MADR) |
//!
//! # Configuration
//!
//! Rules can be configured in your `.mdbook-lint.toml`:
//!
//! ```toml
//! # Provider-level format setting (affects all ADR rules)
//! [ADR]
//! format = "auto"  # "auto", "nygard", or "madr"
//! ```

pub mod format;
pub mod frontmatter;

mod adr001;
mod adr002;
mod adr003;
mod adr004;
mod adr005;
mod adr006;
mod adr007;
mod adr008;
mod adr009;
mod adr010;
mod adr011;
mod adr012;
mod adr013;
mod adr014;
mod adr015;
mod adr016;
mod adr017;

use crate::{RuleProvider, RuleRegistry};

pub use adr001::Adr001;
pub use adr002::Adr002;
pub use adr003::Adr003;
pub use adr004::Adr004;
pub use adr005::Adr005;
pub use adr006::Adr006;
pub use adr007::Adr007;
pub use adr008::Adr008;
pub use adr009::Adr009;
pub use adr010::Adr010;
pub use adr011::Adr011;
pub use adr012::Adr012;
pub use adr013::Adr013;
pub use adr014::Adr014;
pub use adr015::Adr015;
pub use adr016::Adr016;
pub use adr017::Adr017;
pub use format::AdrFormat;
pub use frontmatter::AdrFrontmatter;

/// Provider for ADR (Architecture Decision Record) rules
///
/// This provider registers rules for validating ADRs against both the
/// Nygard format and MADR 4.0 format. Format detection is automatic
/// by default but can be configured.
pub struct AdrRuleProvider;

impl RuleProvider for AdrRuleProvider {
    fn provider_id(&self) -> &'static str {
        "adr"
    }

    fn description(&self) -> &'static str {
        "Architecture Decision Record linting rules (ADR001-ADR017)"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn register_rules(&self, registry: &mut RuleRegistry) {
        // Single-document rules
        registry.register(Box::new(Adr001::default()));
        registry.register(Box::new(Adr002::default()));
        registry.register(Box::new(Adr003::default()));
        registry.register(Box::new(Adr004::default()));
        registry.register(Box::new(Adr005::default()));
        registry.register(Box::new(Adr006::default()));
        registry.register(Box::new(Adr007::default()));
        registry.register(Box::new(Adr008::default()));
        registry.register(Box::new(Adr009::default()));

        // Content quality rules
        registry.register(Box::new(Adr014::default()));
        registry.register(Box::new(Adr015::default()));
        registry.register(Box::new(Adr016::default()));
        registry.register(Box::new(Adr017::default()));

        // Collection rules (multi-document)
        registry.register_collection_rule(Box::new(Adr010));
        registry.register_collection_rule(Box::new(Adr011));
        registry.register_collection_rule(Box::new(Adr012));
        registry.register_collection_rule(Box::new(Adr013));
    }

    fn rule_ids(&self) -> Vec<&'static str> {
        vec![
            "ADR001", "ADR002", "ADR003", "ADR004", "ADR005", "ADR006", "ADR007", "ADR008",
            "ADR009", "ADR010", "ADR011", "ADR012", "ADR013", "ADR014", "ADR015", "ADR016",
            "ADR017",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::Document;
    use mdbook_lint_core::rule::Rule;
    use std::path::PathBuf;

    fn create_test_document(content: &str) -> Document {
        // Use a path that matches ADR directory detection
        Document::new(content.to_string(), PathBuf::from("adr/0001-test-adr.md")).unwrap()
    }

    #[test]
    fn test_provider_metadata() {
        let provider = AdrRuleProvider;
        assert_eq!(provider.provider_id(), "adr");
        assert!(provider.description().contains("ADR"));
        assert_eq!(provider.version(), "0.1.0");
    }

    #[test]
    fn test_provider_rule_ids() {
        let provider = AdrRuleProvider;
        let ids = provider.rule_ids();
        assert!(ids.contains(&"ADR001"));
        assert!(ids.contains(&"ADR002"));
        assert!(ids.contains(&"ADR003"));
    }

    #[test]
    fn test_valid_nygard_adr() {
        let content = r#"# 1. Use Rust for implementation

Date: 2024-01-15

## Status

Accepted

## Context

We need to choose a programming language.

## Decision

We will use Rust.

## Consequences

Team will need Rust training.
"#;
        let doc = create_test_document(content);

        let rule1 = Adr001::default();
        let rule2 = Adr002::default();
        let rule3 = Adr003::default();

        assert!(rule1.check(&doc).unwrap().is_empty());
        assert!(rule2.check(&doc).unwrap().is_empty());
        assert!(rule3.check(&doc).unwrap().is_empty());
    }

    #[test]
    fn test_valid_madr_adr() {
        let content = r#"---
status: accepted
date: 2024-01-15
decision-makers:
  - Alice Smith
---

# Use PostgreSQL for persistence

## Context and Problem Statement

We need to select a database.

## Decision Outcome

Chosen option: PostgreSQL.
"#;
        let doc = create_test_document(content);

        let rule1 = Adr001::default();
        let rule2 = Adr002::default();
        let rule3 = Adr003::default();

        assert!(rule1.check(&doc).unwrap().is_empty());
        assert!(rule2.check(&doc).unwrap().is_empty());
        assert!(rule3.check(&doc).unwrap().is_empty());
    }

    #[test]
    fn test_invalid_nygard_adr_all_rules_fail() {
        // Missing number in title, no status section, no date
        let content = r#"# Use Rust

## Context

We need a language.
"#;
        let doc = create_test_document(content);

        let rule1 = Adr001::default();
        let rule2 = Adr002::default();
        let rule3 = Adr003::default();

        assert!(!rule1.check(&doc).unwrap().is_empty(), "ADR001 should fail");
        assert!(!rule2.check(&doc).unwrap().is_empty(), "ADR002 should fail");
        assert!(!rule3.check(&doc).unwrap().is_empty(), "ADR003 should fail");
    }

    #[test]
    fn test_invalid_madr_adr_missing_fields() {
        // Frontmatter present but missing status and date
        let content = r#"---
decision-makers:
  - Alice
---

# Title

## Context

Content.
"#;
        let doc = create_test_document(content);

        let rule1 = Adr001::default();
        let rule2 = Adr002::default();
        let rule3 = Adr003::default();

        assert!(
            rule1.check(&doc).unwrap().is_empty(),
            "ADR001 should pass (title exists)"
        );
        assert!(
            !rule2.check(&doc).unwrap().is_empty(),
            "ADR002 should fail (no status)"
        );
        assert!(
            !rule3.check(&doc).unwrap().is_empty(),
            "ADR003 should fail (no date)"
        );
    }
}
