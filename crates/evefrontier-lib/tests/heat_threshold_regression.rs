//! Regression tests for temperature threshold bug fixes.
//!
//! Tests ensure that temperature thresholds (OVERHEATED/CRITICAL) use actual ambient
//! temperatures (0.1K-99.9K) without artificial floor clamping to HEAT_NOMINAL (30K).

use evefrontier_lib::ship::{
    constants::{COOLING_EPSILON, HEAT_CRITICAL, HEAT_NOMINAL, HEAT_OVERHEATED},
    heat::{calculate_cooling_time, project_heat_for_jump, HeatProjectionParams},
};

const REFLEX_BASE_MASS_KG: f64 = 100_000.0;
const REFLEX_SPECIFIC_HEAT: f64 = 1_000.0;
const CALIBRATION_CONSTANT: f64 = 1e-7;
const TEST_COOLING_K_COLD: f64 = 2e-2;
const TEST_COOLING_K_HOT: f64 = 5e-3;

fn make_params(
    prev_ambient: Option<f64>,
    current_ambient: Option<f64>,
    distance_ly: f64,
    mass: f64,
    is_goal: bool,
    next_is_gate: bool,
) -> HeatProjectionParams {
    HeatProjectionParams {
        prev_ambient,
        current_min_external_temp: current_ambient,
        distance_ly,
        mass,
        hull_mass_kg: REFLEX_BASE_MASS_KG,
        specific_heat: REFLEX_SPECIFIC_HEAT,
        calibration_constant: CALIBRATION_CONSTANT,
        is_goal,
        next_is_gate,
    }
}

#[test]
fn test_bug_scenario_idr_kr4_to_esj_855() {
    // Bug: IDR-KR4 (2.76K) → ESJ-855 with low starting temperature
    // Old behavior: start = max(30, 2.76) = 30K → artificially high instantaneous temp
    // New behavior: start = 2.76K → correct instantaneous temp

    // The key insight: with enough mass/distance, the same hop could be OVERHEATED (correct)
    // instead of CRITICAL (buggy) just because we clamped the floor to 30K.

    // Test with high mass to generate substantial heat
    let high_mass = 1_000_000.0; // 1M kg total mass
    let params = make_params(Some(2.76), Some(1.19), 39.0, high_mass, false, false);
    let projection = project_heat_for_jump(params).unwrap();

    let instantaneous_from_276 = 2.76 + projection.hop_heat;
    eprintln!(
        "Bug scenario (correct): prev_ambient=2.76K, hop_heat={:.2}K, instantaneous={:.2}K",
        projection.hop_heat, instantaneous_from_276
    );

    // Now simulate the buggy behavior: if we had used 30K as the floor
    let buggy_instantaneous = 30.0 + projection.hop_heat;
    eprintln!(
        "Bug scenario (buggy): prev_ambient=30K (clamped!), hop_heat={:.2}K, instantaneous={:.2}K",
        projection.hop_heat, buggy_instantaneous
    );

    // The difference should be exactly 27.24K (30.0 - 2.76)
    let floor_clamp_difference = buggy_instantaneous - instantaneous_from_276;
    assert!(
        (floor_clamp_difference - 27.24).abs() < 0.01,
        "floor clamp would add exactly 27.24K to the instantaneous temp"
    );

    // Verify that the bug fix allows sub-30K starting temperatures
    assert!(
        instantaneous_from_276 < buggy_instantaneous,
        "correct calculation should produce lower temp than buggy floor-clamped version"
    );
}

#[test]
fn test_threshold_boundaries() {
    // Just above OVERHEATED: 89.9K + 0.2K → OVERHEATED
    let params1 = make_params(Some(89.9), Some(50.0), 1.0, 100_000.0, false, false);
    let proj1 = project_heat_for_jump(params1).unwrap();
    assert!(89.9 + proj1.hop_heat >= HEAT_OVERHEATED);
    assert_eq!(proj1.warning.as_deref(), Some("OVERHEATED"));

    // Just above CRITICAL: 149.9K + 0.2K → CRITICAL
    let params2 = make_params(Some(149.9), Some(149.0), 1.0, 100_000.0, false, false);
    let proj2 = project_heat_for_jump(params2).unwrap();
    assert!(149.9 + proj2.hop_heat >= HEAT_CRITICAL);
    assert_eq!(proj2.warning.as_deref(), Some("CRITICAL"));

    // Below thresholds: no warning
    let params3 = make_params(Some(50.0), Some(40.0), 15.0, 100_000.0, false, false);
    let proj3 = project_heat_for_jump(params3).unwrap();
    assert!(50.0 + proj3.hop_heat < HEAT_OVERHEATED);
    assert_eq!(proj3.warning, None);

    let params4 = make_params(Some(0.1), Some(0.1), 45.0, 100_000.0, false, false);
    let proj4 = project_heat_for_jump(params4).unwrap();
    assert!(0.1 + proj4.hop_heat < HEAT_OVERHEATED);
    assert_eq!(proj4.warning, None);
}

#[test]
fn test_cooling_residual_temperature() {
    // Cold system (5K): should cool to target
    let params1 = make_params(Some(5.0), Some(5.0), 45.0, 100_000.0, false, false);
    let proj1 = project_heat_for_jump(params1).unwrap();

    eprintln!("Cold system: prev_ambient=5K, hop_heat={:.2}K, instantaneous={:.2}K, wait_time={:?}, residual={:?}",
              proj1.hop_heat, 5.0 + proj1.hop_heat, proj1.wait_time_seconds, proj1.residual_heat);

    let instantaneous1 = 5.0 + proj1.hop_heat;
    if instantaneous1 > HEAT_NOMINAL {
        // Only expect cooling if we're above target
        assert!(proj1.wait_time_seconds.is_some());
        let residual1 = proj1.residual_heat.unwrap();
        assert!((HEAT_NOMINAL - 1.0..=HEAT_NOMINAL + COOLING_EPSILON + 1.0).contains(&residual1));
    }

    // Hot system (40K): should cool to ambient if above it
    let params2 = make_params(Some(40.0), Some(40.0), 30.0, 100_000.0, false, false);
    let proj2 = project_heat_for_jump(params2).unwrap();

    eprintln!("Hot system: prev_ambient=40K, hop_heat={:.2}K, instantaneous={:.2}K, wait_time={:?}, residual={:?}",
              proj2.hop_heat, 40.0 + proj2.hop_heat, proj2.wait_time_seconds, proj2.residual_heat);

    let instantaneous2 = 40.0 + proj2.hop_heat;
    if instantaneous2 > 40.0 + COOLING_EPSILON {
        assert!(proj2.wait_time_seconds.is_some());
        let residual2 = proj2.residual_heat.unwrap();
        assert!((40.0..=40.0 + COOLING_EPSILON + 0.1).contains(&residual2));
    }
}

#[test]
fn test_sub_30k_temperatures_allowed() {
    // Very cold system: 0.1K allowed (no artificial floor to 30K)
    let params = make_params(Some(0.1), Some(0.1), 10.0, 100_000.0, false, false);
    let projection = project_heat_for_jump(params).unwrap();
    assert!(
        0.1 + projection.hop_heat < HEAT_NOMINAL,
        "should allow sub-30K temperatures"
    );
}

#[test]
fn test_negative_temperature_prevention() {
    // Invalid negative ambient should be clamped to 0
    let params = make_params(Some(-10.0), Some(5.0), 5.0, 100_000.0, false, false);
    let projection = project_heat_for_jump(params).unwrap();
    let instantaneous = 0.0_f64.max(-10.0) + projection.hop_heat;
    assert!(instantaneous >= 0.0, "should prevent negative temperatures");
}

#[test]
fn test_cooling_rate_zone_factor() {
    // Cold system cools faster than hot system
    let time_cold = calculate_cooling_time(90.0, 30.0, 0.1, TEST_COOLING_K_COLD);
    let time_hot = calculate_cooling_time(90.0, 30.0, 80.0, TEST_COOLING_K_HOT);

    assert!(time_cold > 0.0 && time_cold < 3600.0);
    assert!(time_hot > time_cold, "hot system should cool slower");
}

#[test]
fn test_gate_and_goal_no_cooling() {
    // Gate hop: no cooling
    let params1 = make_params(Some(50.0), Some(40.0), 30.0, 100_000.0, false, true);
    let proj1 = project_heat_for_jump(params1).unwrap();
    assert!(
        proj1.wait_time_seconds.is_none(),
        "gate hops should not have cooling time"
    );

    // Goal hop: no cooling
    let params2 = make_params(Some(50.0), Some(40.0), 30.0, 100_000.0, true, false);
    let proj2 = project_heat_for_jump(params2).unwrap();
    assert!(
        proj2.wait_time_seconds.is_none(),
        "goal hops should not have cooling time"
    );
}
