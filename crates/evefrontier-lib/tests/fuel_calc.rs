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

    let projections = calculate_route_fuel(ship, &loadout, &distances(), &config);
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

    let static_proj = calculate_route_fuel(ship, &loadout, &distances(), &static_config);
    let dynamic_proj = calculate_route_fuel(ship, &loadout, &distances(), &dynamic_config);

    let static_total = static_proj.last().unwrap().cumulative;
    let dynamic_total = dynamic_proj.last().unwrap().cumulative;

    assert!(dynamic_total < static_total);

    let last_dynamic = dynamic_proj.last().unwrap();
    assert!((last_dynamic.remaining.unwrap() - 896.0508517954859).abs() < 1e-6);
}
