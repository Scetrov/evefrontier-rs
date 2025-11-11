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
    Search,
    Path,
}

impl RouteOutputKind {
    /// Human-readable label shown in textual renderings.
    pub fn label(self) -> &'static str {
        match self {
            RouteOutputKind::Route => "Route",
            RouteOutputKind::Search => "Search",
            RouteOutputKind::Path => "Path",
        }
    }
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
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RouteStep {
    pub index: usize,
    pub id: SystemId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl RouteStep {
    fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or("<unknown>")
    }
}

/// Structured representation of a planned route that higher-level consumers can serialise.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RouteSummary {
    pub kind: RouteOutputKind,
    pub algorithm: RouteAlgorithm,
    pub hops: usize,
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

        let steps = plan
            .steps
            .iter()
            .enumerate()
            .map(|(index, system_id)| RouteStep {
                index,
                id: *system_id,
                name: starmap.system_name(*system_id).map(|name| name.to_string()),
            })
            .collect::<Vec<_>>();

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
            "{}: {} -> {} ({} hops, algorithm: {})",
            self.kind.label(),
            self.start.display_name(),
            self.goal.display_name(),
            self.hops,
            self.algorithm
        );

        match self.kind {
            RouteOutputKind::Path => {
                let joined = self
                    .steps
                    .iter()
                    .map(|step| format!("{} ({})", step.display_name(), step.id))
                    .collect::<Vec<_>>()
                    .join(" -> ");
                let _ = writeln!(buffer, "{joined}");
            }
            _ => {
                for step in &self.steps {
                    let _ = writeln!(
                        buffer,
                        "{:>3}: {} ({})",
                        step.index,
                        step.display_name(),
                        step.id
                    );
                }
            }
        }

        buffer
    }

    fn render_rich(&self) -> String {
        let mut buffer = String::new();
        let _ = writeln!(
            buffer,
            "**{}** — _{} → {}_ ({} hops, algorithm: `{}`)",
            self.kind.label(),
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
        buffer
    }

    fn render_note(&self) -> String {
        let mut buffer = String::new();
        let _ = writeln!(buffer, "{}:", self.kind.label());
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
        buffer
    }
}
