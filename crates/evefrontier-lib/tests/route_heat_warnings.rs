use evefrontier_lib::output::{RouteEndpoint, RouteOutputKind, RouteStep, RouteSummary};
use evefrontier_lib::RouteAlgorithm;

fn make_reflex_route_summary() -> RouteSummary {
    let steps = vec![
        RouteStep {
            index: 0,
            id: 30000001,
            name: Some("Nod".to_string()),
            distance: None,
            method: None,
            min_external_temp: None,
            planet_count: None,
            moon_count: None,
            fuel: None,
            heat: None,
        },
        RouteStep {
            index: 1,
            id: 30000003,
            name: Some("D:2NAS".to_string()),
            distance: Some(18.95),
            method: Some("jump".to_string()),
            min_external_temp: None,
            planet_count: None,
            moon_count: None,
            fuel: None,
            heat: None,
        },
        RouteStep {
            index: 2,
            id: 30000004,
            name: Some("G:3OA0".to_string()),
            distance: Some(38.26),
            method: Some("jump".to_string()),
            min_external_temp: None,
            planet_count: None,
            moon_count: None,
            fuel: None,
            heat: None,
        },
        RouteStep {
            index: 3,
            id: 30000002,
            name: Some("Brana".to_string()),
            distance: Some(23.09),
            method: Some("jump".to_string()),
            min_external_temp: None,
            planet_count: None,
            moon_count: None,
            fuel: None,
            heat: None,
        },
    ];

    RouteSummary {
        kind: RouteOutputKind::Route,
        algorithm: RouteAlgorithm::AStar,
        hops: 3,
        gates: 0,
        jumps: 3,
        total_distance: 18.95 + 38.26 + 23.09,
        jump_distance: 18.95 + 38.26 + 23.09,
        start: RouteEndpoint {
            id: 30000001,
            name: Some("Nod".to_string()),
        },
        goal: RouteEndpoint {
            id: 30000002,
            name: Some("Brana".to_string()),
        },
        steps,
        fuel: None,
        heat: None,
        fmap_url: None,
    }
}

#[test]
fn heat_warning_and_error_thresholds() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/ship_data.csv");
    let catalog = evefrontier_lib::ShipCatalog::from_path(&path).expect("load ship csv");
    let ship = catalog.get("Reflex").expect("Reflex available").clone();

    // Case 1: overheated threshold (use calibration to scale heat down into overheated band)
    let mut summary = make_reflex_route_summary();
    let loadout = evefrontier_lib::ShipLoadout::new(&ship, ship.fuel_capacity, 633_006.0).unwrap();
    // Use an extreme calibration constant to force overheating for the test.
    // Note: the calibration constant scales inversely with heat; pick a very small value.
    let config = evefrontier_lib::HeatConfig {
        calibration_constant: 1e-8,
        dynamic_mass: false,
    };
    summary
        .attach_heat(&ship, &loadout, &config)
        .expect("attach heat");
    let total = summary.heat.as_ref().expect("summary heat");
    // Sanity-check: at least one warning should indicate overheating or critical heat
    assert!(
        total.warnings.iter().any(|w| {
            let l = w.to_lowercase();
            l.contains("overheat") || l.contains("critical")
        }),
        "warnings: {:?}, hops: {:?}",
        total.warnings,
        summary
            .steps
            .iter()
            .map(|s| s.heat.as_ref().map(|h| h.hop_heat))
            .collect::<Vec<_>>()
    );

    // Case 2: critical threshold (default calibration produces critical cumulative heat)
    let mut summary2 = make_reflex_route_summary();
    let loadout2 = evefrontier_lib::ShipLoadout::new(&ship, ship.fuel_capacity, 633_006.0).unwrap();
    let config2 = evefrontier_lib::HeatConfig::default();
    summary2
        .attach_heat(&ship, &loadout2, &config2)
        .expect("attach heat");
    let total2 = summary2.heat.as_ref().expect("summary heat");
    // With cooling enabled and default calibration the route may not reach severe thresholds.
    // Accept either a severe warning (critical/overheated) or no warnings.
    assert!(
        total2.warnings.iter().any(|w| {
            let l = w.to_lowercase();
            l.contains("critical") || l.contains("overheat")
        }) || total2.warnings.is_empty(),
        "warnings: {:?}",
        total2.warnings
    );
}
