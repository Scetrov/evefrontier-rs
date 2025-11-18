use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

/// Request structure for route computation.
/// TODO: Replace with actual route request model (see docs/TODO.md lines 110-123).
#[derive(Deserialize)]
struct RouteRequest {
    from: String,
    to: String,
}

/// Response structure for route computation.
/// TODO: Replace with actual route response model (see docs/TODO.md lines 110-123).
#[derive(Serialize)]
struct RouteResponse {
    message: String,
}

/// Lambda function handler.
/// TODO: Implement actual routing logic using evefrontier_lib::plan_route (see docs/TODO.md lines 110-123).
async fn function_handler(event: LambdaEvent<RouteRequest>) -> Result<RouteResponse, Error> {
    let (request, _context) = event.into_parts();

    // Placeholder response
    Ok(RouteResponse {
        message: format!(
            "Route computation from '{}' to '{}' not yet implemented",
            request.from, request.to
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
