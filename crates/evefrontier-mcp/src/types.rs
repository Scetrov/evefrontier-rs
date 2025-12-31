//! Type definitions for MCP tool inputs and outputs
//!
//! This module defines all the serializable request and response types
//! for MCP tools, with JSON Schema generation for automatic validation.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// TOOL INPUTS
// ============================================================================

/// Input for the route_plan tool
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RoutePlanInput {
    /// Starting system name (required)
    pub origin: String,

    /// Goal system name (required)
    pub destination: String,

    /// Routing algorithm: "bfs", "dijkstra", or "a-star" (default: "a-star")
    #[serde(default)]
    pub algorithm: Option<String>,

    /// Maximum jump distance in light-years (optional)
    pub max_jump: Option<f64>,

    /// Maximum system temperature in Kelvin (optional)
    pub max_temperature: Option<f64>,

    /// System names to exclude from route (optional)
    #[serde(default)]
    pub avoid_systems: Vec<String>,

    /// Use spatial-only routing, ignore jump gates (default: false)
    #[serde(default)]
    pub avoid_gates: bool,
}

/// Input for the system_info tool
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SystemInfoInput {
    /// System name to query (supports fuzzy matching)
    pub system_name: String,
}

/// Input for the systems_nearby tool
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SystemsNearbyInput {
    /// Center system name (required)
    pub system_name: String,

    /// Search radius in light-years (required)
    pub radius_ly: f64,

    /// Maximum system temperature in Kelvin (optional)
    pub max_temperature: Option<f64>,

    /// Maximum number of results (default: 20, max: 100)
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    20
}

/// Input for the gates_from tool
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GatesFromInput {
    /// System name to query gates from
    pub system_name: String,
}

// ============================================================================
// TOOL OUTPUTS
// ============================================================================

/// Output from the route_plan tool
#[derive(Debug, Clone, Serialize)]
pub struct RoutePlanOutput {
    /// Whether a route was found
    pub success: bool,

    /// Human-readable summary
    pub summary: String,

    /// Detailed route information (if found)
    pub route: Option<RouteDetails>,

    /// Error details (if not found)
    pub error: Option<RouteError>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RouteDetails {
    /// Algorithm used
    pub algorithm: String,

    /// Starting system
    pub origin: SystemSummary,

    /// Goal system
    pub destination: SystemSummary,

    /// Number of hops
    pub hop_count: usize,

    /// Total distance in light-years (for spatial routes)
    pub total_distance_ly: Option<f64>,

    /// Number of gate jumps
    pub gate_jumps: usize,

    /// Number of spatial jumps
    pub spatial_jumps: usize,

    /// Ordered list of systems in the route
    pub waypoints: Vec<Waypoint>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Waypoint {
    pub system_name: String,
    pub system_id: u32,
    pub position: Position3D,
    pub min_temperature_k: f64,
    /// Type of jump to reach this system from previous
    pub edge_type: Option<String>, // "gate" or "spatial"
    /// Distance from previous system in light-years
    pub distance_ly: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RouteError {
    pub code: String,
    pub message: String,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemSummary {
    pub name: String,
    pub system_id: u32,
    pub position: Position3D,
    pub min_temperature_k: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Position3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Output from the system_info tool
#[derive(Debug, Clone, Serialize)]
pub struct SystemInfoOutput {
    pub found: bool,
    pub system: Option<SystemDetails>,
    pub error: Option<SystemError>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemDetails {
    pub system_id: u32,
    pub name: String,
    pub position: Position3D,
    pub min_external_temperature_k: f64,
    pub planet_count: usize,
    pub moon_count: usize,
    pub connected_gates: Vec<GateConnection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GateConnection {
    pub destination_system: String,
    pub destination_id: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemError {
    pub code: String,
    pub message: String,
    pub suggestions: Vec<String>,
}

/// Output from the systems_nearby tool
#[derive(Debug, Clone, Serialize)]
pub struct SystemsNearbyOutput {
    pub center_system: String,
    pub radius_ly: f64,
    pub count: usize,
    pub systems: Vec<NearbySystem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NearbySystem {
    pub name: String,
    pub system_id: u32,
    pub distance_ly: f64,
    pub min_temperature_k: f64,
}

/// Output from the gates_from tool
#[derive(Debug, Clone, Serialize)]
pub struct GatesFromOutput {
    pub system_name: String,
    pub gate_count: usize,
    pub gates: Vec<GateConnection>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_plan_input_deserialization() {
        let json = r#"{
            "origin": "Nod",
            "destination": "Brana",
            "algorithm": "a-star",
            "max_temperature": 500.0,
            "avoid_systems": [],
            "avoid_gates": false
        }"#;

        let input: RoutePlanInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.origin, "Nod");
        assert_eq!(input.destination, "Brana");
        assert_eq!(input.algorithm, Some("a-star".to_string()));
    }

    #[test]
    fn test_route_plan_input_minimal() {
        let json = r#"{
            "origin": "Nod",
            "destination": "Brana"
        }"#;

        let input: RoutePlanInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.origin, "Nod");
        assert_eq!(input.algorithm, None);
        assert!(input.avoid_systems.is_empty());
        assert!(!input.avoid_gates);
    }

    #[test]
    fn test_systems_nearby_input_default_limit() {
        let json = r#"{
            "system_name": "Brana",
            "radius_ly": 50.0
        }"#;

        let input: SystemsNearbyInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.limit, 20);
    }

    #[test]
    fn test_route_plan_output_serialization() {
        let output = RoutePlanOutput {
            success: true,
            summary: "Found route: Nod → Brana (2 hops)".to_string(),
            route: Some(RouteDetails {
                algorithm: "a-star".to_string(),
                origin: SystemSummary {
                    name: "Nod".to_string(),
                    system_id: 1,
                    position: Position3D { x: 0.0, y: 0.0, z: 0.0 },
                    min_temperature_k: 300.0,
                },
                destination: SystemSummary {
                    name: "Brana".to_string(),
                    system_id: 2,
                    position: Position3D { x: 10.0, y: 0.0, z: 0.0 },
                    min_temperature_k: 300.0,
                },
                hop_count: 2,
                total_distance_ly: Some(10.0),
                gate_jumps: 2,
                spatial_jumps: 0,
                waypoints: vec![],
            }),
            error: None,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("Found route"));
        assert!(json.contains("Nod"));
    }
}
