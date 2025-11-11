use std::fmt;

use std::collections::{HashSet};

use serde::Serialize;

use crate::db::{Starmap, SystemId};
use crate::error::{Error, Result};
use crate::graph::{build_gate_graph, build_hybrid_graph, build_spatial_graph, Graph};
use crate::path::{
    find_route_a_star, find_route_bfs, find_route_dijkstra, PathConstraints as SearchConstraints,
};

/// Supported routing algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteAlgorithm {
    /// Breadth-first search (unweighted graph).
    Bfs,
    /// Dijkstra's algorithm (weighted graph).
    Dijkstra,
    /// A* search (heuristic guided).
    #[serde(rename = "a-star")]
    AStar,
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
#[derive(Debug, Default, Clone)]
pub struct RouteConstraints {
    pub max_jump: Option<f64>,
    pub avoid_systems: Vec<String>,
    pub avoid_gates: bool,
    pub max_temperature: Option<f64>,
}

impl RouteConstraints {
    fn to_search_constraints(&self, avoided: HashSet<SystemId>) -> SearchConstraints {
        SearchConstraints {
            max_jump: self.max_jump,
            avoid_gates: self.avoid_gates,
            avoided_systems: avoided,
            max_temperature: self.max_temperature,
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
}

impl RouteRequest {
    /// Convenience constructor for BFS routes without extra constraints.
    pub fn bfs(start: impl Into<String>, goal: impl Into<String>) -> Self {
        Self {
            start: start.into(),
            goal: goal.into(),
            algorithm: RouteAlgorithm::Bfs,
            constraints: RouteConstraints::default(),
        }
    }
}

/// Planned route returned by the library.
#[derive(Debug, Clone, Serialize)]
pub struct RoutePlan {
    pub algorithm: RouteAlgorithm,
    pub start: SystemId,
    pub goal: SystemId,
    pub steps: Vec<SystemId>,
}

impl RoutePlan {
    /// Number of hops in the route.
    pub fn hop_count(&self) -> usize {
        self.steps.len().saturating_sub(1)
    }
}

/// Compute a route using the requested algorithm and constraints.
pub fn plan_route(starmap: &Starmap, request: &RouteRequest) -> Result<RoutePlan> {
    let start_id =
        starmap
            .system_id_by_name(&request.start)
            .ok_or_else(|| Error::UnknownSystem {
                name: request.start.clone(),
            })?;
    let goal_id = starmap
        .system_id_by_name(&request.goal)
        .ok_or_else(|| Error::UnknownSystem {
            name: request.goal.clone(),
        })?;

    let avoided = resolve_avoided_systems(starmap, &request.constraints.avoid_systems)?;
    let constraints = request.constraints.to_search_constraints(avoided);

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

    let graph = select_graph(starmap, request.algorithm, &constraints);

    let route = match request.algorithm {
        RouteAlgorithm::Bfs => {
            find_route_bfs(&graph, Some(starmap), start_id, goal_id, &constraints)
        }
        RouteAlgorithm::Dijkstra => {
            find_route_dijkstra(&graph, Some(starmap), start_id, goal_id, &constraints)
        }
        RouteAlgorithm::AStar => {
            find_route_a_star(&graph, Some(starmap), start_id, goal_id, &constraints)
        }
    };

    let Some(route) = route else {
        return Err(Error::RouteNotFound {
            start: request.start.clone(),
            goal: request.goal.clone(),
        });
    };

    Ok(RoutePlan {
        algorithm: request.algorithm,
        start: start_id,
        goal: goal_id,
        steps: route,
    })
}

fn resolve_avoided_systems(starmap: &Starmap, avoided: &[String]) -> Result<HashSet<SystemId>> {
    let mut resolved = HashSet::new();
    for name in avoided {
        let id = starmap
            .system_id_by_name(name)
            .ok_or_else(|| Error::UnknownSystem { name: name.clone() })?;
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
        .and_then(|sys| sys.metadata.temperature)
        .map(|temperature| temperature <= limit)
        .unwrap_or(true)
}

fn select_graph(
    starmap: &Starmap,
    algorithm: RouteAlgorithm,
    constraints: &SearchConstraints,
) -> Graph {
    if constraints.avoid_gates {
        return build_spatial_graph(starmap);
    }

    match algorithm {
        RouteAlgorithm::Bfs => build_gate_graph(starmap),
        RouteAlgorithm::Dijkstra | RouteAlgorithm::AStar => build_hybrid_graph(starmap),
    }
}
