use mdbook_lint::rules::MdBookRuleProvider;
use mdbook_lint::{Config, Document, PluginRegistry, StandardRuleProvider};
use std::path::PathBuf;

#[test]
fn test_only_standard_rules_active() {
    let content = r#"# Test Document

This code block is missing a language tag:

```
fn main() {
    println!("Hello, world!");
}
```
"#;

    // Create engine with only standard rules (no MDBOOK rules)
    let mut registry = PluginRegistry::new();
    registry
        .register_provider(Box::new(StandardRuleProvider))
        .unwrap();
    let engine = registry.create_engine().unwrap();

    let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
    let violations = engine.lint_document(&document).unwrap();

    println!(
        "Standard rules only - found {} violations:",
        violations.len()
    );
    for violation in &violations {
        println!("  {}: {}", violation.rule_id, violation.message);
    }

    // Should find MD040 violation (no MDBOOK001 to conflict with)
    let md040_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "MD040").collect();
    let mdbook001_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id == "MDBOOK001")
        .collect();

    assert_eq!(
        md040_violations.len(),
        1,
        "Should have MD040 violation when only standard rules active"
    );
    assert_eq!(
        mdbook001_violations.len(),
        0,
        "Should have no MDBOOK001 violations when only standard rules active"
    );
    assert_eq!(md040_violations[0].line, 5);
}

#[test]
fn test_only_mdbook_rules_active() {
    let content = r#"# Test Document

This code block is missing a language tag:

```
fn main() {
    println!("Hello, world!");
}
```
"#;

    // Create engine with only MDBOOK rules (no standard rules)
    let mut registry = PluginRegistry::new();
    registry
        .register_provider(Box::new(MdBookRuleProvider))
        .unwrap();
    let engine = registry.create_engine().unwrap();

    let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
    let violations = engine.lint_document(&document).unwrap();

    println!("MDBOOK rules only - found {} violations:", violations.len());
    for violation in &violations {
        println!("  {}: {}", violation.rule_id, violation.message);
    }

    // Should find MDBOOK001 violation (no MD040 to conflict with)
    let md040_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "MD040").collect();
    let mdbook001_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id == "MDBOOK001")
        .collect();

    assert_eq!(
        md040_violations.len(),
        0,
        "Should have no MD040 violations when only MDBOOK rules active"
    );
    assert_eq!(
        mdbook001_violations.len(),
        1,
        "Should have MDBOOK001 violation when only MDBOOK rules active"
    );
    assert_eq!(mdbook001_violations[0].line, 5);
}

#[test]
fn test_both_rulesets_active_with_deduplication() {
    let content = r#"# Test Document

This code block is missing a language tag:

```
fn main() {
    println!("Hello, world!");
}
```
"#;

    // Create engine with both standard and MDBOOK rules
    let mut registry = PluginRegistry::new();
    registry
        .register_provider(Box::new(StandardRuleProvider))
        .unwrap();
    registry
        .register_provider(Box::new(MdBookRuleProvider))
        .unwrap();
    let engine = registry.create_engine().unwrap();

    let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
    let violations = engine.lint_document(&document).unwrap();

    println!(
        "Both rulesets active - found {} violations:",
        violations.len()
    );
    for violation in &violations {
        println!("  {}: {}", violation.rule_id, violation.message);
    }

    // Should find only MDBOOK001 violation (MD040 deduplicated)
    let md040_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "MD040").collect();
    let mdbook001_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id == "MDBOOK001")
        .collect();

    assert_eq!(
        md040_violations.len(),
        0,
        "MD040 should be deduplicated when both rulesets active"
    );
    assert_eq!(
        mdbook001_violations.len(),
        1,
        "Should have MDBOOK001 violation (higher precedence)"
    );
    assert_eq!(mdbook001_violations[0].line, 5);
}

#[test]
fn test_config_based_rule_filtering() {
    let content = r#"# Test Document

```
missing language
```
"#;

    let mut registry = PluginRegistry::new();
    registry
        .register_provider(Box::new(StandardRuleProvider))
        .unwrap();
    registry
        .register_provider(Box::new(MdBookRuleProvider))
        .unwrap();
    let engine = registry.create_engine().unwrap();
    let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();

    // Test 1: Disable MD040, enable MDBOOK001
    let mut config = Config::default();
    config.core.disabled_rules.push("MD040".to_string());

    let violations = engine
        .lint_document_with_config(&document, &config.core)
        .unwrap();
    let md040_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "MD040").collect();
    let mdbook001_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id == "MDBOOK001")
        .collect();

    assert_eq!(
        md040_violations.len(),
        0,
        "MD040 should be disabled by config"
    );
    assert_eq!(
        mdbook001_violations.len(),
        1,
        "MDBOOK001 should still be active"
    );

    // Test 2: Disable MDBOOK001, enable MD040
    let mut config = Config::default();
    config.core.disabled_rules.push("MDBOOK001".to_string());

    let violations = engine
        .lint_document_with_config(&document, &config.core)
        .unwrap();
    let md040_violations: Vec<_> = violations.iter().filter(|v| v.rule_id == "MD040").collect();
    let mdbook001_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id == "MDBOOK001")
        .collect();

    assert_eq!(
        md040_violations.len(),
        1,
        "MD040 should be active when MDBOOK001 disabled"
    );
    assert_eq!(
        mdbook001_violations.len(),
        0,
        "MDBOOK001 should be disabled by config"
    );
}

#[test]
fn test_non_overlapping_rules_unaffected() {
    let content = r#"# Test Document

```
missing language
```

- item 1
- item 2
    
More content here.
"#;

    let mut registry = PluginRegistry::new();
    registry
        .register_provider(Box::new(StandardRuleProvider))
        .unwrap();
    registry
        .register_provider(Box::new(MdBookRuleProvider))
        .unwrap();
    let engine = registry.create_engine().unwrap();

    let document = Document::new(content.to_string(), PathBuf::from("test.md")).unwrap();
    let violations = engine.lint_document(&document).unwrap();

    println!("All violations found:");
    for violation in &violations {
        println!(
            "  {}: line {} - {}",
            violation.rule_id, violation.line, violation.message
        );
    }

    // Should have MDBOOK001 (deduplicated MD040) plus potentially other rules
    let code_block_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id == "MD040" || v.rule_id == "MDBOOK001")
        .collect();

    assert_eq!(
        code_block_violations.len(),
        1,
        "Should have exactly 1 code block violation after deduplication"
    );
    assert_eq!(
        code_block_violations[0].rule_id, "MDBOOK001",
        "Should prefer MDBOOK001 over MD040"
    );

    // Other rules should still work normally
    let other_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id != "MD040" && v.rule_id != "MDBOOK001")
        .collect();

    println!("Non-overlapping violations: {}", other_violations.len());
    for violation in &other_violations {
        println!("  {}: {}", violation.rule_id, violation.message);
    }
}
