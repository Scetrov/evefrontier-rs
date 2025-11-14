use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/fixtures/minimal_static_data.db")
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
fn dijkstra_algorithm_is_supported() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("route")
        .arg("--from")
        .arg("Y:170N")
        .arg("--to")
        .arg("BetaTest")
        .arg("--algorithm")
        .arg("dijkstra");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("algorithm: dijkstra"));
}

#[test]
fn basic_format_outputs_minimal_path() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("basic")
        .arg("route")
        .arg("--from")
        .arg("Y:170N")
        .arg("--to")
        .arg("BetaTest");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("+ Y:170N"))
        .stdout(predicate::str::contains("- BetaTest"));
}

#[test]
fn unknown_system_error_is_friendly() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("route")
        .arg("--from")
        .arg("Y:170N")
        .arg("--to")
        .arg("GammaTest");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unknown system 'GammaTest'"))
        .stderr(predicate::str::contains("Did you mean"));
}

#[test]
fn route_not_found_error_suggests_next_steps() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("route")
        .arg("--from")
        .arg("Y:170N")
        .arg("--to")
        .arg("AlphaTest")
        .arg("--avoid")
        .arg("AlphaTest");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(
            "No route found between Y:170N and AlphaTest.",
        ))
        .stderr(predicate::str::contains("Try a different algorithm"));
}
