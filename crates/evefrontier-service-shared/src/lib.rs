//! Shared infrastructure for EVE Frontier HTTP microservices.
//!
//! This crate provides common functionality used across all microservice containers:
//!
//! - [`AppState`]: Pre-loaded starmap and spatial index for zero-latency access
//! - [`health`]: Health check handlers for Kubernetes liveness/readiness probes
//! - [`ProblemDetails`]: RFC 9457 Problem Details for consistent error responses
//! - [`ServiceResponse`]: Wrapper for successful responses with content type
//! - [`metrics`]: Prometheus metrics infrastructure
//! - [`logging`]: Structured JSON logging setup
//! - [`middleware`]: Request tracking and metrics middleware
//! - Request types with validation for each endpoint
//!
//! # Architecture
//!
//! The services follow a thin-handler pattern where all business logic resides
//! in `evefrontier-lib`. This crate provides only HTTP glue:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │  axum Handler                                               │
//! │  - Parse request JSON                                       │
//! │  - Validate parameters                                      │
//! │  - Call evefrontier-lib APIs                                │
//! │  - Format response                                          │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Testing Support
//!
//! The [`test_utils`] module provides test fixtures and mock state for handler
//! testing. Enable the `test-utils` feature to access it from dependent crates.

#![deny(warnings)]

mod health;
pub mod logging;
pub mod metrics;
pub mod middleware;
mod problem;
mod request;
mod response;
mod state;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;

pub use health::{health_live, health_ready, HealthStatus};
pub use logging::{init_logging, LogFormat, LoggingConfig};
pub use metrics::{
    init_metrics, metrics_handler, record_neighbors_returned, record_route_calculated,
    record_route_failed, record_route_hops, record_systems_queried, MetricsConfig, MetricsError,
};
pub use middleware::{extract_or_generate_request_id, MetricsLayer, RequestId};
pub use problem::{
    from_lib_error, ProblemDetails, PROBLEM_INTERNAL_ERROR, PROBLEM_INVALID_REQUEST,
    PROBLEM_ROUTE_NOT_FOUND, PROBLEM_SERVICE_UNAVAILABLE, PROBLEM_UNKNOWN_SYSTEM,
};
pub use request::{RouteAlgorithm, RouteRequest, ScoutGatesRequest, ScoutRangeRequest, Validate};
pub use response::ServiceResponse;
pub use state::{AppState, AppStateError};
