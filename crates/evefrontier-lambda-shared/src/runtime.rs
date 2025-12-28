//! Lambda runtime initialization with bundled dataset and spatial index.
//!
//! This module provides a lazily-initialized singleton containing the loaded
//! `Starmap` and `SpatialIndex`. The data is bundled at compile time using
//! `include_bytes!` for fast Lambda cold starts.
//!
//! # Dataset Bundling
//!
//! Lambda handlers bundle the dataset by including bytes directly in the binary:
//!
//! ```text
//! static DB_BYTES: &[u8] = include_bytes!("../../../data/static_data.db");
//! static INDEX_BYTES: &[u8] = include_bytes!("../../../data/static_data.db.spatial.bin");
//! ```
//!
//! # Cold-Start Performance
//!
//! The runtime logs timing metrics during initialization to help identify
//! cold-start bottlenecks:
//!
//! - `db_load_ms`: Time to deserialize SQLite database and load starmap
//! - `index_load_ms`: Time to decompress and load spatial index
//! - `total_init_ms`: Total initialization time

use std::sync::{Arc, OnceLock};
use std::time::Instant;

use rusqlite::Connection;
use tracing::{error, info};

use evefrontier_lib::db::{load_starmap_from_connection, Starmap};
use evefrontier_lib::spatial::SpatialIndex;
use evefrontier_lib::Error as LibError;

use crate::problem::ProblemDetails;

/// Lazily-initialized Lambda runtime state.
static RUNTIME: OnceLock<Result<LambdaRuntime, InitError>> = OnceLock::new();

/// Error during runtime initialization.
#[derive(Debug, Clone)]
pub struct InitError {
    pub message: String,
}

impl std::fmt::Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lambda initialization failed: {}", self.message)
    }
}

impl std::error::Error for InitError {}

impl From<LibError> for InitError {
    fn from(err: LibError) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

impl From<rusqlite::Error> for InitError {
    fn from(err: rusqlite::Error) -> Self {
        Self {
            message: format!("SQLite error: {}", err),
        }
    }
}

/// Initialized Lambda runtime containing the starmap and spatial index.
///
/// This struct is designed to be initialized once at cold start and reused
/// across all invocations.
pub struct LambdaRuntime {
    starmap: Starmap,
    spatial_index: Arc<SpatialIndex>,
}

impl LambdaRuntime {
    /// Access the loaded starmap.
    pub fn starmap(&self) -> &Starmap {
        &self.starmap
    }

    /// Access the loaded spatial index.
    pub fn spatial_index(&self) -> &SpatialIndex {
        &self.spatial_index
    }

    /// Get a shared reference to the spatial index for use in route requests.
    pub fn spatial_index_arc(&self) -> Option<Arc<SpatialIndex>> {
        Some(Arc::clone(&self.spatial_index))
    }
}

/// Initialize the Lambda runtime from bundled data.
///
/// This function is designed to be called by `include_bytes!` data from each
/// Lambda handler. It uses rusqlite's serialize feature to load the database
/// directly from memory.
///
/// # Arguments
///
/// * `db_bytes` - SQLite database bytes (from `include_bytes!`)
/// * `index_bytes` - Spatial index bytes (from `include_bytes!`)
///
/// # Returns
///
/// Returns a reference to the initialized `LambdaRuntime`, or an `InitError`
/// if initialization fails.
///
/// # Panics
///
/// This function will panic if called more than once. Use `get_runtime()` for
/// subsequent accesses.
pub fn init_runtime(db_bytes: &'static [u8], index_bytes: &'static [u8]) -> &'static LambdaRuntime {
    let result = RUNTIME.get_or_init(|| {
        let total_start = Instant::now();

        info!(
            db_size_bytes = db_bytes.len(),
            index_size_bytes = index_bytes.len(),
            "initializing Lambda runtime"
        );

        // Load database from bytes using rusqlite's serialize feature
        let db_start = Instant::now();
        let starmap = load_starmap_from_bytes(db_bytes)?;
        let db_load_ms = db_start.elapsed().as_millis();

        info!(
            db_load_ms = db_load_ms,
            system_count = starmap.systems.len(),
            "starmap loaded from bundled database"
        );

        // Load spatial index from bytes
        let index_start = Instant::now();
        let spatial_index = load_spatial_index_from_bytes(index_bytes)?;
        let index_load_ms = index_start.elapsed().as_millis();

        info!(
            index_load_ms = index_load_ms,
            indexed_systems = spatial_index.len(),
            "spatial index loaded"
        );

        let total_init_ms = total_start.elapsed().as_millis();
        info!(
            total_init_ms = total_init_ms,
            db_load_ms = db_load_ms,
            index_load_ms = index_load_ms,
            "Lambda runtime initialization complete"
        );

        Ok(LambdaRuntime {
            starmap,
            spatial_index: Arc::new(spatial_index),
        })
    });

    match result {
        Ok(runtime) => runtime,
        Err(e) => {
            error!(error = %e, "Lambda runtime initialization failed");
            panic!("Lambda runtime initialization failed: {}", e);
        }
    }
}

/// Get the initialized runtime.
///
/// # Panics
///
/// Panics if `init_runtime` has not been called or if initialization failed.
pub fn get_runtime() -> &'static LambdaRuntime {
    match RUNTIME.get() {
        Some(Ok(runtime)) => runtime,
        Some(Err(e)) => panic!("Lambda runtime initialization failed: {}", e),
        None => panic!("Lambda runtime not initialized. Call init_runtime() first."),
    }
}

/// Create a `ProblemDetails` for initialization errors.
///
/// Use this when the handler fails during cold start to return a proper
/// RFC 9457 error response.
pub fn init_error_to_problem(request_id: &str) -> ProblemDetails {
    match RUNTIME.get() {
        Some(Err(e)) => ProblemDetails::internal_error(e.message.clone(), request_id),
        _ => ProblemDetails::internal_error("Runtime initialization failed", request_id),
    }
}

/// Load starmap from in-memory SQLite bytes.
///
/// Uses rusqlite's serialize feature to deserialize the database directly
/// from the bundled bytes without writing to disk.
fn load_starmap_from_bytes(db_bytes: &'static [u8]) -> Result<Starmap, InitError> {
    // Create an in-memory database and deserialize the bundled bytes into it.
    // rusqlite's serialize feature allows loading a database from a byte slice.
    let mut conn = Connection::open_in_memory()?;

    // Use deserialize_bytes which is specifically designed for include_bytes! data
    conn.deserialize_bytes(rusqlite::MAIN_DB, db_bytes)
        .map_err(|e| InitError {
            message: format!("Failed to deserialize database: {}", e),
        })?;

    let starmap = load_starmap_from_connection(&conn).map_err(|e| InitError {
        message: format!("Failed to load starmap: {}", e),
    })?;

    Ok(starmap)
}

/// Load spatial index from compressed bytes.
///
/// The index format includes a header, zstd-compressed data, and checksum.
fn load_spatial_index_from_bytes(index_bytes: &[u8]) -> Result<SpatialIndex, InitError> {
    SpatialIndex::load_from_bytes(index_bytes).map_err(|e| InitError {
        message: format!("Failed to load spatial index: {}", e),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require the fixture database, which won't be available
    // in the Lambda build environment. Tests use conditional compilation.

    #[test]
    fn test_init_error_display() {
        let err = InitError {
            message: "test error".to_string(),
        };
        assert!(err.to_string().contains("test error"));
    }

    #[test]
    fn test_init_error_from_lib_error() {
        let lib_err = LibError::UnsupportedSchema;
        let init_err: InitError = lib_err.into();
        assert!(init_err.message.contains("unsupported"));
    }
}
