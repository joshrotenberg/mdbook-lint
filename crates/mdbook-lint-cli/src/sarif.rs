//! SARIF v2.1.0 report generation.
//!
//! The CI documentation and the companion GitHub Action have advertised SARIF
//! output for some time (issue #471). This module implements it against the
//! OASIS SARIF 2.1.0 schema, deriving rule descriptors from `RuleMetadata` so
//! the report stays consistent with the registry.

use mdbook_lint_core::{LintEngine, Severity, Violation};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;

/// Schema URL emitted in the report.
const SARIF_SCHEMA: &str = "https://json.schemastore.org/sarif-2.1.0.json";

/// SARIF specification version this report conforms to.
const SARIF_VERSION: &str = "2.1.0";

/// Tool home page, reported as the driver's `informationUri`.
const INFORMATION_URI: &str = "https://github.com/joshrotenberg/mdbook-lint";

/// Base URL of the published rule reference.
const DOCS_BASE_URI: &str = "https://joshrotenberg.github.io/mdbook-lint/rules";

/// Map a violation severity onto a SARIF result level.
///
/// SARIF defines `none`, `note`, `warning`, and `error`. Info maps to `note`,
/// which is how code-scanning surfaces advisory results.
fn sarif_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "note",
    }
}

/// Documentation sub-path for a rule ID.
///
/// `MDBOOK` is checked before `MD`, since the former also starts with `MD`.
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

/// Published documentation URL for a rule, when its ruleset is recognized.
fn help_uri(rule_id: &str) -> Option<String> {
    let subdir = docs_subdir(rule_id)?;
    Some(format!(
        "{DOCS_BASE_URI}/{subdir}/{}.html",
        rule_id.to_lowercase()
    ))
}

/// Normalize a file path into a SARIF `artifactLocation` URI.
///
/// SARIF consumers, GitHub code scanning in particular, expect paths relative
/// to the repository root with forward slashes. Absolute paths under the
/// working directory are made relative; anything else is passed through with
/// separators normalized.
fn artifact_uri(path: &str) -> String {
    let path = Path::new(path);

    let relative = std::env::current_dir()
        .ok()
        .and_then(|cwd| path.strip_prefix(&cwd).ok().map(Path::to_path_buf))
        .unwrap_or_else(|| path.to_path_buf());

    relative
        .to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string()
}

/// Build a rule descriptor for the driver's `rules` array.
///
/// Metadata comes from the registry where the rule is known. A violation always
/// carries an ID, name, and message, so a descriptor can still be produced for
/// a rule the registry cannot resolve.
fn rule_descriptor(rule_id: &str, rule_name: &str, engine: &LintEngine) -> Value {
    let mut descriptor = json!({
        "id": rule_id,
        "name": rule_name,
    });

    if let Some(uri) = help_uri(rule_id) {
        descriptor["helpUri"] = json!(uri);
    }

    // Document rules and collection rules live in separate registry lists.
    let registry = engine.registry();
    let resolved = registry
        .get_rule(rule_id)
        .map(|r| (r.description().to_string(), r.metadata()))
        .or_else(|| {
            registry
                .get_collection_rule(rule_id)
                .map(|r| (r.description().to_string(), r.metadata()))
        });

    if let Some((description, metadata)) = resolved {
        descriptor["shortDescription"] = json!({ "text": description });

        let mut properties = json!({
            "category": format!("{:?}", metadata.category),
            "stability": format!("{:?}", metadata.stability),
            "runsByDefault": metadata.runs_by_default(),
        });
        if let Some(version) = metadata.introduced_in {
            properties["introducedIn"] = json!(version);
        }
        if metadata.deprecated {
            properties["deprecated"] = json!(true);
            if let Some(reason) = metadata.deprecated_reason {
                properties["deprecatedReason"] = json!(reason);
            }
        }
        descriptor["properties"] = properties;
    }

    descriptor
}

/// Build a complete SARIF v2.1.0 report for the given violations.
///
/// Rule descriptors are emitted for the rules that actually produced results,
/// and each result references its descriptor by `ruleIndex`.
pub fn build_report(violations_by_file: &[(String, Vec<Violation>)], engine: &LintEngine) -> Value {
    // Stable descriptor ordering, so repeated runs over the same input produce
    // byte-identical reports.
    let mut rule_names: BTreeMap<&str, &str> = BTreeMap::new();
    for (_, violations) in violations_by_file {
        for violation in violations {
            rule_names
                .entry(violation.rule_id.as_str())
                .or_insert(violation.rule_name.as_str());
        }
    }

    let rule_index: BTreeMap<&str, usize> = rule_names
        .keys()
        .enumerate()
        .map(|(index, id)| (*id, index))
        .collect();

    let rules: Vec<Value> = rule_names
        .iter()
        .map(|(id, name)| rule_descriptor(id, name, engine))
        .collect();

    // Results are sorted by location and then rule ID. The engine does not
    // guarantee an order for violations reported at the same position, so two
    // runs over identical input could otherwise emit results in different
    // orders, producing spurious diffs and defeating caching.
    let mut ordered: Vec<(String, usize, usize, &str, Value)> = violations_by_file
        .iter()
        .flat_map(|(file_path, violations)| {
            let uri = artifact_uri(file_path);
            let rule_index = &rule_index;
            violations.iter().map(move |violation| {
                let mut region = json!({ "startLine": violation.line.max(1) });
                if violation.column > 0 {
                    region["startColumn"] = json!(violation.column);
                }

                let mut result = json!({
                    "ruleId": violation.rule_id,
                    "level": sarif_level(violation.severity),
                    "message": { "text": violation.message },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": { "uri": uri },
                            "region": region,
                        }
                    }],
                });

                if let Some(index) = rule_index.get(violation.rule_id.as_str()) {
                    result["ruleIndex"] = json!(index);
                }

                (
                    uri.clone(),
                    violation.line,
                    violation.column,
                    violation.rule_id.as_str(),
                    result,
                )
            })
        })
        .collect();

    ordered.sort_by(|a, b| {
        (&a.0, a.1, a.2, a.3)
            .cmp(&(&b.0, b.1, b.2, b.3))
            .then_with(|| a.4.to_string().cmp(&b.4.to_string()))
    });

    let results: Vec<Value> = ordered
        .into_iter()
        .map(|(_, _, _, _, value)| value)
        .collect();

    json!({
        "$schema": SARIF_SCHEMA,
        "version": SARIF_VERSION,
        "runs": [{
            "tool": {
                "driver": {
                    "name": "mdbook-lint",
                    "version": env!("CARGO_PKG_VERSION"),
                    "semanticVersion": env!("CARGO_PKG_VERSION"),
                    "informationUri": INFORMATION_URI,
                    "rules": rules,
                }
            },
            "results": results,
        }]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mdbook_lint_core::PluginRegistry;
    use mdbook_lint_rulesets::{MdBookRuleProvider, StandardRuleProvider};

    fn test_engine() -> LintEngine {
        let mut registry = PluginRegistry::new();
        registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();
        registry
            .register_provider(Box::new(MdBookRuleProvider))
            .unwrap();
        registry.create_engine().unwrap()
    }

    fn violation(rule_id: &str, rule_name: &str, line: usize, column: usize) -> Violation {
        Violation {
            rule_id: rule_id.to_string(),
            rule_name: rule_name.to_string(),
            message: format!("{rule_id} fired"),
            line,
            column,
            severity: Severity::Warning,
            fix: None,
        }
    }

    #[test]
    fn test_report_has_required_sarif_envelope() {
        let report = build_report(&[], &test_engine());

        assert_eq!(report["version"], SARIF_VERSION);
        assert_eq!(report["$schema"], SARIF_SCHEMA);

        let driver = &report["runs"][0]["tool"]["driver"];
        assert_eq!(driver["name"], "mdbook-lint");
        assert_eq!(driver["informationUri"], INFORMATION_URI);
        assert!(!driver["version"].as_str().unwrap().is_empty());

        // An empty run is still a valid report with an empty results array.
        assert_eq!(report["runs"][0]["results"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_severity_maps_to_sarif_levels() {
        assert_eq!(sarif_level(Severity::Error), "error");
        assert_eq!(sarif_level(Severity::Warning), "warning");
        assert_eq!(sarif_level(Severity::Info), "note");
    }

    #[test]
    fn test_results_carry_location_and_rule_index() {
        let violations = vec![(
            "docs/guide.md".to_string(),
            vec![violation("MD001", "heading-increment", 4, 2)],
        )];
        let report = build_report(&violations, &test_engine());

        let result = &report["runs"][0]["results"][0];
        assert_eq!(result["ruleId"], "MD001");
        assert_eq!(result["level"], "warning");
        assert_eq!(result["message"]["text"], "MD001 fired");

        let location = &result["locations"][0]["physicalLocation"];
        assert_eq!(location["artifactLocation"]["uri"], "docs/guide.md");
        assert_eq!(location["region"]["startLine"], 4);
        assert_eq!(location["region"]["startColumn"], 2);

        // ruleIndex must point at the matching descriptor.
        let index = result["ruleIndex"].as_u64().unwrap() as usize;
        let rules = report["runs"][0]["tool"]["driver"]["rules"]
            .as_array()
            .unwrap();
        assert_eq!(rules[index]["id"], "MD001");
    }

    #[test]
    fn test_rule_descriptors_are_deduplicated_and_sourced_from_metadata() {
        let violations = vec![
            (
                "a.md".to_string(),
                vec![
                    violation("MD001", "heading-increment", 1, 1),
                    violation("MD001", "heading-increment", 9, 1),
                ],
            ),
            (
                "b.md".to_string(),
                vec![violation("MD009", "no-trailing-spaces", 2, 1)],
            ),
        ];
        let report = build_report(&violations, &test_engine());

        let rules = report["runs"][0]["tool"]["driver"]["rules"]
            .as_array()
            .unwrap();
        assert_eq!(rules.len(), 2, "one descriptor per distinct rule");

        let md001 = rules.iter().find(|r| r["id"] == "MD001").unwrap();
        assert_eq!(md001["name"], "heading-increment");
        // Description and properties come from the registry, not the violation.
        assert!(
            md001["shortDescription"]["text"]
                .as_str()
                .is_some_and(|s| !s.is_empty())
        );
        assert_eq!(md001["properties"]["category"], "Structure");
        assert_eq!(md001["properties"]["stability"], "Stable");
        assert_eq!(
            md001["helpUri"],
            "https://joshrotenberg.github.io/mdbook-lint/rules/standard/md001.html"
        );

        assert_eq!(report["runs"][0]["results"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_unknown_rule_still_produces_a_descriptor() {
        // A violation from a rule the registry cannot resolve must not be dropped.
        let violations = vec![(
            "a.md".to_string(),
            vec![violation("ZZZ999", "mystery", 1, 1)],
        )];
        let report = build_report(&violations, &test_engine());

        let rules = report["runs"][0]["tool"]["driver"]["rules"]
            .as_array()
            .unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0]["id"], "ZZZ999");
        assert!(rules[0]["shortDescription"].is_null());
        assert!(
            rules[0]["helpUri"].is_null(),
            "no docs path for unknown prefix"
        );
        assert_eq!(report["runs"][0]["results"][0]["ruleId"], "ZZZ999");
    }

    #[test]
    fn test_help_uri_routes_by_ruleset() {
        assert!(help_uri("MD001").unwrap().contains("/standard/md001.html"));
        assert!(
            help_uri("MDBOOK005")
                .unwrap()
                .contains("/mdbook/mdbook005.html"),
            "MDBOOK must not be treated as MD"
        );
        assert!(help_uri("CONTENT004").unwrap().contains("/content/"));
        assert!(help_uri("ADR001").unwrap().contains("/adr/"));
        assert!(help_uri("ZZZ999").is_none());
    }

    #[test]
    fn test_artifact_uri_normalizes_separators() {
        assert_eq!(artifact_uri("docs/guide.md"), "docs/guide.md");
        assert_eq!(artifact_uri("./docs/guide.md"), "docs/guide.md");
        assert_eq!(artifact_uri(r"docs\guide.md"), "docs/guide.md");
    }

    #[test]
    fn test_results_are_sorted_by_location_then_rule() {
        // The engine does not order violations reported at the same position, so
        // the report imposes its own order. Without this, two runs over identical
        // input could emit results in different orders.
        let violations = vec![
            (
                "b.md".to_string(),
                vec![violation("MD009", "no-trailing-spaces", 5, 1)],
            ),
            (
                "a.md".to_string(),
                vec![
                    violation("MD060", "z-rule", 3, 1),
                    violation("MD001", "heading-increment", 3, 1),
                    violation("MD001", "heading-increment", 1, 1),
                ],
            ),
        ];
        let report = build_report(&violations, &test_engine());
        let results = report["runs"][0]["results"].as_array().unwrap();

        let order: Vec<(String, u64, String)> = results
            .iter()
            .map(|r| {
                (
                    r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                        .as_str()
                        .unwrap()
                        .to_string(),
                    r["locations"][0]["physicalLocation"]["region"]["startLine"]
                        .as_u64()
                        .unwrap(),
                    r["ruleId"].as_str().unwrap().to_string(),
                )
            })
            .collect();

        assert_eq!(
            order,
            vec![
                ("a.md".to_string(), 1, "MD001".to_string()),
                ("a.md".to_string(), 3, "MD001".to_string()),
                ("a.md".to_string(), 3, "MD060".to_string()),
                ("b.md".to_string(), 5, "MD009".to_string()),
            ]
        );
    }

    #[test]
    fn test_report_is_deterministic() {
        let violations = vec![(
            "a.md".to_string(),
            vec![
                violation("MD009", "no-trailing-spaces", 1, 1),
                violation("MD001", "heading-increment", 2, 1),
            ],
        )];
        let engine = test_engine();

        let first = serde_json::to_string(&build_report(&violations, &engine)).unwrap();
        let second = serde_json::to_string(&build_report(&violations, &engine)).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn test_zero_column_is_omitted_rather_than_emitted_as_zero() {
        // SARIF columns are 1-based; 0 is not a legal startColumn.
        let violations = vec![("a.md".to_string(), vec![violation("MD001", "x", 3, 0)])];
        let report = build_report(&violations, &test_engine());

        let region = &report["runs"][0]["results"][0]["locations"][0]["physicalLocation"]["region"];
        assert_eq!(region["startLine"], 3);
        assert!(region["startColumn"].is_null());
    }
}
