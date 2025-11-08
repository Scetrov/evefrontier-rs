//! EveFrontier library entry points.
//!
//! This crate exposes helpers to locate the EveFrontier dataset, load the
//! starmap into memory, build graph representations, and run pathfinding
//! algorithms. Higher-level consumers (CLI, Lambdas) should only depend on the
//! functions exported here instead of reimplementing behavior.

pub mod dataset;
pub mod db;
pub mod error;
pub mod github;
pub mod graph;
pub mod path;

pub use dataset::{default_dataset_path, ensure_c3e6_dataset};
pub use db::{load_starmap, Starmap, System, SystemId};
pub use error::{Error, Result};
pub use graph::{build_graph, Graph};
pub use path::find_route;
