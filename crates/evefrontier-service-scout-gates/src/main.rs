//! EVE Frontier gate scout HTTP microservice.
//!
//! This service provides a REST API for finding gate-connected neighbors
//! of a solar system.
//!
//! # Endpoints
//!
//! - `POST /api/v1/scout/gates` - Find gate-connected neighbors
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
use tracing::{error, info};

use evefrontier_service_shared::{
    AppState, ProblemDetails, ScoutGatesRequest, ServiceResponse, Validate, health_live,
    health_ready,
};

/// Gate neighbor information.
#[derive(Debug, Serialize)]
struct GateNeighbor {
    /// System ID.
    id: i64,
    /// System name.
    name: String,
}

/// Scout gates response returned to the caller.
#[derive(Debug, Serialize)]
struct ScoutGatesResponse {
    /// The queried system name.
    system: String,
    /// System ID.
    system_id: i64,
    /// Number of gate-connected neighbors.
    count: usize,
    /// List of gate-connected neighbors.
    neighbors: Vec<GateNeighbor>,
}

/// HTTP response - either success or RFC 9457 error.
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Response {
    Success(ServiceResponse<ScoutGatesResponse>),
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

    info!(data_path = %data_path, port = port, "starting scout-gates service");

    // Load application state
    let state = AppState::load(&data_path).map_err(|e| {
        error!(error = %e, path = %data_path, "failed to load application state");
        e
    })?;

    info!(
        systems = state.starmap().systems.len(),
        "application state loaded"
    );

    // Build the router
    let app = Router::new()
        .route("/api/v1/scout/gates", post(scout_gates_handler))
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

/// Handle POST /api/v1/scout/gates requests.
async fn scout_gates_handler(
    State(state): State<AppState>,
    Json(request): Json<ScoutGatesRequest>,
) -> Response {
    // Generate a request ID for tracing
    let request_id = generate_request_id();

    info!(
        request_id = %request_id,
        system = %request.system,
        "handling scout gates request"
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

    // Get gate-connected neighbors from adjacency list
    let neighbors: Vec<GateNeighbor> = starmap
        .adjacency
        .get(&system_id)
        .map(|ids| {
            ids.iter()
                .filter_map(|&id| {
                    starmap.system_name(id).map(|name| GateNeighbor {
                        id,
                        name: name.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let response = ScoutGatesResponse {
        system: request.system.clone(),
        system_id,
        count: neighbors.len(),
        neighbors,
    };

    info!(
        request_id = %request_id,
        system = %request.system,
        neighbors = response.count,
        "scout gates completed"
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
