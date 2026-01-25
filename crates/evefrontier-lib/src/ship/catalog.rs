//! Ship catalog loading and management.
//!
//! This module handles loading ship data from CSV files and provides
//! catalog lookup functionality.

use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use csv::{ReaderBuilder, Trim};

use crate::error::{Error, Result};

use super::attributes::ShipAttributes;

/// Collection of ship definitions loaded from a CSV file.
#[derive(Debug, Clone, Default)]
pub struct ShipCatalog {
    ships: HashMap<String, ShipAttributes>,
    source: Option<PathBuf>,
}

impl ShipCatalog {
    /// Load a ship catalog from a file path.
    ///
    /// If the provided path appears to be a checksum sidecar (e.g. `.../e6c4-ship_data.csv.sha256`),
    /// attempt to locate the corresponding CSV next to it and use that file instead.
    pub fn from_path(path: &Path) -> Result<Self> {
        // Handle checksum sidecar files
        if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.eq_ignore_ascii_case("sha256"))
            .unwrap_or(false)
        {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if let Some(stripped) = file_name.strip_suffix(".sha256") {
                    let candidate = path.with_file_name(stripped);
                    if candidate.exists() {
                        let file = fs::File::open(&candidate)?;
                        let mut catalog = Self::from_reader(file)?;
                        catalog.source = Some(candidate);
                        return Ok(catalog);
                    }
                }
            }
        }

        let file = fs::File::open(path)?;
        let mut catalog = Self::from_reader(file)?;
        catalog.source = Some(path.to_path_buf());
        Ok(catalog)
    }

    /// Load a ship catalog from a reader (e.g., file or in-memory buffer).
    pub fn from_reader<R: Read>(reader: R) -> Result<Self> {
        let mut csv_reader = ReaderBuilder::new().trim(Trim::Fields).from_reader(reader);

        let headers = csv_reader
            .headers()
            .map_err(|err| Error::ShipDataValidation {
                message: format!("failed to read ship_data.csv headers: {err}"),
            })?
            .clone();

        // Helper to normalize header strings for robust matching.
        let normalize = |s: &str| {
            s.to_ascii_lowercase()
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect::<String>()
        };

        let normalized_headers: Vec<String> = headers.iter().map(&normalize).collect();

        // Mapping of canonical field name -> possible header synonyms (normalized)
        let synonyms: &[(&str, &[&str])] = &[
            ("name", &["name", "shipname", "ship_name", "ship"]),
            (
                "base_mass_kg",
                &["base_mass_kg", "mass_kg", "mass", "masskg", "masskg_kg"],
            ),
            (
                "specific_heat",
                &["specific_heat", "specificheat_c", "specificheat"],
            ),
            (
                "fuel_capacity",
                &[
                    "fuel_capacity",
                    "fuel_capacity_units",
                    "fuelcapacity_units",
                    "fuelcapacity",
                ],
            ),
            (
                "cargo_capacity",
                &["cargo_capacity", "capacity_m3", "capacity"],
            ),
        ];

        // Build index map for each canonical field
        use std::collections::BTreeMap;
        let mut index_map: BTreeMap<&str, usize> = BTreeMap::new();

        for (canon, alts) in synonyms {
            'outer: for alt in *alts {
                let alt_n = normalize(alt);
                for (i, h) in normalized_headers.iter().enumerate() {
                    if h == &alt_n {
                        index_map.insert(*canon, i);
                        break 'outer;
                    }
                }
            }
        }

        // Check required fields presence
        let required: Vec<&str> = vec![
            "name",
            "base_mass_kg",
            "specific_heat",
            "fuel_capacity",
            "cargo_capacity",
        ];
        let missing: Vec<&str> = required
            .into_iter()
            .filter(|c| !index_map.contains_key(c))
            .collect();

        if !missing.is_empty() {
            return Err(Error::ShipDataValidation {
                message: format!(
                    "ship_data.csv missing required columns: {}. Available: {}",
                    missing.join(", "),
                    headers
                        .iter()
                        .map(|h| h.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            });
        }

        let mut ships = HashMap::new();

        let mut row_num: usize = 1; // header is typically line 1
        for result in csv_reader.records() {
            row_num += 1;
            let record = result.map_err(|e| Error::ShipDataValidation {
                message: e.to_string(),
            })?;
            let row = row_num as u64;

            let get = |field: &str| -> Option<String> {
                index_map
                    .get(field)
                    .and_then(|&i| record.get(i))
                    .map(|s| s.trim().to_string())
            };

            let name = get("name").unwrap_or_default();
            let base_mass_kg: f64 = get("base_mass_kg")
                .ok_or_else(|| Error::ShipDataValidation {
                    message: format!("missing base_mass_kg for ship '{}' at row {}", name, row),
                })?
                .parse::<f64>()
                .map_err(|e| Error::ShipDataValidation {
                    message: format!(
                        "invalid base_mass_kg for ship '{}' at row {}: {}",
                        name, row, e
                    ),
                })?;
            let specific_heat: f64 = get("specific_heat")
                .ok_or_else(|| Error::ShipDataValidation {
                    message: format!("missing specific_heat for ship '{}' at row {}", name, row),
                })?
                .parse::<f64>()
                .map_err(|e| Error::ShipDataValidation {
                    message: format!(
                        "invalid specific_heat for ship '{}' at row {}: {}",
                        name, row, e
                    ),
                })?;
            let fuel_capacity: f64 = get("fuel_capacity")
                .ok_or_else(|| Error::ShipDataValidation {
                    message: format!("missing fuel_capacity for ship '{}' at row {}", name, row),
                })?
                .parse::<f64>()
                .map_err(|e| Error::ShipDataValidation {
                    message: format!(
                        "invalid fuel_capacity for ship '{}' at row {}: {}",
                        name, row, e
                    ),
                })?;
            let cargo_capacity: f64 = get("cargo_capacity")
                .ok_or_else(|| Error::ShipDataValidation {
                    message: format!("missing cargo_capacity for ship '{}' at row {}", name, row),
                })?
                .parse::<f64>()
                .map_err(|e| Error::ShipDataValidation {
                    message: format!(
                        "invalid cargo_capacity for ship '{}' at row {}: {}",
                        name, row, e
                    ),
                })?;

            let ship = ShipAttributes {
                name: name.trim().to_string(),
                base_mass_kg,
                specific_heat,
                fuel_capacity,
                cargo_capacity,
            };

            ship.validate()?;

            let key = normalize_name(&ship.name);
            if ships.contains_key(&key) {
                return Err(Error::DuplicateShipName { name: key });
            }
            ships.insert(key, ship);
        }

        Ok(Self {
            ships,
            source: None,
        })
    }

    /// Get a ship by name (case-insensitive).
    pub fn get(&self, name: &str) -> Option<&ShipAttributes> {
        self.ships.get(&normalize_name(name))
    }

    /// Get a sorted list of all ship names.
    pub fn ship_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.ships.values().map(|s| s.name.clone()).collect();
        names.sort();
        names
    }

    /// Get all ships sorted by name.
    pub fn ships_sorted(&self) -> Vec<&ShipAttributes> {
        let mut ships: Vec<&ShipAttributes> = self.ships.values().collect();
        ships.sort_by(|a, b| a.name.cmp(&b.name));
        ships
    }

    /// Get the source path if the catalog was loaded from a file.
    pub fn source_path(&self) -> Option<&Path> {
        self.source.as_deref()
    }
}

/// Normalize a ship name for case-insensitive lookup.
fn normalize_name(name: &str) -> String {
    name.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn capacity_m_caret_normalizes_to_capacity_m3_and_is_accepted() {
        let csv =
            "name,base_mass_kg,fuel_capacity,capacity_m^3,specific_heat\nReflex,1000,500,100,1.0\n";
        let r = Cursor::new(csv);
        let catalog = ShipCatalog::from_reader(r)
            .expect("should parse capacity_m^3 header via normalization");
        let ship = catalog.get("Reflex").expect("ship exists");
        assert_eq!(ship.cargo_capacity, 100.0);
    }
}
