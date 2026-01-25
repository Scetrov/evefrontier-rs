//! Fuel calculation and projection types.
//!
//! This module handles fuel consumption calculations for ship jumps,
//! including per-hop projections and route-wide fuel planning.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

use super::attributes::{ShipAttributes, ShipLoadout};
use super::constants::FUEL_MASS_PER_UNIT_KG;

/// Fuel calculation configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FuelConfig {
    /// Fuel quality percentage (1-100). Higher quality = more efficient jumps.
    pub quality: f64,
    /// Enable per-hop dynamic mass recalculation as fuel is consumed.
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
    /// Validate the fuel configuration.
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

    /// Converts the quality percentage (1-100) to a multiplier factor (0.01-1.0).
    ///
    /// This method validates the fuel configuration before performing the conversion,
    /// ensuring the quality value is within the valid range. It is kept private as
    /// external callers should use the public fuel calculation functions which handle
    /// validation and factor conversion internally.
    ///
    /// # Errors
    /// Returns an error if validation fails (e.g., quality outside 1-100 range).
    fn quality_factor(&self) -> Result<f64> {
        self.validate()?;
        Ok(self.quality / 100.0)
    }
}

/// Fuel projection for a single hop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FuelProjection {
    /// Fuel units consumed for this hop.
    pub hop_cost: f64,
    /// Cumulative fuel consumed up to and including this hop.
    pub cumulative: f64,
    /// Fuel remaining after this hop.
    pub remaining: Option<f64>,
    /// Warning message (e.g., "REFUEL" if insufficient fuel).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

/// Project fuel consumption for a single hop, including refuel detection and warning generation.
///
/// This helper encapsulates the common logic for tracking fuel consumption across route hops:
/// - Compares hop fuel cost against remaining fuel to detect insufficient fuel scenarios
/// - Generates REFUEL warning when necessary and resets remaining fuel to capacity
/// - Otherwise subtracts hop cost from remaining fuel (clamped to zero)
/// - Returns both the FuelProjection and the updated remaining fuel value
///
/// # Arguments
/// * `hop_cost` - Fuel units required for this hop (from calculate_jump_fuel_cost)
/// * `cumulative_fuel` - Total fuel consumed so far across all previous hops
/// * `remaining_fuel` - Fuel available before this hop
/// * `fuel_capacity` - Ship's fuel tank capacity (used for refuel reset)
///
/// # Returns
/// A tuple of (FuelProjection, new_remaining_fuel) where:
/// - FuelProjection contains hop_cost, cumulative, remaining, and optional warning
/// - new_remaining_fuel is the updated fuel level after this hop (capacity if refueled, or remaining - hop_cost)
///
/// # Examples
/// ```
/// use evefrontier_lib::ship::project_fuel_for_hop;
///
/// // Normal hop: sufficient fuel
/// let (projection, remaining) = project_fuel_for_hop(50.0, 100.0, 200.0, 1000.0);
/// assert_eq!(projection.hop_cost, 50.0);
/// assert_eq!(projection.cumulative, 100.0);
/// assert_eq!(projection.remaining, Some(150.0));
/// assert_eq!(projection.warning, None);
/// assert_eq!(remaining, 150.0);
///
/// // Refuel scenario: insufficient fuel
/// let (projection, remaining) = project_fuel_for_hop(250.0, 100.0, 200.0, 1000.0);
/// assert_eq!(projection.hop_cost, 250.0);
/// assert_eq!(projection.cumulative, 100.0);
/// assert_eq!(projection.remaining, Some(1000.0));  // Reset to capacity
/// assert_eq!(projection.warning, Some("REFUEL".to_string()));
/// assert_eq!(remaining, 1000.0);
/// ```
pub fn project_fuel_for_hop(
    hop_cost: f64,
    cumulative_fuel: f64,
    remaining_fuel: f64,
    fuel_capacity: f64,
) -> (FuelProjection, f64) {
    if hop_cost > remaining_fuel {
        // Insufficient fuel: REFUEL warning and reset to capacity
        (
            FuelProjection {
                hop_cost,
                cumulative: cumulative_fuel,
                remaining: Some(fuel_capacity),
                warning: Some("REFUEL".to_string()),
            },
            fuel_capacity,
        )
    } else {
        // Sufficient fuel: consume fuel for this hop
        let new_remaining = (remaining_fuel - hop_cost).max(0.0);
        (
            FuelProjection {
                hop_cost,
                cumulative: cumulative_fuel,
                remaining: Some(new_remaining),
                warning: None,
            },
            new_remaining,
        )
    }
}

/// Calculate fuel units required for a single jump.
///
/// Formula: hop_cost = (total_mass_kg / 100_000) × (fuel_quality / 100) × distance_ly
///
/// # Arguments
/// - `total_mass_kg`: total operational mass (hull + fuel + cargo) in kilograms
/// - `distance_ly`: jump distance in light-years (must be > 0)
/// - `fuel_config`: contains `quality` (1..100) used as percentage and `dynamic_mass` flag
///
/// # Returns
/// `Result<f64>` with the fuel units required for the hop or an error if inputs
/// are invalid (e.g., non-finite values or invalid quality).
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

/// Calculate the maximum distance (light-years) a ship can travel given a fuel load and
/// fuel quality using the same mass-distance conversion factor used by the fuel cost formula.
///
/// Formula (consistent with `calculate_jump_fuel_cost`):
/// max_distance = (fuel_units * quality_factor * CONVERSION) / ship_mass
///
/// Where:
/// - `fuel_units` is the fuel load in units
/// - `quality_factor` is `fuel_quality / 100.0` (same scale used by `FuelConfig`)
/// - `CONVERSION` is 100_000.0 (mass-distance conversion factor)
pub fn calculate_maximum_distance(
    fuel_units: f64,
    ship_mass_kg: f64,
    fuel_quality: f64,
) -> Result<f64> {
    if !fuel_units.is_finite() || fuel_units < 0.0 {
        return Err(Error::ShipDataValidation {
            message: format!(
                "fuel_units must be finite and non-negative, got {}",
                fuel_units
            ),
        });
    }

    if !ship_mass_kg.is_finite() || ship_mass_kg <= 0.0 {
        return Err(Error::ShipDataValidation {
            message: format!(
                "ship_mass_kg must be finite and positive, got {}",
                ship_mass_kg
            ),
        });
    }

    if !fuel_quality.is_finite() {
        return Err(Error::ShipDataValidation {
            message: format!("fuel_quality must be finite, got {}", fuel_quality),
        });
    }

    // Interpret quality as percent (1..100) to match FuelConfig behavior
    let quality_factor = fuel_quality / 100.0;
    let conversion = 100_000.0;

    Ok((fuel_units * quality_factor * conversion) / ship_mass_kg)
}

/// Compute per-hop and cumulative fuel projections for a full route.
///
/// Repeatedly calls `calculate_jump_fuel_cost` for each hop. When `fuel_config.dynamic_mass` is
/// true, remaining fuel is decremented per hop and used to recompute mass; otherwise static
/// mode uses the initial fuel load for all hops.
///
/// # Returns
/// A vector of `FuelProjection` containing `hop_cost`, `cumulative`, `remaining` and
/// optional `warning` fields.
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
