#![recursion_limit = "512"]

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use sanctifier_core::SanctifyConfig;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tracing::error;

mod commands;
mod logging;
mod telemetry;
pub mod vulndb;

#[derive(Parser)]
#[command(
    name = "sanctifier",
    version,
    about = "Soroban smart contract security analyzer"
)]
struct Cli {
    /// Disable coloured output (also respects NO_COLOR env var)
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Analyze a Soroban contract for vulnerabilities
    Analyze(commands::analyze::AnalyzeArgs),
    /// Initialize a .sanctify.toml configuration file
    Init(commands::init::InitArgs),
    /// Language Server Protocol (LSP) for editor integration
    Lsp(commands::lsp::LspArgs),
    /// Generate a security report
    Report(commands::report::ReportArgs),
    /// Estimate gas / instruction costs for a contract source file or workspace
    Gas(commands::gas::GasArgs),
    /// Detect potential storage key collisions in Soroban contracts
    Storage(commands::storage::StorageArgs),
    /// Initialize Sanctifier in a new project
    Init(commands::init::InitArgs),
    /// Install git hooks (pre-commit, pre-push) to run Sanctifier automatically
    InstallHooks(commands::install_hooks::InstallHooksArgs),
    /// Show per-contract complexity metrics (cyclomatic complexity, nesting, LOC)
    Complexity(commands::complexity::ComplexityArgs),
    /// Generate a Graphviz DOT call graph of cross-contract calls (env.invoke_contract)
    Callgraph {
        /// Path to a contract directory, workspace directory, or a single .rs file
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Output format: text | json | junit
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Output DOT file path
        #[arg(short, long, default_value = "callgraph.dot")]
        output: PathBuf,
    },
    /// Apply auto-fix patches to a contract; use --interactive to review each patch
    Fix(commands::fix::FixArgs),
    /// Explain a finding code (e.g. S001, S003) with details and remediation
    Explain(commands::explain::ExplainArgs),
    /// Check for and download the latest Sanctifier binary
    Update,
    /// Self-update with checksum verification via GitHub Releases
    Upgrade(commands::upgrade::UpgradeArgs),
    /// Detect reentrancy vulnerabilities (state mutation before external call)
    Reentrancy(commands::reentrancy::ReentrancyArgs),
    /// Verify local source against on-chain bytecode
    Verify(commands::verify::VerifyArgs),
    /// Analyze an entire Cargo workspace (multiple contracts/libs)
    Workspace(commands::workspace::WorkspaceArgs),
    /// Watch for file changes and auto-rerun analysis
    Watch(commands::watch::WatchArgs),
    /// Generate shell completions for bash, zsh, fish, powershell, or elvish
    Completions {
        /// Shell type: bash, zsh, fish, powershell, elvish
        #[arg(value_parser = clap::value_parser!(Shell))]
        shell: Shell,
    },
    /// Suppress a finding by adding it to .sanctify.toml
    Suppress(commands::suppress::SuppressArgs),
    /// Start HTTP server mode for CI integration
    Serve(commands::serve::ServeArgs),
    /// Run the analyser on a contract corpus and emit a per-rule performance table
    Benchmark(commands::benchmark::BenchmarkArgs),
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        std::process::exit(2);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Analyze(args) => commands::analyze::exec(args),
        Commands::Init(args) => commands::init::exec(args, None),
        Commands::Lsp(args) => commands::lsp::exec(args),
        Commands::Report(args) => commands::report::exec(args),
    };

    loop {
        let config_path = current.join(".sanctify.toml");
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                match toml::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => {
                        eprintln!(
                            "Error: Found .sanctify.toml at {} but it could not be parsed:\n  {}\n\
                             \n\
                             Run 'sanctifier init' to regenerate a valid config, or check the schema at:\n\
                             https://github.com/HyperSafeD/Sanctifier/blob/main/schemas/sanctify-config.schema.json",
                            config_path.display(),
                            e
                        );
                        std::process::exit(1);
                    }
                }
            }
        }
        if !current.pop() {
            break;
        }
    }
    SanctifyConfig::default()
}
