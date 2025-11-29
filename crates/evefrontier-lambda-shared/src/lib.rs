//! Shared infrastructure for EveFrontier AWS Lambda functions.
//!
//! This crate provides common functionality used across all Lambda handlers:
//!
//! - [`LambdaRuntime`]: Pre-loaded starmap and spatial index for zero-latency access
//! - [`init_tracing`]: JSON-formatted tracing for CloudWatch Logs
//! - [`ProblemDetails`]: RFC 9457 Problem Details for consistent error responses
//! - [`LambdaResponse`]: Wrapper for successful responses with content type
//! - Request types with validation for each Lambda endpoint

#![deny(warnings)]

mod problem;
mod requests;
mod response;
mod runtime;
mod tracing_init;

pub use problem::{
    from_lib_error, ProblemDetails, PROBLEM_INTERNAL_ERROR, PROBLEM_INVALID_REQUEST,
    PROBLEM_ROUTE_NOT_FOUND, PROBLEM_SERVICE_UNAVAILABLE, PROBLEM_UNKNOWN_SYSTEM,
};
pub use requests::{RouteRequest, ScoutGatesRequest, ScoutRangeRequest, Validate};
pub use response::LambdaResponse;
pub use runtime::{get_runtime, init_error_to_problem, init_runtime, InitError, LambdaRuntime};
pub use tracing_init::init_tracing;
