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

#[test]
#[ignore]
fn observed_h123k_planet6_temperature_with_alternate_k_matches_28_1k() {
    // Arrange: ensure dataset present
    let db_path = ensure_c3e6_dataset(None).expect("dataset available");

    // Query dataset for luminosity and distance of H:123K - Planet 6
    let (luminosity, distance_m) = query_h123k_planet6(&db_path).expect("query H:123K - Planet 6");

    // Given the in-game observation: ~28.1 K at this location
    let observed_temp = 28.1_f64;

    // Default model parameters
    let mut params = TemperatureModelParams::default();

    // Verify current model yields ~0.1 K (documents the discrepancy)
    let t_default =
        compute_temperature_meters(distance_m, luminosity, &params).expect("compute default");
    assert!(
        t_default <= params.min_kelvin + 0.02,
        "expected ~0.1K with default model, got {:.3}K",
        t_default
    );

    // Solve for k that would produce the observed temperature with current b/T_min/T_max
    let k_needed = solve_k_for_observation(
        distance_m,
        luminosity,
        params.k,
        params.b,
        params.min_kelvin,
        params.max_kelvin,
        observed_temp,
    );

    // Use the derived k and recompute
    params.k = k_needed;
    let t_alt = compute_temperature_meters(distance_m, luminosity, &params).expect("compute alt");

    // Assert we match the observation within tolerance
    assert!(
        (t_alt - observed_temp).abs() < 0.2,
        "alt-k mismatch: got {:.2}K, want {:.2}K (k_needed={:.6e})",
        t_alt,
        observed_temp,
        k_needed
    );
}
