//! MCP tool implementations for route planning and system queries
//!
//! This module defines the four main tools exposed by the MCP server:
//! - route_plan: Plan a route between two systems with constraints
//! - system_info: Get detailed information about a single system
//! - systems_nearby: Find systems within a spatial radius
//! - gates_from: Get gate-connected neighbors of a system

use crate::types::*;
use crate::Error;
use tracing::{debug, info};

/// Route planning tool handler
///
/// This tool accepts origin and destination system names and returns
/// a step-by-step route with optional constraint filtering.
pub struct RoutePlanTool;

impl RoutePlanTool {
    /// Handle a route planning request
    ///
    /// # Arguments
    ///
    /// * `input` - RoutePlanInput with origin, destination, and optional constraints
    ///
    /// # Returns
    ///
    /// RoutePlanOutput with either route details or error information
    pub async fn execute(input: RoutePlanInput) -> crate::Result<RoutePlanOutput> {
        info!(
            "Route planning: {} → {} (algorithm: {:?})",
            input.origin, input.destination, input.algorithm
        );

        // Validate inputs
        Self::validate_input(&input)?;

        // TODO: Phase 3 - Integrate with evefrontier-lib
        // 1. Load starmap via evefrontier-lib
        // 2. Fuzzy match origin and destination system names
        // 3. Call appropriate routing algorithm (bfs, dijkstra, or a-star)
        // 4. Apply constraints (max_temperature, avoid_systems, avoid_gates)
        // 5. Construct RoutePlanOutput with route details

        // Stub response for now (Phase 3+)
        Ok(RoutePlanOutput {
            success: false,
            summary: "Route planning not yet implemented".to_string(),
            route: None,
            error: Some(RouteError {
                code: "NOT_IMPLEMENTED".to_string(),
                message: "Route planning tool implementation pending (Phase 3)".to_string(),
                suggestions: vec![],
            }),
        })
    }

    /// Validate route planning input
    fn validate_input(input: &RoutePlanInput) -> crate::Result<()> {
        if input.origin.is_empty() {
            return Err(Error::invalid_param("origin", "Cannot be empty"));
        }

        if input.destination.is_empty() {
            return Err(Error::invalid_param("destination", "Cannot be empty"));
        }

        if input.origin == input.destination {
            return Err(Error::invalid_param(
                "destination",
                "Destination must be different from origin",
            ));
        }

        // Validate algorithm if provided
        if let Some(algo) = &input.algorithm {
            match algo.as_str() {
                "bfs" | "dijkstra" | "a-star" => {}
                _ => {
                    return Err(Error::invalid_param(
                        "algorithm",
                        format!("Unknown algorithm '{}'. Valid: bfs, dijkstra, a-star", algo),
                    ))
                }
            }
        }

        // Validate max_jump if provided
        if let Some(max_jump) = input.max_jump {
            if max_jump <= 0.0 {
                return Err(Error::invalid_param("max_jump", "Must be positive"));
            }
        }

        // Validate max_temperature if provided
        if let Some(max_temp) = input.max_temperature {
            if max_temp <= 0.0 {
                return Err(Error::invalid_param(
                    "max_temperature",
                    "Must be positive (Kelvin)",
                ));
            }
        }

        Ok(())
    }
}

/// System information tool handler
///
/// This tool returns metadata about a single system including coordinates,
/// temperature range, planet/moon count, and connected gates.
pub struct SystemInfoTool;

impl SystemInfoTool {
    /// Handle a system information request
    ///
    /// TODO: Implement in Phase 4
    pub async fn execute(input: SystemInfoInput) -> crate::Result<SystemInfoOutput> {
        debug!("Querying system info: {}", input.system_name);

        // TODO: Phase 4
        // 1. Fuzzy match system name
        // 2. Query database for system metadata
        // 3. Construct SystemInfoOutput

        Ok(SystemInfoOutput {
            found: false,
            system: None,
            error: Some(SystemError {
                code: "NOT_IMPLEMENTED".to_string(),
                message: "System info tool implementation pending (Phase 4)".to_string(),
                suggestions: vec![],
            }),
        })
    }
}

/// Systems nearby tool handler
///
/// This tool uses the spatial index to find systems within a given
/// radius (light-years) and optional temperature filter.
pub struct SystemsNearbyTool;

impl SystemsNearbyTool {
    /// Handle a nearby systems query
    ///
    /// TODO: Implement in Phase 4
    pub async fn execute(input: SystemsNearbyInput) -> crate::Result<SystemsNearbyOutput> {
        debug!(
            "Finding systems near {} within {} ly",
            input.system_name, input.radius_ly
        );

        // TODO: Phase 4
        // 1. Fuzzy match center system
        // 2. Load or auto-build spatial index
        // 3. Query nearby systems within radius
        // 4. Apply temperature filter if provided
        // 5. Construct SystemsNearbyOutput

        Ok(SystemsNearbyOutput {
            center_system: input.system_name,
            radius_ly: input.radius_ly,
            count: 0,
            systems: vec![],
        })
    }
}

/// Gates from tool handler
///
/// This tool returns the list of systems directly connected to a given
/// system via jump gates.
pub struct GatesFromTool;

impl GatesFromTool {
    /// Handle a gates query
    ///
    /// TODO: Implement in Phase 4
    pub async fn execute(input: GatesFromInput) -> crate::Result<GatesFromOutput> {
        debug!("Querying gates from: {}", input.system_name);

        // TODO: Phase 4
        // 1. Fuzzy match system name
        // 2. Query database for gate connections
        // 3. Construct GatesFromOutput

        Ok(GatesFromOutput {
            system_name: input.system_name,
            gate_count: 0,
            gates: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_stubs_defined() {
        // Verify tool structures are properly defined
        let _route_tool = RoutePlanTool;
        let _info_tool = SystemInfoTool;
        let _nearby_tool = SystemsNearbyTool;
        let _gates_tool = GatesFromTool;
    }

    #[tokio::test]
    async fn test_route_plan_validation_empty_origin() {
        let input = RoutePlanInput {
            origin: "".to_string(),
            destination: "Brana".to_string(),
            algorithm: None,
            max_jump: None,
            max_temperature: None,
            avoid_systems: vec![],
            avoid_gates: false,
        };

        let result = RoutePlanTool::execute(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_route_plan_validation_same_origin_destination() {
        let input = RoutePlanInput {
            origin: "Nod".to_string(),
            destination: "Nod".to_string(),
            algorithm: None,
            max_jump: None,
            max_temperature: None,
            avoid_systems: vec![],
            avoid_gates: false,
        };

        let result = RoutePlanTool::execute(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_route_plan_validation_invalid_algorithm() {
        let input = RoutePlanInput {
            origin: "Nod".to_string(),
            destination: "Brana".to_string(),
            algorithm: Some("invalid".to_string()),
            max_jump: None,
            max_temperature: None,
            avoid_systems: vec![],
            avoid_gates: false,
        };

        let result = RoutePlanTool::execute(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_route_plan_validation_negative_max_jump() {
        let input = RoutePlanInput {
            origin: "Nod".to_string(),
            destination: "Brana".to_string(),
            algorithm: None,
            max_jump: Some(-5.0),
            max_temperature: None,
            avoid_systems: vec![],
            avoid_gates: false,
        };

        let result = RoutePlanTool::execute(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_route_plan_execution_valid_input() {
        let input = RoutePlanInput {
            origin: "Nod".to_string(),
            destination: "Brana".to_string(),
            algorithm: Some("a-star".to_string()),
            max_jump: None,
            max_temperature: Some(500.0),
            avoid_systems: vec![],
            avoid_gates: false,
        };

        let result = RoutePlanTool::execute(input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.success); // Stub returns not implemented
    }
}
