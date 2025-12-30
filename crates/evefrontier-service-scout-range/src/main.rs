//! EVE Frontier spatial range scout HTTP microservice.
//!
//! This service provides a REST API for finding systems within spatial range
//! of a given system.
//!
//! # Endpoints
//!
//! - `POST /api/v1/scout/range` - Find systems within spatial range
//! - `GET /health/live` - Kubernetes liveness probe
//! - `GET /health/ready` - Kubernetes readiness probe
//!
//! # Configuration
//!
//! - `EVEFRONTIER_DATA_PATH` - Path to the static_data.db file (required)
//! - `RUST_LOG` - Log level (default: info)
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
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};

use evefrontier_lib::spatial::NeighbourQuery;
use evefrontier_service_shared::{
    AppState, ProblemDetails, ScoutRangeRequest, ServiceResponse, Validate, health_live,
    health_ready,
};

/// Nearby system information.
#[derive(Debug, Serialize)]
struct NearbySystem {
    /// System ID.
    id: i64,
    /// System name.
    name: String,
    /// Distance in light-years.
    distance_ly: f64,
}

/// Scout range response returned to the caller.
#[derive(Debug, Serialize)]
struct ScoutRangeResponse {
    /// The queried system name.
    system: String,
    /// System ID.
    system_id: i64,
    /// Number of nearby systems found.
    count: usize,
    /// List of nearby systems, sorted by distance.
    nearby: Vec<NearbySystem>,
}

/// HTTP response - either success or RFC 9457 error.
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Response {
    Success(ServiceResponse<ScoutRangeResponse>),
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

/// Initialize tracing with JSON formatting for production.
fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().json())
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    // Load configuration from environment
    let data_path =
        env::var("EVEFRONTIER_DATA_PATH").unwrap_or_else(|_| "/data/static_data.db".to_string());
    let port: u16 = env::var("SERVICE_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    info!(data_path = %data_path, port = port, "starting scout-range service");

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
        .route("/api/v1/scout/range", post(scout_range_handler))
        .route("/health/live", get(health_live))
        .route("/health/ready", get(health_ready))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Bind and serve
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(addr = %addr, "listening on");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Handle POST /api/v1/scout/range requests.
async fn scout_range_handler(
    State(state): State<AppState>,
    Json(request): Json<ScoutRangeRequest>,
) -> Response {
    // Generate a request ID for tracing
    let request_id = generate_request_id();

    info!(
        request_id = %request_id,
        system = %request.system,
        limit = request.limit,
        radius = ?request.radius,
        max_temperature = ?request.max_temperature,
        "handling scout range request"
    );

    // Validate the request
    if let Err(problem) = request.validate(&request_id) {
        return Response::Error(*problem);
    }

    let starmap = state.starmap();

    // Look up the system
    let system_id = match starmap.name_to_id.get(&request.system) {
        Some(&id) => id,
        None => {
            // Try fuzzy matching
            let suggestions = starmap.fuzzy_system_matches(&request.system, 3);
            return Response::Error(ProblemDetails::unknown_system(
                &request.system,
                &suggestions,
                &request_id,
            ));
        }
    };

    // Get the system position for spatial query
    let system = match starmap.systems.get(&system_id) {
        Some(sys) => sys,
        None => {
            return Response::Error(ProblemDetails::internal_error(
                "System found in name index but not in systems map",
                &request_id,
            ));
        }
    };

    let position = match &system.position {
        Some(pos) => [pos.x, pos.y, pos.z],
        None => {
            return Response::Error(ProblemDetails::internal_error(
                format!("System '{}' has no position data", request.system),
                &request_id,
            ));
        }
    };

    // Check for spatial index
    let spatial_index = match state.spatial_index() {
        Some(idx) => idx,
        None => {
            warn!(
                request_id = %request_id,
                "spatial index not available for range query"
            );
            return Response::Error(ProblemDetails::service_unavailable(
                "Spatial index not available. Range queries require a precomputed spatial index.",
                &request_id,
            ));
        }
    };

    // Build the query
    let query = NeighbourQuery {
        k: request.limit,
        radius: request.radius,
        max_temperature: request.max_temperature,
    };

    // Query the spatial index with the system's position
    let results = spatial_index.nearest_filtered(position, &query);

    // Convert results to response (filter out the queried system itself)
    let nearby: Vec<NearbySystem> = results
        .into_iter()
        .filter(|(id, _)| *id != system_id) // Exclude the queried system
        .filter_map(|(id, distance)| {
            starmap.system_name(id).map(|name| NearbySystem {
                id,
                name: name.to_string(),
                distance_ly: distance,
            })
        })
        .collect();

    let response = ScoutRangeResponse {
        system: request.system.clone(),
        system_id,
        count: nearby.len(),
        nearby,
    };

    info!(
        request_id = %request_id,
        system = %request.system,
        found = response.count,
        "scout range completed"
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
