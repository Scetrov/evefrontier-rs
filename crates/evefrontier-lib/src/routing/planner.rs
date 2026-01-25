//! Route planning strategies implementing the Strategy pattern.
//!
//! This module provides the `RoutePlanner` trait and implementations for
//! different routing algorithms (BFS, Dijkstra, A*). The strategy pattern
//! allows adding new algorithms without modifying the `plan_route` orchestrator.

use crate::db::{Starmap, SystemId};
use crate::graph::Graph;
use crate::path::{
    find_route_a_star, find_route_bfs, find_route_dijkstra, PathConstraints as SearchConstraints,
};
use crate::ship::FuelConfig;

use super::{RouteAlgorithm, RouteOptimization, RouteRequest};

/// Trait for route planning strategies.
///
/// Each implementation encapsulates a specific pathfinding algorithm and
/// its associated behavior (graph selection, optimization mode, etc.).
pub trait RoutePlanner: Send + Sync {
    /// The algorithm identifier for this planner.
    fn algorithm(&self) -> RouteAlgorithm;

    /// Execute the pathfinding algorithm on the given graph.
    ///
    /// Returns `Some(path)` if a route is found, `None` otherwise.
    fn find_path(
        &self,
        graph: &Graph,
        starmap: Option<&Starmap>,
        start: SystemId,
        goal: SystemId,
        constraints: &SearchConstraints,
    ) -> Option<Vec<SystemId>>;

    /// Whether this planner requires a spatial index for optimal performance.
    fn requires_spatial_index(&self) -> bool {
        false
    }
}

/// Breadth-first search planner for unweighted graph traversal.
///
/// BFS finds the path with the fewest hops (edges) but does not
/// consider edge weights (distances).
#[derive(Debug, Clone, Default)]
pub struct BfsPlanner;

impl RoutePlanner for BfsPlanner {
    fn algorithm(&self) -> RouteAlgorithm {
        RouteAlgorithm::Bfs
    }

    fn find_path(
        &self,
        graph: &Graph,
        starmap: Option<&Starmap>,
        start: SystemId,
        goal: SystemId,
        constraints: &SearchConstraints,
    ) -> Option<Vec<SystemId>> {
        find_route_bfs(graph, starmap, start, goal, constraints)
    }
}

/// Dijkstra's algorithm planner for weighted graph traversal.
///
/// Can optimize for either distance or fuel consumption.
#[derive(Debug, Clone)]
pub struct DijkstraPlanner {
    optimization: RouteOptimization,
    fuel_config: FuelConfig,
    ship_mass: Option<f64>,
}

impl DijkstraPlanner {
    /// Create a distance-optimizing Dijkstra planner.
    pub fn distance() -> Self {
        Self {
            optimization: RouteOptimization::Distance,
            fuel_config: FuelConfig::default(),
            ship_mass: None,
        }
    }

    /// Create a fuel-optimizing Dijkstra planner.
    pub fn fuel(fuel_config: FuelConfig, ship_mass: f64) -> Self {
        Self {
            optimization: RouteOptimization::Fuel,
            fuel_config,
            ship_mass: Some(ship_mass),
        }
    }

    /// Create a planner from a route request.
    pub fn from_request(request: &RouteRequest) -> Self {
        let ship_mass = request
            .constraints
            .ship
            .as_ref()
            .zip(request.constraints.loadout.as_ref())
            .map(|(ship, loadout)| loadout.total_mass_kg(ship));

        Self {
            optimization: request.optimization,
            fuel_config: request.fuel_config,
            ship_mass,
        }
    }
}

impl RoutePlanner for DijkstraPlanner {
    fn algorithm(&self) -> RouteAlgorithm {
        RouteAlgorithm::Dijkstra
    }

    fn find_path(
        &self,
        graph: &Graph,
        starmap: Option<&Starmap>,
        start: SystemId,
        goal: SystemId,
        constraints: &SearchConstraints,
    ) -> Option<Vec<SystemId>> {
        if self.optimization == RouteOptimization::Fuel {
            if let Some(mass) = self.ship_mass {
                return crate::path::find_route_dijkstra_fuel(
                    graph,
                    starmap,
                    start,
                    goal,
                    constraints,
                    mass,
                    &self.fuel_config,
                );
            }
            tracing::warn!(
                "fuel optimization requested but missing ship/loadout; falling back to distance"
            );
        }
        find_route_dijkstra(graph, starmap, start, goal, constraints)
    }

    fn requires_spatial_index(&self) -> bool {
        true
    }
}

/// A* algorithm planner for heuristic-guided traversal.
///
/// Uses Euclidean distance as the heuristic. Can fall back to
/// fuel-based Dijkstra when optimizing for fuel consumption.
#[derive(Debug, Clone)]
pub struct AStarPlanner {
    optimization: RouteOptimization,
    fuel_config: FuelConfig,
    ship_mass: Option<f64>,
}

impl AStarPlanner {
    /// Create a distance-optimizing A* planner.
    pub fn distance() -> Self {
        Self {
            optimization: RouteOptimization::Distance,
            fuel_config: FuelConfig::default(),
            ship_mass: None,
        }
    }

    /// Create a fuel-optimizing A* planner.
    ///
    /// Note: Fuel optimization uses Dijkstra internally to maintain
    /// admissibility of the heuristic.
    pub fn fuel(fuel_config: FuelConfig, ship_mass: f64) -> Self {
        Self {
            optimization: RouteOptimization::Fuel,
            fuel_config,
            ship_mass: Some(ship_mass),
        }
    }

    /// Create a planner from a route request.
    pub fn from_request(request: &RouteRequest) -> Self {
        let ship_mass = request
            .constraints
            .ship
            .as_ref()
            .zip(request.constraints.loadout.as_ref())
            .map(|(ship, loadout)| loadout.total_mass_kg(ship));

        Self {
            optimization: request.optimization,
            fuel_config: request.fuel_config,
            ship_mass,
        }
    }
}

impl RoutePlanner for AStarPlanner {
    fn algorithm(&self) -> RouteAlgorithm {
        RouteAlgorithm::AStar
    }

    fn find_path(
        &self,
        graph: &Graph,
        starmap: Option<&Starmap>,
        start: SystemId,
        goal: SystemId,
        constraints: &SearchConstraints,
    ) -> Option<Vec<SystemId>> {
        // A* with fuel optimization is approximated by running Dijkstra with fuel costs
        // to keep heuristic admissibility simple.
        if self.optimization == RouteOptimization::Fuel {
            if let Some(mass) = self.ship_mass {
                return crate::path::find_route_dijkstra_fuel(
                    graph,
                    starmap,
                    start,
                    goal,
                    constraints,
                    mass,
                    &self.fuel_config,
                );
            }
            tracing::warn!(
                "fuel optimization requested but missing ship/loadout; falling back to distance A*"
            );
        }
        find_route_a_star(graph, starmap, start, goal, constraints)
    }

    fn requires_spatial_index(&self) -> bool {
        true
    }
}

/// Select the appropriate planner for a given request.
pub fn select_planner(request: &RouteRequest) -> Box<dyn RoutePlanner> {
    match request.algorithm {
        RouteAlgorithm::Bfs => Box::new(BfsPlanner),
        RouteAlgorithm::Dijkstra => Box::new(DijkstraPlanner::from_request(request)),
        RouteAlgorithm::AStar => Box::new(AStarPlanner::from_request(request)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bfs_planner_returns_correct_algorithm() {
        let planner = BfsPlanner;
        assert_eq!(planner.algorithm(), RouteAlgorithm::Bfs);
        assert!(!planner.requires_spatial_index());
    }

    #[test]
    fn dijkstra_planner_returns_correct_algorithm() {
        let planner = DijkstraPlanner::distance();
        assert_eq!(planner.algorithm(), RouteAlgorithm::Dijkstra);
        assert!(planner.requires_spatial_index());
    }

    #[test]
    fn astar_planner_returns_correct_algorithm() {
        let planner = AStarPlanner::distance();
        assert_eq!(planner.algorithm(), RouteAlgorithm::AStar);
        assert!(planner.requires_spatial_index());
    }

    #[test]
    fn select_planner_chooses_correct_type() {
        let bfs_request = RouteRequest::bfs("A", "B");
        let planner = select_planner(&bfs_request);
        assert_eq!(planner.algorithm(), RouteAlgorithm::Bfs);
    }
}
