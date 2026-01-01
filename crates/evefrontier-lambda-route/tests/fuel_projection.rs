use evefrontier_lambda_route::Response;
use evefrontier_lambda_shared::test_utils::{
    fixture_db_bytes, fixture_index_bytes, fixture_ship_bytes,
};
use evefrontier_lambda_shared::{init_runtime, RouteRequest};
use lambda_runtime::{Context, LambdaEvent};

fn init_fixture_runtime() {
    let _ = init_runtime(
        fixture_db_bytes(),
        fixture_index_bytes(),
        fixture_ship_bytes(),
    );
}

async fn invoke(request: RouteRequest) -> Response {
    let context = Context::default();
    let payload = serde_json::to_value(request).expect("serializable payload");
    let event = LambdaEvent::new(payload, context);
    evefrontier_lambda_route::handler(event)
        .await
        .expect("handler should succeed")
}

#[tokio::test]
async fn returns_fuel_projection_when_ship_provided() {
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
        fuel_quality: Some(10.0),
        cargo_mass: Some(1000.0),
        fuel_load: Some(1750.0),
        dynamic_mass: Some(false),
    };

    let response = invoke(request).await;

    match response {
        Response::Success(success) => {
            let summary = success.data.summary;
            assert!(summary.fuel.is_some(), "fuel summary should be present");
            assert!(success.data.steps.iter().any(|s| s.fuel.is_some()));
        }
        Response::Error(err) => panic!("unexpected error: {err:?}"),
    }
}

#[tokio::test]
async fn omits_fuel_projection_without_ship() {
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

    let response = invoke(request).await;

    match response {
        Response::Success(success) => {
            let summary = success.data.summary;
            assert!(summary.fuel.is_none());
            assert!(success.data.steps.iter().all(|s| s.fuel.is_none()));
        }
        Response::Error(err) => panic!("unexpected error: {err:?}"),
    }
}

#[tokio::test]
async fn rejects_unknown_ship_name() {
    init_fixture_runtime();

    let request = RouteRequest {
        from: "Nod".to_string(),
        to: "Brana".to_string(),
        algorithm: evefrontier_lambda_shared::RouteAlgorithm::AStar,
        max_jump: None,
        avoid: vec![],
        avoid_gates: false,
        max_temperature: None,
        ship: Some("UnknownShip".to_string()),
        fuel_quality: Some(10.0),
        cargo_mass: None,
        fuel_load: None,
        dynamic_mass: None,
    };

    let response = invoke(request).await;

    match response {
        Response::Success(_) => panic!("expected validation error for unknown ship"),
        Response::Error(err) => assert_eq!(err.status, 400),
    }
}
