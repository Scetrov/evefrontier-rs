//! Test utilities for Lambda handler testing.
//!
//! This module provides shared test infrastructure for all Lambda crates,
//! including fixture loading and mock Lambda context creation.
//!
//! # Usage
//!
//! These utilities are only available in test builds:
//!
//! ```ignore
//! use evefrontier_lambda_shared::test_utils::{
//!     fixture_db_bytes, fixture_spatial_index, mock_request_id,
//! };
//!
//! #[test]
//! fn test_handler() {
//!     let db_bytes = fixture_db_bytes();
//!     let index = fixture_spatial_index();
//!     let request_id = mock_request_id("test-request-123");
//!     // ... test handler logic
//! }
//! ```

use std::path::PathBuf;
use std::sync::OnceLock;

use evefrontier_lib::db::Starmap;
use evefrontier_lib::load_starmap;
use evefrontier_lib::spatial::SpatialIndex;

/// Path to the minimal test fixture database.
///
/// Contains 8 systems: Nod, Brana, D:2NAS, G:3OA0, H:2L2S, J:35IA, Y:3R7E, E1J-M5G.
fn fixture_db_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/minimal_static_data.db")
}

/// Load the fixture database bytes for bundling tests.
///
/// Returns the raw SQLite database bytes from the minimal test fixture.
/// This is useful for testing the runtime initialization path.
pub fn fixture_db_bytes() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| std::fs::read(fixture_db_path()).expect("fixture database should exist"))
}

/// Load the fixture starmap for direct testing.
///
/// Returns a reference to a lazily-loaded `Starmap` from the fixture database.
/// This avoids repeated database loading across multiple tests.
pub fn fixture_starmap() -> &'static Starmap {
    static STARMAP: OnceLock<Starmap> = OnceLock::new();
    STARMAP.get_or_init(|| load_starmap(&fixture_db_path()).expect("fixture starmap should load"))
}

/// Build a spatial index from the fixture starmap.
///
/// Returns a reference to a lazily-built `SpatialIndex` for spatial query tests.
/// The index is built dynamically since pre-built index files aren't committed.
pub fn fixture_spatial_index() -> &'static SpatialIndex {
    static INDEX: OnceLock<SpatialIndex> = OnceLock::new();
    INDEX.get_or_init(|| {
        let starmap = fixture_starmap();
        SpatialIndex::build(starmap)
    })
}

/// Serialize the fixture spatial index for runtime initialization tests.
///
/// Returns the serialized bytes of the spatial index, suitable for testing
/// `SpatialIndex::load_from_bytes()`.
pub fn fixture_index_bytes() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| {
        let index = fixture_spatial_index();
        // Use tempfile crate for RAII-based cleanup that guarantees file removal
        // when the NamedTempFile is dropped, even in failure scenarios.
        let temp_file = tempfile::NamedTempFile::new().expect("failed to create temporary file");
        let temp_path = temp_file.path();
        index.save(temp_path).expect("index save should succeed");
        // Read bytes before temp_file is dropped (which deletes the file)
        std::fs::read(temp_path).expect("index read should succeed")
    })
}

/// Load the fixture ship_data.csv bytes for bundling tests.
pub fn fixture_ship_bytes() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(|| {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/ship_data.csv");
        std::fs::read(path).expect("ship fixture should exist")
    })
}

/// Create a mock request ID for testing.
///
/// Since `lambda_runtime::Context` is non-exhaustive and cannot be directly
/// constructed, tests should use the request ID directly for assertions.
///
/// # Arguments
///
/// * `suffix` - A unique identifier suffix for the test request
///
/// # Returns
///
/// A request ID string in the format "test-request-{suffix}"
pub fn mock_request_id(suffix: &str) -> String {
    format!("test-request-{}", suffix)
}

/// System names available in the fixture database for test assertions.
///
/// These are real systems from the e6c3 dataset:
/// - Nod, Brana: Primary test systems with gate connections
/// - D:2NAS, G:3OA0, H:2L2S, J:35IA, Y:3R7E: Secondary systems
/// - E1J-M5G: Edge case system
pub mod systems {
    /// Primary test system (has gate connections).
    pub const NOD: &str = "Nod";
    /// Secondary test system (has gate connections to Nod).
    pub const BRANA: &str = "Brana";
    /// System with spatial connections.
    pub const D_2NAS: &str = "D:2NAS";
    /// System with spatial connections.
    pub const G_3OA0: &str = "G:3OA0";
    /// System with spatial connections.
    pub const H_2L2S: &str = "H:2L2S";
    /// System with spatial connections.
    pub const J_35IA: &str = "J:35IA";
    /// System with spatial connections.
    pub const Y_3R7E: &str = "Y:3R7E";
    /// Edge case system.
    pub const E1J_M5G: &str = "E1J-M5G";

    /// A system name that does not exist in the fixture.
    pub const NONEXISTENT: &str = "NonExistentSystem12345";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_db_bytes_loads() {
        let bytes = fixture_db_bytes();
        assert!(!bytes.is_empty(), "fixture should have content");
        // SQLite database starts with "SQLite format 3\0"
        assert!(
            bytes.starts_with(b"SQLite format 3\0"),
            "should be SQLite format"
        );
    }

    #[test]
    fn fixture_starmap_has_expected_systems() {
        let starmap = fixture_starmap();
        assert!(
            starmap.system_id_by_name(systems::NOD).is_some(),
            "Nod should exist"
        );
        assert!(
            starmap.system_id_by_name(systems::BRANA).is_some(),
            "Brana should exist"
        );
        assert!(
            starmap.system_id_by_name(systems::NONEXISTENT).is_none(),
            "nonexistent should not exist"
        );
    }

    #[test]
    fn fixture_spatial_index_has_systems() {
        let index = fixture_spatial_index();
        assert!(!index.is_empty(), "index should have systems");
    }

    #[test]
    fn fixture_index_bytes_serializes() {
        let bytes = fixture_index_bytes();
        assert!(!bytes.is_empty(), "index bytes should have content");
        // EFSI magic header
        assert!(bytes.starts_with(b"EFSI"), "should have EFSI magic header");
    }

    #[test]
    fn mock_request_id_formats_correctly() {
        let id = mock_request_id("123");
        assert_eq!(id, "test-request-123");
    }
}
