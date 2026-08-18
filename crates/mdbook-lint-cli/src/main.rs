mod config;
mod ignore;
#[cfg(feature = "lsp")]
mod lsp_server;
mod output;
mod preprocessor;
mod rustdoc;
mod sarif;

use ignore::filter_ignored_paths;

use config::Config;

use clap::{Parser, Subcommand, ValueEnum, builder::styling};
use mdbook_lint_core::{
    Document, PluginRegistry, Severity,
    error::Result,
    rule::{RuleCategory, RuleStability},
};
#[cfg(feature = "adr")]
use mdbook_lint_rulesets::AdrRuleProvider;
#[cfg(feature = "content")]
use mdbook_lint_rulesets::ContentRuleProvider;
use mdbook_lint_rulesets::{MdBookRuleProvider, RulePreset, StandardRuleProvider};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tabled::{Table, Tabled, settings::Style};

// Cargo-style help coloring
const STYLES: styling::Styles = styling::Styles::styled()
    .header(styling::AnsiColor::Green.on_default().bold())
    .usage(styling::AnsiColor::Green.on_default().bold())
    .literal(styling::AnsiColor::Cyan.on_default().bold())
    .placeholder(styling::AnsiColor::Cyan.on_default());

#[derive(Parser)]
#[command(name = "mdbook-lint")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "Josh Rotenberg <joshrotenberg@gmail.com>")]
#[command(about = "A markdown linter for mdBook projects")]
#[command(styles = STYLES)]
struct Cli {
    /// Use verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Do not print informational messages
    #[arg(short, long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run as mdBook preprocessor (reads from stdin, writes to stdout)
    Preprocessor,

    /// Lint markdown files directly
    Lint {
        /// Markdown files or directories to lint
        files: Vec<String>,
        /// Path to configuration file (TOML, YAML, or JSON)
        #[arg(short, long)]
        config: Option<String>,
        /// Use only standard rules (MD001-MD059), exclude mdBook rules
        #[arg(long)]
        standard_only: bool,
        /// Use only mdBook rules (MDBOOK001-004), exclude standard rules
        #[arg(long)]
        mdbook_only: bool,
        /// Fail on warnings (in addition to errors)
        #[arg(long)]
        fail_on_warnings: bool,
        /// Enable markdownlint compatibility mode (disables rules that are disabled by default in markdownlint)
        #[arg(long)]
        markdownlint_compatible: bool,
        /// Output format
        #[arg(long, value_enum, default_value = "default")]
        output: OutputFormat,
        /// Automatically fix issues where possible
        #[arg(long)]
        fix: bool,
        /// Apply all fixes including potentially unsafe ones (implies --fix)
        #[arg(long)]
        fix_unsafe: bool,
        /// Preview fixes without applying them (can be used with --fix or --fix-unsafe)
        #[arg(long)]
        dry_run: bool,
        /// Disable backup file creation when fixing
        #[arg(long)]
        no_backup: bool,
        /// Disable specific rules (comma-separated list, e.g., MD001,MD002)
        #[arg(long, value_delimiter = ',')]
        disable: Option<Vec<String>>,
        /// Enable only specific rules (comma-separated list, e.g., MD001,MD002)
        #[arg(long, value_delimiter = ',')]
        enable: Option<Vec<String>>,
        /// Use a curated rule preset
        #[arg(
            long,
            value_name = "NAME",
            conflicts_with_all = ["enable", "standard_only", "mdbook_only"]
        )]
        preset: Option<RulePreset>,
        /// Control colored output (auto, always, never)
        #[arg(long, value_enum, default_value = "auto")]
        color: ColorChoice,
        /// Write the report to a file instead of stdout (used with --output sarif)
        #[arg(long, value_name = "PATH")]
        output_file: Option<PathBuf>,
    },

    /// Automatically fix issues in markdown files (shorthand for `lint --fix`)
    Fix {
        /// Markdown files or directories to fix
        files: Vec<String>,
        /// Path to configuration file (TOML, YAML, or JSON)
        #[arg(short, long)]
        config: Option<String>,
        /// Use only standard rules (MD001-MD059), exclude mdBook rules
        #[arg(long)]
        standard_only: bool,
        /// Use only mdBook rules (MDBOOK001-004), exclude standard rules
        #[arg(long)]
        mdbook_only: bool,
        /// Apply all fixes including potentially unsafe ones
        #[arg(long, name = "unsafe")]
        fix_unsafe: bool,
        /// Preview fixes without applying them
        #[arg(long)]
        dry_run: bool,
        /// Disable backup file creation when fixing
        #[arg(long)]
        no_backup: bool,
        /// Disable specific rules (comma-separated list, e.g., MD001,MD002)
        #[arg(long, value_delimiter = ',')]
        disable: Option<Vec<String>>,
        /// Enable only specific rules (comma-separated list, e.g., MD001,MD002)
        #[arg(long, value_delimiter = ',')]
        enable: Option<Vec<String>>,
        /// Use a curated rule preset
        #[arg(
            long,
            value_name = "NAME",
            conflicts_with_all = ["enable", "standard_only", "mdbook_only"]
        )]
        preset: Option<RulePreset>,
        /// Control colored output (auto, always, never)
        #[arg(long, value_enum, default_value = "auto")]
        color: ColorChoice,
    },

    /// List available rules by category
    Rules {
        /// Show detailed information about each rule
        #[arg(short, long)]
        detailed: bool,
        /// Filter by rule category
        #[arg(short, long)]
        category: Option<String>,
        /// Show only rules from specific provider
        #[arg(short, long)]
        provider: Option<String>,
        /// Show only standard rules (MD001-MD059)
        #[arg(long)]
        standard_only: bool,
        /// Show only mdBook-specific rules
        #[arg(long)]
        mdbook_only: bool,
        /// Show only rules in a curated preset
        #[arg(
            long,
            value_name = "NAME",
            conflicts_with_all = ["standard_only", "mdbook_only"]
        )]
        preset: Option<RulePreset>,
        /// Output format for rule information
        #[arg(short, long, value_enum, default_value = "default")]
        format: RulesFormat,
        /// Output in JSON format (shorthand for --format json)
        #[arg(long)]
        json: bool,
    },

    /// Check configuration file validity
    Check {
        /// Path to configuration file to validate
        config: PathBuf,
    },

    /// Generate default configuration file
    Init {
        /// Output format for configuration
        #[arg(short, long, value_enum, default_value = "toml")]
        format: ConfigFormat,
        /// Output file path (defaults to mdbook-lint.{format})
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Include all available rules in generated config
        #[arg(long)]
        include_all: bool,
    },

    /// Check if this preprocessor supports a renderer
    Supports {
        /// The renderer to check
        renderer: String,
    },

    /// Run as Language Server Protocol (LSP) server
    #[cfg(feature = "lsp")]
    Lsp {
        /// Use stdio for communication (default)
        #[arg(long)]
        stdio: bool,
        /// TCP port to listen on (alternative to stdio)
        #[arg(long, conflicts_with = "stdio")]
        port: Option<u16>,
    },

    /// Lint markdown in Rust documentation comments (//!)
    Rustdoc {
        /// Rust source files or directories to lint
        paths: Vec<String>,
        /// Path to configuration file (TOML, YAML, or JSON)
        #[arg(short, long)]
        config: Option<String>,
        /// Fail on warnings (in addition to errors)
        #[arg(long)]
        fail_on_warnings: bool,
        /// Output format
        #[arg(long, value_enum, default_value = "default")]
        output: OutputFormat,
        /// Disable specific rules (comma-separated list, e.g., MD001,MD002)
        #[arg(long, value_delimiter = ',')]
        disable: Option<Vec<String>>,
        /// Enable only specific rules (comma-separated list, e.g., MD001,MD002)
        #[arg(long, value_delimiter = ',')]
        enable: Option<Vec<String>>,
        /// Use a curated rule preset
        #[arg(long, value_name = "NAME", conflicts_with = "enable")]
        preset: Option<RulePreset>,
        /// Control colored output (auto, always, never)
        #[arg(long, value_enum, default_value = "auto")]
        color: ColorChoice,
    },
}

#[derive(ValueEnum, Clone, PartialEq, Debug)]
enum ColorChoice {
    /// Automatically detect if colors should be used
    Auto,
    /// Always use colors
    Always,
    /// Never use colors
    Never,
}

#[derive(ValueEnum, Clone, PartialEq, Debug)]
enum OutputFormat {
    /// Default human-readable format
    Default,
    /// JSON format for machine processing
    Json,
    /// GitHub Actions format
    Github,
    /// SARIF v2.1.0 for code scanning tools
    Sarif,
}

#[derive(ValueEnum, Clone, PartialEq, Debug)]
enum ConfigFormat {
    /// TOML format (recommended)
    Toml,
    /// YAML format
    Yaml,
    /// JSON format
    Json,
}

#[derive(ValueEnum, Clone, PartialEq, Debug)]
enum RulesFormat {
    /// Default human-readable format
    Default,
    /// JSON format for machine processing
    Json,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRuleProvider {
    provider_id: String,
    version: String,
    description: String,
    rules: Vec<JsonRule>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRule {
    id: String,
    name: String,
    description: String,
    category: JsonRuleCategory,
    stability: JsonRuleStability,
    deprecated: bool,
    deprecated_reason: Option<String>,
    replacement: Option<String>,
    introduced_in: Option<String>,
    can_fix: bool,
}

#[derive(Serialize, Deserialize, Debug)]
enum JsonRuleCategory {
    Structure,
    Formatting,
    Content,
    Links,
    Accessibility,
    MdBook,
}

#[derive(Serialize, Deserialize, Debug)]
enum JsonRuleStability {
    Stable,
    Experimental,
    Deprecated,
    Reserved,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRulesOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    preset: Option<JsonPreset>,
    total_rules: usize,
    providers: Vec<JsonRuleProvider>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonPreset {
    name: String,
    description: String,
    rule_ids: Vec<String>,
}

impl From<&RuleCategory> for JsonRuleCategory {
    fn from(category: &RuleCategory) -> Self {
        match category {
            RuleCategory::Structure => JsonRuleCategory::Structure,
            RuleCategory::Formatting => JsonRuleCategory::Formatting,
            RuleCategory::Content => JsonRuleCategory::Content,
            RuleCategory::Links => JsonRuleCategory::Links,
            RuleCategory::Accessibility => JsonRuleCategory::Accessibility,
            RuleCategory::MdBook => JsonRuleCategory::MdBook,
        }
    }
}

impl From<&RuleStability> for JsonRuleStability {
    fn from(stability: &RuleStability) -> Self {
        match stability {
            RuleStability::Stable => JsonRuleStability::Stable,
            RuleStability::Experimental => JsonRuleStability::Experimental,
            RuleStability::Deprecated => JsonRuleStability::Deprecated,
            RuleStability::Reserved => JsonRuleStability::Reserved,
        }
    }
}

/// Row for the detailed rules table display
#[derive(Tabled)]
struct DetailedRuleTableRow {
    #[tabled(rename = "Rule")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(rename = "Category")]
    category: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Default")]
    default_on: String,
    #[tabled(rename = "Fix")]
    can_fix: String,
}

/// A rule as presented by the `rules` command.
///
/// Document rules and collection rules are stored separately in the registry
/// but are both part of the public rule inventory, so listing resolves either
/// kind through this shape. Collection rules analyze several documents at once
/// and never carry fixes.
struct ListedRule {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    metadata: mdbook_lint_core::rule::RuleMetadata,
    can_fix: bool,
}

/// Every rule ID in the public inventory: document rules plus collection rules.
///
/// `LintEngine::available_rules` covers document rules only, so any listing
/// built on it alone omits collection rules such as the cross-document ADR
/// checks (ADR010 through ADR013).
fn all_listed_rule_ids(engine: &mdbook_lint_core::LintEngine) -> Vec<String> {
    let mut ids: Vec<String> = engine
        .available_rules()
        .into_iter()
        .map(String::from)
        .collect();
    ids.extend(
        engine
            .registry()
            .collection_rule_ids()
            .into_iter()
            .map(String::from),
    );
    ids.sort();
    ids.dedup();
    ids
}

/// Resolve a rule ID against both the document and collection registries.
fn resolve_listed_rule(engine: &mdbook_lint_core::LintEngine, rule_id: &str) -> Option<ListedRule> {
    if let Some(rule) = engine.registry().get_rule(rule_id) {
        return Some(ListedRule {
            id: rule.id(),
            name: rule.name(),
            description: rule.description(),
            metadata: rule.metadata(),
            can_fix: rule.can_fix(),
        });
    }
    engine
        .registry()
        .get_collection_rule(rule_id)
        .map(|rule| ListedRule {
            id: rule.id(),
            name: rule.name(),
            description: rule.description(),
            metadata: rule.metadata(),
            can_fix: false,
        })
}

/// Suffix describing a rule's lifecycle for the simple `rules` listing.
///
/// Only non-default states are annotated, so a plain entry means "stable and on
/// by default".
fn rule_status_suffix(metadata: &mdbook_lint_core::rule::RuleMetadata) -> &'static str {
    use mdbook_lint_core::rule::RuleStability;

    if metadata.deprecated {
        return " [deprecated]";
    }
    match metadata.stability {
        RuleStability::Experimental => " [experimental, off by default]",
        RuleStability::Deprecated => " [deprecated]",
        RuleStability::Reserved => " [reserved]",
        RuleStability::Stable => "",
    }
}

/// Truncate a string to a maximum length, adding "..." if truncated
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Known subcommands that should not trigger smart detection
const KNOWN_SUBCOMMANDS: &[&str] = &[
    "preprocessor",
    "lint",
    "fix",
    "rules",
    "check",
    "init",
    "supports",
    "lsp",
    "rustdoc",
    "help",
    "--help",
    "-h",
    "--version",
    "-V",
];

/// Lint-specific flags that indicate the user wants to lint
const LINT_FLAGS: &[&str] = &[
    "--fix",
    "--fix-unsafe",
    "--dry-run",
    "--no-backup",
    "--config",
    "-c",
    "--standard-only",
    "--mdbook-only",
    "--fail-on-warnings",
    "--markdownlint-compatible",
    "--output",
    "--disable",
    "--enable",
    "--preset",
];

/// Check if an argument looks like a lint target (file path, directory, or glob pattern)
fn looks_like_lint_target(arg: &str) -> bool {
    // Skip if it starts with a dash (it's a flag)
    if arg.starts_with('-') {
        return false;
    }

    // Check for markdown file extensions
    if arg.ends_with(".md") || arg.ends_with(".markdown") {
        return true;
    }

    // Check if it's a path (contains path separators or is a relative/absolute path)
    if arg.contains('/') || arg.contains('\\') || arg == "." || arg == ".." {
        return true;
    }

    // Check if it looks like a glob pattern
    if arg.contains('*') || arg.contains('?') {
        return true;
    }

    // Check if it's an existing file or directory
    let path = std::path::Path::new(arg);
    if path.exists() {
        return true;
    }

    false
}

/// Check if arguments suggest the user wants to run the lint command
fn should_infer_lint_subcommand(args: &[String]) -> bool {
    // Skip the program name (first argument)
    let args: Vec<&str> = args.iter().skip(1).map(|s| s.as_str()).collect();

    if args.is_empty() {
        return false; // No args = preprocessor mode
    }

    let first_arg = args[0];

    // If first arg is a known subcommand, don't infer
    if KNOWN_SUBCOMMANDS.contains(&first_arg.to_lowercase().as_str()) {
        return false;
    }

    // Check if any argument is a lint-specific flag
    for arg in &args {
        if LINT_FLAGS.contains(arg) {
            return true;
        }
    }

    // Check if any argument looks like a lint target
    for arg in &args {
        if looks_like_lint_target(arg) {
            return true;
        }
    }

    false
}

/// Modify args to insert "lint" subcommand if needed
fn maybe_insert_lint_subcommand(args: Vec<String>) -> Vec<String> {
    if should_infer_lint_subcommand(&args) {
        let mut new_args = vec![args[0].clone(), "lint".to_string()];
        new_args.extend(args.into_iter().skip(1));
        new_args
    } else {
        args
    }
}

fn main() {
    // Get args and potentially insert "lint" subcommand
    let args: Vec<String> = std::env::args().collect();
    let args = maybe_insert_lint_subcommand(args);

    let cli = Cli::parse_from(args);

    let result = match cli.command {
        Some(Commands::Preprocessor) => run_preprocessor_mode(),
        Some(Commands::Lint {
            files,
            config,
            standard_only,
            mdbook_only,
            fail_on_warnings,
            markdownlint_compatible,
            output,
            fix,
            fix_unsafe,
            dry_run,
            no_backup,
            disable,
            enable,
            preset,
            color,
            output_file,
        }) => {
            // Set up color choice before running
            match color {
                ColorChoice::Always => anstream::ColorChoice::Always.write_global(),
                ColorChoice::Never => anstream::ColorChoice::Never.write_global(),
                ColorChoice::Auto => anstream::ColorChoice::Auto.write_global(),
            }
            run_cli_mode(
                &files,
                config.as_deref(),
                standard_only,
                mdbook_only,
                fail_on_warnings,
                markdownlint_compatible,
                output,
                fix,
                fix_unsafe,
                dry_run,
                !no_backup,
                disable.as_ref(),
                enable.as_ref(),
                preset,
                cli.verbose,
                cli.quiet,
                output_file.as_deref(),
            )
        }
        Some(Commands::Fix {
            files,
            config,
            standard_only,
            mdbook_only,
            fix_unsafe,
            dry_run,
            no_backup,
            disable,
            enable,
            preset,
            color,
        }) => {
            // Set up color choice before running
            match color {
                ColorChoice::Always => anstream::ColorChoice::Always.write_global(),
                ColorChoice::Never => anstream::ColorChoice::Never.write_global(),
                ColorChoice::Auto => anstream::ColorChoice::Auto.write_global(),
            }
            // Fix subcommand is equivalent to lint --fix
            run_cli_mode(
                &files,
                config.as_deref(),
                standard_only,
                mdbook_only,
                false,                 // fail_on_warnings
                false,                 // markdownlint_compatible
                OutputFormat::Default, // output format
                true,                  // fix is always true for this subcommand
                fix_unsafe,
                dry_run,
                !no_backup,
                disable.as_ref(),
                enable.as_ref(),
                preset,
                cli.verbose,
                cli.quiet,
                None, // fix subcommand has no report file
            )
        }
        Some(Commands::Rules {
            detailed,
            category,
            provider,
            standard_only,
            mdbook_only,
            preset,
            format,
            json,
        }) => {
            // --json flag overrides --format
            let effective_format = if json { RulesFormat::Json } else { format };
            run_rules_command(
                detailed,
                category.as_deref(),
                provider.as_deref(),
                standard_only,
                mdbook_only,
                preset,
                effective_format,
            )
        }
        Some(Commands::Check { config }) => run_check_command(&config),
        Some(Commands::Init {
            format,
            output,
            include_all,
        }) => run_init_command(format, output, include_all),
        Some(Commands::Supports { renderer }) => run_supports_check(&renderer),
        #[cfg(feature = "lsp")]
        Some(Commands::Lsp { stdio, port }) => run_lsp_server(stdio, port),
        Some(Commands::Rustdoc {
            paths,
            config,
            fail_on_warnings,
            output,
            disable,
            enable,
            preset,
            color,
        }) => {
            match color {
                ColorChoice::Always => anstream::ColorChoice::Always.write_global(),
                ColorChoice::Never => anstream::ColorChoice::Never.write_global(),
                ColorChoice::Auto => anstream::ColorChoice::Auto.write_global(),
            }
            run_rustdoc_mode(
                &paths,
                config.as_deref(),
                fail_on_warnings,
                output,
                disable.as_ref(),
                enable.as_ref(),
                preset,
                cli.verbose,
                cli.quiet,
            )
        }
        None => {
            // No subcommand provided - default to preprocessor mode
            // This matches mdBook's expectation for preprocessors
            run_preprocessor_mode()
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

/// Recursively collect all markdown files from a directory
fn collect_markdown_files(dir: &PathBuf, files: &mut Vec<PathBuf>) -> Result<()> {
    let entries = std::fs::read_dir(dir).map_err(|e| {
        mdbook_lint::error::MdBookLintError::document_error(format!(
            "Failed to read directory {}: {e}",
            dir.display()
        ))
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            mdbook_lint::error::MdBookLintError::document_error(format!(
                "Failed to read directory entry: {e}"
            ))
        })?;

        let path = entry.path();

        if path.is_dir() {
            // Skip hidden directories like .git
            if let Some(name) = path.file_name()
                && name.to_string_lossy().starts_with('.')
            {
                continue;
            }
            collect_markdown_files(&path, files)?;
        } else if let Some(ext) = path.extension()
            && matches!(ext.to_str(), Some("md") | Some("markdown"))
        {
            files.push(path);
        }
    }

    Ok(())
}

/// Check if a file is tracked by git
fn is_git_tracked(path: &PathBuf) -> Result<bool> {
    use std::process::Command;

    let output = Command::new("git")
        .args(["ls-files", "--error-unmatch"])
        .arg(path)
        .output();

    match output {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Ok(false), // Git not available or not in a git repo
    }
}

/// Create a backup file with .bak extension
fn create_backup_file(path: &PathBuf) -> Result<()> {
    let backup_path = path.with_extension(format!(
        "{}.bak",
        path.extension().and_then(|ext| ext.to_str()).unwrap_or("")
    ));

    std::fs::copy(path, &backup_path).map_err(|e| {
        mdbook_lint::error::MdBookLintError::document_error(format!(
            "Failed to create backup file {}: {e}",
            backup_path.display()
        ))
    })?;

    println!("Created backup: {}", backup_path.display());
    Ok(())
}

/// Emit a SARIF report to `output_file`, or to stdout when no path is given.
///
/// Writing to a file keeps stdout clean so normal diagnostics and the process
/// exit status are unaffected, which is what the CI documentation describes.
fn emit_sarif(
    violations_by_file: &[(String, Vec<mdbook_lint_core::violation::Violation>)],
    engine: &mdbook_lint_core::LintEngine,
    output_file: Option<&Path>,
) -> Result<()> {
    let report = sarif::build_report(violations_by_file, engine);
    let rendered = serde_json::to_string_pretty(&report).map_err(|e| {
        mdbook_lint::error::MdBookLintError::config_error(format!(
            "Failed to serialize SARIF report: {e}"
        ))
    })?;

    match output_file {
        Some(path) => {
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
            {
                std::fs::create_dir_all(parent).map_err(|e| {
                    mdbook_lint::error::MdBookLintError::config_error(format!(
                        "Failed to create directory for {}: {e}",
                        path.display()
                    ))
                })?;
            }
            std::fs::write(path, rendered).map_err(|e| {
                mdbook_lint::error::MdBookLintError::config_error(format!(
                    "Failed to write SARIF report to {}: {e}",
                    path.display()
                ))
            })?;
        }
        None => println!("{rendered}"),
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_cli_mode(
    files: &[String],
    config_path: Option<&str>,
    standard_only: bool,
    mdbook_only: bool,
    fail_on_warnings: bool,
    markdownlint_compatible: bool,
    output_format: OutputFormat,
    fix: bool,
    fix_unsafe: bool,
    dry_run: bool,
    backup: bool,
    disable: Option<&Vec<String>>,
    enable: Option<&Vec<String>>,
    preset: Option<RulePreset>,
    verbose: bool,
    quiet: bool,
    output_file: Option<&Path>,
) -> Result<()> {
    // Validate mutually exclusive flags
    if standard_only && mdbook_only {
        return Err(mdbook_lint::error::MdBookLintError::config_error(
            "Cannot specify both --standard-only and --mdbook-only",
        ));
    }

    // Validate fix flags
    if dry_run && !fix && !fix_unsafe {
        return Err(mdbook_lint::error::MdBookLintError::config_error(
            "--dry-run requires either --fix or --fix-unsafe",
        ));
    }

    // fix_unsafe implies fix
    let apply_fixes = fix || fix_unsafe;

    // Validate disable/enable flags
    if disable.is_some() && enable.is_some() {
        return Err(mdbook_lint::error::MdBookLintError::config_error(
            "Cannot specify both --disable and --enable flags",
        ));
    }

    if (disable.is_some() || enable.is_some()) && (standard_only || mdbook_only) {
        return Err(mdbook_lint::error::MdBookLintError::config_error(
            "--disable and --enable flags cannot be used with --standard-only or --mdbook-only",
        ));
    }

    // Load configuration - try discovery if no explicit path
    let (mut config, config_source) = if let Some(path) = config_path {
        // Explicit config path provided
        let config_content = std::fs::read_to_string(path).map_err(|e| {
            mdbook_lint::error::MdBookLintError::config_error(format!(
                "Failed to read config file {path}: {e}"
            ))
        })?;

        // Detect format from extension and content
        let cfg = if path.ends_with(".toml") {
            Config::from_toml_str(&config_content)?
        } else if path.ends_with(".yaml") || path.ends_with(".yml") {
            Config::from_yaml_str(&config_content)?
        } else if path.ends_with(".json") {
            Config::from_json_str(&config_content)?
        } else {
            // Try to auto-detect format
            config_content.parse()?
        };
        (cfg, Some(path.to_string()))
    } else if let Some(discovered_path) = Config::discover_config(None) {
        // Try to discover config file
        let path_str = discovered_path.display().to_string();
        let cfg = Config::from_file(&discovered_path)?;
        (cfg, Some(path_str))
    } else {
        // No config found, use defaults
        (Config::default(), None)
    };

    // Print config path in verbose mode
    if verbose && let Some(ref path) = config_source {
        output::print_status("Config", path);
    }

    // Override config with CLI flags
    if fail_on_warnings {
        config.fail_on_warnings = true;
    }
    if markdownlint_compatible {
        config.core.markdownlint_compatible = true;
    }

    // A CLI preset overrides configured base selectors while retaining
    // configured disabled-rules as subtractive customizations.
    if let Some(preset) = preset {
        config.preset = Some(preset);
        config.core.enabled_rules.clear();
    }

    // Apply disable/enable flags
    if let Some(disabled_rules) = disable {
        // Add to existing disabled rules
        config
            .core
            .disabled_rules
            .extend(disabled_rules.iter().cloned());
    }

    if let Some(enabled_rules) = enable {
        // --enable is an explicit selection: run exactly these rules.
        //
        // This previously inverted the selection, disabling every other rule
        // and leaving enabled_rules empty. The outcome was the same for stable
        // rules, but the registry could not tell that a rule had been chosen
        // deliberately, so `--enable <experimental rule>` selected a rule that
        // then failed the default-activation check. Recording the selection
        // directly keeps that information; enabled_rules already means
        // "only these" in RuleRegistry::should_run_rule.
        config.core.disabled_rules.clear();
        config.core.enabled_rules = enabled_rules.clone();
    }

    let effective_core = config.effective_core_config();

    // Create appropriate engine based on flags
    let mut registry = PluginRegistry::new();

    if standard_only {
        registry.register_provider(Box::new(StandardRuleProvider))?;
        #[cfg(feature = "content")]
        registry.register_provider(Box::new(ContentRuleProvider))?;
        #[cfg(feature = "adr")]
        registry.register_provider(Box::new(AdrRuleProvider))?;
    } else if mdbook_only {
        registry.register_provider(Box::new(MdBookRuleProvider))?;
        #[cfg(feature = "content")]
        registry.register_provider(Box::new(ContentRuleProvider))?;
        #[cfg(feature = "adr")]
        registry.register_provider(Box::new(AdrRuleProvider))?;
    } else {
        // Default: use all rules (standard + mdBook + content if enabled)
        registry.register_provider(Box::new(StandardRuleProvider))?;
        registry.register_provider(Box::new(MdBookRuleProvider))?;
        #[cfg(feature = "content")]
        registry.register_provider(Box::new(ContentRuleProvider))?;
        #[cfg(feature = "adr")]
        registry.register_provider(Box::new(AdrRuleProvider))?;
    }

    let engine = registry.create_engine_with_config(Some(&effective_core))?;

    let mut total_violations = 0;
    let mut has_errors = false;
    let mut violations_by_file = Vec::new();

    // Check if stdin is requested (file argument is "-")
    let has_stdin = files.iter().any(|f| f == "-");

    // Validate stdin usage
    if has_stdin {
        if files.len() > 1 {
            return Err(mdbook_lint::error::MdBookLintError::config_error(
                "Cannot mix stdin (-) with other file arguments",
            ));
        }
        if apply_fixes && !dry_run {
            return Err(mdbook_lint::error::MdBookLintError::config_error(
                "Cannot use --fix with stdin input. Use --fix --dry-run to preview fixes.",
            ));
        }
    }

    // Process stdin if requested
    if has_stdin {
        let mut content = String::new();
        io::stdin().read_to_string(&mut content).map_err(|e| {
            mdbook_lint::error::MdBookLintError::document_error(format!(
                "Failed to read from stdin: {e}"
            ))
        })?;

        // Create document with synthetic path
        let stdin_path = PathBuf::from("<stdin>");
        let document = Document::new(content, stdin_path.clone())?;

        // Lint with configuration
        let violations = engine.lint_document_with_config(&document, &effective_core)?;

        if !violations.is_empty() {
            violations_by_file.push(("<stdin>".to_string(), violations.clone()));
            total_violations += violations.len();

            for violation in &violations {
                if violation.severity == Severity::Error {
                    has_errors = true;
                }
            }
        }
    } else {
        // Process files
        // Collect all markdown files from the provided paths
        let mut markdown_files = Vec::new();
        for file_path in files {
            let path = PathBuf::from(file_path);

            if path.is_dir() {
                // Recursively find all markdown files in directory
                collect_markdown_files(&path, &mut markdown_files)?;
            } else {
                // Skip non-markdown files
                if let Some(ext) = path.extension()
                    && !matches!(ext.to_str(), Some("md") | Some("markdown"))
                {
                    continue;
                }
                markdown_files.push(path);
            }
        }

        // Drop any files matching the configured ignore-paths patterns
        filter_ignored_paths(&mut markdown_files, &effective_core.ignore_paths);

        // Process markdown files in parallel
        let violations_mutex = Mutex::new(Vec::new());
        let total_count = AtomicUsize::new(0);
        let errors_found = AtomicBool::new(false);

        markdown_files.par_iter().for_each(|path| {
            let file_path = path.to_string_lossy().to_string();

            // Read file content
            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to read file {}: {e}", path.display());
                    return;
                }
            };

            // Create document
            let document = match Document::new(content, path.clone()) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Failed to parse document {}: {e}", path.display());
                    return;
                }
            };

            // Lint with configuration
            let violations = match engine.lint_document_with_config(&document, &effective_core) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Failed to lint {}: {e}", path.display());
                    return;
                }
            };

            if !violations.is_empty() {
                let violation_count = violations.len();
                let has_error = violations.iter().any(|v| v.severity == Severity::Error);

                // Update atomics
                total_count.fetch_add(violation_count, Ordering::Relaxed);
                if has_error {
                    errors_found.store(true, Ordering::Relaxed);
                }

                // Add to results
                if let Ok(mut guard) = violations_mutex.lock() {
                    guard.push((file_path, violations));
                }
            }
        });

        // Collect results and sort by file path for deterministic output
        violations_by_file = violations_mutex.into_inner().unwrap_or_default();
        violations_by_file.sort_by(|a, b| a.0.cmp(&b.0));
        total_violations = total_count.load(Ordering::Relaxed);
        has_errors = errors_found.load(Ordering::Relaxed);
    }

    // Apply fixes if requested
    let mut fixes_applied = 0;
    let mut files_modified = 0;

    if apply_fixes {
        for (file_path, violations) in &violations_by_file {
            let fixable_violations: Vec<_> = violations
                .iter()
                .filter(|v| v.fix.is_some() && config.should_auto_fix_rule(&v.rule_id))
                .cloned()
                .collect();

            if !fixable_violations.is_empty() {
                let path = PathBuf::from(file_path);

                // Read original content
                let original_content = std::fs::read_to_string(&path).map_err(|e| {
                    mdbook_lint::error::MdBookLintError::document_error(format!(
                        "Failed to read file {}: {e}",
                        path.display()
                    ))
                })?;

                let (fixed_content, unfixed) =
                    engine.apply_fixes(&original_content, &fixable_violations);
                let applied_count = fixable_violations.len() - unfixed.len();

                if !unfixed.is_empty() {
                    eprintln!(
                        "Skipped {} conflicting or invalid fix(es) in {}",
                        unfixed.len(),
                        file_path
                    );
                }

                if applied_count > 0 && fixed_content != original_content {
                    if dry_run {
                        println!("Would fix {} issue(s) in {}", applied_count, file_path);
                        // TODO: Show diff preview
                    } else {
                        // Create backup if requested and not using git
                        if backup && !is_git_tracked(&path)? {
                            create_backup_file(&path)?;
                        }

                        // Write fixed content
                        std::fs::write(&path, fixed_content).map_err(|e| {
                            mdbook_lint::error::MdBookLintError::document_error(format!(
                                "Failed to write fixed file {}: {e}",
                                path.display()
                            ))
                        })?;

                        println!("Fixed {} issue(s) in {}", applied_count, file_path);
                        fixes_applied += applied_count;
                        files_modified += 1;
                    }
                }
            }
        }

        if !dry_run && fixes_applied > 0 {
            println!(
                "Applied {} fix(es) across {} file(s)",
                fixes_applied, files_modified
            );
        }
    }

    // Re-lint files after fixes to get accurate violations for display and exit code
    if apply_fixes && !dry_run && fixes_applied > 0 {
        violations_by_file.clear();
        total_violations = 0;
        has_errors = false;

        // Process each file again to get post-fix violations
        for file_path in files {
            let path = PathBuf::from(file_path);

            // Handle directories by re-collecting markdown files
            let mut current_markdown_files = Vec::new();
            if path.is_dir() {
                collect_markdown_files(&path, &mut current_markdown_files)?;
            } else if let Some(ext) = path.extension()
                && matches!(ext.to_str(), Some("md") | Some("markdown"))
            {
                current_markdown_files.push(path);
            }

            filter_ignored_paths(&mut current_markdown_files, &effective_core.ignore_paths);

            for md_path in current_markdown_files {
                let file_path = md_path.to_string_lossy().to_string();

                // Read file content (now potentially fixed)
                let content = std::fs::read_to_string(&md_path).map_err(|e| {
                    mdbook_lint::error::MdBookLintError::document_error(format!(
                        "Failed to read file {}: {e}",
                        md_path.display()
                    ))
                })?;

                // Create document and lint
                let document = Document::new(content, md_path.clone())?;
                let violations = engine.lint_document_with_config(&document, &effective_core)?;

                if !violations.is_empty() {
                    violations_by_file.push((file_path, violations.clone()));
                    total_violations += violations.len();

                    for violation in &violations {
                        if violation.severity == Severity::Error {
                            has_errors = true;
                        }
                    }
                }
            }
        }
    }

    // Count errors and warnings for summary
    let error_count = violations_by_file
        .iter()
        .flat_map(|(_, v)| v)
        .filter(|v| v.severity == Severity::Error)
        .count();
    let warning_count = violations_by_file
        .iter()
        .flat_map(|(_, v)| v)
        .filter(|v| v.severity == Severity::Warning)
        .count();

    // Output results
    match output_format {
        OutputFormat::Default => {
            output::print_cargo_style(&violations_by_file);
            output::print_summary(total_violations, error_count, warning_count, quiet);
        }
        OutputFormat::Json => {
            let output = serde_json::json!({
                "total_violations": total_violations,
                "has_errors": has_errors,
                "files": violations_by_file.iter().map(|(file, violations)| {
                    serde_json::json!({
                        "file": file,
                        "violations": violations
                    })
                }).collect::<Vec<_>>()
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        OutputFormat::Github => {
            for (file_path, violations) in &violations_by_file {
                for violation in violations {
                    let level = match violation.severity {
                        Severity::Error => "error",
                        Severity::Warning => "warning",
                        Severity::Info => "notice",
                    };
                    println!(
                        "::{level} file={file_path},line={}::{}: {}",
                        violation.line, violation.rule_id, violation.message
                    );
                }
            }
        }
        OutputFormat::Sarif => {
            emit_sarif(&violations_by_file, &engine, output_file)?;
        }
    }

    // Determine exit code
    // For fix mode, we already re-linted and updated has_errors/total_violations
    // For non-fix mode, use original values
    if has_errors || (total_violations > 0 && config.fail_on_warnings) {
        process::exit(1);
    }

    Ok(())
}

fn run_rules_command(
    detailed: bool,
    category_filter: Option<&str>,
    provider_filter: Option<&str>,
    standard_only: bool,
    mdbook_only: bool,
    preset: Option<RulePreset>,
    format: RulesFormat,
) -> Result<()> {
    // Validate mutually exclusive flags
    if standard_only && mdbook_only {
        return Err(mdbook_lint::error::MdBookLintError::config_error(
            "Cannot specify both --standard-only and --mdbook-only",
        ));
    }

    // Create appropriate engine based on flags
    let mut registry = PluginRegistry::new();

    if standard_only {
        registry.register_provider(Box::new(StandardRuleProvider))?;
        #[cfg(feature = "content")]
        registry.register_provider(Box::new(ContentRuleProvider))?;
        #[cfg(feature = "adr")]
        registry.register_provider(Box::new(AdrRuleProvider))?;
    } else if mdbook_only {
        registry.register_provider(Box::new(MdBookRuleProvider))?;
        #[cfg(feature = "content")]
        registry.register_provider(Box::new(ContentRuleProvider))?;
        #[cfg(feature = "adr")]
        registry.register_provider(Box::new(AdrRuleProvider))?;
    } else {
        // Default: show all rules (standard + mdBook + content if enabled)
        registry.register_provider(Box::new(StandardRuleProvider))?;
        registry.register_provider(Box::new(MdBookRuleProvider))?;
        #[cfg(feature = "content")]
        registry.register_provider(Box::new(ContentRuleProvider))?;
        #[cfg(feature = "adr")]
        registry.register_provider(Box::new(AdrRuleProvider))?;
    }

    let engine = registry.create_engine()?;
    let providers = registry.providers();
    let listed_rule_ids: Vec<String> = all_listed_rule_ids(&engine)
        .into_iter()
        .filter(|rule_id| preset.is_none_or(|selected| selected.contains(rule_id)))
        .filter(|rule_id| {
            provider_filter.is_none_or(|filter| {
                providers.iter().any(|provider| {
                    provider.provider_id() == filter
                        && provider.rule_ids().contains(&rule_id.as_str())
                })
            })
        })
        .filter(|rule_id| {
            let Some(filter) = category_filter else {
                return true;
            };
            resolve_listed_rule(&engine, rule_id).is_some_and(|rule| {
                format!("{:?}", rule.metadata.category).eq_ignore_ascii_case(filter)
            })
        })
        .collect();

    match format {
        RulesFormat::Json => {
            // JSON output mode
            let mut json_providers = Vec::new();
            let mut total_rules = 0;

            for provider in providers {
                // Apply provider filter
                if let Some(filter) = provider_filter
                    && provider.provider_id() != filter
                {
                    continue;
                }

                let mut json_rules = Vec::new();

                for rule_id in provider.rule_ids() {
                    if !listed_rule_ids.iter().any(|listed| listed == rule_id) {
                        continue;
                    }
                    if let Some(rule) = resolve_listed_rule(&engine, rule_id) {
                        let metadata = &rule.metadata;

                        let json_rule = JsonRule {
                            id: rule.id.to_string(),
                            name: rule.name.to_string(),
                            description: rule.description.to_string(),
                            category: JsonRuleCategory::from(&metadata.category),
                            stability: JsonRuleStability::from(&metadata.stability),
                            deprecated: metadata.deprecated,
                            deprecated_reason: metadata.deprecated_reason.map(String::from),
                            replacement: metadata.replacement.map(String::from),
                            introduced_in: metadata.introduced_in.map(String::from),
                            can_fix: rule.can_fix,
                        };

                        json_rules.push(json_rule);
                        total_rules += 1;
                    }
                }

                if !json_rules.is_empty() || provider_filter.is_some() {
                    let json_provider = JsonRuleProvider {
                        provider_id: provider.provider_id().to_string(),
                        version: provider.version().to_string(),
                        description: provider.description().to_string(),
                        rules: json_rules,
                    };

                    json_providers.push(json_provider);
                }
            }

            let json_output = JsonRulesOutput {
                preset: preset.map(|selected| JsonPreset {
                    name: selected.name().to_string(),
                    description: selected.description().to_string(),
                    rule_ids: selected
                        .rule_ids()
                        .iter()
                        .map(|rule_id| (*rule_id).to_string())
                        .collect(),
                }),
                total_rules,
                providers: json_providers,
            };

            println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
        }
        RulesFormat::Default => {
            // Collect rules into table rows
            let mut rows: Vec<_> = Vec::new();
            let mut total_rules = 0;

            for rule_id in &listed_rule_ids {
                if let Some(rule) = resolve_listed_rule(&engine, rule_id) {
                    let metadata = &rule.metadata;

                    total_rules += 1;

                    if detailed {
                        // Detailed table with category and status
                        let status = if metadata.deprecated {
                            "Deprecated".to_string()
                        } else {
                            format!("{:?}", metadata.stability)
                        };

                        rows.push(DetailedRuleTableRow {
                            id: rule.id.to_string(),
                            name: rule.name.to_string(),
                            description: truncate_string(rule.description, 50),
                            category: format!("{:?}", metadata.category),
                            status,
                            default_on: if metadata.runs_by_default() {
                                "Yes"
                            } else {
                                "-"
                            }
                            .to_string(),
                            can_fix: if rule.can_fix { "Yes" } else { "-" }.to_string(),
                        });
                    }
                }
            }

            if detailed {
                // Print detailed table
                if let Some(selected) = preset {
                    println!(
                        "mdbook-lint Rules in preset '{}'\n{}\n",
                        selected.name(),
                        selected.description()
                    );
                } else {
                    println!("mdbook-lint Rules\n");
                }

                let table = Table::new(&rows).with(Style::rounded()).to_string();
                println!("{table}");

                println!("\nTotal: {total_rules} rules available");
            } else {
                // Simple list mode with rule name
                if let Some(selected) = preset {
                    println!("Rules in preset '{}':", selected.name());
                    println!("{}", selected.description());
                } else {
                    println!("Available rules:");
                }
                for rule_id in &listed_rule_ids {
                    if let Some(rule) = resolve_listed_rule(&engine, rule_id) {
                        let metadata = &rule.metadata;

                        println!(
                            "  {:<11} {:<32} {}{}",
                            rule.id,
                            rule.name,
                            truncate_string(rule.description, 50),
                            rule_status_suffix(metadata)
                        );
                    }
                }

                if preset.is_none() {
                    println!(
                        "\nRules marked [experimental] do not run by default. Enable them with\n\
                         --enable <RULE>, or with experimental-rules = [\"<RULE>\"] (or [\"*\"])\n\
                         in configuration to add them alongside the stable defaults."
                    );
                }
                println!("\nUse --detailed for a formatted table with more information.");
            }
        }
    }

    Ok(())
}

fn run_check_command(config_path: &PathBuf) -> Result<()> {
    let config_content = std::fs::read_to_string(config_path).map_err(|e| {
        mdbook_lint::error::MdBookLintError::config_error(format!(
            "Failed to read config file {}: {}",
            config_path.display(),
            e
        ))
    })?;

    // Try to parse the configuration
    let config = if config_path.extension().and_then(|s| s.to_str()) == Some("toml") {
        Config::from_toml_str(&config_content)?
    } else if matches!(
        config_path.extension().and_then(|s| s.to_str()),
        Some("yaml") | Some("yml")
    ) {
        Config::from_yaml_str(&config_content)?
    } else if config_path.extension().and_then(|s| s.to_str()) == Some("json") {
        Config::from_json_str(&config_content)?
    } else {
        config_content.parse()?
    };

    // Build the rule registry to validate rule names
    let mut registry = PluginRegistry::new();
    registry.register_provider(Box::new(StandardRuleProvider))?;
    registry.register_provider(Box::new(MdBookRuleProvider))?;
    #[cfg(feature = "content")]
    registry.register_provider(Box::new(ContentRuleProvider))?;
    #[cfg(feature = "adr")]
    registry.register_provider(Box::new(AdrRuleProvider))?;
    let engine = registry.create_engine()?;

    let available_rules: std::collections::HashSet<String> = engine
        .available_rules()
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    // Valid categories
    let valid_categories: std::collections::HashSet<&str> = [
        "structure",
        "style",
        "whitespace",
        "code",
        "links",
        "mdbook",
        "accessibility",
    ]
    .into_iter()
    .collect();

    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    if let Some(preset) = config.preset {
        if !config.core.enabled_rules.is_empty() {
            warnings.push(format!(
                "Preset '{preset}' is ignored because enabled-rules is an explicit rule selection"
            ));
        } else {
            if !config.core.experimental_rules.is_empty() {
                warnings.push(format!(
                    "experimental-rules is ignored when preset '{preset}' is selected"
                ));
            }
            if !config.core.enabled_categories.is_empty()
                || !config.core.disabled_categories.is_empty()
            {
                warnings.push(format!(
                    "rule categories are ignored when preset '{preset}' is selected"
                ));
            }
            if config.core.markdownlint_compatible {
                warnings.push(format!(
                    "markdownlint-compatible is ignored when preset '{preset}' is selected"
                ));
            }
        }
    }

    // Validate enabled-rules
    for rule_id in &config.core.enabled_rules {
        if !available_rules.contains(rule_id) {
            errors.push(format!("Unknown rule in enabled-rules: '{rule_id}'"));
            // Suggest similar rules
            if let Some(suggestion) = find_similar_rule(rule_id, &available_rules) {
                errors.push(format!("  Did you mean '{suggestion}'?"));
            }
        }
    }

    // Validate disabled-rules
    for rule_id in &config.core.disabled_rules {
        if !available_rules.contains(rule_id) {
            errors.push(format!("Unknown rule in disabled-rules: '{rule_id}'"));
            if let Some(suggestion) = find_similar_rule(rule_id, &available_rules) {
                errors.push(format!("  Did you mean '{suggestion}'?"));
            }
        }
    }

    // Validate experimental-rules. "*" selects every experimental rule.
    for selector in &config.core.experimental_rules {
        if selector == "*" {
            continue;
        }
        if !available_rules.contains(selector) {
            errors.push(format!("Unknown rule in experimental-rules: '{selector}'"));
            if let Some(suggestion) = find_similar_rule(selector, &available_rules) {
                errors.push(format!("  Did you mean '{suggestion}'?"));
            }
        } else if !rule_is_experimental(selector) {
            warnings.push(format!(
                "Rule '{selector}' in experimental-rules is not experimental; it already runs by default"
            ));
        }
    }

    // Validate enabled-categories
    for category in &config.core.enabled_categories {
        if !valid_categories.contains(category.as_str()) {
            errors.push(format!(
                "Unknown category in enabled-categories: '{category}'"
            ));
            errors.push(format!(
                "  Valid categories: {}",
                valid_categories
                    .iter()
                    .copied()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    // Validate disabled-categories
    for category in &config.core.disabled_categories {
        if !valid_categories.contains(category.as_str()) {
            errors.push(format!(
                "Unknown category in disabled-categories: '{category}'"
            ));
            errors.push(format!(
                "  Valid categories: {}",
                valid_categories
                    .iter()
                    .copied()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    // Validate rule-specific configs reference valid rules
    for rule_id in config.core.rule_configs.keys() {
        if !available_rules.contains(rule_id) {
            warnings.push(format!(
                "Configuration for unknown rule: '{rule_id}' (will be ignored)"
            ));
            if let Some(suggestion) = find_similar_rule(rule_id, &available_rules) {
                warnings.push(format!("  Did you mean '{suggestion}'?"));
            }
        }
    }

    // Print warnings
    for warning in &warnings {
        eprintln!("Warning: {warning}");
    }

    // Print errors
    for error in &errors {
        eprintln!("Error: {error}");
    }

    if !errors.is_empty() {
        return Err(mdbook_lint::error::MdBookLintError::config_error(format!(
            "Configuration file has {} error(s)",
            errors.len()
        )));
    }

    if warnings.is_empty() {
        println!("Configuration file {} is valid", config_path.display());
    } else {
        println!(
            "Configuration file {} is valid (with {} warning(s))",
            config_path.display(),
            warnings.len()
        );
    }

    Ok(())
}

/// Find a similar rule name for typo suggestions
/// Return true if `rule_id` names a registered experimental rule.
fn rule_is_experimental(rule_id: &str) -> bool {
    use mdbook_lint_core::rule::RuleStability;

    let mut registry = PluginRegistry::new();
    if registry
        .register_provider(Box::new(StandardRuleProvider))
        .is_err()
        || registry
            .register_provider(Box::new(MdBookRuleProvider))
            .is_err()
    {
        return false;
    }
    #[cfg(feature = "content")]
    let _ = registry.register_provider(Box::new(ContentRuleProvider));
    #[cfg(feature = "adr")]
    let _ = registry.register_provider(Box::new(AdrRuleProvider));

    let Ok(engine) = registry.create_engine() else {
        return false;
    };
    engine
        .registry()
        .get_rule(rule_id)
        .is_some_and(|rule| rule.metadata().stability == RuleStability::Experimental)
}

fn find_similar_rule(input: &str, available: &std::collections::HashSet<String>) -> Option<String> {
    let input_lower = input.to_lowercase();

    // First pass: check for case-insensitive exact match
    for rule in available {
        if input_lower == rule.to_lowercase() {
            return Some(rule.clone());
        }
    }

    // Second pass: find closest match by Levenshtein distance
    let mut best_match: Option<(String, usize)> = None;

    for rule in available {
        let rule_lower = rule.to_lowercase();
        let distance = levenshtein_distance(&input_lower, &rule_lower);

        // Only consider matches with distance <= 2
        if distance <= 2 && (best_match.is_none() || distance < best_match.as_ref().unwrap().1) {
            best_match = Some((rule.clone(), distance));
        }
    }

    best_match.map(|(rule, _)| rule)
}

/// Simple Levenshtein distance implementation for typo detection
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(a_len + 1) {
        row[0] = i;
    }
    for (j, val) in matrix[0].iter_mut().enumerate().take(b_len + 1) {
        *val = j;
    }

    for (i, a_char) in a_chars.iter().enumerate() {
        for (j, b_char) in b_chars.iter().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }

    matrix[a_len][b_len]
}

/// Comprehensive example configuration with all rules documented.
/// This is embedded from example-mdbook-lint.toml at compile time.
const EXAMPLE_CONFIG_TOML: &str = include_str!("../example-mdbook-lint.toml");

/// Count the distinct rule IDs documented in a config file's comments.
///
/// Rules are documented one per comment block, as `# MD001 - description`.
/// Counting them here keeps the `init --include-all` message in step with the
/// file instead of drifting from a hardcoded literal every time a rule is added.
fn documented_rule_count(config: &str) -> usize {
    let mut ids: Vec<&str> = config
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("# ")?;
            let id = rest.split_whitespace().next()?;
            let (prefix, digits) = id.split_at(id.find(|c: char| c.is_ascii_digit())?);
            let known = matches!(prefix, "MD" | "MDBOOK" | "CONTENT" | "ADR");
            (known && !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
                .then_some(id)
        })
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids.len()
}

fn run_init_command(
    format: ConfigFormat,
    output_path: Option<PathBuf>,
    include_all: bool,
) -> Result<()> {
    let (content, extension) = if include_all {
        // Use the comprehensive example config with all rules documented
        match format {
            ConfigFormat::Toml => (EXAMPLE_CONFIG_TOML.to_string(), "toml"),
            ConfigFormat::Yaml => {
                // For YAML/JSON, we still generate programmatically since the
                // example config is TOML-specific with its comments
                let config = Config::default();
                (config.to_yaml_string()?, "yaml")
            }
            ConfigFormat::Json => {
                let config = Config::default();
                (config.to_json_string()?, "json")
            }
        }
    } else {
        // Generate minimal config programmatically
        let config = Config::default();
        match format {
            ConfigFormat::Toml => (config.to_toml_string()?, "toml"),
            ConfigFormat::Yaml => (config.to_yaml_string()?, "yaml"),
            ConfigFormat::Json => (config.to_json_string()?, "json"),
        }
    };

    let output_file =
        output_path.unwrap_or_else(|| PathBuf::from(format!(".mdbook-lint.{extension}")));

    std::fs::write(&output_file, &content).map_err(|e| {
        mdbook_lint::error::MdBookLintError::config_error(format!(
            "Failed to write config file {}: {}",
            output_file.display(),
            e
        ))
    })?;

    println!("Configuration file created: {}", output_file.display());
    if include_all {
        println!(
            "Includes documentation for all {} available rules",
            documented_rule_count(EXAMPLE_CONFIG_TOML)
        );
        println!("Uncomment and modify settings as needed for your project");
    } else {
        println!("Run with --include-all for a comprehensive example with all rules documented");
    }

    Ok(())
}

fn run_supports_check(renderer: &str) -> Result<()> {
    // mdBook preprocessors should support all renderers by default
    // unless they have specific renderer requirements
    match renderer {
        "html" | "markdown" | "epub" | "pdf" => {
            process::exit(0); // Success - we support this renderer
        }
        _ => {
            process::exit(0); // We support all renderers by default
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_rustdoc_mode(
    paths: &[String],
    config_path: Option<&str>,
    fail_on_warnings: bool,
    output_format: OutputFormat,
    disable: Option<&Vec<String>>,
    enable: Option<&Vec<String>>,
    preset: Option<RulePreset>,
    verbose: bool,
    quiet: bool,
) -> Result<()> {
    use rustdoc::{extract_module_docs, find_rust_files, map_line_to_source};

    // Validate disable/enable flags
    if disable.is_some() && enable.is_some() {
        return Err(mdbook_lint::error::MdBookLintError::config_error(
            "Cannot specify both --disable and --enable flags",
        ));
    }

    // Load configuration
    let (mut config, config_source) = if let Some(path) = config_path {
        let config_content = std::fs::read_to_string(path).map_err(|e| {
            mdbook_lint::error::MdBookLintError::config_error(format!(
                "Failed to read config file {path}: {e}"
            ))
        })?;

        let cfg = if path.ends_with(".toml") {
            Config::from_toml_str(&config_content)?
        } else if path.ends_with(".yaml") || path.ends_with(".yml") {
            Config::from_yaml_str(&config_content)?
        } else if path.ends_with(".json") {
            Config::from_json_str(&config_content)?
        } else {
            config_content.parse()?
        };
        (cfg, Some(path.to_string()))
    } else if let Some(discovered_path) = Config::discover_config(None) {
        let path_str = discovered_path.display().to_string();
        let cfg = Config::from_file(&discovered_path)?;
        (cfg, Some(path_str))
    } else {
        (Config::default(), None)
    };

    if verbose && let Some(ref path) = config_source {
        output::print_status("Config", path);
    }

    if fail_on_warnings {
        config.fail_on_warnings = true;
    }

    if let Some(preset) = preset {
        config.preset = Some(preset);
        config.core.enabled_rules.clear();
    }

    // Disable rules that don't make sense for rustdoc by default:
    // - MD041: First line heading - rustdoc often starts with description
    // - MD047: Trailing newline - doc comments don't have trailing newlines
    // - MD025: Single H1 - rustdoc idiomatically uses multiple # sections (Examples, Panics, Safety)
    config.core.disabled_rules.extend([
        "MD041".to_string(),
        "MD047".to_string(),
        "MD025".to_string(),
    ]);

    // Apply disable/enable flags
    if let Some(disabled_rules) = disable {
        config
            .core
            .disabled_rules
            .extend(disabled_rules.iter().cloned());
    }

    if let Some(enabled_rules) = enable {
        config.core.disabled_rules.clear();
        config.core.enabled_rules = enabled_rules.clone();
    }

    let effective_core = config.effective_core_config();

    // Create engine with standard rules only (mdBook rules don't apply to rustdoc)
    let mut registry = PluginRegistry::new();
    registry.register_provider(Box::new(StandardRuleProvider))?;
    #[cfg(feature = "content")]
    registry.register_provider(Box::new(ContentRuleProvider))?;
    #[cfg(feature = "adr")]
    registry.register_provider(Box::new(AdrRuleProvider))?;

    let engine = registry.create_engine_with_config(Some(&effective_core))?;

    // Collect Rust files
    let mut rust_files = Vec::new();
    for path_str in paths {
        let path = PathBuf::from(path_str);
        match find_rust_files(&path) {
            Ok(files) => rust_files.extend(files),
            Err(e) => {
                eprintln!("Warning: Failed to read {}: {}", path_str, e);
            }
        }
    }

    if rust_files.is_empty() {
        if !quiet {
            println!("No Rust files found");
        }
        return Ok(());
    }

    if verbose {
        output::print_status("Checking", &format!("{} Rust file(s)", rust_files.len()));
    }

    let mut total_violations = 0;
    let mut has_errors = false;
    let mut violations_by_file: Vec<(String, Vec<mdbook_lint_core::violation::Violation>)> =
        Vec::new();

    for rust_path in &rust_files {
        let content = match std::fs::read_to_string(rust_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to read {}: {}", rust_path.display(), e);
                continue;
            }
        };

        let source_path = rust_path.to_string_lossy().to_string();

        if let Some(extracted) = extract_module_docs(&content) {
            // Create a document from the extracted markdown
            // Use a synthetic path that indicates this is from rustdoc
            let doc_path = PathBuf::from(format!("{}#rustdoc", source_path));
            let document = match Document::new(extracted.content.clone(), doc_path) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Failed to parse doc from {}: {}", source_path, e);
                    continue;
                }
            };

            let mut violations = match engine.lint_document_with_config(&document, &effective_core)
            {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Failed to lint {}: {}", source_path, e);
                    continue;
                }
            };

            // Map line numbers back to the original Rust file
            for violation in &mut violations {
                violation.line = map_line_to_source(violation.line, extracted.start_line);
            }

            if !violations.is_empty() {
                total_violations += violations.len();
                for v in &violations {
                    if v.severity == Severity::Error {
                        has_errors = true;
                    }
                }
                violations_by_file.push((source_path, violations));
            }
        }
    }

    // Count errors and warnings
    let error_count = violations_by_file
        .iter()
        .flat_map(|(_, v)| v)
        .filter(|v| v.severity == Severity::Error)
        .count();
    let warning_count = violations_by_file
        .iter()
        .flat_map(|(_, v)| v)
        .filter(|v| v.severity == Severity::Warning)
        .count();

    // Output results
    match output_format {
        OutputFormat::Default => {
            output::print_cargo_style(&violations_by_file);
            output::print_summary(total_violations, error_count, warning_count, quiet);
        }
        OutputFormat::Json => {
            let output = serde_json::json!({
                "total_violations": total_violations,
                "has_errors": has_errors,
                "files": violations_by_file.iter().map(|(file, violations)| {
                    serde_json::json!({
                        "file": file,
                        "violations": violations
                    })
                }).collect::<Vec<_>>()
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        OutputFormat::Github => {
            for (file_path, violations) in &violations_by_file {
                for violation in violations {
                    let level = match violation.severity {
                        Severity::Error => "error",
                        Severity::Warning => "warning",
                        Severity::Info => "notice",
                    };
                    println!(
                        "::{level} file={file_path},line={}::{}: {}",
                        violation.line, violation.rule_id, violation.message
                    );
                }
            }
        }
        OutputFormat::Sarif => {
            // The rustdoc command has no --output-file flag; redirect stdout instead.
            emit_sarif(&violations_by_file, &engine, None)?;
        }
    }

    if has_errors || (total_violations > 0 && config.fail_on_warnings) {
        process::exit(1);
    }

    Ok(())
}

fn run_preprocessor_mode() -> Result<()> {
    preprocessor::handle_preprocessing()
}

#[cfg(feature = "lsp")]
fn run_lsp_server(stdio: bool, port: Option<u16>) -> Result<()> {
    tokio::runtime::Runtime::new()?
        .block_on(async { lsp_server::run_lsp_server(stdio, port).await })
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::path::Path;

    #[test]
    fn test_documented_rule_count_matches_readme() {
        let counted = documented_rule_count(EXAMPLE_CONFIG_TOML);
        assert!(
            counted > 0,
            "no rule IDs found in example-mdbook-lint.toml comments"
        );

        // README describes the same file; the two must not drift apart.
        // Read at runtime rather than include_str!: README.md lives outside this
        // package, so it is relocated when the crate is packaged for publishing.
        // In the repo (and in CI) it is always there; anywhere else, skip.
        let readme_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../README.md");
        let Ok(readme) = std::fs::read_to_string(&readme_path) else {
            return;
        };

        let claimed = readme
            .lines()
            .find_map(|line| {
                let (before, _) = line.split_once(" rules documented")?;
                before.split_whitespace().next_back()?.parse::<usize>().ok()
            })
            .expect("README no longer contains an 'N rules documented' claim");

        assert_eq!(
            claimed, counted,
            "README claims {claimed} documented rules but example-mdbook-lint.toml documents {counted}"
        );
    }

    #[test]
    fn test_documented_rule_count_parsing() {
        let config = "\
# MD001 - Heading levels
# MD001 - repeated, counted once
# MDBOOK002 - SUMMARY.md structure
# CONTENT001 - Content rule
# ADR001 - ADR rule
# Not a rule ID at all
# MDX999 - unknown prefix
#MD002 - no space after hash
# MD - no digits
[MD003]
style = \"consistent\"
";
        assert_eq!(documented_rule_count(config), 4);
    }

    #[test]
    fn test_cli_parsing() {
        // Test basic lint command
        let args = vec!["mdbook-lint", "lint", "test.md"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Some(Commands::Lint { files, .. }) => {
                assert_eq!(files, vec!["test.md"]);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_lint_with_options() {
        let args = vec![
            "mdbook-lint",
            "lint",
            "test.md",
            "--config",
            "config.toml",
            "--standard-only",
            "--fail-on-warnings",
            "--output",
            "json",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Some(Commands::Lint {
                files,
                config,
                standard_only,
                fail_on_warnings,
                output,
                ..
            }) => {
                assert_eq!(files, vec!["test.md"]);
                assert_eq!(config, Some("config.toml".to_string()));
                assert!(standard_only);
                assert!(fail_on_warnings);
                assert_eq!(output, OutputFormat::Json);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_rules_command() {
        let args = vec!["mdbook-lint", "rules", "--detailed"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Some(Commands::Rules {
                detailed, format, ..
            }) => {
                assert!(detailed);
                assert_eq!(format, RulesFormat::Default);
            }
            _ => panic!("Expected Rules command"),
        }
    }

    #[test]
    fn test_cli_rules_json_format() {
        let args = vec!["mdbook-lint", "rules", "--format", "json"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Some(Commands::Rules { format, .. }) => {
                assert_eq!(format, RulesFormat::Json);
            }
            _ => panic!("Expected Rules command"),
        }
    }

    #[test]
    fn test_cli_init_command() {
        let args = vec!["mdbook-lint", "init", "--format", "yaml", "--include-all"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Some(Commands::Init {
                format,
                include_all,
                ..
            }) => {
                assert_eq!(format, ConfigFormat::Yaml);
                assert!(include_all);
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_cli_preprocessor_command() {
        let args = vec!["mdbook-lint", "preprocessor"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Some(Commands::Preprocessor) => {}
            _ => panic!("Expected Preprocessor command"),
        }
    }

    #[test]
    fn test_output_format_enum() {
        assert_eq!(
            OutputFormat::from_str("default", true).unwrap(),
            OutputFormat::Default
        );
        assert_eq!(
            OutputFormat::from_str("json", true).unwrap(),
            OutputFormat::Json
        );
        assert_eq!(
            OutputFormat::from_str("github", true).unwrap(),
            OutputFormat::Github
        );
    }

    #[test]
    fn test_config_format_enum() {
        assert_eq!(
            ConfigFormat::from_str("toml", true).unwrap(),
            ConfigFormat::Toml
        );
        assert_eq!(
            ConfigFormat::from_str("yaml", true).unwrap(),
            ConfigFormat::Yaml
        );
        assert_eq!(
            ConfigFormat::from_str("json", true).unwrap(),
            ConfigFormat::Json
        );
    }

    #[test]
    fn test_rules_format_enum() {
        assert_eq!(
            RulesFormat::from_str("default", true).unwrap(),
            RulesFormat::Default
        );
        assert_eq!(
            RulesFormat::from_str("json", true).unwrap(),
            RulesFormat::Json
        );
    }

    #[test]
    fn test_cli_with_multiple_files() {
        let args = vec!["mdbook-lint", "lint", "file1.md", "file2.md", "src/"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Some(Commands::Lint { files, .. }) => {
                assert_eq!(files, vec!["file1.md", "file2.md", "src/"]);
            }
            _ => panic!("Expected Lint command"),
        }
    }

    #[test]
    fn test_cli_no_subcommand() {
        let args = vec!["mdbook-lint"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_create_engine_based_on_flags() {
        // Test engine creation with different rule sets
        let mut all_registry = PluginRegistry::new();
        all_registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();
        all_registry
            .register_provider(Box::new(MdBookRuleProvider))
            .unwrap();
        #[cfg(feature = "content")]
        all_registry
            .register_provider(Box::new(ContentRuleProvider))
            .unwrap();
        #[cfg(feature = "adr")]
        all_registry
            .register_provider(Box::new(AdrRuleProvider))
            .unwrap();
        let all_engine = all_registry.create_engine().unwrap();
        let all_rules = all_engine.available_rules().len();

        let mut standard_registry = PluginRegistry::new();
        standard_registry
            .register_provider(Box::new(StandardRuleProvider))
            .unwrap();
        let standard_engine = standard_registry.create_engine().unwrap();
        let standard_rules = standard_engine.available_rules().len();

        let mut mdbook_registry = PluginRegistry::new();
        mdbook_registry
            .register_provider(Box::new(MdBookRuleProvider))
            .unwrap();
        let mdbook_engine = mdbook_registry.create_engine().unwrap();
        let mdbook_rules = mdbook_engine.available_rules().len();

        // All rules should be more than either individual set
        assert!(all_rules > standard_rules);
        assert!(all_rules > mdbook_rules);
        assert!(mdbook_rules >= 4); // At least MDBOOK001-004
    }

    #[test]
    fn test_error_handling_in_main_functions() {
        // Test that error types are properly handled
        use mdbook_lint::error::MdBookLintError;

        let err = MdBookLintError::config_error("Test error");
        assert!(err.to_string().contains("Test error"));
    }

    #[test]
    fn test_looks_like_lint_target() {
        // Markdown files
        assert!(looks_like_lint_target("README.md"));
        assert!(looks_like_lint_target("docs/guide.md"));
        assert!(looks_like_lint_target("file.markdown"));

        // Paths
        assert!(looks_like_lint_target("src/"));
        assert!(looks_like_lint_target("./docs"));
        assert!(looks_like_lint_target("."));
        assert!(looks_like_lint_target(".."));
        assert!(looks_like_lint_target("path/to/file"));

        // Glob patterns
        assert!(looks_like_lint_target("*.md"));
        assert!(looks_like_lint_target("src/**/*.md"));
        assert!(looks_like_lint_target("doc?.md"));

        // Not lint targets
        assert!(!looks_like_lint_target("--fix"));
        assert!(!looks_like_lint_target("-c"));
        assert!(!looks_like_lint_target("lint"));
        assert!(!looks_like_lint_target("rules"));
    }

    #[test]
    fn test_should_infer_lint_subcommand() {
        // Helper to create owned String vec from str slice
        fn args(a: &[&str]) -> Vec<String> {
            a.iter().map(|s| s.to_string()).collect()
        }

        // Should infer lint
        assert!(should_infer_lint_subcommand(&args(&[
            "mdbook-lint",
            "README.md"
        ])));
        assert!(should_infer_lint_subcommand(&args(&[
            "mdbook-lint",
            "src/"
        ])));
        assert!(should_infer_lint_subcommand(&args(&[
            "mdbook-lint",
            "--fix",
            "docs/"
        ])));
        assert!(should_infer_lint_subcommand(&args(&[
            "mdbook-lint",
            "*.md"
        ])));
        assert!(should_infer_lint_subcommand(&args(&["mdbook-lint", "."])));
        assert!(should_infer_lint_subcommand(&args(&[
            "mdbook-lint",
            "--config",
            "custom.toml",
            "."
        ])));

        // Should NOT infer lint
        assert!(!should_infer_lint_subcommand(&args(&["mdbook-lint"]))); // No args = preprocessor
        assert!(!should_infer_lint_subcommand(&args(&[
            "mdbook-lint",
            "lint",
            "README.md"
        ])));
        assert!(!should_infer_lint_subcommand(&args(&[
            "mdbook-lint",
            "rules"
        ])));
        assert!(!should_infer_lint_subcommand(&args(&[
            "mdbook-lint",
            "preprocessor"
        ])));
        assert!(!should_infer_lint_subcommand(&args(&[
            "mdbook-lint",
            "--help"
        ])));
        assert!(!should_infer_lint_subcommand(&args(&["mdbook-lint", "-V"])));
        assert!(!should_infer_lint_subcommand(&args(&[
            "mdbook-lint",
            "check",
            "config.toml"
        ])));
    }

    #[test]
    fn test_maybe_insert_lint_subcommand() {
        // Should insert lint
        assert_eq!(
            maybe_insert_lint_subcommand(vec!["mdbook-lint".to_string(), "README.md".to_string()]),
            vec![
                "mdbook-lint".to_string(),
                "lint".to_string(),
                "README.md".to_string()
            ]
        );

        assert_eq!(
            maybe_insert_lint_subcommand(vec![
                "mdbook-lint".to_string(),
                "--fix".to_string(),
                "docs/".to_string()
            ]),
            vec![
                "mdbook-lint".to_string(),
                "lint".to_string(),
                "--fix".to_string(),
                "docs/".to_string()
            ]
        );

        // Should NOT insert lint
        assert_eq!(
            maybe_insert_lint_subcommand(vec!["mdbook-lint".to_string()]),
            vec!["mdbook-lint".to_string()]
        );

        assert_eq!(
            maybe_insert_lint_subcommand(vec![
                "mdbook-lint".to_string(),
                "lint".to_string(),
                "README.md".to_string()
            ]),
            vec![
                "mdbook-lint".to_string(),
                "lint".to_string(),
                "README.md".to_string()
            ]
        );

        assert_eq!(
            maybe_insert_lint_subcommand(vec!["mdbook-lint".to_string(), "rules".to_string()]),
            vec!["mdbook-lint".to_string(), "rules".to_string()]
        );
    }

    #[test]
    fn test_smart_cli_with_clap_parsing() {
        // Test that after inserting "lint", clap can parse correctly
        let args =
            maybe_insert_lint_subcommand(vec!["mdbook-lint".to_string(), "README.md".to_string()]);
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Some(Commands::Lint { files, .. }) => {
                assert_eq!(files, vec!["README.md"]);
            }
            _ => panic!("Expected Lint command"),
        }

        // Test with flags
        let args = maybe_insert_lint_subcommand(vec![
            "mdbook-lint".to_string(),
            "--fix".to_string(),
            "src/".to_string(),
        ]);
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Some(Commands::Lint { files, fix, .. }) => {
                assert_eq!(files, vec!["src/"]);
                assert!(fix);
            }
            _ => panic!("Expected Lint command"),
        }
    }
}
