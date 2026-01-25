use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

use crate::db::{Starmap, SystemId};
use crate::graph::{Edge, EdgeKind, Graph};
use crate::ship::{calculate_jump_heat, HeatConfig, ShipAttributes, ShipLoadout, HEAT_CRITICAL};

// =============================================================================
// Edge Predicates - composable functions for edge filtering
// =============================================================================

/// Check if an edge meets the maximum jump distance constraint.
/// Only applies to spatial edges; gates are always allowed.
fn edge_meets_distance_limit(edge: &Edge, max_jump: Option<f64>) -> bool {
    if edge.kind != EdgeKind::Spatial {
        return true;
    }
    max_jump.is_none_or(|limit| edge.distance <= limit)
}

/// Check if an edge is allowed based on gate avoidance policy.
fn edge_meets_gate_policy(edge: &Edge, avoid_gates: bool) -> bool {
    !(avoid_gates && edge.kind == EdgeKind::Gate)
}

// =============================================================================
// System Predicates - composable functions for system filtering
// =============================================================================

/// Check if a system is not in the avoided systems set.
fn system_meets_avoidance(target: SystemId, avoided: &HashSet<SystemId>) -> bool {
    !avoided.contains(&target)
}

/// Check if a system meets the temperature constraint.
/// Only applies to spatial jumps; non-spatial always passes.
fn system_meets_temperature(
    edge: &Edge,
    starmap: Option<&Starmap>,
    target: SystemId,
    max_temperature: Option<f64>,
) -> bool {
    if edge.kind != EdgeKind::Spatial {
        return true;
    }
    let Some(limit) = max_temperature else {
        return true;
    };
    let temp = starmap
        .and_then(|m| m.systems.get(&target))
        .and_then(|s| s.metadata.star_temperature);
    temp.is_none_or(|t| t <= limit)
}

// =============================================================================
// Heat Safety Predicates
// =============================================================================

/// Parameters for heat safety evaluation.
pub(crate) struct HeatSafetyContext<'a> {
    pub ship: &'a ShipAttributes,
    pub loadout: &'a ShipLoadout,
    pub heat_config: HeatConfig,
    pub starmap: Option<&'a Starmap>,
}

/// Evaluate whether a spatial hop is safe from a heat perspective.
/// Returns `true` if the hop won't result in critical engine state.
fn hop_meets_heat_safety(edge: &Edge, target: SystemId, ctx: &HeatSafetyContext<'_>) -> bool {
    // Get ambient temperature at destination (min_external_temp, not star surface)
    let ambient_temp = ctx
        .starmap
        .and_then(|m| m.systems.get(&target))
        .and_then(|s| s.metadata.min_external_temp)
        .unwrap_or(0.0);

    let mass = ctx.loadout.total_mass_kg(ctx.ship);
    let hop_energy = calculate_jump_heat(
        mass,
        edge.distance,
        ctx.ship.base_mass_kg,
        ctx.heat_config.calibration_constant,
    );

    match hop_energy {
        Ok(energy) => {
            let hop_heat = energy / (mass * ctx.ship.specific_heat);
            let total_heat = ambient_temp + hop_heat;

            if total_heat >= HEAT_CRITICAL {
                let to_name = ctx
                    .starmap
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
            true
        }
        Err(e) => {
            // Conservative fail-safe: reject on calculation error
            let to_name = ctx
                .starmap
                .and_then(|m| m.systems.get(&target))
                .map(|s| s.name.as_str())
                .unwrap_or("unknown");
            tracing::warn!(
                "heat calculation failed for {}: {e:#?}; rejecting edge as conservative fail-safe",
                to_name
            );
            false
        }
    }
}

/// Wrapper to handle the optional heat safety check with proper error handling.
fn check_heat_safety(
    edge: &Edge,
    target: SystemId,
    constraints: &PathConstraints,
    starmap: Option<&Starmap>,
) -> bool {
    // Only applies to spatial edges
    if edge.kind != EdgeKind::Spatial {
        return true;
    }

    if !constraints.avoid_critical_state {
        return true;
    }

    // Validate required configuration
    let (Some(ship), Some(loadout)) = (&constraints.ship, &constraints.loadout) else {
        tracing::error!("avoid_critical_state requested but missing ship/loadout; rejecting edge");
        return false;
    };

    let Some(heat_config) = constraints.heat_config else {
        tracing::error!(
            "heat_config must be set when avoid_critical_state is true; rejecting edge"
        );
        return false;
    };

    let ctx = HeatSafetyContext {
        ship,
        loadout,
        heat_config,
        starmap,
    };

    hop_meets_heat_safety(edge, target, &ctx)
}

// =============================================================================
// PathConstraints
// =============================================================================

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
    /// Check if an edge to a target system is allowed under these constraints.
    ///
    /// This method composes multiple predicate checks:
    /// 1. Edge distance limit (for spatial jumps)
    /// 2. Gate avoidance policy
    /// 3. System avoidance list
    /// 4. Temperature constraints
    /// 5. Heat safety (avoid critical engine state)
    pub(crate) fn allows(&self, starmap: Option<&Starmap>, edge: &Edge, target: SystemId) -> bool {
        // Check edge predicates
        if !edge_meets_distance_limit(edge, self.max_jump) {
            return false;
        }

        if !edge_meets_gate_policy(edge, self.avoid_gates) {
            return false;
        }

        // Check system predicates
        if !system_meets_avoidance(target, &self.avoided_systems) {
            return false;
        }

        if !system_meets_temperature(edge, starmap, target, self.max_temperature) {
            return false;
        }

        // Check heat safety
        if !check_heat_safety(edge, target, self, starmap) {
            return false;
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

/// Run Dijkstra's algorithm where edge costs are measured in fuel units instead
/// of distance. Gate traversals have zero fuel cost; spatial hops compute fuel
/// using `calculate_jump_fuel_cost` with a static total mass approximation.
pub fn find_route_dijkstra_fuel(
    graph: &Graph,
    starmap: Option<&Starmap>,
    start: SystemId,
    goal: SystemId,
    constraints: &PathConstraints,
    total_mass_kg: f64,
    fuel_config: &crate::ship::FuelConfig,
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

            // Compute fuel cost: gates are free, spatial edges consume fuel.
            let edge_cost = match edge.kind {
                EdgeKind::Gate => 0.0,
                EdgeKind::Spatial => match crate::ship::calculate_jump_fuel_cost(
                    total_mass_kg,
                    edge.distance,
                    fuel_config,
                ) {
                    Ok(c) => c,
                    Err(_) => {
                        // Conservative: if fuel calc fails, reject the edge
                        continue;
                    }
                },
            };

            let next_cost = current_distance + edge_cost;
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
    use crate::graph::{Edge, EdgeKind};

    // =========================================================================
    // Predicate Unit Tests
    // =========================================================================

    #[test]
    fn edge_meets_distance_limit_no_constraint() {
        let edge = Edge {
            target: 1,
            kind: EdgeKind::Spatial,
            distance: 100.0,
        };
        assert!(edge_meets_distance_limit(&edge, None));
    }

    #[test]
    fn edge_meets_distance_limit_within_limit() {
        let edge = Edge {
            target: 1,
            kind: EdgeKind::Spatial,
            distance: 50.0,
        };
        assert!(edge_meets_distance_limit(&edge, Some(60.0)));
    }

    #[test]
    fn edge_meets_distance_limit_exceeds_limit() {
        let edge = Edge {
            target: 1,
            kind: EdgeKind::Spatial,
            distance: 100.0,
        };
        assert!(!edge_meets_distance_limit(&edge, Some(60.0)));
    }

    #[test]
    fn edge_meets_distance_limit_gate_ignores_limit() {
        let edge = Edge {
            target: 1,
            kind: EdgeKind::Gate,
            distance: 1000.0, // far exceeds any jump limit
        };
        assert!(edge_meets_distance_limit(&edge, Some(10.0)));
    }

    #[test]
    fn edge_meets_gate_policy_allows_gates() {
        let gate = Edge {
            target: 1,
            kind: EdgeKind::Gate,
            distance: 0.0,
        };
        assert!(edge_meets_gate_policy(&gate, false));
    }

    #[test]
    fn edge_meets_gate_policy_blocks_gates() {
        let gate = Edge {
            target: 1,
            kind: EdgeKind::Gate,
            distance: 0.0,
        };
        assert!(!edge_meets_gate_policy(&gate, true));
    }

    #[test]
    fn edge_meets_gate_policy_allows_spatial_when_avoiding_gates() {
        let spatial = Edge {
            target: 1,
            kind: EdgeKind::Spatial,
            distance: 10.0,
        };
        assert!(edge_meets_gate_policy(&spatial, true));
    }

    #[test]
    fn system_meets_avoidance_not_avoided() {
        let avoided = HashSet::new();
        assert!(system_meets_avoidance(1, &avoided));
    }

    #[test]
    fn system_meets_avoidance_is_avoided() {
        let mut avoided = HashSet::new();
        avoided.insert(1);
        assert!(!system_meets_avoidance(1, &avoided));
    }

    #[test]
    fn system_meets_temperature_no_constraint() {
        let edge = Edge {
            target: 1,
            kind: EdgeKind::Spatial,
            distance: 10.0,
        };
        assert!(system_meets_temperature(&edge, None, 1, None));
    }

    #[test]
    fn system_meets_temperature_gate_ignores_limit() {
        let edge = Edge {
            target: 1,
            kind: EdgeKind::Gate,
            distance: 0.0,
        };
        // Gate edges should always pass temperature check
        assert!(system_meets_temperature(&edge, None, 1, Some(100.0)));
    }

    // =========================================================================
    // PathConstraints Tests
    // =========================================================================

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

    #[test]
    fn dijkstra_fuel_prefers_gate_route_when_cheaper() {
        use crate::db::{Starmap, System, SystemPosition};
        use crate::ship::{FuelConfig, ShipAttributes, ShipLoadout};

        // A spatial shortcut exists (A->C) shorter than A->B->C; distance-Dijkstra
        // should pick A->C, but fuel optimization (gates free) should pick A->B->C.
        let a = System {
            id: 1,
            name: "A".to_string(),
            metadata: crate::db::SystemMetadata {
                constellation_id: None,
                constellation_name: None,
                region_id: None,
                region_name: None,
                security_status: None,
                star_temperature: None,
                star_luminosity: None,
                min_external_temp: None,
                planet_count: None,
                moon_count: None,
            },
            position: SystemPosition::new(0.0, 0.0, 0.0),
        };
        let b = System {
            id: 2,
            name: "B".to_string(),
            metadata: crate::db::SystemMetadata {
                constellation_id: None,
                constellation_name: None,
                region_id: None,
                region_name: None,
                security_status: None,
                star_temperature: None,
                star_luminosity: None,
                min_external_temp: None,
                planet_count: None,
                moon_count: None,
            },
            position: SystemPosition::new(0.0, 150.0, 0.0),
        };
        let c = System {
            id: 3,
            name: "C".to_string(),
            metadata: crate::db::SystemMetadata {
                constellation_id: None,
                constellation_name: None,
                region_id: None,
                region_name: None,
                security_status: None,
                star_temperature: None,
                star_luminosity: None,
                min_external_temp: None,
                planet_count: None,
                moon_count: None,
            },
            position: SystemPosition::new(120.0, 0.0, 0.0),
        };

        let mut systems = std::collections::HashMap::new();
        systems.insert(a.id, a.clone());
        systems.insert(b.id, b.clone());
        systems.insert(c.id, c.clone());

        let mut name_to_id = std::collections::HashMap::new();
        name_to_id.insert(a.name.clone(), a.id);
        name_to_id.insert(b.name.clone(), b.id);
        name_to_id.insert(c.name.clone(), c.id);

        let mut adj = std::collections::HashMap::new();
        adj.insert(a.id, vec![b.id]);
        adj.insert(b.id, vec![a.id, c.id]);
        adj.insert(c.id, vec![b.id]);

        let starmap = Starmap {
            systems,
            name_to_id,
            adjacency: std::sync::Arc::new(adj),
        };

        let graph = crate::graph::build_hybrid_graph(&starmap);

        let route_distance =
            find_route_dijkstra(&graph, Some(&starmap), a.id, c.id, &Default::default())
                .expect("route found");
        assert_eq!(route_distance, vec![a.id, c.id]);

        let ship = ShipAttributes {
            name: "TestShip".to_string(),
            base_mass_kg: 1e6,
            specific_heat: 1.0,
            fuel_capacity: 1000.0,
            cargo_capacity: 1000.0,
        };
        let loadout = ShipLoadout::new(&ship, 500.0, 0.0).expect("valid loadout");
        let fuel_cfg = FuelConfig::default();
        let mass = loadout.total_mass_kg(&ship);

        let route_fuel = find_route_dijkstra_fuel(
            &graph,
            Some(&starmap),
            a.id,
            c.id,
            &Default::default(),
            mass,
            &fuel_cfg,
        )
        .expect("fuel route found");
        assert_eq!(route_fuel, vec![a.id, b.id, c.id]);
    }
}
