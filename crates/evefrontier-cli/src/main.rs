use std::fmt;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use evefrontier_lib::{
    ensure_dataset, load_starmap, plan_route, spatial_index_path, try_load_spatial_index,
    DatasetRelease, Error as RouteError, RouteAlgorithm, RouteConstraints, RouteOutputKind,
    RouteRenderMode, RouteRequest, RouteSummary, SpatialIndex,
};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "EVE Frontier dataset utilities",
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
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,

    /// Dataset release tag to download (defaults to the latest release when omitted).
    #[arg(long, global = true)]
    dataset: Option<String>,

    /// Select the output format for CLI responses.
    #[arg(long, value_enum, default_value_t = OutputFormat::default(), global = true)]
    format: OutputFormat,

    /// Suppress the EVE Frontier CLI logo banner.
    #[arg(long, action = ArgAction::SetTrue, global = true)]
    no_logo: bool,

    /// Suppress the footer with timing information.
    #[arg(long, action = ArgAction::SetTrue, global = true)]
    no_footer: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Ensure the dataset is downloaded and report its location.
    Download,
    /// Compute a route between two system names using the loaded dataset.
    Route(RouteCommandArgs),
    /// Build or rebuild the spatial index for faster routing.
    IndexBuild(IndexBuildArgs),
}

#[derive(Args, Debug, Clone)]
struct IndexBuildArgs {
    /// Force rebuild even if index already exists.
    #[arg(long, action = ArgAction::SetTrue)]
    force: bool,
}

#[derive(Args, Debug, Clone)]
struct RouteCommandArgs {
    #[command(flatten)]
    endpoints: RouteEndpoints,
    #[command(flatten)]
    options: RouteOptionsArgs,
}

impl RouteCommandArgs {
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
            spatial_index: None, // Will be set separately after loading
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
    ///
    /// Only applies to spatial jumps - systems with star temperature above this
    /// threshold cannot be reached via spatial jumps (ships would overheat).
    /// Gate jumps are unaffected by temperature.
    #[arg(long = "max-temp")]
    max_temp: Option<f64>,

    /// Suppress minimum external temperature annotations in route output.
    #[arg(long = "no-temp", action = ArgAction::SetTrue)]
    no_temp: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum, Default)]
enum RouteAlgorithmArg {
    Bfs,
    Dijkstra,
    #[default]
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

// Views removed; CLI always uses RouteOutputKind::Route.

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum, Default)]
enum OutputFormat {
    #[default]
    Text,
    Rich,
    Json,
    /// Minimal path-only output with +/|/- prefixes.
    Basic,
    /// Emoji-enhanced readable output per EXAMPLES.md.
    Emoji,
    /// Enhanced format with system details (temp, planets, moons).
    Enhanced,
    #[value(alias = "notepad")]
    Note,
}

impl OutputFormat {
    fn supports_banner(self) -> bool {
        matches!(
            self,
            OutputFormat::Text | OutputFormat::Rich | OutputFormat::Emoji | OutputFormat::Enhanced
        )
    }

    fn supports_footer(self) -> bool {
        matches!(
            self,
            OutputFormat::Text
                | OutputFormat::Rich
                | OutputFormat::Emoji
                | OutputFormat::Basic
                | OutputFormat::Enhanced
        )
    }

    fn render_download(self, output: &DownloadOutput) -> Result<()> {
        // Download output is always plain text regardless of selected format.
        println!(
            "Dataset available at {} (requested release: {})",
            output.dataset_path, output.release
        );
        Ok(())
    }

    fn render_route_result(self, summary: &RouteSummary, show_temps: bool) -> Result<()> {
        match self {
            OutputFormat::Text => {
                // Human-friendly route view per docs/EXAMPLES.md
                let hops = summary.hops;
                let start = summary.start.name.as_deref().unwrap_or("<unknown>");
                let goal = summary.goal.name.as_deref().unwrap_or("<unknown>");
                // Include algorithm hint to keep tests informative
                println!(
                    "Route from {} to {} ({} jumps; algorithm: {}):",
                    start, goal, hops, summary.algorithm
                );
                for step in &summary.steps {
                    let name = step.name.as_deref().unwrap_or("<unknown>");
                    if let (Some(distance), Some(method)) = (step.distance, step.method.as_deref())
                    {
                        if show_temps {
                            if let Some(t) = step.min_external_temp {
                                println!(
                                    " - {} [min {:.2}K] ({:.0}ly via {})",
                                    name, t, distance, method
                                );
                            } else {
                                println!(" - {} ({:.0}ly via {})", name, distance, method);
                            }
                        } else {
                            println!(" - {} ({:.0}ly via {})", name, distance, method);
                        }
                    } else if show_temps {
                        if let Some(t) = step.min_external_temp {
                            println!(" - {} [min {:.2}K]", name, t);
                        } else {
                            println!(" - {}", name);
                        }
                    } else {
                        println!(" - {}", name);
                    }
                }
                println!("\nTotal distance: {:.0}ly", summary.total_distance);
                println!("Total ly jumped: {:.0}ly", summary.jump_distance);
            }
            OutputFormat::Rich => {
                print!(
                    "{}",
                    summary.render_with(RouteRenderMode::RichText, show_temps)
                );
            }
            OutputFormat::Json => {
                let mut stdout = io::stdout();
                serde_json::to_writer_pretty(&mut stdout, summary)?;
                stdout.write_all(b"\n")?;
            }
            OutputFormat::Basic => {
                // Render a minimal path: first line with '+', middle lines with '|', last with '-'
                let len = summary.steps.len();
                if len == 0 {
                    return Ok(());
                }
                for (i, step) in summary.steps.iter().enumerate() {
                    let prefix = if i == 0 {
                        '+'
                    } else if i + 1 == len {
                        '-'
                    } else {
                        '|'
                    };
                    let name = step.name.as_deref().unwrap_or("<unknown>");
                    if show_temps {
                        if let Some(t) = step.min_external_temp {
                            println!("{} {} [min {:.2}K]", prefix, name, t);
                        } else {
                            println!("{} {}", prefix, name);
                        }
                    } else {
                        println!("{} {}", prefix, name);
                    }
                }
                println!("via {} gates / {} jump drive", summary.gates, summary.jumps);
            }
            OutputFormat::Emoji => {
                // Header: "Route from A to B (N jumps):"
                let hops = summary.hops;
                let start = summary.start.name.as_deref().unwrap_or("<unknown>");
                let goal = summary.goal.name.as_deref().unwrap_or("<unknown>");
                println!("Route from {} to {} ({} jumps):", start, goal, hops);
                let len = summary.steps.len();
                for (i, step) in summary.steps.iter().enumerate() {
                    let name = step.name.as_deref().unwrap_or("<unknown>");
                    let icon = if i == 0 {
                        "🚥"
                    } else if i + 1 == len {
                        "🚀️"
                    } else {
                        "📍"
                    };
                    if let (Some(distance), Some(method)) = (step.distance, step.method.as_deref())
                    {
                        if show_temps {
                            if let Some(t) = step.min_external_temp {
                                println!(
                                    " {} {} [min {:.2}K] ({:.0}ly via {})",
                                    icon, name, t, distance, method
                                );
                            } else {
                                println!(" {} {} ({:.0}ly via {})", icon, name, distance, method);
                            }
                        } else {
                            println!(" {} {} ({:.0}ly via {})", icon, name, distance, method);
                        }
                    } else if show_temps {
                        if let Some(t) = step.min_external_temp {
                            println!(" {} {} [min {:.2}K]", icon, name, t);
                        } else {
                            println!(" {} {}", icon, name);
                        }
                    } else {
                        println!(" {} {}", icon, name);
                    }
                }
                println!("\nTotal distance: {:.0}ly", summary.total_distance);
                println!("Total ly jumped: {:.0}ly", summary.jump_distance);
            }
            OutputFormat::Note => {
                // Strict notepad format per EXAMPLES.md using Sta/Dst/Jmp lines with showinfo anchors.
                // Sta: first, Dst: second (if present and there are >=3 steps), Jmp: last (if there are >=2 steps)
                let first = summary.steps.first();
                if let Some(step) = first {
                    let name = step.name.as_deref().unwrap_or("<unknown>");
                    println!("Sta <a href=\"showinfo:5//{}\">{}</a>", step.id, name);
                }
                if summary.steps.len() >= 3 {
                    let step = &summary.steps[1];
                    let name = step.name.as_deref().unwrap_or("<unknown>");
                    println!("Dst <a href=\"showinfo:5//{}\">{}</a>", step.id, name);
                }
                if summary.steps.len() >= 2 {
                    let step = summary.steps.last().expect("len>=2 has last");
                    let name = step.name.as_deref().unwrap_or("<unknown>");
                    println!("Jmp <a href=\"showinfo:5//{}\">{}</a>", step.id, name);
                }
            }
            OutputFormat::Enhanced => {
                // Enhanced format with inverted tag labels and system details
                // Color definitions for enhanced mode
                let supports_color = std::env::var_os("NO_COLOR").is_none()
                    && std::env::var("TERM")
                        .map(|t| !t.eq_ignore_ascii_case("dumb"))
                        .unwrap_or(true);

                // Tag colors use reverse video (inverted) + bold for visibility
                // Format: \x1b[1;7m for bold+reverse, then foreground color
                let (
                    tag_strt,
                    tag_gate,
                    tag_jump,
                    tag_goal,
                    white_bold,
                    gray,
                    cyan,
                    green,
                    blue,
                    orange,
                    red,
                    reset,
                ) = if supports_color {
                    (
                        "\x1b[1;7;32m",   // bold reverse green background for STRT
                        "\x1b[1;7;36m",   // bold reverse cyan background for GATE
                        "\x1b[1;7;33m",   // bold reverse yellow background for JUMP
                        "\x1b[1;7;35m",   // bold reverse magenta background for GOAL
                        "\x1b[1;97m",     // bright bold white for system names
                        "\x1b[90m",       // gray for tree lines
                        "\x1b[36m",       // cyan for temp
                        "\x1b[32m",       // green for planets
                        "\x1b[34m",       // blue for moons
                        "\x1b[38;5;208m", // orange for warm systems (>20K)
                        "\x1b[31m",       // red for hot systems (>50K)
                        "\x1b[0m",        // reset
                    )
                } else {
                    ("", "", "", "", "", "", "", "", "", "", "", "")
                };

                // Helper to format numbers with thousand separators
                fn format_with_separators(n: u64) -> String {
                    if n < 1000 {
                        return n.to_string();
                    }
                    let s = n.to_string();
                    let mut result = String::new();
                    for (i, c) in s.chars().rev().enumerate() {
                        if i > 0 && i % 3 == 0 {
                            result.push(',');
                        }
                        result.push(c);
                    }
                    result.chars().rev().collect()
                }

                let hops = summary.hops;
                let start = summary.start.name.as_deref().unwrap_or("<unknown>");
                let goal = summary.goal.name.as_deref().unwrap_or("<unknown>");
                println!(
                    "Route from {}{}{} to {}{}{} ({} jumps):",
                    white_bold, start, reset, white_bold, goal, reset, hops
                );

                let len = summary.steps.len();
                for (i, step) in summary.steps.iter().enumerate() {
                    let name = step.name.as_deref().unwrap_or("<unknown>");
                    let is_last = i + 1 == len;

                    // Determine the tag based on position and method
                    // Tags have spaces on both sides for padding within the colored background
                    let (tag_color, tag_text) = if i == 0 {
                        (tag_strt, " STRT ")
                    } else if is_last {
                        (tag_goal, " GOAL ")
                    } else {
                        match step.method.as_deref() {
                            Some("gate") => (tag_gate, " GATE "),
                            Some("jump") => (tag_jump, " JUMP "),
                            _ => (tag_jump, " JUMP "),
                        }
                    };

                    // Determine jump type label for the brackets
                    let jump_type = match step.method.as_deref() {
                        Some("gate") => "gate",
                        Some("jump") => "jump",
                        _ => "",
                    };

                    // Determine circle color based on temperature
                    // >50K = red, >20K = orange, else default (no color)
                    let temp = step.min_external_temp.unwrap_or(0.0);
                    let circle = if temp > 50.0 {
                        format!("{}●{}", red, reset)
                    } else if temp > 20.0 {
                        format!("{}●{}", orange, reset)
                    } else {
                        "●".to_string()
                    };

                    // Print the tag and system name with optional distance and jump type
                    if let Some(distance) = step.distance {
                        let dist_str = format_with_separators(distance as u64);
                        if !jump_type.is_empty() {
                            println!(
                                "{}{}{} {} {}{}{} ({}, {}ly)",
                                tag_color,
                                tag_text,
                                reset,
                                circle,
                                white_bold,
                                name,
                                reset,
                                jump_type,
                                dist_str
                            );
                        } else {
                            println!(
                                "{}{}{} {} {}{}{} ({}ly)",
                                tag_color,
                                tag_text,
                                reset,
                                circle,
                                white_bold,
                                name,
                                reset,
                                dist_str
                            );
                        }
                    } else {
                        println!(
                            "{}{}{} {} {}{}{}",
                            tag_color, tag_text, reset, circle, white_bold, name, reset
                        );
                    }

                    // Print details line if not the last step
                    if !is_last {
                        // Build stat parts, omitting zeros
                        let mut parts: Vec<String> = Vec::new();

                        // Temperature (always show if available) - right-aligned to 6 chars for consistency
                        if let Some(t) = step.min_external_temp {
                            parts.push(format!("{}min {:>6.2}K{}", cyan, t, reset));
                        }

                        // Planets (omit if zero) - right-aligned count
                        let planets = step.planet_count.unwrap_or(0);
                        if planets > 0 {
                            let label = if planets == 1 { "Planet" } else { "Planets" };
                            parts.push(format!("{}{:>2} {}{}", green, planets, label, reset));
                        }

                        // Moons (omit if zero) - right-aligned count
                        let moons = step.moon_count.unwrap_or(0);
                        if moons > 0 {
                            let label = if moons == 1 { "Moon" } else { "Moons" };
                            parts.push(format!("{}{:>2} {}{}", blue, moons, label, reset));
                        }

                        if !parts.is_empty() {
                            println!(
                                "       {gray}│{reset} {details}",
                                gray = gray,
                                reset = reset,
                                details = parts.join(&format!("{}, {}", gray, reset))
                            );
                        }
                    }
                }

                // Footer with route summary statistics
                let gate_distance = summary.total_distance - summary.jump_distance;
                let total_str = format_with_separators(summary.total_distance as u64);
                let gates_str = format_with_separators(gate_distance as u64);
                let jumps_str = format_with_separators(summary.jump_distance as u64);

                // Find max width for right-alignment (add 2 for "ly" suffix)
                let max_width = total_str.len().max(gates_str.len()).max(jumps_str.len());

                println!();
                println!(
                    "{gray}───────────────────────────────────────{reset}",
                    gray = gray,
                    reset = reset
                );
                println!(
                    "  {cyan}Total Distance:{reset}  {white}{:>width$}ly{reset}",
                    total_str,
                    cyan = cyan,
                    white = white_bold,
                    reset = reset,
                    width = max_width
                );
                println!(
                    "  {green}Via Gates:{reset}       {white}{:>width$}ly{reset}",
                    gates_str,
                    green = green,
                    white = white_bold,
                    reset = reset,
                    width = max_width
                );
                println!(
                    "  {orange}Via Jumps:{reset}       {white}{:>width$}ly{reset}",
                    jumps_str,
                    orange = orange,
                    white = white_bold,
                    reset = reset,
                    width = max_width
                );
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

    fn should_show_footer(&self) -> bool {
        self.output_format().supports_footer() && !self.options.no_footer
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let context = AppContext::new(cli.global);

    // For JSON output, suppress tracing to keep stdout clean
    if context.output_format() != OutputFormat::Json {
        init_tracing();
    }

    let start = std::time::Instant::now();

    if context.should_show_logo() {
        print_logo();
    }

    let result = match cli.command {
        Command::Download => handle_download(&context),
        Command::Route(route_args) => {
            handle_route_command(&context, &route_args, RouteOutputKind::Route)
        }
        Command::IndexBuild(args) => handle_index_build(&context, &args),
    };

    if result.is_ok() && context.should_show_footer() {
        let elapsed = start.elapsed();
        print_footer(elapsed);
    }

    result
}

fn handle_download(context: &AppContext) -> Result<()> {
    let release = context.dataset_release();
    let paths = ensure_dataset(context.target_path(), release.clone())
        .context("failed to locate or download the EVE Frontier dataset")?;
    let output = DownloadOutput::new(&paths.database, &release);
    context.output_format().render_download(&output)
}

fn handle_index_build(context: &AppContext, args: &IndexBuildArgs) -> Result<()> {
    let paths = ensure_dataset(context.target_path(), context.dataset_release())
        .context("failed to locate or download the EVE Frontier dataset")?;

    let index_path = spatial_index_path(&paths.database);

    // Check if index already exists
    if index_path.exists() && !args.force {
        println!(
            "Spatial index already exists at {}\nUse --force to rebuild.",
            index_path.display()
        );
        return Ok(());
    }

    println!("Loading starmap from {}...", paths.database.display());
    let starmap = load_starmap(&paths.database)
        .with_context(|| format!("failed to load dataset from {}", paths.database.display()))?;

    println!(
        "Building spatial index for {} systems...",
        starmap.systems.len()
    );
    let index = SpatialIndex::build(&starmap);

    let systems_with_temp = starmap
        .systems
        .values()
        .filter(|s| s.metadata.min_external_temp.is_some())
        .count();

    println!("Saving index to {}...", index_path.display());
    index
        .save(&index_path)
        .context("failed to save spatial index")?;

    let file_size = std::fs::metadata(&index_path).map(|m| m.len()).unwrap_or(0);

    println!("Spatial index built successfully:");
    println!("  Path: {}", index_path.display());
    println!("  Systems indexed: {}", index.len());
    println!("  Systems with temperature: {}", systems_with_temp);
    println!("  File size: {} bytes", file_size);

    Ok(())
}

fn handle_route_command(
    context: &AppContext,
    args: &RouteCommandArgs,
    kind: RouteOutputKind,
) -> Result<()> {
    let paths = ensure_dataset(context.target_path(), context.dataset_release())
        .context("failed to locate or download the EVE Frontier dataset")?;
    let starmap = load_starmap(&paths.database)
        .with_context(|| format!("failed to load dataset from {}", paths.database.display()))?;

    // Try to load a pre-built spatial index to speed up routing
    let spatial_index = try_load_spatial_index(&paths.database).map(Arc::new);

    let mut request = args.to_request();
    if let Some(index) = spatial_index {
        request = request.with_spatial_index(index);
    }

    let plan = match plan_route(&starmap, &request) {
        Ok(plan) => plan,
        Err(err) => return Err(handle_route_failure(&request, err)),
    };

    let summary = RouteSummary::from_plan(kind, &starmap, &plan)
        .context("failed to build route summary for display")?;
    let show_temps = !args.options.no_temp;
    context
        .output_format()
        .render_route_result(&summary, show_temps)
}

fn handle_route_failure(request: &RouteRequest, err: RouteError) -> anyhow::Error {
    match err {
        RouteError::UnknownSystem { name, suggestions } => {
            anyhow::anyhow!(format_unknown_system_message(&name, &suggestions))
        }
        RouteError::RouteNotFound { start, goal } => {
            anyhow::anyhow!(format_route_not_found_message(
                &start,
                &goal,
                &request.constraints
            ))
        }
        other => anyhow::Error::new(other),
    }
}

fn format_unknown_system_message(name: &str, suggestions: &[String]) -> String {
    let mut message = format!("Unknown system '{}'.", name);
    if !suggestions.is_empty() {
        let formatted = if suggestions.len() == 1 {
            let suggestion = suggestions.first().expect("len checked above");
            format!("Did you mean '{suggestion}'?")
        } else {
            let joined = suggestions
                .iter()
                .map(|s| format!("'{}'", s))
                .collect::<Vec<_>>()
                .join(", ");
            format!("Did you mean one of: {}?", joined)
        };
        message.push(' ');
        message.push_str(&formatted);
    }
    message
}

fn format_route_not_found_message(
    start: &str,
    goal: &str,
    constraints: &RouteConstraints,
) -> String {
    let mut message = format!("No route found between {} and {}.", start, goal);
    let mut tips = Vec::new();
    if constraints.max_jump.is_some() {
        tips.push("increase --max-jump");
    }
    if constraints.avoid_gates {
        tips.push("allow gates (omit --avoid-gates)");
    }
    if constraints.max_temperature.is_some() {
        tips.push("raise --max-temp");
    }
    if tips.is_empty() {
        message.push_str(
            " Try a different algorithm (for example, --algorithm dijkstra) or relax constraints.",
        );
    } else {
        message.push(' ');
        message.push_str(&format!("Try {}.", tips.join(", ")));
    }
    message
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);
}

fn print_logo() {
    const ORANGE_RAW: &str = "\x1b[38;5;208m";
    const RESET_RAW: &str = "\x1b[0m";
    // Respect environment conventions to avoid emitting ANSI escapes in
    // non-capable environments. Honor the NO_COLOR env var and `TERM=dumb`.
    fn supports_color() -> bool {
        if std::env::var_os("NO_COLOR").is_some() {
            return false;
        }
        if let Ok(term) = std::env::var("TERM") {
            if term.eq_ignore_ascii_case("dumb") {
                return false;
            }
        }
        true
    }

    // Detect Unicode support by checking common environment hints. Falls back to ASCII
    // box-drawing characters for maximum terminal compatibility.
    fn supports_unicode() -> bool {
        // Check for explicit Unicode support hints
        if let Ok(lang) = std::env::var("LANG") {
            if lang.to_uppercase().contains("UTF") {
                return true;
            }
        }
        if let Ok(lc_all) = std::env::var("LC_ALL") {
            if lc_all.to_uppercase().contains("UTF") {
                return true;
            }
        }
        // On Windows, assume Unicode support unless TERM suggests otherwise
        #[cfg(windows)]
        {
            if let Ok(term) = std::env::var("TERM") {
                // Some legacy Windows terminals don't support Unicode
                return !term.eq_ignore_ascii_case("dumb");
            }
            return true;
        }
        // On Unix-like systems, default to false unless explicitly set
        #[cfg(not(windows))]
        {
            false
        }
    }

    let (orange, cyan, reset) = if supports_color() {
        (ORANGE_RAW, "\x1b[36m", RESET_RAW)
    } else {
        ("", "", "")
    };
    let use_unicode = supports_unicode();

    if use_unicode {
        // Sci-fi glitch/neon style banner with cyan border and orange text
        // Inner width = 50 chars (48 dashes in borders)
        // Using rounded corners (╭╮╰╯) for a softer look
        println!(
            "{cyan}╭────────────────────────────────────────────────╮{reset}
{cyan}│{orange} ░█▀▀░█░█░█▀▀░░░█▀▀░█▀▄░█▀█░█▀█░▀█▀░▀█▀░█▀▀░█▀▄ {cyan}│{reset}
{cyan}│{orange} ░█▀▀░▀▄▀░█▀▀░░░█▀▀░█▀▄░█░█░█░█░░█░░░█░░█▀▀░█▀▄ {cyan}│{reset}
{cyan}│{orange} ░▀▀▀░░▀░░▀▀▀░░░▀░░░▀░▀░▀▀▀░▀░▀░░▀░░▀▀▀░▀▀▀░▀░▀ {cyan}│{reset}
{cyan}├────────────────────────────────────────────────┤{reset}
{cyan}│{orange}                    [ C L I ]                   {cyan}│{reset}
{cyan}╰────────────────────────────────────────────────╯{reset}",
            cyan = cyan,
            orange = orange,
            reset = reset
        );
    } else {
        // Fallback ASCII banner
        println!(
            "{color}+--------------------------------------------------+
|  EVE FRONTIER                                    |
|  >> PATHFINDER COMMAND LINE INTERFACE            |
+--------------------------------------------------+{reset}",
            color = orange,
            reset = reset
        );
    }
}

fn print_footer(elapsed: std::time::Duration) {
    const GRAY_RAW: &str = "\x1b[90m";
    const RESET_RAW: &str = "\x1b[0m";

    fn supports_color() -> bool {
        if std::env::var_os("NO_COLOR").is_some() {
            return false;
        }
        if let Ok(term) = std::env::var("TERM") {
            if term.eq_ignore_ascii_case("dumb") {
                return false;
            }
        }
        true
    }

    let (gray, reset) = if supports_color() {
        (GRAY_RAW, RESET_RAW)
    } else {
        ("", "")
    };

    let elapsed_ms = elapsed.as_millis();
    let time_str = if elapsed_ms < 1000 {
        format!("{}ms", elapsed_ms)
    } else {
        format!("{:.2}s", elapsed.as_secs_f64())
    };

    println!("\n{gray}Completed in {}{reset}", time_str);
}
