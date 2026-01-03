use std::path::PathBuf;

use evefrontier_lib::{load_starmap, plan_route, RouteAlgorithm, RouteConstraints, RouteRequest};

#[test]
fn a_star_blocked_by_avoid_critical_state() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");

    // Load ship catalog and pick a ship known from fixtures
    let ship_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/ship_data.csv");
    let catalog = evefrontier_lib::ShipCatalog::from_path(&ship_path).expect("load ship csv");
    let ship = catalog.get("Reflex").expect("Reflex available").clone();

    let loadout = evefrontier_lib::ShipLoadout::new(&ship, ship.fuel_capacity, 0.0).unwrap();

    let request = RouteRequest {
        start: "Nod".to_string(),
        goal: "Brana".to_string(),
        algorithm: RouteAlgorithm::AStar,
        constraints: RouteConstraints {
            max_jump: Some(300.0),
            avoid_gates: true,
            avoid_critical_state: true,
            ship: Some(ship),
            loadout: Some(loadout),
            heat_config: Some(evefrontier_lib::ship::HeatConfig {
                calibration_constant: 1e-8, // aggressive -> makes jumps more heating
                dynamic_mass: false,
            }),
            ..RouteConstraints::default()
        },
        spatial_index: None,
    };

    let err =
        plan_route(&starmap, &request).expect_err("should be blocked by critical-state avoidance");
    assert!(format!("{err}").contains("no route found"));
}

#[test]
fn a_star_allows_when_not_avoiding_critical_state() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");

    let ship_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/ship_data.csv");
    let catalog = evefrontier_lib::ShipCatalog::from_path(&ship_path).expect("load ship csv");
    let ship = catalog.get("Reflex").expect("Reflex available").clone();

    let loadout = evefrontier_lib::ShipLoadout::new(&ship, ship.fuel_capacity, 0.0).unwrap();

    let request = RouteRequest {
        start: "Nod".to_string(),
        goal: "Brana".to_string(),
        algorithm: RouteAlgorithm::AStar,
        constraints: RouteConstraints {
            max_jump: Some(300.0),
            avoid_gates: true,
            avoid_critical_state: false,
            ship: Some(ship),
            loadout: Some(loadout),
            heat_config: Some(evefrontier_lib::ship::HeatConfig::default()),
            ..RouteConstraints::default()
        },
        spatial_index: None,
    };

    let plan = plan_route(&starmap, &request).expect("route planned");
    assert!(plan.steps.len() >= 2);
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/minimal_static_data.db")
}
