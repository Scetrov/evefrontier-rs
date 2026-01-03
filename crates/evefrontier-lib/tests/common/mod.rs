//! Common test utilities and fixture helpers.
//!
//! This module provides shared test infrastructure for integration tests,
//! including temporary file management and spatial index fixture generation.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use evefrontier_lib::output;
use evefrontier_lib::ship::{ShipAttributes, ShipCatalog};
use tempfile::TempDir;

/// Path to fixtures directory used by tests (ship data, minimal DB, etc.)
#[allow(dead_code)]
pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures")
}

/// Convenience helper to load the "Reflex" ship from test fixtures.
#[allow(dead_code)]
pub fn reflex_ship() -> ShipAttributes {
    let path = fixtures_dir().join("ship_data.csv");
    let catalog = ShipCatalog::from_path(&path).expect("load fixture ship_data.csv");
    catalog
        .get("Reflex")
        .expect("Reflex ship present in fixtures")
        .clone()
}

/// Path to the minimal test fixture database.
pub fn fixture_db_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/minimal_static_data.db")
}

/// Test environment with temporary files for spatial index testing.
///
/// Provides a clean directory with a copy of the fixture database,
/// and helper methods to create release markers and spatial index files.
#[allow(dead_code)]
pub struct SpatialTestEnv {
    /// Temp directory (dropped on struct drop)
    _temp_dir: TempDir,
    /// Path to the database file
    pub db_path: PathBuf,
    /// Path to the spatial index file
    pub index_path: PathBuf,
}

#[allow(dead_code)]
impl SpatialTestEnv {
    /// Create a new test environment by copying the fixture database.
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("create temp dir");
        let db_path = temp_dir.path().join("static_data.db");
        let index_path = temp_dir.path().join("static_data.db.spatial.bin");

        // Copy fixture database
        fs::copy(fixture_db_path(), &db_path).expect("copy fixture database");

        Self {
            _temp_dir: temp_dir,
            db_path,
            index_path,
        }
    }

    /// Create an empty test environment (no database copy).
    pub fn empty() -> Self {
        let temp_dir = TempDir::new().expect("create temp dir");
        let db_path = temp_dir.path().join("static_data.db");
        let index_path = temp_dir.path().join("static_data.db.spatial.bin");

        Self {
            _temp_dir: temp_dir,
            db_path,
            index_path,
        }
    }

    /// Path to the temporary directory.
    pub fn temp_path(&self) -> &std::path::Path {
        self._temp_dir.path()
    }

    /// Create a `.db.release` marker file with the given tag.
    pub fn create_release_marker(&self, tag: &str) {
        let marker_path = self.db_path.with_extension("db.release");
        let mut file = fs::File::create(&marker_path).expect("create release marker");
        writeln!(file, "requested=latest").expect("write marker line");
        writeln!(file, "resolved={}", tag).expect("write resolved tag");
    }

    /// Create a `.db.release` marker file with custom content.
    pub fn create_release_marker_raw(&self, content: &str) {
        let marker_path = self.db_path.with_extension("db.release");
        fs::write(&marker_path, content).expect("write marker file");
    }

    /// Append data to the database file (to change its checksum).
    pub fn modify_database(&self) {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&self.db_path)
            .expect("open database for append");
        file.write_all(b"-- test modification to change checksum\n")
            .expect("append data");
    }

    /// Write arbitrary bytes to the index file (for format testing).
    pub fn write_raw_index(&self, bytes: &[u8]) {
        fs::write(&self.index_path, bytes).expect("write raw index");
    }
}

impl Default for SpatialTestEnv {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a byte array to hex string for assertions.
pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Parse hex string to bytes for test fixtures.
pub fn from_hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("valid hex"))
        .collect()
}

/// Minimal `RouteStep` builder for use across integration tests.
#[allow(dead_code)]
pub struct RouteStepBuilder {
    step: output::RouteStep,
}

impl RouteStepBuilder {
    pub fn new() -> Self {
        Self {
            step: output::RouteStep {
                index: 1,
                id: 1,
                name: Some("Step 1".to_string()),
                distance: Some(1.0),
                method: Some("jump".to_string()),
                min_external_temp: None,
                planet_count: None,
                moon_count: None,
                fuel: None,
                heat: None,
            },
        }
    }

    pub fn index(mut self, idx: usize) -> Self {
        self.step.index = idx;
        self
    }

    pub fn id(mut self, id: i64) -> Self {
        self.step.id = id;
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.step.name = Some(name.to_string());
        self
    }

    pub fn distance(mut self, d: f64) -> Self {
        self.step.distance = Some(d);
        self
    }

    #[allow(dead_code)]
    pub fn method(mut self, method: &str) -> Self {
        self.step.method = Some(method.to_string());
        self
    }

    #[allow(dead_code)]
    pub fn min_temp(mut self, t: f64) -> Self {
        self.step.min_external_temp = Some(t);
        self
    }

    pub fn build(self) -> output::RouteStep {
        self.step
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_path_exists() {
        assert!(
            fixture_db_path().exists(),
            "fixture database should exist at {:?}",
            fixture_db_path()
        );
    }

    #[test]
    fn test_spatial_env_creates_db_copy() {
        let env = SpatialTestEnv::new();
        assert!(env.db_path.exists(), "database copy should exist");
        assert!(!env.index_path.exists(), "index should not exist yet");
    }

    #[test]
    fn test_hex_roundtrip() {
        let original = [0xDE, 0xAD, 0xBE, 0xEF];
        let hex = to_hex(&original);
        assert_eq!(hex, "deadbeef");

        let parsed = from_hex(&hex);
        assert_eq!(parsed, original);
    }
}
