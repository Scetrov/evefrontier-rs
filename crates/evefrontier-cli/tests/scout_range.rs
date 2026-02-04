//! Integration tests for `scout range` subcommand.
//!
//! Tests verify:
//! - Default limit returns nearby systems
//! - Radius filter limits results
//! - Max temperature filter works
//! - Combined filters work correctly
//! - JSON output produces valid, parseable JSON
//! - Unknown system returns error with fuzzy suggestions

use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/fixtures/minimal/static_data.db")
        .canonicalize()
        .expect("fixture dataset present")
}

fn cli() -> Command {
    cargo_bin_cmd!("evefrontier-cli")
}

fn prepare_command() -> (Command, tempfile::TempDir) {
    let temp_dir = tempdir().expect("create temp dir");
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&cache_dir).expect("create cache dir");
    let mut cmd = cli();
    cmd.env("EVEFRONTIER_DATASET_SOURCE", fixture_path())
        .env("EVEFRONTIER_DATASET_CACHE_DIR", &cache_dir)
        .env("RUST_LOG", "error")
        .arg("--no-logo")
        .arg("--data-dir")
        .arg(temp_dir.path());
    (cmd, temp_dir)
}

#[test]
fn test_scout_range_default_limit() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("basic")
        .arg("scout")
        .arg("range")
        .arg("Nod");

    cmd.assert()
        .success()
        // Basic format should show the origin and nearby systems
        .stdout(predicate::str::contains("Nod"))
        .stdout(predicate::str::contains("ly"));
}

#[test]
fn test_scout_range_with_limit() {
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

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON output");

    // Query should reflect the limit
    assert_eq!(json["query"]["limit"], 3);
}

#[test]
fn test_scout_range_with_radius() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--radius")
        .arg("150.0");

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON output");

    // Query should reflect the radius
    assert_eq!(json["query"]["radius"], 150.0);
}

#[test]
fn test_scout_range_with_max_temp() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--max-temp")
        .arg("10000.0");

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON output");

    // Query should reflect the max temp (field is "max_temperature")
    assert_eq!(json["query"]["max_temperature"], 10000.0);
}

#[test]
fn test_scout_range_combined_filters() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--limit")
        .arg("5")
        .arg("--radius")
        .arg("200.0")
        .arg("--max-temp")
        .arg("5000.0");

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON output");

    // Query should reflect all filters
    assert_eq!(json["query"]["limit"], 5);
    assert_eq!(json["query"]["radius"], 200.0);
    assert_eq!(json["query"]["max_temperature"], 5000.0);
}

#[test]
fn test_scout_range_json_output() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--limit")
        .arg("5");

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify it's valid JSON
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON output");

    // Verify expected structure
    assert_eq!(json["system"], "Nod");
    assert!(json["system_id"].as_u64().is_some());
    assert!(json["query"]["limit"].as_u64().is_some());
    assert!(json["count"].as_u64().is_some());
    assert!(json["systems"].is_array());

    // Check system structure if results exist
    let systems = json["systems"].as_array().expect("systems array");
    if !systems.is_empty() {
        let first_system = &systems[0];
        assert!(first_system["name"].is_string());
        assert!(first_system["id"].as_u64().is_some());
        assert!(first_system["distance_ly"].as_f64().is_some());
        assert!(first_system["min_temp_k"].as_f64().is_some());
    }
}

#[test]
fn test_scout_range_unknown_system_suggests_matches() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("scout").arg("range").arg("Nodd"); // Typo: Nodd instead of Nod

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unknown system 'Nodd'"))
        .stderr(predicate::str::contains("Did you mean"));
}

#[test]
fn test_scout_range_enhanced_output() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("enhanced")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--limit")
        .arg("5");

    cmd.assert()
        .success()
        // Enhanced format shows header with system name and count
        .stdout(predicate::str::contains("Systems in range"))
        .stdout(predicate::str::contains("Nod"))
        .stdout(predicate::str::contains("found"))
        .stdout(predicate::str::contains("ly"));
}

#[test]
fn test_scout_range_limit_validation() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--limit")
        .arg("0");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("limit"));
}

#[test]
fn test_scout_range_limit_max_validation() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--limit")
        .arg("101");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("limit"));
}

#[test]
fn test_scout_range_help() {
    let mut cmd = cli();
    cmd.arg("scout").arg("range").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("range"))
        .stdout(predicate::str::contains("SYSTEM"))
        .stdout(predicate::str::contains("limit"))
        .stdout(predicate::str::contains("radius"))
        .stdout(predicate::str::contains("max-temp"));
}

#[test]
fn test_scout_range_text_format() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("text")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--limit")
        .arg("3");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Systems within range of Nod"))
        // Text format uses numbered list with distance
        .stdout(predicate::str::contains("ly"));
}

#[test]
fn test_scout_range_emoji_format() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("emoji")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--limit")
        .arg("3");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Systems within range of Nod"))
        // Emoji format uses star emoji
        .stdout(predicate::str::contains("🌟"));
}

#[test]
fn test_scout_range_note_format() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("note")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--limit")
        .arg("3");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Systems within range of Nod"))
        // Note format uses in-game hyperlinks
        .stdout(predicate::str::contains("<a href=\"showinfo:5//"));
}

#[test]
fn test_scout_range_with_avoid() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--radius")
        .arg("200.0")
        .arg("--avoid")
        .arg("Brana")
        .arg("--avoid")
        .arg("H:2L2S");

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON output");

    // Check that Brana and H:2L2S are not in the results
    let systems = json["systems"].as_array().expect("systems is an array");
    for system in systems {
        let name = system["name"].as_str().expect("name is a string");
        assert_ne!(name, "Brana", "Brana should be excluded by --avoid");
        assert_ne!(name, "H:2L2S", "H:2L2S should be excluded by --avoid");
    }
}

#[test]
fn test_scout_range_with_max_jump() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--radius")
        .arg("100.0")
        .arg("--max-jump")
        .arg("50.0");

    // This test verifies the command accepts --max-jump parameter
    // Note: --max-jump affects pathfinding, not direct spatial queries,
    // so its effect is limited in scout range without route planning mode
    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let _json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON output");
    // Just verify it parses successfully - max-jump doesn't filter scout range results directly
}

#[test]
fn test_scout_range_with_avoid_gates() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--radius")
        .arg("50.0")
        .arg("--avoid-gates");

    // This test verifies --avoid-gates excludes gate-connected systems
    // Nod has 3 gate neighbors in fixture: D:2NAS, H:2L2S, J:35IA
    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON output");
    let systems = json["systems"].as_array().expect("systems array");

    // Verify none of the gate-connected neighbors appear in results
    let gate_neighbors = ["D:2NAS", "H:2L2S", "J:35IA"];
    for system in systems {
        let name = system["name"].as_str().expect("system name");
        assert!(
            !gate_neighbors.contains(&name),
            "Gate-connected system {} should be excluded with --avoid-gates",
            name
        );
    }
}

#[test]
fn test_scout_range_with_dynamic_mass() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--radius")
        .arg("50.0")
        .arg("--ship")
        .arg("Reflex")
        .arg("--dynamic-mass");

    // This test verifies the command accepts --dynamic-mass parameter
    // Dynamic mass enables per-hop mass recalculation for fuel projections
    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON output");
    let systems = json["systems"].as_array().expect("systems array");

    // Verify fuel projections exist when ship is specified
    if !systems.is_empty() {
        let first_system = &systems[0];
        // Fuel info should be present when ship specified
        assert!(
            first_system.get("fuel_info").is_some(),
            "fuel_info present with --ship"
        );
    }
}

#[test]
fn test_scout_range_static_vs_dynamic_mass() {
    // Test that static mass (default) and dynamic mass produce different results
    // when cargo is consumed (dynamic mass should show decreasing mass over route)

    let (mut cmd_static, _temp1) = prepare_command();
    cmd_static
        .arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--radius")
        .arg("50.0")
        .arg("--ship")
        .arg("Reflex")
        .arg("--cargo-mass")
        .arg("1000.0"); // Add cargo to make mass difference visible

    let assert_static = cmd_static.assert().success();
    let output_static = assert_static.get_output();
    let stdout_static = String::from_utf8_lossy(&output_static.stdout);
    let json_static: serde_json::Value = serde_json::from_str(&stdout_static).expect("valid JSON");

    let (mut cmd_dynamic, _temp2) = prepare_command();
    cmd_dynamic
        .arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--radius")
        .arg("50.0")
        .arg("--ship")
        .arg("Reflex")
        .arg("--cargo-mass")
        .arg("1000.0")
        .arg("--dynamic-mass");

    let assert_dynamic = cmd_dynamic.assert().success();
    let output_dynamic = assert_dynamic.get_output();
    let stdout_dynamic = String::from_utf8_lossy(&output_dynamic.stdout);
    let json_dynamic: serde_json::Value =
        serde_json::from_str(&stdout_dynamic).expect("valid JSON");

    // Both should succeed and return systems
    let systems_static = json_static["systems"].as_array().expect("systems array");
    let systems_dynamic = json_dynamic["systems"].as_array().expect("systems array");

    assert_eq!(
        systems_static.len(),
        systems_dynamic.len(),
        "Same systems returned regardless of mass calculation mode"
    );

    // Note: The actual fuel consumption difference would be visible in the fuel_info
    // field if we implemented detailed per-system fuel tracking. For now, we just
    // verify both modes execute successfully.
}

#[test]
fn test_scout_range_avoid_critical_state() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--radius")
        .arg("100.0")
        .arg("--avoid-critical-state");

    // This test verifies --avoid-critical-state parameter works
    // It should limit max_temperature to 150K
    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON output");

    // Verify query parameters reflect heat constraint
    let query = &json["query"];
    assert_eq!(
        query["max_temperature"], 150.0,
        "--avoid-critical-state should set max_temperature to 150K"
    );

    // All returned systems should have temperature <= 150K
    let systems = json["systems"].as_array().expect("systems array");
    for system in systems {
        if let Some(temp) = system["external_temp_k"].as_f64() {
            assert!(
                temp <= 150.0,
                "System {} has temperature {}K > 150K (critical threshold)",
                system["name"],
                temp
            );
        }
    }
}

#[test]
fn test_scout_range_no_avoid_critical_state() {
    // Test that --no-avoid-critical-state disables heat constraints
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("range")
        .arg("Nod")
        .arg("--radius")
        .arg("100.0")
        .arg("--avoid-critical-state")
        .arg("--no-avoid-critical-state"); // This should override

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON output");

    // Verify query parameters show no temperature limit
    let query = &json["query"];
    assert!(
        query["max_temperature"].is_null(),
        "--no-avoid-critical-state should override --avoid-critical-state"
    );
}
