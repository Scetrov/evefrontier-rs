//! AWS Lambda function for scouting adjacent gates.
//!
//! This Lambda returns the gate-connected neighbours of a system.

use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::Serialize;
use serde_json::Value;
use tracing::{error, info};

use evefrontier_lambda_shared::{
    get_runtime, init_runtime, init_tracing, LambdaResponse, ProblemDetails, ScoutGatesRequest,
    Validate,
};

/// Bundled SQLite database (from data/static_data.db).

/// Bundled spatial index (from data/static_data.db.spatial.bin).
// Bundled dataset bytes (build-time include).
// In CI, large data artifacts under `data/` are not committed.
// Use a feature flag to optionally bundle real dataset; otherwise fall back to fixture.
#[cfg(feature = "bundle-data")]
static DB_BYTES: &[u8] = include_bytes!("../../../data/static_data.db");
#[cfg(not(feature = "bundle-data"))]
static DB_BYTES: &[u8] = &[];

// Spatial index bytes: when not bundling, provide empty slice to trigger runtime auto-build.
#[cfg(feature = "bundle-data")]
static INDEX_BYTES: &[u8] = include_bytes!("../../../data/static_data.db.spatial.bin");
#[cfg(not(feature = "bundle-data"))]
static INDEX_BYTES: &[u8] = &[];

/// Information about a neighboring system.
#[derive(Debug, Serialize)]
struct Neighbor {
    /// System name.
    name: String,
    /// System ID.
    id: i64,
}

/// Response for scout-gates endpoint.
#[derive(Debug, Serialize)]
struct ScoutGatesResponse {
    /// The queried system name.
    system: String,
    /// The queried system ID.
    system_id: i64,
    /// Number of gate-connected neighbors.
    count: usize,
    /// List of neighboring systems.
    neighbors: Vec<Neighbor>,
}

/// Lambda response - either success or RFC 9457 error.
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Response {
    Success(LambdaResponse<ScoutGatesResponse>),
    Error(ProblemDetails),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    init_tracing();

    // Initialize runtime with bundled data (logs cold-start timing)
    let _runtime = init_runtime(DB_BYTES, INDEX_BYTES);

    lambda_runtime::run(service_fn(handler)).await
}

async fn handler(event: LambdaEvent<Value>) -> Result<Response, Error> {
    let request_id = event.context.request_id.clone();

    // Parse the request
    let request: ScoutGatesRequest = match serde_json::from_value(event.payload) {
        Ok(req) => req,
        Err(e) => {
            error!(request_id = %request_id, error = %e, "failed to parse request");
            return Ok(Response::Error(ProblemDetails::bad_request(
                format!("Invalid request: {}", e),
                &request_id,
            )));
        }
    };

    info!(
        request_id = %request_id,
        system = %request.system,
        "handling scout-gates request"
    );

    // Validate the request
    if let Err(problem) = request.validate(&request_id) {
        return Ok(Response::Error(*problem));
    }

    let runtime = get_runtime();
    let starmap = runtime.starmap();

    // Look up the system
    let system_id = match starmap.system_id_by_name(&request.system) {
        Some(id) => id,
        None => {
            let suggestions = starmap.fuzzy_system_matches(&request.system, 3);
            return Ok(Response::Error(ProblemDetails::unknown_system(
                &request.system,
                &suggestions,
                &request_id,
            )));
        }
    };

    // Get gate-connected neighbors from adjacency list
    let neighbors: Vec<Neighbor> = starmap
        .adjacency
        .get(&system_id)
        .map(|ids| {
            ids.iter()
                .filter_map(|&id| {
                    starmap.system_name(id).map(|name| Neighbor {
                        name: name.to_string(),
                        id,
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
        neighbor_count = response.count,
        "gate neighbors found"
    );

    Ok(Response::Success(LambdaResponse::new(response)))
}
