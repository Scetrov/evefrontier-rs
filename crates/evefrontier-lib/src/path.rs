use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

use crate::db::{Starmap, SystemId};
use crate::graph::{Edge, EdgeKind, Graph};

/// Constraints applied during pathfinding.
#[derive(Debug, Default, Clone)]
pub struct PathConstraints {
    /// Maximum distance allowed for any single edge.
    pub max_jump: Option<f64>,
    /// Skip gate edges entirely when `true`.
    pub avoid_gates: bool,
    /// Set of system identifiers that must not appear in the resulting path.
    pub avoided_systems: HashSet<SystemId>,
    /// Maximum allowed system temperature in Kelvin.
    pub max_temperature: Option<f64>,
}

impl PathConstraints {
    fn allows(&self, starmap: Option<&Starmap>, edge: &Edge, target: SystemId) -> bool {
        if let Some(limit) = self.max_jump {
            if edge.distance > limit {
                return false;
            }
        }

        if self.avoid_gates && edge.kind == EdgeKind::Gate {
            return false;
        }

        if self.avoided_systems.contains(&target) {
            return false;
        }

        if let Some(limit) = self.max_temperature {
            if let Some(map) = starmap {
                if let Some(system) = map.systems.get(&target) {
                    if let Some(temperature) = system.metadata.temperature {
                        if temperature > limit {
                            return false;
                        }
                    }
                }
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
        let current_distance = match distances.get(&entry.node) {
            Some(distance) if (*distance - entry.cost.0).abs() < f64::EPSILON => *distance,
            Some(distance) if *distance < entry.cost.0 => continue,
            Some(distance) => *distance,
            None => continue,
        };

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
        let current_score = match g_score.get(&entry.node) {
            Some(score) if (*score - entry.cost.0).abs() < f64::EPSILON => *score,
            Some(score) if *score < entry.cost.0 => continue,
            Some(score) => *score,
            None => continue,
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
