use serde::Serialize;

use evefrontier_lib::output::{RouteStep, RouteSummary};
use evefrontier_lib::ship::FuelProjection;

/// Fuel projection for a single hop.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FuelProjectionDto {
    /// Hop cost in whole fuel units (integers)
    pub hop_cost: i64,
    /// Cumulative fuel generated as whole units
    pub cumulative: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

/// Heat projection DTO for a single hop.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HeatProjectionDto {
    pub hop_heat: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub residual_heat: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

impl From<&evefrontier_lib::ship::HeatProjection> for HeatProjectionDto {
    fn from(value: &evefrontier_lib::ship::HeatProjection) -> Self {
        Self {
            hop_heat: value.hop_heat,
            residual_heat: value.residual_heat,
            warning: value.warning.clone(),
        }
    }
}

/// Aggregated heat summary DTO
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HeatSummaryDto {
    // No cumulative total provided; only aggregate warnings are exposed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

impl From<&FuelProjection> for FuelProjectionDto {
    fn from(value: &FuelProjection) -> Self {
        Self {
            hop_cost: value.hop_cost.ceil() as i64,
            cumulative: value.cumulative.ceil() as i64,
            remaining: value.remaining.map(|v| v.ceil() as i64),
            warning: value.warning.clone(),
        }
    }
}

/// Aggregated fuel totals for the entire route.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FuelSummaryDto {
    /// Total fuel projection (ceiling to whole units)
    pub total: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ship_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

/// A single route step.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RouteStepDto {
    pub system: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_ly: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuel: Option<FuelProjectionDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heat: Option<HeatProjectionDto>,
}

impl RouteStepDto {
    fn from_step(step: &RouteStep) -> Self {
        Self {
            system: step.name.as_deref().unwrap_or("<unknown>").to_string(),
            distance_ly: step.distance,
            method: step.method.clone(),
            fuel: step.fuel.as_ref().map(FuelProjectionDto::from),
            heat: step.heat.as_ref().map(HeatProjectionDto::from),
        }
    }
}

/// Summary of the computed route.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RouteSummaryDto {
    pub total_distance_ly: f64,
    pub hops: usize,
    pub gates: usize,
    pub jumps: usize,
    pub algorithm: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuel: Option<FuelSummaryDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heat: Option<HeatSummaryDto>,
}

/// Complete route response returned by the Lambda handler.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RouteResponseDto {
    pub steps: Vec<RouteStepDto>,
    pub summary: RouteSummaryDto,
}

impl RouteResponseDto {
    pub fn from_summary(summary: &RouteSummary) -> Self {
        let steps = summary.steps.iter().map(RouteStepDto::from_step).collect();

        let fuel = summary.fuel.as_ref().map(|f| FuelSummaryDto {
            total: f.total.ceil() as i64,
            remaining: f.remaining.map(|v| v.ceil() as i64),
            ship_name: f.ship_name.clone(),
            warnings: f.warnings.clone(),
        });

        let heat = summary.heat.as_ref().map(|h| HeatSummaryDto {
            warnings: h.warnings.clone(),
        });

        let summary_dto = RouteSummaryDto {
            total_distance_ly: summary.total_distance,
            hops: summary.hops,
            gates: summary.gates,
            jumps: summary.jumps,
            algorithm: summary.algorithm.to_string(),
            fuel,
            heat,
        };

        Self {
            steps,
            summary: summary_dto,
        }
    }
}
