//! Scout subcommand handlers for gate neighbors and spatial range queries.
//!
//! This module provides the implementation for:
//! - `scout gates <SYSTEM>` — list gate-connected neighbors
//! - `scout range <SYSTEM>` — find systems within spatial range

use anyhow::{Context, Result};
use evefrontier_lib::{
    ensure_dataset, load_starmap, try_load_spatial_index, DatasetRelease, NeighbourQuery,
};

use crate::output_helpers::{
    format_scout_gates_basic, format_scout_gates_emoji, format_scout_gates_enhanced,
    format_scout_gates_note, format_scout_gates_text, format_scout_range_basic,
    format_scout_range_emoji, format_scout_range_enhanced, format_scout_range_note,
    format_scout_range_text, GateNeighbor, RangeNeighbor, RangeQueryParams, ScoutGatesResult,
    ScoutRangeResult,
};
use crate::terminal::ColorPalette;
use crate::{OutputFormat, ScoutGatesArgs, ScoutRangeArgs};

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

    // Convert to response, excluding the origin system and CCP systems, with full metadata
    let include_ccp = args.include_ccp_systems;
    let systems: Vec<RangeNeighbor> = results
        .into_iter()
        .filter(|(id, _)| *id != system_id)
        .filter_map(|(id, distance)| {
            let name = starmap.system_name(id)?;
            // Filter out CCP developer/staging systems unless explicitly included
            if !include_ccp && is_ccp_system(name) {
                return None;
            }
            let sys = starmap.systems.get(&id);
            let min_temp_k = sys.and_then(|s| s.metadata.min_external_temp);
            let planet_count = sys.and_then(|s| s.metadata.planet_count);
            let moon_count = sys.and_then(|s| s.metadata.moon_count);
            Some(RangeNeighbor {
                name: name.to_string(),
                id,
                distance_ly: distance,
                min_temp_k,
                planet_count,
                moon_count,
            })
        })
        .take(args.limit)
        .collect();

    let result = ScoutRangeResult {
        system: args.system.clone(),
        system_id,
        query: RangeQueryParams {
            limit: args.limit,
            radius: args.radius,
            max_temperature: args.max_temp,
        },
        count: systems.len(),
        systems,
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
