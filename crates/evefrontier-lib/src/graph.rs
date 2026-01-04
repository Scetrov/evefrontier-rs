use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;

use tracing::warn;

use crate::db::{Starmap, SystemId, SystemPosition};
use crate::spatial::{NeighbourQuery, SpatialIndex};

/// Default maximum number of nearest neighbors to include in the spatial graph.
/// This limits the fan-out per node when constructing spatial graphs. The default is `0`,
/// which means "unlimited" neighbours (no truncation) — this avoids accidentally hiding
/// ship-capable long jumps behind a small fixed fan-out. The behaviour can be tuned via
/// the CLI flag `--max-spatial-neighbours`.
const DEFAULT_MAX_SPATIAL_NEIGHBORS: usize = 0;
/// For very large datasets, requesting unlimited neighbours can be pathological (O(n^2)).
/// To avoid hang/very long runs we cap the number of neighbours fetched from the index when
/// no `max_jump` radius is provided and the dataset is large.
const LARGE_DATASET_NEIGHBOUR_THRESHOLD: usize = 5_000;
/// Maximum neighbours to fetch per system when capping to avoid pathological behaviour.
const MAX_SAFE_NEIGHBOURS: usize = 5_000;

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

impl Graph {
    /// Create a Graph from explicit parts. This is crate-visible and useful for
    /// testing or for internal operations that need to construct a graph with
    /// a precomputed adjacency map (for example, a version with unsafe edges
    /// pruned).
    pub(crate) fn from_parts(
        mode: GraphMode,
        adjacency: std::collections::HashMap<SystemId, Vec<Edge>>,
    ) -> Self {
        Self {
            mode,
            adjacency: Arc::new(adjacency),
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
    build_spatial_graph_indexed(starmap, &GraphBuildOptions::default())
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
                    .map(|target| {
                        // Compute a physical distance for gate edges when both systems have
                        // valid positions. This ensures the hybrid graph combines edges
                        // using consistent units (light-years) rather than mixing a
                        // unitless hop-count (1.0) with spatial distances. Using physical
                        // distances prevents Dijkstra/A* from preferring many small gate
                        // hops simply because they were assigned a low unit cost.
                        let distance = if let (Some(from), Some(to)) = (
                            starmap.systems.get(&system_id).and_then(|s| s.position),
                            starmap.systems.get(&target).and_then(|s| s.position),
                        ) {
                            from.distance_to(&to)
                        } else {
                            // Fallback for systems without positions: keep previous behaviour
                            // to avoid changing semantics when coordinates are missing.
                            1.0
                        };

                        Edge {
                            target,
                            kind: EdgeKind::Gate,
                            distance,
                        }
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
        // If DEFAULT (0) or the configured max is 0, treat as unlimited (do not truncate).
        if DEFAULT_MAX_SPATIAL_NEIGHBORS != 0 {
            edges.truncate(DEFAULT_MAX_SPATIAL_NEIGHBORS);
        }

        adjacency.insert(system_id, edges);
    }

    for &system_id in starmap.systems.keys() {
        adjacency.entry(system_id).or_default();
    }

    adjacency
}

/// Merge spatial edges into gate adjacency for hybrid routing.
///
/// This function preserves both gate and spatial edges to the same target system,
/// as they represent different routing options. A "duplicate" is only an edge with
/// both the same target AND the same kind (e.g., two spatial edges to the same system).
/// When duplicates exist, we keep the edge with the shorter distance.
fn merge_adjacency(
    starmap: &Starmap,
    mut gate: HashMap<SystemId, Vec<Edge>>,
    spatial: HashMap<SystemId, Vec<Edge>>,
) -> HashMap<SystemId, Vec<Edge>> {
    for (system_id, spatial_edges) in spatial {
        let entry = gate.entry(system_id).or_default();
        for edge in spatial_edges {
            // Check for duplicate: same target AND same kind
            // Note: A spatial edge to X is NOT considered duplicate of a gate edge to X
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

/// Options for building spatial or hybrid graphs with index support.
#[derive(Debug, Clone)]
pub struct GraphBuildOptions {
    /// Pre-built spatial index. If None and spatial edges are needed,
    /// the index will be built automatically (with a warning).
    pub spatial_index: Option<Arc<SpatialIndex>>,
    /// Maximum jump distance for spatial edges (light-years).
    pub max_jump: Option<f64>,
    /// Maximum temperature for spatial jump targets (Kelvin).
    /// Systems with min_external_temp above this are excluded.
    pub max_temperature: Option<f64>,
    /// Maximum number of nearest neighbours to include for spatial edges.
    pub max_spatial_neighbors: usize,
}

impl Default for GraphBuildOptions {
    fn default() -> Self {
        Self {
            spatial_index: None,
            max_jump: None,
            max_temperature: None,
            max_spatial_neighbors: DEFAULT_MAX_SPATIAL_NEIGHBORS,
        }
    }
}

/// Build a routing graph that only considers spatial jumps, using a spatial index.
///
/// If no index is provided, builds one automatically (with a warning for large datasets).
pub fn build_spatial_graph_indexed(starmap: &Starmap, options: &GraphBuildOptions) -> Graph {
    let index = get_or_build_index(starmap, options);
    let adjacency = build_spatial_adjacency_indexed(starmap, &index, options);

    Graph {
        mode: GraphMode::Spatial,
        adjacency: Arc::new(adjacency),
    }
}

/// Build a routing graph that combines gate and spatial edges, using a spatial index.
///
/// If no index is provided, builds one automatically (with a warning for large datasets).
pub fn build_hybrid_graph_indexed(starmap: &Starmap, options: &GraphBuildOptions) -> Graph {
    let index = get_or_build_index(starmap, options);
    let gate = build_gate_adjacency(starmap);
    let spatial = build_spatial_adjacency_indexed(starmap, &index, options);
    let adjacency = merge_adjacency(starmap, gate, spatial);

    Graph {
        mode: GraphMode::Hybrid,
        adjacency: Arc::new(adjacency),
    }
}

fn get_or_build_index(starmap: &Starmap, options: &GraphBuildOptions) -> Arc<SpatialIndex> {
    if let Some(ref index) = options.spatial_index {
        return Arc::clone(index);
    }

    let system_count = starmap.systems.len();
    if system_count > 100 {
        warn!(
            systems = system_count,
            "spatial index not provided, building in-memory (this may be slow for large datasets)"
        );
    }

    Arc::new(SpatialIndex::build(starmap))
}

fn build_spatial_adjacency_indexed(
    starmap: &Starmap,
    index: &SpatialIndex,
    options: &GraphBuildOptions,
) -> HashMap<SystemId, Vec<Edge>> {
    let mut adjacency: HashMap<SystemId, Vec<Edge>> = HashMap::new();

    for system in starmap.systems.values() {
        let Some(position) = system.position else {
            adjacency.entry(system.id).or_default();
            continue;
        };

        let query_point = [position.x, position.y, position.z];
        let max_neighbors = options.max_spatial_neighbors;

        // If the caller provided an explicit radius (`max_jump`) use an efficient
        // radius query which returns only systems within that distance. This avoids
        // fetching the whole dataset when a physical per-hop limit is known.
        let neighbors: Vec<(SystemId, f64)> = if let Some(radius) = options.max_jump {
            index.within_radius_filtered(query_point, radius, options.max_temperature)
        } else if max_neighbors == 0 {
            // Unlimited neighbours requested but no radius provided. For small datasets
            // we can safely fetch all neighbours; for very large datasets this becomes
            // O(n^2) and can take an exceptionally long time. Cap the fetch to a
            // reasonable upper bound and emit a warning so callers can tune behaviour
            // with `--max-spatial-neighbours` or `--max-jump`.
            let system_count = starmap.systems.len();
            if system_count > LARGE_DATASET_NEIGHBOUR_THRESHOLD {
                warn!(
                    systems = system_count,
                    cap = MAX_SAFE_NEIGHBOURS,
                    "unlimited spatial neighbours requested on large dataset; capping neighbours per-node for performance; consider using --max-spatial-neighbours or --max-jump"
                );
                let k = MAX_SAFE_NEIGHBOURS + 1; // +1 to account for self
                let query = NeighbourQuery {
                    k,
                    radius: None,
                    max_temperature: options.max_temperature,
                };
                index.nearest_filtered(query_point, &query)
            } else {
                // small dataset: fetch all
                let k = system_count;
                let query = NeighbourQuery {
                    k,
                    radius: None,
                    max_temperature: options.max_temperature,
                };
                index.nearest_filtered(query_point, &query)
            }
        } else {
            // Bounded number of neighbours requested
            let k = max_neighbors + 1; // +1 to account for self
            let query = NeighbourQuery {
                k,
                radius: None,
                max_temperature: options.max_temperature,
            };
            index.nearest_filtered(query_point, &query)
        };

        let iter = neighbors.into_iter().filter(|(id, _)| *id != system.id); // Exclude self

        let edges: Vec<Edge> = if max_neighbors == 0
            && options.max_jump.is_none()
            && starmap.systems.len() > LARGE_DATASET_NEIGHBOUR_THRESHOLD
        {
            // We already capped neighbor fetch above; use all returned edges
            iter.map(|(target, distance)| Edge {
                target,
                kind: EdgeKind::Spatial,
                distance,
            })
            .collect()
        } else if max_neighbors == 0 {
            // Unlimited: use whatever the radius query returned
            iter.map(|(target, distance)| Edge {
                target,
                kind: EdgeKind::Spatial,
                distance,
            })
            .collect()
        } else {
            iter.take(max_neighbors)
                .map(|(target, distance)| Edge {
                    target,
                    kind: EdgeKind::Spatial,
                    distance,
                })
                .collect()
        };

        adjacency.insert(system.id, edges);
    }

    // Ensure all systems have an entry
    for &system_id in starmap.systems.keys() {
        adjacency.entry(system_id).or_default();
    }

    adjacency
}

// Tests for gate distance semantics and hybrid routing behaviour
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Starmap, System, SystemMetadata, SystemPosition};
    use crate::path::find_route_dijkstra;

    #[test]
    fn gate_edges_use_physical_distance_when_positions_present() {
        // Build two systems with positions and a gate between them
        let a = System {
            id: 1,
            name: "A".to_string(),
            metadata: SystemMetadata {
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
            metadata: SystemMetadata {
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
            position: SystemPosition::new(3.0, 4.0, 0.0),
        };

        let mut systems = std::collections::HashMap::new();
        systems.insert(a.id, a.clone());
        systems.insert(b.id, b.clone());

        let mut name_to_id = std::collections::HashMap::new();
        name_to_id.insert(a.name.clone(), a.id);
        name_to_id.insert(b.name.clone(), b.id);

        let mut adj = std::collections::HashMap::new();
        adj.insert(a.id, vec![b.id]);
        adj.insert(b.id, vec![a.id]);

        let starmap = Starmap {
            systems,
            name_to_id,
            adjacency: std::sync::Arc::new(adj),
        };

        let gate_adj = build_gate_adjacency(&starmap);
        let edges = gate_adj.get(&a.id).expect("adjacency present");
        assert_eq!(edges.len(), 1);
        // Distance between (0,0,0) and (3,4,0) is 5
        assert!((edges[0].distance - 5.0).abs() < 1e-9);
    }

    #[test]
    fn dijkstra_prefers_spatial_shortcut_over_many_gate_hops() {
        // Construct three systems A,B,C where gates connect A->B->C but a spatial
        // shortcut A->C exists that is shorter in physical distance than the
        // sum of gate distances.
        let a = System {
            id: 1,
            name: "A".to_string(),
            metadata: SystemMetadata {
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
        // Place B far off so A->B + B->C > A->C
        let b = System {
            id: 2,
            name: "B".to_string(),
            metadata: SystemMetadata {
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
            position: SystemPosition::new(0.0, 50.0, 0.0),
        };
        let c = System {
            id: 3,
            name: "C".to_string(),
            metadata: SystemMetadata {
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
            position: SystemPosition::new(100.0, 0.0, 0.0),
        };

        let mut systems = std::collections::HashMap::new();
        systems.insert(a.id, a.clone());
        systems.insert(b.id, b.clone());
        systems.insert(c.id, c.clone());

        let mut name_to_id = std::collections::HashMap::new();
        name_to_id.insert(a.name.clone(), a.id);
        name_to_id.insert(b.name.clone(), b.id);
        name_to_id.insert(c.name.clone(), c.id);

        // Gate adjacency A<->B, B<->C (no direct gate A<->C)
        let mut adj = std::collections::HashMap::new();
        adj.insert(a.id, vec![b.id]);
        adj.insert(b.id, vec![a.id, c.id]);
        adj.insert(c.id, vec![b.id]);

        let starmap = Starmap {
            systems,
            name_to_id,
            adjacency: std::sync::Arc::new(adj),
        };

        // Build hybrid graph which includes spatial A->C edge
        let graph = build_hybrid_graph(&starmap);

        // Run Dijkstra from A to C; with gate edges weighted by physical distance
        // the expected chosen route should be the direct spatial edge A->C.
        let route = find_route_dijkstra(&graph, Some(&starmap), a.id, c.id, &Default::default())
            .expect("route found");

        assert_eq!(route, vec![a.id, c.id]);
    }

    #[test]
    fn default_max_spatial_neighbors_is_unlimited() {
        assert_eq!(
            GraphBuildOptions::default().max_spatial_neighbors,
            DEFAULT_MAX_SPATIAL_NEIGHBORS
        );
    }
}
