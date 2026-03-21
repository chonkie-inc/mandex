mod commands;
mod config;
#[cfg(feature = "reranker")]
mod rerank;
mod storage;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mx", version, about = "Documentation packages for AI agents")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Download a documentation package
    Pull {
        /// Package name, optionally with version (e.g. pytorch@2.3.0)
        package: String,
    },
    /// Search across installed documentation packages
    Search {
        /// Optional package name to search within
        #[arg(num_args = 1..=2)]
        args: Vec<String>,
        /// Number of results to return
        #[arg(short = 'n', long = "limit")]
        limit: Option<usize>,
        /// Enable ONNX reranker for this query
        #[arg(long = "rerank", conflicts_with = "no_rerank")]
        rerank: bool,
        /// Disable ONNX reranker for this query
        #[arg(long = "no-rerank", conflicts_with = "rerank")]
        no_rerank: bool,
        /// Number of FTS5 candidates to fetch before reranking
        #[arg(long = "rerank-candidates")]
        rerank_candidates: Option<usize>,
    },
    /// Show a specific documentation entry
    Show {
        /// Package name
        package: String,
        /// Entry name
        entry: String,
    },
    /// List installed documentation packages
    List,
    /// Build a .mandex package from a documentation directory
    Build {
        /// Path to the documentation directory
        path: String,
        /// Package name
        #[arg(long)]
        name: String,
        /// Package version
        #[arg(long)]
        version: String,
        /// Output file path (defaults to {name}@{version}.mandex)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Show info about an installed package
    Info {
        /// Package name
        package: String,
    },
    /// Remove an installed package
    Remove {
        /// Package name
        package: String,
        /// Specific version to remove (removes all versions if omitted)
        #[arg(long)]
        version: Option<String>,
    },
    /// Set up AI coding assistant integrations
    Init {
        /// Non-interactive: auto-install detected integrations
        #[arg(long, short)]
        yes: bool,
    },
    /// Sync documentation for all project dependencies
    Sync,
}

fn main() -> Result<()> {
    let cfg = config::ConfigFile::load()?;
    let cli = Cli::parse();

    match cli.command {
        Commands::Pull { package } => commands::pull::run(&package),
        Commands::Search {
            args,
            limit,
            rerank,
            no_rerank,
            rerank_candidates,
        } => {
            let (package, query) = if args.len() == 2 {
                (Some(args[0].as_str()), args[1].as_str())
            } else {
                (None, args[0].as_str())
            };

            let results_limit = limit.unwrap_or(cfg.search.results);
            let use_rerank = if rerank {
                true
            } else if no_rerank {
                false
            } else {
                cfg.search.rerank
            };
            let candidates = rerank_candidates.unwrap_or(cfg.search.rerank_candidates);

            commands::search::run(package, query, results_limit, use_rerank, candidates, &cfg)
        }
        Commands::Show { package, entry } => commands::show::run(&package, &entry),
        Commands::List => commands::list::run(),
        Commands::Build {
            path,
            name,
            version,
            output,
        } => commands::build::run(&path, &name, &version, output.as_deref()),
        Commands::Info { package } => commands::info::run(&package),
        Commands::Remove { package, version } => {
            commands::remove::run(&package, version.as_deref())
        }
        Commands::Init { yes } => commands::init::run(yes),
        Commands::Sync => commands::sync::run(),
    }
}
