use std::fs;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
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
fn route_subcommand_outputs_steps() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("route")
        .arg("--from")
        .arg("Y:170N")
        .arg("--to")
        .arg("BetaTest");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Route: Y:170N -> BetaTest"))
        .stdout(predicate::str::contains("algorithm: bfs"));
}

#[test]
fn search_subcommand_supports_json_output() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("search")
        .arg("--from")
        .arg("Y:170N")
        .arg("--to")
        .arg("BetaTest");

    let output = cmd.assert().success().get_output().stdout.clone();
    let json: Value = serde_json::from_slice(&output).expect("valid json");

    assert_eq!(json["kind"], "search");
    assert_eq!(json["algorithm"], "bfs");
    assert_eq!(json["start"]["name"], "Y:170N");
    assert_eq!(json["goal"]["name"], "BetaTest");
}

#[test]
fn path_subcommand_shows_arrow_format() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("path")
        .arg("--from")
        .arg("Y:170N")
        .arg("--to")
        .arg("BetaTest");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Path: Y:170N -> BetaTest"))
        .stdout(predicate::str::contains("Y:170N"))
        .stdout(predicate::str::contains("->"));
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
fn note_format_outputs_in_game_layout() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("note")
        .arg("path")
        .arg("--from")
        .arg("Y:170N")
        .arg("--to")
        .arg("BetaTest");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Path:"))
        .stdout(predicate::str::contains("Y:170N"))
        .stdout(predicate::str::contains("BetaTest"));
}
