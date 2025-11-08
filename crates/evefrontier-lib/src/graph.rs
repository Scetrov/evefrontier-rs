use std::collections::HashMap;
use std::sync::Arc;

use crate::db::{Starmap, SystemId};

/// Graph structure used by pathfinding algorithms.
#[derive(Debug, Clone, Default)]
pub struct Graph {
    adjacency: Arc<HashMap<SystemId, Vec<SystemId>>>,
}

impl Graph {
    /// Return the neighbours for a given system identifier.
    pub fn neighbours(&self, system: SystemId) -> &[SystemId] {
        self.adjacency
            .get(&system)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

/// Build a pathfinding graph from the in-memory starmap.
pub fn build_graph(starmap: &Starmap) -> Graph {
    Graph {
        adjacency: Arc::clone(&starmap.adjacency),
    }
}
