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
/// Canonical heat classification thresholds (game-provided constants)
/// These represent absolute heat units used for route warnings.
pub const HEAT_NOMINAL: f64 = 30.0;
pub const HEAT_OVERHEATED: f64 = 90.0;
pub const HEAT_CRITICAL: f64 = 150.0;

/// Base cooling power (W/K units, effectively k scaled by mass). Tunable constant used for Newton's
/// Law of Cooling model. This value is calibrated to produce wait times in the minutes
/// range for ships in the 10^7 kg mass bracket.
pub const BASE_COOLING_POWER: f64 = 1e6;
/// Epsilon used to prevent logarithm domain errors when cooling toward ambient temperature.
pub const COOLING_EPSILON: f64 = 0.01;

/// Compute a zone factor from an external temperature (Kelvin). Colder environments cool more
/// effectively (factor closer to 1.0); hot environments cool poorly (factor near 0.0).
pub fn compute_zone_factor(min_external_temp: Option<f64>) -> f64 {
    match min_external_temp {
        None => 0.1, // conservative default when unknown
        Some(t) if !t.is_finite() => 0.1,
        Some(t) if t <= 30.0 => 1.0,
        Some(t) if t <= 100.0 => 0.7,
        Some(t) if t <= 300.0 => 0.4,
        Some(t) if t <= 1000.0 => 0.2,
        Some(_) => 0.05,
    }
}

/// Compute the cooling constant k (1/s) for Newton's Law of Cooling.
/// k = (BASE_COOLING_POWER * zone_factor) / thermal_mass
pub fn compute_cooling_constant(
    total_mass_kg: f64,
    specific_heat: f64,
    min_external_temp: Option<f64>,
) -> f64 {
    if !total_mass_kg.is_finite()
        || total_mass_kg <= 0.0
        || !specific_heat.is_finite()
        || specific_heat <= 0.0
    {
        return 0.0;
    }
    let zone_factor = compute_zone_factor(min_external_temp);
    (BASE_COOLING_POWER * zone_factor) / (total_mass_kg * specific_heat)
}

/// Calculate the time (seconds) required to cool from start_temp to target_temp
/// given a cooling constant k and environment temperature env_temp.
///
/// Formula: t = -(1/k) * ln((T_target - T_env) / (T_start - T_env))
pub fn calculate_cooling_time(start_temp: f64, target_temp: f64, env_temp: f64, k: f64) -> f64 {
    if start_temp <= target_temp || k <= 0.0 {
        return 0.0;
    }
    // Ambient temperature is the physical floor: we can't cool below it.
    // If env_temp >= target_temp, clamp the effective target to just above env_temp so that the
    // model reflects this constraint and avoids taking ln(0) or ln of a negative value when
    // target_temp is at or below env_temp.
    let target = target_temp.max(env_temp + COOLING_EPSILON);
    if start_temp <= target {
        return 0.0;
    }

    let ratio = (target - env_temp) / (start_temp - env_temp);
    -(1.0 / k) * ratio.ln()
}

/// Compute dissipation (heat-units per second) for a ship given its mass and specific heat and
/// an optional external temperature. Retained for backward compatibility or linear approximations.
pub fn compute_dissipation_per_sec(
    total_mass_kg: f64,
    specific_heat: f64,
    min_external_temp: Option<f64>,
) -> f64 {
    compute_cooling_constant(total_mass_kg, specific_heat, min_external_temp)
}

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

/// Calculate fuel units required for a single jump.
///
/// Formula: hop_cost = (total_mass_kg / 100_000) × (fuel_quality / 100) × distance_ly
///
/// Arguments:
/// - `total_mass_kg`: total operational mass (hull + fuel + cargo) in kilograms
/// - `distance_ly`: jump distance in light-years (must be > 0)
/// - `fuel_config`: contains `quality` (1..100) used as percentage and `dynamic_mass` flag
///
/// Returns a `Result<f64>` with the fuel units required for the hop or an error if inputs
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
/// Returns a vector of `FuelProjection` containing `hop_cost`, `cumulative`, `remaining` and
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

/// Configuration for heat calculations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeatConfig {
    pub calibration_constant: f64,
    pub dynamic_mass: bool,
}

impl Default for HeatConfig {
    fn default() -> Self {
        Self {
            // Use fixed internal calibration by default to keep outputs stable.
            calibration_constant: 1e-7,
            dynamic_mass: false,
            // cooling_mode removed: library uses Zone model by default
        }
    }
}

// CoolingMode removed: the library uses the Zone cooling model by default. If a
// more sophisticated or configurable cooling model is required in the future,
// reintroduce an explicit enum and associated configuration and tests.

/// Calculate the heat energy generated by a single jump.
///
/// Formula: energy = (3 × total_mass_kg × distance_ly) / (calibration_constant × hull_mass_kg)
///
/// Note: this function returns an energy-like value; callers may convert this to a temperature
/// change by dividing by (mass × specific_heat) to obtain delta-T in Kelvin.
pub fn calculate_jump_heat(
    total_mass_kg: f64,
    distance_ly: f64,
    hull_mass_kg: f64,
    calibration_constant: f64,
) -> Result<f64> {
    // Distance of zero is allowed (gate transitions -> zero heat)
    if !distance_ly.is_finite() || distance_ly < 0.0 {
        return Err(Error::ShipDataValidation {
            message: format!(
                "distance must be finite and non-negative, got {}",
                distance_ly
            ),
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

    if !hull_mass_kg.is_finite() || hull_mass_kg <= 0.0 {
        return Err(Error::ShipDataValidation {
            message: format!(
                "hull_mass_kg must be finite and positive, got {}",
                hull_mass_kg
            ),
        });
    }

    if !calibration_constant.is_finite() || calibration_constant <= 0.0 {
        return Err(Error::ShipDataValidation {
            message: format!(
                "calibration_constant must be finite and positive, got {}",
                calibration_constant
            ),
        });
    }

    if distance_ly == 0.0 {
        return Ok(0.0);
    }

    // Compute heat using formula from research.md
    let heat = (3.0 * total_mass_kg * distance_ly) / (calibration_constant * hull_mass_kg);

    if !heat.is_finite() {
        return Err(Error::ShipDataValidation {
            message: "calculated heat must be finite".to_string(),
        });
    }

    if heat < 0.0 {
        return Err(Error::ShipDataValidation {
            message: format!("calculated heat must be non-negative, got {}", heat),
        });
    }

    Ok(heat)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeatProjection {
    pub hop_heat: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_time_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub residual_heat: Option<f64>,
    pub can_proceed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeatSummary {
    /// Total time spent cooling between hops (seconds).
    pub total_wait_time_seconds: f64,
    /// Final residual heat at the destination after any cooling at the point of arrival.
    pub final_residual_heat: f64,
    /// Warnings collected across all steps of the route.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ShipCatalog {
    ships: HashMap<String, ShipAttributes>,
    source: Option<PathBuf>,
}

impl ShipCatalog {
    pub fn from_path(path: &Path) -> Result<Self> {
        // If the provided path appears to be a checksum sidecar (e.g. `.../e6c4-ship_data.csv.sha256`),
        // attempt to locate the corresponding CSV (`.../e6c4-ship_data.csv`) next to it and use that
        // file instead. This handles cases where cache discovery accidentally returns the sidecar
        // path; attempting to parse the sidecar as CSV leads to confusing "missing columns" errors.
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

    pub fn from_reader<R: Read>(reader: R) -> Result<Self> {
        let mut csv_reader = ReaderBuilder::new().trim(Trim::Fields).from_reader(reader);

        let headers = csv_reader
            .headers()
            .map_err(|err| Error::ShipDataValidation {
                message: format!("failed to read ship_data.csv headers: {err}"),
            })?
            .clone();

        // Helper to normalize header strings for robust matching.
        // Normalization lower-cases the header and strips any non-alphanumeric
        // characters except underscores. This makes common variants like
        // "Fuel-Capacity" or "Fuel.Capacity" normalize to the same token
        // (e.g. "fuelcapacity"), and converts "capacity_m^3" to
        // "capacity_m3" which is handled by the synonyms below. This is a
        // documented transformation and the synonym set accounts for typical
        // variations.
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
            // Note: heat-specific columns (tolerance, dissipation) are NOT
            // expected in the canonical CSV and are intentionally omitted here.
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
        // Required fields exclude heat-specific metrics which we can default when
        // releases omit them (older releases may not include heat columns).
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

        // Track record position for better error messages (helps identify bad rows).
        // We maintain a manual row counter to avoid borrowing `csv_reader`
        // immutably while iterating via `records()` (which borrows it mutably).
        let mut row_num: usize = 1; // header is typically line 1
        for result in csv_reader.records() {
            row_num += 1; // first record will be row 2
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
            // Heat-specific fields are not parsed from CSV (canonical CSV does not include them).

            let ship = ShipAttributes {
                name: name.trim().to_string(),
                base_mass_kg,
                specific_heat,
                fuel_capacity,
                cargo_capacity,
                // no per-ship heat fields
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn defaults_are_valid() {
        // No per-ship heat defaults; thresholds are canonical constants validated elsewhere.
        assert!(HEAT_NOMINAL.is_finite());
        assert!(HEAT_OVERHEATED.is_finite());
        assert!(HEAT_CRITICAL.is_finite());
    }

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

    #[test]
    fn test_compute_cooling_constant() {
        // Base case: 1M kg ship, 1.0 specific heat, cold env (factor 1.0)
        // k = (1e6 * 1.0) / (1e6 * 1.0) = 1.0
        let k = compute_cooling_constant(1e6, 1.0, Some(30.0));
        assert!((k - 1.0).abs() < f64::EPSILON);

        // invalid inputs
        assert_eq!(compute_cooling_constant(0.0, 1.0, None), 0.0);
        assert_eq!(compute_cooling_constant(1e6, 0.0, None), 0.0);
    }

    #[test]
    fn test_calculate_cooling_time() {
        let k = 1.0;
        let env = 30.0;

        // start <= target: no wait
        assert_eq!(calculate_cooling_time(50.0, 60.0, env, k), 0.0);

        // start > target: should take time
        // t = -ln((60 - 30) / (100 - 30)) = -ln(30/70) = ln(7/3) ≈ 0.847
        let t = calculate_cooling_time(100.0, 60.0, env, k);
        assert!((t - (70.0 / 30.0f64).ln()).abs() < 1e-6);

        // target < env: clamped to env + COOLING_EPSILON
        // target effectively becomes 30.01 (env + 0.01)
        let t_clamped = calculate_cooling_time(100.0, 10.0, env, k);
        let t_expected = -(1.0 / k) * ((COOLING_EPSILON) / (100.0 - 30.0)).ln();
        assert!((t_clamped - t_expected).abs() < 1e-6);

        // k <= 0
        assert_eq!(calculate_cooling_time(100.0, 60.0, env, 0.0), 0.0);
    }
}
