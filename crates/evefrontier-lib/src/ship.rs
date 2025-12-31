use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use csv::{ReaderBuilder, Trim};
use serde::{Deserialize, Serialize};

use crate::{error::Result, Error};

/// Mass of one fuel unit in kilograms.
pub const FUEL_MASS_PER_UNIT_KG: f64 = 1.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShipAttributes {
    pub name: String,
    pub base_mass_kg: f64,
    pub specific_heat: f64,
    pub fuel_capacity: f64,
    pub cargo_capacity: f64,
    pub max_heat_tolerance: f64,
    pub heat_dissipation_rate: f64,
}

impl ShipAttributes {
    fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::ShipDataValidation {
                message: "ship name must not be empty".to_string(),
            });
        }

        let fields = [
            (self.base_mass_kg, "base_mass_kg"),
            (self.specific_heat, "specific_heat"),
            (self.fuel_capacity, "fuel_capacity"),
            (self.cargo_capacity, "cargo_capacity"),
            (self.max_heat_tolerance, "max_heat_tolerance"),
            (self.heat_dissipation_rate, "heat_dissipation_rate"),
        ];

        for (value, field) in fields {
            if !value.is_finite() || value <= 0.0 {
                return Err(Error::ShipDataValidation {
                    message: format!("{field} must be a finite positive number"),
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShipLoadout {
    pub fuel_load: f64,
    pub cargo_mass_kg: f64,
}

impl ShipLoadout {
    pub fn new(ship: &ShipAttributes, fuel_load: f64, cargo_mass_kg: f64) -> Result<Self> {
        if !fuel_load.is_finite() || fuel_load < 0.0 {
            return Err(Error::ShipDataValidation {
                message: "fuel_load must be finite and non-negative".to_string(),
            });
        }
        if fuel_load > ship.fuel_capacity {
            return Err(Error::ShipDataValidation {
                message: "fuel_load exceeds ship fuel_capacity".to_string(),
            });
        }
        if !cargo_mass_kg.is_finite() || cargo_mass_kg < 0.0 {
            return Err(Error::ShipDataValidation {
                message: "cargo_mass_kg must be finite and non-negative".to_string(),
            });
        }

        Ok(Self {
            fuel_load,
            cargo_mass_kg,
        })
    }

    pub fn full_fuel(ship: &ShipAttributes) -> Self {
        Self {
            fuel_load: ship.fuel_capacity,
            cargo_mass_kg: 0.0,
        }
    }

    pub fn total_mass_kg(&self, ship: &ShipAttributes) -> f64 {
        ship.base_mass_kg + (self.fuel_load * FUEL_MASS_PER_UNIT_KG) + self.cargo_mass_kg
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FuelConfig {
    pub quality: f64,
    pub dynamic_mass: bool,
}

impl Default for FuelConfig {
    fn default() -> Self {
        Self {
            quality: 10.0,
            dynamic_mass: false,
        }
    }
}

impl FuelConfig {
    pub fn validate(&self) -> Result<()> {
        if !self.quality.is_finite() {
            return Err(Error::ShipDataValidation {
                message: "fuel_quality must be finite".to_string(),
            });
        }

        if !(1.0..=100.0).contains(&self.quality) {
            return Err(Error::ShipDataValidation {
                message: format!(
                    "fuel_quality must be between 1 and 100, got {}",
                    self.quality
                ),
            });
        }

        Ok(())
    }

    fn quality_factor(&self) -> Result<f64> {
        self.validate()?;
        Ok(self.quality / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FuelProjection {
    pub hop_cost: f64,
    pub cumulative: f64,
    pub remaining: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

pub fn calculate_jump_fuel_cost(
    total_mass_kg: f64,
    distance_ly: f64,
    fuel_config: &FuelConfig,
) -> Result<f64> {
    if !distance_ly.is_finite() || distance_ly <= 0.0 {
        return Err(Error::ShipDataValidation {
            message: format!("distance must be finite and positive, got {}", distance_ly),
        });
    }

    if !total_mass_kg.is_finite() || total_mass_kg <= 0.0 {
        return Err(Error::ShipDataValidation {
            message: format!(
                "total_mass_kg must be finite and positive, got {}",
                total_mass_kg
            ),
        });
    }

    let mass_factor = total_mass_kg / 100_000.0;
    let quality_factor = fuel_config.quality_factor()?;

    Ok(mass_factor * quality_factor * distance_ly)
}

pub fn calculate_route_fuel(
    ship: &ShipAttributes,
    loadout: &ShipLoadout,
    distances_ly: &[f64],
    fuel_config: &FuelConfig,
) -> Result<Vec<FuelProjection>> {
    fuel_config.validate()?;

    let mut projections = Vec::with_capacity(distances_ly.len());
    let mut cumulative = 0.0;
    let mut dynamic_fuel_load = loadout.fuel_load;

    for &distance in distances_ly {
        if !distance.is_finite() || distance <= 0.0 {
            return Err(Error::ShipDataValidation {
                message: format!("distance must be finite and positive, got {}", distance),
            });
        }

        let effective_fuel = if fuel_config.dynamic_mass {
            dynamic_fuel_load
        } else {
            loadout.fuel_load
        };

        let mass =
            ship.base_mass_kg + loadout.cargo_mass_kg + (effective_fuel * FUEL_MASS_PER_UNIT_KG);

        if !mass.is_finite() || mass <= 0.0 {
            return Err(Error::ShipDataValidation {
                message: format!("computed mass must be finite and positive, got {}", mass),
            });
        }

        let hop_cost = calculate_jump_fuel_cost(mass, distance, fuel_config)?;
        cumulative += hop_cost;

        let remaining = if fuel_config.dynamic_mass {
            dynamic_fuel_load = (dynamic_fuel_load - hop_cost).max(0.0);
            Some(dynamic_fuel_load)
        } else {
            Some((loadout.fuel_load - cumulative).max(0.0))
        };

        projections.push(FuelProjection {
            hop_cost,
            cumulative,
            remaining,
            warning: None,
        });
    }

    Ok(projections)
}

#[derive(Debug, Clone, Default)]
pub struct ShipCatalog {
    ships: HashMap<String, ShipAttributes>,
    source: Option<PathBuf>,
}

impl ShipCatalog {
    pub fn from_path(path: &Path) -> Result<Self> {
        let file = fs::File::open(path)?;
        let mut catalog = Self::from_reader(file)?;
        catalog.source = Some(path.to_path_buf());
        Ok(catalog)
    }

    pub fn from_reader<R: Read>(reader: R) -> Result<Self> {
        let mut csv_reader = ReaderBuilder::new().trim(Trim::Fields).from_reader(reader);

        let headers = csv_reader
            .headers()
            .map_err(|err| Error::ShipDataValidation {
                message: format!("failed to read ship_data.csv headers: {err}"),
            })?
            .clone();

        let required_headers = [
            "name",
            "base_mass_kg",
            "specific_heat",
            "fuel_capacity",
            "cargo_capacity",
            "max_heat_tolerance",
            "heat_dissipation_rate",
        ];

        let missing: Vec<&str> = required_headers
            .iter()
            .copied()
            .filter(|header| !headers.iter().any(|h| h == *header))
            .collect();

        if !missing.is_empty() {
            return Err(Error::ShipDataValidation {
                message: format!(
                    "ship_data.csv missing required columns: {}",
                    missing.join(", ")
                ),
            });
        }

        let mut ships = HashMap::new();

        for record in csv_reader.deserialize::<ShipAttributes>() {
            let mut ship: ShipAttributes = record.map_err(|err| Error::ShipDataValidation {
                message: err.to_string(),
            })?;
            ship.name = ship.name.trim().to_string();
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

    pub fn get(&self, name: &str) -> Option<&ShipAttributes> {
        self.ships.get(&normalize_name(name))
    }

    pub fn ship_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.ships.values().map(|s| s.name.clone()).collect();
        names.sort();
        names
    }

    pub fn ships_sorted(&self) -> Vec<&ShipAttributes> {
        let mut ships: Vec<&ShipAttributes> = self.ships.values().collect();
        ships.sort_by(|a, b| a.name.cmp(&b.name));
        ships
    }

    pub fn source_path(&self) -> Option<&Path> {
        self.source.as_deref()
    }
}

fn normalize_name(name: &str) -> String {
    name.trim().to_lowercase()
}
