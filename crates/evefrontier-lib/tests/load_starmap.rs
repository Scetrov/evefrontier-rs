use std::path::PathBuf;

use evefrontier_lib::{build_graph, find_route, load_starmap, Result};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/minimal_static_data.db")
}

#[test]
fn load_fixture_and_find_route() -> Result<()> {
    let path = fixture_path();
    let starmap = load_starmap(&path)?;

    assert_eq!(starmap.systems.len(), 3);
    assert_eq!(starmap.adjacency.len(), 3);

    let start = starmap.system_id_by_name("Y:170N").expect("start exists");
    let goal = starmap.system_id_by_name("BetaTest").expect("goal exists");

    let graph = build_graph(&starmap);
    let route = find_route(&graph, start, goal).expect("route should exist");

    assert_eq!(route.first().copied(), Some(start));
    assert_eq!(route.last().copied(), Some(goal));
    assert!(route.len() >= 2);

    Ok(())
}
