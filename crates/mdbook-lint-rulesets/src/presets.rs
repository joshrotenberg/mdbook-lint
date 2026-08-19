//! Built-in rule presets.
//!
//! Presets are curated product policy rather than intrinsic rule metadata. The
//! ordered lists in this module are therefore the single source of truth used
//! by configuration, CLI selection, and rule discovery.

use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use std::{fmt, str::FromStr};

/// Rules in the low-noise baseline for incremental CI adoption.
///
/// Membership is deliberately explicit: newly stable rules are not added
/// automatically. Changes to this list are user-visible and require release
/// notes plus corpus review.
pub const BASELINE_RULE_IDS: &[&str] = &["MD001", "MD003", "MD009", "MD010", "MD011", "MD014"];

/// A built-in, named rule preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RulePreset {
    /// Low-noise mechanical checks suitable for initial CI adoption.
    Baseline,
}

impl RulePreset {
    /// Canonical configuration and CLI name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
        }
    }

    /// Short explanation shown by rule discovery.
    pub const fn description(self) -> &'static str {
        match self {
            Self::Baseline => {
                "Low-noise stable structural, syntax, and whitespace checks for incremental CI adoption"
            }
        }
    }

    /// Ordered rule IDs in this preset.
    pub const fn rule_ids(self) -> &'static [&'static str] {
        match self {
            Self::Baseline => BASELINE_RULE_IDS,
        }
    }

    /// Whether a rule belongs to this preset.
    pub fn contains(self, rule_id: &str) -> bool {
        self.rule_ids().contains(&rule_id)
    }
}

impl fmt::Display for RulePreset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for RulePreset {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "baseline" => Ok(Self::Baseline),
            _ => Err(format!(
                "unknown preset '{value}'; available presets: baseline"
            )),
        }
    }
}

impl<'de> Deserialize<'de> for RulePreset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_membership_is_an_ordered_stable_contract() {
        assert_eq!(
            RulePreset::Baseline.rule_ids(),
            ["MD001", "MD003", "MD009", "MD010", "MD011", "MD014"]
        );
    }

    #[test]
    fn preset_names_round_trip() {
        let preset: RulePreset = "baseline".parse().unwrap();
        assert_eq!(preset, RulePreset::Baseline);
        assert_eq!(preset.to_string(), "baseline");
    }
}
