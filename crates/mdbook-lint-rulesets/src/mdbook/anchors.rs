//! Shared heading-anchor helpers for the mdBook rules.
//!
//! MDBOOK002 (same-document anchors) and MDBOOK006 (cross-file anchors) both need
//! the anchor ids mdBook generates for a markdown file's headings. The generation
//! has to stay identical between the two, so it lives here.

use std::collections::HashMap;

/// Generate an anchor ID from heading text (matching mdBook 0.5.x behavior)
///
/// The algorithm:
/// - Alphanumeric characters become lowercase
/// - Hyphens and underscores are preserved as-is
/// - Whitespace becomes hyphens
/// - Other characters (punctuation) are removed
/// - Leading/trailing hyphens are trimmed
/// - Consecutive hyphens are NOT collapsed (mdBook preserves them)
pub(super) fn generate_anchor_id(heading_text: &str) -> String {
    let mut fragment = String::new();

    for ch in heading_text.chars() {
        if ch.is_alphanumeric() {
            fragment.extend(ch.to_lowercase());
        } else if ch == '-' || ch == '_' {
            // Preserve hyphens and underscores as-is
            fragment.push(ch);
        } else if ch.is_whitespace() {
            // Replace whitespace (spaces, tabs) with hyphens
            fragment.push('-');
        }
        // Other characters (punctuation like +, &, etc.) are removed/ignored
    }

    // Remove leading/trailing hyphens only
    // Do NOT consolidate multiple consecutive hyphens - mdBook preserves them
    fragment.trim_matches('-').to_string()
}

/// Extract heading text from an ATX heading line
pub(super) fn extract_atx_heading(line: &str) -> Option<String> {
    if !line.starts_with('#') {
        return None;
    }

    // Count leading hashes
    let hash_count = line.chars().take_while(|&c| c == '#').count();
    if hash_count == 0 || hash_count > 6 {
        return None; // Invalid heading level
    }

    // Extract text after hashes
    let rest = &line[hash_count..];
    let text = if let Some(stripped) = rest.strip_prefix(' ') {
        stripped
    } else {
        rest
    };

    // Remove trailing hashes if present (closed ATX style)
    let text = text.trim_end_matches(['#', ' ']);

    if text.is_empty() {
        return None;
    }

    Some(text.to_string())
}

/// Extract all heading anchors from markdown content, in document order
///
/// Duplicate headings get the same unique ids mdBook generates: the bare anchor
/// first, then `-1`, `-2`, and so on.
pub(super) fn extract_heading_anchors(content: &str) -> Vec<String> {
    let mut anchors = Vec::new();
    let mut anchor_counts: HashMap<String, usize> = HashMap::new();

    for line in content.lines() {
        let line = line.trim();

        if let Some(heading_text) = extract_atx_heading(line) {
            let base_anchor = generate_anchor_id(&heading_text);
            if !base_anchor.is_empty() {
                let count = anchor_counts.entry(base_anchor.clone()).or_insert(0);
                let anchor = if *count == 0 {
                    base_anchor.clone()
                } else {
                    format!("{base_anchor}-{count}")
                };
                *count += 1;
                anchors.push(anchor);
            }
        }
    }

    // TODO: Handle Setext headings (underlined with = or -)
    // This is less common in mdBook but could be added for completeness

    anchors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_anchor_id_basic() {
        assert_eq!(generate_anchor_id("Getting Started"), "getting-started");
        assert_eq!(generate_anchor_id("API Reference"), "api-reference");
        // Punctuation is dropped, but the hyphens it leaves behind are not collapsed
        assert_eq!(generate_anchor_id("C++ & Rust"), "c--rust");
        assert_eq!(generate_anchor_id("snake_case-name"), "snake_case-name");
    }

    #[test]
    fn test_extract_atx_heading_variants() {
        assert_eq!(
            extract_atx_heading("## Getting Started"),
            Some("Getting Started".to_string())
        );
        assert_eq!(
            extract_atx_heading("### Closed ###"),
            Some("Closed".to_string())
        );
        assert_eq!(extract_atx_heading("####### Too deep"), None);
        assert_eq!(extract_atx_heading("Not a heading"), None);
        assert_eq!(extract_atx_heading("## "), None);
    }

    #[test]
    fn test_extract_heading_anchors_dedupes() {
        let content = "# Intro\n\n## Setup\n\n## Setup\n\n## Setup\n";
        assert_eq!(
            extract_heading_anchors(content),
            vec!["intro", "setup", "setup-1", "setup-2"]
        );
    }
}
