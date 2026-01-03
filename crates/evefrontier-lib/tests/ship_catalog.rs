use std::path::PathBuf;

use evefrontier_lib::error::Error;
use evefrontier_lib::ship::ShipCatalog;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/ship_data.csv")
}

#[test]
fn loads_fixture_catalog_and_lists_ships() {
    let catalog = ShipCatalog::from_path(&fixture_path()).expect("fixture should load");

    let mut names = catalog.ship_names();
    names.sort();
    assert_eq!(names, vec!["Forager", "Reflex", "Warden"]);

    let reflex = catalog.get("reflex").expect("reflex ship present");
    assert!(reflex.base_mass_kg > 0.0);
    assert!(reflex.fuel_capacity > 0.0);
}

#[test]
fn rejects_duplicate_names_case_insensitive() {
    let csv = "name,base_mass_kg,specific_heat,fuel_capacity,cargo_capacity\n".to_string()
        + "Reflex,1,1,1,1\n"
        + "reflex,2,2,2,2\n";

    let err = ShipCatalog::from_reader(csv.as_bytes()).expect_err("should reject duplicates");
    match err {
        Error::DuplicateShipName { name } => assert_eq!(name, "reflex"),
        other => panic!("unexpected error: {:?}", other),
    }
}

#[test]
fn rejects_invalid_numeric_values() {
    let csv = "name,base_mass_kg,specific_heat,fuel_capacity,cargo_capacity\n".to_string()
        + "Reflex,-1,1,1,1\n";

    let err = ShipCatalog::from_reader(csv.as_bytes()).expect_err("should reject invalid values");
    match err {
        Error::ShipDataValidation { message } => {
            assert!(message.contains("base_mass_kg"))
        }
        other => panic!("unexpected error: {:?}", other),
    }
}
