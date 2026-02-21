use clap::{Parser, Subcommand};
use colored::*;
<<<<<<< HEAD
use std::fs;
use std::path::{Path, PathBuf};
use sanctifier_core::{Analyzer, ArithmeticIssue, SizeWarning, UnsafePattern, PatternType, Finding, SanctifyConfig};
=======
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::fs;
use sanctifier_core::{Analyzer, CustomRule, CustomRuleMatch, ArithmeticIssue, SizeWarning, UnsafePattern, PatternType};
>>>>>>> 1c80bbf (feat: Support for Custom Linter Rules)

#[derive(Parser)]
#[command(name = "sanctifier")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze a Soroban contract
    Analyze {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(short, long, default_value = "text")]
        format: String,
        #[arg(short, long, default_value = "64000")]
        limit: usize,
    },
    /// Initialize Sanctifier
    Init,
}

#[derive(Deserialize)]
struct Config {
    rules: Vec<CustomRule>,
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Analyze { path, format, limit } => {
            let is_json = format == "json";
<<<<<<< HEAD
            if !is_soroban_project(path) { eprintln!("{} Error: {:?} is not a valid Soroban project.", "❌".red(), path); std::process::exit(1); }
            let mut config = if Path::new(".sanctify.toml").exists() {
                let content = fs::read_to_string(".sanctify.toml").unwrap_or_default();
                toml::from_str(&content).unwrap_or_else(|_| SanctifyConfig::default())
            } else { SanctifyConfig::default() };
            if *limit != 64000 { config.ledger_limit = *limit; }
            if is_json { eprintln!("{} Analyzing contract at {:?}...", "🔍".blue(), path); } else { println!("{} Analyzing contract at {:?}...", "🔍".blue(), path); }
            let analyzer = Analyzer::new(config);
            let mut all_size_warnings = Vec::new();
            let mut all_unsafe_patterns = Vec::new();
            let mut all_auth_gaps = Vec::new();
            let mut all_panic_issues = Vec::new();
            let mut all_arithmetic_issues = Vec::new();
            if path.is_dir() {
                analyze_directory(path, &analyzer, &mut all_size_warnings, &mut all_unsafe_patterns, &mut all_auth_gaps, &mut all_panic_issues, &mut all_arithmetic_issues);
            } else {
                if let Ok(content) = fs::read_to_string(path) {
                    all_size_warnings.extend(analyzer.analyze_ledger_size(&content));
                    let p = analyzer.analyze_unsafe_patterns(&content);
                    for mut x in p { x.snippet = format!("{}: {}", path.display(), x.snippet); all_unsafe_patterns.push(x); }
                    let g = analyzer.scan_auth_gaps(&content);
                    for x in g { all_auth_gaps.push(format!("{}: {}", path.display(), x)); }
                    let pan = analyzer.scan_panics(&content);
                    for x in pan { let mut m = x.clone(); m.location = format!("{}: {}", path.display(), m.location); all_panic_issues.push(m); }
                    let ar = analyzer.scan_arithmetic_overflow(&content);
                    for mut x in ar { x.location = format!("{}: {}", path.display(), x.location); all_arithmetic_issues.push(x); }
=======

            if !is_soroban_project(path) {
                eprintln!("{} Error: {:?} is not a valid Soroban project. (Missing Cargo.toml with 'soroban-sdk' dependency)", "❌".red(), path);
                std::process::exit(1);
            }

            // In JSON mode, send informational lines to stderr so stdout is clean JSON.
            if is_json {
                eprintln!("{} Sanctifier: Valid Soroban project found at {:?}", "✨".green(), path);
                eprintln!("{} Analyzing contract at {:?}...", "🔍".blue(), path);
            } else {
                println!("{} Sanctifier: Valid Soroban project found at {:?}", "✨".green(), path);
                println!("{} Analyzing contract at {:?}...", "🔍".blue(), path);
            }
            
            let config = load_config(path);
            let mut analyzer = Analyzer::new(false);
            analyzer.ledger_limit = *limit;
            
            let mut all_size_warnings: Vec<SizeWarning> = Vec::new();
            let mut all_unsafe_patterns: Vec<UnsafePattern> = Vec::new();
            let mut all_auth_gaps: Vec<String> = Vec::new();
            let mut all_panic_issues = Vec::new();
            let mut all_arithmetic_issues: Vec<ArithmeticIssue> = Vec::new();
            let mut all_custom_rule_matches: Vec<CustomRuleMatch> = Vec::new();

            if path.is_dir() {
                analyze_directory(path, &analyzer, &config.rules, &mut all_size_warnings, &mut all_unsafe_patterns, &mut all_auth_gaps, &mut all_panic_issues, &mut all_arithmetic_issues, &mut all_custom_rule_matches);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = fs::read_to_string(path) {
                    all_size_warnings.extend(analyzer.analyze_ledger_size(&content));

                    let patterns = analyzer.analyze_unsafe_patterns(&content);
                    for mut p in patterns {
                        p.snippet = format!("{}: {}", path.display(), p.snippet);
                        all_unsafe_patterns.push(p);
                    }

                    let gaps = analyzer.scan_auth_gaps(&content);
                    for g in gaps {
                        all_auth_gaps.push(format!("{}: {}", path.display(), g));
                    }

                    let panics = analyzer.scan_panics(&content);
                    for p in panics {
                        let mut p_mod = p.clone();
                        p_mod.location = format!("{}: {}", path.display(), p.location);
                        all_panic_issues.push(p_mod);
                    }

                    let arith = analyzer.scan_arithmetic_overflow(&content);
                    for mut a in arith {
                        a.location = format!("{}: {}", path.display(), a.location);
                        all_arithmetic_issues.push(a);
                    }

                    let custom_matches = analyzer.analyze_custom_rules(&content, &config.rules);
                    all_custom_rule_matches.extend(custom_matches);
>>>>>>> 1c80bbf (feat: Support for Custom Linter Rules)
                }
            }
            if is_json {
                let mut findings = Vec::new();
                for w in &all_size_warnings { findings.push(Finding { severity: "warning".to_string(), file: w.struct_name.clone(), line: 0, message: format!("Ledger size warning: estimated {} bytes", w.estimated_size) }); }
                for p in &all_unsafe_patterns { findings.push(Finding { severity: "warning".to_string(), file: p.snippet.clone(), line: p.line, message: "Unsafe pattern detected".to_string() }); }
                for g in &all_auth_gaps { findings.push(Finding { severity: "error".to_string(), file: g.clone(), line: 0, message: "Potential authentication gap".to_string() }); }
                for p in &all_panic_issues { findings.push(Finding { severity: "warning".to_string(), file: p.location.clone(), line: 0, message: format!("Explicit panic: {}", p.issue_type) }); }
                for a in &all_arithmetic_issues { findings.push(Finding { severity: "warning".to_string(), file: a.location.clone(), line: 0, message: format!("Unchecked `{}`: {}", a.operation, a.suggestion) }); }
                println!("{}", serde_json::to_string_pretty(&findings).unwrap());
            } else {
<<<<<<< HEAD
                if all_size_warnings.is_empty() && all_unsafe_patterns.is_empty() && all_auth_gaps.is_empty() && all_panic_issues.is_empty() && all_arithmetic_issues.is_empty() { println!("No issues found."); } else {
                    for gap in &all_auth_gaps { println!("{} Auth Gap: {}", "🛑".red(), gap); }
                    for ar in &all_arithmetic_issues { println!("{} Unchecked {}: {} ({})", "🔢".yellow(), ar.operation, ar.suggestion, ar.location); }
=======
                println!("{} Static analysis complete.", "✅".green());
            }
            
            if format == "json" {
                let output = serde_json::json!({
                    "size_warnings": all_size_warnings,
                    "unsafe_patterns": all_unsafe_patterns,
                    "auth_gaps": all_auth_gaps,
                    "panic_issues": all_panic_issues,
                    "arithmetic_issues": all_arithmetic_issues,
                    "custom_rule_matches": all_custom_rule_matches,
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string()));
            } else {
                if !all_size_warnings.is_empty() {
                    println!("\n{} Found potential Ledger Size Issues!", "📦".yellow());
                    for warning in all_size_warnings {
                        println!(
                            "   {} Warning: Struct {} is approaching ledger entry size limit!",
                            "⚠️".yellow(),
                            warning.struct_name.bold()
                        );
                        println!(
                            "      Estimated size: {} bytes (Limit: {} bytes)",
                            warning.estimated_size.to_string().red(),
                            warning.limit
                        );
                    }
                } else {
                    println!("\nNo ledger size issues found.");
                }

                if !all_auth_gaps.is_empty() {
                    println!("\n{} Found potential Authentication Gaps!", "🛑".red());
                    for gap in all_auth_gaps {
                        println!("   {} Function {} is modifying state without require_auth()", "->".red(), gap.bold());
                    }
                } else {
                    println!("\nNo authentication gaps found.");
                }

                if !all_panic_issues.is_empty() {
                    println!("\n{} Found explicit Panics/Unwraps!", "🛑".red());
                    for issue in all_panic_issues {
                        println!(
                            "   {} Function {}: Using {} (Location: {})",
                            "->".red(),
                            issue.function_name.bold(),
                            issue.issue_type.yellow().bold(),
                            issue.location
                        );
                    }
                    println!("   {} Tip: Prefer returning Result or Error types for better contract safety.", "💡".blue());
                } else {
                    println!("\nNo panic/unwrap issues found.");
                }

                if !all_arithmetic_issues.is_empty() {
                    println!("\n{} Found unchecked Arithmetic Operations!", "🔢".yellow());
                    for issue in all_arithmetic_issues {
                        println!(
                            "   {} Function {}: Unchecked `{}` ({})",
                            "->".red(),
                            issue.function_name.bold(),
                            issue.operation.yellow().bold(),
                            issue.location
                        );
                        println!("      {} {}", "💡".blue(), issue.suggestion);
                    }
                } else {
                    println!("\nNo arithmetic overflow risks found.");
>>>>>>> 1c80bbf (feat: Support for Custom Linter Rules)
                }

                if !all_custom_rule_matches.is_empty() {
                    println!("\n{} Found Custom Rule Matches!", "📜".yellow());
                    for m in all_custom_rule_matches {
                        println!(
                            "   {} Rule {}: `{}` (Line: {})",
                            "->".yellow(),
                            m.rule_name.bold(),
                            m.snippet.trim().italic(),
                            m.line
                        );
                    }
                } else {
                    println!("\nNo custom rule matches found.");
                }
            }
        }
        Commands::Init => {
<<<<<<< HEAD
            let config = SanctifyConfig::default();
            let toml = toml::to_string_pretty(&config).unwrap_or_default();
            if !Path::new(".sanctify.toml").exists() { fs::write(".sanctify.toml", toml).ok(); println!("{} Created .sanctify.toml", "✅".green()); } else { println!("{} .sanctify.toml already exists", "⚠️".yellow()); }
=======
            let config_path = Path::new(".sanctify.toml");
            if config_path.exists() {
                println!("{} .sanctify.toml already exists.", "✅".green());
                return;
            }

            let default_config = r#"
# Sanctifier Configuration
# ------------------------
# Define custom rules for the static analyzer.
# Each rule has a name, a regex pattern, and a description.

[[rules]]
name = "no-vec"
pattern = "Vec<"
description = "Discourage the use of `Vec` in favor of `Map` or other bounded data structures."

"#;
            if fs::write(config_path, default_config).is_ok() {
                println!("{} Created default .sanctify.toml configuration.", "⚙️".cyan());
            } else {
                eprintln!("{} Failed to create .sanctify.toml.", "❌".red());
            }
>>>>>>> 1c80bbf (feat: Support for Custom Linter Rules)
        }
    }
}

fn is_soroban_project(path: &Path) -> bool {
    let cargo = if path.is_dir() { path.join("Cargo.toml") } else { path.to_path_buf() };
    if !cargo.exists() { return false; }
    fs::read_to_string(cargo).map(|c| c.contains("soroban-sdk")).unwrap_or(false)
}

<<<<<<< HEAD
fn analyze_directory(dir: &Path, analyzer: &Analyzer, all_size_warnings: &mut Vec<SizeWarning>, all_unsafe_patterns: &mut Vec<UnsafePattern>, all_auth_gaps: &mut Vec<String>, all_panic_issues: &mut Vec<sanctifier_core::PanicIssue>, all_arithmetic_issues: &mut Vec<ArithmeticIssue>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() { analyze_directory(&path, analyzer, all_size_warnings, all_unsafe_patterns, all_auth_gaps, all_panic_issues, all_arithmetic_issues); } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    let w = analyzer.analyze_ledger_size(&content);
                    for mut x in w { x.struct_name = format!("{}: {}", path.display(), x.struct_name); all_size_warnings.push(x); }
                    let p = analyzer.analyze_unsafe_patterns(&content);
                    for mut x in p { x.snippet = format!("{}: {}", path.display(), x.snippet); all_unsafe_patterns.push(x); }
                    let g = analyzer.scan_auth_gaps(&content);
                    for x in g { all_auth_gaps.push(format!("{}: {}", path.display(), x)); }
                    let pan = analyzer.scan_panics(&content);
                    for x in pan { let mut m = x.clone(); m.location = format!("{}: {}", path.display(), m.location); all_panic_issues.push(m); }
                    let ar = analyzer.scan_arithmetic_overflow(&content);
                    for mut x in ar { x.location = format!("{}: {}", path.display(), x.location); all_arithmetic_issues.push(x); }
=======
fn analyze_directory(
    dir: &Path,
    analyzer: &Analyzer,
    rules: &[CustomRule],
    all_size_warnings: &mut Vec<SizeWarning>,
    all_unsafe_patterns: &mut Vec<UnsafePattern>,
    all_auth_gaps: &mut Vec<String>,
    all_panic_issues: &mut Vec<sanctifier_core::PanicIssue>,
    all_arithmetic_issues: &mut Vec<ArithmeticIssue>,
    all_custom_rule_matches: &mut Vec<CustomRuleMatch>,
) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                analyze_directory(&path, analyzer, rules, all_size_warnings, all_unsafe_patterns, all_auth_gaps, all_panic_issues, all_arithmetic_issues, all_custom_rule_matches);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    let warnings = analyzer.analyze_ledger_size(&content);
                    all_size_warnings.extend(warnings);

                    let gaps = analyzer.scan_auth_gaps(&content);
                    for g in gaps {
                        all_auth_gaps.push(format!("{}: {}", path.display(), g));
                    }

                    let panics = analyzer.scan_panics(&content);
                    for p in panics {
                        let mut p_mod = p.clone();
                        p_mod.location = format!("{}: {}", path.display(), p.location);
                        all_panic_issues.push(p_mod);
                    }

                    let arith = analyzer.scan_arithmetic_overflow(&content);
                    for mut a in arith {
                        a.location = format!("{}: {}", path.display(), a.location);
                        all_arithmetic_issues.push(a);
                    }

                    let custom_matches = analyzer.analyze_custom_rules(&content, rules);
                    all_custom_rule_matches.extend(custom_matches);
>>>>>>> 1c80bbf (feat: Support for Custom Linter Rules)
                }
            }
        }
    }
}

fn load_config(path: &Path) -> Config {
    find_config_path(path)
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|content| toml::from_str(&content).ok())
        .unwrap_or(Config { rules: vec![] })
}

fn find_config_path(start_path: &Path) -> Option<PathBuf> {
    let mut current = if start_path.is_dir() {
        Some(start_path.to_path_buf())
    } else {
        start_path.parent().map(|p| p.to_path_buf())
    };

    while let Some(path) = current {
        let config_path = path.join(".sanctify.toml");
        if config_path.exists() {
            return Some(config_path);
        }
        current = if path.parent().is_some() {
            path.parent().map(|p| p.to_path_buf())
        } else {
            None
        }
    }
    None
}
