mod commands;
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Pull { package } => commands::pull::run(&package),
        Commands::Search { args } => {
            let (package, query) = if args.len() == 2 {
                (Some(args[0].as_str()), args[1].as_str())
            } else {
                (None, args[0].as_str())
            };
            commands::search::run(package, query)
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
    }
}
