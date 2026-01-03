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
            max_jump: Some(300.0),
            avoid_gates: true,
            avoid_critical_state: true,
            ship: Some(ship.clone()),
            loadout: Some(loadout),
            heat_config: Some(heat_config_aggressive),
            ..RouteConstraints::default()
        },
        spatial_index: None,
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
            max_jump: Some(300.0),
            avoid_gates: true,
            avoid_critical_state: true,
            ship: Some(ship.clone()),
            loadout: Some(loadout),
            heat_config: Some(heat_config_dynamic),
            ..RouteConstraints::default()
        },
        spatial_index: None,
    };

    let err_static = plan_route(&starmap, &req_static).err();
    let err_dynamic = plan_route(&starmap, &req_dynamic).err();

    // Current behavior: dynamic_mass does not enable per-hop relaxation in the avoidance check,
    // so both requests should produce the same result (both blocked in this aggressive calibration).
    assert!(
        err_static.is_some(),
        "expected static request to be blocked"
    );
    assert!(
        err_dynamic.is_some(),
        "expected dynamic request to be blocked"
    );
}
