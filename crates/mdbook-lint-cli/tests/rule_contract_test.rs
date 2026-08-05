//! Contract test for the public rule surface (issue #467).
//!
//! Rule registration, documentation, and the example configuration are one
//! coherent surface. Existing tests prove the example config parses; they do
//! not prove that every documented rule maps to a registered implementation,
//! or that every registered rule is discoverable.
//!
//! This test derives the rule inventory from the providers and checks the other
//! surfaces against it, so drift fails in CI with the exact IDs named.
//!
//! ## Known gaps
//!
//! Surfaces that are currently out of sync are listed explicitly below and
//! asserted by *equality*, not as a floor. Adding a rule without documenting it
//! fails, and documenting one of the listed rules also fails until the list is
//! updated. The lists may only shrink.

use mdbook_lint_core::PluginRegistry;
use mdbook_lint_core::rule::RuleStability;
use mdbook_lint_rulesets::{MdBookRuleProvider, StandardRuleProvider};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Registered rules that have no documentation page yet.
///
/// Tracked by #482. This list may only shrink.
const KNOWN_UNDOCUMENTED: &[&str] = &[
    "CONTENT001",
    "CONTENT002",
    "CONTENT003",
    "CONTENT004",
    "CONTENT005",
    "CONTENT006",
    "CONTENT007",
    "CONTENT009",
    "CONTENT010",
    "CONTENT011",
    "MD060",
    "MDBOOK016",
    "MDBOOK017",
    "MDBOOK021",
    "MDBOOK022",
    "MDBOOK023",
];

/// Rule IDs referenced by the example config that are not registered.
///
/// MD057 is a reserved markdownlint number that was never implemented here, so
/// `standard/mod.rs` skips registering it. The example config documents the
/// number for readers coming from markdownlint.
const KNOWN_UNREGISTERED_IN_EXAMPLE_CONFIG: &[&str] = &["MD057"];

/// Repository root, derived from this crate's manifest directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root should resolve")
}

/// One entry in the rule inventory.
struct RuleEntry {
    name: &'static str,
    description: &'static str,
    stability: RuleStability,
    /// Collection rules analyze several documents together.
    is_collection: bool,
}

/// Build the rule inventory from every provider, document and collection rules alike.
fn inventory() -> BTreeMap<String, RuleEntry> {
    let mut registry = PluginRegistry::new();
    registry
        .register_provider(Box::new(StandardRuleProvider))
        .expect("standard provider registers");
    registry
        .register_provider(Box::new(MdBookRuleProvider))
        .expect("mdbook provider registers");
    #[cfg(feature = "content")]
    registry
        .register_provider(Box::new(mdbook_lint_rulesets::ContentRuleProvider))
        .expect("content provider registers");
    #[cfg(feature = "adr")]
    registry
        .register_provider(Box::new(mdbook_lint_rulesets::AdrRuleProvider))
        .expect("adr provider registers");

    let engine = registry.create_engine().expect("engine builds");
    let mut entries = BTreeMap::new();

    for id in engine.available_rules() {
        let rule = engine
            .registry()
            .get_rule(id)
            .expect("advertised rule resolves");
        entries.insert(
            id.to_string(),
            RuleEntry {
                name: rule.name(),
                description: rule.description(),
                stability: rule.metadata().stability,
                is_collection: false,
            },
        );
    }

    for id in engine.registry().collection_rule_ids() {
        let rule = engine
            .registry()
            .get_collection_rule(id)
            .expect("advertised collection rule resolves");
        entries.insert(
            id.to_string(),
            RuleEntry {
                name: rule.name(),
                description: rule.description(),
                stability: rule.metadata().stability,
                is_collection: true,
            },
        );
    }

    entries
}

/// Documentation subdirectory for a rule ID.
///
/// Checked before the `MD` prefix, since `MDBOOK` also starts with `MD`.
fn docs_subdir(rule_id: &str) -> Option<&'static str> {
    if rule_id.starts_with("MDBOOK") {
        Some("mdbook")
    } else if rule_id.starts_with("CONTENT") {
        Some("content")
    } else if rule_id.starts_with("ADR") {
        Some("adr")
    } else if rule_id.starts_with("MD") {
        Some("standard")
    } else {
        None
    }
}

/// Rule IDs that have a documentation page on disk.
fn documented_rule_ids() -> BTreeSet<String> {
    let rules_dir = repo_root().join("docs/src/rules");
    let mut ids = BTreeSet::new();

    for subdir in ["standard", "mdbook", "content", "adr"] {
        let dir = rules_dir.join(subdir);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue; // A ruleset with no docs directory yet.
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let upper = stem.to_uppercase();
            // Category and index pages are not rule pages.
            if docs_subdir(&upper).is_some()
                && upper
                    .trim_start_matches(|c: char| c.is_ascii_alphabetic())
                    .chars()
                    .all(|c| c.is_ascii_digit())
                && upper.chars().any(|c| c.is_ascii_digit())
            {
                ids.insert(upper);
            }
        }
    }

    ids
}

/// Rule IDs mentioned as configurable sections in the example config.
fn example_config_rule_ids() -> BTreeSet<String> {
    let path = repo_root().join("crates/mdbook-lint-cli/example-mdbook-lint.toml");
    let content = std::fs::read_to_string(&path).expect("example config is readable");

    content
        .lines()
        .filter_map(|line| {
            // Entries appear as commented headers: "# MD009 - Trailing spaces"
            let rest = line.strip_prefix("# ")?;
            let token = rest.split_whitespace().next()?;
            if docs_subdir(token).is_some()
                && token.chars().any(|c| c.is_ascii_digit())
                && token
                    .trim_start_matches(|c: char| c.is_ascii_alphabetic())
                    .chars()
                    .all(|c| c.is_ascii_digit())
            {
                Some(token.to_string())
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn test_inventory_is_not_empty() {
    let inventory = inventory();
    assert!(
        inventory.len() >= 90,
        "expected the full rule inventory, got {} rules",
        inventory.len()
    );
}

#[test]
fn test_every_registered_rule_has_documentation() {
    let inventory = inventory();
    let documented = documented_rule_ids();

    let undocumented: BTreeSet<String> = inventory
        .keys()
        .filter(|id| !documented.contains(*id))
        .cloned()
        .collect();

    let known: BTreeSet<String> = KNOWN_UNDOCUMENTED.iter().map(|s| s.to_string()).collect();

    let newly_undocumented: Vec<&String> = undocumented.difference(&known).collect();
    assert!(
        newly_undocumented.is_empty(),
        "these rules are registered but have no documentation page:\n  {}\n\
         Add docs/src/rules/<ruleset>/<id>.md for each, or add the ID to \
         KNOWN_UNDOCUMENTED with a tracking issue.",
        newly_undocumented
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let now_documented: Vec<&String> = known.difference(&undocumented).collect();
    assert!(
        now_documented.is_empty(),
        "these rules are now documented and must be removed from KNOWN_UNDOCUMENTED:\n  {}",
        now_documented
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
}

#[test]
fn test_every_documentation_page_maps_to_a_registered_rule() {
    let inventory = inventory();
    let documented = documented_rule_ids();

    let orphans: Vec<&String> = documented
        .iter()
        .filter(|id| !inventory.contains_key(*id))
        .collect();

    assert!(
        orphans.is_empty(),
        "these documentation pages describe rules that are not registered:\n  {}\n\
         Either register the rule or remove the page.",
        orphans
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
}

#[test]
fn test_documentation_lives_in_the_ruleset_directory() {
    let inventory = inventory();
    let rules_dir = repo_root().join("docs/src/rules");

    for id in inventory.keys() {
        let subdir = docs_subdir(id).unwrap_or_else(|| panic!("unrecognized rule ID prefix: {id}"));
        let expected = rules_dir
            .join(subdir)
            .join(format!("{}.md", id.to_lowercase()));

        if KNOWN_UNDOCUMENTED.contains(&id.as_str()) {
            continue;
        }
        assert!(
            expected.exists(),
            "{id} should be documented at {}",
            expected.display()
        );
    }
}

#[test]
fn test_example_config_references_only_registered_rules() {
    let inventory = inventory();
    let referenced = example_config_rule_ids();

    assert!(
        !referenced.is_empty(),
        "expected the example config to document rule sections"
    );

    let unregistered: BTreeSet<String> = referenced
        .iter()
        .filter(|id| !inventory.contains_key(*id))
        .cloned()
        .collect();

    let known: BTreeSet<String> = KNOWN_UNREGISTERED_IN_EXAMPLE_CONFIG
        .iter()
        .map(|s| s.to_string())
        .collect();

    let unexpected: Vec<&String> = unregistered.difference(&known).collect();
    assert!(
        unexpected.is_empty(),
        "the example config documents rules that are not registered:\n  {}",
        unexpected
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let now_registered: Vec<&String> = known.difference(&unregistered).collect();
    assert!(
        now_registered.is_empty(),
        "these are now registered and must leave KNOWN_UNREGISTERED_IN_EXAMPLE_CONFIG:\n  {}",
        now_registered
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
}

#[test]
fn test_rule_metadata_is_well_formed() {
    for (id, entry) in inventory() {
        assert!(!entry.name.is_empty(), "{id} has an empty name");
        assert!(
            !entry.description.is_empty(),
            "{id} has an empty description"
        );
        assert!(
            entry
                .name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "{id} name '{}' should be lowercase kebab-case",
            entry.name
        );
        assert!(
            docs_subdir(&id).is_some(),
            "{id} does not match a known rule ID prefix"
        );
    }
}

#[test]
fn test_no_registered_rule_is_reserved() {
    // Reserved numbers are deliberately never registered. If one appears in the
    // inventory, either the rule was implemented (and its metadata should change)
    // or a placeholder was registered by mistake.
    let reserved: Vec<String> = inventory()
        .iter()
        .filter(|(_, entry)| entry.stability == RuleStability::Reserved)
        .map(|(id, _)| id.clone())
        .collect();

    assert!(
        reserved.is_empty(),
        "reserved rules should not be registered: {}",
        reserved.join(", ")
    );
}

#[test]
fn test_collection_rules_are_part_of_the_inventory() {
    // Regression guard: collection rules live in a separate registry list, so a
    // listing built only on document rules silently drops them.
    let inventory = inventory();
    let collection: Vec<&String> = inventory
        .iter()
        .filter(|(_, e)| e.is_collection)
        .map(|(id, _)| id)
        .collect();

    #[cfg(feature = "adr")]
    assert!(
        !collection.is_empty(),
        "expected the cross-document ADR rules in the inventory"
    );
    let _ = collection;
}
