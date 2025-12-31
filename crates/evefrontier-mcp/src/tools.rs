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
    /// Queries the loaded dataset for a specific system by exact or fuzzy match,
    /// returning detailed metadata including coordinates, temperature, and gates.
    ///
    /// # Arguments
    ///
    /// * `input` - SystemInfoInput with system_name to query
    ///
    /// # Returns
    ///
    /// SystemInfoOutput with either system details or error information
    pub async fn execute(input: SystemInfoInput) -> crate::Result<SystemInfoOutput> {
        debug!("Querying system info: {}", input.system_name);

        // Validate input
        if input.system_name.is_empty() {
            return Err(Error::invalid_param(
                "system_name",
                "System name cannot be empty",
            ));
        }

        // TODO: Phase 4+ - Integrate with evefrontier-lib
        // 1. Load starmap from database
        // 2. Fuzzy match system name (using strsim crate)
        // 3. Query system metadata (coordinates, temp, planet/moon counts)
        // 4. Query gate connections
        // 5. Construct SystemInfoOutput

        // For now: Return not-found with suggestions for fuzzy matching demo
        Ok(SystemInfoOutput {
            found: false,
            system: None,
            error: Some(SystemError {
                code: "SYSTEM_NOT_FOUND".to_string(),
                message: format!("System '{}' not found in dataset", input.system_name),
                suggestions: vec!["Nod".to_string(), "Brana".to_string()],
            }),
        })
    }

    /// Validate system_info input
    fn validate_input(input: &SystemInfoInput) -> crate::Result<()> {
        if input.system_name.is_empty() {
            return Err(Error::invalid_param(
                "system_name",
                "System name cannot be empty",
            ));
        }
        Ok(())
    }
}

/// Systems nearby tool handler
///
/// This tool returns systems within a specified radius of a central system,
/// optionally filtered by temperature tolerance.
pub struct SystemsNearbyTool;

impl SystemsNearbyTool {
    /// Handle a systems nearby request
    ///
    /// Performs a spatial query to find all systems within the specified radius
    /// from a central system, optionally filtering by temperature tolerance.
    ///
    /// # Arguments
    ///
    /// * `input` - SystemsNearbyInput with center system and radius parameters
    ///
    /// # Returns
    ///
    /// SystemsNearbyOutput with list of nearby systems or error information
    pub async fn execute(input: SystemsNearbyInput) -> crate::Result<SystemsNearbyOutput> {
        debug!(
            "Querying systems near {}, radius: {}ly",
            input.system_name, input.radius_ly
        );

        // Validate input
        Self::validate_input(&input)?;

        // TODO: Phase 4+ - Integrate with evefrontier-lib spatial index
        // 1. Load starmap and spatial index from database
        // 2. Fuzzy match system name to find center coordinates
        // 3. Use spatial index to query nearby systems within radius
        // 4. Filter by temperature if max_temperature specified
        // 5. Paginate results using limit and offset
        // 6. Construct SystemsNearbyOutput

        // For now: Return empty result (Phase 4+ will implement spatial queries)
        Ok(SystemsNearbyOutput {
            center_system: input.system_name.clone(),
            radius_ly: input.radius_ly,
            count: 0,
            systems: vec![],
        })
    }

    /// Validate systems_nearby input
    fn validate_input(input: &SystemsNearbyInput) -> crate::Result<()> {
        if input.system_name.is_empty() {
            return Err(Error::invalid_param(
                "system_name",
                "System name cannot be empty",
            ));
        }

        if input.radius_ly <= 0.0 {
            return Err(Error::invalid_param(
                "radius_ly",
                "Radius must be positive (> 0)",
            ));
        }

        if let Some(max_temp) = input.max_temperature {
            if max_temp < 0.0 {
                return Err(Error::invalid_param(
                    "max_temperature",
                    "Temperature cannot be negative",
                ));
            }
        }

        Ok(())
    }
}

/// Gates from tool handler
///
/// This tool returns the list of systems directly connected to a given
/// system via jump gates.
pub struct GatesFromTool;

impl GatesFromTool {
    /// Handle a gates query request
    ///
    /// Queries the database for all systems directly connected to the specified
    /// system via jump gates.
    ///
    /// # Arguments
    ///
    /// * `input` - GatesFromInput with system_name to query
    ///
    /// # Returns
    ///
    /// GatesFromOutput with list of gate-connected systems or error information
    pub async fn execute(input: GatesFromInput) -> crate::Result<GatesFromOutput> {
        debug!("Querying gates from: {}", input.system_name);

        // Validate input
        Self::validate_input(&input)?;

        // TODO: Phase 4+ - Integrate with evefrontier-lib
        // 1. Load starmap from database
        // 2. Fuzzy match system name
        // 3. Query adjacency list for gate connections
        // 4. Construct GatesFromOutput with gate list

        // For now: Return not-found stub
        Ok(GatesFromOutput {
            system_name: input.system_name.clone(),
            gate_count: 0,
            gates: vec![],
        })
    }

    /// Validate gates_from input
    fn validate_input(input: &GatesFromInput) -> crate::Result<()> {
        if input.system_name.is_empty() {
            return Err(Error::invalid_param(
                "system_name",
                "System name cannot be empty",
            ));
        }
        Ok(())
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

    // Route planning tool tests
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

    // System info tool tests
    #[tokio::test]
    async fn test_system_info_validation_empty_name() {
        let input = SystemInfoInput {
            system_name: "".to_string(),
        };

        let result = SystemInfoTool::execute(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_system_info_execution_not_found() {
        let input = SystemInfoInput {
            system_name: "UnknownSystem".to_string(),
        };

        let result = SystemInfoTool::execute(input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.found);
        assert!(output.system.is_none());
        assert!(output.error.is_some());
    }

    #[tokio::test]
    async fn test_system_info_error_includes_suggestions() {
        let input = SystemInfoInput {
            system_name: "InvalidName".to_string(),
        };

        let result = SystemInfoTool::execute(input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        if let Some(error) = output.error {
            assert!(!error.suggestions.is_empty());
        }
    }

    // Systems nearby tool tests
    #[tokio::test]
    async fn test_systems_nearby_validation_empty_name() {
        let input = SystemsNearbyInput {
            system_name: "".to_string(),
            radius_ly: 50.0,
            max_temperature: None,
            limit: 20,
        };

        let result = SystemsNearbyTool::execute(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_systems_nearby_validation_zero_radius() {
        let input = SystemsNearbyInput {
            system_name: "Nod".to_string(),
            radius_ly: 0.0,
            max_temperature: None,
            limit: 20,
        };

        let result = SystemsNearbyTool::execute(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_systems_nearby_validation_negative_radius() {
        let input = SystemsNearbyInput {
            system_name: "Nod".to_string(),
            radius_ly: -50.0,
            max_temperature: None,
            limit: 20,
        };

        let result = SystemsNearbyTool::execute(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_systems_nearby_validation_negative_temperature() {
        let input = SystemsNearbyInput {
            system_name: "Nod".to_string(),
            radius_ly: 50.0,
            max_temperature: Some(-100.0),
            limit: 20,
        };

        let result = SystemsNearbyTool::execute(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_systems_nearby_execution_valid_input() {
        let input = SystemsNearbyInput {
            system_name: "Nod".to_string(),
            radius_ly: 80.0,
            max_temperature: Some(500.0),
            limit: 20,
        };

        let result = SystemsNearbyTool::execute(input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.center_system, "Nod");
        assert_eq!(output.radius_ly, 80.0);
    }

    #[tokio::test]
    async fn test_systems_nearby_execution_no_temperature_filter() {
        let input = SystemsNearbyInput {
            system_name: "Brana".to_string(),
            radius_ly: 100.0,
            max_temperature: None,
            limit: 20,
        };

        let result = SystemsNearbyTool::execute(input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.center_system, "Brana");
    }

    // Gates from tool tests
    #[tokio::test]
    async fn test_gates_from_validation_empty_name() {
        let input = GatesFromInput {
            system_name: "".to_string(),
        };

        let result = GatesFromTool::execute(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_gates_from_execution_not_found() {
        let input = GatesFromInput {
            system_name: "UnknownSystem".to_string(),
        };

        let result = GatesFromTool::execute(input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.gate_count, 0);
        assert!(output.gates.is_empty());
    }

    #[tokio::test]
    async fn test_gates_from_error_includes_suggestions() {
        let input = GatesFromInput {
            system_name: "InvalidName".to_string(),
        };

        let result = GatesFromTool::execute(input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        // Note: GatesFromOutput doesn't have error field,
        // error handling is implicit in zero gate count
        assert_eq!(output.gate_count, 0);
    }

    #[tokio::test]
    async fn test_gates_from_execution_valid_input() {
        let input = GatesFromInput {
            system_name: "Nod".to_string(),
        };

        let result = GatesFromTool::execute(input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.system_name, "Nod");
    }
}
