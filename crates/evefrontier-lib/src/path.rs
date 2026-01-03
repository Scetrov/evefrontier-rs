use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

use crate::db::{Starmap, SystemId};
use crate::graph::{Edge, EdgeKind, Graph};
use crate::ship::{calculate_jump_heat, HeatConfig, ShipAttributes, ShipLoadout, HEAT_CRITICAL};

/// Constraints applied during pathfinding.
#[derive(Debug, Default, Clone)]
pub struct PathConstraints {
    /// Maximum distance allowed for any single edge.
    pub max_jump: Option<f64>,
    /// Skip gate edges entirely when `true`.
    pub avoid_gates: bool,
    /// Set of system identifiers that must not appear in the resulting path.
    pub avoided_systems: HashSet<SystemId>,
    /// Maximum allowed stellar surface temperature in Kelvin (only enforced for spatial jumps).
    pub max_temperature: Option<f64>,
    /// Avoid hops that would cause the engine to become critical (requires ship/loadout).
    pub avoid_critical_state: bool,
    /// Optional ship attributes used to evaluate heat for a hop.
    pub ship: Option<ShipAttributes>,
    /// Optional loadout used to compute mass for heat calculation.
    pub loadout: Option<ShipLoadout>,
    /// Optional heat configuration (calibration constant etc.); required when `avoid_critical_state` is `true`.
    pub heat_config: Option<HeatConfig>,
}

impl PathConstraints {
    fn allows(&self, starmap: Option<&Starmap>, edge: &Edge, target: SystemId) -> bool {
        if self.max_jump.is_some_and(|limit| edge.distance > limit) {
            return false;
        }

        if self.avoid_gates && edge.kind == EdgeKind::Gate {
            return false;
        }

        if self.avoided_systems.contains(&target) {
            return false;
        }

        // Temperature and heat constraints only apply to spatial jumps
        if edge.kind != EdgeKind::Spatial {
            return true;
        }

        if let Some(limit) = self.max_temperature {
            let temp = starmap
                .and_then(|m| m.systems.get(&target))
                .and_then(|s| s.metadata.star_temperature);
            if temp.is_some_and(|t| t > limit) {
                return false;
            }
        }

        // If configured, avoid hops that would result in critical engine state.
        // Check: ambient_temp + hop_heat >= HEAT_CRITICAL (150K)
        // Note: ambient_temp is min_external_temp (0.1K-99.9K range), not star surface temperature
        if self.avoid_critical_state {
            if let (Some(ship), Some(loadout)) = (&self.ship, &self.loadout) {
                // Get the minimum external temperature at the destination system
                // This is the temperature at the coldest habitable zone (furthest planet/moon)
                let ambient_temp = starmap
                    .and_then(|m| m.systems.get(&target))
                    .and_then(|s| s.metadata.min_external_temp)
                    .unwrap_or(0.0);

                let mass = loadout.total_mass_kg(ship);
                // Require an explicit heat_config when avoid_critical_state is requested; if
                // missing, treat as a configuration error and conservatively reject the edge.
                let heat_config = if let Some(cfg) = self.heat_config {
                    cfg
                } else {
                    tracing::error!(
                        "heat_config must be set when avoid_critical_state is true; rejecting edge"
                    );
                    return false;
                };

                let hop_energy = calculate_jump_heat(
                    mass,
                    edge.distance,
                    ship.base_mass_kg,
                    heat_config.calibration_constant,
                );

                match hop_energy {
                    Ok(energy) => {
                        let hop_heat = energy / (mass * ship.specific_heat);
                        let total_heat = ambient_temp + hop_heat;
                        if total_heat >= HEAT_CRITICAL {
                            let to_name = starmap
                                .and_then(|m| m.systems.get(&target))
                                .map(|s| s.name.as_str())
                                .unwrap_or("unknown");

                            tracing::debug!(
                                "blocking edge to {} due to critical heat: ambient={:.1}K, hop_heat={:.1}K, total={:.1}K (limit={:.1}K)",
                                to_name,
                                ambient_temp,
                                hop_heat,
                                total_heat,
                                HEAT_CRITICAL
                            );
                            return false;
                        }
                    }
                    Err(e) => {
                        // Conservative fail-safe: if heat calculation errors, reject the edge and
                        // log the error so callers can surface problems instead of allowing
                        // potentially unsafe routes.
                        let to_name = starmap
                            .and_then(|m| m.systems.get(&target))
                            .map(|s| s.name.as_str())
                            .unwrap_or("unknown");
                        tracing::warn!(
                            "heat calculation failed for {}: {e:#?}; rejecting edge as conservative fail-safe",
                            to_name
                        );
                        return false;
                    }
                }
            } else {
                // Missing ship/loadout is a configuration error when `avoid_critical_state` is
                // requested. Conservatively reject the edge and log an error so callers can
                // detect misconfiguration rather than silently weakening a safety check.
                tracing::error!(
                    "avoid_critical_state requested but missing ship/loadout; rejecting edge"
                );
                return false;
            }
        }

        true
    }
}

/// Find a route between `start` and `goal` using breadth-first search without
/// additional constraints.
pub fn find_route(graph: &Graph, start: SystemId, goal: SystemId) -> Option<Vec<SystemId>> {
    let constraints = PathConstraints::default();
    find_route_bfs(graph, None, start, goal, &constraints)
}

/// Run breadth-first search with optional constraints.
pub fn find_route_bfs(
    graph: &Graph,
    starmap: Option<&Starmap>,
    start: SystemId,
    goal: SystemId,
    constraints: &PathConstraints,
) -> Option<Vec<SystemId>> {
    if start == goal {
        return Some(vec![start]);
    }

    let mut parents: HashMap<SystemId, Option<SystemId>> = HashMap::new();
    let mut queue = VecDeque::new();

    parents.insert(start, None);
    queue.push_back(start);

    while let Some(current) = queue.pop_front() {
        for edge in graph.neighbours(current) {
            let next = edge.target;
            if parents.contains_key(&next) {
                continue;
            }
            if !constraints.allows(starmap, edge, next) {
                continue;
            }

            parents.insert(next, Some(current));
            if next == goal {
                return Some(reconstruct_path(&parents, start, goal));
            }
            queue.push_back(next);
        }
    }

    None
}

/// Run Dijkstra's algorithm to find the lowest-cost path that satisfies the
/// provided constraints.
pub fn find_route_dijkstra(
    graph: &Graph,
    starmap: Option<&Starmap>,
    start: SystemId,
    goal: SystemId,
    constraints: &PathConstraints,
) -> Option<Vec<SystemId>> {
    if start == goal {
        return Some(vec![start]);
    }

    let mut distances: HashMap<SystemId, f64> = HashMap::new();
    let mut parents: HashMap<SystemId, Option<SystemId>> = HashMap::new();
    let mut queue = BinaryHeap::new();

    distances.insert(start, 0.0);
    parents.insert(start, None);
    queue.push(QueueEntry::new(start, 0.0));

    while let Some(entry) = queue.pop() {
        // Skip stale queue entries where we've already found a better path
        let Some(&current_distance) = distances.get(&entry.node) else {
            continue;
        };
        if current_distance < entry.cost.0 {
            continue;
        }

        if entry.node == goal {
            return Some(reconstruct_path(&parents, start, goal));
        }

        for edge in graph.neighbours(entry.node) {
            let next = edge.target;
            if !constraints.allows(starmap, edge, next) {
                continue;
            }

            let next_cost = current_distance + edge.distance;
            if next_cost < *distances.get(&next).unwrap_or(&f64::INFINITY) {
                distances.insert(next, next_cost);
                parents.insert(next, Some(entry.node));
                queue.push(QueueEntry::new(next, next_cost));
            }
        }
    }

    None
}

/// Run A* search with an admissible heuristic derived from system positions
/// when available.
pub fn find_route_a_star(
    graph: &Graph,
    starmap: Option<&Starmap>,
    start: SystemId,
    goal: SystemId,
    constraints: &PathConstraints,
) -> Option<Vec<SystemId>> {
    if start == goal {
        return Some(vec![start]);
    }

    let mut g_score: HashMap<SystemId, f64> = HashMap::new();
    let mut parents: HashMap<SystemId, Option<SystemId>> = HashMap::new();
    let mut queue = BinaryHeap::new();

    g_score.insert(start, 0.0);
    parents.insert(start, None);
    let start_estimate = heuristic_distance(starmap, start, goal);
    queue.push(AStarEntry::new(start, 0.0, start_estimate));

    while let Some(entry) = queue.pop() {
        // Check if this is the best-known cost for the current node.
        // First, if the cost matches the stored g_score (within floating-point epsilon), use it.
        // Second, if we already found a better path (score < entry cost), skip this entry.
        // Otherwise, use the stored score.
        let current_score = if let Some(score) = g_score.get(&entry.node) {
            if (*score - entry.cost.0).abs() < f64::EPSILON {
                *score
            } else if *score < entry.cost.0 {
                continue;
            } else {
                *score
            }
        } else {
            continue;
        };

        if entry.node == goal {
            return Some(reconstruct_path(&parents, start, goal));
        }

        for edge in graph.neighbours(entry.node) {
            let next = edge.target;
            if !constraints.allows(starmap, edge, next) {
                continue;
            }

            let tentative_g = current_score + edge.distance;
            if tentative_g < *g_score.get(&next).unwrap_or(&f64::INFINITY) {
                g_score.insert(next, tentative_g);
                parents.insert(next, Some(entry.node));
                let heuristic = heuristic_distance(starmap, next, goal);
                queue.push(AStarEntry::new(next, tentative_g, heuristic));
            }
        }
    }

    None
}

fn heuristic_distance(starmap: Option<&Starmap>, from: SystemId, to: SystemId) -> f64 {
    let Some(map) = starmap else {
        return 0.0;
    };

    let Some(goal) = map.systems.get(&to) else {
        return 0.0;
    };
    let Some(goal_position) = goal.position else {
        return 0.0;
    };

    if let Some(current) = map.systems.get(&from).and_then(|system| system.position) {
        current.distance_to(&goal_position)
    } else {
        0.0
    }
}

fn reconstruct_path(
    parents: &HashMap<SystemId, Option<SystemId>>,
    start: SystemId,
    goal: SystemId,
) -> Vec<SystemId> {
    let mut path = Vec::new();
    let mut current = Some(goal);
    while let Some(node) = current {
        path.push(node);
        if node == start {
            break;
        }
        current = parents.get(&node).copied().flatten();
    }
    path.reverse();
    path
}

#[derive(Copy, Clone, Debug, Default)]
struct FloatOrd(f64);

impl PartialEq for FloatOrd {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for FloatOrd {}

impl PartialOrd for FloatOrd {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FloatOrd {
    fn cmp(&self, other: &Self) -> Ordering {
        // Use a total ordering for floats which also places NaN values after
        // finite numbers. `total_cmp` provides a deterministic, IEEE-754
        // compatible total order and avoids handling NaN specially here.
        self.0.total_cmp(&other.0)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct QueueEntry {
    node: SystemId,
    cost: FloatOrd,
}

impl QueueEntry {
    fn new(node: SystemId, cost: f64) -> Self {
        Self {
            node,
            cost: FloatOrd(cost),
        }
    }
}

impl Ord for QueueEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering so BinaryHeap becomes a min-heap by cost.
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| other.node.cmp(&self.node))
    }
}

impl PartialOrd for QueueEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct AStarEntry {
    node: SystemId,
    cost: FloatOrd,
    estimate: FloatOrd,
}

impl AStarEntry {
    fn new(node: SystemId, cost: f64, heuristic: f64) -> Self {
        Self {
            node,
            cost: FloatOrd(cost),
            estimate: FloatOrd(cost + heuristic),
        }
    }
}

impl Ord for AStarEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .estimate
            .cmp(&self.estimate)
            .then_with(|| other.node.cmp(&self.node))
    }
}

impl PartialOrd for AStarEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Starmap, System, SystemMetadata};

    #[test]
    fn default_constraints_are_non_blocking() {
        let c = PathConstraints::default();
        assert!(!c.avoid_critical_state);
        assert!(c.ship.is_none());
        assert!(c.loadout.is_none());
    }

    #[test]
    fn heat_calc_error_blocks_edge() {
        // Build a minimal starmap with a single system target having a known ambient temp
        let mut systems = std::collections::HashMap::new();
        let sys = System {
            id: 1,
            name: "Target".to_string(),
            metadata: SystemMetadata {
                constellation_id: None,
                constellation_name: None,
                region_id: None,
                region_name: None,
                security_status: None,
                star_temperature: None,
                star_luminosity: None,
                min_external_temp: Some(10.0),
                planet_count: None,
                moon_count: None,
            },
            position: None,
        };
        systems.insert(1, sys);
        let starmap = Starmap {
            systems,
            name_to_id: std::collections::HashMap::new(),
            adjacency: std::sync::Arc::new(std::collections::HashMap::new()),
        };

        // Create constraints that request avoid_critical_state and provide a ship with an
        // invalid hull_mass (0.0) so that calculate_jump_heat returns an error.
        let constraints = PathConstraints {
            avoid_critical_state: true,
            ship: Some(ShipAttributes {
                name: "BugShip".to_string(),
                base_mass_kg: 0.0, // invalid to trigger error
                specific_heat: 1.0,
                fuel_capacity: 100.0,
                cargo_capacity: 100.0,
            }),
            loadout: Some(ShipLoadout {
                fuel_load: 10.0,
                cargo_mass_kg: 0.0,
            }),
            heat_config: Some(HeatConfig::default()),
            ..Default::default()
        };

        let edge = crate::graph::Edge {
            target: 1,
            kind: crate::graph::EdgeKind::Spatial,
            distance: 10.0,
        };

        // With conservative fail-safe, a heat calculation error should block the edge
        assert!(!constraints.allows(Some(&starmap), &edge, 1));
    }
}
