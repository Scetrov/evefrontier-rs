use std::fmt;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use evefrontier_lib::{
    build_graph, ensure_dataset, find_route, load_starmap, DatasetRelease, Error as LibError,
    Starmap, SystemId,
};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "EveFrontier dataset utilities",
    long_about = None,
    propagate_version = true,
    arg_required_else_help = true
)]
struct Cli {
    #[command(flatten)]
    global: GlobalOptions,

    #[command(subcommand)]
    command: Command,
}

#[derive(Args, Debug, Clone)]
struct GlobalOptions {
    /// Override the dataset directory or file path.
    #[arg(long)]
    data_dir: Option<PathBuf>,

    /// Dataset release tag to download (defaults to the latest release when omitted).
    #[arg(long)]
    dataset: Option<String>,

    /// Select the output format for CLI responses.
    #[arg(long, value_enum, default_value_t = OutputFormat::default())]
    format: OutputFormat,

    /// Suppress the EveFrontier CLI logo banner.
    #[arg(long, action = ArgAction::SetTrue)]
    no_logo: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Ensure the dataset is downloaded and report its location.
    Download,
    /// Compute a route between two system names using the loaded dataset.
    Route(RouteArgs),
}

#[derive(Args, Debug, Clone)]
struct RouteArgs {
    /// Starting system name.
    #[arg(long = "from")]
    from: String,
    /// Destination system name.
    #[arg(long = "to")]
    to: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum, Default)]
enum OutputFormat {
    #[default]
    Text,
    Json,
}

impl OutputFormat {
    fn is_text(self) -> bool {
        matches!(self, OutputFormat::Text)
    }

    fn render_download(self, output: &DownloadOutput) -> Result<()> {
        match self {
            OutputFormat::Text => {
                println!(
                    "Dataset available at {} (requested release: {})",
                    output.dataset_path, output.release
                );
            }
            OutputFormat::Json => {
                let mut stdout = io::stdout();
                serde_json::to_writer_pretty(&mut stdout, output)?;
                stdout.write_all(b"\n")?;
            }
        }
        Ok(())
    }

    fn render_route(self, summary: &RouteSummary) -> Result<()> {
        match self {
            OutputFormat::Text => render_route_text(summary),
            OutputFormat::Json => {
                let mut stdout = io::stdout();
                serde_json::to_writer_pretty(&mut stdout, summary)?;
                stdout.write_all(b"\n")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
struct DownloadOutput {
    dataset_path: String,
    release: ReleaseRequest,
}

impl DownloadOutput {
    fn new(dataset_path: &Path, release: &DatasetRelease) -> Self {
        Self {
            dataset_path: dataset_path.display().to_string(),
            release: release.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ReleaseRequest {
    Latest,
    Tag { value: String },
}

impl From<&DatasetRelease> for ReleaseRequest {
    fn from(value: &DatasetRelease) -> Self {
        match value {
            DatasetRelease::Latest => ReleaseRequest::Latest,
            DatasetRelease::Tag(tag) => ReleaseRequest::Tag { value: tag.clone() },
        }
    }
}

impl fmt::Display for ReleaseRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReleaseRequest::Latest => write!(f, "latest"),
            ReleaseRequest::Tag { value } => write!(f, "tag {}", value),
        }
    }
}

#[derive(Debug, Clone)]
struct AppContext {
    options: GlobalOptions,
}

impl AppContext {
    fn new(options: GlobalOptions) -> Self {
        Self { options }
    }

    fn dataset_release(&self) -> DatasetRelease {
        self.options
            .dataset
            .as_deref()
            .map(DatasetRelease::tag)
            .unwrap_or_else(DatasetRelease::latest)
    }

    fn target_path(&self) -> Option<&Path> {
        self.options.data_dir.as_deref()
    }

    fn output_format(&self) -> OutputFormat {
        self.options.format
    }

    fn should_show_logo(&self) -> bool {
        self.output_format().is_text() && !self.options.no_logo
    }
}

#[derive(Debug, Clone, Serialize)]
struct RouteSummary {
    start: RouteEndpoint,
    goal: RouteEndpoint,
    steps: Vec<RouteStep>,
}

impl RouteSummary {
    fn hop_count(&self) -> usize {
        self.steps.len().saturating_sub(1)
    }
}

#[derive(Debug, Clone, Serialize)]
struct RouteEndpoint {
    id: SystemId,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

impl RouteEndpoint {
    fn from_step(step: &RouteStep) -> Self {
        Self {
            id: step.id,
            name: step.name.clone(),
        }
    }

    fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or("<unknown>")
    }
}

#[derive(Debug, Clone, Serialize)]
struct RouteStep {
    index: usize,
    id: SystemId,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

impl RouteStep {
    fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or("<unknown>")
    }
}

fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();
    let context = AppContext::new(cli.global);

    if context.should_show_logo() {
        print_logo();
    }

    match cli.command {
        Command::Download => handle_download(&context),
        Command::Route(route_args) => handle_route(&context, &route_args),
    }
}

fn handle_download(context: &AppContext) -> Result<()> {
    let release = context.dataset_release();
    let dataset_path = ensure_dataset(context.target_path(), release.clone())
        .context("failed to locate or download the EveFrontier dataset")?;
    let output = DownloadOutput::new(&dataset_path, &release);
    context.output_format().render_download(&output)
}

fn handle_route(context: &AppContext, args: &RouteArgs) -> Result<()> {
    let release = context.dataset_release();
    let dataset_path = ensure_dataset(context.target_path(), release)
        .context("failed to locate or download the EveFrontier dataset")?;
    let starmap = load_starmap(&dataset_path)
        .with_context(|| format!("failed to load dataset from {}", dataset_path.display()))?;
    let start_id =
        starmap
            .system_id_by_name(&args.from)
            .ok_or_else(|| LibError::UnknownSystem {
                name: args.from.clone(),
            })?;
    let goal_id = starmap
        .system_id_by_name(&args.to)
        .ok_or_else(|| LibError::UnknownSystem {
            name: args.to.clone(),
        })?;

    let graph = build_graph(&starmap);
    let route = find_route(&graph, start_id, goal_id).ok_or_else(|| LibError::RouteNotFound {
        start: args.from.clone(),
        goal: args.to.clone(),
    })?;

    let summary = build_route_summary(&starmap, &route)
        .context("failed to build route summary for display")?;
    context.output_format().render_route(&summary)
}

fn build_route_summary(starmap: &Starmap, route: &[SystemId]) -> Result<RouteSummary> {
    if route.is_empty() {
        return Err(anyhow!("route contained no systems"));
    }

    let steps = route
        .iter()
        .enumerate()
        .map(|(index, system_id)| RouteStep {
            index,
            id: *system_id,
            name: starmap.system_name(*system_id).map(|name| name.to_string()),
        })
        .collect::<Vec<_>>();

    let start = RouteEndpoint::from_step(steps.first().expect("validated non-empty route"));
    let goal = RouteEndpoint::from_step(steps.last().expect("validated non-empty route"));

    Ok(RouteSummary { start, goal, steps })
}

fn render_route_text(summary: &RouteSummary) {
    println!(
        "Route: {} -> {} ({} hops)",
        summary.start.display_name(),
        summary.goal.display_name(),
        summary.hop_count()
    );

    for step in &summary.steps {
        println!("{:>3}: {} ({})", step.index, step.display_name(), step.id);
    }
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(env_filter)
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);
}

fn print_logo() {
    const ORANGE: &str = "\x1b[38;5;208m";
    const RESET: &str = "\x1b[0m";
    const TITLE: &str = "EveFrontier CLI";
    const WIDTH: usize = 30;

    let horizontal = "─".repeat(WIDTH);
    let centered = format!("{:^width$}", TITLE, width = WIDTH);

    println!(
        "{color}╭{line}╮\n│{text}│\n╰{line}╯{reset}",
        color = ORANGE,
        line = horizontal.as_str(),
        text = centered.as_str(),
        reset = RESET
    );
}
