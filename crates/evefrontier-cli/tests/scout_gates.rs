//! Integration tests for `scout gates` subcommand.
//!
//! Tests verify:
//! - Basic output format shows gate neighbors
//! - JSON output produces valid, parseable JSON
//! - Unknown system returns error with fuzzy suggestions
//! - System with no gate neighbors returns empty result

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
fn test_scout_gates_basic_output() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("basic")
        .arg("scout")
        .arg("gates")
        .arg("Nod");

    cmd.assert()
        .success()
        // Basic format should show the system name and neighbors
        .stdout(predicate::str::contains("Nod"))
        // Fixture has D:2NAS, H:2L2S, J:35IA as neighbors of Nod
        .stdout(predicate::str::contains("D:2NAS").or(predicate::str::contains("H:2L2S")));
}

#[test]
fn test_scout_gates_enhanced_output() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("enhanced")
        .arg("scout")
        .arg("gates")
        .arg("Nod");

    cmd.assert()
        .success()
        // Enhanced format shows header with system name and count
        .stdout(predicate::str::contains("Gate neighbors"))
        .stdout(predicate::str::contains("Nod"))
        .stdout(predicate::str::contains("found"))
        // Should show at least one neighbor with GATE tag
        .stdout(predicate::str::contains("[GATE]"))
        .stdout(predicate::str::contains("D:2NAS").or(predicate::str::contains("H:2L2S")));
}

#[test]
fn test_scout_gates_json_output() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("gates")
        .arg("Nod");

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify it's valid JSON
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON output");

    // Verify expected structure
    assert_eq!(json["system"], "Nod");
    assert!(json["system_id"].as_u64().is_some());
    assert!(json["count"].as_u64().is_some());
    assert!(json["neighbors"].is_array());

    // Should have at least one neighbor (Brana)
    let neighbors = json["neighbors"].as_array().expect("neighbors array");
    assert!(!neighbors.is_empty());

    // Check neighbor structure
    let first_neighbor = &neighbors[0];
    assert!(first_neighbor["name"].is_string());
    assert!(first_neighbor["id"].as_u64().is_some());
}

#[test]
fn test_scout_gates_unknown_system_suggests_matches() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("scout").arg("gates").arg("Nodd"); // Typo: Nodd instead of Nod

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unknown system 'Nodd'"))
        .stderr(predicate::str::contains("Did you mean"));
}

#[test]
fn test_scout_gates_completely_unknown_system() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("scout").arg("gates").arg("XyzNotASystem");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Unknown system 'XyzNotASystem'"));
}

#[test]
fn test_scout_gates_case_mismatch_suggests_correct() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("json")
        .arg("scout")
        .arg("gates")
        .arg("nod"); // lowercase

    // Case mismatch should give helpful error with suggestion
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Did you mean").and(predicate::str::contains("Nod")));
}

#[test]
fn test_scout_gates_help() {
    let mut cmd = cli();
    cmd.arg("scout").arg("gates").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("gate"))
        .stdout(predicate::str::contains("SYSTEM"));
}

#[test]
fn test_scout_gates_text_format() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("text")
        .arg("scout")
        .arg("gates")
        .arg("Nod");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Gate neighbors of Nod"))
        // Text format uses " - " prefix
        .stdout(predicate::str::contains(" - "));
}

#[test]
fn test_scout_gates_emoji_format() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("emoji")
        .arg("scout")
        .arg("gates")
        .arg("Nod");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Gate neighbors of Nod"))
        // Emoji format uses door emoji
        .stdout(predicate::str::contains("🚪"));
}

#[test]
fn test_scout_gates_note_format() {
    let (mut cmd, _temp) = prepare_command();
    cmd.arg("--format")
        .arg("note")
        .arg("scout")
        .arg("gates")
        .arg("Nod");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Gate neighbors of Nod"))
        // Note format uses in-game hyperlinks
        .stdout(predicate::str::contains("<a href=\"showinfo:5//"));
}