//! AWS Lambda function for route planning.
//!
//! This Lambda handles route requests between two systems, supporting
//! multiple algorithms and constraints.

use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::Serialize;
use serde_json::Value;
use tracing::{error, info};

use evefrontier_lambda_shared::{
    from_lib_error, get_runtime, init_runtime, init_tracing, LambdaResponse, ProblemDetails,
    RouteRequest, Validate,
};
use evefrontier_lib::{
    plan_route, RouteAlgorithm as LibAlgorithm, RouteConstraints as LibConstraints,
    RouteRequest as LibRequest,
};

/// Bundled SQLite database (from data/static_data.db).
static DB_BYTES: &[u8] = include_bytes!("../../../data/static_data.db");

/// Bundled spatial index (from data/static_data.db.spatial.bin).
static INDEX_BYTES: &[u8] = include_bytes!("../../../data/static_data.db.spatial.bin");

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

/// Lambda response - either success or RFC 9457 error.
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Response {
    Success(LambdaResponse<RouteResponse>),
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
    let request: RouteRequest = match serde_json::from_value(event.payload) {
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
        from = %request.from,
        to = %request.to,
        algorithm = ?request.algorithm,
        "handling route request"
    );

    // Validate the request
    if let Err(problem) = request.validate(&request_id) {
        return Ok(Response::Error(*problem));
    }

    let runtime = get_runtime();
    let starmap = runtime.starmap();

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
        },
    };

    // Plan the route
    let plan = match plan_route(starmap, &lib_request) {
        Ok(plan) => plan,
        Err(e) => {
            error!(request_id = %request_id, error = %e, "route planning failed");
            return Ok(Response::Error(from_lib_error(&e, &request_id)));
        }
    };

    // Convert system IDs to names
    let route: Vec<String> = plan
        .steps
        .iter()
        .filter_map(|&id| starmap.system_name(id).map(String::from))
        .collect();

    let response = RouteResponse {
        hops: plan.hop_count(),
        gates: plan.gates,
        jumps: plan.jumps,
        algorithm: plan.algorithm.to_string(),
        route,
    };

    info!(
        request_id = %request_id,
        hops = response.hops,
        gates = response.gates,
        jumps = response.jumps,
        "route computed successfully"
    );

    Ok(Response::Success(LambdaResponse::new(response)))
}
