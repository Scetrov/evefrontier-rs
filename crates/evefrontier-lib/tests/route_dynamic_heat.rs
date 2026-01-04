use std::path::PathBuf;

use evefrontier_lib::{load_starmap, plan_route, RouteAlgorithm, RouteConstraints, RouteRequest};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/minimal/static_data.db")
}

#[test]
fn dynamic_mass_does_not_change_per_hop_avoidance_behavior() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");

    // Load ship fixture
    let ship_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/ship_data.csv");
    let catalog = evefrontier_lib::ShipCatalog::from_path(&ship_path).expect("load ship csv");
    let ship = catalog.get("Reflex").expect("Reflex available").clone();

    let loadout = evefrontier_lib::ShipLoadout::new(&ship, ship.fuel_capacity, 0.0).unwrap();

    // Aggressive calibration to force hops to be considered dangerous
    let heat_config_aggressive = evefrontier_lib::ship::HeatConfig {
        calibration_constant: 1e-8,
        dynamic_mass: false,
    };

    let req_static = RouteRequest {
        start: "Nod".to_string(),
        goal: "Brana".to_string(),
        algorithm: RouteAlgorithm::AStar,
        constraints: RouteConstraints {
            max_jump: Some(40.0),
            avoid_critical_state: true,
            ship: Some(ship.clone()),
            loadout: Some(loadout),
            heat_config: Some(heat_config_aggressive),
            ..Default::default()
        },
        spatial_index: None,
        max_spatial_neighbors: evefrontier_lib::GraphBuildOptions::default().max_spatial_neighbors,
        optimization: evefrontier_lib::routing::RouteOptimization::Distance,
        fuel_config: evefrontier_lib::ship::FuelConfig::default(),
    };

    // Same but enabling dynamic_mass in heat config (note: heat checks are conservative)
    let heat_config_dynamic = evefrontier_lib::ship::HeatConfig {
        calibration_constant: 1e-8,
        dynamic_mass: true,
    };

    let req_dynamic = RouteRequest {
        start: "Nod".to_string(),
        goal: "Brana".to_string(),
        algorithm: RouteAlgorithm::AStar,
        constraints: RouteConstraints {
            max_jump: Some(40.0),
            avoid_critical_state: true,
            ship: Some(ship.clone()),
            loadout: Some(loadout),
            heat_config: Some(heat_config_dynamic),
            ..Default::default()
        },
        spatial_index: None,
        max_spatial_neighbors: evefrontier_lib::GraphBuildOptions::default().max_spatial_neighbors,
        optimization: evefrontier_lib::routing::RouteOptimization::Distance,
        fuel_config: evefrontier_lib::ship::FuelConfig::default(),
    };

    let err_static = plan_route(&starmap, &req_static).err();
    let err_dynamic = plan_route(&starmap, &req_dynamic).err();

    // Current behavior: enabling `dynamic_mass` should not change whether an avoidance
    // policy blocks a route. The important invariant is that both requests yield the
    // same outcome (both allowed or both blocked). If we later choose to change this
    // policy, update this test accordingly and add documentation in `HEAT_MECHANICS.md`.
    assert_eq!(
        err_static.is_some(),
        err_dynamic.is_some(),
        "dynamic_mass should not change blocking outcome"
    );
}
