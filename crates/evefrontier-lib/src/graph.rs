use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;

use crate::db::{Starmap, SystemId, SystemPosition};

/// Cap the spatial graph fan-out so each system only links to its closest
/// neighbours. Twelve mirrors the maximum number of bidirectional stargate
/// connections observed in the production dataset, which keeps spatial routing
/// performant without starving dense areas of nearby candidates.
const MAX_SPATIAL_NEIGHBORS: usize = 12;

/// Routing graph variants supported by the planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphMode {
    Gate,
    Spatial,
    Hybrid,
}

/// Classification for the edge used in the routing graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EdgeKind {
    Gate,
    Spatial,
}

/// Edge within the routing graph.
#[derive(Debug, Clone)]
pub struct Edge {
    pub target: SystemId,
    pub kind: EdgeKind,
    pub distance: f64,
}

/// Graph structure used by pathfinding algorithms.
#[derive(Debug, Clone)]
pub struct Graph {
    mode: GraphMode,
    adjacency: Arc<HashMap<SystemId, Vec<Edge>>>,
}

impl Graph {
    /// Mode that produced this graph (gate, spatial, or hybrid).
    pub fn mode(&self) -> GraphMode {
        self.mode
    }

    /// Return the neighbours for a given system identifier.
    pub fn neighbours(&self, system: SystemId) -> &[Edge] {
        self.adjacency
            .get(&system)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self {
            mode: GraphMode::Gate,
            adjacency: Arc::new(HashMap::new()),
        }
    }
}

/// Build the default gate-based routing graph.
pub fn build_graph(starmap: &Starmap) -> Graph {
    build_gate_graph(starmap)
}

/// Build a routing graph that only considers gate edges.
pub fn build_gate_graph(starmap: &Starmap) -> Graph {
    Graph {
        mode: GraphMode::Gate,
        adjacency: Arc::new(build_gate_adjacency(starmap)),
    }
}

/// Build a routing graph that only considers spatial jumps.
pub fn build_spatial_graph(starmap: &Starmap) -> Graph {
    Graph {
        mode: GraphMode::Spatial,
        adjacency: Arc::new(build_spatial_adjacency(starmap)),
    }
}

/// Build a routing graph that combines gate and spatial edges.
pub fn build_hybrid_graph(starmap: &Starmap) -> Graph {
    let gate = build_gate_adjacency(starmap);
    let spatial = build_spatial_adjacency(starmap);
    let adjacency = merge_adjacency(starmap, gate, spatial);

    Graph {
        mode: GraphMode::Hybrid,
        adjacency: Arc::new(adjacency),
    }
}

fn build_gate_adjacency(starmap: &Starmap) -> HashMap<SystemId, Vec<Edge>> {
    let mut adjacency: HashMap<SystemId, Vec<Edge>> = HashMap::new();
    for &system_id in starmap.systems.keys() {
        let edges = starmap
            .adjacency
            .as_ref()
            .get(&system_id)
            .map(|targets| {
                targets
                    .iter()
                    .copied()
                    .map(|target| Edge {
                        target,
                        kind: EdgeKind::Gate,
                        distance: 1.0,
                    })
                    .collect()
            })
            .unwrap_or_default();
        adjacency.insert(system_id, edges);
    }
    adjacency
}

fn build_spatial_adjacency(starmap: &Starmap) -> HashMap<SystemId, Vec<Edge>> {
    let mut adjacency: HashMap<SystemId, Vec<Edge>> = HashMap::new();
    let positioned: Vec<(SystemId, SystemPosition)> = starmap
        .systems
        .values()
        .filter_map(|system| system.position.map(|pos| (system.id, pos)))
        .collect();

    if positioned.is_empty() {
        for &system_id in starmap.systems.keys() {
            adjacency.entry(system_id).or_default();
        }
        return adjacency;
    }

    for &(system_id, position) in &positioned {
        let mut edges: Vec<Edge> = positioned
            .iter()
            .filter(|(other_id, _)| *other_id != system_id)
            .map(|(other_id, other_position)| Edge {
                target: *other_id,
                kind: EdgeKind::Spatial,
                distance: position.distance_to(other_position),
            })
            .collect();

        edges.sort_by(|a, b| compare_distance(a.distance, b.distance));
        edges.truncate(MAX_SPATIAL_NEIGHBORS);

        adjacency.insert(system_id, edges);
    }

    for &system_id in starmap.systems.keys() {
        adjacency.entry(system_id).or_default();
    }

    adjacency
}

fn merge_adjacency(
    starmap: &Starmap,
    mut gate: HashMap<SystemId, Vec<Edge>>,
    spatial: HashMap<SystemId, Vec<Edge>>,
) -> HashMap<SystemId, Vec<Edge>> {
    for (system_id, spatial_edges) in spatial {
        let entry = gate.entry(system_id).or_default();
        for edge in spatial_edges {
            if let Some(existing) = entry
                .iter_mut()
                .find(|existing| existing.target == edge.target && existing.kind == edge.kind)
            {
                if edge.distance < existing.distance {
                    *existing = edge.clone();
                }
                continue;
            }
            entry.push(edge);
        }
        entry.sort_by(|a, b| {
            compare_distance(a.distance, b.distance).then_with(|| a.kind.cmp(&b.kind))
        });
    }

    for &system_id in starmap.systems.keys() {
        gate.entry(system_id).or_default();
    }

    gate
}

fn compare_distance(a: f64, b: f64) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Greater)
}
