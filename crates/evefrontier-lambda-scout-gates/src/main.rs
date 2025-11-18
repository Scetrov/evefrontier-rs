use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

/// Request structure for gate scouting.
/// TODO: Replace with actual scout-gates request model (see docs/TODO.md lines 110-123).
#[derive(Deserialize)]
struct ScoutGatesRequest {
    system: String,
}

/// Response structure for gate scouting.
/// TODO: Replace with actual scout-gates response model (see docs/TODO.md lines 110-123).
#[derive(Serialize)]
struct ScoutGatesResponse {
    message: String,
}

/// Lambda function handler.
/// TODO: Implement actual gate scouting logic using evefrontier_lib (see docs/TODO.md lines 110-123).
async fn function_handler(
    event: LambdaEvent<ScoutGatesRequest>,
) -> Result<ScoutGatesResponse, Error> {
    let (request, _context) = event.into_parts();

    // Placeholder response
    Ok(ScoutGatesResponse {
        message: format!(
            "Gate scouting for system '{}' not yet implemented",
            request.system
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
