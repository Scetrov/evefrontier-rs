use evefrontier_lib::ship::calculate_jump_heat;
use evefrontier_lib::ship::ShipCatalog;

fn fixture_catalog() -> ShipCatalog {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/ship_data.csv");
    ShipCatalog::from_path(&path).expect("load fixture ship_data.csv")
}

#[test]
fn reflex_full_fuel_no_cargo_static() {
    let catalog = fixture_catalog();
    let _ship = catalog.get("Reflex").expect("Reflex exists");
    let total_mass = 12_383_006.0;
    let hull = 10_000_000.0;
    let heat = calculate_jump_heat(total_mass, 18.95, hull, 1.0).expect("calc heat");
    assert!(
        (heat - 70.39738911).abs() < 0.01,
        "expected approx 70.39738911, got {}",
        heat
    );

    let heat2 = calculate_jump_heat(total_mass, 38.26, hull, 1.0).expect("calc heat");
    assert!(
        (heat2 - 142.132142868).abs() < 0.01,
        "expected approx 142.132142868, got {}",
        heat2
    );

    let heat3 = calculate_jump_heat(total_mass, 23.09, hull, 1.0).expect("calc heat");
    assert!(
        (heat3 - 85.777082562).abs() < 0.01,
        "expected approx 85.777082562, got {}",
        heat3
    );
}

#[test]
fn gate_transition_zero_heat() {
    let heat = calculate_jump_heat(12_383_006.0, 0.0, 10_000_000.0, 1.0).expect("calc heat");
    assert_eq!(heat, 0.0);
}

#[test]
fn dynamic_mass_reduction_effect() {
    // After consuming some fuel mass
    let reduced_mass = 12_359_536.0;
    let heat = calculate_jump_heat(reduced_mass, 38.26, 10_000_000.0, 1.0).expect("calc heat");
    assert!(
        (heat - 141.86275420799998).abs() < 0.02,
        "expected approx 141.862754208, got {}",
        heat
    );
}

#[test]
fn empty_cargo_minimum_mass() {
    let total = 10_001_750.0;
    let heat = calculate_jump_heat(total, 18.95, 10_000_000.0, 1.0).expect("calc heat");
    assert!(
        (heat - 56.85994875).abs() < 0.01,
        "expected approx 56.85994875, got {}",
        heat
    );
}
