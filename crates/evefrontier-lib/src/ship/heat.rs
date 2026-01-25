//! Heat calculation, cooling, and projection types.
//!
//! This module handles heat generation from ship jumps, cooling calculations
//! using Newton's Law of Cooling, and per-hop heat projections.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

use super::constants::{
    BASE_COOLING_POWER, COOLING_EPSILON, HEAT_CRITICAL, HEAT_NOMINAL, HEAT_OVERHEATED,
};

/// Configuration for heat calculations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeatConfig {
    /// Calibration constant for heat energy formula.
    pub calibration_constant: f64,
    /// Enable per-hop dynamic mass recalculation as fuel is consumed.
    pub dynamic_mass: bool,
}

impl Default for HeatConfig {
    fn default() -> Self {
        Self {
            // Use fixed internal calibration by default to keep outputs stable.
            calibration_constant: 1e-7,
            dynamic_mass: false,
        }
    }
}

/// Heat projection for a single hop.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeatProjection {
    /// Temperature increase (delta-T) in Kelvin for this hop.
    pub hop_heat: f64,
    /// Warning message (e.g., "OVERHEATED", "CRITICAL").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
    /// Optional cooldown time in seconds to reach nominal temperature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_time_seconds: Option<f64>,
    /// Residual temperature at arrival after any optional cooldown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub residual_heat: Option<f64>,
    /// Whether the ship can safely proceed to the next hop.
    pub can_proceed: bool,
}

/// Summary of heat state across an entire route.
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

/// Parameters for heat projection calculation.
///
/// Groups related parameters to avoid exceeding clippy's `too_many_arguments` threshold.
#[derive(Debug, Clone, Copy)]
pub struct HeatProjectionParams {
    /// Total operational mass (hull + fuel + cargo) in kilograms
    pub mass: f64,
    /// Ship's specific heat capacity in J/(kg·K)
    pub specific_heat: f64,
    /// Hop distance in light-years (>= 0; 0 means gate/zero-heat)
    pub distance_ly: f64,
    /// Hull mass used by heat energy calibration formula
    pub hull_mass_kg: f64,
    /// Calibration constant for heat energy formula
    pub calibration_constant: f64,
    /// Ambient temperature at origin system (K), if known
    pub prev_ambient: Option<f64>,
    /// Ambient temperature at destination system (K), if known
    pub current_min_external_temp: Option<f64>,
    /// True if this hop arrives at final destination (no cooldown required)
    pub is_goal: bool,
    /// True if next hop is a gate (cooldown not required before gate)
    pub next_is_gate: bool,
}

/// Compute a zone factor from an external temperature (Kelvin).
///
/// Colder environments cool more effectively (factor closer to 1.0);
/// hot environments cool poorly (factor near 0.0).
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
///
/// Formula: k = (BASE_COOLING_POWER * zone_factor) / thermal_mass
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
/// Formula: t = -(1/k) * ln((T_target - T_env) / (start_temp - T_env))
pub fn calculate_cooling_time(start_temp: f64, target_temp: f64, env_temp: f64, k: f64) -> f64 {
    if !start_temp.is_finite()
        || !target_temp.is_finite()
        || !env_temp.is_finite()
        || !k.is_finite()
        || start_temp <= target_temp
        || k <= 0.0
    {
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
/// an optional external temperature.
///
/// Retained for backward compatibility or linear approximations.
pub fn compute_dissipation_per_sec(
    total_mass_kg: f64,
    specific_heat: f64,
    min_external_temp: Option<f64>,
) -> f64 {
    compute_cooling_constant(total_mass_kg, specific_heat, min_external_temp)
}

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

/// Project the per-hop heat (delta-T), warnings, and optional cooldown based on
/// ship properties and environmental conditions.
///
/// This mirrors the route attach_heat logic and is exposed to keep calculations DRY
/// across callers (route, scout, and Lambdas).
///
/// # Arguments
/// * `params` - Heat projection parameters bundled in `HeatProjectionParams`
///
/// # Returns
/// `HeatProjection` with:
/// - `hop_heat`: temperature delta (K) for this hop
/// - `warning`: OVERHEATED/CRITICAL if instantaneous temperature exceeds thresholds
/// - `wait_time_seconds`: optional cooldown to reach nominal temperature before next jump
/// - `residual_heat`: temperature at arrival after any optional cooldown
/// - `can_proceed`: whether ship can proceed under the cooling model
pub fn project_heat_for_jump(params: HeatProjectionParams) -> Result<HeatProjection> {
    // Validate inputs
    if !params.mass.is_finite() || params.mass <= 0.0 {
        return Err(Error::ShipDataValidation {
            message: format!(
                "computed mass must be finite and positive, got {}",
                params.mass
            ),
        });
    }
    if !params.specific_heat.is_finite() || params.specific_heat <= 0.0 {
        return Err(Error::ShipDataValidation {
            message: format!("invalid specific_heat: {}", params.specific_heat),
        });
    }
    if !params.distance_ly.is_finite() || params.distance_ly < 0.0 {
        return Err(Error::ShipDataValidation {
            message: format!(
                "distance must be finite and non-negative, got {}",
                params.distance_ly
            ),
        });
    }

    // Zero-distance hops (gates) generate no heat
    if params.distance_ly == 0.0 {
        return Ok(HeatProjection {
            hop_heat: 0.0,
            warning: None,
            wait_time_seconds: None,
            residual_heat: Some(HEAT_NOMINAL),
            can_proceed: true,
        });
    }

    // Calculate energy then convert to delta-T: ΔT = energy / (m · c)
    let hop_energy = calculate_jump_heat(
        params.mass,
        params.distance_ly,
        params.hull_mass_kg,
        params.calibration_constant,
    )?;
    let hop_heat = hop_energy / (params.mass * params.specific_heat);

    // Starting temperature is nominal or the origin ambient, whichever is higher
    let start_temp = HEAT_NOMINAL.max(params.prev_ambient.unwrap_or(0.0));
    let candidate = start_temp + hop_heat;

    // Determine warnings from instantaneous temperature
    let mut warn: Option<String> = None;
    if candidate >= HEAT_CRITICAL {
        warn = Some("CRITICAL".to_string());
    } else if candidate >= HEAT_OVERHEATED {
        warn = Some("OVERHEATED".to_string());
    }

    // Cooldown policy: if above nominal and there is a subsequent jump
    // (not goal and not gate), cool back toward nominal before proceeding.
    let mut wait_time: Option<f64> = None;
    let mut residual = candidate;
    let mut can_proceed = true;

    let target = HEAT_NOMINAL;
    if candidate > target && !params.is_goal && !params.next_is_gate {
        let k = compute_cooling_constant(
            params.mass,
            params.specific_heat,
            params.current_min_external_temp,
        );
        if k > 0.0 {
            let env_temp = params.current_min_external_temp.unwrap_or(0.0);
            let wait = calculate_cooling_time(candidate, target, env_temp, k);
            if wait > 0.0 {
                wait_time = Some(wait);
                // After waiting, residual is at the floor (nominal or ambient)
                residual = target.max(env_temp);
            }
            can_proceed = true;
        } else {
            // No cooling available (invalid k); cannot safely proceed
            wait_time = None;
            can_proceed = false;
            residual = candidate;
        }
    }

    Ok(HeatProjection {
        hop_heat,
        warning: warn,
        wait_time_seconds: wait_time,
        residual_heat: Some(residual),
        can_proceed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() {
        assert!(HEAT_NOMINAL.is_finite());
        assert!(HEAT_OVERHEATED.is_finite());
        assert!(HEAT_CRITICAL.is_finite());
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
    fn calculate_cooling_time_exponential_decay_formula() {
        let k = 1.0;
        let env = 30.0;

        // start <= target: no wait
        assert_eq!(calculate_cooling_time(50.0, 60.0, env, k), 0.0);

        // start > target: should take time
        // t = -ln((60 - 30) / (100 - 30)) = -ln(3/7) = ln(7/3) ≈ 0.847
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
