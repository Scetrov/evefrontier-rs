//! AWS Lambda function for route planning with optional fuel projection.

use evefrontier_lambda_route::run;

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    run().await
}
