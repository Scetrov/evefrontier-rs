use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::str::contains;
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
fn lists_ships_with_attributes() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("ships");

    cmd.assert()
        .success()
        .stdout(contains("Available ships (3):"))
        .stdout(contains("Name"))
        .stdout(contains("Base Mass"))
        .stdout(contains("Reflex"))
        .stdout(contains("10000000"))
        .stdout(contains("1750"))
        .stdout(contains("800000"));
}

#[test]
fn lists_ships_from_cached_dataset() {
    let temp_dir = tempdir().expect("create temp dir");
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&cache_dir).expect("create cache dir");

    let mut cmd = cli();
    cmd.env("EVEFRONTIER_DATASET_SOURCE", fixture_db())
        .env("EVEFRONTIER_DATASET_CACHE_DIR", &cache_dir)
        .env("RUST_LOG", "error")
        .arg("--no-logo")
        .arg("--data-dir")
        .arg(temp_dir.path());

    cmd.arg("ships");

    cmd.assert()
        .success()
        .stdout(contains("Available ships (3):"))
        .stdout(contains("Reflex"));
}
