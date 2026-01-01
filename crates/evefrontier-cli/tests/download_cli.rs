use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;
use std::path::PathBuf;
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

#[test]
fn download_shows_ship_data_path_when_env_set() {
    let temp_dir = tempdir().expect("create temp dir");
    let cache_dir = temp_dir.path().join("cache");

    let mut cmd = cargo_bin_cmd!("evefrontier-cli");
    cmd.env("EVEFRONTIER_DATASET_SOURCE", fixture_db())
        .env("EVEFRONTIER_DATASET_CACHE_DIR", &cache_dir)
        .env("EVEFRONTIER_SHIP_DATA", fixture_ship())
        .arg("--no-logo")
        .arg("--data-dir")
        .arg(temp_dir.path())
        .arg("download");

    cmd.assert()
        .success()
        .stdout(contains("Dataset available at"))
        .stdout(contains("Ship data available at"));
}
