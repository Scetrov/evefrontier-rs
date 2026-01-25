//! Ship attributes and loadout configuration.
//!
//! This module contains the core ship data structures that describe a ship's
//! physical properties and operational configuration.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

use super::constants::FUEL_MASS_PER_UNIT_KG;

/// Ship physical attributes loaded from ship data catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShipAttributes {
    pub name: String,
    pub base_mass_kg: f64,
    pub specific_heat: f64,
    pub fuel_capacity: f64,
    pub cargo_capacity: f64,
    // Per-ship heat tolerance and dissipation are not provided by the canonical
    // game CSV; we do not store per-ship tolerances. Heat warnings are based
    // on canonical thresholds (HEAT_OVERHEATED, HEAT_CRITICAL).
}

impl ShipAttributes {
    /// Validate ship attributes for correctness.
    pub fn validate(&self) -> Result<()> {
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

/// Ship operational loadout (fuel and cargo configuration).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShipLoadout {
    pub fuel_load: f64,
    pub cargo_mass_kg: f64,
}

impl ShipLoadout {
    /// Create a new loadout, validating against ship capacities.
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

    /// Create a loadout with full fuel tank and no cargo.
    pub fn full_fuel(ship: &ShipAttributes) -> Self {
        Self {
            fuel_load: ship.fuel_capacity,
            cargo_mass_kg: 0.0,
        }
    }

    /// Calculate total operational mass (hull + fuel + cargo).
    pub fn total_mass_kg(&self, ship: &ShipAttributes) -> f64 {
        ship.base_mass_kg + (self.fuel_load * FUEL_MASS_PER_UNIT_KG) + self.cargo_mass_kg
    }
}
