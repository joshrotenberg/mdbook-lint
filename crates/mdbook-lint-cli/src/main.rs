mod config;
#[cfg(feature = "lsp")]
mod lsp_server;
mod preprocessor;

use config::Config;

use clap::{Parser, Subcommand, ValueEnum};
use mdbook_lint_core::{
    Document, PluginRegistry, Severity,
    error::Result,
    rule::{RuleCategory, RuleStability},
};
use mdbook_lint_rulesets::{MdBookRuleProvider, StandardRuleProvider};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "mdbook-lint")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "Josh Rotenberg <joshrotenberg@gmail.com>")]
#[command(about = "A markdown linter for mdBook projects")]
struct Cli {
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
        /// Output format for rule information
        #[arg(short, long, value_enum, default_value = "default")]
        format: RulesFormat,
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
}

#[derive(ValueEnum, Clone, PartialEq, Debug)]
enum OutputFormat {
    /// Default human-readable format
    Default,
    /// JSON format for machine processing
    Json,
    /// GitHub Actions format
    Github,
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
    total_rules: usize,
    providers: Vec<JsonRuleProvider>,
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

fn main() {
    let cli = Cli::parse();

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
        }) => run_cli_mode(
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
        ),
        Some(Commands::Rules {
            detailed,
            category,
            provider,
            standard_only,
            mdbook_only,
            format,
        }) => run_rules_command(
            detailed,
            category.as_deref(),
            provider.as_deref(),
            standard_only,
            mdbook_only,
            format,
        ),
        Some(Commands::Check { config }) => run_check_command(&config),
        Some(Commands::Init {
            format,
            output,
            include_all,
        }) => run_init_command(format, output, include_all),
        Some(Commands::Supports { renderer }) => run_supports_check(&renderer),
        #[cfg(feature = "lsp")]
        Some(Commands::Lsp { stdio, port }) => run_lsp_server(stdio, port),
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

/// Simple fix description for basic auto-fixable violations
#[derive(Debug, Clone)]
struct SimpleFix {
    rule_id: String,
    line: usize,
}

/// Determine if a violation can be auto-fixed and return fix information
fn get_simple_fix(violation: &mdbook_lint_core::violation::Violation) -> Option<SimpleFix> {
    match violation.rule_id.as_str() {
        "MD009" => {
            // Trailing spaces - can be fixed by removing trailing whitespace
            Some(SimpleFix {
                rule_id: violation.rule_id.clone(),
                line: violation.line,
            })
        }
        // Add more fixable rules here in the future
        _ => None,
    }
}

/// Apply fixes to file content, returning the fixed content if any fixes were applied
fn apply_fixes_to_content(
    content: &str,
    violations: &[&mdbook_lint_core::violation::Violation],
) -> Result<Option<String>> {
    if violations.is_empty() {
        return Ok(None);
    }

    // Get simple fixes for fixable violations
    let mut fixes: Vec<SimpleFix> = violations
        .iter()
        .filter_map(|v| get_simple_fix(v))
        .collect();

    if fixes.is_empty() {
        return Ok(None);
    }

    // Sort fixes by line (descending) to avoid offset issues when applying
    fixes.sort_by(|a, b| b.line.cmp(&a.line));

    let lines: Vec<&str> = content.lines().collect();
    let mut modified_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    let mut fixes_applied = 0;

    for fix in fixes {
        let line_idx = fix.line.saturating_sub(1);
        if line_idx < modified_lines.len() {
            match fix.rule_id.as_str() {
                "MD009" => {
                    // Remove trailing spaces
                    let original_line = &modified_lines[line_idx];
                    let trimmed_line = original_line.trim_end().to_string();
                    if trimmed_line != *original_line {
                        modified_lines[line_idx] = trimmed_line;
                        fixes_applied += 1;
                    }
                }
                _ => {
                    // Future: handle other fix types
                }
            }
        }
    }

    if fixes_applied > 0 {
        let fixed_content = modified_lines.join("\n");
        if fixed_content != content {
            Ok(Some(fixed_content))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
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

    // Load configuration
    let mut config = if let Some(path) = config_path {
        let config_content = std::fs::read_to_string(path).map_err(|e| {
            mdbook_lint::error::MdBookLintError::config_error(format!(
                "Failed to read config file {path}: {e}"
            ))
        })?;

        // Detect format from extension and content
        if path.ends_with(".toml") {
            Config::from_toml_str(&config_content)?
        } else if path.ends_with(".yaml") || path.ends_with(".yml") {
            Config::from_yaml_str(&config_content)?
        } else if path.ends_with(".json") {
            Config::from_json_str(&config_content)?
        } else {
            // Try to auto-detect format
            config_content.parse()?
        }
    } else {
        Config::default()
    };

    // Override config with CLI flags
    if fail_on_warnings {
        config.fail_on_warnings = true;
    }
    if markdownlint_compatible {
        config.core.markdownlint_compatible = true;
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
        // Clear existing disabled rules and only enable specified rules
        config.core.disabled_rules.clear();

        // Get all available rule IDs and disable everything except enabled ones
        let all_rule_ids = get_all_available_rule_ids();
        for rule_id in all_rule_ids {
            if !enabled_rules.contains(&rule_id) {
                config.core.disabled_rules.push(rule_id);
            }
        }
    }

    // Create appropriate engine based on flags
    let mut registry = PluginRegistry::new();

    if standard_only {
        registry.register_provider(Box::new(StandardRuleProvider))?;
    } else if mdbook_only {
        registry.register_provider(Box::new(MdBookRuleProvider))?;
    } else {
        // Default: use all rules (standard + mdBook)
        registry.register_provider(Box::new(StandardRuleProvider))?;
        registry.register_provider(Box::new(MdBookRuleProvider))?;
    }

    let engine = registry.create_engine()?;

    let mut total_violations = 0;
    let mut has_errors = false;
    let mut violations_by_file = Vec::new();

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

    // Process each markdown file
    for path in markdown_files {
        let file_path = path.to_string_lossy().to_string();

        // Read file content
        let content = std::fs::read_to_string(&path).map_err(|e| {
            mdbook_lint::error::MdBookLintError::document_error(format!(
                "Failed to read file {}: {e}",
                path.display()
            ))
        })?;

        // Create document
        let document = Document::new(content, path.clone())?;

        // Lint with configuration
        let violations = engine.lint_document_with_config(&document, &config.core)?;

        if !violations.is_empty() {
            violations_by_file.push((file_path.clone(), violations.clone()));
            total_violations += violations.len();

            for violation in &violations {
                if violation.severity == Severity::Error {
                    has_errors = true;
                }
            }
        }
    }

    // Apply fixes if requested
    let mut fixes_applied = 0;
    let mut files_modified = 0;

    if apply_fixes {
        for (file_path, violations) in &violations_by_file {
            let fixable_violations: Vec<_> = violations
                .iter()
                .filter(|v| get_simple_fix(v).is_some())
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

                if let Some(fixed_content) =
                    apply_fixes_to_content(&original_content, &fixable_violations)?
                {
                    if dry_run {
                        println!(
                            "Would fix {} issue(s) in {}",
                            fixable_violations.len(),
                            file_path
                        );
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

                        println!(
                            "Fixed {} issue(s) in {}",
                            fixable_violations.len(),
                            file_path
                        );
                        fixes_applied += fixable_violations.len();
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
                let violations = engine.lint_document_with_config(&document, &config.core)?;

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

    // Output results
    match output_format {
        OutputFormat::Default => {
            for (file_path, violations) in &violations_by_file {
                for violation in violations {
                    println!("{file_path}:{violation}");
                }
            }

            if total_violations == 0 {
                println!("✅ No issues found");
            } else {
                println!("Found {total_violations} violation(s)");
            }
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
    } else if mdbook_only {
        registry.register_provider(Box::new(MdBookRuleProvider))?;
    } else {
        // Default: show both standard and mdBook rules
        registry.register_provider(Box::new(StandardRuleProvider))?;
        registry.register_provider(Box::new(MdBookRuleProvider))?;
    }

    let engine = registry.create_engine()?;
    let providers = registry.providers();

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
                    if let Some(rule) = engine.registry().get_rule(rule_id) {
                        let metadata = rule.metadata();

                        // Apply category filter
                        if let Some(filter) = category_filter
                            && format!("{:?}", metadata.category).to_lowercase()
                                != filter.to_lowercase()
                        {
                            continue;
                        }

                        let json_rule = JsonRule {
                            id: rule.id().to_string(),
                            name: rule.name().to_string(),
                            description: rule.description().to_string(),
                            category: JsonRuleCategory::from(&metadata.category),
                            stability: JsonRuleStability::from(&metadata.stability),
                            deprecated: metadata.deprecated,
                            deprecated_reason: metadata.deprecated_reason.map(String::from),
                            replacement: metadata.replacement.map(String::from),
                            introduced_in: metadata.introduced_in.map(String::from),
                            can_fix: rule.can_fix(),
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
                total_rules,
                providers: json_providers,
            };

            println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
        }
        RulesFormat::Default => {
            // Default human-readable output
            if detailed {
                println!("📋 mdbook-lint Rule Information");
                println!("================================\n");

                println!("Available Rule Providers:");
                for provider in providers {
                    if let Some(filter) = provider_filter
                        && provider.provider_id() != filter
                    {
                        continue;
                    }

                    println!(
                        "\n📦 Provider: {} (v{})",
                        provider.provider_id(),
                        provider.version()
                    );
                    println!("   Description: {}", provider.description());
                    println!("   Rules: {}", provider.rule_ids().len());

                    if !provider.rule_ids().is_empty() {
                        println!("   Rule IDs: {}", provider.rule_ids().join(", "));
                    }
                }

                println!("\nDetailed Rule Information:");
                for rule_id in engine.available_rules() {
                    if let Some(rule) = engine.registry().get_rule(rule_id) {
                        let metadata = rule.metadata();

                        // Apply category filter
                        if let Some(filter) = category_filter
                            && format!("{:?}", metadata.category).to_lowercase()
                                != filter.to_lowercase()
                        {
                            continue;
                        }

                        println!("\n🔍 {}: {}", rule.id(), rule.name());
                        println!("   Description: {}", rule.description());
                        println!("   Category: {:?}", metadata.category);
                        println!("   Stability: {:?}", metadata.stability);
                        if let Some(version) = metadata.introduced_in {
                            println!("   Introduced in: {version}");
                        }
                        if metadata.deprecated {
                            println!(
                                "   ⚠️  DEPRECATED: {}",
                                metadata.deprecated_reason.unwrap_or("No reason provided")
                            );
                            if let Some(replacement) = metadata.replacement {
                                println!("   Replacement: {replacement}");
                            }
                        }
                    }
                }
            } else {
                // Simple list mode
                println!("Available Providers:");
                for provider in providers {
                    if let Some(filter) = provider_filter
                        && provider.provider_id() != filter
                    {
                        continue;
                    }
                    println!(
                        "  {} (v{}) - {} rules",
                        provider.provider_id(),
                        provider.version(),
                        provider.rule_ids().len()
                    );
                }

                println!("\nAvailable Rules:");
                let rule_ids = engine.available_rules();
                for (i, rule_id) in rule_ids.iter().enumerate() {
                    if i > 0 && i % 10 == 0 {
                        println!();
                    }
                    print!("{rule_id:12} ");
                }
                println!("\n\nTotal: {} rules available", rule_ids.len());

                if !detailed {
                    println!("\nUse --detailed for more information about each rule.");
                }
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
    let _config = if config_path.extension().and_then(|s| s.to_str()) == Some("toml") {
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

    println!("✅ Configuration file {} is valid", config_path.display());
    Ok(())
}

fn run_init_command(
    format: ConfigFormat,
    output_path: Option<PathBuf>,
    include_all: bool,
) -> Result<()> {
    let default_config = if include_all {
        // Create config with all available rules listed
        let mut registry = PluginRegistry::new();
        registry.register_provider(Box::new(StandardRuleProvider))?;
        registry.register_provider(Box::new(MdBookRuleProvider))?;
        let engine = registry.create_engine()?;

        let mut config = Config::default();

        // Add all available rules as enabled
        let rule_ids = engine.available_rules();
        config.core.enabled_rules = rule_ids.into_iter().map(|s| s.to_string()).collect();
        config
    } else {
        Config::default()
    };

    let (content, extension) = match format {
        ConfigFormat::Toml => (default_config.to_toml_string()?, "toml"),
        ConfigFormat::Yaml => (default_config.to_yaml_string()?, "yaml"),
        ConfigFormat::Json => (default_config.to_json_string()?, "json"),
    };

    let output_file =
        output_path.unwrap_or_else(|| PathBuf::from(format!("mdbook-lint.{extension}")));

    std::fs::write(&output_file, content).map_err(|e| {
        mdbook_lint::error::MdBookLintError::config_error(format!(
            "Failed to write config file {}: {}",
            output_file.display(),
            e
        ))
    })?;

    println!("✅ Configuration file created: {}", output_file.display());
    if include_all {
        println!("📋 Includes all 63 available rules");
    }
    println!("💡 Edit the file to customize rule settings for your project");

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

fn run_preprocessor_mode() -> Result<()> {
    preprocessor::handle_preprocessing()
}

#[cfg(feature = "lsp")]
fn run_lsp_server(stdio: bool, port: Option<u16>) -> Result<()> {
    tokio::runtime::Runtime::new()?
        .block_on(async { lsp_server::run_lsp_server(stdio, port).await })
}

/// Get all available rule IDs from all providers
fn get_all_available_rule_ids() -> Vec<String> {
    let mut registry = PluginRegistry::new();

    // Add all providers
    registry
        .register_provider(Box::new(StandardRuleProvider))
        .unwrap();
    registry
        .register_provider(Box::new(MdBookRuleProvider))
        .unwrap();

    // Create engine to get available rules
    let engine = registry.create_engine().unwrap();
    engine
        .available_rules()
        .iter()
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

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
}
