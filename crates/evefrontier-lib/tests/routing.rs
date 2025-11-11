use std::path::PathBuf;

use evefrontier_lib::{load_starmap, plan_route, RouteAlgorithm, RouteConstraints, RouteRequest};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/minimal_static_data.db")
}

#[test]
fn bfs_route_plan_succeeds() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let request = RouteRequest::bfs("Y:170N", "BetaTest");
    let plan = plan_route(&starmap, &request).expect("route exists");

    assert_eq!(plan.algorithm, RouteAlgorithm::Bfs);
    assert_eq!(plan.start, starmap.system_id_by_name("Y:170N").unwrap());
    assert_eq!(plan.goal, starmap.system_id_by_name("BetaTest").unwrap());
    assert!(plan.hop_count() >= 1);
}

#[test]
fn unsupported_algorithm_is_rejected() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let request = RouteRequest {
        start: "Y:170N".to_string(),
        goal: "BetaTest".to_string(),
        algorithm: RouteAlgorithm::Dijkstra,
        constraints: RouteConstraints::default(),
    };

    let error = plan_route(&starmap, &request).expect_err("unsupported option");
    let message = format!("{error}");
    assert!(message.contains("algorithm dijkstra"));
}

#[test]
fn constraints_are_flagged() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let request = RouteRequest {
        start: "Y:170N".to_string(),
        goal: "BetaTest".to_string(),
        algorithm: RouteAlgorithm::Bfs,
        constraints: RouteConstraints {
            max_jump: Some(5.0),
            ..RouteConstraints::default()
        },
    };

    let error = plan_route(&starmap, &request).expect_err("unsupported constraint");
    assert!(format!("{error}").contains("--max-jump"));
}
