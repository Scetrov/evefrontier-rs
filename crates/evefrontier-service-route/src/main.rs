//! EVE Frontier route planning HTTP microservice.
//!
//! This service provides a REST API for computing routes between solar systems,
//! supporting multiple algorithms and constraints.
//!
//! # Endpoints
//!
//! - `POST /api/v1/route` - Compute a route between two systems
//! - `GET /metrics` - Prometheus metrics endpoint
//! - `GET /health/live` - Kubernetes liveness probe
//! - `GET /health/ready` - Kubernetes readiness probe
//!
//! # Configuration
//!
//! - `EVEFRONTIER_DATA_PATH` - Path to the static_data.db file (required)
//! - `RUST_LOG` - Log level (default: info)
//! - `LOG_FORMAT` - Log format: json (default) or text
//! - `SERVICE_PORT` - HTTP port (default: 8080)

use std::env;
use std::net::SocketAddr;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::Serialize;
use tracing::{error, info};

use evefrontier_lib::{
    RouteAlgorithm as LibAlgorithm, RouteConstraints as LibConstraints, RouteRequest as LibRequest,
    plan_route,
};
use evefrontier_service_shared::{
    AppState, LoggingConfig, MetricsConfig, MetricsLayer, ProblemDetails, RouteRequest,
    ServiceResponse, Validate, from_lib_error, health_live, health_ready, init_logging,
    init_metrics, metrics_handler, record_route_calculated, record_route_failed, record_route_hops,
};

/// Route response returned to the caller.
#[derive(Debug, Serialize)]
struct RouteResponse {
    /// Total number of hops in the route.
    hops: usize,
    /// Number of gate jumps.
    gates: usize,
    /// Number of spatial jumps.
    jumps: usize,
    /// Algorithm used.
    algorithm: String,
    /// Ordered list of system names in the route.
    route: Vec<String>,
}

/// HTTP response - either success or RFC 9457 error.
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Response {
    Success(ServiceResponse<RouteResponse>),
    Error(ProblemDetails),
}

impl IntoResponse for Response {
    fn into_response(self) -> axum::response::Response {
        match self {
            Response::Success(data) => (StatusCode::OK, Json(data)).into_response(),
            Response::Error(problem) => problem.into_response(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging (reads LOG_FORMAT from environment)
    let logging_config = LoggingConfig::from_env().with_service("route");
    init_logging(&logging_config);

    // Initialize metrics
    let metrics_config = MetricsConfig::from_env();
    if let Err(e) = init_metrics(&metrics_config) {
        // Log but don't fail - metrics are optional
        tracing::warn!(error = %e, "failed to initialize metrics, continuing without metrics");
    }

    // Load configuration from environment
    let data_path =
        env::var("EVEFRONTIER_DATA_PATH").unwrap_or_else(|_| "/data/static_data.db".to_string());
    let port: u16 = env::var("SERVICE_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    info!(data_path = %data_path, port = port, "starting route service");

    // Load application state
    let state = AppState::load(&data_path).map_err(|e| {
        error!(error = %e, path = %data_path, "failed to load application state");
        e
    })?;

    info!(
        systems = state.starmap().systems.len(),
        spatial_index = state.has_spatial_index(),
        "application state loaded"
    );

    // Build the router
    let app = Router::new()
        .route("/api/v1/route", post(route_handler))
        .route("/metrics", get(metrics_handler))
        .route("/health/live", get(health_live))
        .route("/health/ready", get(health_ready))
        .layer(MetricsLayer)
        .with_state(state);

    // Bind and serve
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(addr = %addr, "listening on");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Handle POST /api/v1/route requests.
async fn route_handler(
    State(state): State<AppState>,
    Json(request): Json<RouteRequest>,
) -> Response {
    // Generate a request ID for tracing
    let request_id = generate_request_id();

    info!(
        request_id = %request_id,
        from = %request.from,
        to = %request.to,
        algorithm = ?request.algorithm,
        "handling route request"
    );

    // Validate the request
    if let Err(problem) = request.validate(&request_id) {
        record_route_failed("validation_error", "route");
        return Response::Error(*problem);
    }

    let starmap = state.starmap();

    // Convert to library request
    let lib_request = LibRequest {
        start: request.from.clone(),
        goal: request.to.clone(),
        algorithm: LibAlgorithm::from(request.algorithm),
        constraints: LibConstraints {
            max_jump: request.max_jump,
            avoid_systems: request.avoid.clone(),
            avoid_gates: request.avoid_gates,
            max_temperature: request.max_temperature,
            // NOTE: `avoid_critical_state` is intentionally not exposed on the service API in
            // this change and is currently CLI-only. If we decide to support it via the
            // service, add a request field, validation, and tests; consider adding a
            // follow-up issue to track the work.
            avoid_critical_state: false,
            ship: None,
            loadout: None,
            heat_config: None,
        },
        spatial_index: state.spatial_index_arc(),
        max_spatial_neighbors: evefrontier_lib::GraphBuildOptions::default().max_spatial_neighbors,
        optimization: evefrontier_lib::routing::RouteOptimization::Distance,
        fuel_config: evefrontier_lib::ship::FuelConfig::default(),
    };

    // Plan the route
    let plan = match plan_route(starmap, &lib_request) {
        Ok(plan) => plan,
        Err(e) => {
            error!(request_id = %request_id, error = %e, "route planning failed");
            // Determine failure reason from error
            let reason = if e.to_string().contains("Unknown system") {
                "unknown_system"
            } else if e.to_string().contains("No path") || e.to_string().contains("no path") {
                "no_path"
            } else {
                "internal_error"
            };
            record_route_failed(reason, "route");
            return Response::Error(from_lib_error(&e, &request_id));
        }
    };

    // Convert system IDs to names
    let route: Vec<String> = plan
        .steps
        .iter()
        .filter_map(|&id| starmap.system_name(id).map(String::from))
        .collect();

    let algorithm_name = plan.algorithm.to_string();
    let hops = plan.hop_count();

    let response = RouteResponse {
        hops,
        gates: plan.gates,
        jumps: plan.jumps,
        algorithm: algorithm_name.clone(),
        route,
    };

    // Record business metrics
    record_route_calculated(&algorithm_name.to_lowercase(), "route");
    record_route_hops(hops, &algorithm_name.to_lowercase());

    info!(
        request_id = %request_id,
        hops = response.hops,
        gates = response.gates,
        jumps = response.jumps,
        "route computed successfully"
    );

    Response::Success(ServiceResponse::new(response))
}

/// Generate a unique request ID for tracing.
fn generate_request_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    format!("req-{:x}", timestamp)
}
