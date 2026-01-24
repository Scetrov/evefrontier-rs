use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod commands;
mod output;
mod output_helpers;
mod terminal;
#[cfg(test)]
mod test_helpers;

use evefrontier_lib::{
    compute_dataset_checksum, decode_fmap_token, encode_fmap_token, ensure_dataset, load_starmap,
    plan_route, read_release_tag, spatial_index_path, try_load_spatial_index, verify_freshness,
    DatasetMetadata, DatasetRelease, Error as RouteError, FreshnessResult, RouteAlgorithm,
    RouteConstraints, RouteOutputKind, RouteRequest, RouteSummary, ShipCatalog, ShipLoadout,
    SpatialIndex, VerifyDiagnostics, VerifyOutput, Waypoint, WaypointType,
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

    /// Disable ANSI colors in CLI output (overrides NO_COLOR env var when set).
    #[arg(long = "no-color", action = ArgAction::SetTrue, global = true)]
    no_color: bool,

    /// Suppress the footer with timing information.
    #[arg(long, action = ArgAction::SetTrue, global = true)]
    no_footer: bool,

    /// Override the fmap base URL used in rendered route outputs (default: https://fmap.scetrov.live).
    #[arg(long, global = true, value_name = "URL")]
    fmap_base_url: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct McpCommandArgs {
    /// Override log level (trace, debug, info, warn, error). Defaults to RUST_LOG env var or 'info'.
    #[arg(long, value_parser = ["trace", "debug", "info", "warn", "error"])]
    pub log_level: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct ScoutCommandArgs {
    #[command(subcommand)]
    pub subcommand: ScoutSubcommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ScoutSubcommand {
    /// List gate-connected neighbors of a system.
    Gates(ScoutGatesArgs),
    /// Find systems within spatial range of a system.
    Range(ScoutRangeArgs),
}

#[derive(Args, Debug, Clone)]
pub struct ScoutGatesArgs {
    /// System name to query (case-insensitive, fuzzy matched).
    pub system: String,

    /// Include CCP developer/staging systems (AD###, V-###) in results.
    #[arg(long, action = ArgAction::SetTrue)]
    pub include_ccp_systems: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ScoutRangeArgs {
    /// System name to query (case-insensitive, fuzzy matched).
    pub system: String,

    /// Maximum number of results to return (1-100).
    #[arg(long, short = 'n', default_value = "10")]
    pub limit: usize,

    /// Maximum distance in light-years from the origin system.
    #[arg(long, short = 'r')]
    pub radius: Option<f64>,

    /// Maximum star temperature in Kelvin (filters out hotter systems).
    #[arg(long = "max-temp", short = 't')]
    pub max_temp: Option<f64>,

    /// Include CCP developer/staging systems (AD###, V-###) in results.
    #[arg(long, action = ArgAction::SetTrue)]
    pub include_ccp_systems: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Ensure the dataset is downloaded and report its location.
    Download,
    /// Compute a route between two system names using the loaded dataset.
    Route(RouteCommandArgs),
    /// Build or rebuild the spatial index for faster routing.
    IndexBuild(IndexBuildArgs),
    /// Verify that the spatial index is fresh (matches the current dataset).
    IndexVerify(IndexVerifyArgs),
    /// List available ships from ship_data.csv.
    Ships,
    /// Encode a route to an fmap URL token.
    FmapEncode(FmapEncodeArgs),
    /// Decode an fmap URL token back to a route.
    FmapDecode(FmapDecodeArgs),
    /// Launch the Model Context Protocol (MCP) server via stdio transport.
    Mcp(McpCommandArgs),
    /// Scout nearby systems (gates or spatial range).
    Scout(ScoutCommandArgs),
}

#[derive(Args, Debug, Clone)]
struct IndexBuildArgs {
    /// Force rebuild even if index already exists.
    #[arg(long, action = ArgAction::SetTrue)]
    force: bool,
}

#[derive(Args, Debug, Clone)]
struct IndexVerifyArgs {
    /// Output in JSON format instead of human-readable text.
    #[arg(long, action = ArgAction::SetTrue)]
    json: bool,

    /// Only output on failure (quiet mode for scripts).
    #[arg(short, long, action = ArgAction::SetTrue)]
    quiet: bool,

    /// Require release tag match in addition to checksum (strict mode).
    #[arg(long, action = ArgAction::SetTrue)]
    strict: bool,
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
                avoid_critical_state: self.options.avoid_critical_state,
                ship: None,
                loadout: None,
                heat_config: None,
            },
            spatial_index: None, // Will be set separately after loading
            max_spatial_neighbors: self.options.max_spatial_neighbours,
            optimization: match self.options.optimize.unwrap_or_default() {
                RouteOptimizeArg::Distance => evefrontier_lib::routing::RouteOptimization::Distance,
                RouteOptimizeArg::Fuel => evefrontier_lib::routing::RouteOptimization::Fuel,
            },
            fuel_config: evefrontier_lib::ship::FuelConfig {
                quality: self.options.fuel_quality as f64,
                dynamic_mass: self.options.dynamic_mass,
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
    ///
    /// Only applies to spatial jumps - systems with star temperature above this
    /// threshold cannot be reached via spatial jumps (ships would overheat).
    /// Gate jumps are unaffected by temperature.
    #[arg(long = "max-temp")]
    max_temp: Option<f64>,

    /// Suppress minimum external temperature annotations in route output.
    #[arg(long = "no-temp", action = ArgAction::SetTrue)]
    no_temp: bool,

    /// Ship name for fuel projection (case-insensitive).
    #[arg(long = "ship")]
    ship: Option<String>,

    /// Fuel quality rating (1-100, default 10).
    #[arg(long = "fuel-quality", default_value = "10")]
    fuel_quality: i64,

    /// Cargo mass in kilograms.
    #[arg(long = "cargo-mass", default_value = "0")]
    cargo_mass: f64,

    /// Initial fuel load (units). Defaults to full capacity.
    #[arg(long = "fuel-load")]
    fuel_load: Option<f64>,

    /// Recalculate mass after each hop as fuel is consumed.
    #[arg(long = "dynamic-mass", action = ArgAction::SetTrue)]
    dynamic_mass: bool,

    /// Avoid hops that would cause engine to reach critical heat state (requires --ship)
    /// This behavior is enabled by default; use `--no-avoid-critical-state` to opt out.
    #[arg(long = "avoid-critical-state", action = ArgAction::SetTrue)]
    avoid_critical_state: bool,

    /// Disable the default avoidance of critical engine state (opt-out flag).
    #[arg(long = "no-avoid-critical-state", action = ArgAction::SetTrue)]
    no_avoid_critical_state: bool,

    /// Maximum number of spatial neighbours to consider when building the spatial/hybrid graph.
    /// Defaults to 250 to limit fan-out for common runs and improve performance.
    #[arg(long = "max-spatial-neighbours", default_value_t = 250usize)]
    max_spatial_neighbours: usize,

    /// Optimization objective for planning: distance or fuel.
    #[arg(long = "optimize", value_enum)]
    optimize: Option<RouteOptimizeArg>,
}

#[derive(Args, Debug, Clone)]
struct FmapEncodeArgs {
    /// System names to encode (comma-separated or repeated --system flags).
    /// First system is the start, last is the destination.
    #[arg(value_name = "SYSTEM", required = true)]
    systems: Vec<String>,

    /// Waypoint type for each system: start, jump, npc-gate, smart-gate, destination.
    /// Defaults: first=start, middle=jump, last=destination.
    #[arg(long = "type", value_name = "TYPE")]
    types: Vec<String>,

    /// Output in JSON format (includes metadata).
    #[arg(long = "json", action = ArgAction::SetTrue)]
    json: bool,
}

#[derive(Args, Debug, Clone)]
struct FmapDecodeArgs {
    /// Base64url-encoded fmap token string.
    #[arg(value_name = "TOKEN", required = true)]
    token: String,

    /// Output in JSON format (includes metadata).
    #[arg(long = "json", action = ArgAction::SetTrue)]
    json: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum, Default)]
enum RouteAlgorithmArg {
    Bfs,
    #[default]
    Dijkstra,
    #[value(name = "a-star")]
    AStar,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum, Default)]
enum RouteOptimizeArg {
    /// Shortest distance (default)
    #[default]
    Distance,
    /// Minimize fuel consumption (requires --ship)
    Fuel,
}

// Note: Dijkstra is the intentionally selected default algorithm (marked with #[default]).
// The ordering here is chosen for presentation and the default is explicit via the attribute.

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
    Text,
    Rich,
    Json,
    /// Minimal path-only output with +/|/- prefixes.
    Basic,
    /// Emoji-enhanced readable output per EXAMPLES.md.
    Emoji,
    /// Enhanced format with system details (temp, planets, moons).
    #[default]
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
        if let Some(ref ship) = output.ship_data_path {
            println!("Ship data available at {}", ship);
        }
        Ok(())
    }

    fn render_route_result(
        self,
        summary: &RouteSummary,
        show_temps: bool,
        base_url: &str,
    ) -> Result<()> {
        match self {
            OutputFormat::Text => {
                output::render_text(summary, show_temps, base_url);
            }
            OutputFormat::Rich => {
                output::render_rich(summary, show_temps, base_url);
            }
            OutputFormat::Json => {
                output::render_json(summary)?;
            }
            OutputFormat::Basic => {
                output::render_basic(summary, show_temps, base_url);
            }
            OutputFormat::Emoji => {
                output::render_emoji(summary, show_temps, base_url);
            }
            OutputFormat::Note => {
                output::render_note(summary, base_url);
            }
            OutputFormat::Enhanced => {
                output::render_enhanced(summary, base_url);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
struct DownloadOutput {
    dataset_path: String,
    release: ReleaseRequest,
    /// Optional path to the cached ship_data CSV if available.
    ship_data_path: Option<String>,
}

impl DownloadOutput {
    fn new(dataset_path: &Path, release: &DatasetRelease, ship_data: Option<&Path>) -> Self {
        Self {
            dataset_path: dataset_path.display().to_string(),
            release: release.into(),
            ship_data_path: ship_data.map(|p| p.display().to_string()),
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

    fn fmap_base_url(&self) -> &str {
        self.options
            .fmap_base_url
            .as_deref()
            .unwrap_or(output::DEFAULT_FMAP_BASE_URL)
    }
}

// The CLI uses Tokio's async runtime for the entire process (`#[tokio::main]`) to support
// launching the MCP stdio server directly from the CLI. This choice simplifies integration but
// introduces a small runtime overhead for otherwise synchronous subcommands; if startup overhead
// becomes a concern we can restrict the runtime to the MCP subcommand only using a dedicated
// runtime builder.
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let context = AppContext::new(cli.global);
    // Apply --no-color override early so all downstream rendering respects it.
    crate::terminal::set_color_disabled(context.options.no_color);

    // For JSON output, suppress tracing to keep stdout clean. If launching the MCP
    // subcommand, skip global tracing initialization so the MCP command can set
    // up a stderr-only tracing subscriber without conflicting with the global default.
    if context.output_format() != OutputFormat::Json && !matches!(cli.command, Command::Mcp(_)) {
        init_tracing();
    }

    let start = std::time::Instant::now();

    // Suppress CLI banner when acting as a stdio-based MCP server to avoid
    // corrupting the JSON-RPC protocol on stdout.
    if !matches!(cli.command, Command::Mcp(_)) && context.should_show_logo() {
        output::print_logo();
    }

    let result = match cli.command {
        Command::Download => handle_download(&context),
        Command::Route(route_args) => {
            handle_route_command(&context, &route_args, RouteOutputKind::Route)
        }
        Command::IndexBuild(args) => handle_index_build(&context, &args),
        Command::IndexVerify(args) => handle_index_verify(&context, &args),
        Command::Ships => handle_list_ships(&context),
        Command::FmapEncode(args) => handle_fmap_encode(&context, &args),
        Command::FmapDecode(args) => handle_fmap_decode(&args),
        Command::Mcp(args) => {
            commands::mcp::run_mcp_server(&context.options, args.log_level.as_deref()).await
        }
        Command::Scout(args) => handle_scout_command(&context, &args),
    };

    if result.is_ok() && context.should_show_footer() {
        let elapsed = start.elapsed();
        crate::output_helpers::print_footer(elapsed);
    }

    result
}

fn handle_download(context: &AppContext) -> Result<()> {
    let release = context.dataset_release();
    // Ensure dataset operation runs in a blocking region so it can perform
    // reqwest::blocking operations (which create their own runtime) without
    // being dropped from inside the async runtime which causes panics.
    let paths =
        tokio::task::block_in_place(|| ensure_dataset(context.target_path(), release.clone()))
            .context("failed to locate or download the EVE Frontier dataset")?;
    // Prefer DatasetPaths.ship_data, fall back to env var if provided
    let ship_path_buf: Option<PathBuf> = if let Some(p) = &paths.ship_data {
        Some(p.clone())
    } else {
        std::env::var_os("EVEFRONTIER_SHIP_DATA").map(PathBuf::from)
    };

    let output = DownloadOutput::new(&paths.database, &release, ship_path_buf.as_deref());
    context.output_format().render_download(&output)
}

fn handle_index_build(context: &AppContext, args: &IndexBuildArgs) -> Result<()> {
    let paths = tokio::task::block_in_place(|| {
        ensure_dataset(context.target_path(), context.dataset_release())
    })
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

    // Compute dataset checksum for freshness verification (v2 format)
    println!("Computing dataset checksum...");
    let checksum =
        compute_dataset_checksum(&paths.database).context("failed to compute dataset checksum")?;

    // Read release tag from marker file if present
    let release_tag = read_release_tag(&paths.database);

    // Create metadata for v2 format
    let metadata = DatasetMetadata {
        checksum,
        release_tag: release_tag.clone(),
        build_timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
    };

    println!(
        "Building spatial index (v2) for {} systems...",
        starmap.systems.len()
    );
    let index = SpatialIndex::build_with_metadata(&starmap, metadata);

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
    println!("  Format: v2 (with metadata)");
    println!("  Systems indexed: {}", index.len());
    println!("  Systems with temperature: {}", systems_with_temp);
    if let Some(ref tag) = release_tag {
        println!("  Dataset release: {}", tag);
    }
    println!("  Dataset checksum: {}...", hex::encode(&checksum[..8]));
    println!("  File size: {} bytes", file_size);

    Ok(())
}

/// Exit codes for index-verify command (per contract)
mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const STALE: i32 = 1;
    pub const MISSING: i32 = 2;
    pub const FORMAT_ERROR: i32 = 3;
    pub const DATASET_MISSING: i32 = 4;
    pub const ERROR: i32 = 5;
}

fn handle_index_verify(context: &AppContext, args: &IndexVerifyArgs) -> Result<()> {
    let start = std::time::Instant::now();

    // Resolve paths (run in blocking region to allow internal blocking I/O).
    let paths = tokio::task::block_in_place(|| {
        ensure_dataset(context.target_path(), context.dataset_release())
    })
    .context("failed to locate or download the EVE Frontier dataset")?;
    let index_path = spatial_index_path(&paths.database);

    // Run verification
    let result = verify_freshness(&index_path, &paths.database);

    // Compute diagnostics
    let verification_time_ms = start.elapsed().as_millis() as u64;
    let diagnostics = VerifyDiagnostics {
        dataset_path: paths.database.display().to_string(),
        index_path: index_path.display().to_string(),
        dataset_size: std::fs::metadata(&paths.database).ok().map(|m| m.len()),
        index_size: std::fs::metadata(&index_path).ok().map(|m| m.len()),
        index_version: detect_index_version(&index_path),
        verification_time_ms,
    };

    // Determine freshness and recommended action
    let (is_fresh, recommended_action, exit_code) = match &result {
        FreshnessResult::Fresh { .. } => (true, None, exit_codes::SUCCESS),
        FreshnessResult::Stale { .. } => (
            false,
            Some("evefrontier-cli index-build".to_string()),
            exit_codes::STALE,
        ),
        FreshnessResult::LegacyFormat { .. } => (
            false,
            Some("evefrontier-cli index-build --force".to_string()),
            exit_codes::FORMAT_ERROR,
        ),
        FreshnessResult::Missing { .. } => (
            false,
            Some("evefrontier-cli index-build".to_string()),
            exit_codes::MISSING,
        ),
        FreshnessResult::DatasetMissing { .. } => (
            false,
            Some("evefrontier-cli download".to_string()),
            exit_codes::DATASET_MISSING,
        ),
        FreshnessResult::Error { .. } => (false, None, exit_codes::ERROR),
    };

    // Build output structure
    let output = VerifyOutput {
        result: result.clone(),
        is_fresh,
        recommended_action: recommended_action.clone(),
        diagnostics: Some(diagnostics),
    };

    // Output based on format and quiet mode
    if args.json {
        // JSON output
        let json = serde_json::to_string_pretty(&output)?;
        if !args.quiet || !is_fresh {
            println!("{}", json);
        }
    } else {
        // Human-readable output
        if !args.quiet || !is_fresh {
            print_human_readable_result(&result, &output);
        }
    }

    // Exit with appropriate code
    if !is_fresh {
        std::process::exit(exit_code);
    }

    Ok(())
}

/// Detect the version byte from a spatial index file header.
fn detect_index_version(path: &std::path::Path) -> Option<u8> {
    use std::io::Read;
    let mut file = std::fs::File::open(path).ok()?;
    let mut header = [0u8; 16];
    file.read_exact(&mut header).ok()?;
    if &header[0..4] == b"EFSI" {
        Some(header[4])
    } else {
        None
    }
}

/// Print human-readable verification result.
fn print_human_readable_result(result: &FreshnessResult, output: &VerifyOutput) {
    match result {
        FreshnessResult::Fresh {
            checksum,
            release_tag,
        } => {
            println!("✓ Spatial index is fresh");
            if let Some(tag) = release_tag {
                println!("  Dataset:  {} ({}...)", tag, &checksum[..16]);
            } else {
                println!("  Dataset:  {}...", &checksum[..16]);
            }
            if let Some(ref diag) = output.diagnostics {
                if let Some(version) = diag.index_version {
                    println!("  Index:    v{} format", version);
                }
            }
        }
        FreshnessResult::Stale {
            expected_checksum,
            actual_checksum,
            expected_tag,
            actual_tag,
        } => {
            println!("✗ Spatial index is STALE");
            println!("  Dataset checksum:  {}...", &actual_checksum[..16]);
            println!("  Index source:      {}...", &expected_checksum[..16]);
            if expected_tag.is_some() || actual_tag.is_some() {
                println!(
                    "  Expected tag: {:?}, Actual tag: {:?}",
                    expected_tag, actual_tag
                );
            }
            println!();
            if let Some(ref action) = output.recommended_action {
                println!("  Run '{}' to regenerate", action);
            }
        }
        FreshnessResult::LegacyFormat {
            index_path,
            message,
        } => {
            println!("✗ Spatial index uses legacy format (v1)");
            println!("  Index file: {}", index_path);
            println!("  {}", message);
            println!();
            if let Some(ref action) = output.recommended_action {
                println!("  Run '{}' to upgrade to v2", action);
            }
        }
        FreshnessResult::Missing { expected_path } => {
            println!("✗ Spatial index not found");
            println!("  Expected: {}", expected_path);
            println!();
            if let Some(ref action) = output.recommended_action {
                println!("  Run '{}' to create", action);
            }
        }
        FreshnessResult::DatasetMissing { expected_path } => {
            println!("✗ Dataset not found");
            println!("  Expected: {}", expected_path);
            println!();
            if let Some(ref action) = output.recommended_action {
                println!("  Run '{}' to download", action);
            }
        }
        FreshnessResult::Error { message } => {
            println!("✗ Verification error");
            println!("  {}", message);
        }
    }
}

fn handle_route_command(
    context: &AppContext,
    args: &RouteCommandArgs,
    kind: RouteOutputKind,
) -> Result<()> {
    // Resolve dataset in a blocking region to avoid constructing blocking
    // HTTP clients inside the async runtime thread (see tokio reqwest runtime drop issue).
    let paths = tokio::task::block_in_place(|| {
        ensure_dataset(context.target_path(), context.dataset_release())
    })
    .context("failed to locate or download the EVE Frontier dataset")?;

    let starmap = load_starmap(&paths.database)
        .with_context(|| format!("failed to load dataset from {}", paths.database.display()))?;

    // Only load the spatial index when the selected algorithm can make use of it.
    // BFS does not use spatial indexing, so we avoid unnecessary I/O in that case.
    let needs_spatial_index = !matches!(args.options.algorithm, RouteAlgorithmArg::Bfs);
    let spatial_index = if needs_spatial_index {
        try_load_spatial_index(&paths.database).map(Arc::new)
    } else {
        None
    };

    let mut request = args.to_request();
    if let Some(index) = spatial_index {
        request = request.with_spatial_index(index);
    }

    // Respect explicit request semantics:
    // - If user explicitly requested heat-aware planning (`--avoid-critical-state`) they must
    //   also provide `--ship`. This preserves the historical behavior and avoids surprising
    //   automatic ship injection when the user explicitly opted into heat checks.
    if args.options.avoid_critical_state && args.options.ship.is_none() {
        return Err(anyhow::anyhow!(
            "--ship is required for heat-aware planning"
        ));
    }

    // Determine whether the user provided any route-specific options; if not, we're in
    // a zero-config invocation and may apply friendly defaults (like default ship).
    let user_provided_options = args.options.max_jump.is_some()
        || args.options.algorithm != RouteAlgorithmArg::default()
        || args.options.optimize.is_some()
        || !args.options.avoid.is_empty()
        || args.options.avoid_gates
        || args.options.max_temp.is_some()
        || args.options.ship.is_some()
        || args.options.fuel_quality != 10
        || args.options.cargo_mass != 0.0
        || args.options.fuel_load.is_some()
        || args.options.dynamic_mass
        || args.options.no_avoid_critical_state
        || args.options.avoid_critical_state
        || args.options.max_spatial_neighbours != 250usize;

    // Determine the effective ship name (support 'None' to explicitly disable ship-based planning).
    // Only inject a default ship when the user did not provide other routing options (zero-config case).
    let effective_ship_name: Option<String> = match args.options.ship.as_deref() {
        Some(s) if s.eq_ignore_ascii_case("none") => None,
        Some(s) => Some(s.to_string()),
        None => {
            if user_provided_options {
                None
            } else {
                Some("Reflex".to_string())
            }
        }
    };

    // Determine whether we should avoid critical engine state for this request.
    // Priority: explicit opt-out (--no-avoid-critical-state) > explicit opt-in (--avoid-critical-state) > implicit when a ship is available
    let avoid_critical = if args.options.no_avoid_critical_state {
        false
    } else if args.options.avoid_critical_state {
        true
    } else {
        // If a ship is available (explicit or default), enable avoid_critical_state behavior implicitly
        effective_ship_name.is_some()
    };

    request.constraints.avoid_critical_state = avoid_critical;

    // If it's a zero-config run, we want to default to Fuel optimization (with our default ship)
    // to provide the most feature-rich initial experience for users.
    if !user_provided_options && args.options.optimize.is_none() {
        request.optimization = evefrontier_lib::routing::RouteOptimization::Fuel;
    }

    // Load ship data and populate loadout if we have an effective ship name (and it's not "None").
    if let Some(ship_name) = effective_ship_name {
        // If the user passed --ship "None" we would have resolved to None above.

        // Attempt to load the ship catalog, but treat failures differently depending on
        // whether the user explicitly requested a ship.
        match load_ship_catalog(&paths) {
            Ok(catalog) => {
                let ship = catalog.get(&ship_name).ok_or_else(|| {
                    anyhow::anyhow!(format!("ship {} not found in catalog", ship_name))
                })?;

                let fuel_load = args.options.fuel_load.unwrap_or(ship.fuel_capacity);
                let loadout = ShipLoadout::new(ship, fuel_load, args.options.cargo_mass)
                    .context("invalid ship loadout")?;

                request.constraints.ship = Some(ship.clone());
                request.constraints.loadout = Some(loadout);

                // Only populate heat-specific configuration when heat-aware planning is requested.
                if request.constraints.avoid_critical_state {
                    let heat_config = evefrontier_lib::ship::HeatConfig {
                        calibration_constant: 1e-7,
                        dynamic_mass: args.options.dynamic_mass,
                    };
                    request.constraints.heat_config = Some(heat_config);
                }
            }
            Err(e) => {
                if args.options.ship.is_some() {
                    // User explicitly requested a ship — this is an error we should propagate.
                    return Err(e).context("failed to load requested ship data");
                } else {
                    // Implicit default ship couldn't be loaded (missing ship_data.csv etc.).
                    // Don't fail the entire command for this non-critical missing file; warn and
                    // proceed without a default ship.
                    eprintln!(
                        "Warning: failed to load ship data: {}. Proceeding without default ship.",
                        e
                    );
                }
            }
        }
    }

    let plan = match plan_route(&starmap, &request) {
        Ok(plan) => plan,
        Err(err) => return Err(handle_route_failure(&request, err)),
    };

    let mut summary = RouteSummary::from_plan(kind, &starmap, &plan, Some(&request))
        .context("failed to build route summary for display")?;

    // Generate fmap URL for the route using the summary steps which have method info
    let waypoints: Result<Vec<Waypoint>> = summary
        .steps
        .iter()
        .enumerate()
        .map(|(idx, step)| {
            let wtype = if idx == 0 {
                WaypointType::Start
            } else {
                // Use the method field to determine if it's a gate or spatial jump.
                // Treat unknown methods as a jump but emit a warning to surface data issues.
                match step.method.as_deref() {
                    Some("gate") => WaypointType::NpcGate,
                    Some("jump") => WaypointType::Jump,
                    Some(other) => {
                        eprintln!(
                            "Warning: unexpected route step method '{}' for system id {}; treating as 'jump' for fmap URL generation.",
                            other,
                            step.id
                        );
                        WaypointType::Jump
                    }
                    None => WaypointType::Jump,
                }
            };
            let system_id_u32 = u32::try_from(step.id)
                .with_context(|| format!("system id {} out of range for fmap token", step.id))?;
            Ok(Waypoint {
                system_id: system_id_u32,
                waypoint_type: wtype,
            })
        })
        .collect();

    match waypoints.and_then(|w| encode_fmap_token(&w).map_err(Into::into)) {
        Ok(token) => {
            summary.fmap_url = Some(token.token);
        }
        Err(err) => {
            // Do not fail the entire command on optional URL generation issues,
            // but make the failure visible to the user by setting a placeholder.
            // This ensures the render functions can show that URL generation was attempted but failed.
            eprintln!(
                "Warning: failed to generate fmap URL for this route: {}",
                err
            );
            summary.fmap_url = Some("(generation failed)".to_string());
        }
    }

    // If we have an effective ship & loadout (explicit or injected default), attach
    // fuel and heat projections so the summary reflects those values and the footer
    // estimation box can be rendered when appropriate.
    if let (Some(ship), Some(loadout)) = (&request.constraints.ship, &request.constraints.loadout) {
        let fuel_config = evefrontier_lib::ship::FuelConfig {
            quality: request.fuel_config.quality,
            dynamic_mass: request.fuel_config.dynamic_mass,
        };

        summary
            .attach_fuel(ship, loadout, &fuel_config)
            .context("failed to attach fuel projection")?;

        // Attach heat projections using the same dynamic_mass behaviour
        let heat_config = evefrontier_lib::ship::HeatConfig {
            calibration_constant: 1e-7,
            dynamic_mass: request.fuel_config.dynamic_mass,
        };

        summary
            .attach_heat(ship, loadout, &heat_config)
            .context("failed to attach heat projection")?;
    }

    let show_temps = !args.options.no_temp;
    context
        .output_format()
        .render_route_result(&summary, show_temps, context.fmap_base_url())
}

fn handle_list_ships(context: &AppContext) -> Result<()> {
    let paths = tokio::task::block_in_place(|| {
        ensure_dataset(context.target_path(), context.dataset_release())
    })
    .context("failed to locate or download the EVE Frontier dataset")?;

    let catalog = load_ship_catalog(&paths)?;
    print_ship_catalog(&catalog);
    Ok(())
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
    if constraints.avoid_critical_state {
        // If the user explicitly asked to avoid critical engine states, suggest removing
        // the restriction. If no ship was supplied, also suggest specifying one so the
        // planner can evaluate heat-aware routes; when a ship is already present, only
        // recommend omitting the restriction since adding a ship is redundant.
        if constraints.ship.is_some() {
            tips.push("omit --avoid-critical-state");
        } else {
            tips.push("omit --avoid-critical-state or specify a ship with --ship");
        }
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

fn load_ship_catalog(paths: &evefrontier_lib::DatasetPaths) -> Result<ShipCatalog> {
    // Prefer ship data discovered by the dataset resolver (populated in `DatasetPaths`)
    if let Some(ref ship_path) = paths.ship_data {
        if ship_path.exists() {
            return ShipCatalog::from_path(ship_path)
                .with_context(|| format!("failed to load ship data from {}", ship_path.display()));
        }
    }

    let candidates = ship_data_candidates(&paths.database);
    let path = candidates
        .iter()
        .find(|p| p.exists())
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "ship_data.csv not found; set EVEFRONTIER_SHIP_DATA or place file next to dataset"
            )
        })?;

    ShipCatalog::from_path(&path)
        .with_context(|| format!("failed to load ship data from {}", path.display()))
}

fn ship_data_candidates(database: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(env_path) = std::env::var("EVEFRONTIER_SHIP_DATA") {
        candidates.push(PathBuf::from(env_path));
    }

    if let Some(parent) = database.parent() {
        candidates.push(parent.join("ship_data.csv"));
    }

    if cfg!(debug_assertions) {
        let fixture =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/ship_data.csv");
        candidates.push(fixture);
    }

    candidates
}

fn print_ship_catalog(catalog: &ShipCatalog) {
    let ships = catalog.ships_sorted();
    if ships.is_empty() {
        println!("No ships available in catalog.");
        return;
    }

    println!("Available ships ({}):", ships.len());
    println!(
        "{:<16} {:>14} {:>10} {:>12}",
        "Name", "Base Mass (kg)", "Fuel Cap", "Cargo Cap"
    );
    for ship in ships {
        println!(
            "{:<16} {:>14.0} {:>10.0} {:>12.0}",
            ship.name, ship.base_mass_kg, ship.fuel_capacity, ship.cargo_capacity
        );
    }
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);
}

fn handle_fmap_encode(context: &AppContext, args: &FmapEncodeArgs) -> Result<()> {
    if args.systems.is_empty() {
        anyhow::bail!("At least one system name is required");
    }

    // Parse waypoint types with defaults
    let mut waypoint_types = Vec::new();
    for (i, _system_name) in args.systems.iter().enumerate() {
        let wtype = if i < args.types.len() {
            match args.types[i].as_str() {
                "start" => WaypointType::Start,
                "jump" => WaypointType::Jump,
                "npc-gate" => WaypointType::NpcGate,
                "smart-gate" => WaypointType::SmartGate,
                "destination" | "dest" => WaypointType::SetDestination,
                other => anyhow::bail!("invalid waypoint type: {}", other),
            }
        } else if i == 0 {
            WaypointType::Start
        } else if i == args.systems.len() - 1 {
            WaypointType::SetDestination
        } else {
            WaypointType::Jump
        };
        waypoint_types.push(wtype);
    }

    // Check if we need database lookup (if any system name fails to parse as u32)
    let needs_db_lookup = args.systems.iter().any(|sys| sys.parse::<u32>().is_err());

    // Resolve system names to IDs
    let mut waypoints = Vec::new();
    let starmap =
        if needs_db_lookup {
            let paths = ensure_dataset(context.target_path(), context.dataset_release())
                .context("failed to locate or download the EVE Frontier dataset")?;
            Some(load_starmap(&paths.database).with_context(|| {
                format!("failed to load dataset from {}", paths.database.display())
            })?)
        } else {
            None
        };

    for (system_name, wtype) in args.systems.iter().zip(waypoint_types.iter()) {
        // Try to parse as a numeric system ID first
        let system_id = match system_name.parse::<u32>() {
            Ok(id) => id,
            Err(_) => {
                // Look up system name in the database
                let db = starmap.as_ref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "system name '{}' requires database lookup, but database failed to load",
                        system_name
                    )
                })?;
                match db.system_id_by_name(system_name) {
                    Some(id) => id as u32,
                    None => {
                        // System not found, provide helpful suggestions
                        let suggestions = db.fuzzy_system_matches(system_name, 5);
                        if suggestions.is_empty() {
                            anyhow::bail!(
                                "unknown system '{}'. Use a numeric system ID or an exact system name from the database",
                                system_name
                            );
                        } else {
                            anyhow::bail!(
                                "unknown system '{}'. Did you mean one of: {}? Or use a numeric system ID",
                                system_name,
                                suggestions.join(", ")
                            );
                        }
                    }
                }
            }
        };

        waypoints.push(Waypoint {
            system_id,
            waypoint_type: *wtype,
        });
    }

    // Encode the token
    let token =
        encode_fmap_token(&waypoints).map_err(|e| anyhow::anyhow!("encoding failed: {}", e))?;

    if args.json {
        #[derive(Serialize)]
        struct FmapOutput {
            token: String,
            waypoint_count: usize,
            bit_width: u8,
            version: u8,
        }

        let output = FmapOutput {
            token: token.token.clone(),
            waypoint_count: token.waypoint_count,
            bit_width: token.bit_width,
            version: token.version,
        };

        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("fmap token: {}", token.token);
        println!("waypoints: {}", token.waypoint_count);
        println!("bit width: {}", token.bit_width);
    }

    Ok(())
}

fn handle_fmap_decode(args: &FmapDecodeArgs) -> Result<()> {
    // Decode the token
    let decoded =
        decode_fmap_token(&args.token).map_err(|e| anyhow::anyhow!("decoding failed: {}", e))?;

    if args.json {
        #[derive(Serialize)]
        struct WaypointOutput {
            system_id: u32,
            waypoint_type: String,
        }

        #[derive(Serialize)]
        struct FmapDecodedOutput {
            version: u8,
            bit_width: u8,
            waypoint_count: usize,
            waypoints: Vec<WaypointOutput>,
        }

        let waypoints = decoded
            .waypoints
            .iter()
            .map(|wp| WaypointOutput {
                system_id: wp.system_id,
                waypoint_type: format!("{:?}", wp.waypoint_type).to_lowercase(),
            })
            .collect();

        let output = FmapDecodedOutput {
            version: decoded.version,
            bit_width: decoded.bit_width,
            waypoint_count: decoded.waypoint_count,
            waypoints,
        };

        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("fmap decoded successfully");
        println!("version: {}", decoded.version);
        println!("bit width: {}", decoded.bit_width);
        println!("waypoints: {}", decoded.waypoint_count);
        println!();
        println!("{:<15} {:<20}", "System ID", "Type");
        println!("{}", "-".repeat(35));
        for wp in decoded.waypoints {
            println!(
                "{:<15} {:<20}",
                wp.system_id,
                format!("{:?}", wp.waypoint_type)
            );
        }
    }

    Ok(())
}

fn handle_scout_command(context: &AppContext, args: &ScoutCommandArgs) -> Result<()> {
    match &args.subcommand {
        ScoutSubcommand::Gates(gate_args) => commands::scout::handle_scout_gates(
            gate_args,
            context.output_format(),
            context.target_path(),
        ),
        ScoutSubcommand::Range(range_args) => commands::scout::handle_scout_range(
            range_args,
            context.output_format(),
            context.target_path(),
        ),
    }
}
