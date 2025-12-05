//! Cargo-style colored output for mdbook-lint
//!
//! This module provides formatted, colored output similar to Cargo/rustc.

use anstream::println;
use anstyle::{AnsiColor, Style};
use mdbook_lint_core::violation::{Severity, Violation};
use std::fs;

/// Styles for different output elements
struct OutputStyles {
    error: Style,
    warning: Style,
    info: Style,
    success: Style,
    bold: Style,
    blue: Style,
}

impl Default for OutputStyles {
    fn default() -> Self {
        Self {
            error: Style::new().fg_color(Some(AnsiColor::Red.into())).bold(),
            warning: Style::new().fg_color(Some(AnsiColor::Yellow.into())).bold(),
            info: Style::new().fg_color(Some(AnsiColor::Cyan.into())).bold(),
            success: Style::new().fg_color(Some(AnsiColor::Green.into())).bold(),
            bold: Style::new().bold(),
            blue: Style::new().fg_color(Some(AnsiColor::Blue.into())).bold(),
        }
    }
}

/// Formats and prints violations in cargo-style output
pub fn print_cargo_style(violations_by_file: &[(String, Vec<Violation>)]) {
    let styles = OutputStyles::default();

    for (file_path, violations) in violations_by_file {
        // Try to read file content for context
        let file_content = fs::read_to_string(file_path).ok();
        let lines: Vec<&str> = file_content
            .as_ref()
            .map(|c| c.lines().collect())
            .unwrap_or_default();

        for violation in violations {
            print_violation(&styles, file_path, violation, &lines);
        }
    }
}

/// Print a single violation with cargo-style formatting
fn print_violation(styles: &OutputStyles, file_path: &str, violation: &Violation, lines: &[&str]) {
    let (severity_style, severity_label) = match violation.severity {
        Severity::Error => (styles.error, "error"),
        Severity::Warning => (styles.warning, "warning"),
        Severity::Info => (styles.info, "info"),
    };

    // Line 1: severity[RULE_ID]: message
    println!(
        "{severity_style}{severity_label}[{}]{severity_style:#}: {}",
        violation.rule_id, violation.message
    );

    // Line 2: --> file:line:column
    println!(
        "  {blue}-->{blue:#} {file_path}:{line}:{col}",
        blue = styles.blue,
        line = violation.line,
        col = violation.column
    );

    // Show source context if available
    if let Some(source_line) = lines.get(violation.line.saturating_sub(1)) {
        let line_num = violation.line;
        let line_num_width = line_num.to_string().len().max(2);

        // Empty line with pipe
        println!(
            "  {blue}{:>width$} |{blue:#}",
            "",
            width = line_num_width,
            blue = styles.blue
        );

        // Source line
        println!(
            "  {blue}{:>width$} |{blue:#} {source_line}",
            line_num,
            width = line_num_width,
            blue = styles.blue
        );

        // Underline/caret line
        let caret_pos = violation.column.saturating_sub(1);
        let underline = build_underline(source_line, caret_pos, &violation.rule_name);
        println!(
            "  {blue}{:>width$} |{blue:#} {severity_style}{underline}{severity_style:#}",
            "",
            width = line_num_width,
            blue = styles.blue,
            severity_style = severity_style
        );
    }

    // Empty line between violations
    println!();
}

/// Build the underline string with carets pointing to the issue
fn build_underline(source_line: &str, caret_pos: usize, rule_name: &str) -> String {
    // Calculate the visual position accounting for tabs
    let mut visual_pos = 0;
    for (i, ch) in source_line.chars().enumerate() {
        if i >= caret_pos {
            break;
        }
        visual_pos += if ch == '\t' { 4 } else { 1 };
    }

    // Determine underline length - try to underline the relevant token
    let underline_len = determine_underline_length(source_line, caret_pos);

    format!(
        "{:>width$}{} {}",
        "",
        "^".repeat(underline_len),
        rule_name,
        width = visual_pos
    )
}

/// Determine how many characters to underline
fn determine_underline_length(source_line: &str, start_pos: usize) -> usize {
    let chars: Vec<char> = source_line.chars().collect();
    if start_pos >= chars.len() {
        return 1;
    }

    // For most violations, underline until whitespace or end of line
    let mut len = 0;
    for ch in chars.iter().skip(start_pos) {
        if ch.is_whitespace() && len > 0 {
            break;
        }
        len += 1;
        // Cap at reasonable length
        if len >= 40 {
            break;
        }
    }

    len.max(1)
}

/// Print summary line
pub fn print_summary(total_violations: usize, error_count: usize, warning_count: usize) {
    let styles = OutputStyles::default();

    if total_violations == 0 {
        println!(
            "{success}No issues found{success:#}",
            success = styles.success
        );
    } else {
        let mut parts = Vec::new();

        if error_count > 0 {
            parts.push(format!(
                "{error}{} error(s){error:#}",
                error_count,
                error = styles.error
            ));
        }

        if warning_count > 0 {
            parts.push(format!(
                "{warning}{} warning(s){warning:#}",
                warning_count,
                warning = styles.warning
            ));
        }

        let info_count = total_violations - error_count - warning_count;
        if info_count > 0 {
            parts.push(format!("{} info", info_count));
        }

        println!(
            "{bold}Found:{bold:#} {}",
            parts.join(", "),
            bold = styles.bold
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_underline_length() {
        assert_eq!(determine_underline_length("hello world", 0), 5);
        assert_eq!(determine_underline_length("hello world", 6), 5);
        assert_eq!(determine_underline_length("x", 0), 1);
        assert_eq!(determine_underline_length("", 0), 1);
    }

    #[test]
    fn test_build_underline() {
        let underline = build_underline("# Hello", 0, "heading-increment");
        assert!(underline.contains("^"));
        assert!(underline.contains("heading-increment"));
    }
}
