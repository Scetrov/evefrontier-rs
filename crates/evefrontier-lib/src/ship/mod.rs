//! Ship data types, fuel and heat calculations, and catalog management.
//!
//! This module is organized into focused submodules:
//!
//! - [`attributes`] - Ship physical attributes and loadout configuration
//! - [`fuel`] - Fuel calculation and projection types
//! - [`heat`] - Heat calculation, cooling, and projection types
//! - [`catalog`] - Ship catalog loading and management
//! - [`constants`] - Shared constants used across calculations
//!
//! # Example
//!
//! ```no_run
//! use evefrontier_lib::ship::{
//!     ShipAttributes, ShipLoadout, FuelConfig, calculate_jump_fuel_cost,
//!     HeatConfig, calculate_jump_heat, ShipCatalog,
//! };
//!
//! // Load ships from catalog
//! let catalog = ShipCatalog::from_path(std::path::Path::new("ship_data.csv")).unwrap();
//! let ship = catalog.get("Reflex").unwrap();
//!
//! // Create a loadout
//! let loadout = ShipLoadout::new(ship, 500.0, 0.0).unwrap();
//!
//! // Calculate fuel cost for a jump
//! let mass = loadout.total_mass_kg(ship);
//! let fuel_config = FuelConfig::default();
//! let fuel_cost = calculate_jump_fuel_cost(mass, 10.0, &fuel_config).unwrap();
//! ```

pub mod attributes;
pub mod catalog;
pub mod constants;
pub mod fuel;
pub mod heat;

// Re-export all public items for backward compatibility
pub use attributes::{ShipAttributes, ShipLoadout};
pub use catalog::ShipCatalog;
pub use constants::{
    BASE_COOLING_POWER, COOLING_EPSILON, FUEL_MASS_PER_UNIT_KG, HEAT_CRITICAL, HEAT_NOMINAL,
    HEAT_OVERHEATED,
};
pub use fuel::{
    calculate_jump_fuel_cost, calculate_maximum_distance, calculate_route_fuel,
    project_fuel_for_hop, FuelConfig, FuelProjection,
};
pub use heat::{
    calculate_cooling_time, calculate_jump_heat, compute_cooling_constant,
    compute_dissipation_per_sec, compute_zone_factor, project_heat_for_jump, HeatConfig,
    HeatProjection, HeatProjectionParams, HeatSummary,
};
