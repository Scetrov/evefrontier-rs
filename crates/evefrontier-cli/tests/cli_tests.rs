//! Integration tests for CLI commands, particularly index-verify.
//!
//! These tests use `assert_cmd` to verify CLI behavior including:
//! - index-verify with fresh, stale, missing, and legacy format scenarios
//! - JSON output format
//! - Exit codes

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Path to the test fixture database.
fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/fixtures/minimal/static_data.db")
        .canonicalize()
        .expect("fixture dataset present")
}

/// Create a CLI command with proper environment setup for testing.
fn cli() -> Command {
    cargo_bin_cmd!("evefrontier-cli")
}

/// Helper to create a temporary test environment.
struct TestEnv {
    _temp_dir: TempDir,
    data_dir: PathBuf,
    db_path: PathBuf,
    index_path: PathBuf,
    cache_dir: PathBuf,
}

impl TestEnv {
    /// Create a new test environment by copying the fixture database.
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("create temp dir");
        let data_dir = temp_dir.path().to_path_buf();
        let cache_dir = data_dir.join("cache");
        let db_path = data_dir.join("static_data.db");
        let index_path = data_dir.join("static_data.db.spatial.bin");

        // Create cache directory
        fs::create_dir_all(&cache_dir).expect("create cache dir");

        // Copy fixture database
        fs::copy(fixture_path(), &db_path).expect("copy fixture");

        Self {
            _temp_dir: temp_dir,
            data_dir,
            db_path,
            index_path,
            cache_dir,
        }
    }

    /// Create a command configured with proper environment for this test env.
    fn command(&self) -> Command {
        let mut cmd = cli();
        cmd.env("EVEFRONTIER_DATASET_SOURCE", fixture_path())
            .env("EVEFRONTIER_DATASET_CACHE_DIR", &self.cache_dir)
            .env("RUST_LOG", "error");
        cmd
    }

    /// Create a release marker file.
    fn create_release_marker(&self, tag: &str) {
        let marker_path = self.db_path.with_extension("db.release");
        let mut file = fs::File::create(&marker_path).expect("create marker");
        writeln!(file, "requested=latest").expect("write marker");
        writeln!(file, "resolved={}", tag).expect("write marker");
    }

    /// Build a v2 index using the CLI.
    fn build_v2_index(&self) {
        self.command()
            .args([
                "--data-dir",
                self.data_dir.to_str().unwrap(),
                "index-build",
                "--force",
            ])
            .assert()
            .success();
    }

    /// Build a v1 index (without metadata) by using the library directly.
    fn build_v1_index(&self) {
        use evefrontier_lib::{load_starmap, SpatialIndex};

        let starmap = load_starmap(&self.db_path).expect("load starmap");
        let index = SpatialIndex::build(&starmap); // v1 format (no metadata)
        index.save(&self.index_path).expect("save index");
    }
}

// =============================================================================
// T036: test_index_verify_fresh
// =============================================================================

#[test]
fn test_index_verify_fresh() {
    let env = TestEnv::new();
    env.create_release_marker("e6c3");
    env.build_v2_index();

    env.command()
        .args(["--data-dir", env.data_dir.to_str().unwrap(), "index-verify"])
        .assert()
        .success()
        .stdout(predicate::str::contains("✓ Spatial index is fresh"));
}

// =============================================================================
// T037: test_index_verify_stale
// =============================================================================

#[test]
fn test_index_verify_stale() {
    let env = TestEnv::new();
    env.create_release_marker("e6c3");
    env.build_v2_index();

    // Modify the database to change its checksum
    {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&env.db_path)
            .expect("open db for append");
        file.write_all(b"extra data to change checksum")
            .expect("append data");
    }

    env.command()
        .args(["--data-dir", env.data_dir.to_str().unwrap(), "index-verify"])
        .assert()
        .code(1) // STALE exit code
        .stdout(predicate::str::contains("✗ Spatial index is STALE"));
}

// =============================================================================
// T038: test_index_verify_missing
// =============================================================================

#[test]
fn test_index_verify_missing() {
    let env = TestEnv::new();
    // Don't build an index

    env.command()
        .args(["--data-dir", env.data_dir.to_str().unwrap(), "index-verify"])
        .assert()
        .code(2) // MISSING exit code
        .stdout(predicate::str::contains("✗ Spatial index not found"));
}

// =============================================================================
// T039: test_index_verify_json_output
// =============================================================================

#[test]
fn test_index_verify_json_output() {
    let env = TestEnv::new();
    env.create_release_marker("e6c3");
    env.build_v2_index();

    // Disable logo and footer for clean JSON output
    let output = env
        .command()
        .args([
            "--no-logo",
            "--no-footer",
            "--data-dir",
            env.data_dir.to_str().unwrap(),
            "index-verify",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).expect("valid JSON output");

    // Verify JSON structure
    assert!(json.get("result").is_some(), "result field should exist");
    assert!(
        json.get("is_fresh").is_some(),
        "is_fresh field should exist"
    );
    assert!(
        json.get("diagnostics").is_some(),
        "diagnostics field should exist"
    );

    // Verify freshness
    assert_eq!(json["is_fresh"], true);
    assert_eq!(json["result"]["status"], "fresh");
}

// =============================================================================
// T040: test_index_verify_exit_codes
// =============================================================================

#[test]
fn test_index_verify_exit_codes() {
    // Test SUCCESS (0) - fresh index
    {
        let env = TestEnv::new();
        env.build_v2_index();

        env.command()
            .args(["--data-dir", env.data_dir.to_str().unwrap(), "index-verify"])
            .assert()
            .code(0);
    }

    // Test MISSING (2) - no index file
    {
        let env = TestEnv::new();
        // Don't build index

        env.command()
            .args(["--data-dir", env.data_dir.to_str().unwrap(), "index-verify"])
            .assert()
            .code(2);
    }

    // Test FORMAT_ERROR (3) - legacy v1 format
    {
        let env = TestEnv::new();
        env.build_v1_index();

        env.command()
            .args(["--data-dir", env.data_dir.to_str().unwrap(), "index-verify"])
            .assert()
            .code(3);
    }
}

// Additional test for legacy format output message
#[test]
fn test_index_verify_legacy_format() {
    let env = TestEnv::new();
    env.build_v1_index();

    env.command()
        .args(["--data-dir", env.data_dir.to_str().unwrap(), "index-verify"])
        .assert()
        .code(3) // FORMAT_ERROR exit code
        .stdout(predicate::str::contains("legacy format"));
}

// Test quiet mode only outputs on failure
#[test]
fn test_index_verify_quiet_mode() {
    let env = TestEnv::new();
    env.build_v2_index();

    // Quiet mode with fresh index should produce no output
    // Also disable logo and footer for a clean test
    env.command()
        .args([
            "--no-logo",
            "--no-footer",
            "--data-dir",
            env.data_dir.to_str().unwrap(),
            "index-verify",
            "--quiet",
        ])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}
