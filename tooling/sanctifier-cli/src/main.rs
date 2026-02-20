use clap::{Parser, Subcommand};
use colored::*;
use std::path::{Path, PathBuf};
use std::fs;
use sanctifier_core::{Analyzer, SizeWarning, UnsafePattern, PatternType};

#[derive(Parser)]
#[command(name = "sanctifier")]
#[command(about = "Stellar Soroban Security & Formal Verification Suite", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze a Soroban contract for vulnerabilities
    Analyze {
        /// Path to the contract directory or Cargo.toml
        #[arg(default_value = ".")]
        path: PathBuf,
        
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Limit for ledger entry size in bytes
        #[arg(short, long, default_value = "64000")]
        limit: usize,
    },
    /// Generate a security report
    Report {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Initialize Sanctifier in a new project
    Init,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Analyze { path, format, limit } => {
            if !is_soroban_project(path) {
                eprintln!("{} Error: {:?} is not a valid Soroban project. (Missing Cargo.toml with 'soroban-sdk' dependency)", "❌".red(), path);
                std::process::exit(1);
            }

            println!("{} Sanctifier: Valid Soroban project found at {:?}", "✨".green(), path);
            println!("{} Analyzing contract at {:?}...", "🔍".blue(), path);
            
            let mut analyzer = Analyzer::new(false);
            analyzer.ledger_limit = *limit;
            
            let mut all_size_warnings = Vec::new();
            let mut all_unsafe_patterns = Vec::new();
            let mut all_auth_gaps = Vec::new();
            let mut all_panic_issues = Vec::new();

            println!("Debug: is_dir? {}, extension: {:?}", path.is_dir(), path.extension());
            if path.is_dir() {
<<<<<<< HEAD
                analyze_directory(path, &analyzer, &mut all_size_warnings, &mut all_unsafe_patterns, &mut all_auth_gaps);
=======
                analyze_directory(path, &analyzer, &mut all_warnings, &mut all_auth_gaps, &mut all_panic_issues);
>>>>>>> 713f0fc (feat: add static analysis check)
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                println!("Debug: Is a .rs file");
                if let Ok(content) = fs::read_to_string(path) {
<<<<<<< HEAD
                    let size_warnings = analyzer.analyze_ledger_size(&content);
                    all_size_warnings.extend(size_warnings.clone());
                    
                    let unsafe_patterns = analyzer.analyze_unsafe_patterns(&content);
                    all_unsafe_patterns.extend(unsafe_patterns.clone());

=======
                    all_warnings.extend(analyzer.analyze_ledger_size(&content));
                    
>>>>>>> 713f0fc (feat: add static analysis check)
                    let gaps = analyzer.scan_auth_gaps(&content);
                    for g in gaps {
                        all_auth_gaps.push(format!("{}: {}", path.display(), g));
                    }
<<<<<<< HEAD
                    println!("Found {} size warnings, {} unsafe patterns, and {} auth gaps in {:?}", size_warnings.len(), unsafe_patterns.len(), gaps.len(), path);
                } else {
                    println!("Debug: Failed to read file {:?}", path);
=======

                    let panics = analyzer.scan_panics(&content);
                    for p in panics {
                        let mut p_mod = p.clone();
                        p_mod.location = format!("{}: {}", path.display(), p.location);
                        all_panic_issues.push(p_mod);
                    }
>>>>>>> 713f0fc (feat: add static analysis check)
                }
            } else {
                println!("Debug: Path neither dir nor .rs file");
            }

            println!("{} Static analysis complete.", "✅".green());
            
            if format == "json" {
                let output = serde_json::json!({
                    "size_warnings": all_size_warnings,
                    "unsafe_patterns": all_unsafe_patterns,
                    "auth_gaps": all_auth_gaps,
                });
                println!("{}", serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string()));
            } else {
                if all_size_warnings.is_empty() && all_unsafe_patterns.is_empty() {
                    println!("No issues found.");
                } else {
                    for warning in all_size_warnings {
                        println!(
                            "{} Warning: Struct {} is approaching ledger entry size limit!",
                            "⚠️".yellow(),
                            warning.struct_name.bold()
                        );
                        println!(
                            "   Estimated size: {} bytes (Limit: {} bytes)",
                            warning.estimated_size.to_string().red(),
                            warning.limit
                        );
                    }

                    for pattern in all_unsafe_patterns {
                        let msg = match pattern.pattern_type {
                            PatternType::Panic => "Explicit panic!() call detected".red(),
                            PatternType::Unwrap => "unwrap() call detected".yellow(),
                            PatternType::Expect => "expect() call detected".yellow(),
                        };
                        println!(
                            "{} {}: {}",
                            "🚨".red(),
                            msg,
                            format!("{}:{}", pattern.line, pattern.snippet).bold()
                        );
                    }
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
            }
        },
        Commands::Report { output } => {
            println!("{} Generating report...", "📄".yellow());
            if let Some(p) = output {
                println!("Report saved to {:?}", p);
            } else {
                println!("Report printed to stdout.");
            }
        },
        Commands::Init => {
            println!("{} Initializing Sanctifier configuration...", "⚙️".cyan());
            println!("Created .sanctify.toml");
        }
    }
}

<<<<<<< HEAD
fn analyze_directory(
    dir: &Path, 
    analyzer: &Analyzer, 
    all_size_warnings: &mut Vec<SizeWarning>,
    all_unsafe_patterns: &mut Vec<UnsafePattern>,
    all_auth_gaps: &mut Vec<String>
) {
=======
fn is_soroban_project(path: &Path) -> bool {
    let cargo_toml_path = if path.is_dir() {
        path.join("Cargo.toml")
    } else if path.file_name().and_then(|s| s.to_str()) == Some("Cargo.toml") {
        path.to_path_buf()
    } else {
        // If it's a .rs file, look for Cargo.toml in parent directories
        let mut current = path.parent();
        let mut found = None;
        while let Some(p) = current {
            let cargo = p.join("Cargo.toml");
            if cargo.exists() {
                found = Some(cargo);
                break;
            }
            current = p.parent();
        }
        match found {
            Some(f) => f,
            None => return false,
        }
    };

    if !cargo_toml_path.exists() {
        return false;
    }

    if let Ok(content) = fs::read_to_string(cargo_toml_path) {
        content.contains("soroban-sdk")
    } else {
        false
    }
}

<<<<<<< HEAD
fn analyze_directory(dir: &Path, analyzer: &Analyzer, all_warnings: &mut Vec<sanctifier_core::SizeWarning>, all_auth_gaps: &mut Vec<String>) {
>>>>>>> 2010490 (feat: implement 'analyze' command scaffolding with Soroban project verification (#4))
=======
fn analyze_directory(
    dir: &Path, 
    analyzer: &Analyzer, 
    all_warnings: &mut Vec<sanctifier_core::SizeWarning>, 
    all_auth_gaps: &mut Vec<String>,
    all_panic_issues: &mut Vec<sanctifier_core::PanicIssue>
) {
>>>>>>> 713f0fc (feat: add static analysis check)
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
<<<<<<< HEAD
                analyze_directory(&path, analyzer, all_size_warnings, all_unsafe_patterns, all_auth_gaps);
=======
                analyze_directory(&path, analyzer, all_warnings, all_auth_gaps, all_panic_issues);
>>>>>>> 713f0fc (feat: add static analysis check)
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    let warnings = analyzer.analyze_ledger_size(&content);
                    for mut w in warnings {
                        w.struct_name = format!("{}: {}", path.display(), w.struct_name);
                        all_size_warnings.push(w);
                    }

                    let patterns = analyzer.analyze_unsafe_patterns(&content);
                    for mut p in patterns {
                        p.snippet = format!("{}:{}", path.display(), p.snippet);
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
                }
            }
        }
    }
}
