use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

/// Request structure for range scouting.
/// TODO: Replace with actual scout-range request model (see docs/TODO.md lines 110-123).
#[derive(Deserialize)]
struct ScoutRangeRequest {
    system: String,
    range: f64,
}

/// Response structure for range scouting.
/// TODO: Replace with actual scout-range response model (see docs/TODO.md lines 110-123).
#[derive(Serialize)]
struct ScoutRangeResponse {
    message: String,
}

/// Lambda function handler.
/// TODO: Implement actual range scouting logic using evefrontier_lib (see docs/TODO.md lines 110-123).
async fn function_handler(
    event: LambdaEvent<ScoutRangeRequest>,
) -> Result<ScoutRangeResponse, Error> {
    let (request, _context) = event.into_parts();

    // Placeholder response
    Ok(ScoutRangeResponse {
        message: format!(
            "Range scouting for system '{}' within {} ly not yet implemented",
            request.system, request.range
        ),
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    lambda_runtime::run(service_fn(function_handler)).await
}
