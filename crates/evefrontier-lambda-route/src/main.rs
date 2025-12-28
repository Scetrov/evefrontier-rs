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
#[cfg(feature = "bundle-data")]
static DB_BYTES: &[u8] = include_bytes!("../../../data/static_data.db");
#[cfg(not(feature = "bundle-data"))]
static DB_BYTES: &[u8] = &[];

/// Bundled spatial index (from data/static_data.db.spatial.bin).
#[cfg(feature = "bundle-data")]
static INDEX_BYTES: &[u8] = include_bytes!("../../../data/static_data.db.spatial.bin");
#[cfg(not(feature = "bundle-data"))]
static INDEX_BYTES: &[u8] = &[];

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

#[cfg(test)]
mod tests {
    use super::*;
    use evefrontier_lambda_shared::{
        test_utils, RouteAlgorithm as LambdaRouteAlgorithm, RouteRequest, Validate,
    };
    use evefrontier_lib::{
        plan_route, RouteAlgorithm as LibRouteAlgorithm, RouteConstraints,
        RouteRequest as LibRequest,
    };
    use serde_json::json;

    // Use shared test utilities for fixture loading to ensure consistency across Lambda test suites.
    fn fixture_starmap() -> &'static evefrontier_lib::Starmap {
        test_utils::fixture_starmap()
    }

    // ==================== Request Parsing Tests ====================

    #[test]
    fn test_parse_valid_route_request() {
        let json = json!({
            "from": "Nod",
            "to": "Brana",
            "algorithm": "a-star"
        });
        let request: RouteRequest = serde_json::from_value(json).unwrap();
        assert_eq!(request.from, "Nod");
        assert_eq!(request.to, "Brana");
    }

    #[test]
    fn test_parse_route_request_with_constraints() {
        let json = json!({
            "from": "Nod",
            "to": "Brana",
            "algorithm": "dijkstra",
            "max_jump": 50.0,
            "avoid": ["System1"],
            "avoid_gates": true,
            "max_temperature": 100.0
        });
        let request: RouteRequest = serde_json::from_value(json).unwrap();
        assert_eq!(request.max_jump, Some(50.0));
        assert!(request.avoid_gates);
        assert_eq!(request.max_temperature, Some(100.0));
    }

    #[test]
    fn test_parse_route_request_invalid_json() {
        let json = json!({
            "from": "Nod"
            // missing "to" field
        });
        let result: Result<RouteRequest, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    // ==================== Validation Tests ====================

    #[test]
    fn test_validate_valid_request() {
        let request = RouteRequest {
            from: "Nod".to_string(),
            to: "Brana".to_string(),
            algorithm: LambdaRouteAlgorithm::AStar,
            max_jump: None,
            avoid: vec![],
            avoid_gates: false,
            max_temperature: None,
        };
        assert!(request.validate("test-req").is_ok());
    }

    #[test]
    fn test_validate_empty_from() {
        let request = RouteRequest {
            from: "".to_string(),
            to: "Brana".to_string(),
            algorithm: LambdaRouteAlgorithm::Bfs,
            max_jump: None,
            avoid: vec![],
            avoid_gates: false,
            max_temperature: None,
        };
        let err = request.validate("test-req").unwrap_err();
        assert_eq!(err.status, 400);
    }

    #[test]
    fn test_validate_empty_to() {
        let request = RouteRequest {
            from: "Nod".to_string(),
            to: "   ".to_string(), // whitespace only
            algorithm: LambdaRouteAlgorithm::Bfs,
            max_jump: None,
            avoid: vec![],
            avoid_gates: false,
            max_temperature: None,
        };
        let err = request.validate("test-req").unwrap_err();
        assert_eq!(err.status, 400);
    }

    // ==================== Route Planning Tests (using fixture) ====================

    #[test]
    fn test_plan_route_nod_to_brana_bfs() {
        let starmap = fixture_starmap();
        let request = LibRequest::bfs("Nod", "Brana");
        let plan = plan_route(starmap, &request).expect("route exists");

        assert!(plan.hop_count() >= 1);
        assert_eq!(plan.algorithm, LibRouteAlgorithm::Bfs);
    }

    #[test]
    fn test_plan_route_nod_to_brana_dijkstra() {
        let starmap = fixture_starmap();
        let request = LibRequest {
            start: "Nod".to_string(),
            goal: "Brana".to_string(),
            algorithm: LibRouteAlgorithm::Dijkstra,
            constraints: RouteConstraints::default(),
        };
        let plan = plan_route(starmap, &request).expect("route exists");

        assert!(plan.hop_count() >= 1);
        assert_eq!(plan.algorithm, LibRouteAlgorithm::Dijkstra);
    }

    #[test]
    fn test_plan_route_nod_to_brana_astar() {
        let starmap = fixture_starmap();
        let request = LibRequest {
            start: "Nod".to_string(),
            goal: "Brana".to_string(),
            algorithm: LibRouteAlgorithm::AStar,
            constraints: RouteConstraints::default(),
        };
        let plan = plan_route(starmap, &request).expect("route exists");

        assert!(plan.hop_count() >= 1);
        assert_eq!(plan.algorithm, LibRouteAlgorithm::AStar);
    }

    #[test]
    fn test_plan_route_with_max_jump() {
        let starmap = fixture_starmap();
        let request = LibRequest {
            start: "Nod".to_string(),
            goal: "Brana".to_string(),
            algorithm: LibRouteAlgorithm::AStar,
            constraints: RouteConstraints {
                max_jump: Some(300.0),
                avoid_gates: true,
                ..RouteConstraints::default()
            },
        };
        let plan = plan_route(starmap, &request).expect("route exists");

        // Should find a spatial route
        assert!(plan.hop_count() >= 1);
    }

    #[test]
    fn test_plan_route_unknown_system() {
        let starmap = fixture_starmap();
        let request = LibRequest::bfs("NonExistentSystem12345", "Brana");
        let result = plan_route(starmap, &request);

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{}", err).to_lowercase();
        assert!(
            err_msg.contains("not found") || err_msg.contains("unknown"),
            "Expected error about unknown system, got: {}",
            err
        );
    }

    #[test]
    fn test_plan_route_avoided_goal() {
        let starmap = fixture_starmap();
        let request = LibRequest {
            start: "Nod".to_string(),
            goal: "Brana".to_string(),
            algorithm: LibRouteAlgorithm::Bfs,
            constraints: RouteConstraints {
                avoid_systems: vec!["Brana".to_string()],
                ..RouteConstraints::default()
            },
        };
        let result = plan_route(starmap, &request);

        // Should fail since goal is avoided
        assert!(result.is_err());
    }

    // ==================== Response Construction Tests ====================

    #[test]
    fn test_route_response_serialization() {
        let response = RouteResponse {
            hops: 3,
            gates: 2,
            jumps: 1,
            algorithm: "A*".to_string(),
            route: vec![
                "Nod".to_string(),
                "System1".to_string(),
                "Brana".to_string(),
            ],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"hops\":3"));
        assert!(json.contains("\"gates\":2"));
        assert!(json.contains("\"jumps\":1"));
        assert!(json.contains("\"algorithm\":\"A*\""));
        assert!(json.contains("\"route\":["));
    }

    #[test]
    fn test_response_enum_success_serialization() {
        let route_response = RouteResponse {
            hops: 1,
            gates: 1,
            jumps: 0,
            algorithm: "BFS".to_string(),
            route: vec!["Nod".to_string(), "Brana".to_string()],
        };
        let response = Response::Success(LambdaResponse::new(route_response));

        let json = serde_json::to_string(&response).unwrap();
        // Untagged enum means success response is just the inner data
        assert!(json.contains("\"hops\":1"));
        assert!(json.contains("\"content_type\":\"application/json\""));
    }

    #[test]
    fn test_response_enum_error_serialization() {
        let response = Response::Error(ProblemDetails::bad_request(
            "Test error message",
            "test-request-id",
        ));

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":400"));
        assert!(json.contains("\"type\":\"/problems/invalid-request\""));
    }
}
