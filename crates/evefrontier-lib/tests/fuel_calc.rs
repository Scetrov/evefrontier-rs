use std::path::PathBuf;

use evefrontier_lib::ship::{calculate_route_fuel, FuelConfig, ShipCatalog, ShipLoadout};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/ship_data.csv")
}

fn distances() -> Vec<f64> {
    vec![18.95, 38.26, 23.09]
}

#[test]
fn calculates_static_fuel_projection() {
    let catalog = ShipCatalog::from_path(&fixture_path()).expect("fixture should load");
    let ship = catalog.get("Reflex").expect("reflex present");

    let loadout = ShipLoadout::new(ship, 1750.0, 633_006.0).expect("valid loadout");
    let config = FuelConfig {
        quality: 10.0,
        dynamic_mass: false,
    };

    let projections = calculate_route_fuel(ship, &loadout, &distances(), &config)
        .expect("fuel calculation succeeds");
    assert_eq!(projections.len(), 3);

    let first = &projections[0];
    assert!((first.hop_cost - 201.5286262).abs() < 1e-6);
    assert!((first.cumulative - 201.5286262).abs() < 1e-6);
    assert!((first.remaining.unwrap() - 1548.4713738).abs() < 1e-6);

    let last = projections.last().unwrap();
    assert!((last.cumulative - 853.9709068).abs() < 1e-6);
    assert!((last.remaining.unwrap() - 896.0290932).abs() < 1e-6);
}

#[test]
fn calculates_dynamic_mass_and_reduces_total_cost() {
    let catalog = ShipCatalog::from_path(&fixture_path()).expect("fixture should load");
    let ship = catalog.get("Reflex").expect("reflex present");

    let loadout = ShipLoadout::new(ship, 1750.0, 633_006.0).expect("valid loadout");
    let static_config = FuelConfig {
        quality: 10.0,
        dynamic_mass: false,
    };
    let dynamic_config = FuelConfig {
        quality: 10.0,
        dynamic_mass: true,
    };

    let static_proj = calculate_route_fuel(ship, &loadout, &distances(), &static_config)
        .expect("static calc succeeds");
    let dynamic_proj = calculate_route_fuel(ship, &loadout, &distances(), &dynamic_config)
        .expect("dynamic calc succeeds");

    let static_total = static_proj.last().unwrap().cumulative;
    let dynamic_total = dynamic_proj.last().unwrap().cumulative;

    assert!(dynamic_total < static_total);

    let last_dynamic = dynamic_proj.last().unwrap();
    assert!((last_dynamic.remaining.unwrap() - 896.0508517954859).abs() < 1e-6);
}

#[test]
fn calculate_jump_cost_direct() {
    use evefrontier_lib::ship::calculate_jump_fuel_cost;

    // Known case: mass 12_383_006, distance 18.95, quality 10 (10%)
    // mass_factor = mass / 100_000 = 123.83006
    // quality_factor = 0.10
    // hop_cost = mass_factor * quality_factor * distance = 123.83006 * 0.1 * 18.95
    let mass = 12_383_006.0;
    let distance = 18.95;
    let cfg = FuelConfig {
        quality: 10.0,
        dynamic_mass: false,
    };

    let hop = calculate_jump_fuel_cost(mass, distance, &cfg).expect("should compute hop cost");
    let expected = (mass / 100_000.0) * (cfg.quality / 100.0) * distance;
    assert!(
        (hop - expected).abs() < 1e-12,
        "hop {} expected {}",
        hop,
        expected
    );
}

#[test]
fn calculate_maximum_distance_roundtrip() {
    use evefrontier_lib::ship::calculate_maximum_distance;

    // Given fuel_units=1750, ship_mass=9_750_000, quality=10 (10%)
    // expected max_distance = (1750 * 0.1 * 100000) / 9_750_000
    let maxd = calculate_maximum_distance(1750.0, 9_750_000.0, 10.0).expect("max distance");
    let expected = (1750.0 * 0.1 * 100_000.0) / 9_750_000.0;
    assert!((maxd - expected).abs() < 1e-12);
}

#[test]
fn calculate_maximum_distance_invalid_inputs() {
    use evefrontier_lib::ship::calculate_maximum_distance;

    assert!(calculate_maximum_distance(-1.0, 1_000_000.0, 10.0).is_err());
    assert!(calculate_maximum_distance(100.0, 0.0, 10.0).is_err());
    assert!(calculate_maximum_distance(100.0, 1_000_000.0, f64::NAN).is_err());
}

#[test]
fn calculate_jump_cost_invalid_quality() {
    use evefrontier_lib::ship::calculate_jump_fuel_cost;

    let res = calculate_jump_fuel_cost(
        1_000_000.0,
        10.0,
        &FuelConfig {
            quality: 0.0,
            dynamic_mass: false,
        },
    );
    assert!(res.is_err(), "quality 0 should be rejected by validation");
}
