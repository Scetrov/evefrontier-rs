use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn avoid_critical_state_without_ship_errors() {
    // Prepare a temporary copy of the fixture dataset to avoid protected fixture guard
    let dir = tempdir().expect("tempdir");
    let dest_db = dir.path().join("minimal_static_data.db");
    let src_db = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/fixtures/minimal_static_data.db");
    fs::copy(&src_db, &dest_db).expect("copy db");

    let mut cmd = cargo_bin_cmd!("evefrontier-cli");
    cmd.arg("route")
        .arg("--from")
        .arg("Nod")
        .arg("--to")
        .arg("Brana")
        .arg("--data-dir")
        .arg(dest_db)
        .arg("--avoid-gates")
        .arg("--avoid-critical-state");

    cmd.assert().failure().stderr(
        predicate::str::contains("--ship is required for heat-aware planning")
            .or(predicate::str::contains("ship '")),
    );
}

#[test]
fn avoid_critical_state_with_ship_succeeds_or_blocks() {
    // Prepare a temporary copy of the fixture dataset and ship CSV to avoid protected fixture guard
    let dir = tempdir().expect("tempdir");
    let dest_db = dir.path().join("minimal_static_data.db");
    let src_db = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/fixtures/minimal_static_data.db");
    fs::copy(&src_db, &dest_db).expect("copy db");
    let src_ship =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/ship_data.csv");
    let dest_ship = dir.path().join("ship_data.csv");
    fs::copy(&src_ship, &dest_ship).expect("copy ship csv");

    let src_release = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/fixtures/static_data.db.release");
    let dest_release = dir.path().join("minimal_static_data.db.release");
    if src_release.exists() {
        fs::copy(&src_release, &dest_release).expect("copy release marker");
    }

    let mut cmd = cargo_bin_cmd!("evefrontier-cli");
    cmd.arg("route")
        .arg("--from")
        .arg("Nod")
        .arg("--to")
        .arg("Brana")
        .arg("--data-dir")
        .arg(dest_db)
        .arg("--avoid-gates")
        .arg("--avoid-critical-state")
        .arg("--ship")
        .arg("Reflex");

    let assert = cmd.assert();
    // Accept either a success (exit code 0) or a clean failure saying no route found.
    let output = assert.get_output();
    let success = output.status.success();
    if success {
        // Ensure some route output present
        assert!(String::from_utf8_lossy(&output.stdout).contains("Route"));
    } else {
        // Check for helpful message
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("No route found") || stderr.contains("no route found"));
    }
}

#[test]
fn avoid_critical_state_with_ship_allows_route_when_gates_allowed() {
    // Prepare a temporary copy of the fixture dataset and ship CSV to avoid protected fixture guard
    let dir = tempdir().expect("tempdir");
    let dest_db = dir.path().join("minimal_static_data.db");
    let src_db = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/fixtures/minimal_static_data.db");
    fs::copy(&src_db, &dest_db).expect("copy db");
    let src_ship =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/ship_data.csv");
    let dest_ship = dir.path().join("ship_data.csv");
    fs::copy(&src_ship, &dest_ship).expect("copy ship csv");

    // Copy release marker so ensure_dataset treats the copy as fresh
    let src_release = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/fixtures/static_data.db.release");
    let dest_release = dir.path().join("minimal_static_data.db.release");
    if src_release.exists() {
        fs::copy(&src_release, &dest_release).expect("copy release marker");
    }

    let mut cmd = cargo_bin_cmd!("evefrontier-cli");
    cmd.arg("route")
        .arg("--from")
        .arg("Nod")
        .arg("--to")
        .arg("Brana")
        .arg("--data-dir")
        .arg(dest_db)
        .arg("--avoid-critical-state")
        .arg("--ship")
        .arg("Reflex");

    // This should succeed (gates allowed), and produce route output
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Route"));
}
