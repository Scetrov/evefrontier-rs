//! Application state for HTTP microservices.
//!
//! This module provides the shared state structure that axum handlers use to
//! access the loaded starmap and spatial index.

use std::path::Path;
use std::sync::Arc;

use evefrontier_lib::db::{load_starmap, Starmap};
use evefrontier_lib::spatial::{try_load_spatial_index, SpatialIndex};
use evefrontier_lib::Error as LibError;

/// Error during application state initialization.
#[derive(Debug)]
pub enum AppStateError {
    /// Failed to load the starmap database.
    StarmapLoad(LibError),

    /// Failed to load the spatial index.
    SpatialIndexLoad(LibError),

    /// Database file not found.
    DatabaseNotFound(String),

    /// Spatial index file not found.
    SpatialIndexNotFound(String),
}

impl std::fmt::Display for AppStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StarmapLoad(e) => write!(f, "failed to load starmap: {}", e),
            Self::SpatialIndexLoad(e) => write!(f, "failed to load spatial index: {}", e),
            Self::DatabaseNotFound(path) => write!(f, "database not found: {}", path),
            Self::SpatialIndexNotFound(path) => write!(f, "spatial index not found: {}", path),
        }
    }
}

impl std::error::Error for AppStateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::StarmapLoad(e) => Some(e),
            Self::SpatialIndexLoad(e) => Some(e),
            _ => None,
        }
    }
}

impl From<LibError> for AppStateError {
    fn from(err: LibError) -> Self {
        Self::StarmapLoad(err)
    }
}

/// Shared application state for all axum handlers.
///
/// This struct is cheaply cloneable (using `Arc` internally) and should be
/// shared via axum's `State` extractor.
///
/// # Example
///
/// ```ignore
/// use axum::{Router, routing::get, extract::State};
/// use evefrontier_service_shared::AppState;
///
/// async fn handler(State(state): State<AppState>) {
///     let starmap = state.starmap();
///     // ... use starmap
/// }
///
/// let state = AppState::load("path/to/database.db").unwrap();
/// let app = Router::new()
///     .route("/api/v1/route", get(handler))
///     .with_state(state);
/// ```
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    starmap: Starmap,
    spatial_index: Option<Arc<SpatialIndex>>,
}

impl AppState {
    /// Load application state from a database file.
    ///
    /// Attempts to load the starmap from the specified database path. Also
    /// attempts to load a spatial index from `{db_path}.spatial.bin` if present.
    ///
    /// # Arguments
    ///
    /// * `db_path` - Path to the SQLite database file
    ///
    /// # Returns
    ///
    /// Returns an `AppState` on success, or an `AppStateError` if loading fails.
    pub fn load(db_path: impl AsRef<Path>) -> Result<Self, AppStateError> {
        let db_path = db_path.as_ref();

        if !db_path.exists() {
            return Err(AppStateError::DatabaseNotFound(
                db_path.display().to_string(),
            ));
        }

        // Load the starmap
        tracing::info!(path = %db_path.display(), "loading starmap");
        let starmap = load_starmap(db_path)?;
        tracing::info!(
            system_count = starmap.systems.len(),
            "starmap loaded successfully"
        );

        // Try to load spatial index (optional) - uses the library's try_load function
        // which handles path construction and error handling
        tracing::info!(path = %db_path.display(), "attempting to load spatial index");
        let spatial_index = try_load_spatial_index(db_path).map(Arc::new);
        if let Some(ref index) = spatial_index {
            tracing::info!(
                indexed_systems = index.len(),
                "spatial index loaded successfully"
            );
        } else {
            tracing::info!("spatial index not found, spatial queries may be slower");
        }

        Ok(Self {
            inner: Arc::new(AppStateInner {
                starmap,
                spatial_index,
            }),
        })
    }

    /// Create application state from pre-loaded components.
    ///
    /// This is useful for testing or when loading from bundled bytes.
    pub fn from_components(starmap: Starmap, spatial_index: Option<SpatialIndex>) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                starmap,
                spatial_index: spatial_index.map(Arc::new),
            }),
        }
    }

    /// Access the loaded starmap.
    pub fn starmap(&self) -> &Starmap {
        &self.inner.starmap
    }

    /// Access the loaded spatial index, if available.
    pub fn spatial_index(&self) -> Option<&SpatialIndex> {
        self.inner.spatial_index.as_deref()
    }

    /// Check if the spatial index is available.
    pub fn has_spatial_index(&self) -> bool {
        self.inner.spatial_index.is_some()
    }

    /// Get an Arc-wrapped reference to the spatial index for route planning.
    ///
    /// Returns `None` if the spatial index is not available.
    pub fn spatial_index_arc(&self) -> Option<Arc<SpatialIndex>> {
        self.inner.spatial_index.clone()
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("system_count", &self.inner.starmap.systems.len())
            .field("has_spatial_index", &self.inner.spatial_index.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use evefrontier_lib::db::{SystemMetadata, SystemPosition};
    use std::collections::HashMap;

    fn minimal_starmap() -> Starmap {
        let mut systems = HashMap::new();
        systems.insert(
            1,
            evefrontier_lib::db::System {
                id: 1,
                name: "TestSystem".to_string(),
                metadata: SystemMetadata {
                    constellation_id: None,
                    constellation_name: None,
                    region_id: None,
                    region_name: None,
                    security_status: None,
                    star_temperature: Some(5500.0),
                    star_luminosity: None,
                    min_external_temp: None,
                    planet_count: None,
                    moon_count: None,
                },
                position: Some(SystemPosition {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }),
            },
        );

        Starmap {
            systems,
            adjacency: Arc::new(HashMap::new()),
            name_to_id: {
                let mut m = HashMap::new();
                m.insert("testsystem".to_string(), 1);
                m
            },
        }
    }

    #[test]
    fn test_app_state_from_components() {
        let starmap = minimal_starmap();
        let state = AppState::from_components(starmap, None);

        assert_eq!(state.starmap().systems.len(), 1);
        assert!(!state.has_spatial_index());
        assert!(state.spatial_index().is_none());
    }

    #[test]
    fn test_app_state_clone() {
        let starmap = minimal_starmap();
        let state1 = AppState::from_components(starmap, None);
        let state2 = state1.clone();

        // Both should point to the same inner data
        assert_eq!(
            state1.starmap().systems.len(),
            state2.starmap().systems.len()
        );
    }

    #[test]
    fn test_app_state_debug() {
        let starmap = minimal_starmap();
        let state = AppState::from_components(starmap, None);
        let debug = format!("{:?}", state);

        assert!(debug.contains("AppState"));
        assert!(debug.contains("system_count"));
        assert!(debug.contains("has_spatial_index"));
    }

    #[test]
    fn test_app_state_error_display() {
        let err = AppStateError::DatabaseNotFound("/path/to/db".to_string());
        assert!(err.to_string().contains("/path/to/db"));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_app_state_load_nonexistent() {
        let result = AppState::load("/nonexistent/path/to/database.db");
        assert!(result.is_err());

        match result.unwrap_err() {
            AppStateError::DatabaseNotFound(path) => {
                assert!(path.contains("nonexistent"));
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }
}
