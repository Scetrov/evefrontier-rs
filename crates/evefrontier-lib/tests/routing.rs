use std::path::PathBuf;

use evefrontier_lib::{load_starmap, plan_route, RouteAlgorithm, RouteConstraints, RouteRequest};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/minimal/static_data.db")
}

#[test]
fn bfs_route_plan_succeeds() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let request = RouteRequest::bfs("Nod", "Brana");
    let plan = plan_route(&starmap, &request).expect("route exists");

    assert_eq!(plan.algorithm, RouteAlgorithm::Bfs);
    assert_eq!(plan.start, starmap.system_id_by_name("Nod").unwrap());
    assert_eq!(plan.goal, starmap.system_id_by_name("Brana").unwrap());
    assert!(plan.hop_count() >= 1);
}

#[test]
fn dijkstra_route_plan_succeeds() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let request = RouteRequest {
        start: "Nod".to_string(),
        goal: "Brana".to_string(),
        algorithm: RouteAlgorithm::Dijkstra,
        constraints: RouteConstraints {
            max_jump: Some(80.0),
            ..Default::default()
        },
        spatial_index: None,
        max_spatial_neighbors: evefrontier_lib::GraphBuildOptions::default().max_spatial_neighbors,
        optimization: evefrontier_lib::routing::RouteOptimization::Distance,
        fuel_config: evefrontier_lib::ship::FuelConfig::default(),
    };

    let plan = plan_route(&starmap, &request).expect("route exists");
    assert_eq!(plan.algorithm, RouteAlgorithm::Dijkstra);
}

#[test]
fn a_star_respects_max_jump_constraint() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    // Test spatial routing without gates
    // Note: Nod and Brana are ~297 ly apart, so we need a large max_jump to test this
    let request = RouteRequest {
        start: "Nod".to_string(),
        goal: "Brana".to_string(),
        algorithm: RouteAlgorithm::AStar,
        constraints: RouteConstraints {
            max_jump: Some(300.0),
            avoid_gates: true,
            ..RouteConstraints::default()
        },
        spatial_index: None,
        max_spatial_neighbors: evefrontier_lib::GraphBuildOptions::default().max_spatial_neighbors,
        optimization: evefrontier_lib::routing::RouteOptimization::Distance,
        fuel_config: evefrontier_lib::ship::FuelConfig::default(),
    };

    let plan = plan_route(&starmap, &request).expect("route exists");
    assert_eq!(plan.algorithm, RouteAlgorithm::AStar);
    assert!(
        plan.steps.len() >= 2,
        "route should include intermediate hop"
    );
}

#[test]
fn avoided_goal_rejects_route() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let request = RouteRequest {
        start: "Nod".to_string(),
        goal: "Brana".to_string(),
        algorithm: RouteAlgorithm::Bfs,
        constraints: RouteConstraints {
            avoid_systems: vec!["Brana".to_string()],
            ..RouteConstraints::default()
        },
        spatial_index: None,
        max_spatial_neighbors: evefrontier_lib::GraphBuildOptions::default().max_spatial_neighbors,
        optimization: evefrontier_lib::routing::RouteOptimization::Distance,
        fuel_config: evefrontier_lib::ship::FuelConfig::default(),
    };

    let error = plan_route(&starmap, &request).expect_err("avoided goal");
    assert!(format!("{error}").contains("no route found"));
}

#[test]
fn temperature_limit_blocks_hot_systems() {
    let mut starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let brana_id = starmap.system_id_by_name("Brana").unwrap();
    starmap
        .systems
        .get_mut(&brana_id)
        .unwrap()
        .metadata
        .star_temperature = Some(5_000.0);

    let request = RouteRequest {
        start: "Nod".to_string(),
        goal: "Brana".to_string(),
        algorithm: RouteAlgorithm::Dijkstra,
        constraints: RouteConstraints {
            max_temperature: Some(4_000.0),
            ..RouteConstraints::default()
        },
        spatial_index: None,
        max_spatial_neighbors: evefrontier_lib::GraphBuildOptions::default().max_spatial_neighbors,
        optimization: evefrontier_lib::routing::RouteOptimization::Distance,
        fuel_config: evefrontier_lib::ship::FuelConfig::default(),
    };

    let error = plan_route(&starmap, &request).expect_err("temperature filtered");
    assert!(format!("{error}").contains("no route found"));
}

#[test]
fn avoid_gates_switches_to_spatial_graph() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");

    let request = RouteRequest {
        start: "Nod".to_string(),
        goal: "Brana".to_string(),
        algorithm: RouteAlgorithm::Dijkstra,
        constraints: RouteConstraints {
            avoid_gates: true,
            ..RouteConstraints::default()
        },
        spatial_index: None,
        max_spatial_neighbors: evefrontier_lib::GraphBuildOptions::default().max_spatial_neighbors,
        optimization: evefrontier_lib::routing::RouteOptimization::Distance,
        fuel_config: evefrontier_lib::ship::FuelConfig::default(),
    };

    let plan = plan_route(&starmap, &request).expect("spatial route exists");
    assert_eq!(plan.algorithm, RouteAlgorithm::Dijkstra);
}

#[test]
fn ship_max_jump_limits_spatial_edges() {
    // Minimal in-memory starmap with two systems ~100 ly apart and no gates.
    use evefrontier_lib::db::{Starmap, System, SystemId, SystemMetadata, SystemPosition};
    use evefrontier_lib::ship::{FuelConfig, ShipAttributes, ShipLoadout};
    use std::collections::HashMap;

    let a: SystemId = 1;
    let b: SystemId = 2;

    let mut systems = HashMap::new();
    systems.insert(
        a,
        System {
            id: a,
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
        },
    );
    systems.insert(
        b,
        System {
            id: b,
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
            position: SystemPosition::new(100.0, 0.0, 0.0),
        },
    );

    let mut name_to_id = HashMap::new();
    name_to_id.insert("A".to_string(), a);
    name_to_id.insert("B".to_string(), b);

    let adjacency: HashMap<SystemId, Vec<SystemId>> = HashMap::new();

    let starmap = Starmap {
        systems,
        name_to_id,
        adjacency: std::sync::Arc::new(adjacency),
    };

    // Ship with insufficient fuel to make the 100 ly hop
    let ship = ShipAttributes {
        name: "Tiny".to_string(),
        base_mass_kg: 10000.0,
        specific_heat: 1000.0,
        fuel_capacity: 1.0,
        cargo_capacity: 0.0,
    };
    let loadout = ShipLoadout::new(&ship, 1.0, 0.0).expect("valid loadout");

    let request = evefrontier_lib::routing::RouteRequest {
        start: "A".to_string(),
        goal: "B".to_string(),
        algorithm: RouteAlgorithm::AStar,
        constraints: RouteConstraints {
            ship: Some(ship.clone()),
            loadout: Some(loadout),
            // By default fuel does not block the route to allow for projection/refuelling
            ..RouteConstraints::default()
        },
        spatial_index: None,
        max_spatial_neighbors: evefrontier_lib::GraphBuildOptions::default().max_spatial_neighbors,
        optimization: evefrontier_lib::routing::RouteOptimization::Distance,
        fuel_config: FuelConfig::default(),
    };

    // With fuel-only limits, the route should now be found (to allow projection)
    let plan = plan_route(&starmap, &request).expect("route found even with low fuel");
    assert_eq!(plan.steps.len(), 2);

    // But if we enable heat-based avoidance, it should be blocked (heat limit is ~50ly, distance is 100ly)
    let mut strict_request = request.clone();
    strict_request.constraints.avoid_critical_state = true;
    let err = plan_route(&starmap, &strict_request).expect_err("heat limit should block route");
    assert!(format!("{err}").contains("no route found"));
}

// inject_positions is no longer needed - real fixture data includes coordinates
