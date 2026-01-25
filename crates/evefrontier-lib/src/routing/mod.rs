//! Route planning module for EVE Frontier pathfinding.
//!
//! This module provides:
//! - [`RouteAlgorithm`] - Supported routing algorithms (BFS, Dijkstra, A*)
//! - [`RouteConstraints`] - Constraints applied during route planning
//! - [`RouteRequest`] - High-level route planning request
//! - [`RoutePlan`] - Planned route result
//! - [`plan_route`] - Main entry point for computing routes
//!
//! # Strategy Pattern
//!
//! The routing module uses the Strategy pattern via the [`RoutePlanner`] trait.
//! Each algorithm (BFS, Dijkstra, A*) is encapsulated in its own planner struct,
//! allowing new algorithms to be added without modifying the core orchestration logic.
//!
//! # Example
//!
//! ```ignore
//! use evefrontier_lib::{plan_route, RouteRequest, load_starmap};
//!
//! let starmap = load_starmap("path/to/database.db")?;
//! let request = RouteRequest::bfs("Nod", "Brana");
//! let plan = plan_route(&starmap, &request)?;
//! println!("Route: {} hops", plan.hop_count());
//! ```

mod planner;

pub use planner::{select_planner, AStarPlanner, BfsPlanner, DijkstraPlanner, RoutePlanner};

use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

use serde::Serialize;

use crate::db::{Starmap, SystemId};
use crate::error::{Error, Result};
use crate::graph::{
    build_gate_graph, build_hybrid_graph_indexed, build_spatial_graph_indexed, EdgeKind, Graph,
    GraphBuildOptions, GraphMode,
};
use crate::path::PathConstraints as SearchConstraints;
use crate::spatial::SpatialIndex;

/// Supported routing algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RouteAlgorithm {
    /// Breadth-first search (unweighted graph).
    Bfs,
    /// Dijkstra's algorithm (weighted graph).
    Dijkstra,
    /// A* search (heuristic guided).
    #[default]
    #[serde(rename = "a-star")]
    AStar,
}

/// Optimization objective for route planning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RouteOptimization {
    /// Optimize for shortest distance (default behavior).
    Distance,
    /// Optimize for minimal fuel consumption (requires ship + loadout).
    #[default]
    Fuel,
}

impl fmt::Display for RouteAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            RouteAlgorithm::Bfs => "bfs",
            RouteAlgorithm::Dijkstra => "dijkstra",
            RouteAlgorithm::AStar => "a-star",
        };
        f.write_str(value)
    }
}

/// Constraints applied during route planning.
#[derive(Debug, Clone)]
pub struct RouteConstraints {
    pub max_jump: Option<f64>,
    pub avoid_systems: Vec<String>,
    pub avoid_gates: bool,
    pub max_temperature: Option<f64>,
    /// Avoid hops that would result in the engine becoming critical (requires ship/loadout).
    pub avoid_critical_state: bool,
    /// Optional ship information used when evaluating heat-based constraints.
    pub ship: Option<crate::ship::ShipAttributes>,
    pub loadout: Option<crate::ship::ShipLoadout>,
    pub heat_config: Option<crate::ship::HeatConfig>,
}

impl Default for RouteConstraints {
    fn default() -> Self {
        Self {
            max_jump: None,
            avoid_systems: Vec::new(),
            avoid_gates: false,
            max_temperature: None,
            // Sensible default: avoid critical state unless the caller disables it
            avoid_critical_state: true,
            ship: None,
            loadout: None,
            heat_config: None,
        }
    }
}

impl RouteConstraints {
    fn to_search_constraints(&self, avoided: HashSet<SystemId>) -> SearchConstraints {
        SearchConstraints {
            max_jump: self.max_jump,
            avoid_gates: self.avoid_gates,
            avoided_systems: avoided,
            max_temperature: self.max_temperature,
            avoid_critical_state: self.avoid_critical_state,
            ship: self.ship.clone(),
            loadout: self.loadout,
            heat_config: self.heat_config,
        }
    }
}

/// High-level route planning request.
#[derive(Debug, Clone)]
pub struct RouteRequest {
    pub start: String,
    pub goal: String,
    pub algorithm: RouteAlgorithm,
    pub constraints: RouteConstraints,
    /// Pre-loaded spatial index for faster graph construction.
    /// If `None`, the index will be built on demand (with a warning for large datasets).
    pub spatial_index: Option<Arc<SpatialIndex>>,
    /// Maximum spatial neighbours to consider when building the spatial/hybrid graph.
    pub max_spatial_neighbors: usize,
    /// Optimization objective used by the planner (distance or fuel).
    pub optimization: RouteOptimization,
    /// Fuel configuration used when optimizing for fuel (quality/dynamic_mass).
    pub fuel_config: crate::ship::FuelConfig,
}

impl RouteRequest {
    /// Convenience constructor for BFS routes without extra constraints.
    pub fn bfs(start: impl Into<String>, goal: impl Into<String>) -> Self {
        Self {
            start: start.into(),
            goal: goal.into(),
            algorithm: RouteAlgorithm::Bfs,
            constraints: RouteConstraints::default(),
            spatial_index: None,
            max_spatial_neighbors: crate::graph::GraphBuildOptions::default().max_spatial_neighbors,
            optimization: RouteOptimization::Distance,
            fuel_config: crate::ship::FuelConfig::default(),
        }
    }

    /// Attach a pre-loaded spatial index to the request.
    pub fn with_spatial_index(mut self, index: Arc<SpatialIndex>) -> Self {
        self.spatial_index = Some(index);
        self
    }
}

/// Planned route returned by the library.
#[derive(Debug, Clone, Serialize)]
pub struct RoutePlan {
    pub algorithm: RouteAlgorithm,
    pub start: SystemId,
    pub goal: SystemId,
    pub steps: Vec<SystemId>,
    pub gates: usize,
    pub jumps: usize,
}

impl RoutePlan {
    /// Number of hops in the route.
    pub fn hop_count(&self) -> usize {
        self.steps.len().saturating_sub(1)
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Resolve system names to IDs, returning an error for unknown systems.
fn resolve_system(starmap: &Starmap, name: &str) -> Result<SystemId> {
    starmap.system_id_by_name(name).ok_or_else(|| {
        let suggestions = starmap.fuzzy_system_matches(name, 3);
        Error::UnknownSystem {
            name: name.to_string(),
            suggestions,
        }
    })
}

/// Resolve a list of avoided system names to their IDs.
fn resolve_avoided_systems(starmap: &Starmap, avoided: &[String]) -> Result<HashSet<SystemId>> {
    let mut resolved = HashSet::new();
    for name in avoided {
        let id = resolve_system(starmap, name)?;
        resolved.insert(id);
    }
    Ok(resolved)
}

/// Check if a system meets temperature constraints.
fn system_meets_temperature(starmap: &Starmap, system: SystemId, limit: Option<f64>) -> bool {
    let Some(limit) = limit else {
        return true;
    };
    starmap
        .systems
        .get(&system)
        .and_then(|sys| sys.metadata.star_temperature)
        .map(|temperature| temperature <= limit)
        .unwrap_or(true)
}

/// Compute effective constraints including ship-based limits.
fn compute_effective_constraints(
    starmap: &Starmap,
    request: &RouteRequest,
    start_id: SystemId,
    base_constraints: &SearchConstraints,
) -> SearchConstraints {
    let mut effective = base_constraints.clone();

    if let (Some(ship), Some(_loadout)) = (&request.constraints.ship, &request.constraints.loadout)
    {
        // Heat-based maximum distance only applies when avoiding critical engine state
        if request.constraints.avoid_critical_state {
            let ambient = starmap
                .systems
                .get(&start_id)
                .and_then(|s| s.metadata.min_external_temp)
                .unwrap_or(0.0);
            let heat_cfg = request.constraints.heat_config.unwrap_or_default();

            let allowed_delta = crate::ship::HEAT_CRITICAL - ambient;
            if allowed_delta > 0.0 && heat_cfg.calibration_constant > 0.0 {
                let heat_max = allowed_delta
                    * (heat_cfg.calibration_constant * ship.base_mass_kg * ship.specific_heat)
                    / 3.0;

                effective.max_jump = match effective.max_jump {
                    Some(user) => Some(user.min(heat_max)),
                    None => Some(heat_max),
                };

                tracing::debug!(
                    "computed heat-based max_jump: {:.2} ly (ambient={:.1}K)",
                    heat_max,
                    ambient
                );
            }
        }
    }

    effective
}

/// Select the appropriate graph for the given algorithm and constraints.
fn select_graph(
    starmap: &Starmap,
    algorithm: RouteAlgorithm,
    constraints: &SearchConstraints,
    spatial_index: Option<Arc<SpatialIndex>>,
    max_spatial_neighbors: usize,
) -> Graph {
    let options = GraphBuildOptions {
        spatial_index,
        max_jump: constraints.max_jump,
        max_temperature: constraints.max_temperature,
        max_spatial_neighbors,
    };

    if constraints.avoid_gates {
        return build_spatial_graph_indexed(starmap, &options);
    }

    match algorithm {
        RouteAlgorithm::Bfs => build_gate_graph(starmap),
        RouteAlgorithm::Dijkstra | RouteAlgorithm::AStar => {
            build_hybrid_graph_indexed(starmap, &options)
        }
    }
}

/// Build a filtered adjacency list that respects search constraints.
fn build_filtered_adjacency(
    graph: &Graph,
    starmap: &Starmap,
    constraints: &SearchConstraints,
) -> std::collections::HashMap<SystemId, Vec<crate::graph::Edge>> {
    let mut filtered = std::collections::HashMap::new();
    for &sid in starmap.systems.keys() {
        let mut out = Vec::new();
        for e in graph.neighbours(sid) {
            if constraints.allows(Some(starmap), e, e.target) {
                out.push(e.clone());
            }
        }
        filtered.insert(sid, out);
    }
    filtered
}

/// Classify edges in a route as gates or spatial jumps.
fn classify_edges(graph: &Graph, steps: &[SystemId]) -> (usize, usize) {
    if steps.len() < 2 {
        return (0, 0);
    }

    match graph.mode() {
        GraphMode::Gate => (steps.len() - 1, 0),
        GraphMode::Spatial => (0, steps.len() - 1),
        GraphMode::Hybrid => {
            let mut gates = 0usize;
            let mut jumps = 0usize;
            for pair in steps.windows(2) {
                let u = pair[0];
                let v = pair[1];
                let chosen = graph
                    .neighbours(u)
                    .iter()
                    .filter(|e| e.target == v)
                    .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

                match chosen.map(|e| e.kind) {
                    Some(EdgeKind::Gate) => gates += 1,
                    Some(EdgeKind::Spatial) => jumps += 1,
                    None => gates += 1, // Fallback
                }
            }
            (gates, jumps)
        }
    }
}

/// Validate that all edges in a route are safe under the given constraints.
/// Returns an alternative route if the original contains unsafe hops.
fn validate_route_edges(
    route: &[SystemId],
    graph: &Graph,
    starmap: &Starmap,
    request: &RouteRequest,
    constraints: &SearchConstraints,
    start_id: SystemId,
    goal_id: SystemId,
) -> Result<Option<Vec<SystemId>>> {
    for pair in route.windows(2) {
        let u = pair[0];
        let v = pair[1];

        let chosen = graph
            .neighbours(u)
            .iter()
            .filter(|e| e.target == v)
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

        let Some(edge) = chosen else {
            return Err(Error::RouteNotFound {
                start: request.start.clone(),
                goal: request.goal.clone(),
            });
        };

        // Check for critical heat on spatial edges
        if edge.kind == crate::graph::EdgeKind::Spatial && request.constraints.avoid_critical_state
        {
            if let (Some(ship), Some(loadout), Some(heat_cfg)) = (
                &request.constraints.ship,
                &request.constraints.loadout,
                &request.constraints.heat_config,
            ) {
                let ambient_temp = starmap
                    .systems
                    .get(&v)
                    .and_then(|s| s.metadata.min_external_temp)
                    .unwrap_or(0.0);
                let mass = loadout.total_mass_kg(ship);

                if let Ok(energy) = crate::ship::calculate_jump_heat(
                    mass,
                    edge.distance,
                    ship.base_mass_kg,
                    heat_cfg.calibration_constant,
                ) {
                    let hop_heat = energy / (mass * ship.specific_heat);
                    let total = ambient_temp + hop_heat;

                    if total >= crate::ship::HEAT_CRITICAL {
                        // Try re-planning with filtered graph
                        return try_alternative_route(
                            graph,
                            starmap,
                            request,
                            constraints,
                            start_id,
                            goal_id,
                        );
                    }
                }
            }
        }
    }

    Ok(None) // Original route is valid
}

/// Attempt to find an alternative route using a filtered graph.
fn try_alternative_route(
    graph: &Graph,
    starmap: &Starmap,
    request: &RouteRequest,
    constraints: &SearchConstraints,
    start_id: SystemId,
    goal_id: SystemId,
) -> Result<Option<Vec<SystemId>>> {
    let filtered_adj = build_filtered_adjacency(graph, starmap, constraints);
    let filtered_graph = crate::graph::Graph::from_parts(graph.mode(), filtered_adj);

    let planner = select_planner(request);
    let alt_route = planner.find_path(
        &filtered_graph,
        Some(starmap),
        start_id,
        goal_id,
        constraints,
    );

    if alt_route.is_some() {
        Ok(alt_route)
    } else {
        Err(Error::RouteNotFound {
            start: request.start.clone(),
            goal: request.goal.clone(),
        })
    }
}

// =============================================================================
// Main Entry Point
// =============================================================================

/// Compute a route using the requested algorithm and constraints.
///
/// This is the main entry point for route planning. It:
/// 1. Resolves system names to IDs
/// 2. Validates start/goal against constraints
/// 3. Selects the appropriate planner strategy
/// 4. Builds the graph and executes pathfinding
/// 5. Validates the route for safety (heat constraints)
pub fn plan_route(starmap: &Starmap, request: &RouteRequest) -> Result<RoutePlan> {
    // Step 1: Resolve system names
    let start_id = resolve_system(starmap, &request.start)?;
    let goal_id = resolve_system(starmap, &request.goal)?;

    // Step 2: Resolve avoided systems and build base constraints
    let avoided = resolve_avoided_systems(starmap, &request.constraints.avoid_systems)?;
    let base_constraints = request.constraints.to_search_constraints(avoided.clone());

    // Step 3: Validate start/goal against constraints
    if base_constraints.avoided_systems.contains(&start_id)
        || base_constraints.avoided_systems.contains(&goal_id)
        || !system_meets_temperature(starmap, start_id, base_constraints.max_temperature)
        || !system_meets_temperature(starmap, goal_id, base_constraints.max_temperature)
    {
        return Err(Error::RouteNotFound {
            start: request.start.clone(),
            goal: request.goal.clone(),
        });
    }

    // Step 4: Compute effective constraints with ship-based limits
    let effective_constraints =
        compute_effective_constraints(starmap, request, start_id, &base_constraints);

    // Step 5: Build graph and select planner
    let graph = select_graph(
        starmap,
        request.algorithm,
        &effective_constraints,
        request.spatial_index.as_ref().cloned(),
        request.max_spatial_neighbors,
    );

    let planner = select_planner(request);

    // Step 6: Execute pathfinding
    let route = planner
        .find_path(
            &graph,
            Some(starmap),
            start_id,
            goal_id,
            &effective_constraints,
        )
        .ok_or_else(|| Error::RouteNotFound {
            start: request.start.clone(),
            goal: request.goal.clone(),
        })?;

    // Step 7: Validate route edges for safety
    if let Some(alt_route) = validate_route_edges(
        &route,
        &graph,
        starmap,
        request,
        &base_constraints,
        start_id,
        goal_id,
    )? {
        let (gates, jumps) = classify_edges(&graph, &alt_route);
        return Ok(RoutePlan {
            algorithm: request.algorithm,
            start: start_id,
            goal: goal_id,
            steps: alt_route,
            gates,
            jumps,
        });
    }

    // Step 8: Build and return the route plan
    let (gates, jumps) = classify_edges(&graph, &route);

    Ok(RoutePlan {
        algorithm: request.algorithm,
        start: start_id,
        goal: goal_id,
        steps: route,
        gates,
        jumps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_route_constraints_have_none_optional_fields() {
        let c = RouteConstraints::default();
        assert!(c.avoid_critical_state);
        assert!(c.ship.is_none());
        assert!(c.loadout.is_none());
        assert!(c.heat_config.is_none());
    }

    #[test]
    fn route_plan_hop_count() {
        let plan = RoutePlan {
            algorithm: RouteAlgorithm::Bfs,
            start: 1,
            goal: 3,
            steps: vec![1, 2, 3],
            gates: 2,
            jumps: 0,
        };
        assert_eq!(plan.hop_count(), 2);
    }

    #[test]
    fn route_plan_empty_hop_count() {
        let plan = RoutePlan {
            algorithm: RouteAlgorithm::Bfs,
            start: 1,
            goal: 1,
            steps: vec![1],
            gates: 0,
            jumps: 0,
        };
        assert_eq!(plan.hop_count(), 0);
    }
}
