use std::path::PathBuf;

use evefrontier_lib::ship::{ShipAttributes, ShipCatalog};

pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures")
}

pub fn reflex_ship() -> ShipAttributes {
    let path = fixtures_dir().join("ship_data.csv");
    let catalog = ShipCatalog::from_path(&path).expect("load fixture ship_data.csv");
    catalog
        .get("Reflex")
        .expect("Reflex ship present in fixtures")
        .clone()
}
