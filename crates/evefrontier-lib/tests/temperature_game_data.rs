//! Temperature model validation against actual EVE Frontier game measurements.
//!
//! This test suite validates both temperature calculation models (logistic curve and
//! inverse-tangent) against real measurements taken in-game on 2026-02-03.
//!
//! Test data collected by warping to different distances from stars and recording
//! ambient temperature readings at each location.

use evefrontier_lib::temperature::{
    compute_temperature_light_seconds, TemperatureMethod, TemperatureModelParams,
};

/// Game measurement data: (system_name, distance_ls, luminosity_watts, measured_temp_k)
const GAME_MEASUREMENTS: &[(&str, f64, f64, f64)] = &[
    ("A9R-PQ4", 1168.0, 117079286879216367304704.00, 0.6),
    ("O86-215", 3078.0, 210611439263697778669256704.00, 0.0),
    ("E37-N15", 2735.0, 46609614223656827010154496.00, 5.1),
    ("UVG-MV3", 6283.0, 2399495653136391448790302720.00, 15.7),
    ("U98-VK4", 20850.0, 95197165897008878019531505664.00, 28.4),
    ("EKF-2N4", 1297.0, 3482357323030063141093376.00, 2.9),
    ("ER7-MN4", 165.0, 5945032820976178790662144.00, 28.3),
    ("E6S-8S4", 3290.0, 526627990081004722192384.00, 0.4),
    ("O43-CT4", 532.0, 269067787727477819523989504.00, 49.9),
    ("E17-S05", 17.0, 705658793434329632473088.00, 63.3),
];

#[test]
fn test_logistic_curve_against_game_data() {
    let params = TemperatureModelParams {
        method: TemperatureMethod::LogisticCurve,
        ..Default::default()
    };

    let mut errors = Vec::new();
    let mut max_error = 0.0;
    let mut max_error_system = "";

    for (system, distance_ls, luminosity, measured_temp) in GAME_MEASUREMENTS {
        let calculated = compute_temperature_light_seconds(*distance_ls, *luminosity, &params)
            .expect("temperature calculation should succeed");

        let error = (calculated - measured_temp).abs();
        let error_pct = if *measured_temp > 0.0 {
            (error / measured_temp) * 100.0
        } else {
            0.0
        };

        errors.push(error);

        if error > max_error {
            max_error = error;
            max_error_system = system;
        }

        // Most systems should be within 5% error (logistic curve achieves ~2-5% MAE)
        // We allow up to 35% for edge cases (very cold systems near detection limits)
        assert!(
            error_pct < 35.0,
            "System {} error too high: {:.2}% (calculated: {:.4}K, measured: {:.1}K)",
            system,
            error_pct,
            calculated,
            measured_temp
        );
    }

    let mean_error = errors.iter().sum::<f64>() / errors.len() as f64;

    // Validate overall model performance
    assert!(
        mean_error < 3.0,
        "Mean absolute error too high: {:.4}K",
        mean_error
    );
    assert!(
        max_error < 10.0,
        "Max error too high: {:.4}K in system {}",
        max_error,
        max_error_system
    );

    println!("Logistic Curve Model Performance:");
    println!("  Mean Absolute Error: {:.4} K", mean_error);
    println!(
        "  Max Absolute Error: {:.4} K ({})",
        max_error, max_error_system
    );
}

#[test]
fn test_inverse_tangent_model_documentation() {
    let params = TemperatureModelParams {
        method: TemperatureMethod::InverseTangent,
        ..Default::default()
    };

    // The inverse-tangent model uses a flux-based formula that achieves validated performance
    // matching the logistic curve. This test validates the model against actual game data.
    let mut errors = Vec::new();

    for (system, distance_ls, luminosity, measured_temp) in GAME_MEASUREMENTS {
        let calculated = compute_temperature_light_seconds(*distance_ls, *luminosity, &params)
            .expect("temperature calculation should succeed");

        let error = (calculated - measured_temp).abs();
        errors.push((system, error, measured_temp, calculated));
    }

    let mean_error = errors.iter().map(|(_, e, _, _)| e).sum::<f64>() / errors.len() as f64;

    // Flux-based inverse-tangent model is validated and matches logistic curve performance
    // Formula: T = (200/π) · atan(√(L / (d² × 10²¹)))
    // Expected MAE: ~1.2K (< 2% average error)
    assert!(
        mean_error < 3.0,
        "Expected inverse-tangent MAE < 3K (validated model), got {:.4}K",
        mean_error
    );

    // Should achieve <5% error on most systems
    let low_error_count = errors
        .iter()
        .filter(|(_, err, measured, _)| {
            let pct_err = if **measured > 0.1 {
                (*err / *measured) * 100.0
            } else {
                0.0
            };
            pct_err < 5.0
        })
        .count();

    assert!(
        low_error_count >= 8,
        "Expected at least 8/10 systems with <5% error, got {}/10",
        low_error_count
    );

    println!("Flux-Based Inverse-Tangent Model (VALIDATED - Default Method):");
    println!("  Mean Absolute Error: {:.4} K", mean_error);
    println!("  Formula: T = (200/π) · atan(√(L / (d² × 10²¹)))");
    println!(
        "  Matches heatsense.pages.dev performance. Physically meaningful (uses radiative flux)."
    );
}

#[test]
fn test_specific_system_o43_ct4() {
    // O43-CT4: High-luminosity system with measured temperature of 49.9K
    let params = TemperatureModelParams {
        method: TemperatureMethod::LogisticCurve,
        ..Default::default()
    };

    let temp = compute_temperature_light_seconds(532.0, 269067787727477819523989504.00, &params)
        .expect("temperature calculation should succeed");

    // Should be within 1K of measured value
    assert!(
        (temp - 49.9).abs() < 1.0,
        "O43-CT4 temperature mismatch: calculated {:.2}K, measured 49.9K",
        temp
    );
}

#[test]
fn test_specific_system_er7_mn4() {
    // ER7-MN4: Very close to star (165 LS) with measured temperature of 28.3K
    let params = TemperatureModelParams {
        method: TemperatureMethod::LogisticCurve,
        ..Default::default()
    };

    let temp = compute_temperature_light_seconds(165.0, 5945032820976178790662144.00, &params)
        .expect("temperature calculation should succeed");

    // Should be within 0.5K of measured value
    assert!(
        (temp - 28.3).abs() < 0.5,
        "ER7-MN4 temperature mismatch: calculated {:.2}K, measured 28.3K",
        temp
    );
}

#[test]
fn test_specific_system_e17_s05() {
    // E17-S05: Extremely close to star (17 LS) with highest measured temperature (63.3K)
    let params = TemperatureModelParams {
        method: TemperatureMethod::LogisticCurve,
        ..Default::default()
    };

    let temp = compute_temperature_light_seconds(17.0, 705658793434329632473088.00, &params)
        .expect("temperature calculation should succeed");

    // Should be within 2K of measured value
    assert!(
        (temp - 63.3).abs() < 2.0,
        "E17-S05 temperature mismatch: calculated {:.2}K, measured 63.3K",
        temp
    );
}

#[test]
fn test_cold_system_e6s_8s4() {
    // E6S-8S4: Very cold system at outer edge, measured 0.4K
    let params = TemperatureModelParams {
        method: TemperatureMethod::LogisticCurve,
        ..Default::default()
    };

    let temp = compute_temperature_light_seconds(3290.0, 526627990081004722192384.00, &params)
        .expect("temperature calculation should succeed");

    // Should be within 0.2K of measured value
    assert!(
        (temp - 0.4).abs() < 0.2,
        "E6S-8S4 temperature mismatch: calculated {:.2}K, measured 0.4K",
        temp
    );
}

#[test]
fn test_model_comparison() {
    // Compare both models across all measurements
    let logistic_params = TemperatureModelParams {
        method: TemperatureMethod::LogisticCurve,
        ..Default::default()
    };

    let inverse_params = TemperatureModelParams {
        method: TemperatureMethod::InverseTangent,
        ..Default::default()
    };

    for (system, distance_ls, luminosity, measured_temp) in GAME_MEASUREMENTS {
        let logistic_temp =
            compute_temperature_light_seconds(*distance_ls, *luminosity, &logistic_params)
                .expect("logistic calculation should succeed");

        let inverse_temp =
            compute_temperature_light_seconds(*distance_ls, *luminosity, &inverse_params)
                .expect("inverse-tangent calculation should succeed");

        let logistic_error = (logistic_temp - measured_temp).abs();
        let inverse_error = (inverse_temp - measured_temp).abs();

        // Both models should achieve similar accuracy (<3K error) for most systems
        if *measured_temp > 1.0 {
            // Skip very cold systems where both models may struggle
            assert!(
                logistic_error < 5.0 && inverse_error < 5.0,
                "System {}: Both models should be accurate (logistic: {:.4}K error, inverse: {:.4}K error)",
                system,
                logistic_error,
                inverse_error
            );
        }
    }
}
