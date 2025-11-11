use std::path::PathBuf;

use evefrontier_lib::{
    load_starmap, plan_route, RouteAlgorithm, RouteConstraints, RouteRequest, SystemPosition,
};

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
fn dijkstra_route_plan_succeeds() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let request = RouteRequest {
        start: "Y:170N".to_string(),
        goal: "BetaTest".to_string(),
        algorithm: RouteAlgorithm::Dijkstra,
        constraints: RouteConstraints::default(),
    };

    let plan = plan_route(&starmap, &request).expect("route exists");
    assert_eq!(plan.algorithm, RouteAlgorithm::Dijkstra);
}

#[test]
fn a_star_respects_max_jump_constraint() {
    let mut starmap = load_starmap(&fixture_path()).expect("fixture loads");
    inject_positions(&mut starmap);
    let request = RouteRequest {
        start: "Y:170N".to_string(),
        goal: "BetaTest".to_string(),
        algorithm: RouteAlgorithm::AStar,
        constraints: RouteConstraints {
            max_jump: Some(15.0),
            avoid_gates: true,
            ..RouteConstraints::default()
        },
    };

    let plan = plan_route(&starmap, &request).expect("route exists");
    assert_eq!(plan.algorithm, RouteAlgorithm::AStar);
    assert!(
        plan.steps.len() >= 3,
        "route should include intermediate hop"
    );
}

#[test]
fn avoided_goal_rejects_route() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let request = RouteRequest {
        start: "Y:170N".to_string(),
        goal: "BetaTest".to_string(),
        algorithm: RouteAlgorithm::Bfs,
        constraints: RouteConstraints {
            avoid_systems: vec!["BetaTest".to_string()],
            ..RouteConstraints::default()
        },
    };

    let error = plan_route(&starmap, &request).expect_err("avoided goal");
    assert!(format!("{error}").contains("no route found"));
}

#[test]
fn temperature_limit_blocks_hot_systems() {
    let mut starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let beta_id = starmap.system_id_by_name("BetaTest").unwrap();
    starmap
        .systems
        .get_mut(&beta_id)
        .unwrap()
        .metadata
        .temperature = Some(5_000.0);

    let request = RouteRequest {
        start: "Y:170N".to_string(),
        goal: "BetaTest".to_string(),
        algorithm: RouteAlgorithm::Dijkstra,
        constraints: RouteConstraints {
            max_temperature: Some(4_000.0),
            ..RouteConstraints::default()
        },
    };

    let error = plan_route(&starmap, &request).expect_err("temperature filtered");
    assert!(format!("{error}").contains("no route found"));
}

#[test]
fn avoid_gates_switches_to_spatial_graph() {
    let mut starmap = load_starmap(&fixture_path()).expect("fixture loads");
    inject_positions(&mut starmap);

    let request = RouteRequest {
        start: "Y:170N".to_string(),
        goal: "BetaTest".to_string(),
        algorithm: RouteAlgorithm::Dijkstra,
        constraints: RouteConstraints {
            avoid_gates: true,
            ..RouteConstraints::default()
        },
    };

    let plan = plan_route(&starmap, &request).expect("spatial route exists");
    assert_eq!(plan.algorithm, RouteAlgorithm::Dijkstra);
}

fn inject_positions(starmap: &mut evefrontier_lib::Starmap) {
    let positions = [
        (
            "Y:170N",
            SystemPosition {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        ),
        (
            "AlphaTest",
            SystemPosition {
                x: 10.0,
                y: 0.0,
                z: 0.0,
            },
        ),
        (
            "BetaTest",
            SystemPosition {
                x: 20.0,
                y: 0.0,
                z: 0.0,
            },
        ),
    ];

    for (name, position) in positions {
        if let Some(id) = starmap.system_id_by_name(name) {
            if let Some(system) = starmap.systems.get_mut(&id) {
                system.position = Some(position);
            }
        }
    }
}
