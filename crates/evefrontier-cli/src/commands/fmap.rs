//! Fmap encode/decode command handlers.

use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

use evefrontier_lib::{
    decode_fmap_token, encode_fmap_token, ensure_dataset, load_starmap, DatasetRelease, Waypoint,
    WaypointType,
};

use crate::output::OutputFormat;

/// Arguments for the fmap-encode command.
#[derive(Debug, Clone)]
pub struct FmapEncodeArgs {
    /// System names to encode (comma-separated or repeated --system flags).
    pub systems: Vec<String>,
    /// Waypoint type for each system.
    pub types: Vec<String>,
    /// Output in JSON format.
    pub json: bool,
}

/// Arguments for the fmap-decode command.
#[derive(Debug, Clone)]
pub struct FmapDecodeArgs {
    /// Base64url-encoded fmap token string.
    pub token: String,
    /// Output in JSON format.
    pub json: bool,
}

/// Handle the fmap-encode subcommand.
///
/// Encodes a list of systems into an fmap URL token.
pub fn handle_fmap_encode(
    target_path: Option<&Path>,
    release: DatasetRelease,
    _format: OutputFormat,
    args: &FmapEncodeArgs,
) -> Result<()> {
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
            let paths = ensure_dataset(target_path, release)
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

/// Handle the fmap-decode subcommand.
///
/// Decodes an fmap URL token back to a list of waypoints.
pub fn handle_fmap_decode(args: &FmapDecodeArgs) -> Result<()> {
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
