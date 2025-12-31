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
        .arg("1750");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("fuel:"))
        .stdout(predicate::str::contains("Total fuel"));
}
