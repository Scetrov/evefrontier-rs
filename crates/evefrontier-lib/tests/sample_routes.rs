//! Integration tests validating routes against SampleRoutes.csv discovered paths.
//!
//! This test module uses the `route_testing.db` fixture which contains 717 systems
//! that cover ~50% of the routes in SampleRoutes.csv. It validates that our pathfinding
//! implementation can find valid routes between the same endpoints.

use evefrontier_lib::{
    find_route_a_star, find_route_bfs, find_route_dijkstra,
    graph::{build_gate_graph, build_hybrid_graph, build_spatial_graph},
    load_starmap, PathConstraints, Starmap,
};
use serde::Deserialize;
use std::path::PathBuf;

/// A test route from testable_routes.json
#[derive(Debug, Deserialize)]
struct TestRoute {
    #[allow(dead_code)]
    route_id: u32,
    start: String,
    end: String,
    avoid_gates: bool,
    max_ly: f64,
    expected_length: usize,
}

/// Container for testable routes
#[derive(Debug, Deserialize)]
struct TestableRoutes {
    #[allow(dead_code)]
    description: String,
    #[allow(dead_code)]
    total_routes: usize,
    #[allow(dead_code)]
    testable_routes: usize,
    #[allow(dead_code)]
    coverage_percent: f64,
    routes: Vec<TestRoute>,
}

fn fixture_db_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("docs/fixtures/route_testing.db")
}

fn testable_routes_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("docs/fixtures/testable_routes.json")
}

fn load_testable_routes() -> TestableRoutes {
    let path = testable_routes_path();
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e))
}

fn resolve_system_id(starmap: &Starmap, name: &str) -> Option<i64> {
    starmap.system_id_by_name(name)
}

/// Test that gate-only routes can find paths for routes that don't avoid gates
/// Note: Many routes in the fixture may not be fully connected via gates
/// since the fixture only contains a subset of systems. We only check that
/// the routing algorithm works correctly on the available connections.
#[test]
fn test_gate_only_routes_find_paths() {
    let db_path = fixture_db_path();
    if !db_path.exists() {
        eprintln!(
            "Skipping test: route_testing.db not found at {}",
            db_path.display()
        );
        return;
    }

    let starmap = load_starmap(&db_path).expect("Failed to load starmap");
    let graph = build_gate_graph(&starmap);
    let testable = load_testable_routes();
    let constraints = PathConstraints::default();

    // Filter to gate-only routes (avoid_gates = false)
    let gate_routes: Vec<_> = testable.routes.iter().filter(|r| !r.avoid_gates).collect();

    let mut found = 0;
    let mut not_found = 0;
    let mut missing_systems = 0;

    for route in &gate_routes {
        let start_id = match resolve_system_id(&starmap, &route.start) {
            Some(id) => id,
            None => {
                missing_systems += 1;
                continue;
            }
        };

        let end_id = match resolve_system_id(&starmap, &route.end) {
            Some(id) => id,
            None => {
                missing_systems += 1;
                continue;
            }
        };

        match find_route_bfs(&graph, Some(&starmap), start_id, end_id, &constraints) {
            Some(_path) => found += 1,
            None => not_found += 1,
        }
    }

    println!("\nGate-only route results:");
    println!("  Total gate routes: {}", gate_routes.len());
    println!("  Found: {}", found);
    println!("  Not found: {}", not_found);
    println!("  Missing systems: {}", missing_systems);

    // We expect to find at least some routes - the fixture is a subset so not all will work
    // Gate routes require full connectivity between start and end via gates only
    assert!(
        found > 0,
        "Expected to find at least some gate routes, found {}",
        found
    );
}

/// Test that spatial routing can find paths when gates are avoided
#[test]
fn test_spatial_routes_find_paths() {
    let db_path = fixture_db_path();
    if !db_path.exists() {
        eprintln!(
            "Skipping test: route_testing.db not found at {}",
            db_path.display()
        );
        return;
    }

    let starmap = load_starmap(&db_path).expect("Failed to load starmap");
    let graph = build_spatial_graph(&starmap);
    let testable = load_testable_routes();

    // Filter to spatial routes (avoid_gates = true)
    let spatial_routes: Vec<_> = testable.routes.iter().filter(|r| r.avoid_gates).collect();

    let mut found = 0;
    let mut not_found = 0;
    let mut missing_systems = 0;

    for route in &spatial_routes {
        let start_id = match resolve_system_id(&starmap, &route.start) {
            Some(id) => id,
            None => {
                missing_systems += 1;
                continue;
            }
        };

        let end_id = match resolve_system_id(&starmap, &route.end) {
            Some(id) => id,
            None => {
                missing_systems += 1;
                continue;
            }
        };

        // Build constraints with the route's max_ly limit
        let constraints = PathConstraints {
            max_jump: Some(route.max_ly),
            avoid_gates: true,
            ..Default::default()
        };

        match find_route_dijkstra(&graph, Some(&starmap), start_id, end_id, &constraints) {
            Some(_path) => found += 1,
            None => not_found += 1,
        }
    }

    println!("\nSpatial route results:");
    println!("  Total spatial routes: {}", spatial_routes.len());
    println!("  Found: {}", found);
    println!("  Not found: {}", not_found);
    println!("  Missing systems: {}", missing_systems);

    // We expect to find most routes
    let success_rate = found as f64 / spatial_routes.len().max(1) as f64;
    assert!(
        success_rate > 0.3,
        "Expected to find at least 30% of spatial routes, found {:.1}%",
        success_rate * 100.0
    );
}

/// Test that hybrid routing can find paths for mixed routes
#[test]
fn test_hybrid_routes_find_paths() {
    let db_path = fixture_db_path();
    if !db_path.exists() {
        eprintln!(
            "Skipping test: route_testing.db not found at {}",
            db_path.display()
        );
        return;
    }

    let starmap = load_starmap(&db_path).expect("Failed to load starmap");
    let graph = build_hybrid_graph(&starmap);
    let testable = load_testable_routes();

    let mut found = 0;
    let mut not_found = 0;
    let mut missing_systems = 0;

    // Test all routes with hybrid routing
    for route in &testable.routes {
        let start_id = match resolve_system_id(&starmap, &route.start) {
            Some(id) => id,
            None => {
                missing_systems += 1;
                continue;
            }
        };

        let end_id = match resolve_system_id(&starmap, &route.end) {
            Some(id) => id,
            None => {
                missing_systems += 1;
                continue;
            }
        };

        // Build constraints with the route's max_ly limit and avoid_gates flag
        let constraints = PathConstraints {
            max_jump: Some(route.max_ly),
            avoid_gates: route.avoid_gates,
            ..Default::default()
        };

        match find_route_a_star(&graph, Some(&starmap), start_id, end_id, &constraints) {
            Some(_path) => found += 1,
            None => not_found += 1,
        }
    }

    println!("\nHybrid route results:");
    println!("  Total routes: {}", testable.routes.len());
    println!("  Found: {}", found);
    println!("  Not found: {}", not_found);
    println!("  Missing systems: {}", missing_systems);

    // We expect to find most routes with hybrid routing
    let success_rate = found as f64 / testable.routes.len() as f64;
    assert!(
        success_rate > 0.5,
        "Expected to find at least 50% of routes with hybrid routing, found {:.1}%",
        success_rate * 100.0
    );
}

/// Test that route lengths are reasonable compared to expected
#[test]
fn test_route_length_sanity() {
    let db_path = fixture_db_path();
    if !db_path.exists() {
        eprintln!(
            "Skipping test: route_testing.db not found at {}",
            db_path.display()
        );
        return;
    }

    let starmap = load_starmap(&db_path).expect("Failed to load starmap");
    let graph = build_hybrid_graph(&starmap);
    let testable = load_testable_routes();

    let mut within_bounds = 0;
    let mut too_long = 0;
    let mut not_found = 0;

    // Sample some routes to check length
    for route in testable.routes.iter().take(100) {
        let start_id = match resolve_system_id(&starmap, &route.start) {
            Some(id) => id,
            None => continue,
        };

        let end_id = match resolve_system_id(&starmap, &route.end) {
            Some(id) => id,
            None => continue,
        };

        let constraints = PathConstraints {
            max_jump: Some(route.max_ly),
            avoid_gates: route.avoid_gates,
            ..Default::default()
        };

        match find_route_a_star(&graph, Some(&starmap), start_id, end_id, &constraints) {
            Some(path) => {
                // Our path should be at most 2x the expected length
                // (since we may take different but valid routes)
                if path.len() <= route.expected_length * 2 {
                    within_bounds += 1;
                } else {
                    too_long += 1;
                }
            }
            None => not_found += 1,
        }
    }

    println!("\nRoute length sanity check (first 100):");
    println!("  Within bounds (<= 2x expected): {}", within_bounds);
    println!("  Too long (> 2x expected): {}", too_long);
    println!("  Not found: {}", not_found);

    // Most found routes should be within reasonable bounds
    let found_total = within_bounds + too_long;
    if found_total > 0 {
        let within_rate = within_bounds as f64 / found_total as f64;
        assert!(
            within_rate > 0.5,
            "Expected at least 50% of routes to be within 2x expected length, got {:.1}%",
            within_rate * 100.0
        );
    }
}

/// Test specific known routes from the sample set
/// These routes are chosen from actual discovered paths in the fixture
#[test]
fn test_known_routes() {
    let db_path = fixture_db_path();
    if !db_path.exists() {
        eprintln!(
            "Skipping test: route_testing.db not found at {}",
            db_path.display()
        );
        return;
    }

    let starmap = load_starmap(&db_path).expect("Failed to load starmap");
    let graph = build_hybrid_graph(&starmap);
    let constraints = PathConstraints {
        max_jump: Some(80.0),
        ..Default::default()
    };

    // Test routes that should work in the corridor fixture
    // These are systems that appear frequently together in sample routes
    let known_routes = [
        ("Strym", "U:36OE"), // Direct connection from Strym
        ("Brana", "H:2L2S"), // Another common route
    ];

    let mut routes_found = 0;
    for (start, end) in known_routes {
        let start_id = match resolve_system_id(&starmap, start) {
            Some(id) => id,
            None => {
                println!("Skipping {}->{}: start system not in fixture", start, end);
                continue;
            }
        };

        let end_id = match resolve_system_id(&starmap, end) {
            Some(id) => id,
            None => {
                println!("Skipping {}->{}: end system not in fixture", start, end);
                continue;
            }
        };

        let path = find_route_a_star(&graph, Some(&starmap), start_id, end_id, &constraints);

        if let Some(p) = path {
            println!("Route {}->{}: {} hops", start, end, p.len());
            routes_found += 1;
        } else {
            println!("Route {}->{}: not found", start, end);
        }
    }

    assert!(routes_found > 0, "Should find at least one known route");
}
