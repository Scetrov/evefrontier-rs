use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

fn fixture_db() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/fixtures/minimal_static_data.db")
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

#[test]
fn json_output_includes_fuel_projection() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("route")
        .arg("--from")
        .arg("Nod")
        .arg("--to")
        .arg("Brana")
        .arg("--ship")
        .arg("Reflex")
        .arg("--fuel-quality")
        .arg("10")
        .arg("--fuel-load")
        .arg("1750")
        .arg("--cargo-mass")
        .arg("0");

    let output = cmd.assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    let value: Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert!(value.get("fuel").is_some(), "summary fuel present");
    assert_eq!(value["fuel"]["ship_name"], "Reflex");
    assert!(value["fuel"]["total"].as_f64().unwrap() > 0.0);

    let steps = value["steps"].as_array().expect("steps array");
    assert!(steps.len() >= 2, "at least origin and one hop");
    let hop_fuel = &steps[1]["fuel"];
    assert!(hop_fuel.is_object(), "hop fuel present");
    assert!(hop_fuel["hop_cost"].as_f64().unwrap() >= 0.0);
}

#[test]
fn json_output_includes_heat_projection() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("route")
        .arg("--from")
        .arg("Nod")
        .arg("--to")
        .arg("Brana")
        .arg("--ship")
        .arg("Reflex")
        .arg("--fuel-quality")
        .arg("10")
        .arg("--fuel-load")
        .arg("1750")
        .arg("--cargo-mass")
        .arg("0");

    let output = cmd.assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    let value: Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert!(value.get("heat").is_some(), "summary heat present");
    // `warnings` may be omitted when empty; accept either an array or absence.
    if let Some(warnings) = value["heat"].get("warnings") {
        assert!(warnings.is_array(), "summary heat warnings present");
    }

    let steps = value["steps"].as_array().expect("steps array");
    assert!(steps.len() >= 2, "at least origin and one hop");
    let hop_heat = &steps[1]["heat"];
    assert!(hop_heat.is_object(), "hop heat present");
    assert!(hop_heat["hop_heat"].as_f64().unwrap() >= 0.0);
}

#[test]
fn text_output_mentions_heat_when_ship_selected() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("route")
        .arg("--from")
        .arg("Nod")
        .arg("--to")
        .arg("Brana")
        .arg("--ship")
        .arg("Reflex")
        .arg("--fuel-quality")
        .arg("10")
        .arg("--fuel-load")
        .arg("1750")
        .arg("--format")
        .arg("enhanced");

    // The footer heat summary was removed; ensure per-step heat is shown instead.
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("heat +"));
}

#[test]
fn text_output_mentions_fuel_when_ship_selected() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("route")
        .arg("--from")
        .arg("Nod")
        .arg("--to")
        .arg("Brana")
        .arg("--ship")
        .arg("Reflex")
        .arg("--fuel-quality")
        .arg("10")
        .arg("--fuel-load")
        .arg("1750")
        .arg("--format")
        .arg("enhanced");

    // Enhanced format shows "Fuel (Reflex):" and "Remaining:" labels
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Fuel (Reflex):"))
        .stdout(predicate::str::contains("Remaining:"));
    // Fuel total should use thousand separators and include fuel quality suffix
    cmd.assert()
        .stdout(predicate::str::contains("(10% Fuel)"))
        .stdout(predicate::str::contains(","));
}

#[test]
fn text_output_fuel_and_remaining_align() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("route")
        .arg("--from")
        .arg("Nod")
        .arg("--to")
        .arg("Brana")
        .arg("--ship")
        .arg("Reflex")
        .arg("--fuel-quality")
        .arg("10")
        .arg("--fuel-load")
        .arg("1750")
        .arg("--format")
        .arg("enhanced");

    let output = cmd.assert().success().get_output().stdout.clone();
    let stdout = String::from_utf8(output).unwrap();

    let lines: Vec<&str> = stdout.lines().collect();
    let fuel_line = lines
        .iter()
        .find(|l| l.contains("Fuel (Reflex):"))
        .expect("fuel line present");
    let rem_line = lines
        .iter()
        .find(|l| l.contains("Remaining:"))
        .expect("remaining line present");

    let fuel_pos = fuel_line
        .chars()
        .position(|c| c.is_ascii_digit())
        .expect("digit in fuel");
    let rem_pos = rem_line
        .chars()
        .position(|c| c.is_ascii_digit())
        .expect("digit in remaining");

    assert_eq!(
        fuel_pos, rem_pos,
        "Fuel and Remaining numbers should start at the same column"
    );
}

#[test]
fn text_output_shows_refuel_when_insufficient_fuel() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("route")
        .arg("--from")
        .arg("Nod")
        .arg("--to")
        .arg("Brana")
        .arg("--ship")
        .arg("Reflex")
        .arg("--fuel-quality")
        .arg("10")
        // Intentionally tiny fuel load to force a refuel
        .arg("--fuel-load")
        .arg("1")
        .arg("--format")
        .arg("enhanced");

    let output = cmd.assert().success().get_output().stdout.clone();
    let stdout = String::from_utf8(output).unwrap();

    // Should show a single REFUEL tag on the step where refueling is required
    let refuel_count = stdout.matches("REFUEL").count();
    assert_eq!(
        refuel_count, 1,
        "expected exactly one REFUEL tag in enhanced output"
    );

    // Remaining should reset to the original fuel load (1)
    let remaining_line = stdout
        .lines()
        .find(|l| l.contains("Remaining:"))
        .expect("remaining line present");
    let rem_part = remaining_line.split("Remaining:").nth(1).unwrap().trim();
    // Strip common ANSI sequences (color codes) used in the enhanced output
    let rem_clean = rem_part
        .replace("\x1b[0m", "")
        .replace("\x1b[1;97m", "")
        .trim()
        .to_string();
    assert_eq!(
        rem_clean, "1",
        "expected remaining to show original fuel load"
    );
}

#[test]
fn cli_accepts_cooling_mode_flag() {
    // Cooling mode flag was removed; test not needed.
}

#[test]
fn cli_default_calibration_produces_heat() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("route")
        .arg("--from")
        .arg("Nod")
        .arg("--to")
        .arg("Brana")
        .arg("--ship")
        .arg("Reflex")
        .arg("--fuel-quality")
        .arg("10")
        .arg("--fuel-load")
        .arg("1750")
        .arg("--cargo-mass")
        .arg("0");

    let output = cmd.assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let value: Value = serde_json::from_str(&stdout).expect("valid JSON");
    let steps = value["steps"].as_array().expect("steps array");
    let hop_heat = steps[1]["heat"]["hop_heat"].as_f64().unwrap();
    // Ensure the fixed default calibration produces non-zero hop heat for jump hops.
    if let Some(method) = steps[1]["method"].as_str() {
        if method == "jump" {
            assert!(
                hop_heat > 0.0,
                "hop heat should be > 0 for jump with default calibration"
            );
        }
    }
}
