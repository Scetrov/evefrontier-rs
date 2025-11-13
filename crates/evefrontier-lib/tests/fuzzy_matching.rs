use std::path::PathBuf;

use evefrontier_lib::{load_starmap, plan_route, RouteRequest};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/fixtures/minimal_static_data.db")
}

#[test]
fn fuzzy_matches_returns_similar_names() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    
    // Test exact match doesn't use fuzzy
    let exact = starmap.fuzzy_system_matches("Y:170N", 3);
    assert!(exact.contains(&"Y:170N".to_string()));
    
    // Test typo: O instead of 0
    let typo = starmap.fuzzy_system_matches("Y:17ON", 3);
    assert!(!typo.is_empty(), "should find similar systems");
    assert!(typo.contains(&"Y:170N".to_string()), "should suggest Y:170N for Y:17ON");
    
    // Test partial match
    let partial = starmap.fuzzy_system_matches("AlphaTst", 3);
    assert!(partial.contains(&"AlphaTest".to_string()), "should suggest AlphaTest");
}

#[test]
fn unknown_system_includes_suggestions() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    
    let request = RouteRequest::bfs("Y:17ON", "BetaTest"); // Typo: O instead of 0
    let err = plan_route(&starmap, &request).expect_err("should fail with unknown system");
    
    let error_message = format!("{}", err);
    assert!(error_message.contains("unknown system name"), "error should mention unknown system");
    assert!(error_message.contains("Did you mean"), "error should include suggestions");
    assert!(error_message.contains("Y:170N"), "error should suggest Y:170N");
}

#[test]
fn fuzzy_matches_respects_limit() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    
    let matches = starmap.fuzzy_system_matches("Test", 2);
    assert!(matches.len() <= 2, "should respect limit of 2");
}

#[test]
fn fuzzy_matches_filters_low_similarity() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    
    // Very different name should return no matches
    let no_match = starmap.fuzzy_system_matches("CompletlyWrongXYZ", 3);
    assert!(no_match.is_empty() || !no_match.iter().any(|s| s == "Y:170N"), 
            "should not match very different names");
}

#[test]
fn avoided_system_typo_includes_suggestions() {
    let starmap = load_starmap(&fixture_path()).expect("fixture loads");
    
    let request = RouteRequest {
        start: "Y:170N".to_string(),
        goal: "BetaTest".to_string(),
        algorithm: evefrontier_lib::RouteAlgorithm::Bfs,
        constraints: evefrontier_lib::RouteConstraints {
            avoid_systems: vec!["AlphaTst".to_string()], // Typo
            ..Default::default()
        },
    };
    
    let err = plan_route(&starmap, &request).expect_err("should fail with unknown avoided system");
    let error_message = format!("{}", err);
    assert!(error_message.contains("AlphaTst"), "error should mention the typo");
    assert!(error_message.contains("Did you mean"), "error should include suggestions");
}
