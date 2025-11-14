use std::path::PathBuf;

use evefrontier_lib::{
    load_starmap, RouteAlgorithm, RouteOutputKind, RoutePlan, RouteRenderMode, RouteSummary,
};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/fixtures/minimal_static_data.db")
        .canonicalize()
        .expect("fixture dataset present")
}

fn load_fixture_starmap() -> evefrontier_lib::Starmap {
    load_starmap(&fixture_path()).expect("starmap loads from fixture")
}

#[test]
fn summary_rejects_empty_plans() {
    let starmap = load_fixture_starmap();
    let system = starmap.system_id_by_name("Y:170N").expect("system exists");
    let plan = RoutePlan {
        algorithm: RouteAlgorithm::Bfs,
        start: system,
        goal: system,
        steps: Vec::new(),
        gates: 0,
        jumps: 0,
    };

    let err = RouteSummary::from_plan(RouteOutputKind::Route, &starmap, &plan)
        .expect_err("empty plans are rejected");
    assert_eq!(format!("{err}"), "route plan was empty");
}

#[test]
fn summary_from_plan_populates_names() {
    let starmap = load_fixture_starmap();
    let start = starmap
        .system_id_by_name("Y:170N")
        .expect("start system exists");
    let goal = starmap
        .system_id_by_name("BetaTest")
        .expect("goal system exists");
    let plan = RoutePlan {
        algorithm: RouteAlgorithm::Bfs,
        start,
        goal,
        steps: vec![start, goal],
        gates: 1,
        jumps: 0,
    };

    let summary =
        RouteSummary::from_plan(RouteOutputKind::Route, &starmap, &plan).expect("summary builds");

    assert_eq!(summary.start.name.as_deref(), Some("Y:170N"));
    assert_eq!(summary.goal.name.as_deref(), Some("BetaTest"));
    assert_eq!(summary.hops, 1);
}

#[test]
fn render_modes_include_expected_tokens() {
    let starmap = load_fixture_starmap();
    let start = starmap
        .system_id_by_name("Y:170N")
        .expect("start system exists");
    let goal = starmap
        .system_id_by_name("BetaTest")
        .expect("goal system exists");
    let plan = RoutePlan {
        algorithm: RouteAlgorithm::AStar,
        start,
        goal,
        steps: vec![start, goal],
        gates: 0, // In A* hybrid this may be spatial; tests only assert tokens
        jumps: 1,
    };

    let summary =
        RouteSummary::from_plan(RouteOutputKind::Search, &starmap, &plan).expect("summary builds");

    let plain = summary.render(RouteRenderMode::PlainText);
    assert!(plain.contains("Search: Y:170N -> BetaTest"));
    assert!(plain.contains("algorithm: a-star"));

    let rich = summary.render(RouteRenderMode::RichText);
    assert!(rich.contains("**Search**"));
    assert!(rich.contains("`a-star`"));

    let note = summary.render(RouteRenderMode::InGameNote);
    assert!(note.contains("Search:"));
    assert!(note.contains("Y:170N"));
}
