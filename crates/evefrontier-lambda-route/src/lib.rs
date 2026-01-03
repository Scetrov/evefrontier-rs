mod models;

use std::env;
use std::fs;
use std::io::Cursor;
use std::sync::OnceLock;

use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::Value;
use tracing::{error, info};

use evefrontier_lambda_shared::{
    from_lib_error, get_runtime, init_runtime, init_tracing, LambdaResponse, ProblemDetails,
    RouteRequest, Validate,
};
use evefrontier_lib::output::{RouteOutputKind, RouteSummary};
use evefrontier_lib::ship::{FuelConfig, ShipCatalog, ShipLoadout};
use evefrontier_lib::{
    plan_route, Error as LibError, RouteAlgorithm as LibAlgorithm,
    RouteConstraints as LibConstraints, RouteRequest as LibRequest,
};

pub use models::{FuelProjectionDto, FuelSummaryDto, RouteResponseDto, RouteStepDto};

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

/// Bundled ship data (from data/ship_data.csv). Enable via `bundle-ship-data`.
#[cfg(feature = "bundle-ship-data")]
static SHIP_DATA_BYTES: &[u8] = include_bytes!("../../../data/ship_data.csv");
#[cfg(not(feature = "bundle-ship-data"))]
static SHIP_DATA_BYTES: &[u8] = &[];

static SHIP_CATALOG: OnceLock<Result<ShipCatalog, LibError>> = OnceLock::new();

/// Lambda response - either success or RFC 9457 error.
#[derive(Debug, serde::Serialize)]
#[serde(untagged)]
pub enum Response {
    Success(LambdaResponse<RouteResponseDto>),
    Error(ProblemDetails),
}

/// Entry point used by the Lambda runtime.
pub async fn run() -> Result<(), Error> {
    init_tracing();

    // Initialize runtime with bundled data (logs cold-start timing)
    let _runtime = init_runtime(DB_BYTES, INDEX_BYTES, SHIP_DATA_BYTES);

    lambda_runtime::run(service_fn(handler)).await
}

/// Lambda handler invoked per request.
pub async fn handler(event: LambdaEvent<Value>) -> Result<Response, Error> {
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
        ship = ?request.ship,
        "handling route request"
    );

    // Validate the request
    if let Err(problem) = request.validate(&request_id) {
        return Ok(Response::Error(*problem));
    }

    Ok(handle_route_request(&request, &request_id))
}

/// Core handler logic separated for reuse in tests.
fn handle_route_request(request: &RouteRequest, request_id: &str) -> Response {
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
            // NOTE: This Lambda currently does not expose `avoid_critical_state` and
            // therefore performs heat-unaware planning. If we want parity with the
            // CLI, add the field to the API, validate it, and propagate it here. See
            // TODO: create follow-up issue if API exposure is desired.
            avoid_critical_state: false,
            ship: None,
            loadout: None,
            heat_config: None,
        },
        spatial_index: Some(runtime.spatial_index_arc()),
    };

    // Plan the route
    let plan = match plan_route(starmap, &lib_request) {
        Ok(plan) => plan,
        Err(e) => {
            error!(request_id = %request_id, error = %e, "route planning failed");
            return Response::Error(from_lib_error(&e, request_id));
        }
    };

    let mut summary = match RouteSummary::from_plan(RouteOutputKind::Route, starmap, &plan) {
        Ok(summary) => summary,
        Err(e) => return Response::Error(from_lib_error(&e, request_id)),
    };

    if let Some(ship_name) = request.ship.as_ref() {
        let ship_name_trimmed = ship_name.trim();
        if ship_name_trimmed.is_empty() {
            return Response::Error(ProblemDetails::bad_request(
                "ship name cannot be empty",
                request_id,
            ));
        }

        let catalog = match ship_catalog() {
            Ok(cat) => cat,
            Err(err) => {
                return Response::Error(from_lib_error(err, request_id));
            }
        };

        let ship = match catalog.get(ship_name_trimmed) {
            Some(ship) => ship,
            None => {
                return Response::Error(ProblemDetails::bad_request(
                    format!("ship '{}' not found in catalog", ship_name_trimmed),
                    request_id,
                ))
            }
        };

        let fuel_load = request.fuel_load.unwrap_or(ship.fuel_capacity);
        let cargo_mass = request.cargo_mass.unwrap_or(0.0);

        let loadout = match ShipLoadout::new(ship, fuel_load, cargo_mass) {
            Ok(loadout) => loadout,
            Err(err) => {
                return Response::Error(ProblemDetails::bad_request(
                    format!("invalid ship loadout: {}", err),
                    request_id,
                ))
            }
        };

        let fuel_config = FuelConfig {
            quality: request.fuel_quality.unwrap_or(10.0),
            dynamic_mass: request.dynamic_mass.unwrap_or(false),
        };

        if let Err(err) = summary.attach_fuel(ship, &loadout, &fuel_config) {
            return Response::Error(from_lib_error(&err, request_id));
        }
        // Attach heat projections mirroring fuel calculations
        let heat_config = evefrontier_lib::ship::HeatConfig {
            // Fixed calibration constant; API no longer accepts overrides.
            calibration_constant: 1e-7,
            dynamic_mass: request.dynamic_mass.unwrap_or(false),
        };

        if let Err(err) = summary.attach_heat(ship, &loadout, &heat_config) {
            return Response::Error(from_lib_error(&err, request_id));
        }
    }

    let response = RouteResponseDto::from_summary(&summary);

    info!(
        request_id = %request_id,
        hops = response.summary.hops,
        gates = response.summary.gates,
        jumps = response.summary.jumps,
        fuel_total = response.summary.fuel.as_ref().map(|f| f.total),
        "route computed successfully"
    );

    Response::Success(LambdaResponse::new(response))
}

fn ship_catalog() -> Result<&'static ShipCatalog, &'static LibError> {
    // Prefer a catalog loaded at runtime (cold-start). This supports Lambda
    // bundling where the shared runtime pre-parsed the CSV into memory.
    if let Ok(runtime) = std::panic::catch_unwind(get_runtime) {
        if let Some(catalog) = runtime.ship_catalog() {
            return Ok(catalog);
        }
    }

    // Otherwise, lazily initialize a per-crate catalog from bundled bytes or
    // the `EVEFRONTIER_SHIP_DATA` env var. This preserves previous behaviour
    // for tests and non-Lambda execution.
    let result = SHIP_CATALOG.get_or_init(|| {
        if !SHIP_DATA_BYTES.is_empty() {
            return ShipCatalog::from_reader(Cursor::new(SHIP_DATA_BYTES));
        }

        if let Ok(path) = env::var("EVEFRONTIER_SHIP_DATA") {
            match fs::read(&path) {
                Ok(bytes) => return ShipCatalog::from_reader(Cursor::new(bytes)),
                Err(err) => {
                    return Err(LibError::ShipDataValidation {
                        message: format!(
                            "failed to read ship data from EVEFRONTIER_SHIP_DATA ({}): {}",
                            path, err
                        ),
                    });
                }
            }
        }

        Err(LibError::ShipDataValidation {
            message: "ship data not bundled; enable 'bundle-ship-data' feature or set EVEFRONTIER_SHIP_DATA"
                .to_string(),
        })
    });

    match result {
        Ok(catalog) => Ok(catalog),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use evefrontier_lambda_shared::test_utils::{self, mock_request_id};

    fn init_fixture_runtime() {
        // Initialize the runtime with fixture data for tests.
        let _ = init_runtime(
            test_utils::fixture_db_bytes(),
            test_utils::fixture_index_bytes(),
            test_utils::fixture_ship_bytes(),
        );
    }

    #[tokio::test]
    async fn parses_and_validates_request() {
        init_fixture_runtime();

        let request = RouteRequest {
            from: "Nod".to_string(),
            to: "Brana".to_string(),
            algorithm: evefrontier_lambda_shared::RouteAlgorithm::AStar,
            max_jump: None,
            avoid: vec![],
            avoid_gates: false,
            max_temperature: None,
            ship: None,
            fuel_quality: None,
            cargo_mass: None,
            fuel_load: None,
            dynamic_mass: None,
        };

        let response = handle_route_request(&request, &mock_request_id("validate"));
        match response {
            Response::Success(inner) => {
                assert!(inner.data.summary.hops >= 1);
                assert!(inner.data.summary.fuel.is_none());
            }
            Response::Error(err) => panic!("unexpected error: {err:?}"),
        }
    }

    #[tokio::test]
    async fn response_includes_heat_when_ship_provided() {
        init_fixture_runtime();

        let request = RouteRequest {
            from: "Nod".to_string(),
            to: "Brana".to_string(),
            algorithm: evefrontier_lambda_shared::RouteAlgorithm::AStar,
            max_jump: None,
            avoid: vec![],
            avoid_gates: false,
            max_temperature: None,
            ship: Some("Reflex".to_string()),
            fuel_quality: None,
            cargo_mass: Some(633_006.0),
            fuel_load: None,
            dynamic_mass: None,
        };

        let response = handle_route_request(&request, &mock_request_id("heat"));
        match response {
            Response::Success(inner) => {
                // Summary should include heat
                assert!(inner.data.summary.heat.is_some());
                // At least one step should include heat projection
                assert!(inner.data.steps.iter().any(|s| s.heat.is_some()));
            }
            Response::Error(err) => panic!("unexpected error: {err:?}"),
        }
    }

    #[test]
    fn ship_catalog_loads_from_fixture() {
        let catalog = ship_catalog().expect("catalog should load");
        assert!(!catalog.ship_names().is_empty());
    }
}
