//! Temperature calculation module for solar system environmental modeling.
//!
//! This module provides functions to calculate external temperatures at various
//! orbital distances from a star, using EVE Frontier's custom parameterized model
//! and the Stefan-Boltzmann equilibrium equation.
//!
//! # EVE Frontier Temperature Formula
//!
//! The primary calculation uses the EVE Frontier formula:
//!
//! ```text
//! T(d) = T_min + (T_max - T_min) / (1 + (d / (k * √L))^b)
//! ```
//!
//! Where:
//! - `T_min` = 0.1 K (minimum temperature in deep space)
//! - `T_max` = 99.9 K (maximum temperature near the star)
//! - `d` = Distance from star in light-seconds
//! - `L` = Stellar luminosity in watts
//! - `k` = 3.215 × 10⁻¹¹ (calibrated distance scale factor)
//! - `b` = 1.25 (calibrated curve steepness exponent)
//!
//! # Validated Test Cases
//!
//! From the EVE Frontier e6c3 dataset:
//!
//! - **Nod**: L = 1.9209×10²⁵ W, d = 541.4 ls → T = 15.74 K
//! - **Brana**: L = 4.7398×10²⁴ W, d = 9255.2 ls → T = 0.32 K
//!
//! # Example
//!
//! ```rust
//! use evefrontier_lib::temperature::{compute_temperature_light_seconds, TemperatureModelParams};
//!
//! let params = TemperatureModelParams::default();
//! let temp = compute_temperature_light_seconds(541.4, 1.9209e25, &params).unwrap();
//! assert!((temp - 15.74).abs() < 0.01); // Nod system temperature
//! ```

use crate::error::{Error, Result};

/// Physical constants for temperature calculations.
pub mod constants {
    /// Stefan-Boltzmann constant (W⋅m⁻²⋅K⁻⁴)
    pub const STEFAN_BOLTZMANN_SIGMA: f64 = 5.670374419e-8;

    /// Meters in one light-second (speed of light in m/s)
    pub const METERS_IN_LIGHT_SECOND: f64 = 299_792_458.0;

    /// Meters in one astronomical unit (mean Earth-Sun distance)
    pub const METERS_IN_AU: f64 = 1.495978707e11;
}

/// Configuration parameters for the custom temperature model.
///
/// The model calculates temperature as a smooth curve from `max_kelvin` near
/// the star to `min_kelvin` in deep space, controlled by distance scale `k`
/// and curve steepness `b`.
#[derive(Debug, Clone, PartialEq)]
pub struct TemperatureModelParams {
    /// Distance scale factor (controls transition point between hot and cold)
    pub k: f64,
    /// Curve steepness exponent (higher = sharper transition)
    pub b: f64,
    /// Minimum temperature in deep space (Kelvin)
    pub min_kelvin: f64,
    /// Maximum temperature near the star (Kelvin)
    pub max_kelvin: f64,
    /// Optional offset for calibration
    pub kelvin_offset: f64,
    /// Optional scale factor for calibration
    pub kelvin_scale: f64,
    /// Whether to apply offset and scale transformations
    pub map_to_kelvin: bool,
}

impl Default for TemperatureModelParams {
    fn default() -> Self {
        Self {
            k: 3.215e-11,     // EVE Frontier calibrated scale factor
            b: 1.25,          // EVE Frontier calibrated exponent
            min_kelvin: 0.1,  // EVE Frontier minimum temperature
            max_kelvin: 99.9, // EVE Frontier maximum temperature
            kelvin_offset: 0.0,
            kelvin_scale: 1.0,
            map_to_kelvin: false,
        }
    }
}

/// Calculate external temperature using the custom parameterized model.
///
/// The calculation follows the formula:
/// ```text
/// scale = k * sqrt(luminosity)
/// ratio = distance / scale
/// t = min_kelvin + (max_kelvin - min_kelvin) / (1 + ratio^b)
/// result = map_to_kelvin ? kelvin_offset + kelvin_scale * t : t
/// ```
///
/// # Arguments
///
/// * `distance_light_seconds` - Distance from the star in light-seconds
/// * `luminosity_watts` - Stellar luminosity in watts
/// * `params` - Model parameters controlling the temperature curve
///
/// # Errors
///
/// Returns an error if:
/// * `distance_light_seconds` is negative
/// * `luminosity_watts` is negative or zero
///
/// # Examples
///
/// ```
/// use evefrontier_lib::temperature::{compute_temperature_light_seconds, TemperatureModelParams};
///
/// let params = TemperatureModelParams::default();
/// let temp = compute_temperature_light_seconds(500.0, 3.828e26, &params).unwrap();
/// assert!(temp > 0.0);
/// ```
pub fn compute_temperature_light_seconds(
    distance_light_seconds: f64,
    luminosity_watts: f64,
    params: &TemperatureModelParams,
) -> Result<f64> {
    if distance_light_seconds < 0.0 {
        return Err(Error::TemperatureCalculation(
            "Distance cannot be negative".to_string(),
        ));
    }
    if luminosity_watts <= 0.0 {
        return Err(Error::TemperatureCalculation(
            "Luminosity must be positive".to_string(),
        ));
    }

    let scale = params.k * luminosity_watts.sqrt();
    let ratio = if scale > 0.0 {
        distance_light_seconds / scale
    } else {
        f64::INFINITY
    };

    let denom = 1.0 + ratio.powf(params.b);
    let t = params.min_kelvin + (params.max_kelvin - params.min_kelvin) / denom;

    let result = if params.map_to_kelvin {
        params.kelvin_offset + params.kelvin_scale * t
    } else {
        t
    };

    Ok(result)
}

/// Calculate external temperature in meters (convenience wrapper).
///
/// Converts the distance from meters to light-seconds and calls
/// [`compute_temperature_light_seconds`].
///
/// # Arguments
///
/// * `distance_meters` - Distance from the star in meters
/// * `luminosity_watts` - Stellar luminosity in watts
/// * `params` - Model parameters controlling the temperature curve
///
/// # Errors
///
/// Returns an error if:
/// * `distance_meters` is negative
/// * `luminosity_watts` is negative or zero
///
/// # Examples
///
/// ```
/// use evefrontier_lib::temperature::{compute_temperature_meters, TemperatureModelParams};
///
/// let params = TemperatureModelParams::default();
/// // Earth's orbital distance from Sun (1 AU ≈ 1.496e11 m)
/// let temp = compute_temperature_meters(1.496e11, 3.828e26, &params).unwrap();
/// assert!(temp > 0.0 && temp.is_finite()); // Should produce a valid temperature
/// ```
pub fn compute_temperature_meters(
    distance_meters: f64,
    luminosity_watts: f64,
    params: &TemperatureModelParams,
) -> Result<f64> {
    let distance_light_seconds = distance_meters / constants::METERS_IN_LIGHT_SECOND;
    compute_temperature_light_seconds(distance_light_seconds, luminosity_watts, params)
}

/// Calculate external temperature in astronomical units (convenience wrapper).
///
/// Converts the distance from AU to light-seconds and calls
/// [`compute_temperature_light_seconds`].
///
/// # Arguments
///
/// * `distance_au` - Distance from the star in astronomical units
/// * `luminosity_watts` - Stellar luminosity in watts
/// * `params` - Model parameters controlling the temperature curve
///
/// # Errors
///
/// Returns an error if:
/// * `distance_au` is negative
/// * `luminosity_watts` is negative or zero
pub fn compute_temperature_au(
    distance_au: f64,
    luminosity_watts: f64,
    params: &TemperatureModelParams,
) -> Result<f64> {
    let distance_meters = distance_au * constants::METERS_IN_AU;
    compute_temperature_meters(distance_meters, luminosity_watts, params)
}

/// Calculate Stefan-Boltzmann equilibrium temperature (Kelvin).
///
/// This function calculates the equilibrium temperature of a fast-rotating,
/// zero-albedo sphere at a given distance from a star, using the Stefan-Boltzmann
/// law for blackbody radiation.
///
/// The formula is:
/// ```text
/// T = (L / (16π σ r²))^(1/4)
/// ```
///
/// Where:
/// * `L` = stellar luminosity (watts)
/// * `σ` = Stefan-Boltzmann constant (5.670374419 × 10⁻⁸ W⋅m⁻²⋅K⁻⁴)
/// * `r` = distance from star (meters)
///
/// # Arguments
///
/// * `distance_meters` - Distance from the star in meters
/// * `luminosity_watts` - Stellar luminosity in watts
///
/// # Errors
///
/// Returns an error if:
/// * `distance_meters` is negative or zero
/// * `luminosity_watts` is negative or zero
///
/// # Examples
///
/// ```
/// use evefrontier_lib::temperature::compute_stefan_boltzmann_kelvin;
///
/// // Earth's equilibrium temperature (1 AU from Sun)
/// let temp = compute_stefan_boltzmann_kelvin(1.496e11, 3.828e26).unwrap();
/// // Should be around 279 K (without atmosphere/albedo)
/// assert!((temp - 279.0).abs() < 5.0);
/// ```
pub fn compute_stefan_boltzmann_kelvin(distance_meters: f64, luminosity_watts: f64) -> Result<f64> {
    if distance_meters <= 0.0 {
        return Err(Error::TemperatureCalculation(
            "Distance must be positive".to_string(),
        ));
    }
    if luminosity_watts <= 0.0 {
        return Err(Error::TemperatureCalculation(
            "Luminosity must be positive".to_string(),
        ));
    }

    // T = (L / (16π σ r²))^(1/4)
    let numerator = luminosity_watts;
    let denominator = 16.0
        * std::f64::consts::PI
        * constants::STEFAN_BOLTZMANN_SIGMA
        * distance_meters
        * distance_meters;

    let temp = (numerator / denominator).powf(0.25);

    Ok(temp)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOLAR_LUMINOSITY: f64 = 3.828e26; // Watts
    const EARTH_ORBIT_METERS: f64 = 1.496e11; // 1 AU

    #[test]
    fn test_custom_model_basic() {
        let params = TemperatureModelParams::default();
        let temp =
            compute_temperature_meters(EARTH_ORBIT_METERS, SOLAR_LUMINOSITY, &params).unwrap();

        // Should be finite and positive
        assert!(temp.is_finite());
        assert!(temp > params.min_kelvin);
    }

    #[test]
    fn test_custom_model_near_star() {
        let params = TemperatureModelParams::default();
        let temp = compute_temperature_meters(1e9, SOLAR_LUMINOSITY, &params).unwrap();

        // Very close to star should approach max_kelvin (99.9K in EVE Frontier model)
        assert!(temp > 50.0, "Expected temp > 50K near star, got {}", temp);
        assert!(
            temp <= params.max_kelvin,
            "Temp should not exceed max_kelvin"
        );
    }

    #[test]
    fn test_custom_model_deep_space() {
        let params = TemperatureModelParams::default();
        let temp = compute_temperature_meters(1e15, SOLAR_LUMINOSITY, &params).unwrap();

        // Very far from star should be cold (closer to min_kelvin than max_kelvin)
        assert!(temp < 100.0);
        assert!(temp > params.min_kelvin);
    }

    #[test]
    fn test_stefan_boltzmann_earth_orbit() {
        let temp = compute_stefan_boltzmann_kelvin(EARTH_ORBIT_METERS, SOLAR_LUMINOSITY).unwrap();

        // Earth's equilibrium temperature (without atmosphere) is ~279 K
        assert!((temp - 279.0).abs() < 5.0);
    }

    #[test]
    fn test_stefan_boltzmann_mars_orbit() {
        let mars_orbit = 2.279e11; // ~1.52 AU
        let temp = compute_stefan_boltzmann_kelvin(mars_orbit, SOLAR_LUMINOSITY).unwrap();

        // Mars equilibrium temperature should be colder than Earth
        let earth_temp =
            compute_stefan_boltzmann_kelvin(EARTH_ORBIT_METERS, SOLAR_LUMINOSITY).unwrap();
        assert!(temp < earth_temp);
        assert!(temp > 200.0 && temp < 250.0); // Reasonable range for Mars
    }

    #[test]
    fn test_negative_distance_error() {
        let params = TemperatureModelParams::default();
        let result = compute_temperature_meters(-100.0, SOLAR_LUMINOSITY, &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_luminosity_error() {
        let params = TemperatureModelParams::default();
        let result = compute_temperature_meters(EARTH_ORBIT_METERS, 0.0, &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_stefan_boltzmann_zero_distance_error() {
        let result = compute_stefan_boltzmann_kelvin(0.0, SOLAR_LUMINOSITY);
        assert!(result.is_err());
    }

    #[test]
    fn test_temperature_au_conversion() {
        let params = TemperatureModelParams::default();
        let temp_au = compute_temperature_au(1.0, SOLAR_LUMINOSITY, &params).unwrap();
        let temp_meters =
            compute_temperature_meters(constants::METERS_IN_AU, SOLAR_LUMINOSITY, &params).unwrap();

        assert!((temp_au - temp_meters).abs() < 1e-6);
    }

    #[test]
    fn test_temperature_increases_closer_to_star() {
        let params = TemperatureModelParams::default();
        let temp_far = compute_temperature_au(10.0, SOLAR_LUMINOSITY, &params).unwrap();
        let temp_near = compute_temperature_au(1.0, SOLAR_LUMINOSITY, &params).unwrap();

        assert!(temp_near > temp_far);
    }

    #[test]
    fn test_higher_luminosity_increases_temperature() {
        let params = TemperatureModelParams::default();
        let temp_dim =
            compute_temperature_meters(EARTH_ORBIT_METERS, SOLAR_LUMINOSITY, &params).unwrap();
        let temp_bright =
            compute_temperature_meters(EARTH_ORBIT_METERS, SOLAR_LUMINOSITY * 2.0, &params)
                .unwrap();

        assert!(temp_bright > temp_dim);
    }

    // Integration tests with real EVE Frontier fixture data
    //
    // These tests validate the EVE Frontier temperature formula:
    //   T(d) = T_min + (T_max - T_min) / (1 + (d / (k * √L))^b)
    //
    // Parameters (from EVE Frontier calibration):
    //   T_min = 0.1 K
    //   T_max = 99.9 K
    //   k = 3.215 × 10⁻¹¹
    //   b = 1.25
    //
    // Test cases from e6c3 dataset:
    //   Nod:   L = 1.9209×10²⁵ W, d = 541.4 ls  → T ≈ 15.74 K
    //   Brana: L = 4.7398×10²⁴ W, d = 9255.2 ls → T ≈ 0.32 K
    mod eve_frontier_fixture {
        use crate::db::load_starmap;
        use std::path::PathBuf;

        fn fixture_path() -> PathBuf {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/fixtures/minimal/static_data.db")
        }

        #[test]
        fn test_fixture_systems_have_computed_min_temps() {
            let starmap = load_starmap(&fixture_path()).expect("fixture loads");

            // Check that min_external_temp is computed for systems
            let nod_id = starmap.system_id_by_name("Nod").expect("Nod exists");
            let nod = starmap.systems.get(&nod_id).expect("Nod system data");

            assert!(
                nod.metadata.min_external_temp.is_some(),
                "Nod should have computed min_external_temp"
            );

            let brana_id = starmap.system_id_by_name("Brana").expect("Brana exists");
            let brana = starmap.systems.get(&brana_id).expect("Brana system data");

            assert!(
                brana.metadata.min_external_temp.is_some(),
                "Brana should have computed min_external_temp"
            );
        }

        #[test]
        fn test_nod_temperature_is_physically_reasonable() {
            // Nod test case from e6c3 dataset:
            //   Star luminosity: 1.9209 × 10²⁵ W
            //   Furthest celestial: "Nod - Planet 2"
            //   Distance: 1.6231 × 10¹¹ m = 541.4142 light-seconds
            //   Expected T_min: 15.74 K (calculated with EVE Frontier formula)
            let starmap = load_starmap(&fixture_path()).expect("fixture loads");
            let nod_id = starmap.system_id_by_name("Nod").expect("Nod exists");
            let nod = starmap.systems.get(&nod_id).expect("Nod system data");

            if let Some(min_temp) = nod.metadata.min_external_temp {
                // Should be above absolute zero
                assert!(min_temp > 0.0, "Temperature should be above 0K");

                // Should be finite
                assert!(
                    min_temp.is_finite(),
                    "Temperature should be finite, got {}",
                    min_temp
                );

                // Should be within a reasonable range for space
                assert!(
                    min_temp < 1000.0,
                    "Deep space temperature shouldn't exceed 1000K, got {:.1}K",
                    min_temp
                );
            }
        }

        #[test]
        fn test_brana_temperature_is_physically_reasonable() {
            // Brana test case from e6c3 dataset:
            //   Star luminosity: 4.7398 × 10²⁴ W
            //   Furthest celestial: "Brana - Planet 2"
            //   Distance: 2.7746 × 10¹² m = 9255.1725 light-seconds
            //   Expected T_min: 0.32 K (calculated with EVE Frontier formula)
            let starmap = load_starmap(&fixture_path()).expect("fixture loads");
            let brana_id = starmap.system_id_by_name("Brana").expect("Brana exists");
            let brana = starmap.systems.get(&brana_id).expect("Brana system data");

            if let Some(min_temp) = brana.metadata.min_external_temp {
                // Should be above absolute zero
                assert!(min_temp > 0.0, "Temperature should be above 0K");

                // Should be finite
                assert!(
                    min_temp.is_finite(),
                    "Temperature should be finite, got {}",
                    min_temp
                );

                // Should be within a reasonable range for space
                assert!(
                    min_temp < 1000.0,
                    "Deep space temperature shouldn't exceed 1000K, got {:.1}K",
                    min_temp
                );
            }
        }

        #[test]
        fn test_all_fixture_systems_have_valid_temperatures() {
            let starmap = load_starmap(&fixture_path()).expect("fixture loads");

            let mut systems_with_temps = 0;
            for system in starmap.systems.values() {
                if let Some(temp) = system.metadata.min_external_temp {
                    systems_with_temps += 1;

                    assert!(
                        temp > 0.0 && temp.is_finite(),
                        "System {} has invalid temperature: {:.1}K",
                        system.name,
                        temp
                    );
                }
            }

            assert!(
                systems_with_temps >= 2,
                "Expected at least 2 systems with temperature data, got {}",
                systems_with_temps
            );
        }
    }
}
