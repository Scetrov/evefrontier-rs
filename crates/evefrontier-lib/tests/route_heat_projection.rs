use evefrontier_lib::output::{RouteEndpoint, RouteOutputKind, RouteSummary};
mod common;
use common::RouteStepBuilder;
use evefrontier_lib::ship::HEAT_NOMINAL;
use evefrontier_lib::HeatConfig;
use evefrontier_lib::RouteAlgorithm;
// ShipLoadout used via fully-qualified path to avoid unused-import lint in some test builds

fn make_reflex_route_summary() -> RouteSummary {
    let steps = vec![
        RouteStepBuilder::new()
            .index(0)
            .id(30000001)
            .name("Nod")
            .distance(0.0)
            .build(),
        RouteStepBuilder::new()
            .index(1)
            .id(30000003)
            .name("D:2NAS")
            .distance(18.95)
            .build(),
        RouteStepBuilder::new()
            .index(2)
            .id(30000004)
            .name("G:3OA0")
            .distance(38.26)
            .build(),
        RouteStepBuilder::new()
            .index(3)
            .id(30000002)
            .name("Brana")
            .distance(23.09)
            .build(),
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
        parameters: None,
    }
}

#[test]
fn attach_heat_reflex_route() {
    // Load ship fixture
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/ship_data.csv");
    let catalog = evefrontier_lib::ShipCatalog::from_path(&path).expect("load ship csv");
    let ship = catalog.get("Reflex").expect("Reflex available");

    let mut summary = make_reflex_route_summary();
    let loadout = evefrontier_lib::ShipLoadout::new(ship, ship.fuel_capacity, 633_006.0)
        .expect("create loadout");
    // Use calibration=1.0 here so expected values computed below match the helper
    // `calculate_jump_heat(..., calibration=1.0)` used in assertions.
    let config = HeatConfig {
        calibration_constant: 1.0,
        dynamic_mass: false,
    };

    summary
        .attach_heat(ship, &loadout, &config)
        .expect("attach heat");

    // Verify per-hop numbers (computed via calculate_jump_heat to match ship fixture)
    let s1 = summary.steps[1].heat.as_ref().expect("step1 heat");
    let mass = ship.base_mass_kg
        + loadout.cargo_mass_kg
        + (loadout.fuel_load * evefrontier_lib::FUEL_MASS_PER_UNIT_KG);
    let expected1_energy =
        evefrontier_lib::calculate_jump_heat(mass, 18.95, ship.base_mass_kg, 1.0)
            .expect("calc expected1");
    let expected1 = expected1_energy / (mass * ship.specific_heat);
    assert!(
        (s1.hop_heat - expected1).abs() < 0.0001,
        "s1 {} expected {}",
        s1.hop_heat,
        expected1
    );

    let s2 = summary.steps[2].heat.as_ref().expect("step2 heat");
    let expected2_energy =
        evefrontier_lib::calculate_jump_heat(mass, 38.26, ship.base_mass_kg, 1.0)
            .expect("calc expected2");
    let expected2 = expected2_energy / (mass * ship.specific_heat);
    assert!(
        (s2.hop_heat - expected2).abs() < 0.0001,
        "s2 {} expected {}",
        s2.hop_heat,
        expected2
    );

    // In the non-cumulative model, we assume the ship starts each jump at a ready
    // state (nominal or ambient). The residual heat for the step reflects the
    // temperature *after* an optional cooldown period. If a wait is recommended,
    // residual heat should match the target (HEAT_NOMINAL).
    let residual2 = s2.residual_heat.expect("residual heat present");
    if s2.wait_time_seconds.is_some() {
        assert!(
            (residual2 - HEAT_NOMINAL).abs() < 1e-6,
            "residual {} expected {}",
            residual2,
            HEAT_NOMINAL
        );
    } else {
        assert!(
            residual2 < HEAT_NOMINAL,
            "residual {} should be below nominal",
            residual2
        );
    }

    let s3 = summary.steps[3].heat.as_ref().expect("step3 heat");
    let expected3_energy =
        evefrontier_lib::calculate_jump_heat(mass, 23.09, ship.base_mass_kg, 1.0)
            .expect("calc expected3");
    let expected3 = expected3_energy / (mass * ship.specific_heat);
    assert!(
        (s3.hop_heat - expected3).abs() < 0.0001,
        "s3 {} expected {}",
        s3.hop_heat,
        expected3
    );

    // Verify generated hop heat sums to expected total (we no longer expose a cumulative total)
    let expected_total = expected1 + expected2 + expected3;
    let sum_hop: f64 = summary
        .steps
        .iter()
        .skip(1)
        .filter_map(|s| s.heat.as_ref().map(|h| h.hop_heat))
        .sum();

    assert!(
        (sum_hop - expected_total).abs() < 0.0001,
        "sum_hop {} expected {}",
        sum_hop,
        expected_total
    );
}

#[test]
fn attach_heat_suppresses_cooldown_before_gate() {
    let ship = evefrontier_lib::ShipAttributes {
        name: "GateSuppressionShip".to_string(),
        base_mass_kg: 1e6,
        specific_heat: 1.0,
        fuel_capacity: 1000.0,
        cargo_capacity: 0.0,
    };
    let loadout = evefrontier_lib::ShipLoadout::new(&ship, 1000.0, 0.0).expect("loadout ok");
    let config = HeatConfig {
        calibration_constant: 1000.0,
        dynamic_mass: false,
    };

    let mut summary = RouteSummary {
        kind: RouteOutputKind::Route,
        algorithm: RouteAlgorithm::Dijkstra,
        hops: 2,
        gates: 1,
        jumps: 1,
        total_distance: 50.0,
        jump_distance: 40.0,
        start: RouteEndpoint {
            id: 1,
            name: Some("Start".to_string()),
        },
        goal: RouteEndpoint {
            id: 3,
            name: Some("Goal".to_string()),
        },
        parameters: None,
        steps: vec![
            RouteStepBuilder::new().index(0).id(1).name("Start").build(),
            RouteStepBuilder::new()
                .index(1)
                .id(2)
                .name("Sys2")
                .distance(40.0)
                .method("jump")
                .build(),
            RouteStepBuilder::new()
                .index(2)
                .id(3)
                .name("Goal")
                .distance(10.0)
                .method("gate")
                .build(),
        ],
        fuel: None,
        heat: None,
        fmap_url: None,
    };

    summary
        .attach_heat(&ship, &loadout, &config)
        .expect("attach heat");

    let s1 = summary.steps[1].heat.as_ref().expect("step 1 heat");
    assert!(s1.hop_heat > 0.0);
    assert!(
        s1.wait_time_seconds.is_none(),
        "expected no cooldown before a gate jump"
    );
}
