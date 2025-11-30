//! Integration tests for temperature model validation against observed in-game data.
#![allow(dead_code, unused_imports)]
//!
//! These tests require the full e6c3 dataset and are gated behind the `integration` feature
//! to avoid running during normal `cargo test`. Run with:
//!
//! ```bash
//! cargo test -p evefrontier-lib --test temperature_observed -- --include-ignored
//! ```
//!
//! Or enable the integration tests feature (if added).

use evefrontier_lib::dataset::ensure_c3e6_dataset;
use evefrontier_lib::temperature::{compute_temperature_meters, TemperatureModelParams};
use rusqlite::{params, Connection};

fn query_h123k_planet6(db_path: &std::path::Path) -> rusqlite::Result<(f64, f64)> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(
        "SELECT s.star_luminosity, p.centerX, p.centerY, p.centerZ \
         FROM Planets p \
         JOIN SolarSystems s ON p.solarSystemId = s.solarSystemId \
         WHERE s.name = ?1 AND p.name = ?2",
    )?;

    let row = stmt.query_row(params!["H:123K", "H:123K - Planet 6"], |r| {
        let lum: f64 = r.get(0)?;
        let cx: f64 = r.get(1)?;
        let cy: f64 = r.get(2)?;
        let cz: f64 = r.get(3)?;
        let dist = (cx * cx + cy * cy + cz * cz).sqrt();
        Ok((lum, dist))
    })?;

    Ok(row)
}

fn solve_k_for_observation(
    distance_m: f64,
    luminosity_watts: f64,
    _observed_k: f64,
    observed_b: f64,
    t_min: f64,
    t_max: f64,
    observed_temp: f64,
) -> f64 {
    // Invert: T = T_min + (T_max - T_min) / (1 + (d/(k*sqrt(L)))^b)
    // k = d / ( sqrt(L) * ( ((T_max-T_min)/(T_obs-T_min) - 1))^(1/b) )
    let sqrt_l = luminosity_watts.sqrt();
    let inner = (t_max - t_min) / (observed_temp - t_min) - 1.0;
    let ratio = inner.powf(1.0 / observed_b);
    distance_m / (sqrt_l * ratio)
}
