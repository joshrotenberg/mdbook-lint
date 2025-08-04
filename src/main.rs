use clap::{Parser, Subcommand, ValueEnum};
use mdbook_lint::{
    Config, Document, PluginRegistry, Severity, create_engine_with_all_rules, create_mdbook_engine,
    create_standard_engine, error::Result, preprocessor::handle_preprocessing,
    rules::MdBookRuleProvider, standard_provider::StandardRuleProvider,
};
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
        /// Output format
        #[arg(long, value_enum, default_value = "default")]
        output: OutputFormat,
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
        /// Show only mdBook rules (MDBOOK001-004)
        #[arg(long)]
        mdbook_only: bool,
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
}

#[derive(ValueEnum, Clone)]
enum OutputFormat {
    /// Default human-readable format
    Default,
    /// JSON format for machine processing
    Json,
    /// GitHub Actions format
    Github,
}

#[derive(ValueEnum, Clone)]
enum ConfigFormat {
    /// TOML format (recommended)
    Toml,
    /// YAML format
    Yaml,
    /// JSON format
    Json,
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
            output,
        }) => run_cli_mode(
            &files,
            config.as_deref(),
            standard_only,
            mdbook_only,
            fail_on_warnings,
            output,
        ),
        Some(Commands::Rules {
            detailed,
            category,
            provider,
            standard_only,
            mdbook_only,
        }) => run_rules_command(
            detailed,
            category.as_deref(),
            provider.as_deref(),
            standard_only,
            mdbook_only,
        ),
        Some(Commands::Check { config }) => run_check_command(&config),
        Some(Commands::Init {
            format,
            output,
            include_all,
        }) => run_init_command(format, output, include_all),
        Some(Commands::Supports { renderer }) => run_supports_check(&renderer),
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

fn run_cli_mode(
    files: &[String],
    config_path: Option<&str>,
    standard_only: bool,
    mdbook_only: bool,
    fail_on_warnings: bool,
    output_format: OutputFormat,
) -> Result<()> {
    // Validate mutually exclusive flags
    if standard_only && mdbook_only {
        return Err(mdbook_lint::error::MdBookLintError::config_error(
            "Cannot specify both --standard-only and --mdbook-only",
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

    // Create appropriate engine based on flags
    let engine = if standard_only {
        create_standard_engine()
    } else if mdbook_only {
        create_mdbook_engine()
    } else {
        // Default: use all rules (standard + mdBook)
        create_engine_with_all_rules()
    };

    let mut total_violations = 0;
    let mut has_errors = false;
    let mut violations_by_file = Vec::new();

    // Process files
    for file_path in files {
        let path = PathBuf::from(file_path);

        // Skip non-markdown files
        if let Some(ext) = path.extension() {
            if !matches!(ext.to_str(), Some("md") | Some("markdown")) {
                continue;
            }
        }

        // Read file content
        let content = std::fs::read_to_string(&path).map_err(|e| {
            mdbook_lint::error::MdBookLintError::document_error(format!(
                "Failed to read file {file_path}: {e}"
            ))
        })?;

        // Create document
        let document = Document::new(content, path.clone())?;

        // Lint with configuration
        let violations = engine.lint_document_with_config(&document, &config)?;

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

    if detailed {
        println!("📋 mdbook-lint Rule Information");
        println!("================================\n");

        println!("Available Rule Providers:");
        for provider in providers {
            if let Some(filter) = provider_filter {
                if provider.provider_id() != filter {
                    continue;
                }
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
                if let Some(filter) = category_filter {
                    if format!("{:?}", metadata.category).to_lowercase() != filter.to_lowercase() {
                        continue;
                    }
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
            if let Some(filter) = provider_filter {
                if provider.provider_id() != filter {
                    continue;
                }
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
        let engine = create_engine_with_all_rules();
        let mut config = Config::default();

        // Add all available rules as enabled
        let rule_ids = engine.available_rules();
        config.enabled_rules = rule_ids.into_iter().map(|s| s.to_string()).collect();
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
    handle_preprocessing()
}
