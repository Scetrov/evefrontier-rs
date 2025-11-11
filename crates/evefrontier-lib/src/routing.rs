use std::fmt;

use serde::Serialize;

use crate::db::{Starmap, SystemId};
use crate::error::{Error, Result};
use crate::{build_graph, find_route};

/// Supported routing algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteAlgorithm {
    /// Breadth-first search (unweighted graph).
    Bfs,
    /// Dijkstra's algorithm (weighted graph).
    Dijkstra,
    /// A* search (heuristic guided).
    #[serde(rename = "a_star")]
    AStar,
}

impl RouteAlgorithm {
    fn is_supported(self) -> bool {
        matches!(self, RouteAlgorithm::Bfs)
    }
}

impl fmt::Display for RouteAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            RouteAlgorithm::Bfs => "bfs",
            RouteAlgorithm::Dijkstra => "dijkstra",
            RouteAlgorithm::AStar => "a_star",
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
    fn unsupported_option(&self) -> Option<&'static str> {
        if self.max_jump.is_some() {
            return Some("--max-jump");
        }
        if !self.avoid_systems.is_empty() {
            return Some("--avoid");
        }
        if self.avoid_gates {
            return Some("--avoid-gates");
        }
        if self.max_temperature.is_some() {
            return Some("--max-temp");
        }
        None
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
    if let Some(option) = request.constraints.unsupported_option() {
        return Err(Error::UnsupportedRouteOption {
            option: option.to_string(),
        });
    }

    if !request.algorithm.is_supported() {
        return Err(Error::UnsupportedRouteOption {
            option: format!("algorithm {}", request.algorithm),
        });
    }

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

    let graph = build_graph(starmap);
    let Some(route) = find_route(&graph, start_id, goal_id) else {
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
