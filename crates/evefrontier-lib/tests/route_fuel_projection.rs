use std::path::PathBuf;

use evefrontier_lib::routing::{plan_route, RouteAlgorithm, RouteConstraints, RouteRequest};
use evefrontier_lib::ship::{FuelConfig, ShipCatalog, ShipLoadout};
use evefrontier_lib::{load_starmap, RouteOutputKind, RouteSummary};

fn fixture_db_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/fixtures/minimal/static_data.db")
        .canonicalize()
        .expect("fixture dataset present")
}

fn fixture_ship_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/fixtures/ship_data.csv")
        .canonicalize()
        .expect("ship fixture present")
}

#[test]
fn attaches_fuel_projection_to_route_summary() {
    let starmap = load_starmap(&fixture_db_path()).expect("starmap loads");
    let request = RouteRequest {
        start: "Nod".to_string(),
        goal: "Brana".to_string(),
        algorithm: RouteAlgorithm::AStar,
        constraints: RouteConstraints::default(),
        spatial_index: None,
        max_spatial_neighbors: evefrontier_lib::GraphBuildOptions::default().max_spatial_neighbors,
        optimization: evefrontier_lib::routing::RouteOptimization::Distance,
        fuel_config: evefrontier_lib::ship::FuelConfig::default(),
    };
    let plan = plan_route(&starmap, &request).expect("route planned");
    let mut summary =
        RouteSummary::from_plan(RouteOutputKind::Route, &starmap, &plan).expect("summary builds");

    let catalog = ShipCatalog::from_path(&fixture_ship_path()).expect("ship fixture loads");
    let ship = catalog.get("Reflex").expect("reflex present");
    let loadout = ShipLoadout::new(ship, 1750.0, 0.0).expect("valid loadout");
    let fuel_config = FuelConfig {
        quality: 10.0,
        dynamic_mass: true,
    };

    summary
        .attach_fuel(ship, &loadout, &fuel_config)
        .expect("fuel projection attaches");

    let fuel = summary.fuel.as_ref().expect("fuel summary present");
    assert_eq!(fuel.ship_name.as_deref(), Some("Reflex"));
    assert!(fuel.total >= 0.0);

    // All hops after the origin should carry fuel projections.
    for step in summary.steps.iter().skip(1) {
        let projection = step.fuel.as_ref().expect("projection present on hop");
        assert!(projection.hop_cost >= 0.0);
        assert!(projection.cumulative >= 0.0);
    }

    // Cumulative fuel should be non-decreasing across steps.
    let mut prev = 0.0;
    for step in summary.steps.iter().skip(1) {
        let cumulative = step.fuel.as_ref().unwrap().cumulative;
        assert!(cumulative + 1e-9 >= prev);
        prev = cumulative;
    }
}

#[test]
fn gate_steps_do_not_consume_fuel() {
    let starmap = load_starmap(&fixture_db_path()).expect("starmap loads");
    let request = RouteRequest {
        start: "Nod".to_string(),
        goal: "Brana".to_string(),
        algorithm: RouteAlgorithm::AStar,
        constraints: RouteConstraints::default(),
        spatial_index: None,
        max_spatial_neighbors: evefrontier_lib::GraphBuildOptions::default().max_spatial_neighbors,
        optimization: evefrontier_lib::routing::RouteOptimization::Distance,
        fuel_config: evefrontier_lib::ship::FuelConfig::default(),
    };
    let plan = plan_route(&starmap, &request).expect("route planned");
    let mut summary =
        RouteSummary::from_plan(RouteOutputKind::Route, &starmap, &plan).expect("summary builds");

    let catalog = ShipCatalog::from_path(&fixture_ship_path()).expect("ship fixture loads");
    let ship = catalog.get("Reflex").expect("reflex present");
    let loadout = ShipLoadout::new(ship, 1750.0, 0.0).expect("valid loadout");
    let fuel_config = FuelConfig {
        quality: 10.0,
        dynamic_mass: true,
    };

    summary
        .attach_fuel(ship, &loadout, &fuel_config)
        .expect("fuel projection attaches");

    let mut prev_cumulative = 0.0;
    let mut prev_remaining = loadout.fuel_load;

    for step in summary.steps.iter().skip(1) {
        let projection = step.fuel.as_ref().expect("projection present on hop");
        match step.method.as_deref() {
            Some("gate") => {
                assert_eq!(projection.hop_cost, 0.0);
                assert!((projection.cumulative - prev_cumulative).abs() < 1e-9);
                if let Some(remaining) = projection.remaining {
                    assert!((remaining - prev_remaining).abs() < 1e-6);
                }
            }
            Some("jump") => {
                assert!(projection.hop_cost > 0.0);
                if let Some(remaining) = projection.remaining {
                    assert!(remaining <= prev_remaining + 1e-6);
                    prev_remaining = remaining;
                }
            }
            _ => {}
        }

        prev_cumulative = projection.cumulative;
    }
}
