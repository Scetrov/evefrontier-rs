use std::collections::HashMap;
use std::sync::Arc;

use evefrontier_lib::{
    build_gate_graph, build_hybrid_graph, build_spatial_graph, load_starmap, EdgeKind, GraphMode,
    Starmap, System, SystemMetadata, SystemPosition,
};

fn fixture_starmap() -> Starmap {
    let mut systems = HashMap::new();
    systems.insert(
        1,
        System {
            id: 1,
            name: "Alpha".to_string(),
            metadata: empty_metadata(),
            position: Some(SystemPosition {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
        },
    );
    systems.insert(
        2,
        System {
            id: 2,
            name: "Beta".to_string(),
            metadata: empty_metadata(),
            position: Some(SystemPosition {
                x: 3.0,
                y: 0.0,
                z: 0.0,
            }),
        },
    );
    systems.insert(
        3,
        System {
            id: 3,
            name: "Gamma".to_string(),
            metadata: empty_metadata(),
            position: None,
        },
    );

    let name_to_id = systems
        .values()
        .map(|system| (system.name.clone(), system.id))
        .collect();

    let adjacency = Arc::new(HashMap::from([(1, vec![2, 3]), (2, vec![1]), (3, vec![1])]));

    Starmap {
        systems,
        name_to_id,
        adjacency,
    }
}

fn empty_metadata() -> SystemMetadata {
    SystemMetadata {
        constellation_id: None,
        constellation_name: None,
        region_id: None,
        region_name: None,
        security_status: None,
        star_temperature: None,
        star_luminosity: None,
        min_external_temp: None,
    }
}

#[test]
fn gate_graph_retains_gate_edges() {
    let starmap = fixture_starmap();
    let graph = build_gate_graph(&starmap);

    assert_eq!(graph.mode(), GraphMode::Gate);
    let edges: Vec<_> = graph
        .neighbours(1)
        .iter()
        .map(|edge| (edge.target, edge.kind))
        .collect();
    assert_eq!(edges, vec![(2, EdgeKind::Gate), (3, EdgeKind::Gate)]);
}

#[test]
fn spatial_graph_uses_positions() {
    let starmap = fixture_starmap();
    let graph = build_spatial_graph(&starmap);

    assert_eq!(graph.mode(), GraphMode::Spatial);
    let edges = graph.neighbours(1);
    assert_eq!(edges.len(), 1, "only positioned neighbours are connected");
    assert_eq!(edges[0].target, 2);
    assert_eq!(edges[0].kind, EdgeKind::Spatial);
    assert!(edges[0].distance > 0.0);

    assert!(
        graph.neighbours(3).is_empty(),
        "missing positions drop edges"
    );
}

#[test]
fn hybrid_graph_combines_edges() {
    let starmap = fixture_starmap();
    let graph = build_hybrid_graph(&starmap);

    assert_eq!(graph.mode(), GraphMode::Hybrid);

    let mut gate_targets = Vec::new();
    let mut spatial_targets = Vec::new();
    for edge in graph.neighbours(1) {
        match edge.kind {
            EdgeKind::Gate => gate_targets.push(edge.target),
            EdgeKind::Spatial => spatial_targets.push(edge.target),
        }
    }

    gate_targets.sort_unstable();
    spatial_targets.sort_unstable();

    assert_eq!(gate_targets, vec![2, 3]);
    assert_eq!(spatial_targets, vec![2]);
}

#[test]
fn spatial_graph_has_edges_with_positions_in_fixture() {
    let starmap = load_starmap(&fixture_path()).expect("load fixture");
    let graph = build_spatial_graph(&starmap);

    let start = starmap.system_id_by_name("Nod").unwrap();
    let neighbours = graph.neighbours(start);
    assert!(
        !neighbours.is_empty(),
        "fixture now includes coordinates; spatial graph should have edges"
    );
}

fn fixture_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/fixtures/minimal_static_data.db")
}
