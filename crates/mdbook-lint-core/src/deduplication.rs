//! Rule deduplication logic to eliminate duplicate violations
//!
//! This module handles deduplication of violations that are reported by multiple rules
//! for the same issue (e.g., MD040 and MDBOOK001 both flagging missing code block languages).

use crate::violation::Violation;
use std::collections::HashMap;

/// Configuration for violation deduplication
#[derive(Debug, Clone)]
pub struct DeduplicationConfig {
    /// Whether deduplication is enabled (default: true)
    pub enabled: bool,
    /// Rule precedence mapping (higher precedence rules win)
    pub rule_precedence: HashMap<String, u32>,
}

impl Default for DeduplicationConfig {
    fn default() -> Self {
        let mut rule_precedence = HashMap::new();

        // Define default precedence: MDBOOK rules have higher precedence than standard MD rules
        // for overlapping functionality since they provide more specific mdbook context
        rule_precedence.insert("MDBOOK001".to_string(), 100); // Higher precedence
        rule_precedence.insert("MD040".to_string(), 50); // Lower precedence

        Self {
            enabled: true,
            rule_precedence,
        }
    }
}

/// Rule overlap definitions - maps which rules check for the same issues
pub struct RuleOverlaps {
    /// Maps rule pairs that check for the same violation type
    /// Key: violation signature, Value: list of rule IDs that can report this violation
    overlaps: HashMap<String, Vec<String>>,
}

impl Default for RuleOverlaps {
    fn default() -> Self {
        let mut overlaps = HashMap::new();

        // MD040 ↔ MDBOOK001: Both check for missing fenced code block language
        overlaps.insert(
            "missing_code_block_language".to_string(),
            vec!["MD040".to_string(), "MDBOOK001".to_string()],
        );

        Self { overlaps }
    }
}

impl RuleOverlaps {
    /// Get overlapping rules for a given rule ID
    pub fn get_overlapping_rules(&self, rule_id: &str) -> Vec<String> {
        for rule_list in self.overlaps.values() {
            if rule_list.contains(&rule_id.to_string()) {
                return rule_list
                    .iter()
                    .filter(|id| *id != rule_id)
                    .cloned()
                    .collect();
            }
        }
        Vec::new()
    }

    /// Check if a violation signature represents a known overlap
    pub fn is_overlapping_violation(&self, violation: &Violation) -> Option<String> {
        // Determine violation signature based on rule ID and violation characteristics
        let signature = self.get_violation_signature(violation);

        if self.overlaps.contains_key(&signature) {
            Some(signature)
        } else {
            None
        }
    }

    /// Generate a signature for a violation to identify overlap types
    fn get_violation_signature(&self, violation: &Violation) -> String {
        match violation.rule_id.as_str() {
            "MD040" | "MDBOOK001" => "missing_code_block_language".to_string(),
            _ => format!("unique_{}", violation.rule_id),
        }
    }
}

/// Deduplicates violations based on configuration and rule precedence
pub fn deduplicate_violations(
    violations: Vec<Violation>,
    config: &DeduplicationConfig,
) -> Vec<Violation> {
    if !config.enabled {
        return violations;
    }

    let overlaps = RuleOverlaps::default();
    let mut deduplicated = Vec::new();
    let mut violation_groups: HashMap<String, Vec<Violation>> = HashMap::new();

    // Group violations by location and type
    for violation in violations {
        let group_key = format!(
            "{}:{}:{}",
            violation.line,
            violation.column,
            overlaps.get_violation_signature(&violation)
        );

        violation_groups
            .entry(group_key)
            .or_default()
            .push(violation);
    }

    // Process each group and select the best violation
    for (_, mut group) in violation_groups {
        if group.len() == 1 {
            // No duplication, keep the violation
            deduplicated.extend(group);
        } else {
            // Multiple violations at same location - apply deduplication

            // Check if this is a known overlap
            let signature = overlaps.get_violation_signature(&group[0]);
            if overlaps.overlaps.contains_key(&signature) {
                // This is a known overlap - select based on precedence
                group.sort_by(|a, b| {
                    let precedence_a = config.rule_precedence.get(&a.rule_id).unwrap_or(&0);
                    let precedence_b = config.rule_precedence.get(&b.rule_id).unwrap_or(&0);
                    precedence_b.cmp(precedence_a) // Higher precedence first
                });

                // Take the highest precedence violation
                deduplicated.push(group.into_iter().next().unwrap());
            } else {
                // Unknown overlap - keep all violations (conservative approach)
                deduplicated.extend(group);
            }
        }
    }

    // Sort by line number to maintain consistent ordering
    deduplicated.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.column.cmp(&b.column)));

    deduplicated
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::violation::{Severity, Violation};

    fn create_test_violation(
        rule_id: &str,
        line: usize,
        column: usize,
        message: &str,
    ) -> Violation {
        Violation {
            rule_id: rule_id.to_string(),
            rule_name: "test".to_string(),
            message: message.to_string(),
            line,
            column,
            severity: Severity::Warning,
        }
    }

    #[test]
    fn test_no_duplicates() {
        let violations = vec![
            create_test_violation("MD001", 1, 1, "Test message 1"),
            create_test_violation("MD002", 2, 1, "Test message 2"),
        ];

        let config = DeduplicationConfig::default();
        let result = deduplicate_violations(violations.clone(), &config);

        assert_eq!(result.len(), 2);
        assert_eq!(result, violations);
    }

    #[test]
    fn test_md040_mdbook001_deduplication() {
        let violations = vec![
            create_test_violation(
                "MD040",
                5,
                1,
                "Fenced code block is missing language specification",
            ),
            create_test_violation(
                "MDBOOK001",
                5,
                1,
                "Code block is missing language tag for syntax highlighting",
            ),
        ];

        let config = DeduplicationConfig::default();
        let result = deduplicate_violations(violations, &config);

        // Should keep only MDBOOK001 (higher precedence)
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].rule_id, "MDBOOK001");
        assert_eq!(result[0].line, 5);
    }

    #[test]
    fn test_multiple_locations() {
        let violations = vec![
            create_test_violation("MD040", 5, 1, "Missing language at line 5"),
            create_test_violation("MDBOOK001", 5, 1, "Missing language at line 5"),
            create_test_violation("MD040", 10, 1, "Missing language at line 10"),
            create_test_violation("MDBOOK001", 10, 1, "Missing language at line 10"),
        ];

        let config = DeduplicationConfig::default();
        let result = deduplicate_violations(violations, &config);

        // Should keep 2 violations (one for each location), both MDBOOK001
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|v| v.rule_id == "MDBOOK001"));
        assert_eq!(result[0].line, 5);
        assert_eq!(result[1].line, 10);
    }

    #[test]
    fn test_deduplication_disabled() {
        let violations = vec![
            create_test_violation("MD040", 5, 1, "Message 1"),
            create_test_violation("MDBOOK001", 5, 1, "Message 2"),
        ];

        let config = DeduplicationConfig {
            enabled: false,
            ..Default::default()
        };

        let result = deduplicate_violations(violations.clone(), &config);

        // Should keep all violations when disabled
        assert_eq!(result.len(), 2);
        assert_eq!(result, violations);
    }

    #[test]
    fn test_custom_precedence() {
        let violations = vec![
            create_test_violation("MD040", 5, 1, "MD040 message"),
            create_test_violation("MDBOOK001", 5, 1, "MDBOOK001 message"),
        ];

        let mut config = DeduplicationConfig::default();
        // Reverse precedence - make MD040 higher
        config.rule_precedence.insert("MD040".to_string(), 200);
        config.rule_precedence.insert("MDBOOK001".to_string(), 100);

        let result = deduplicate_violations(violations, &config);

        // Should keep MD040 (higher precedence in custom config)
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].rule_id, "MD040");
    }

    #[test]
    fn test_rule_overlaps() {
        let overlaps = RuleOverlaps::default();

        let md040_overlaps = overlaps.get_overlapping_rules("MD040");
        assert_eq!(md040_overlaps, vec!["MDBOOK001"]);

        let mdbook001_overlaps = overlaps.get_overlapping_rules("MDBOOK001");
        assert_eq!(mdbook001_overlaps, vec!["MD040"]);

        let no_overlaps = overlaps.get_overlapping_rules("MD001");
        assert!(no_overlaps.is_empty());
    }
}
