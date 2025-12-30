//! Common test utilities and fixture helpers.
//!
//! This module provides shared test infrastructure for integration tests,
//! including temporary file management and spatial index fixture generation.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use tempfile::TempDir;

/// Path to the minimal test fixture database.
pub fn fixture_db_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/minimal_static_data.db")
}

/// Test environment with temporary files for spatial index testing.
///
/// Provides a clean directory with a copy of the fixture database,
/// and helper methods to create release markers and spatial index files.
pub struct SpatialTestEnv {
    /// Temp directory (dropped on struct drop)
    _temp_dir: TempDir,
    /// Path to the database file
    pub db_path: PathBuf,
    /// Path to the spatial index file
    pub index_path: PathBuf,
}

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
