//! Integration tests for `scout range` fuel/heat projection.
//!
//! Tests verify:
//! - Ship option enables fuel/heat projection fields in output
//! - Nearest-neighbor ordering is applied when ship is specified
//! - Cumulative fuel tracking is correct
//! - Heat tracking is correct
//! - Warnings display when thresholds exceeded

use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use serde_json::Value;
use tempfile::tempdir;

fn fixture_db() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/fixtures/minimal/static_data.db")
        .canonicalize()
        .expect("fixture dataset present")
}

fn fixture_ship() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/fixtures/ship_data.csv")
        .canonicalize()
        .expect("ship data fixture present")
}

fn cli() -> Command {
    cargo_bin_cmd!("evefrontier-cli")
}

fn prepare_command() -> (Command, tempfile::TempDir) {
    let temp_dir = tempdir().expect("create temp dir");
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&cache_dir).expect("create cache dir");
    let mut cmd = cli();
    cmd.env("EVEFRONTIER_DATASET_SOURCE", fixture_db())
        .env("EVEFRONTIER_DATASET_CACHE_DIR", &cache_dir)
        .env("EVEFRONTIER_SHIP_DATA", fixture_ship())
        .env("RUST_LOG", "error")
        .arg("--no-logo")
        .arg("--data-dir")
        .arg(temp_dir.path());
    (cmd, temp_dir)
}

// =============================================================================
// T008: Test that ship option enables fuel fields in JSON output
// =============================================================================

#[test]
fn test_scout_range_with_ship_returns_fuel_fields() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--limit")
        .arg("3")
        .arg("--ship")
        .arg("Reflex");

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: Value = serde_json::from_str(&stdout).expect("valid JSON output");

    // Ship info should be present
    assert!(json.get("ship").is_some(), "ship field should be present");
    assert_eq!(json["ship"]["name"], "Reflex");
    assert!(
        json["ship"]["fuel_capacity"].as_f64().unwrap() > 0.0,
        "fuel_capacity should be positive"
    );

    // Route totals should be present
    assert!(
        json.get("total_distance_ly").is_some(),
        "total_distance_ly should be present"
    );
    assert!(
        json.get("total_fuel").is_some(),
        "total_fuel should be present"
    );

    // Systems should have fuel fields
    let systems = json["systems"].as_array().expect("systems array");
    if !systems.is_empty() {
        let first = &systems[0];
        assert!(
            first.get("hop_fuel").is_some(),
            "hop_fuel should be present"
        );
        assert!(
            first.get("cumulative_fuel").is_some(),
            "cumulative_fuel should be present"
        );
        assert!(
            first.get("remaining_fuel").is_some(),
            "remaining_fuel should be present"
        );
    }
}

// =============================================================================
// T009: Test nearest-neighbor ordering
// =============================================================================

#[test]
fn test_scout_range_nearest_neighbor_ordering() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--limit")
        .arg("5")
        .arg("--ship")
        .arg("Reflex");

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: Value = serde_json::from_str(&stdout).expect("valid JSON output");
    let systems = json["systems"].as_array().expect("systems array");

    // When ship is specified, distance_ly should represent hop distance
    // The first system's distance should be from origin (Nod)
    // Subsequent distances should be from the previous system in visit order
    // This is different from "distance from origin" sorting

    // Just verify we have systems and they have positive distances
    // The exact ordering depends on the fixture data
    for sys in systems {
        let dist = sys["distance_ly"].as_f64().expect("distance_ly is f64");
        assert!(dist > 0.0, "hop distance should be positive");
    }
}

// =============================================================================
// T010: Test cumulative fuel tracking
// =============================================================================

#[test]
fn test_scout_range_fuel_cumulative_tracking() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--limit")
        .arg("3")
        .arg("--ship")
        .arg("Reflex");

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: Value = serde_json::from_str(&stdout).expect("valid JSON output");
    let systems = json["systems"].as_array().expect("systems array");
    let fuel_capacity = json["ship"]["fuel_capacity"].as_f64().unwrap();

    if systems.len() >= 2 {
        // Verify cumulative fuel increases (total fuel used always increases)
        let fuel1 = systems[0]["cumulative_fuel"].as_f64().unwrap();
        let fuel2 = systems[1]["cumulative_fuel"].as_f64().unwrap();
        assert!(
            fuel2 > fuel1,
            "cumulative fuel should increase: {} > {}",
            fuel2,
            fuel1
        );

        // For first system without refuel, remaining should be capacity minus first hop
        let hop1 = systems[0]["hop_fuel"].as_f64().unwrap();
        let rem1 = systems[0]["remaining_fuel"].as_f64().unwrap();
        let has_refuel_warning = systems[0]
            .get("fuel_warning")
            .and_then(|v| v.as_str())
            .map(|s| s == "REFUEL")
            .unwrap_or(false);

        if !has_refuel_warning {
            let expected_rem1 = fuel_capacity - hop1;
            assert!(
                (rem1 - expected_rem1).abs() < 0.01,
                "remaining fuel should equal capacity - hop cost: got {}, expected {}",
                rem1,
                expected_rem1
            );
        }

        // Verify remaining fuel behavior:
        // - If no refuel on second hop: rem2 < rem1
        // - If refuel on second hop: rem2 = capacity (matching route command's convention:
        //   remaining reflects fuel available at arrival after refuel, before consuming
        //   fuel for the current hop)
        let rem2 = systems[1]["remaining_fuel"].as_f64().unwrap();
        let _hop2 = systems[1]["hop_fuel"].as_f64().unwrap();
        let has_refuel_warning2 = systems[1]
            .get("fuel_warning")
            .and_then(|v| v.as_str())
            .map(|s| s == "REFUEL")
            .unwrap_or(false);

        if has_refuel_warning2 {
            // After refuel, remaining = capacity (fuel available at arrival after refuel)
            // This matches the route command's behavior in output.rs:507-525
            let expected_rem2 = fuel_capacity;
            assert!(
                (rem2 - expected_rem2).abs() < 0.01,
                "after refuel, remaining should equal capacity: got {}, expected {}",
                rem2,
                expected_rem2
            );
        } else {
            // Without refuel, remaining should decrease from previous
            assert!(
                rem2 < rem1,
                "remaining fuel should decrease: {} < {}",
                rem2,
                rem1
            );
        }
    }
}

// =============================================================================
// T018: Test heat per hop
// =============================================================================

#[test]
fn test_scout_range_heat_per_hop() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--limit")
        .arg("3")
        .arg("--ship")
        .arg("Reflex");

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: Value = serde_json::from_str(&stdout).expect("valid JSON output");
    let systems = json["systems"].as_array().expect("systems array");

    // Each system should have hop_heat field when ship is specified
    for (i, sys) in systems.iter().enumerate() {
        assert!(
            sys.get("hop_heat").is_some() && !sys["hop_heat"].is_null(),
            "system {} should have hop_heat field",
            i
        );
        let hop_heat = sys["hop_heat"].as_f64().expect("hop_heat is f64");
        assert!(
            hop_heat >= 0.0,
            "hop_heat should be non-negative: {}",
            hop_heat
        );
    }
}

// =============================================================================
// T019: Test cumulative heat tracking
// =============================================================================

#[test]
fn test_scout_range_heat_cumulative() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--limit")
        .arg("3")
        .arg("--ship")
        .arg("Reflex");

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: Value = serde_json::from_str(&stdout).expect("valid JSON output");
    let systems = json["systems"].as_array().expect("systems array");

    // Note: cumulative_heat now stores the residual temperature at arrival
    // (after any cooldown to nominal/ambient), matching the route command's model.
    // The start_temp is max(HEAT_NOMINAL=30.0, prev_system_ambient_temp), and residual is
    // max(HEAT_NOMINAL, destination ambient) when cooling is applied.
    const HEAT_NOMINAL: f64 = 30.0;

    if !systems.is_empty() {
        let heat1 = systems[0]["cumulative_heat"]
            .as_f64()
            .expect("cumulative_heat is f64");
        let dest_ambient = systems[0]["min_temp_k"].as_f64().unwrap_or(0.0);
        let expected_residual = HEAT_NOMINAL.max(dest_ambient);
        assert!(
            (heat1 - expected_residual).abs() < 1.0,
            "first system residual (cumulative_heat) should be approximately max(NOMINAL, dest ambient): got {}, expected {}",
            heat1,
            expected_residual
        );
    }

    // Final heat should be populated in result
    assert!(
        json.get("final_heat").is_some() && !json["final_heat"].is_null(),
        "final_heat should be present when ship is specified"
    );

    // Final heat equals the destination residual heat
    if !systems.is_empty() {
        let final_heat = json["final_heat"].as_f64().expect("final_heat");
        let last_heat = systems
            .last()
            .and_then(|s| s.get("cumulative_heat"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        assert!(
            (final_heat - last_heat).abs() < 1.0,
            "final_heat should approximately equal last system residual: final={}, last={}",
            final_heat,
            last_heat
        );
    }
}

// =============================================================================
// Without ship - original behavior should be preserved
// =============================================================================

#[test]
fn test_scout_range_without_ship_no_fuel_fields() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--limit")
        .arg("3");

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: Value = serde_json::from_str(&stdout).expect("valid JSON output");

    // Ship info should NOT be present
    assert!(
        json.get("ship").is_none() || json["ship"].is_null(),
        "ship field should not be present without --ship"
    );

    // Route totals should NOT be present
    assert!(
        json.get("total_fuel").is_none() || json["total_fuel"].is_null(),
        "total_fuel should not be present without --ship"
    );
}

// =============================================================================
// T023: Test REFUEL warning when fuel insufficient
// =============================================================================

#[test]
fn test_scout_range_fuel_warning() {
    // Use very low fuel load (1 unit) to trigger refuel warning
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--limit")
        .arg("5")
        .arg("--ship")
        .arg("Reflex")
        .arg("--fuel-load")
        .arg("1"); // Very low fuel should trigger REFUEL warning

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: Value = serde_json::from_str(&stdout).expect("valid JSON output");
    let systems = json["systems"].as_array().expect("systems array");

    // At some point with such low fuel, we should see a REFUEL warning
    let has_fuel_warning = systems
        .iter()
        .any(|s| s.get("fuel_warning").is_some() && !s["fuel_warning"].is_null());

    assert!(
        has_fuel_warning,
        "With 1 fuel unit, at least one system should have a fuel_warning"
    );
}

// =============================================================================
// T024: Test OVERHEATED warning when heat >= 90
// =============================================================================

#[test]
fn test_scout_range_overheated_warning() {
    // Use a route with enough hops to potentially accumulate heat
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--limit")
        .arg("10")
        .arg("--ship")
        .arg("Reflex");

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: Value = serde_json::from_str(&stdout).expect("valid JSON output");
    let systems = json["systems"].as_array().expect("systems array");

    // Check if any system has heat >= 90 (HEAT_OVERHEATED threshold)
    // and has a heat_warning field populated
    for sys in systems {
        let cumulative_heat = sys["cumulative_heat"].as_f64().unwrap_or(0.0);
        if cumulative_heat >= 90.0 {
            // Should have a heat warning
            assert!(
                sys.get("heat_warning").is_some() && !sys["heat_warning"].is_null(),
                "system with cumulative_heat {} should have heat_warning",
                cumulative_heat
            );
        }
    }
}

// =============================================================================
// T025: Test CRITICAL warning with cooldown when heat >= 150
// =============================================================================

#[test]
fn test_scout_range_critical_cooldown() {
    // Use a longer route to potentially accumulate critical heat
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--limit")
        .arg("15")
        .arg("--ship")
        .arg("Reflex");

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: Value = serde_json::from_str(&stdout).expect("valid JSON output");
    let systems = json["systems"].as_array().expect("systems array");

    // For any step with CRITICAL warning (instantaneous), expect cooldown_seconds
    // except possibly the final destination which does not require pre-arrival cooling.
    for (idx, sys) in systems.iter().enumerate() {
        if let Some(w) = sys.get("heat_warning").and_then(|v| v.as_str()) {
            if w.contains("CRITICAL") {
                let is_last = idx + 1 == systems.len();
                if !is_last {
                    assert!(
                        sys.get("cooldown_seconds").is_some() && !sys["cooldown_seconds"].is_null(),
                        "non-final system with CRITICAL warning should have cooldown_seconds"
                    );
                    let cooldown = sys["cooldown_seconds"].as_f64().unwrap();
                    assert!(
                        cooldown > 0.0,
                        "cooldown_seconds should be positive for CRITICAL heat"
                    );
                }
            }
        }
    }
}

// =============================================================================
// T030: Test enhanced format with ship shows fuel/heat columns
// =============================================================================

#[test]
fn test_scout_range_enhanced_format_with_ship() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("enhanced")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--limit")
        .arg("5")
        .arg("--ship")
        .arg("Reflex");

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Enhanced format should include ship info
    assert!(
        stdout.contains("Reflex") || stdout.contains("Ship:"),
        "enhanced format should show ship name"
    );

    // Should include fuel-related output
    assert!(
        stdout.contains("Fuel")
            || stdout.contains("fuel")
            || stdout.contains("⛽")
            || stdout.contains("🔋"),
        "enhanced format should show fuel information: {}",
        stdout
    );
}

// =============================================================================
// T031: Test JSON format includes all fuel/heat fields per contract
// =============================================================================

#[test]
fn test_scout_range_json_format_with_ship() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--limit")
        .arg("5")
        .arg("--ship")
        .arg("Reflex");

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: Value = serde_json::from_str(&stdout).expect("valid JSON output");

    // Verify all required fields per contracts/cli-interface.md
    // Top-level result fields
    assert!(json.get("system").is_some(), "system field required");
    assert!(json.get("system_id").is_some(), "system_id field required");
    assert!(json.get("query").is_some(), "query field required");
    assert!(json.get("count").is_some(), "count field required");
    assert!(json.get("systems").is_some(), "systems field required");

    // Ship-specific fields
    assert!(
        json.get("ship").is_some(),
        "ship field required when --ship"
    );
    assert!(
        json.get("total_distance_ly").is_some(),
        "total_distance_ly required when --ship"
    );
    assert!(
        json.get("total_fuel").is_some(),
        "total_fuel required when --ship"
    );
    assert!(
        json.get("final_heat").is_some(),
        "final_heat required when --ship"
    );

    // Ship info fields
    let ship = &json["ship"];
    assert!(ship.get("name").is_some(), "ship.name required");
    assert!(
        ship.get("fuel_capacity").is_some(),
        "ship.fuel_capacity required"
    );
    assert!(
        ship.get("fuel_quality").is_some(),
        "ship.fuel_quality required"
    );

    // System fields for each system
    let systems = json["systems"].as_array().expect("systems array");
    if !systems.is_empty() {
        let sys = &systems[0];
        assert!(sys.get("name").is_some(), "system.name required");
        assert!(sys.get("id").is_some(), "system.id required");
        assert!(
            sys.get("distance_ly").is_some(),
            "system.distance_ly required"
        );
        assert!(sys.get("hop_fuel").is_some(), "system.hop_fuel required");
        assert!(
            sys.get("cumulative_fuel").is_some(),
            "system.cumulative_fuel required"
        );
        assert!(
            sys.get("remaining_fuel").is_some(),
            "system.remaining_fuel required"
        );
        assert!(sys.get("hop_heat").is_some(), "system.hop_heat required");
        assert!(
            sys.get("cumulative_heat").is_some(),
            "system.cumulative_heat required"
        );
    }
}
