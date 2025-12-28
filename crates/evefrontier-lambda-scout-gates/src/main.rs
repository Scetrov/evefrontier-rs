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

#[cfg(test)]
mod tests {
    use super::*;
    use evefrontier_lambda_shared::{ScoutGatesRequest, Validate};
    use evefrontier_lib::load_starmap;
    use serde_json::json;
    use std::path::PathBuf;

    fn fixture_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/minimal_static_data.db")
    }

    // ==================== Request Parsing Tests ====================

    #[test]
    fn test_parse_valid_gates_request() {
        let json = json!({
            "system": "Nod"
        });
        let request: ScoutGatesRequest = serde_json::from_value(json).unwrap();
        assert_eq!(request.system, "Nod");
    }

    #[test]
    fn test_parse_gates_request_missing_system() {
        let json = json!({});
        let result: Result<ScoutGatesRequest, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    // ==================== Validation Tests ====================

    #[test]
    fn test_validate_valid_request() {
        let request = ScoutGatesRequest {
            system: "Nod".to_string(),
        };
        assert!(request.validate("test-req").is_ok());
    }

    #[test]
    fn test_validate_empty_system() {
        let request = ScoutGatesRequest {
            system: "".to_string(),
        };
        let err = request.validate("test-req").unwrap_err();
        assert_eq!(err.status, 400);
    }

    #[test]
    fn test_validate_whitespace_only_system() {
        let request = ScoutGatesRequest {
            system: "   ".to_string(),
        };
        let err = request.validate("test-req").unwrap_err();
        assert_eq!(err.status, 400);
    }

    // ==================== Gate Lookup Tests (using fixture) ====================

    #[test]
    fn test_lookup_system_with_gates_nod() {
        let starmap = load_starmap(&fixture_path()).expect("fixture loads");
        let system_id = starmap.system_id_by_name("Nod").expect("Nod exists");

        // Get neighbors from adjacency list
        let neighbors = starmap.adjacency.get(&system_id);
        assert!(neighbors.is_some(), "Nod should have gate connections");

        let neighbor_ids = neighbors.unwrap();
        assert!(
            !neighbor_ids.is_empty(),
            "Nod should have at least one gate"
        );
    }

    #[test]
    fn test_lookup_system_brana_gates() {
        let starmap = load_starmap(&fixture_path()).expect("fixture loads");
        let system_id = starmap.system_id_by_name("Brana").expect("Brana exists");

        // Get neighbors from adjacency list
        let neighbors = starmap.adjacency.get(&system_id);
        assert!(neighbors.is_some(), "Brana should have gate connections");
    }

    #[test]
    fn test_lookup_unknown_system() {
        let starmap = load_starmap(&fixture_path()).expect("fixture loads");
        let result = starmap.system_id_by_name("NonExistentSystem12345");
        assert!(result.is_none());
    }

    #[test]
    fn test_fuzzy_suggestions_for_unknown_system() {
        let starmap = load_starmap(&fixture_path()).expect("fixture loads");
        let suggestions = starmap.fuzzy_system_matches("Nodd", 3); // typo of "Nod"
                                                                   // Should suggest "Nod" as a close match
        assert!(
            !suggestions.is_empty(),
            "Should provide suggestions for typos"
        );
    }

    #[test]
    fn test_neighbor_resolution() {
        let starmap = load_starmap(&fixture_path()).expect("fixture loads");
        let system_id = starmap.system_id_by_name("Nod").expect("Nod exists");

        // Get neighbors and resolve their names
        if let Some(neighbor_ids) = starmap.adjacency.get(&system_id) {
            for &neighbor_id in neighbor_ids {
                let name = starmap.system_name(neighbor_id);
                assert!(
                    name.is_some(),
                    "All neighbor IDs should resolve to system names"
                );
            }
        }
    }

    // ==================== Response Construction Tests ====================

    #[test]
    fn test_scout_gates_response_serialization() {
        let response = ScoutGatesResponse {
            system: "Nod".to_string(),
            system_id: 12345,
            count: 2,
            neighbors: vec![
                Neighbor {
                    name: "Brana".to_string(),
                    id: 54321,
                },
                Neighbor {
                    name: "H:2L2S".to_string(),
                    id: 67890,
                },
            ],
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["system"], "Nod");
        assert_eq!(json["system_id"], 12345);
        assert_eq!(json["count"], 2);
        assert!(json["neighbors"].is_array());
        assert_eq!(json["neighbors"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_response_enum_success_serialization() {
        let inner = ScoutGatesResponse {
            system: "Nod".to_string(),
            system_id: 1,
            count: 0,
            neighbors: vec![],
        };
        let response = Response::Success(LambdaResponse::new(inner));
        let json = serde_json::to_value(&response).unwrap();

        // LambdaResponse uses #[serde(flatten)] on data, so fields are at root level
        assert_eq!(json["content_type"], "application/json");
        assert_eq!(json["system"], "Nod");
        assert_eq!(json["system_id"], 1);
    }

    #[test]
    fn test_response_enum_error_serialization() {
        let error = ProblemDetails::unknown_system("BadSystem", &["Nod".to_string()], "req-123");
        let response = Response::Error(error);
        let json = serde_json::to_value(&response).unwrap();

        assert_eq!(json["status"], 404);
        assert!(json["title"].as_str().unwrap().contains("Unknown"));
    }
}
