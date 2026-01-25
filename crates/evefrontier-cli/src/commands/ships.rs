//! Ships command handler for listing available ships.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use evefrontier_lib::{DatasetPaths, ShipCatalog};

/// Handle the ships subcommand.
///
/// Lists available ships from the ship_data.csv catalog.
pub fn handle_list_ships(paths: &DatasetPaths) -> Result<()> {
    let catalog = load_ship_catalog(paths)?;
    print_ship_catalog(&catalog);
    Ok(())
}

/// Load the ship catalog from the dataset paths.
///
/// Searches for ship_data.csv in the following order:
/// 1. Path from DatasetPaths.ship_data (populated by dataset resolver)
/// 2. EVEFRONTIER_SHIP_DATA environment variable
/// 3. Adjacent to the database file
/// 4. Debug fixture path (only in debug builds)
pub fn load_ship_catalog(paths: &DatasetPaths) -> Result<ShipCatalog> {
    // Prefer ship data discovered by the dataset resolver (populated in `DatasetPaths`)
    if let Some(ref ship_path) = paths.ship_data {
        if ship_path.exists() {
            return ShipCatalog::from_path(ship_path)
                .with_context(|| format!("failed to load ship data from {}", ship_path.display()));
        }
    }

    let candidates = ship_data_candidates(&paths.database);
    let path = candidates
        .iter()
        .find(|p| p.exists())
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "ship_data.csv not found; set EVEFRONTIER_SHIP_DATA or place file next to dataset"
            )
        })?;

    ShipCatalog::from_path(&path)
        .with_context(|| format!("failed to load ship data from {}", path.display()))
}

/// Get candidate paths for ship_data.csv.
fn ship_data_candidates(database: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(env_path) = std::env::var("EVEFRONTIER_SHIP_DATA") {
        candidates.push(PathBuf::from(env_path));
    }

    if let Some(parent) = database.parent() {
        candidates.push(parent.join("ship_data.csv"));
    }

    if cfg!(debug_assertions) {
        let fixture =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/ship_data.csv");
        candidates.push(fixture);
    }

    candidates
}

/// Print the ship catalog to stdout in a formatted table.
fn print_ship_catalog(catalog: &ShipCatalog) {
    let ships = catalog.ships_sorted();
    if ships.is_empty() {
        println!("No ships available in catalog.");
        return;
    }

    println!("Available ships ({}):", ships.len());
    println!(
        "{:<16} {:>14} {:>10} {:>12}",
        "Name", "Base Mass (kg)", "Fuel Cap", "Cargo Cap"
    );
    for ship in ships {
        println!(
            "{:<16} {:>14.0} {:>10.0} {:>12.0}",
            ship.name, ship.base_mass_kg, ship.fuel_capacity, ship.cargo_capacity
        );
    }
}
