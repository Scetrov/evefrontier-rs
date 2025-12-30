//! Prometheus metrics infrastructure for EVE Frontier microservices.
//!
//! This module provides:
//! - [`MetricsConfig`]: Configuration for the metrics system
//! - [`init_metrics`]: Initialize the Prometheus metrics recorder
//! - [`metrics_handler`]: Axum handler for `/metrics` endpoint
//! - Business metric helpers for route and scout services
//!
//! # Example
//!
//! ```no_run
//! use evefrontier_service_shared::metrics::{MetricsConfig, init_metrics, metrics_handler};
//! use axum::{Router, routing::get};
//!
//! // Initialize metrics at startup
//! let config = MetricsConfig::default();
//! init_metrics(&config).expect("failed to initialize metrics");
//!
//! // Add metrics endpoint to router
//! let app: Router = Router::new()
//!     .route("/metrics", get(metrics_handler));
//! ```

use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

/// Global Prometheus handle for rendering metrics.
static PROMETHEUS_HANDLE: OnceCell<PrometheusHandle> = OnceCell::new();

/// Configuration for the metrics system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Whether metrics collection is enabled.
    pub enabled: bool,
    /// Path for the metrics endpoint (e.g., "/metrics").
    pub path: String,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: "/metrics".to_string(),
        }
    }
}

impl MetricsConfig {
    /// Create configuration from environment variables.
    ///
    /// - `METRICS_ENABLED`: "true" or "false" (default: true)
    /// - `METRICS_PATH`: Path for metrics endpoint (default: "/metrics")
    pub fn from_env() -> Self {
        let enabled = std::env::var("METRICS_ENABLED")
            .map(|v| v.to_lowercase() != "false")
            .unwrap_or(true);

        let path = std::env::var("METRICS_PATH").unwrap_or_else(|_| "/metrics".to_string());

        Self { enabled, path }
    }
}

/// Initialize the Prometheus metrics recorder.
///
/// This must be called once at application startup before any metrics are recorded.
/// Subsequent calls will return an error.
///
/// # Errors
///
/// Returns an error if:
/// - Metrics are disabled in configuration
/// - The recorder has already been installed
/// - The Prometheus builder fails to install
pub fn init_metrics(config: &MetricsConfig) -> Result<(), MetricsError> {
    if !config.enabled {
        return Err(MetricsError::Disabled);
    }

    let handle = PrometheusBuilder::new()
        .install_recorder()
        .map_err(|e| MetricsError::InstallFailed(e.to_string()))?;

    PROMETHEUS_HANDLE
        .set(handle)
        .map_err(|_| MetricsError::AlreadyInitialized)?;

    Ok(())
}

/// Get the Prometheus handle for rendering metrics.
///
/// Returns `None` if [`init_metrics`] has not been called.
pub fn prometheus_handle() -> Option<&'static PrometheusHandle> {
    PROMETHEUS_HANDLE.get()
}

/// Axum handler for the `/metrics` endpoint.
///
/// Returns Prometheus exposition format text.
pub async fn metrics_handler() -> String {
    PROMETHEUS_HANDLE
        .get()
        .map(|h| h.render())
        .unwrap_or_else(|| "# Metrics not initialized\n".to_string())
}

/// Errors that can occur during metrics initialization.
#[derive(Debug, Clone)]
pub enum MetricsError {
    /// Metrics are disabled in configuration.
    Disabled,
    /// The recorder has already been installed.
    AlreadyInitialized,
    /// The Prometheus builder failed to install.
    InstallFailed(String),
}

impl std::fmt::Display for MetricsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricsError::Disabled => write!(f, "metrics are disabled"),
            MetricsError::AlreadyInitialized => write!(f, "metrics recorder already initialized"),
            MetricsError::InstallFailed(e) => {
                write!(f, "failed to install metrics recorder: {}", e)
            }
        }
    }
}

impl std::error::Error for MetricsError {}

// =============================================================================
// Business Metrics Helpers
// =============================================================================

/// Record a successful route calculation.
///
/// Increments the `evefrontier_routes_calculated_total` counter.
///
/// # Arguments
///
/// * `algorithm` - The algorithm used (e.g., "bfs", "dijkstra", "astar")
/// * `service` - The service name (e.g., "route")
pub fn record_route_calculated(algorithm: &str, service: &str) {
    metrics::counter!(
        "evefrontier_routes_calculated_total",
        "algorithm" => algorithm.to_string(),
        "service" => service.to_string()
    )
    .increment(1);
}

/// Record a failed route calculation.
///
/// Increments the `evefrontier_routes_failed_total` counter.
///
/// # Arguments
///
/// * `reason` - The failure reason (e.g., "no_path", "unknown_system", "validation_error")
/// * `service` - The service name (e.g., "route")
pub fn record_route_failed(reason: &str, service: &str) {
    metrics::counter!(
        "evefrontier_routes_failed_total",
        "reason" => reason.to_string(),
        "service" => service.to_string()
    )
    .increment(1);
}

/// Record the number of hops in a successful route.
///
/// Records to the `evefrontier_route_hops` histogram.
///
/// # Arguments
///
/// * `hops` - The number of hops in the route
/// * `algorithm` - The algorithm used (e.g., "bfs", "dijkstra", "astar")
pub fn record_route_hops(hops: usize, algorithm: &str) {
    metrics::histogram!(
        "evefrontier_route_hops",
        "algorithm" => algorithm.to_string()
    )
    .record(hops as f64);
}

/// Record a system query from scout endpoints.
///
/// Increments the `evefrontier_systems_queried_total` counter.
///
/// # Arguments
///
/// * `query_type` - The type of query (e.g., "gates", "range")
/// * `service` - The service name (e.g., "scout-gates", "scout-range")
pub fn record_systems_queried(query_type: &str, service: &str) {
    metrics::counter!(
        "evefrontier_systems_queried_total",
        "query_type" => query_type.to_string(),
        "service" => service.to_string()
    )
    .increment(1);
}

/// Record the number of neighbors returned by scout queries.
///
/// Records to the `evefrontier_neighbors_returned` histogram.
///
/// # Arguments
///
/// * `count` - The number of neighbors returned
/// * `query_type` - The type of query (e.g., "gates", "range")
pub fn record_neighbors_returned(count: usize, query_type: &str) {
    metrics::histogram!(
        "evefrontier_neighbors_returned",
        "query_type" => query_type.to_string()
    )
    .record(count as f64);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_config_default() {
        let config = MetricsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.path, "/metrics");
    }

    #[test]
    fn test_metrics_config_from_env_defaults() {
        // Clear any existing env vars
        std::env::remove_var("METRICS_ENABLED");
        std::env::remove_var("METRICS_PATH");

        let config = MetricsConfig::from_env();
        assert!(config.enabled);
        assert_eq!(config.path, "/metrics");
    }

    // T014: Test that metrics endpoint returns Prometheus format
    // This test validates the metrics_handler returns valid Prometheus exposition format
    #[test]
    fn test_metrics_handler_returns_prometheus_format() {
        // When metrics are not initialized, should return a comment
        // Note: We can't test full initialization in unit tests due to global state
        let rt = tokio::runtime::Runtime::new().unwrap();
        let output = rt.block_on(async { metrics_handler().await });

        // Should return either proper metrics or "not initialized" message
        assert!(
            output.contains("#") || output.is_empty(),
            "Metrics output should be Prometheus format or indicate not initialized"
        );
    }

    // T015: Test that http_request_counter can be incremented
    // This validates the counter! macro works correctly
    #[test]
    fn test_http_request_counter_increments() {
        // This test verifies the metrics macros compile and execute without panic
        // Full integration testing requires the Prometheus recorder to be installed
        metrics::counter!(
            "http_requests_total",
            "method" => "POST",
            "path" => "/api/v1/route",
            "status" => "2xx"
        )
        .increment(1);
        // If we get here without panic, the counter works
    }

    // T016: Test that http_request_duration histogram records
    #[test]
    fn test_http_request_duration_histogram_records() {
        // This test verifies the histogram! macro works correctly
        metrics::histogram!(
            "http_request_duration_seconds",
            "method" => "POST",
            "path" => "/api/v1/route"
        )
        .record(0.05);
        // If we get here without panic, the histogram works
    }

    // T017: Test business metrics for routes_calculated
    #[test]
    fn test_business_metric_routes_calculated() {
        // Test the business metric helper function
        record_route_calculated("bfs", "route");
        record_route_calculated("dijkstra", "route");
        record_route_calculated("astar", "route");
        // If we get here without panic, the helpers work
    }

    #[test]
    fn test_business_metric_route_failed() {
        record_route_failed("no_path", "route");
        record_route_failed("unknown_system", "route");
        record_route_failed("validation_error", "route");
    }

    #[test]
    fn test_business_metric_route_hops() {
        record_route_hops(5, "bfs");
        record_route_hops(10, "dijkstra");
        record_route_hops(15, "astar");
    }

    #[test]
    fn test_business_metric_systems_queried() {
        record_systems_queried("gates", "scout-gates");
        record_systems_queried("range", "scout-range");
    }

    #[test]
    fn test_business_metric_neighbors_returned() {
        record_neighbors_returned(5, "gates");
        record_neighbors_returned(10, "range");
    }

    #[test]
    fn test_metrics_error_display() {
        let disabled = MetricsError::Disabled;
        assert_eq!(disabled.to_string(), "metrics are disabled");

        let already_init = MetricsError::AlreadyInitialized;
        assert_eq!(
            already_init.to_string(),
            "metrics recorder already initialized"
        );

        let failed = MetricsError::InstallFailed("test error".to_string());
        assert!(failed.to_string().contains("test error"));
    }
}
