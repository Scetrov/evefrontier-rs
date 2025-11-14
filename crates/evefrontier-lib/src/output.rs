use std::fmt::Write;

use serde::Serialize;

use crate::db::{Starmap, SystemId};
use crate::error::{Error, Result};
use crate::routing::RoutePlan;
use crate::RouteAlgorithm;

/// Classifies the high-level command that produced a route summary.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RouteOutputKind {
    Route,
}

/// Presentation style for turning a [`RouteSummary`] into text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteRenderMode {
    PlainText,
    RichText,
    InGameNote,
}

/// Endpoint within a planned route.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RouteEndpoint {
    pub id: SystemId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl RouteEndpoint {
    fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or("<unknown>")
    }
}

/// Step taken during traversal of a planned route.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RouteStep {
    pub index: usize,
    pub id: SystemId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Distance in light-years to this step from the previous step (None for the first step).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance: Option<f64>,
    /// How this step was reached: "gate" or "jump".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
}

impl RouteStep {
    fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or("<unknown>")
    }
}

/// Structured representation of a planned route that higher-level consumers can serialise.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RouteSummary {
    pub kind: RouteOutputKind,
    pub algorithm: RouteAlgorithm,
    pub hops: usize,
    pub gates: usize,
    pub jumps: usize,
    /// Total accumulated distance across all hops (light-years).
    pub total_distance: f64,
    /// Distance covered by jump drive (light-years).
    pub jump_distance: f64,
    pub start: RouteEndpoint,
    pub goal: RouteEndpoint,
    pub steps: Vec<RouteStep>,
}

impl RouteSummary {
    /// Convert a [`RoutePlan`] into a structured summary with resolved system names.
    pub fn from_plan(kind: RouteOutputKind, starmap: &Starmap, plan: &RoutePlan) -> Result<Self> {
        if plan.steps.is_empty() {
            return Err(Error::EmptyRoutePlan);
        }

        // Build steps with distances and methods
        let mut steps = Vec::with_capacity(plan.steps.len());
        let mut total_distance = 0.0;
        let mut jump_distance = 0.0;

        for (index, &system_id) in plan.steps.iter().enumerate() {
            let (distance, method) = if index == 0 {
                (None, None)
            } else {
                let prev_id = plan.steps[index - 1];
                let dist = compute_distance(starmap, prev_id, system_id);
                let edge_method = classify_edge_method(starmap, prev_id, system_id);
                
                if let Some(d) = dist {
                    total_distance += d;
                    if edge_method.as_deref() == Some("jump") {
                        jump_distance += d;
                    }
                }
                
                (dist, edge_method)
            };

            steps.push(RouteStep {
                index,
                id: system_id,
                name: starmap.system_name(system_id).map(|name| name.to_string()),
                distance,
                method,
            });
        }

        let start = RouteEndpoint {
            id: steps
                .first()
                .map(|step| step.id)
                .expect("validated non-empty steps"),
            name: steps.first().and_then(|step| step.name.clone()),
        };
        let goal = RouteEndpoint {
            id: steps
                .last()
                .map(|step| step.id)
                .expect("validated non-empty steps"),
            name: steps.last().and_then(|step| step.name.clone()),
        };

        Ok(Self {
            kind,
            algorithm: plan.algorithm,
            hops: plan.hop_count(),
            gates: plan.gates,
            jumps: plan.jumps,
            total_distance,
            jump_distance,
            start,
            goal,
            steps,
        })
    }

    /// Render the summary using the requested textual mode.
    pub fn render(&self, mode: RouteRenderMode) -> String {
        match mode {
            RouteRenderMode::PlainText => self.render_plain(),
            RouteRenderMode::RichText => self.render_rich(),
            RouteRenderMode::InGameNote => self.render_note(),
        }
    }

    fn render_plain(&self) -> String {
        let mut buffer = String::new();
        let _ = writeln!(
            buffer,
            "Route: {} -> {} ({} hops, algorithm: {})",
            self.start.display_name(),
            self.goal.display_name(),
            self.hops,
            self.algorithm
        );

        for step in &self.steps {
            let _ = writeln!(
                buffer,
                "{:>3}: {} ({})",
                step.index,
                step.display_name(),
                step.id
            );
        }

        buffer
            + &format!("via {} gates / {} jump drive\n", self.gates, self.jumps)
    }

    fn render_rich(&self) -> String {
        let mut buffer = String::new();
        let _ = writeln!(
            buffer,
            "**Route** — _{} → {}_ ({} hops, algorithm: `{}`)",
            self.start.display_name(),
            self.goal.display_name(),
            self.hops,
            self.algorithm
        );
        for step in &self.steps {
            let _ = writeln!(
                buffer,
                "* {:>2}. **{}** (`{}`)",
                step.index,
                step.display_name(),
                step.id
            );
        }
        buffer + &format!("via {} gates / {} jump drive\n", self.gates, self.jumps)
    }

    fn render_note(&self) -> String {
        let mut buffer = String::new();
        let _ = writeln!(buffer, "Route:");
        let _ = writeln!(
            buffer,
            "{} -> {} ({} hops via {})",
            self.start.display_name(),
            self.goal.display_name(),
            self.hops,
            self.algorithm
        );
        for step in &self.steps {
            let _ = writeln!(buffer, "{}", step.display_name());
        }
        buffer + &format!("via {} gates / {} jump drive\n", self.gates, self.jumps)
    }
}

/// Compute the Euclidean distance between two systems in light-years.
fn compute_distance(starmap: &Starmap, from: SystemId, to: SystemId) -> Option<f64> {
    let from_sys = starmap.systems.get(&from)?;
    let to_sys = starmap.systems.get(&to)?;
    let from_pos = from_sys.position.as_ref()?;
    let to_pos = to_sys.position.as_ref()?;
    Some(from_pos.distance_to(to_pos))
}

/// Classify whether an edge is a gate or spatial jump.
fn classify_edge_method(starmap: &Starmap, from: SystemId, to: SystemId) -> Option<String> {
    // Check if there's a gate connection
    let has_gate = starmap
        .adjacency
        .get(&from)
        .map(|neighbors| neighbors.contains(&to))
        .unwrap_or(false);
    
    if has_gate {
        Some("gate".to_string())
    } else {
        Some("jump".to_string())
    }
}
