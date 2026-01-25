//! Route command handler for computing paths between star systems.

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};

use evefrontier_lib::{
    encode_fmap_token, ensure_dataset, load_starmap, plan_route, try_load_spatial_index,
    DatasetPaths, DatasetRelease, Error as RouteError, RouteAlgorithm, RouteConstraints,
    RouteOutputKind, RouteRequest, RouteSummary, ShipLoadout, Waypoint, WaypointType,
};

use crate::commands::ships::load_ship_catalog;
use crate::output::OutputFormat;

/// Arguments for the route command.
#[derive(Debug, Clone)]
pub struct RouteCommandArgs {
    /// Starting system name.
    pub from: String,
    /// Destination system name.
    pub to: String,
    /// Algorithm to use when planning the route.
    pub algorithm: RouteAlgorithm,
    /// Maximum jump distance (light-years).
    pub max_jump: Option<f64>,
    /// Systems to avoid.
    pub avoid: Vec<String>,
    /// Avoid gates entirely.
    pub avoid_gates: bool,
    /// Maximum system temperature threshold in Kelvin.
    pub max_temp: Option<f64>,
    /// Suppress minimum external temperature annotations.
    pub no_temp: bool,
    /// Ship name for fuel projection.
    pub ship: Option<String>,
    /// Fuel quality rating (1-100).
    pub fuel_quality: i64,
    /// Cargo mass in kilograms.
    pub cargo_mass: f64,
    /// Initial fuel load (units).
    pub fuel_load: Option<f64>,
    /// Recalculate mass after each hop.
    pub dynamic_mass: bool,
    /// Avoid hops causing critical heat state.
    pub avoid_critical_state: bool,
    /// Disable default avoidance of critical state.
    pub no_avoid_critical_state: bool,
    /// Maximum spatial neighbours to consider.
    pub max_spatial_neighbours: usize,
    /// Optimization objective.
    pub optimize: Option<RouteOptimization>,
}

/// Route optimization objective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RouteOptimization {
    #[default]
    Distance,
    Fuel,
}

impl RouteCommandArgs {
    /// Convert CLI args to a library RouteRequest.
    pub fn to_request(&self) -> RouteRequest {
        RouteRequest {
            start: self.from.clone(),
            goal: self.to.clone(),
            algorithm: self.algorithm,
            constraints: RouteConstraints {
                max_jump: self.max_jump,
                avoid_systems: self.avoid.clone(),
                avoid_gates: self.avoid_gates,
                max_temperature: self.max_temp,
                avoid_critical_state: self.avoid_critical_state,
                ship: None,
                loadout: None,
                heat_config: None,
            },
            spatial_index: None,
            max_spatial_neighbors: self.max_spatial_neighbours,
            optimization: match self.optimize.unwrap_or_default() {
                RouteOptimization::Distance => {
                    evefrontier_lib::routing::RouteOptimization::Distance
                }
                RouteOptimization::Fuel => evefrontier_lib::routing::RouteOptimization::Fuel,
            },
            fuel_config: evefrontier_lib::ship::FuelConfig {
                quality: self.fuel_quality as f64,
                dynamic_mass: self.dynamic_mass,
            },
        }
    }

    /// Check if the user provided any route-specific options.
    pub fn user_provided_options(&self) -> bool {
        self.max_jump.is_some()
            || self.algorithm != RouteAlgorithm::Dijkstra
            || self.optimize.is_some()
            || !self.avoid.is_empty()
            || self.avoid_gates
            || self.max_temp.is_some()
            || self.ship.is_some()
            || self.fuel_quality != 10
            || self.cargo_mass != 0.0
            || self.fuel_load.is_some()
            || self.dynamic_mass
            || self.no_avoid_critical_state
            || self.avoid_critical_state
            || self.max_spatial_neighbours != 250usize
    }
}

/// Handle the route subcommand.
///
/// Computes a route between two star systems using the loaded dataset.
pub fn handle_route_command(
    target_path: Option<&Path>,
    release: DatasetRelease,
    format: OutputFormat,
    fmap_base_url: &str,
    args: &RouteCommandArgs,
    kind: RouteOutputKind,
) -> Result<()> {
    // Resolve dataset in a blocking region
    let paths = tokio::task::block_in_place(|| ensure_dataset(target_path, release))
        .context("failed to locate or download the EVE Frontier dataset")?;

    let starmap = load_starmap(&paths.database)
        .with_context(|| format!("failed to load dataset from {}", paths.database.display()))?;

    // Only load the spatial index when the selected algorithm can make use of it.
    let needs_spatial_index = !matches!(args.algorithm, RouteAlgorithm::Bfs);
    let spatial_index = if needs_spatial_index {
        try_load_spatial_index(&paths.database).map(Arc::new)
    } else {
        None
    };

    let mut request = args.to_request();
    if let Some(index) = spatial_index {
        request = request.with_spatial_index(index);
    }

    // Validate heat-aware planning requires a ship
    if args.avoid_critical_state && args.ship.is_none() {
        return Err(anyhow::anyhow!(
            "--ship is required for heat-aware planning"
        ));
    }

    let user_provided_options = args.user_provided_options();

    // Determine effective ship name
    let effective_ship_name: Option<String> = match args.ship.as_deref() {
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

    // Determine whether to avoid critical engine state
    let avoid_critical = if args.no_avoid_critical_state {
        false
    } else if args.avoid_critical_state {
        true
    } else {
        effective_ship_name.is_some()
    };

    request.constraints.avoid_critical_state = avoid_critical;

    // Default to Fuel optimization for zero-config runs
    if !user_provided_options && args.optimize.is_none() {
        request.optimization = evefrontier_lib::routing::RouteOptimization::Fuel;
    }

    // Load ship data if we have an effective ship name
    if let Some(ship_name) = effective_ship_name {
        match load_ship_catalog(&paths) {
            Ok(catalog) => {
                let ship = catalog.get(&ship_name).ok_or_else(|| {
                    anyhow::anyhow!(format!("ship {} not found in catalog", ship_name))
                })?;

                let fuel_load = args.fuel_load.unwrap_or(ship.fuel_capacity);
                let loadout = ShipLoadout::new(ship, fuel_load, args.cargo_mass)
                    .context("invalid ship loadout")?;

                request.constraints.ship = Some(ship.clone());
                request.constraints.loadout = Some(loadout);

                if request.constraints.avoid_critical_state {
                    let heat_config = evefrontier_lib::ship::HeatConfig {
                        calibration_constant: 1e-7,
                        dynamic_mass: args.dynamic_mass,
                    };
                    request.constraints.heat_config = Some(heat_config);
                }
            }
            Err(e) => {
                if args.ship.is_some() {
                    return Err(e).context("failed to load requested ship data");
                } else {
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

    // Generate fmap URL for the route
    summary.fmap_url = generate_fmap_url(&summary);

    // Attach fuel and heat projections if ship data is available
    attach_projections(&mut summary, &request, &paths)?;

    let show_temps = !args.no_temp;
    format.render_route_result(&summary, show_temps, fmap_base_url)
}

/// Generate fmap URL from route summary.
fn generate_fmap_url(summary: &RouteSummary) -> Option<String> {
    let waypoints: Result<Vec<Waypoint>, _> = summary
        .steps
        .iter()
        .enumerate()
        .map(|(idx, step)| {
            let wtype = if idx == 0 {
                WaypointType::Start
            } else {
                match step.method.as_deref() {
                    Some("gate") => WaypointType::NpcGate,
                    Some("jump") => WaypointType::Jump,
                    Some(other) => {
                        eprintln!(
                            "Warning: unexpected route step method '{}' for system id {}; treating as 'jump'.",
                            other, step.id
                        );
                        WaypointType::Jump
                    }
                    None => WaypointType::Jump,
                }
            };
            let system_id = u32::try_from(step.id)
                .map_err(|_| anyhow::anyhow!("system id {} out of range", step.id))?;
            Ok(Waypoint {
                system_id,
                waypoint_type: wtype,
            })
        })
        .collect();

    match waypoints.and_then(|w| encode_fmap_token(&w).map_err(anyhow::Error::from)) {
        Ok(token) => Some(token.token),
        Err(err) => {
            eprintln!("Warning: failed to generate fmap URL: {}", err);
            Some("(generation failed)".to_string())
        }
    }
}

/// Attach fuel and heat projections to the summary.
fn attach_projections(
    summary: &mut RouteSummary,
    request: &RouteRequest,
    _paths: &DatasetPaths,
) -> Result<()> {
    if let (Some(ship), Some(loadout)) = (&request.constraints.ship, &request.constraints.loadout) {
        let fuel_config = evefrontier_lib::ship::FuelConfig {
            quality: request.fuel_config.quality,
            dynamic_mass: request.fuel_config.dynamic_mass,
        };

        summary
            .attach_fuel(ship, loadout, &fuel_config)
            .context("failed to attach fuel projection")?;

        let heat_config = evefrontier_lib::ship::HeatConfig {
            calibration_constant: 1e-7,
            dynamic_mass: request.fuel_config.dynamic_mass,
        };

        summary
            .attach_heat(ship, loadout, &heat_config)
            .context("failed to attach heat projection")?;
    }

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
