use std::fmt;

use std::collections::HashSet;
use std::sync::Arc;

use serde::Serialize;

use crate::db::{Starmap, SystemId};
use crate::error::{Error, Result};
use crate::graph::{
    build_gate_graph, build_hybrid_graph_indexed, build_spatial_graph_indexed, EdgeKind, Graph,
    GraphBuildOptions, GraphMode,
};
use crate::path::{
    find_route_a_star, find_route_bfs, find_route_dijkstra, PathConstraints as SearchConstraints,
};
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

fn build_filtered_adjacency_from_search(
    graph: &Graph,
    starmap: &Starmap,
    sc: &SearchConstraints,
) -> std::collections::HashMap<SystemId, Vec<crate::graph::Edge>> {
    let mut filtered: std::collections::HashMap<_, _> = std::collections::HashMap::new();
    for &sid in starmap.systems.keys() {
        let mut out = Vec::new();
        for e in graph.neighbours(sid) {
            if sc.allows(Some(starmap), e, e.target) {
                out.push(e.clone());
            }
        }
        filtered.insert(sid, out);
    }
    filtered
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

/// Compute a route using the requested algorithm and constraints.
pub fn plan_route(starmap: &Starmap, request: &RouteRequest) -> Result<RoutePlan> {
    let start_id = starmap.system_id_by_name(&request.start).ok_or_else(|| {
        let suggestions = starmap.fuzzy_system_matches(&request.start, 3);
        Error::UnknownSystem {
            name: request.start.clone(),
            suggestions,
        }
    })?;
    let goal_id = starmap.system_id_by_name(&request.goal).ok_or_else(|| {
        let suggestions = starmap.fuzzy_system_matches(&request.goal, 3);
        Error::UnknownSystem {
            name: request.goal.clone(),
            suggestions,
        }
    })?;

    let avoided = resolve_avoided_systems(starmap, &request.constraints.avoid_systems)?;
    let constraints = request.constraints.to_search_constraints(avoided);

    // Compute a ship-capability-based per-hop max_jump when ship & loadout are present.
    // This computes the maximum distance a ship can traverse for a single jump constrained
    // by fuel availability and heat limits at the origin system, and combines it with any
    // explicitly-requested `max_jump` to produce an effective per-hop limit used for
    // graph construction.
    let mut effective_constraints = constraints.clone();
    if let (Some(ship), Some(loadout)) = (&request.constraints.ship, &request.constraints.loadout) {
        // Fuel-based maximum distance (uses current fuel load and requested fuel quality)
        let fuel_quality = request.fuel_config.quality;
        let mass = loadout.total_mass_kg(ship);
        let per_ly_fuel = (mass / 100_000.0) * (fuel_quality / 100.0);
        let fuel_max = if per_ly_fuel > 0.0 {
            Some(loadout.fuel_load / per_ly_fuel)
        } else {
            None
        };

        // Heat-based maximum distance only applies when the caller requested avoidance of
        // critical engine state; otherwise heat should not implicitly limit per-hop range.
        let heat_max = if request.constraints.avoid_critical_state {
            let ambient = starmap
                .systems
                .get(&start_id)
                .and_then(|s| s.metadata.min_external_temp)
                .unwrap_or(0.0);
            let heat_cfg = request.constraints.heat_config.unwrap_or_default();

            let allowed_delta = crate::ship::HEAT_CRITICAL - ambient;
            if allowed_delta > 0.0 && heat_cfg.calibration_constant > 0.0 {
                Some(
                    allowed_delta
                        * (heat_cfg.calibration_constant * ship.base_mass_kg * ship.specific_heat)
                        / 3.0,
                )
            } else {
                None
            }
        } else {
            None
        };

        // For safety we only apply the heat-based limit to the per-hop max_jump when
        // the caller explicitly requested avoidance of critical engine state. The
        // fuel-based maximum is informative (used for fuel projections) but should not
        // implicitly remove routes from consideration — a route may include long jumps
        // that require refuelling or are otherwise outside the ship's current fuel range.
        if let Some(h) = heat_max {
            effective_constraints.max_jump = match effective_constraints.max_jump {
                Some(user) => Some(user.min(h)),
                None => Some(h),
            };
        }

        tracing::debug!(
            "computed ship-based max_jump: fuel={:?}, heat={:?}, applied_heat_limit={:?}",
            fuel_max,
            heat_max,
            effective_constraints.max_jump
        );
    }

    if constraints.avoided_systems.contains(&start_id)
        || constraints.avoided_systems.contains(&goal_id)
        || !system_meets_temperature(starmap, start_id, constraints.max_temperature)
        || !system_meets_temperature(starmap, goal_id, constraints.max_temperature)
    {
        return Err(Error::RouteNotFound {
            start: request.start.clone(),
            goal: request.goal.clone(),
        });
    }

    let graph = select_graph(
        starmap,
        request.algorithm,
        &effective_constraints,
        request.spatial_index.as_ref().cloned(),
        request.max_spatial_neighbors,
    );

    let route = match request.algorithm {
        RouteAlgorithm::Bfs => find_route_bfs(
            &graph,
            Some(starmap),
            start_id,
            goal_id,
            &effective_constraints,
        ),
        RouteAlgorithm::Dijkstra => {
            if request.optimization == RouteOptimization::Fuel {
                // Fuel optimization requires ship + loadout to compute hop costs. If missing,
                // fall back to distance-based Dijkstra but warn the caller.
                if let (Some(ship), Some(loadout)) =
                    (&request.constraints.ship, &request.constraints.loadout)
                {
                    let mass = loadout.total_mass_kg(ship);
                    crate::path::find_route_dijkstra_fuel(
                        &graph,
                        Some(starmap),
                        start_id,
                        goal_id,
                        &effective_constraints,
                        mass,
                        &request.fuel_config,
                    )
                } else {
                    tracing::warn!("fuel optimization requested but missing ship/loadout; falling back to distance optimization");
                    find_route_dijkstra(
                        &graph,
                        Some(starmap),
                        start_id,
                        goal_id,
                        &effective_constraints,
                    )
                }
            } else {
                find_route_dijkstra(
                    &graph,
                    Some(starmap),
                    start_id,
                    goal_id,
                    &effective_constraints,
                )
            }
        }
        RouteAlgorithm::AStar => {
            // A* with fuel optimization is approximated by running Dijkstra with fuel costs
            // to keep heuristic admissibility simple. If fuel optimization requested and ship
            // info is present, use fuel-based Dijkstra; otherwise use A* on distance.
            if request.optimization == RouteOptimization::Fuel {
                if let (Some(ship), Some(loadout)) =
                    (&request.constraints.ship, &request.constraints.loadout)
                {
                    let mass = loadout.total_mass_kg(ship);
                    crate::path::find_route_dijkstra_fuel(
                        &graph,
                        Some(starmap),
                        start_id,
                        goal_id,
                        &effective_constraints,
                        mass,
                        &request.fuel_config,
                    )
                } else {
                    tracing::warn!("fuel optimization requested but missing ship/loadout; falling back to distance A*");
                    find_route_a_star(
                        &graph,
                        Some(starmap),
                        start_id,
                        goal_id,
                        &effective_constraints,
                    )
                }
            } else {
                find_route_a_star(
                    &graph,
                    Some(starmap),
                    start_id,
                    goal_id,
                    &effective_constraints,
                )
            }
        }
    };

    let Some(route) = route else {
        return Err(Error::RouteNotFound {
            start: request.start.clone(),
            goal: request.goal.clone(),
        });
    };

    // Defensive validation: ensure every edge in the computed route satisfies the
    // search constraints. This guards against any discrepancy where the planner
    // might produce a route containing edges that should have been rejected (for
    // example, heat-critical spatial hops). If such an edge is found, report
    // the route as not found to surface a consistent error to callers.
    for pair in route.windows(2) {
        let u = pair[0];
        let v = pair[1];
        // Determine the planner's chosen edge for this hop (shortest distance). Validate
        // the chosen edge against the avoidance constraints and reject the route if the
        // chosen edge would be unsafe under the request.
        let chosen = graph
            .neighbours(u)
            .iter()
            .filter(|e| e.target == v)
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

        if let Some(edge) = chosen {
            // If this is a spatial edge and we are avoiding critical state, compute the
            // expected total heat and reject the route if it would exceed the critical
            // threshold. Gate edges are considered safe.
            if edge.kind == crate::graph::EdgeKind::Spatial
                && request.constraints.avoid_critical_state
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
                            // Found a critical hop in the chosen route. Instead of failing
                            // immediately, try re-planning on a graph that removes unsafe
                            // spatial edges to allow gate-based alternatives to be found.
                            let search_constraints = request
                                .constraints
                                .to_search_constraints(std::collections::HashSet::new());
                            let filtered = build_filtered_adjacency_from_search(
                                &graph,
                                starmap,
                                &search_constraints,
                            );

                            let filtered_graph =
                                crate::graph::Graph::from_parts(graph.mode(), filtered);

                            // Re-run the requested algorithm on the filtered graph.
                            let alt_route = match request.algorithm {
                                RouteAlgorithm::Bfs => find_route_bfs(
                                    &filtered_graph,
                                    Some(starmap),
                                    start_id,
                                    goal_id,
                                    &constraints,
                                ),
                                RouteAlgorithm::Dijkstra => find_route_dijkstra(
                                    &filtered_graph,
                                    Some(starmap),
                                    start_id,
                                    goal_id,
                                    &constraints,
                                ),
                                RouteAlgorithm::AStar => find_route_a_star(
                                    &filtered_graph,
                                    Some(starmap),
                                    start_id,
                                    goal_id,
                                    &constraints,
                                ),
                            };

                            if let Some(alt) = alt_route {
                                // Replace route with alternative if found and continue validation
                                // using the filtered graph to avoid re-surfacing the same unsafe hop.
                                let (g, j) = classify_edges(&filtered_graph, &alt);
                                return Ok(RoutePlan {
                                    algorithm: request.algorithm,
                                    start: start_id,
                                    goal: goal_id,
                                    steps: alt,
                                    gates: g,
                                    jumps: j,
                                });
                            }

                            return Err(Error::RouteNotFound {
                                start: request.start.clone(),
                                goal: request.goal.clone(),
                            });
                        }
                    }
                }
            }
        } else {
            return Err(Error::RouteNotFound {
                start: request.start.clone(),
                goal: request.goal.clone(),
            });
        }
    }

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

fn resolve_avoided_systems(starmap: &Starmap, avoided: &[String]) -> Result<HashSet<SystemId>> {
    let mut resolved = HashSet::new();
    for name in avoided {
        let id = starmap.system_id_by_name(name).ok_or_else(|| {
            let suggestions = starmap.fuzzy_system_matches(name, 3);
            Error::UnknownSystem {
                name: name.clone(),
                suggestions,
            }
        })?;
        resolved.insert(id);
    }
    Ok(resolved)
}

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

fn select_graph(
    starmap: &Starmap,
    algorithm: RouteAlgorithm,
    constraints: &SearchConstraints,
    spatial_index: Option<Arc<SpatialIndex>>,
    max_spatial_neighbors: usize,
) -> Graph {
    // Build options for indexed graph construction
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
                let mut chosen: Option<(EdgeKind, f64)> = None;
                for edge in graph.neighbours(u) {
                    if edge.target == v {
                        match chosen {
                            None => chosen = Some((edge.kind, edge.distance)),
                            Some((_, d)) => {
                                if edge.distance < d {
                                    chosen = Some((edge.kind, edge.distance));
                                }
                            }
                        }
                    }
                }
                match chosen.map(|(k, _)| k) {
                    Some(EdgeKind::Gate) => gates += 1,
                    Some(EdgeKind::Spatial) => jumps += 1,
                    None => {
                        // Fallback: if no direct edge found (shouldn't happen), assume gate.
                        gates += 1;
                    }
                }
            }
            (gates, jumps)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_route_constraints_have_none_optional_fields() {
        let c = RouteConstraints::default();
        // By default we avoid critical engine states to provide safer routes.
        assert!(c.avoid_critical_state);
        assert!(c.ship.is_none());
        assert!(c.loadout.is_none());
        assert!(c.heat_config.is_none());
    }
}
