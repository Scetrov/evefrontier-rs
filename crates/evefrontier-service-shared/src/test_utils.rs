//! Test utilities for microservice handler testing.
//!
//! This module provides fixtures and helpers for testing HTTP handlers
//! with a pre-loaded minimal starmap.

use std::path::PathBuf;
use std::sync::OnceLock;

use crate::state::AppState;

/// Path to the test fixture database.
pub const TEST_FIXTURE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/fixtures/minimal/static_data.db"
);

/// Lazily-initialized test state using the fixture database.
static TEST_STATE: OnceLock<AppState> = OnceLock::new();

/// Get a shared test AppState loaded from the fixture database.
///
/// This function caches the state after the first load, so subsequent calls
/// are very fast.
///
/// # Panics
///
/// Panics if the fixture database cannot be loaded. This indicates a test
/// configuration issue.
pub fn test_state() -> AppState {
    TEST_STATE
        .get_or_init(|| {
            let path = PathBuf::from(TEST_FIXTURE_PATH);
            AppState::load(&path)
                .unwrap_or_else(|e| panic!("failed to load test fixture from {:?}: {}", path, e))
        })
        .clone()
}

/// Get the absolute path to the test fixture database.
pub fn fixture_db_path() -> PathBuf {
    PathBuf::from(TEST_FIXTURE_PATH)
}

/// Known system names in the test fixture for use in tests.
pub mod fixture_systems {
    /// System "Nod" - a real system from the e6c3 dataset.
    pub const NOD: &str = "Nod";

    /// System "Brana" - connected to Nod via gates.
    pub const BRANA: &str = "Brana";

    /// System "H:2L2S" - another real system in the fixture.
    pub const H_2L2S: &str = "H:2L2S";

    /// System "D:2NAS" - another real system in the fixture.
    pub const D_2NAS: &str = "D:2NAS";
}

/// Generate a unique request ID for testing.
pub fn test_request_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("test-{}", timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_path_exists() {
        let path = fixture_db_path();
        assert!(path.exists(), "fixture database not found at {:?}", path);
    }

    #[test]
    fn test_state_loads_successfully() {
        let state = test_state();
        assert!(!state.starmap().systems.is_empty());
    }

    #[test]
    fn test_state_contains_expected_systems() {
        let state = test_state();
        let starmap = state.starmap();

        // Check that known systems exist (name_to_id uses original casing)
        assert!(
            starmap.name_to_id.contains_key(fixture_systems::NOD),
            "Nod should exist in fixture"
        );
        assert!(
            starmap.name_to_id.contains_key(fixture_systems::BRANA),
            "Brana should exist in fixture"
        );
    }

    #[test]
    fn test_request_id_unique() {
        let id1 = test_request_id();
        let id2 = test_request_id();
        assert_ne!(id1, id2);
    }
}
