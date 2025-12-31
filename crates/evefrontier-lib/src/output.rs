use std::fmt::Write;

use serde::Serialize;

use crate::db::{Starmap, SystemId};
use crate::error::{Error, Result};
use crate::routing::RoutePlan;
use crate::ship::{
    calculate_jump_fuel_cost, FuelConfig, FuelProjection, ShipAttributes, ShipLoadout,
    FUEL_MASS_PER_UNIT_KG,
};
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
    /// Minimum external temperature for the system (Kelvin), if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_external_temp: Option<f64>,
    /// Number of planets in this system.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planet_count: Option<u32>,
    /// Number of moons in this system.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moon_count: Option<u32>,
    /// Fuel projection for this hop (present when ship data supplied).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuel: Option<FuelProjection>,
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
    /// Aggregated fuel projection when ship data is provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuel: Option<FuelSummary>,
    /// fmap URL token for sharing/bookmarking the route.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fmap_url: Option<String>,
}

/// Fuel summary aggregated across all route steps.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FuelSummary {
    pub total: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ship_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
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

            let min_external_temp = starmap
                .systems
                .get(&system_id)
                .and_then(|s| s.metadata.min_external_temp);

            let planet_count = starmap
                .systems
                .get(&system_id)
                .and_then(|s| s.metadata.planet_count);

            let moon_count = starmap
                .systems
                .get(&system_id)
                .and_then(|s| s.metadata.moon_count);

            steps.push(RouteStep {
                index,
                id: system_id,
                name: starmap.system_name(system_id).map(|name| name.to_string()),
                distance,
                method,
                min_external_temp,
                planet_count,
                moon_count,
                fuel: None,
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
            fuel: None,
            fmap_url: None,
        })
    }

    /// Attach fuel projections to each hop using the supplied ship/loadout/config.
    ///
    /// Distance-driven hops receive per-hop fuel data; the first step (origin)
    /// carries no fuel information. The summary's `fuel` field aggregates totals.
    ///
    /// Note: This method inlines gate step handling (zero fuel cost) rather than delegating to
    /// `calculate_route_fuel()` to avoid invoking the full route fuel calculator for each step.
    /// The `calculate_route_fuel()` function remains exported as a public API and is used in tests
    /// and external code for complete route projections with static/dynamic mass modes.
    pub fn attach_fuel(
        &mut self,
        ship: &ShipAttributes,
        loadout: &ShipLoadout,
        fuel_config: &FuelConfig,
    ) -> Result<()> {
        if self.steps.len() <= 1 {
            return Ok(());
        }

        fuel_config.validate()?;

        let mut cumulative = 0.0;
        let mut remaining_fuel = loadout.fuel_load;

        for idx in 1..self.steps.len() {
            let method = self.steps[idx].method.as_deref();

            if method == Some("gate") {
                let remaining = if fuel_config.dynamic_mass {
                    remaining_fuel
                } else {
                    (loadout.fuel_load - cumulative).max(0.0)
                };

                let projection = FuelProjection {
                    hop_cost: 0.0,
                    cumulative,
                    remaining: Some(remaining),
                    warning: None,
                };

                if let Some(step) = self.steps.get_mut(idx) {
                    step.fuel = Some(projection);
                }

                continue;
            }

            let distance = self.steps[idx]
                .distance
                .ok_or_else(|| Error::ShipDataValidation {
                    message: "distance must be present for fuel calculation".to_string(),
                })?;

            if !distance.is_finite() || distance <= 0.0 {
                return Err(Error::ShipDataValidation {
                    message: format!("distance must be finite and positive, got {}", distance),
                });
            }

            let effective_fuel = if fuel_config.dynamic_mass {
                remaining_fuel
            } else {
                loadout.fuel_load
            };

            let mass = ship.base_mass_kg
                + loadout.cargo_mass_kg
                + (effective_fuel * FUEL_MASS_PER_UNIT_KG);

            if !mass.is_finite() || mass <= 0.0 {
                return Err(Error::ShipDataValidation {
                    message: format!("computed mass must be finite and positive, got {}", mass),
                });
            }

            let hop_cost = calculate_jump_fuel_cost(mass, distance, fuel_config)?;
            cumulative += hop_cost;

            let projection = if fuel_config.dynamic_mass {
                remaining_fuel = (remaining_fuel - hop_cost).max(0.0);
                FuelProjection {
                    hop_cost,
                    cumulative,
                    remaining: Some(remaining_fuel),
                    warning: None,
                }
            } else {
                let remaining = (loadout.fuel_load - cumulative).max(0.0);
                FuelProjection {
                    hop_cost,
                    cumulative,
                    remaining: Some(remaining),
                    warning: None,
                }
            };

            if let Some(step) = self.steps.get_mut(idx) {
                step.fuel = Some(projection);
            }
        }

        self.fuel = Some(FuelSummary {
            total: cumulative,
            remaining: Some(if fuel_config.dynamic_mass {
                remaining_fuel
            } else {
                (loadout.fuel_load - cumulative).max(0.0)
            }),
            ship_name: Some(ship.name.clone()),
            warnings: Vec::new(),
        });

        Ok(())
    }

    /// Render the summary using the requested textual mode.
    pub fn render(&self, mode: RouteRenderMode) -> String {
        self.render_with(mode, true)
    }

    /// Render with control over temperature annotations.
    pub fn render_with(&self, mode: RouteRenderMode, show_temps: bool) -> String {
        match mode {
            RouteRenderMode::PlainText => self.render_plain(show_temps),
            RouteRenderMode::RichText => self.render_rich(show_temps),
            RouteRenderMode::InGameNote => self.render_note(show_temps),
        }
    }

    fn render_plain(&self, show_temps: bool) -> String {
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
            let bracket = if show_temps {
                match step.min_external_temp {
                    Some(t) => format!("{}; min {:.2}K", step.id, t),
                    None => format!("{}", step.id),
                }
            } else {
                format!("{}", step.id)
            };
            let _ = writeln!(
                buffer,
                "{:>3}: {} ({})",
                step.index,
                step.display_name(),
                bracket
            );
        }

        buffer + &format!("via {} gates / {} jump drive\n", self.gates, self.jumps)
    }

    fn render_rich(&self, show_temps: bool) -> String {
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
            let bracket = if show_temps {
                match step.min_external_temp {
                    Some(t) => format!("`{}` (min {:.2}K)", step.id, t),
                    None => format!("`{}`", step.id),
                }
            } else {
                format!("`{}`", step.id)
            };
            let _ = writeln!(
                buffer,
                "* {:>2}. **{}** ({})",
                step.index,
                step.display_name(),
                bracket
            );
        }
        buffer + &format!("via {} gates / {} jump drive\n", self.gates, self.jumps)
    }

    fn render_note(&self, show_temps: bool) -> String {
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
            if show_temps {
                match step.min_external_temp {
                    Some(t) => {
                        let _ = writeln!(buffer, "{} (min {:.2}K)", step.display_name(), t);
                    }
                    None => {
                        let _ = writeln!(buffer, "{}", step.display_name());
                    }
                }
            } else {
                let _ = writeln!(buffer, "{}", step.display_name());
            }
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
