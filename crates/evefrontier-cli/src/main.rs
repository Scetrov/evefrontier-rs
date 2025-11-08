use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use evefrontier_lib::{
    build_graph, ensure_c3e6_dataset, find_route, load_starmap, Error as LibError,
};

#[derive(Parser, Debug)]
#[command(author, version, about = "EveFrontier dataset utilities")]
struct Cli {
    /// Override the dataset directory or file path.
    #[arg(long)]
    data_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Ensure the dataset is downloaded and report its location.
    Download,
    /// Compute a route between two system names using the loaded dataset.
    Route {
        /// Starting system name.
        #[arg(long = "from")]
        from: String,
        /// Destination system name.
        #[arg(long = "to")]
        to: String,
    },
}

fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();

    match cli.command {
        Command::Download => handle_download(cli.data_dir.as_deref()),
        Command::Route { from, to } => handle_route(cli.data_dir.as_deref(), &from, &to),
    }
}

fn handle_download(target: Option<&Path>) -> Result<()> {
    let dataset_path = ensure_c3e6_dataset(target)
        .context("failed to locate or download the EveFrontier dataset")?;
    println!("Dataset available at {}", dataset_path.display());
    Ok(())
}

fn handle_route(target: Option<&Path>, from: &str, to: &str) -> Result<()> {
    let dataset_path = ensure_c3e6_dataset(target)
        .context("failed to locate or download the EveFrontier dataset")?;
    let starmap = load_starmap(&dataset_path)
        .with_context(|| format!("failed to load dataset from {}", dataset_path.display()))?;
    let start_id = starmap
        .system_id_by_name(from)
        .ok_or_else(|| LibError::UnknownSystem {
            name: from.to_string(),
        })?;
    let goal_id = starmap
        .system_id_by_name(to)
        .ok_or_else(|| LibError::UnknownSystem {
            name: to.to_string(),
        })?;

    let graph = build_graph(&starmap);
    let route = find_route(&graph, start_id, goal_id).ok_or_else(|| LibError::RouteNotFound {
        start: from.to_string(),
        goal: to.to_string(),
    })?;

    println!("Route:");
    for system_id in route {
        let name = starmap.system_name(system_id).unwrap_or("<unknown>");
        println!("- {} ({})", name, system_id);
    }

    Ok(())
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(env_filter)
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);
}
