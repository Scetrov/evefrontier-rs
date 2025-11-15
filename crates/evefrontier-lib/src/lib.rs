//! EveFrontier library entry points.
//!
//! This crate exposes helpers to locate the EveFrontier dataset, load the
//! starmap into memory, build graph representations, and run pathfinding
//! algorithms. Higher-level consumers (CLI, Lambdas) should only depend on the
//! functions exported here instead of reimplementing behavior.
//!
//! # Quick Start
//!
//! The typical workflow for using this library is:
//!
//! 1. **Ensure the dataset is available** using [`ensure_c3e6_dataset`] or [`ensure_dataset`]
//! 2. **Load the starmap** with [`load_starmap`]
//! 3. **Plan a route** using [`plan_route`] with a [`RouteRequest`]
//! 4. **Format the output** using types from the [`output`] module
//!
//! # Example
//!
//! ```no_run
//! use evefrontier_lib::{
//!     ensure_c3e6_dataset, load_starmap, plan_route,
//!     RouteRequest, RouteAlgorithm, RouteConstraints
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // 1. Ensure dataset is downloaded and get its path
//! let dataset_path = ensure_c3e6_dataset(None)?;
//!
//! // 2. Load the starmap into memory
//! let starmap = load_starmap(&dataset_path)?;
//!
//! // 3. Create a route request
//! let request = RouteRequest {
//!     start: "Nod".to_string(),
//!     goal: "Brana".to_string(),
//!     algorithm: RouteAlgorithm::AStar,
//!     constraints: RouteConstraints::default(),
//! };
//!
//! // 4. Plan the route
//! let plan = plan_route(&starmap, &request)?;
//!
//! println!("Route found with {} hops", plan.hop_count());
//! # Ok(())
//! # }
//! ```
//!
//! # Error Handling
//!
//! All fallible operations return [`Result<T, Error>`](Result), where [`Error`]
//! provides detailed context about failures including:
//!
//! - Dataset not found or download failures
//! - Unknown system names (with fuzzy suggestions)
//! - Route computation failures
//! - Schema incompatibilities
//!
//! # Routing Algorithms
//!
//! Three pathfinding algorithms are available via [`RouteAlgorithm`]:
//!
//! - **BFS**: Breadth-first search for shortest hop count (unweighted)
//! - **Dijkstra**: Shortest path by distance (weighted by light-years)
//! - **A\***: Heuristic-guided search using spatial coordinates (default, typically fastest)
//!
//! # Constraints
//!
//! Routes can be constrained using [`RouteConstraints`]:
//!
//! - `max_jump`: Maximum jump distance in light-years (for spatial routes)
//! - `avoid_systems`: List of system names to exclude from routes
//! - `avoid_gates`: Force spatial-only routing (no jump gates)
//! - `max_temperature`: Exclude systems above a temperature threshold
//!

#![deny(warnings)]

pub mod dataset;
pub mod db;
pub mod error;
pub mod github;
pub mod graph;
pub mod output;
pub mod path;
pub mod routing;
pub mod temperature;

pub use dataset::{default_dataset_path, ensure_c3e6_dataset, ensure_dataset};
pub use db::{load_starmap, Starmap, System, SystemId, SystemMetadata, SystemPosition};
pub use error::{Error, Result};
pub use github::DatasetRelease;
pub use graph::{
    build_gate_graph, build_graph, build_hybrid_graph, build_spatial_graph, Edge, EdgeKind, Graph,
    GraphMode,
};
pub use output::{RouteEndpoint, RouteOutputKind, RouteRenderMode, RouteStep, RouteSummary};
pub use path::{
    find_route, find_route_a_star, find_route_bfs, find_route_dijkstra, PathConstraints,
};
pub use routing::{plan_route, RouteAlgorithm, RouteConstraints, RoutePlan, RouteRequest};
