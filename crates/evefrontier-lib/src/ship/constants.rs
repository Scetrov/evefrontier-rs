//! Ship-related constants used across fuel and heat calculations.

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
///
/// When the target temperature approaches or is at/below the environment temperature,
/// the Newton's Law of Cooling formula can otherwise attempt to take `ln(0)` or a negative
/// value, which is outside the valid domain of the logarithm and would result in NaNs.
/// A tolerance of 0.01 K keeps us just inside the valid domain of `ln`, while being
/// physically and gameplay-wise negligible for cooling times (sub‑percent impact on
/// computed wait durations). This value was chosen as a pragmatic balance between
/// numerical robustness and not materially inflating cooling waits.
pub const COOLING_EPSILON: f64 = 0.01;
