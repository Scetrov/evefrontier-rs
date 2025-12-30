//! Health check handlers for Kubernetes probes.
//!
//! Provides `/health/live` and `/health/ready` endpoints that return JSON
//! status responses for Kubernetes liveness and readiness probes.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

/// Health status response for liveness and readiness probes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Status indicator: "ok" or "degraded".
    pub status: String,

    /// Service name for identification.
    pub service: String,

    /// Service version from build-time.
    pub version: String,

    /// Number of systems loaded (for readiness check).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub systems_loaded: Option<usize>,

    /// Whether spatial index is available (for readiness check).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spatial_index_ready: Option<bool>,
}

impl HealthStatus {
    /// Create a healthy liveness status.
    pub fn alive(service: &str, version: &str) -> Self {
        Self {
            status: "ok".to_string(),
            service: service.to_string(),
            version: version.to_string(),
            systems_loaded: None,
            spatial_index_ready: None,
        }
    }

    /// Create a ready status with system information.
    pub fn ready(service: &str, version: &str, systems: usize, spatial_ready: bool) -> Self {
        Self {
            status: "ok".to_string(),
            service: service.to_string(),
            version: version.to_string(),
            systems_loaded: Some(systems),
            spatial_index_ready: Some(spatial_ready),
        }
    }

    /// Create a not-ready status.
    pub fn not_ready(service: &str, version: &str, reason: &str) -> Self {
        Self {
            status: format!("not_ready: {}", reason),
            service: service.to_string(),
            version: version.to_string(),
            systems_loaded: None,
            spatial_index_ready: None,
        }
    }
}

/// Liveness probe handler.
///
/// Returns 200 OK if the service is running. This is a simple check that does
/// not depend on external resources.
///
/// # Example
///
/// ```text
/// GET /health/live
/// {"status":"ok","service":"route","version":"0.1.0"}
/// ```
pub async fn health_live() -> impl IntoResponse {
    let status = HealthStatus::alive(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    (StatusCode::OK, Json(status))
}

/// Readiness probe handler.
///
/// Returns 200 OK if the service is ready to accept traffic. Checks that the
/// starmap and spatial index are loaded.
///
/// # Example
///
/// ```text
/// GET /health/ready
/// {"status":"ok","service":"route","version":"0.1.0","systems_loaded":5431,"spatial_index_ready":true}
/// ```
pub async fn health_ready(State(state): State<AppState>) -> Response {
    let service = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");

    let starmap = state.starmap();
    let systems_count = starmap.systems.len();
    let spatial_ready = state.spatial_index().is_some();

    // Check minimum viable state
    if systems_count == 0 {
        let status = HealthStatus::not_ready(service, version, "no systems loaded");
        return (StatusCode::SERVICE_UNAVAILABLE, Json(status)).into_response();
    }

    let status = HealthStatus::ready(service, version, systems_count, spatial_ready);
    (StatusCode::OK, Json(status)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_alive() {
        let status = HealthStatus::alive("test-service", "1.0.0");
        assert_eq!(status.status, "ok");
        assert_eq!(status.service, "test-service");
        assert_eq!(status.version, "1.0.0");
        assert!(status.systems_loaded.is_none());
        assert!(status.spatial_index_ready.is_none());
    }

    #[test]
    fn test_health_status_ready() {
        let status = HealthStatus::ready("test-service", "1.0.0", 5000, true);
        assert_eq!(status.status, "ok");
        assert_eq!(status.systems_loaded, Some(5000));
        assert_eq!(status.spatial_index_ready, Some(true));
    }

    #[test]
    fn test_health_status_not_ready() {
        let status = HealthStatus::not_ready("test-service", "1.0.0", "no data");
        assert!(status.status.starts_with("not_ready:"));
        assert!(status.status.contains("no data"));
    }

    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus::alive("route", "0.1.0");
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"service\":\"route\""));
        assert!(!json.contains("systems_loaded")); // skip_serializing_if
    }
}
