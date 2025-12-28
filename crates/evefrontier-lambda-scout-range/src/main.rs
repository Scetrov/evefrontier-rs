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

#[cfg(test)]
mod tests {
    use super::*;
    use evefrontier_lambda_shared::{test_utils, ScoutRangeRequest, Validate};
    use evefrontier_lib::SpatialIndex;
    use serde_json::json;

    // Use shared test utilities for fixture loading to ensure consistency across Lambda test suites.
    fn fixture_starmap() -> &'static evefrontier_lib::Starmap {
        test_utils::fixture_starmap()
    }

    fn fixture_spatial_index() -> &'static SpatialIndex {
        test_utils::fixture_spatial_index()
    }

    // ==================== Request Parsing Tests ====================

    #[test]
    fn test_parse_valid_range_request() {
        let json = json!({
            "system": "Nod"
        });
        let request: ScoutRangeRequest = serde_json::from_value(json).unwrap();
        assert_eq!(request.system, "Nod");
        assert_eq!(request.limit, 10); // default
        assert!(request.radius.is_none());
        assert!(request.max_temperature.is_none());
    }

    #[test]
    fn test_parse_range_request_with_all_fields() {
        let json = json!({
            "system": "Brana",
            "limit": 5,
            "radius": 100.0,
            "max_temperature": 50000.0
        });
        let request: ScoutRangeRequest = serde_json::from_value(json).unwrap();
        assert_eq!(request.system, "Brana");
        assert_eq!(request.limit, 5);
        assert_eq!(request.radius, Some(100.0));
        assert_eq!(request.max_temperature, Some(50000.0));
    }

    #[test]
    fn test_parse_range_request_missing_system() {
        let json = json!({});
        let result: Result<ScoutRangeRequest, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    // ==================== Validation Tests ====================

    #[test]
    fn test_validate_valid_request() {
        let request = ScoutRangeRequest {
            system: "Nod".to_string(),
            limit: 10,
            radius: None,
            max_temperature: None,
        };
        assert!(request.validate("test-req").is_ok());
    }

    #[test]
    fn test_validate_empty_system() {
        let request = ScoutRangeRequest {
            system: "".to_string(),
            limit: 10,
            radius: None,
            max_temperature: None,
        };
        let err = request.validate("test-req").unwrap_err();
        assert_eq!(err.status, 400);
    }

    #[test]
    fn test_validate_limit_zero() {
        let request = ScoutRangeRequest {
            system: "Nod".to_string(),
            limit: 0,
            radius: None,
            max_temperature: None,
        };
        let err = request.validate("test-req").unwrap_err();
        assert_eq!(err.status, 400);
        assert!(err.detail.as_ref().is_some_and(|d| d.contains("limit")));
    }

    #[test]
    fn test_validate_limit_exceeds_max() {
        let request = ScoutRangeRequest {
            system: "Nod".to_string(),
            limit: 101,
            radius: None,
            max_temperature: None,
        };
        let err = request.validate("test-req").unwrap_err();
        assert_eq!(err.status, 400);
        assert!(err.detail.as_ref().is_some_and(|d| d.contains("100")));
    }

    #[test]
    fn test_validate_negative_radius() {
        let request = ScoutRangeRequest {
            system: "Nod".to_string(),
            limit: 10,
            radius: Some(-50.0),
            max_temperature: None,
        };
        let err = request.validate("test-req").unwrap_err();
        assert_eq!(err.status, 400);
        assert!(err.detail.as_ref().is_some_and(|d| d.contains("radius")));
    }

    #[test]
    fn test_validate_negative_temperature() {
        let request = ScoutRangeRequest {
            system: "Nod".to_string(),
            limit: 10,
            radius: None,
            max_temperature: Some(-100.0),
        };
        let err = request.validate("test-req").unwrap_err();
        assert_eq!(err.status, 400);
        assert!(err
            .detail
            .as_ref()
            .is_some_and(|d| d.contains("temperature")));
    }

    // ==================== Spatial Query Tests (using fixture) ====================

    #[test]
    fn test_spatial_index_build() {
        let index = fixture_spatial_index();

        // Fixture has 8 systems
        assert!(!index.is_empty(), "Spatial index should have entries");
    }

    #[test]
    fn test_nearest_systems_from_nod() {
        let starmap = fixture_starmap();
        let index = fixture_spatial_index();

        let system_id = starmap.system_id_by_name("Nod").expect("Nod exists");
        let system = starmap.systems.get(&system_id).expect("Nod in starmap");
        let position = system.position.expect("Nod has position");

        let query = NeighbourQuery {
            k: 5,
            radius: None,
            max_temperature: None,
        };
        let results = index.nearest_filtered([position.x, position.y, position.z], &query);

        // Should find at least some nearby systems
        assert!(!results.is_empty(), "Should find nearby systems");

        // First result should be Nod itself (distance ~0)
        let (first_id, first_distance) = results[0];
        assert_eq!(first_id, system_id, "First result should be origin system");
        assert!(
            first_distance < 0.001,
            "Origin distance should be near zero"
        );
    }

    #[test]
    fn test_nearest_systems_with_radius_filter() {
        let starmap = fixture_starmap();
        let index = fixture_spatial_index();

        let system_id = starmap.system_id_by_name("Nod").expect("Nod exists");
        let system = starmap.systems.get(&system_id).expect("Nod in starmap");
        let position = system.position.expect("Nod has position");

        // Small radius should return fewer systems
        let query_small = NeighbourQuery {
            k: 10,
            radius: Some(1.0), // Very small radius
            max_temperature: None,
        };
        let results_small =
            index.nearest_filtered([position.x, position.y, position.z], &query_small);

        let query_large = NeighbourQuery {
            k: 10,
            radius: Some(1000.0), // Large radius
            max_temperature: None,
        };
        let results_large =
            index.nearest_filtered([position.x, position.y, position.z], &query_large);

        // Large radius should find at least as many as small radius
        assert!(
            results_large.len() >= results_small.len(),
            "Large radius should find more or equal systems"
        );
    }

    #[test]
    fn test_lookup_unknown_system() {
        let starmap = fixture_starmap();
        let result = starmap.system_id_by_name("NonExistentSystem12345");
        assert!(result.is_none());
    }

    // ==================== Response Construction Tests ====================

    #[test]
    fn test_scout_range_response_serialization() {
        let response = ScoutRangeResponse {
            system: "Nod".to_string(),
            system_id: 12345,
            count: 2,
            systems: vec![
                NearbySystem {
                    name: "Brana".to_string(),
                    id: 54321,
                    distance_ly: 42.5,
                    min_temp_k: Some(3500.0),
                },
                NearbySystem {
                    name: "H:2L2S".to_string(),
                    id: 67890,
                    distance_ly: 78.3,
                    min_temp_k: None,
                },
            ],
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["system"], "Nod");
        assert_eq!(json["system_id"], 12345);
        assert_eq!(json["count"], 2);
        assert!(json["systems"].is_array());
        assert_eq!(json["systems"].as_array().unwrap().len(), 2);

        // Check first system
        assert_eq!(json["systems"][0]["name"], "Brana");
        assert_eq!(json["systems"][0]["distance_ly"], 42.5);
        assert_eq!(json["systems"][0]["min_temp_k"], 3500.0);

        // Check second system - min_temp_k should be omitted (skip_serializing_if)
        assert_eq!(json["systems"][1]["name"], "H:2L2S");
        assert!(json["systems"][1]["min_temp_k"].is_null());
    }

    #[test]
    fn test_response_enum_success_serialization() {
        let inner = ScoutRangeResponse {
            system: "Nod".to_string(),
            system_id: 1,
            count: 0,
            systems: vec![],
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
