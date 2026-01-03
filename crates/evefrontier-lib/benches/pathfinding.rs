use criterion::{criterion_group, criterion_main, Criterion};
use evefrontier_lib::{
    load_starmap, plan_route, RouteAlgorithm, RouteConstraints, RouteRequest, Starmap,
};
use once_cell::sync::Lazy;
use std::hint::black_box;
use std::path::PathBuf;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/fixtures/minimal/static_data.db")
}

static STARMAP: Lazy<Starmap> = Lazy::new(|| load_starmap(&fixture_path()).expect("fixture loads"));
static BFS_REQUEST: Lazy<RouteRequest> = Lazy::new(|| RouteRequest::bfs("Nod", "Brana"));
static DIJKSTRA_REQUEST: Lazy<RouteRequest> = Lazy::new(|| RouteRequest {
    start: "Nod".to_string(),
    goal: "Brana".to_string(),
    algorithm: RouteAlgorithm::Dijkstra,
    constraints: RouteConstraints::default(),
    spatial_index: None,
});
static ASTAR_HYBRID_REQUEST: Lazy<RouteRequest> = Lazy::new(|| RouteRequest {
    start: "Nod".to_string(),
    goal: "Brana".to_string(),
    algorithm: RouteAlgorithm::AStar,
    constraints: RouteConstraints::default(),
    spatial_index: None,
});
static ASTAR_SPATIAL_REQUEST: Lazy<RouteRequest> = Lazy::new(|| RouteRequest {
    start: "Nod".to_string(),
    goal: "Brana".to_string(),
    algorithm: RouteAlgorithm::AStar,
    constraints: RouteConstraints {
        max_jump: Some(310.0),
        avoid_gates: true,
        ..RouteConstraints::default()
    },
    spatial_index: None,
});

fn benchmark_pathfinding(c: &mut Criterion) {
    let starmap = &*STARMAP;

    c.bench_function("bfs_nod_brana", |b| {
        let request = &*BFS_REQUEST;
        b.iter(|| {
            let plan = plan_route(starmap, request).expect("route exists");
            black_box(plan.hop_count())
        });
    });

    c.bench_function("dijkstra_nod_brana", |b| {
        let request = &*DIJKSTRA_REQUEST;
        b.iter(|| {
            let plan = plan_route(starmap, request).expect("route exists");
            black_box((plan.gates, plan.jumps))
        });
    });

    c.bench_function("astar_hybrid_nod_brana", |b| {
        let request = &*ASTAR_HYBRID_REQUEST;
        b.iter(|| {
            let plan = plan_route(starmap, request).expect("route exists");
            black_box(plan.steps.len())
        });
    });

    c.bench_function("astar_spatial_nod_brana", |b| {
        let request = &*ASTAR_SPATIAL_REQUEST;
        b.iter(|| {
            let plan = plan_route(starmap, request).expect("route exists");
            black_box(plan.steps.len())
        });
    });
}

criterion_group!(benches, benchmark_pathfinding);
criterion_main!(benches);
