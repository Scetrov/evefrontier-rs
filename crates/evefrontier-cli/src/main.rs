use std::fmt;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use evefrontier_lib::{
    ensure_dataset, load_starmap, plan_route, DatasetRelease, RouteAlgorithm, RouteConstraints,
    RouteOutputKind, RouteRenderMode, RouteRequest, RouteSummary, Starmap,
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
    Route(RouteCommandArgs),
    /// Inspect a candidate route with additional diagnostic metadata.
    Search(RouteCommandArgs),
    /// Output the raw path between two systems for downstream tooling.
    Path(RouteCommandArgs),
}

#[derive(Args, Debug, Clone)]
struct RouteCommandArgs {
    #[command(flatten)]
    endpoints: RouteEndpoints,
    #[command(flatten)]
    options: RouteOptionsArgs,
}

impl RouteCommandArgs {
    fn from(&self) -> &str {
        &self.endpoints.from
    }

    fn to(&self) -> &str {
        &self.endpoints.to
    }

    fn to_request(&self) -> RouteRequest {
        RouteRequest {
            start: self.endpoints.from.clone(),
            goal: self.endpoints.to.clone(),
            algorithm: self.options.algorithm.into(),
            constraints: RouteConstraints {
                max_jump: self.options.max_jump,
                avoid_systems: self.options.avoid.clone(),
                avoid_gates: self.options.avoid_gates,
                max_temperature: self.options.max_temp,
            },
        }
    }
}

#[derive(Args, Debug, Clone)]
struct RouteEndpoints {
    /// Starting system name.
    #[arg(long = "from")]
    from: String,
    /// Destination system name.
    #[arg(long = "to")]
    to: String,
}

#[derive(Args, Debug, Clone)]
struct RouteOptionsArgs {
    /// Algorithm to use when planning the route.
    #[arg(long, value_enum, default_value_t = RouteAlgorithmArg::default())]
    algorithm: RouteAlgorithmArg,

    /// Maximum jump distance (light-years) when computing the route.
    #[arg(long = "max-jump")]
    max_jump: Option<f64>,

    /// Systems to avoid when building the path. Repeat for multiple systems.
    #[arg(long = "avoid")]
    avoid: Vec<String>,

    /// Avoid gates entirely (prefer spatial or traversal routes).
    #[arg(long = "avoid-gates", action = ArgAction::SetTrue)]
    avoid_gates: bool,

    /// Maximum system temperature threshold in Kelvin.
    #[arg(long = "max-temp")]
    max_temp: Option<f64>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum, Default)]
enum RouteAlgorithmArg {
    #[default]
    Bfs,
    Dijkstra,
    #[value(name = "a-star")]
    AStar,
}

impl From<RouteAlgorithmArg> for RouteAlgorithm {
    fn from(value: RouteAlgorithmArg) -> Self {
        match value {
            RouteAlgorithmArg::Bfs => RouteAlgorithm::Bfs,
            RouteAlgorithmArg::Dijkstra => RouteAlgorithm::Dijkstra,
            RouteAlgorithmArg::AStar => RouteAlgorithm::AStar,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum, Default)]
enum OutputFormat {
    #[default]
    Text,
    Rich,
    Json,
    Note,
}

impl OutputFormat {
    fn supports_banner(self) -> bool {
        matches!(self, OutputFormat::Text | OutputFormat::Rich)
    }

    fn render_download(self, output: &DownloadOutput) -> Result<()> {
        match self {
            OutputFormat::Text | OutputFormat::Rich | OutputFormat::Note => {
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

    fn render_route_result(self, summary: &RouteSummary) -> Result<()> {
        match self {
            OutputFormat::Text => {
                print!("{}", summary.render(RouteRenderMode::PlainText));
            }
            OutputFormat::Rich => {
                print!("{}", summary.render(RouteRenderMode::RichText));
            }
            OutputFormat::Json => {
                let mut stdout = io::stdout();
                serde_json::to_writer_pretty(&mut stdout, summary)?;
                stdout.write_all(b"\n")?;
            }
            OutputFormat::Note => {
                print!("{}", summary.render(RouteRenderMode::InGameNote));
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
        self.output_format().supports_banner() && !self.options.no_logo
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
        Command::Route(route_args) => {
            handle_route_command(&context, &route_args, RouteOutputKind::Route)
        }
        Command::Search(route_args) => {
            handle_route_command(&context, &route_args, RouteOutputKind::Search)
        }
        Command::Path(route_args) => {
            handle_route_command(&context, &route_args, RouteOutputKind::Path)
        }
    }
}

fn handle_download(context: &AppContext) -> Result<()> {
    let release = context.dataset_release();
    let dataset_path = ensure_dataset(context.target_path(), release.clone())
        .context("failed to locate or download the EveFrontier dataset")?;
    let output = DownloadOutput::new(&dataset_path, &release);
    context.output_format().render_download(&output)
}

fn handle_route_command(
    context: &AppContext,
    args: &RouteCommandArgs,
    kind: RouteOutputKind,
) -> Result<()> {
    let starmap = load_starmap_from_context(context)?;
    let request = args.to_request();
    let plan = plan_route(&starmap, &request).with_context(|| {
        format!(
            "failed to compute route between {} and {}",
            args.from(),
            args.to()
        )
    })?;

    let summary = RouteSummary::from_plan(kind, &starmap, &plan)
        .context("failed to build route summary for display")?;
    context.output_format().render_route_result(&summary)
}

fn load_starmap_from_context(context: &AppContext) -> Result<Starmap> {
    let dataset_path = ensure_dataset(context.target_path(), context.dataset_release())
        .context("failed to locate or download the EveFrontier dataset")?;
    let starmap = load_starmap(&dataset_path)
        .with_context(|| format!("failed to load dataset from {}", dataset_path.display()))?;
    Ok(starmap)
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
