use std::collections::{HashMap, VecDeque};

use crate::db::SystemId;
use crate::graph::Graph;

/// Find a route between `start` and `goal` using breadth-first search.
///
/// The returned path is the shortest in terms of jump count because the graph
/// is treated as unweighted.
pub fn find_route(graph: &Graph, start: SystemId, goal: SystemId) -> Option<Vec<SystemId>> {
    if start == goal {
        return Some(vec![start]);
    }

    let mut visited = HashMap::new();
    let mut queue = VecDeque::new();

    visited.insert(start, None);
    queue.push_back(start);

    while let Some(current) = queue.pop_front() {
        for &next in graph.neighbours(current) {
            if visited.contains_key(&next) {
                continue;
            }
            visited.insert(next, Some(current));
            if next == goal {
                return Some(reconstruct_path(&visited, start, goal));
            }
            queue.push_back(next);
        }
    }

    None
}

fn reconstruct_path(
    parents: &HashMap<SystemId, Option<SystemId>>,
    start: SystemId,
    goal: SystemId,
) -> Vec<SystemId> {
    let mut path = Vec::new();
    let mut current = Some(goal);
    while let Some(node) = current {
        path.push(node);
        if node == start {
            break;
        }
        current = parents.get(&node).copied().flatten();
    }
    path.reverse();
    path
}
