//! Health check handlers for Kubernetes probes.
//!
//! Provides `/health/live` and `/health/ready` endpoints that return JSON
//! status responses for Kubernetes liveness and readiness probes.

use std::collections::HashMap;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

/// Health check status values.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    /// Check passed successfully.
    Ok,
    /// Check passed but with degraded performance.
    Degraded,
    /// Check failed.
    Error,
}

impl CheckStatus {
    /// Returns true if this status indicates a healthy state.
    pub fn is_healthy(&self) -> bool {
        matches!(self, CheckStatus::Ok | CheckStatus::Degraded)
    }
}

/// Individual health check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Status of this check.
    pub status: CheckStatus,

    /// Optional message providing more details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Additional details specific to the check.
    #[serde(flatten)]
    pub details: HashMap<String, serde_json::Value>,
}

impl CheckResult {
    /// Create an OK check result.
    pub fn ok() -> Self {
        Self {
            status: CheckStatus::Ok,
            message: None,
            details: HashMap::new(),
        }
    }

    /// Create an OK check result with a detail value.
    pub fn ok_with_detail(key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        let mut details = HashMap::new();
        details.insert(key.into(), value.into());
        Self {
            status: CheckStatus::Ok,
            message: None,
            details,
        }
    }

    /// Create a degraded check result.
    pub fn degraded(message: impl Into<String>) -> Self {
        Self {
            status: CheckStatus::Degraded,
            message: Some(message.into()),
            details: HashMap::new(),
        }
    }

    /// Create an error check result.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: CheckStatus::Error,
            message: Some(message.into()),
            details: HashMap::new(),
        }
    }
}

/// Health status response for liveness and readiness probes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Status indicator: "ok", "degraded", or "not_ready".
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

    /// Detailed check results for each dependency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<HashMap<String, CheckResult>>,
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
            checks: None,
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
            checks: None,
        }
    }

    /// Create a ready status with detailed dependency checks.
    pub fn ready_with_checks(
        service: &str,
        version: &str,
        systems: usize,
        spatial_ready: bool,
        checks: HashMap<String, CheckResult>,
    ) -> Self {
        // Determine overall status based on checks
        let all_healthy = checks.values().all(|c| c.status.is_healthy());
        let any_degraded = checks.values().any(|c| c.status == CheckStatus::Degraded);

        let status = if !all_healthy {
            "not_ready".to_string()
        } else if any_degraded {
            "degraded".to_string()
        } else {
            "ok".to_string()
        };

        Self {
            status,
            service: service.to_string(),
            version: version.to_string(),
            systems_loaded: Some(systems),
            spatial_index_ready: Some(spatial_ready),
            checks: Some(checks),
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
            checks: None,
        }
    }

    /// Returns true if all checks passed (status is "ok" or "degraded").
    pub fn is_healthy(&self) -> bool {
        self.status == "ok" || self.status == "degraded"
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
/// starmap and spatial index are loaded. Returns detailed check results for
/// each dependency.
///
/// # Example (success)
///
/// ```text
/// GET /health/ready
/// HTTP/1.1 200 OK
/// {"status":"ok","service":"route","version":"0.1.0","systems_loaded":5431,"spatial_index_ready":true,"checks":{"database":{"status":"ok","systems_count":5431},"spatial_index":{"status":"ok"}}}
/// ```
///
/// # Example (failure)
///
/// ```text
/// GET /health/ready
/// HTTP/1.1 503 Service Unavailable
/// {"status":"not_ready","service":"route","version":"0.1.0","checks":{"database":{"status":"error","message":"no systems loaded"}}}
/// ```
pub async fn health_ready(State(state): State<AppState>) -> Response {
    let service = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");

    let starmap = state.starmap();
    let systems_count = starmap.systems.len();
    let spatial_ready = state.spatial_index().is_some();

    // Build detailed check results
    let mut checks = HashMap::new();

    // Database/starmap check
    if systems_count > 0 {
        checks.insert(
            "database".to_string(),
            CheckResult::ok_with_detail("systems_count", systems_count as i64),
        );
    } else {
        checks.insert(
            "database".to_string(),
            CheckResult::error("no systems loaded"),
        );
    }

    // Spatial index check
    if spatial_ready {
        checks.insert("spatial_index".to_string(), CheckResult::ok());
    } else {
        // Spatial index is optional - degraded but not error
        checks.insert(
            "spatial_index".to_string(),
            CheckResult::degraded("spatial index not loaded"),
        );
    }

    let status =
        HealthStatus::ready_with_checks(service, version, systems_count, spatial_ready, checks);

    // Return 503 if any check failed (not just degraded)
    if !status.is_healthy() {
        return (StatusCode::SERVICE_UNAVAILABLE, Json(status)).into_response();
    }

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
        assert!(status.checks.is_none());
    }

    #[test]
    fn test_health_status_ready() {
        let status = HealthStatus::ready("test-service", "1.0.0", 5000, true);
        assert_eq!(status.status, "ok");
        assert_eq!(status.systems_loaded, Some(5000));
        assert_eq!(status.spatial_index_ready, Some(true));
        assert!(status.checks.is_none());
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

    #[test]
    fn test_check_status_is_healthy() {
        assert!(CheckStatus::Ok.is_healthy());
        assert!(CheckStatus::Degraded.is_healthy());
        assert!(!CheckStatus::Error.is_healthy());
    }

    #[test]
    fn test_check_result_ok() {
        let result = CheckResult::ok();
        assert_eq!(result.status, CheckStatus::Ok);
        assert!(result.message.is_none());
        assert!(result.details.is_empty());
    }

    #[test]
    fn test_check_result_ok_with_detail() {
        let result = CheckResult::ok_with_detail("systems_count", 5000);
        assert_eq!(result.status, CheckStatus::Ok);
        assert!(result.message.is_none());
        assert_eq!(
            result.details.get("systems_count"),
            Some(&serde_json::json!(5000))
        );
    }

    #[test]
    fn test_check_result_degraded() {
        let result = CheckResult::degraded("index not loaded");
        assert_eq!(result.status, CheckStatus::Degraded);
        assert_eq!(result.message, Some("index not loaded".to_string()));
    }

    #[test]
    fn test_check_result_error() {
        let result = CheckResult::error("database unavailable");
        assert_eq!(result.status, CheckStatus::Error);
        assert_eq!(result.message, Some("database unavailable".to_string()));
    }

    #[test]
    fn test_health_ready_returns_checks_map() {
        let mut checks = HashMap::new();
        checks.insert(
            "database".to_string(),
            CheckResult::ok_with_detail("systems_count", 5431),
        );
        checks.insert("spatial_index".to_string(), CheckResult::ok());

        let status = HealthStatus::ready_with_checks("route", "0.1.0", 5431, true, checks);

        assert_eq!(status.status, "ok");
        assert!(status.is_healthy());

        let checks = status.checks.expect("checks should be present");
        assert_eq!(checks.len(), 2);
        assert!(checks.contains_key("database"));
        assert!(checks.contains_key("spatial_index"));

        let db_check = checks.get("database").unwrap();
        assert_eq!(db_check.status, CheckStatus::Ok);
    }

    #[test]
    fn test_health_ready_503_on_failure() {
        let mut checks = HashMap::new();
        checks.insert(
            "database".to_string(),
            CheckResult::error("no systems loaded"),
        );
        checks.insert("spatial_index".to_string(), CheckResult::ok());

        let status = HealthStatus::ready_with_checks("route", "0.1.0", 0, true, checks);

        assert_eq!(status.status, "not_ready");
        assert!(!status.is_healthy());
    }

    #[test]
    fn test_health_degraded_still_healthy() {
        let mut checks = HashMap::new();
        checks.insert("database".to_string(), CheckResult::ok());
        checks.insert(
            "spatial_index".to_string(),
            CheckResult::degraded("index not loaded"),
        );

        let status = HealthStatus::ready_with_checks("route", "0.1.0", 5000, false, checks);

        assert_eq!(status.status, "degraded");
        assert!(status.is_healthy()); // Degraded is still considered healthy
    }

    #[test]
    fn test_check_result_serialization() {
        let result = CheckResult::ok_with_detail("systems_count", 5000);
        let json = serde_json::to_string(&result).unwrap();

        // Check that details are flattened (not nested under "details")
        assert!(json.contains("\"systems_count\":5000"));
        assert!(json.contains("\"status\":\"ok\""));
        assert!(!json.contains("\"details\"")); // Should be flattened
    }

    #[test]
    fn test_health_status_with_checks_serialization() {
        let mut checks = HashMap::new();
        checks.insert(
            "database".to_string(),
            CheckResult::ok_with_detail("systems_count", 100),
        );

        let status = HealthStatus::ready_with_checks("route", "0.1.0", 100, true, checks);
        let json = serde_json::to_string(&status).unwrap();

        // Verify the JSON structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["status"], "ok");
        assert_eq!(parsed["checks"]["database"]["status"], "ok");
        assert_eq!(parsed["checks"]["database"]["systems_count"], 100);
    }
}
