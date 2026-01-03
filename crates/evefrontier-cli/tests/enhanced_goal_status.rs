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
        .env("NO_COLOR", "1")
        .arg("--no-logo")
        .arg("--data-dir")
        .arg(temp_dir.path());
    (cmd, temp_dir)
}

#[test]
fn goal_step_shows_status_line_in_enhanced_format() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("enhanced")
        .arg("route")
        .arg("--from")
        .arg("Nod")
        .arg("--to")
        .arg("Brana");

    let output = cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("GOAL  ● Brana"))
        .stdout(predicate::str::contains("│ min"))
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).expect("utf8 output");
    let goal_pos = stdout.find("GOAL").expect("goal present");
    let status_pos = stdout[goal_pos..]
        .find("│")
        .map(|idx| goal_pos + idx)
        .expect("status line present after goal");
    assert!(status_pos > goal_pos);
}
