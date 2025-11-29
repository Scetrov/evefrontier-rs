//! AWS Lambda function for scouting systems within range.
//!
//! This Lambda returns systems within a spatial radius of a given system.

use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::Serialize;
use serde_json::Value;
use tracing::{error, info};

use evefrontier_lambda_shared::{
    get_runtime, init_runtime, init_tracing, LambdaResponse, ProblemDetails, ScoutRangeRequest,
    Validate,
};
use evefrontier_lib::spatial::NeighbourQuery;

/// Bundled SQLite database (from data/static_data.db).
#[cfg(feature = "bundle-data")]
static DB_BYTES: &[u8] = include_bytes!("../../../data/static_data.db");
#[cfg(not(feature = "bundle-data"))]
static DB_BYTES: &[u8] = &[];

/// Bundled spatial index (from data/static_data.db.spatial.bin).
#[cfg(feature = "bundle-data")]
static INDEX_BYTES: &[u8] = include_bytes!("../../../data/static_data.db.spatial.bin");
#[cfg(not(feature = "bundle-data"))]
static INDEX_BYTES: &[u8] = &[];

/// Information about a system within range.
#[derive(Debug, Serialize)]
struct NearbySystem {
    /// System name.
    name: String,
    /// System ID.
    id: i64,
    /// Distance in light-years.
    distance_ly: f64,
    /// Minimum external temperature in Kelvin (if known).
    #[serde(skip_serializing_if = "Option::is_none")]
    min_temp_k: Option<f64>,
}

/// Response for scout-range endpoint.
#[derive(Debug, Serialize)]
struct ScoutRangeResponse {
    /// The queried system name.
    system: String,
    /// The queried system ID.
    system_id: i64,
    /// Number of systems found.
    count: usize,
    /// List of nearby systems ordered by distance.
    systems: Vec<NearbySystem>,
}

/// Lambda response - either success or RFC 9457 error.
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Response {
    Success(LambdaResponse<ScoutRangeResponse>),
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
    let request: ScoutRangeRequest = match serde_json::from_value(event.payload) {
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
        limit = request.limit,
        radius = ?request.radius,
        max_temp = ?request.max_temperature,
        "handling scout-range request"
    );

    // Validate the request
    if let Err(problem) = request.validate(&request_id) {
        return Ok(Response::Error(*problem));
    }

    let runtime = get_runtime();
    let starmap = runtime.starmap();
    let spatial_index = runtime.spatial_index();

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

    // Get the system's position
    let system = starmap
        .systems
        .get(&system_id)
        .ok_or_else(|| Error::from(format!("System {} found but not in starmap", system_id)))?;

    let position = match system.position {
        Some(pos) => [pos.x, pos.y, pos.z],
        None => {
            return Ok(Response::Error(ProblemDetails::bad_request(
                format!("System '{}' has no spatial coordinates", request.system),
                &request_id,
            )));
        }
    };

    // Build the query
    let query = NeighbourQuery {
        k: request.limit + 1, // +1 to exclude the origin system
        radius: request.radius,
        max_temperature: request.max_temperature,
    };

    // Find nearby systems
    let results = spatial_index.nearest_filtered(position, &query);

    // Convert to response, excluding the origin system
    let systems: Vec<NearbySystem> = results
        .into_iter()
        .filter(|(id, _)| *id != system_id)
        .take(request.limit)
        .filter_map(|(id, distance)| {
            let name = starmap.system_name(id)?;
            let min_temp_k = starmap
                .systems
                .get(&id)
                .and_then(|s| s.metadata.min_external_temp);
            Some(NearbySystem {
                name: name.to_string(),
                id,
                distance_ly: distance,
                min_temp_k,
            })
        })
        .collect();

    let response = ScoutRangeResponse {
        system: request.system.clone(),
        system_id,
        count: systems.len(),
        systems,
    };

    info!(
        request_id = %request_id,
        system = %request.system,
        systems_found = response.count,
        "range query completed"
    );

    Ok(Response::Success(LambdaResponse::new(response)))
}
