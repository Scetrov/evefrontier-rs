use std::path::PathBuf;

use evefrontier_lib::{load_starmap, plan_route, RouteAlgorithm, RouteConstraints, RouteRequest};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/minimal/static_data.db")
}

#[test]
fn bfs_route_plan_succeeds() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let request = RouteRequest::bfs("Nod", "Brana");
    let plan = plan_route(&starmap, &request).expect("route exists");

    assert_eq!(plan.algorithm, RouteAlgorithm::Bfs);
    assert_eq!(plan.start, starmap.system_id_by_name("Nod").unwrap());
    assert_eq!(plan.goal, starmap.system_id_by_name("Brana").unwrap());
    assert!(plan.hop_count() >= 1);
}

#[test]
fn dijkstra_route_plan_succeeds() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let request = RouteRequest {
        start: "Nod".to_string(),
        goal: "Brana".to_string(),
        algorithm: RouteAlgorithm::Dijkstra,
        constraints: RouteConstraints::default(),
        spatial_index: None,
    };

    let plan = plan_route(&starmap, &request).expect("route exists");
    assert_eq!(plan.algorithm, RouteAlgorithm::Dijkstra);
}

#[test]
fn a_star_respects_max_jump_constraint() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    // Test spatial routing without gates
    // Note: Nod and Brana are ~297 ly apart, so we need a large max_jump to test this
    let request = RouteRequest {
        start: "Nod".to_string(),
        goal: "Brana".to_string(),
        algorithm: RouteAlgorithm::AStar,
        constraints: RouteConstraints {
            max_jump: Some(300.0),
            avoid_gates: true,
            ..RouteConstraints::default()
        },
        spatial_index: None,
    };

    let plan = plan_route(&starmap, &request).expect("route exists");
    assert_eq!(plan.algorithm, RouteAlgorithm::AStar);
    assert!(
        plan.steps.len() >= 2,
        "route should include intermediate hop"
    );
}

#[test]
fn avoided_goal_rejects_route() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let request = RouteRequest {
        start: "Nod".to_string(),
        goal: "Brana".to_string(),
        algorithm: RouteAlgorithm::Bfs,
        constraints: RouteConstraints {
            avoid_systems: vec!["Brana".to_string()],
            ..RouteConstraints::default()
        },
        spatial_index: None,
    };

    let error = plan_route(&starmap, &request).expect_err("avoided goal");
    assert!(format!("{error}").contains("no route found"));
}

#[test]
fn temperature_limit_blocks_hot_systems() {
    let mut starmap = load_starmap(&fixture_path()).expect("fixture loads");
    let brana_id = starmap.system_id_by_name("Brana").unwrap();
    starmap
        .systems
        .get_mut(&brana_id)
        .unwrap()
        .metadata
        .star_temperature = Some(5_000.0);

    let request = RouteRequest {
        start: "Nod".to_string(),
        goal: "Brana".to_string(),
        algorithm: RouteAlgorithm::Dijkstra,
        constraints: RouteConstraints {
            max_temperature: Some(4_000.0),
            ..RouteConstraints::default()
        },
        spatial_index: None,
    };

    let error = plan_route(&starmap, &request).expect_err("temperature filtered");
    assert!(format!("{error}").contains("no route found"));
}

#[test]
fn avoid_gates_switches_to_spatial_graph() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");

    let request = RouteRequest {
        start: "Nod".to_string(),
        goal: "Brana".to_string(),
        algorithm: RouteAlgorithm::Dijkstra,
        constraints: RouteConstraints {
            avoid_gates: true,
            ..RouteConstraints::default()
        },
        spatial_index: None,
    };

    let plan = plan_route(&starmap, &request).expect("spatial route exists");
    assert_eq!(plan.algorithm, RouteAlgorithm::Dijkstra);
}

// inject_positions is no longer needed - real fixture data includes coordinates
