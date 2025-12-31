use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod output;
mod terminal;

use evefrontier_lib::{
    compute_dataset_checksum, ensure_dataset, load_starmap, plan_route, read_release_tag,
    spatial_index_path, try_load_spatial_index, verify_freshness, DatasetMetadata, DatasetRelease,
    Error as RouteError, FreshnessResult, RouteAlgorithm, RouteConstraints, RouteOutputKind,
    RouteRequest, RouteSummary, ShipCatalog, ShipLoadout, SpatialIndex, VerifyDiagnostics,
    VerifyOutput,
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
    /// Verify that the spatial index is fresh (matches the current dataset).
    IndexVerify(IndexVerifyArgs),
    /// List available ships from ship_data.csv.
    Ships,
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

    /// Ship name for fuel projection (case-insensitive).
    #[arg(long = "ship")]
    ship: Option<String>,

    /// Fuel quality rating (1-100, default 10).
    #[arg(long = "fuel-quality", default_value = "10")]
    fuel_quality: f64,

    /// Cargo mass in kilograms.
    #[arg(long = "cargo-mass", default_value = "0")]
    cargo_mass: f64,

    /// Initial fuel load (units). Defaults to full capacity.
    #[arg(long = "fuel-load")]
    fuel_load: Option<f64>,

    /// Recalculate mass after each hop as fuel is consumed.
    #[arg(long = "dynamic-mass", action = ArgAction::SetTrue)]
    dynamic_mass: bool,
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
                output::render_text(summary, show_temps);
            }
            OutputFormat::Rich => {
                output::render_rich(summary, show_temps);
            }
            OutputFormat::Json => {
                output::render_json(summary)?;
            }
            OutputFormat::Basic => {
                output::render_basic(summary, show_temps);
            }
            OutputFormat::Emoji => {
                output::render_emoji(summary, show_temps);
            }
            OutputFormat::Note => {
                output::render_note(summary);
            }
            OutputFormat::Enhanced => {
                output::render_enhanced(summary);
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
    };

    if result.is_ok() && context.should_show_footer() {
        let elapsed = start.elapsed();
        output::print_footer(elapsed);
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

    // Resolve paths
    let paths = ensure_dataset(context.target_path(), context.dataset_release())
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
    let paths = ensure_dataset(context.target_path(), context.dataset_release())
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

    let plan = match plan_route(&starmap, &request) {
        Ok(plan) => plan,
        Err(err) => return Err(handle_route_failure(&request, err)),
    };

    let mut summary = RouteSummary::from_plan(kind, &starmap, &plan)
        .context("failed to build route summary for display")?;

    if let Some(ship_name) = args.options.ship.as_ref() {
        let catalog = load_ship_catalog(&paths)?;
        let ship = catalog
            .get(ship_name)
            .ok_or_else(|| anyhow::anyhow!(format!("ship '{}' not found in catalog", ship_name)))?;

        let fuel_load = args.options.fuel_load.unwrap_or(ship.fuel_capacity);

        let loadout = ShipLoadout::new(ship, fuel_load, args.options.cargo_mass)
            .context("invalid ship loadout")?;

        let fuel_config = evefrontier_lib::ship::FuelConfig {
            quality: args.options.fuel_quality,
            dynamic_mass: args.options.dynamic_mass,
        };

        summary
            .attach_fuel(ship, &loadout, &fuel_config)
            .context("failed to attach fuel projection")?;
    }

    let show_temps = !args.options.no_temp;
    context
        .output_format()
        .render_route_result(&summary, show_temps)
}

fn handle_list_ships(context: &AppContext) -> Result<()> {
    let paths = ensure_dataset(context.target_path(), context.dataset_release())
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
