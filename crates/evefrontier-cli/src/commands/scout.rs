//! Scout subcommand handlers for gate neighbors and spatial range queries.
//!
//! This module provides the implementation for:
//! - `scout gates <SYSTEM>` — list gate-connected neighbors
//! - `scout range <SYSTEM>` — find systems within spatial range

use anyhow::{Context, Result};
use evefrontier_lib::{
    ensure_dataset, load_starmap, try_load_spatial_index, DatasetRelease, FuelConfig, HeatConfig,
    NeighbourQuery, ShipCatalog, ShipLoadout,
};

use crate::output::OutputFormat;
use crate::output_helpers::{
    format_scout_gates_basic, format_scout_gates_emoji, format_scout_gates_enhanced,
    format_scout_gates_note, format_scout_gates_text, format_scout_range_basic,
    format_scout_range_emoji, format_scout_range_enhanced, format_scout_range_note,
    format_scout_range_text, GateNeighbor, RangeNeighbor, RangeQueryParams, ScoutGatesResult,
    ScoutRangeResult, ShipInfo,
};
use crate::terminal::ColorPalette;
use crate::{ScoutGatesArgs, ScoutRangeArgs};

// =============================================================================
// CCP System Filter
// =============================================================================

/// Check if a system name is a CCP developer/staging system.
///
/// CCP systems follow these patterns:
/// - `AD###` (e.g., AD035, AD134) — developer staging systems
/// - `V-###` (e.g., V-001, V-999) — test systems
///
/// These are filtered out by default to show only player-accessible systems.
fn is_ccp_system(name: &str) -> bool {
    // AD### pattern: starts with "AD" followed by digits
    if name.len() >= 3 && name.starts_with("AD") {
        return name[2..].chars().all(|c| c.is_ascii_digit());
    }

    // V-### pattern: starts with "V-" followed by digits
    if name.len() >= 3 && name.starts_with("V-") {
        return name[2..].chars().all(|c| c.is_ascii_digit());
    }

    false
}

// =============================================================================
// Nearest-Neighbor Ordering Algorithm
// =============================================================================

/// Intermediate struct for nearest-neighbor ordering that includes spatial position.
struct SystemWithPosition {
    neighbor: RangeNeighbor,
    position: [f64; 3],
}

/// Compute Euclidean distance between two 3D positions.
fn euclidean_distance(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Order systems using the nearest-neighbor heuristic (greedy Hamiltonian path approximation).
///
/// Starting from the origin position, repeatedly selects the closest unvisited system.
/// This produces a visiting order that minimizes individual hop distances, though not
/// necessarily the globally optimal total distance.
///
/// Returns the systems reordered by visit order, with `distance_ly` updated to show
/// the hop distance from the previous system (not distance from origin).
fn nearest_neighbor_order(
    origin_position: [f64; 3],
    systems: Vec<SystemWithPosition>,
) -> Vec<RangeNeighbor> {
    if systems.is_empty() {
        return Vec::new();
    }

    let mut unvisited = systems;
    let mut ordered = Vec::with_capacity(unvisited.len());
    let mut current_position = origin_position;

    while !unvisited.is_empty() {
        // Find the nearest unvisited system to current position
        let (nearest_idx, hop_distance) = unvisited
            .iter()
            .enumerate()
            .map(|(idx, sys)| (idx, euclidean_distance(&current_position, &sys.position)))
            .min_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap_or(std::cmp::Ordering::Equal))
            .expect("unvisited is not empty");

        // Remove from unvisited and update the hop distance
        let mut next = unvisited.remove(nearest_idx);
        next.neighbor.distance_ly = hop_distance;
        current_position = next.position;
        ordered.push(next.neighbor);
    }

    ordered
}

// =============================================================================
// Ship Catalog Loading
// =============================================================================

/// Load ship catalog from dataset paths or environment.
fn load_ship_catalog(paths: &evefrontier_lib::DatasetPaths) -> anyhow::Result<ShipCatalog> {
    use std::path::PathBuf;

    // Prefer ship data discovered by the dataset resolver
    if let Some(ref ship_path) = paths.ship_data {
        if ship_path.exists() {
            return ShipCatalog::from_path(ship_path).map_err(|e| {
                anyhow::anyhow!(
                    "failed to load ship data from {}: {}",
                    ship_path.display(),
                    e
                )
            });
        }
    }

    // Check environment variable
    if let Ok(env_path) = std::env::var("EVEFRONTIER_SHIP_DATA") {
        let path = PathBuf::from(&env_path);
        if path.exists() {
            return ShipCatalog::from_path(&path).map_err(|e| {
                anyhow::anyhow!("failed to load ship data from {}: {}", path.display(), e)
            });
        }
    }

    // Check next to database
    if let Some(parent) = paths.database.parent() {
        let path = parent.join("ship_data.csv");
        if path.exists() {
            return ShipCatalog::from_path(&path).map_err(|e| {
                anyhow::anyhow!("failed to load ship data from {}: {}", path.display(), e)
            });
        }
    }

    // Check fixture path for tests
    #[cfg(debug_assertions)]
    {
        let fixture =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/ship_data.csv");
        if fixture.exists() {
            return ShipCatalog::from_path(&fixture).map_err(|e| {
                anyhow::anyhow!("failed to load ship data from {}: {}", fixture.display(), e)
            });
        }
    }

    Err(anyhow::anyhow!(
        "ship_data.csv not found; set EVEFRONTIER_SHIP_DATA or place file next to dataset"
    ))
}

// =============================================================================
// Handler functions
// =============================================================================

/// Handle the `scout gates` subcommand.
///
/// Lists all systems connected by jump gates to the specified system.
pub fn handle_scout_gates(
    args: &ScoutGatesArgs,
    format: OutputFormat,
    data_dir: Option<&std::path::Path>,
) -> Result<()> {
    // Load dataset
    let paths = tokio::task::block_in_place(|| ensure_dataset(data_dir, DatasetRelease::latest()))
        .context("failed to locate or download the EVE Frontier dataset")?;

    let starmap = load_starmap(&paths.database)
        .with_context(|| format!("failed to load dataset from {}", paths.database.display()))?;

    // Resolve system by name
    let system_id = match starmap.system_id_by_name(&args.system) {
        Some(id) => id,
        None => {
            let suggestions = starmap.fuzzy_system_matches(&args.system, 5);
            return Err(anyhow::anyhow!(format_unknown_system_error(
                &args.system,
                &suggestions
            )));
        }
    };

    // Get gate-connected neighbors from adjacency list with full metadata
    let include_ccp = args.include_ccp_systems;
    let neighbors: Vec<GateNeighbor> = starmap
        .adjacency
        .get(&system_id)
        .map(|ids| {
            let mut neighbors: Vec<GateNeighbor> = ids
                .iter()
                .filter_map(|&id| {
                    let name = starmap.system_name(id)?;
                    // Filter out CCP developer/staging systems unless explicitly included
                    if !include_ccp && is_ccp_system(name) {
                        return None;
                    }
                    let system = starmap.systems.get(&id);
                    let min_temp_k = system.and_then(|s| s.metadata.min_external_temp);
                    let planet_count = system.and_then(|s| s.metadata.planet_count);
                    let moon_count = system.and_then(|s| s.metadata.moon_count);
                    Some(GateNeighbor {
                        name: name.to_string(),
                        id,
                        min_temp_k,
                        planet_count,
                        moon_count,
                    })
                })
                .collect();
            // Sort alphabetically by name for consistent output
            neighbors.sort_by(|a, b| a.name.cmp(&b.name));
            neighbors
        })
        .unwrap_or_default();

    let result = ScoutGatesResult {
        system: args.system.clone(),
        system_id,
        count: neighbors.len(),
        neighbors,
    };

    // Format and print output
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        OutputFormat::Enhanced => {
            let palette = ColorPalette::detect();
            print!("{}", format_scout_gates_enhanced(&result, &palette));
        }
        OutputFormat::Text => {
            print!("{}", format_scout_gates_text(&result, true));
        }
        OutputFormat::Rich => {
            // Rich uses text format with temperatures shown
            print!("{}", format_scout_gates_text(&result, true));
        }
        OutputFormat::Emoji => {
            print!("{}", format_scout_gates_emoji(&result, true));
        }
        OutputFormat::Note => {
            print!("{}", format_scout_gates_note(&result));
        }
        OutputFormat::Basic => {
            print!("{}", format_scout_gates_basic(&result));
        }
    }

    Ok(())
}

/// Handle the `scout range` subcommand.
///
/// Finds systems within a spatial radius of the specified system,
/// optionally filtered by temperature and limited to a maximum count.
pub fn handle_scout_range(
    args: &ScoutRangeArgs,
    format: OutputFormat,
    data_dir: Option<&std::path::Path>,
) -> Result<()> {
    // Additional runtime validation for the limit range; clap should also enforce this via its value parser.
    if args.limit < 1 || args.limit > 100 {
        return Err(anyhow::anyhow!("limit must be between 1 and 100"));
    }

    // Validate positive radius if specified
    if let Some(r) = args.radius {
        if r <= 0.0 {
            return Err(anyhow::anyhow!("radius must be a positive number"));
        }
    }

    // Validate positive temperature if specified
    if let Some(t) = args.max_temp {
        if t <= 0.0 {
            return Err(anyhow::anyhow!("max-temp must be a positive number"));
        }
    }

    // Determine whether the user provided any scout-specific options; if not, we're in
    // a zero-config invocation and may apply friendly defaults (like default ship).
    let user_provided_options = args.radius.is_some()
        || args.max_temp.is_some()
        || args.ship.is_some()
        || args.fuel_quality != 10.0
        || args.cargo_mass != 0.0
        || args.fuel_load.is_some()
        || args.limit != 10; // Default limit

    // Determine the effective ship name (support 'None' to explicitly disable ship-based planning).
    // Only inject a default ship when the user did not provide other options (zero-config case).
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

    // Load dataset
    let paths = tokio::task::block_in_place(|| ensure_dataset(data_dir, DatasetRelease::latest()))
        .context("failed to locate or download the EVE Frontier dataset")?;

    let starmap = load_starmap(&paths.database)
        .with_context(|| format!("failed to load dataset from {}", paths.database.display()))?;

    // Load spatial index (auto-build if missing with warning)
    let spatial_index = match try_load_spatial_index(&paths.database) {
        Some(index) => index,
        None => {
            eprintln!("warning: Spatial index not found. Building on-demand...");
            evefrontier_lib::SpatialIndex::build(&starmap)
        }
    };

    // Resolve system by name
    let system_id = match starmap.system_id_by_name(&args.system) {
        Some(id) => id,
        None => {
            let suggestions = starmap.fuzzy_system_matches(&args.system, 5);
            return Err(anyhow::anyhow!(format_unknown_system_error(
                &args.system,
                &suggestions
            )));
        }
    };

    // Get the system's position
    let system = starmap.systems.get(&system_id).ok_or_else(|| {
        anyhow::anyhow!(
            "System {} found but not in starmap (internal error)",
            system_id
        )
    })?;

    let position = match system.position {
        Some(pos) => [pos.x, pos.y, pos.z],
        None => {
            return Err(anyhow::anyhow!(
                "System '{}' has no spatial coordinates",
                args.system
            ));
        }
    };

    // Build the query
    // Request more results than needed to account for CCP system filtering
    let extra_buffer = if args.include_ccp_systems { 0 } else { 50 };
    let query = NeighbourQuery {
        k: args.limit + 1 + extra_buffer, // +1 to exclude the origin system, +buffer for filtering
        radius: args.radius,
        max_temperature: args.max_temp,
    };

    // Find nearby systems
    let results = spatial_index.nearest_filtered(position, &query);

    // Convert to intermediate format with positions for nearest-neighbor ordering
    let include_ccp = args.include_ccp_systems;
    let systems_with_positions: Vec<SystemWithPosition> = results
        .into_iter()
        .filter(|(id, _)| *id != system_id)
        .filter_map(|(id, distance)| {
            let name = starmap.system_name(id)?;
            // Filter out CCP developer/staging systems unless explicitly included
            if !include_ccp && is_ccp_system(name) {
                return None;
            }
            let sys = starmap.systems.get(&id)?;
            let pos = sys.position?;
            let min_temp_k = sys.metadata.min_external_temp;
            let planet_count = sys.metadata.planet_count;
            let moon_count = sys.metadata.moon_count;
            Some(SystemWithPosition {
                neighbor: RangeNeighbor {
                    name: name.to_string(),
                    id,
                    distance_ly: distance,
                    min_temp_k,
                    planet_count,
                    moon_count,
                    hop_fuel: None,
                    cumulative_fuel: None,
                    remaining_fuel: None,
                    hop_heat: None,
                    cumulative_heat: None,
                    cooldown_seconds: None,
                    fuel_warning: None,
                    heat_warning: None,
                },
                position: [pos.x, pos.y, pos.z],
            })
        })
        .take(args.limit)
        .collect();

    // Build result based on whether ship is specified (explicit or default)
    let result = if let Some(ref ship_name) = effective_ship_name {
        // Load ship catalog - handle errors differently for explicit vs implicit ship
        let ship_catalog_result = load_ship_catalog(&paths);

        let ship_and_catalog = match ship_catalog_result {
            Ok(catalog) => {
                match catalog.get(ship_name) {
                    Some(ship) => Some((ship.clone(), catalog)),
                    None => {
                        // Ship not found in catalog
                        let available = catalog.ship_names();
                        let suggestions: Vec<_> = available
                            .iter()
                            .filter(|n| n.to_lowercase().contains(&ship_name.to_lowercase()))
                            .take(5)
                            .cloned()
                            .collect();
                        let err = if suggestions.is_empty() {
                            anyhow::anyhow!(
                                "Unknown ship '{}'. Available ships: {}",
                                ship_name,
                                available.join(", ")
                            )
                        } else {
                            anyhow::anyhow!(
                                "Unknown ship '{}'. Did you mean one of: {}?",
                                ship_name,
                                suggestions.join(", ")
                            )
                        };

                        if args.ship.is_some() {
                            // User explicitly requested this ship - error
                            return Err(err);
                        } else {
                            // Implicit default ship not found - warn and continue without
                            eprintln!("Warning: {}. Proceeding without ship projections.", err);
                            None
                        }
                    }
                }
            }
            Err(e) => {
                if args.ship.is_some() {
                    // User explicitly requested a ship - this is an error
                    return Err(e).context("failed to load ship catalog");
                } else {
                    // Implicit default ship couldn't be loaded - warn and continue without
                    eprintln!(
                        "Warning: failed to load ship data: {}. Proceeding without ship projections.",
                        e
                    );
                    None
                }
            }
        };

        if let Some((ship, _catalog)) = ship_and_catalog {
            // Create ship loadout (validates fuel_load and cargo_mass)
            let fuel_load = args.fuel_load.unwrap_or(ship.fuel_capacity);
            let _loadout = ShipLoadout::new(&ship, fuel_load, args.cargo_mass)
                .map_err(|e| anyhow::anyhow!("Invalid loadout: {}", e))?;

            // Create fuel config
            let fuel_config = FuelConfig {
                quality: args.fuel_quality,
                dynamic_mass: true, // Always use dynamic mass for scout routes
            };

            // Apply nearest-neighbor ordering
            let mut ordered_systems = nearest_neighbor_order(position, systems_with_positions);
            let total_hops = ordered_systems.len();

            // Pre-collect ambient temperatures for lookback during iteration
            let ambient_temps: Vec<Option<f64>> =
                ordered_systems.iter().map(|s| s.min_temp_k).collect();
            let origin_ambient = system.metadata.min_external_temp;

            // Calculate fuel and heat projections for each hop
            let mut cumulative_fuel = 0.0;
            let mut remaining_fuel = fuel_load;
            let mut total_distance = 0.0;
            let mut last_residual_heat = evefrontier_lib::HEAT_NOMINAL;
            let heat_cfg = HeatConfig::default();

            for (hop_index, sys) in ordered_systems.iter_mut().enumerate() {
                let hop_distance = sys.distance_ly;
                total_distance += hop_distance;

                // Calculate fuel cost using current mass (dynamic mass mode)
                let current_mass = ship.base_mass_kg
                    + (remaining_fuel * evefrontier_lib::FUEL_MASS_PER_UNIT_KG)
                    + args.cargo_mass;
                let hop_fuel = evefrontier_lib::calculate_jump_fuel_cost(
                    current_mass,
                    hop_distance,
                    &fuel_config,
                )
                .unwrap_or(0.0);

                // Track cumulative fuel before projection (cumulative always increases by hop_fuel)
                cumulative_fuel += hop_fuel;

                // Use shared helper to detect refuel and update remaining fuel
                let (projection, new_remaining) = evefrontier_lib::project_fuel_for_hop(
                    hop_fuel,
                    cumulative_fuel,
                    remaining_fuel,
                    fuel_load,
                );

                remaining_fuel = new_remaining;
                sys.fuel_warning = projection.warning.clone();

                // Calculate heat using shared helper to keep DRY and match route semantics
                let prev_ambient = if hop_index > 0 {
                    ambient_temps.get(hop_index - 1).copied().flatten()
                } else {
                    origin_ambient
                };
                let is_goal = hop_index + 1 == total_hops;
                let next_is_gate = false; // scout range visits are jumps, not gates
                let proj = evefrontier_lib::ship::project_heat_for_jump(
                    evefrontier_lib::ship::HeatProjectionParams {
                        mass: current_mass,
                        specific_heat: ship.specific_heat,
                        distance_ly: hop_distance,
                        hull_mass_kg: ship.base_mass_kg,
                        calibration_constant: heat_cfg.calibration_constant,
                        prev_ambient,
                        current_min_external_temp: sys.min_temp_k,
                        is_goal,
                        next_is_gate,
                    },
                )
                .map_err(|e| anyhow::anyhow!("heat projection failed: {}", e))?;

                if let Some(w) = proj.warning.clone() {
                    sys.heat_warning = Some(w);
                }
                if let Some(wait) = proj.wait_time_seconds {
                    sys.cooldown_seconds = Some(wait);
                }

                sys.hop_fuel = Some(hop_fuel);
                sys.cumulative_fuel = Some(cumulative_fuel);
                sys.remaining_fuel = Some(remaining_fuel);
                sys.hop_heat = Some(proj.hop_heat);
                sys.cumulative_heat = proj.residual_heat; // align with route: store residual
                if let Some(r) = proj.residual_heat {
                    last_residual_heat = r;
                }
            }

            // Sum all cooldown times for total wait time
            let total_wait_time_seconds: f64 = ordered_systems
                .iter()
                .filter_map(|s| s.cooldown_seconds)
                .sum();

            ScoutRangeResult {
                system: args.system.clone(),
                system_id,
                query: RangeQueryParams {
                    limit: args.limit,
                    radius: args.radius,
                    max_temperature: args.max_temp,
                },
                ship: Some(ShipInfo {
                    name: ship.name.clone(),
                    fuel_capacity: ship.fuel_capacity,
                    fuel_quality: args.fuel_quality,
                }),
                count: ordered_systems.len(),
                total_distance_ly: Some(total_distance),
                total_fuel: Some(cumulative_fuel),
                final_heat: Some(last_residual_heat), // Match route: final residual at destination
                total_wait_time_seconds: if total_wait_time_seconds > 0.0 {
                    Some(total_wait_time_seconds)
                } else {
                    None
                },
                systems: ordered_systems,
            }
        } else {
            // Ship loading failed for implicit default - fall back to no-ship behavior
            let systems: Vec<RangeNeighbor> = systems_with_positions
                .into_iter()
                .map(|swp| swp.neighbor)
                .collect();

            ScoutRangeResult {
                system: args.system.clone(),
                system_id,
                query: RangeQueryParams {
                    limit: args.limit,
                    radius: args.radius,
                    max_temperature: args.max_temp,
                },
                ship: None,
                count: systems.len(),
                total_distance_ly: None,
                total_fuel: None,
                final_heat: None,
                total_wait_time_seconds: None,
                systems,
            }
        }
    } else {
        // No ship - original behavior (sorted by distance from origin)
        let systems: Vec<RangeNeighbor> = systems_with_positions
            .into_iter()
            .map(|swp| swp.neighbor)
            .collect();

        ScoutRangeResult {
            system: args.system.clone(),
            system_id,
            query: RangeQueryParams {
                limit: args.limit,
                radius: args.radius,
                max_temperature: args.max_temp,
            },
            ship: None,
            count: systems.len(),
            total_distance_ly: None,
            total_fuel: None,
            final_heat: None,
            total_wait_time_seconds: None,
            systems,
        }
    };

    // Format and print output
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        OutputFormat::Enhanced => {
            let palette = ColorPalette::detect();
            print!("{}", format_scout_range_enhanced(&result, &palette));
        }
        OutputFormat::Text => {
            print!("{}", format_scout_range_text(&result, true));
        }
        OutputFormat::Rich => {
            // Rich uses text format with temperatures shown
            print!("{}", format_scout_range_text(&result, true));
        }
        OutputFormat::Emoji => {
            print!("{}", format_scout_range_emoji(&result, true));
        }
        OutputFormat::Note => {
            print!("{}", format_scout_range_note(&result));
        }
        OutputFormat::Basic => {
            print!("{}", format_scout_range_basic(&result));
        }
    }

    Ok(())
}

/// Format error message for unknown system with fuzzy suggestions.
fn format_unknown_system_error(name: &str, suggestions: &[String]) -> String {
    let mut message = format!("Unknown system '{}'.", name);
    if !suggestions.is_empty() {
        let formatted = if suggestions.len() == 1 {
            let suggestion = suggestions.first().expect("len checked above");
            format!("Did you mean '{}'?", suggestion)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ccp_system_ad_pattern() {
        // Valid AD### patterns (must have at least one digit after AD)
        assert!(is_ccp_system("AD0"));
        assert!(is_ccp_system("AD1"));
        assert!(is_ccp_system("AD035"));
        assert!(is_ccp_system("AD134"));
        assert!(is_ccp_system("AD000"));
        assert!(is_ccp_system("AD999"));
        assert!(is_ccp_system("AD12345")); // More than 3 digits

        // Invalid AD patterns
        assert!(!is_ccp_system("AD")); // No digits after AD
        assert!(!is_ccp_system("ADXYZ")); // Letters after AD
        assert!(!is_ccp_system("ad035")); // Lowercase
        assert!(!is_ccp_system("AD03X")); // Mixed letters and digits
    }

    #[test]
    fn test_is_ccp_system_v_pattern() {
        // Valid V-### patterns (must have at least one digit after V-)
        assert!(is_ccp_system("V-0"));
        assert!(is_ccp_system("V-1"));
        assert!(is_ccp_system("V-001"));
        assert!(is_ccp_system("V-123"));
        assert!(is_ccp_system("V-999"));
        assert!(is_ccp_system("V-12345")); // More than 3 digits

        // Invalid V- patterns
        assert!(!is_ccp_system("V-")); // No digits after V-
        assert!(!is_ccp_system("V-ABC")); // Letters after V-
        assert!(!is_ccp_system("v-001")); // Lowercase
        assert!(!is_ccp_system("V-0X1")); // Mixed letters and digits
    }

    #[test]
    fn test_is_ccp_system_player_systems() {
        // Regular player systems should not be CCP systems
        assert!(!is_ccp_system("Nod"));
        assert!(!is_ccp_system("Brana"));
        assert!(!is_ccp_system("H:2L2S"));
        assert!(!is_ccp_system("D:2NAS"));
        assert!(!is_ccp_system("G:3OA0"));
        assert!(!is_ccp_system("J:35IA"));
        assert!(!is_ccp_system("Y:3R7E"));
        assert!(!is_ccp_system("E1J-M5G"));
        assert!(!is_ccp_system("P:STK3"));
        assert!(!is_ccp_system("A:ABC1"));
        assert!(!is_ccp_system("ADVL-something")); // Starts with AD but has V after
    }
}
