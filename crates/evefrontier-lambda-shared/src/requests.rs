//! Request types and validation for Lambda endpoints.

use serde::{Deserialize, Serialize};

use crate::ProblemDetails;

/// Validation trait for Lambda request types.
///
/// Implementations should validate all fields and return a `ProblemDetails`
/// error for invalid input.
pub trait Validate {
    /// Validate the request, returning an error if invalid.
    ///
    /// The `request_id` is used to populate the `instance` field of any
    /// returned `ProblemDetails`.
    ///
    /// Returns a boxed `ProblemDetails` to avoid large `Result::Err` variants.
    fn validate(&self, request_id: &str) -> Result<(), Box<ProblemDetails>>;
}

/// Request for computing a route between two systems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRequest {
    /// Starting system name.
    pub from: String,

    /// Destination system name.
    pub to: String,

    /// Routing algorithm to use.
    #[serde(default)]
    pub algorithm: RouteAlgorithm,

    /// Maximum jump distance in light-years (for spatial routes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_jump: Option<f64>,

    /// Systems to avoid when computing the route.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub avoid: Vec<String>,

    /// If true, avoid gates and use only spatial jumps.
    #[serde(default)]
    pub avoid_gates: bool,

    /// Maximum star temperature threshold in Kelvin.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_temperature: Option<f64>,

    /// Optional ship name for fuel projection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ship: Option<String>,

    /// Fuel quality percentage (1-100). Defaults to 10 when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuel_quality: Option<f64>,

    /// Cargo mass in kilograms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cargo_mass: Option<f64>,

    /// Fuel load in units.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuel_load: Option<f64>,

    /// Optional heat calibration constant to scale heat energy calculations.
    #[serde(skip_serializing_if = "Option::is_none")]
    // `heat_calibration` removed: calibration is fixed server-side to 1e-7

    /// Enable per-hop dynamic mass recalculation.
    #[serde(default)]
    pub dynamic_mass: Option<bool>,

    /// Enable conservative avoidance of hops that would reach critical engine heat.
    #[serde(default = "default_true")]
    pub avoid_critical_state: bool,

    /// Maximum number of spatial neighbors to consider (default from lib).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_spatial_neighbors: Option<usize>,

    /// Optional optimization objective: distance or fuel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimization: Option<RouteOptimization>,
}

fn default_true() -> bool {
    true
}

/// Supported routing algorithms.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RouteAlgorithm {
    /// Breadth-first search (unweighted, shortest hop count).
    Bfs,
    /// Dijkstra's algorithm (weighted by distance).
    Dijkstra,
    /// A* search with spatial heuristic (default, typically fastest).
    #[default]
    AStar,
}

impl From<RouteAlgorithm> for evefrontier_lib::RouteAlgorithm {
    fn from(value: RouteAlgorithm) -> Self {
        match value {
            RouteAlgorithm::Bfs => evefrontier_lib::RouteAlgorithm::Bfs,
            RouteAlgorithm::Dijkstra => evefrontier_lib::RouteAlgorithm::Dijkstra,
            RouteAlgorithm::AStar => evefrontier_lib::RouteAlgorithm::AStar,
        }
    }
}

/// Optional optimization objective for planning.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RouteOptimization {
    Distance,
    Fuel,
}

impl From<RouteOptimization> for evefrontier_lib::routing::RouteOptimization {
    fn from(value: RouteOptimization) -> Self {
        match value {
            RouteOptimization::Distance => evefrontier_lib::routing::RouteOptimization::Distance,
            RouteOptimization::Fuel => evefrontier_lib::routing::RouteOptimization::Fuel,
        }
    }
}

impl Validate for RouteRequest {
    fn validate(&self, request_id: &str) -> Result<(), Box<ProblemDetails>> {
        if self.from.trim().is_empty() {
            return Err(Box::new(ProblemDetails::bad_request(
                "The 'from' field is required and cannot be empty",
                request_id,
            )));
        }

        if self.to.trim().is_empty() {
            return Err(Box::new(ProblemDetails::bad_request(
                "The 'to' field is required and cannot be empty",
                request_id,
            )));
        }

        if let Some(max_jump) = self.max_jump {
            if max_jump <= 0.0 {
                return Err(Box::new(ProblemDetails::bad_request(
                    "The 'max_jump' field must be a positive number",
                    request_id,
                )));
            }
        }

        if let Some(max_temp) = self.max_temperature {
            if max_temp <= 0.0 {
                return Err(Box::new(ProblemDetails::bad_request(
                    "The 'max_temperature' field must be a positive number",
                    request_id,
                )));
            }
        }

        if let Some(ref ship) = self.ship {
            if ship.trim().is_empty() {
                return Err(Box::new(ProblemDetails::bad_request(
                    "The 'ship' field cannot be empty when provided",
                    request_id,
                )));
            }
        }

        if let Some(fuel_quality) = self.fuel_quality {
            if !(1.0..=100.0).contains(&fuel_quality) {
                return Err(Box::new(ProblemDetails::bad_request(
                    "The 'fuel_quality' field must be between 1 and 100",
                    request_id,
                )));
            }
        }

        if let Some(cargo_mass) = self.cargo_mass {
            if cargo_mass < 0.0 {
                return Err(Box::new(ProblemDetails::bad_request(
                    "The 'cargo_mass' field must be zero or greater",
                    request_id,
                )));
            }
        }

        if let Some(fuel_load) = self.fuel_load {
            if fuel_load < 0.0 {
                return Err(Box::new(ProblemDetails::bad_request(
                    "The 'fuel_load' field must be zero or greater",
                    request_id,
                )));
            }
        }

        // No validation required; calibration is fixed.

        Ok(())
    }
}

/// Request for finding gate-connected neighbors of a system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoutGatesRequest {
    /// System name or ID to find neighbors for.
    pub system: String,
}

impl Validate for ScoutGatesRequest {
    fn validate(&self, request_id: &str) -> Result<(), Box<ProblemDetails>> {
        if self.system.trim().is_empty() {
            return Err(Box::new(ProblemDetails::bad_request(
                "The 'system' field is required and cannot be empty",
                request_id,
            )));
        }
        Ok(())
    }
}

/// Request for finding systems within a spatial range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoutRangeRequest {
    /// System name or ID to search from.
    pub system: String,

    /// Maximum number of results to return.
    #[serde(default = "default_limit")]
    pub limit: usize,

    /// Maximum distance in light-years.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<f64>,

    /// Maximum star temperature threshold in Kelvin.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_temperature: Option<f64>,
}

fn default_limit() -> usize {
    10
}

impl Validate for ScoutRangeRequest {
    fn validate(&self, request_id: &str) -> Result<(), Box<ProblemDetails>> {
        if self.system.trim().is_empty() {
            return Err(Box::new(ProblemDetails::bad_request(
                "The 'system' field is required and cannot be empty",
                request_id,
            )));
        }

        if self.limit == 0 {
            return Err(Box::new(ProblemDetails::bad_request(
                "The 'limit' field must be at least 1",
                request_id,
            )));
        }

        if self.limit > 100 {
            return Err(Box::new(ProblemDetails::bad_request(
                "The 'limit' field cannot exceed 100",
                request_id,
            )));
        }

        if let Some(radius) = self.radius {
            if radius <= 0.0 {
                return Err(Box::new(ProblemDetails::bad_request(
                    "The 'radius' field must be a positive number",
                    request_id,
                )));
            }
        }

        if let Some(max_temp) = self.max_temperature {
            if max_temp <= 0.0 {
                return Err(Box::new(ProblemDetails::bad_request(
                    "The 'max_temperature' field must be a positive number",
                    request_id,
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_request_valid() {
        let request = RouteRequest {
            from: "Nod".to_string(),
            to: "Brana".to_string(),
            algorithm: RouteAlgorithm::AStar,
            max_jump: Some(80.0),
            avoid: vec![],
            avoid_gates: false,
            max_temperature: None,
            ship: None,
            fuel_quality: None,
            cargo_mass: None,
            fuel_load: None,
            dynamic_mass: None,
            avoid_critical_state: true,
            max_spatial_neighbors: None,
            optimization: None,
        };
        assert!(request.validate("req-123").is_ok());
    }

    #[test]
    fn test_route_request_empty_from() {
        let request = RouteRequest {
            from: "".to_string(),
            to: "Brana".to_string(),
            algorithm: RouteAlgorithm::AStar,
            max_jump: None,
            avoid: vec![],
            avoid_gates: false,
            max_temperature: None,
            ship: None,
            fuel_quality: None,
            cargo_mass: None,
            fuel_load: None,
            dynamic_mass: None,
            avoid_critical_state: true,
            max_spatial_neighbors: None,
            optimization: None,
        };
        let err = request.validate("req-123").unwrap_err();
        assert_eq!(err.status, 400);
        assert!(err.detail.unwrap().contains("'from' field"));
    }

    #[test]
    fn test_route_request_empty_to() {
        let request = RouteRequest {
            from: "Nod".to_string(),
            to: "".to_string(),
            algorithm: RouteAlgorithm::AStar,
            max_jump: None,
            avoid: vec![],
            avoid_gates: false,
            max_temperature: None,
            ship: None,
            fuel_quality: None,
            cargo_mass: None,
            fuel_load: None,
            dynamic_mass: None,
            avoid_critical_state: true,
            max_spatial_neighbors: None,
            optimization: None,
        };
        let err = request.validate("req-123").unwrap_err();
        assert_eq!(err.status, 400);
        assert!(err.detail.unwrap().contains("'to' field"));
    }

    #[test]
    fn test_route_request_negative_max_jump() {
        let request = RouteRequest {
            from: "Nod".to_string(),
            to: "Brana".to_string(),
            algorithm: RouteAlgorithm::Dijkstra,
            max_jump: Some(-10.0),
            avoid: vec![],
            avoid_gates: false,
            max_temperature: None,
            ship: None,
            fuel_quality: None,
            cargo_mass: None,
            fuel_load: None,
            dynamic_mass: None,
            avoid_critical_state: true,
            max_spatial_neighbors: None,
            optimization: None,
        };
        let err = request.validate("req-123").unwrap_err();
        assert!(err.detail.unwrap().contains("positive number"));
    }

    #[test]
    fn test_scout_gates_request_valid() {
        let request = ScoutGatesRequest {
            system: "Nod".to_string(),
        };
        assert!(request.validate("req-456").is_ok());
    }

    #[test]
    fn test_scout_range_request_valid() {
        let request = ScoutRangeRequest {
            system: "Nod".to_string(),
            radius: Some(80.0),
            max_temperature: None,
            limit: 10,
        };
        assert!(request.validate("req-789").is_ok());
    }

    #[test]
    fn test_route_algorithm_serde() {
        let json = r#""a-star""#;
        let algo: RouteAlgorithm = serde_json::from_str(json).unwrap();
        assert_eq!(algo, RouteAlgorithm::AStar);

        let json = r#""bfs""#;
        let algo: RouteAlgorithm = serde_json::from_str(json).unwrap();
        assert_eq!(algo, RouteAlgorithm::Bfs);
    }

    #[test]
    fn test_route_algorithm_to_lib_conversion() {
        use evefrontier_lib::RouteAlgorithm as LibAlgorithm;

        let bfs: LibAlgorithm = RouteAlgorithm::Bfs.into();
        assert!(matches!(bfs, LibAlgorithm::Bfs));

        let dijkstra: LibAlgorithm = RouteAlgorithm::Dijkstra.into();
        assert!(matches!(dijkstra, LibAlgorithm::Dijkstra));

        let astar: LibAlgorithm = RouteAlgorithm::AStar.into();
        assert!(matches!(astar, LibAlgorithm::AStar));
    }

    #[test]
    fn test_route_algorithm_default() {
        let algo = RouteAlgorithm::default();
        assert_eq!(algo, RouteAlgorithm::AStar);
    }

    #[test]
    fn test_scout_range_default_limit() {
        let json = r#"{"system": "Nod"}"#;
        let req: ScoutRangeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.limit, 10); // default_limit()
    }

    #[test]
    fn test_route_request_with_all_constraints() {
        let req = RouteRequest {
            from: "Nod".to_string(),
            to: "Brana".to_string(),
            algorithm: RouteAlgorithm::Dijkstra,
            max_jump: Some(50.0),
            avoid: vec!["System1".to_string(), "System2".to_string()],
            avoid_gates: true,
            max_temperature: Some(100.0),
            ship: None,
            fuel_quality: None,
            cargo_mass: None,
            fuel_load: None,
            dynamic_mass: None,
            avoid_critical_state: true,
            max_spatial_neighbors: None,
            optimization: None,
        };
        assert!(req.validate("req-constraints").is_ok());
    }

    #[test]
    fn test_route_request_negative_temperature() {
        let req = RouteRequest {
            from: "Nod".to_string(),
            to: "Brana".to_string(),
            algorithm: RouteAlgorithm::AStar,
            max_jump: None,
            avoid: vec![],
            avoid_gates: false,
            max_temperature: Some(-50.0),
            ship: None,
            fuel_quality: None,
            cargo_mass: None,
            fuel_load: None,
            dynamic_mass: None,
            avoid_critical_state: true,
            max_spatial_neighbors: None,
            optimization: None,
        };
        let err = req.validate("req-neg-temp").unwrap_err();
        assert!(err.detail.unwrap().contains("max_temperature"));
    }

    #[test]
    fn test_route_request_with_ship_fields() {
        let req = RouteRequest {
            from: "Nod".to_string(),
            to: "Brana".to_string(),
            algorithm: RouteAlgorithm::AStar,
            max_jump: None,
            avoid: vec![],
            avoid_gates: false,
            max_temperature: None,
            ship: Some("Reflex".to_string()),
            fuel_quality: Some(10.0),
            cargo_mass: Some(1000.0),
            fuel_load: Some(500.0),
            dynamic_mass: Some(true),
            avoid_critical_state: true,
            max_spatial_neighbors: None,
            optimization: None,
        };
        assert!(req.validate("req-ship").is_ok());
    }

    #[test]
    fn test_route_request_rejects_invalid_fuel_quality() {
        let req = RouteRequest {
            from: "Nod".to_string(),
            to: "Brana".to_string(),
            algorithm: RouteAlgorithm::AStar,
            max_jump: None,
            avoid: vec![],
            avoid_gates: false,
            max_temperature: None,
            ship: Some("Reflex".to_string()),
            fuel_quality: Some(150.0),
            cargo_mass: None,
            fuel_load: None,
            dynamic_mass: None,
            avoid_critical_state: true,
            max_spatial_neighbors: None,
            optimization: None,
        };
        let err = req.validate("req-fuel-quality").unwrap_err();
        assert!(err.detail.unwrap().contains("fuel_quality"));
    }

    #[test]
    fn test_scout_range_negative_radius() {
        let req = ScoutRangeRequest {
            system: "Nod".to_string(),
            limit: 10,
            radius: Some(-100.0),
            max_temperature: None,
        };
        let err = req.validate("req-neg-radius").unwrap_err();
        assert!(err.detail.unwrap().contains("radius"));
    }

    #[test]
    fn test_scout_range_request_zero_limit() {
        let req = ScoutRangeRequest {
            system: "Nod".to_string(),
            limit: 0,
            radius: None,
            max_temperature: None,
        };
        let err = req.validate("req-zero-limit").unwrap_err();
        assert!(err.detail.unwrap().contains("limit"));
    }

    #[test]
    fn test_scout_range_request_limit_too_high() {
        let req = ScoutRangeRequest {
            system: "Nod".to_string(),
            radius: Some(80.0),
            max_temperature: None,
            limit: 200,
        };
        let err = req.validate("req-limit-too-high").unwrap_err();
        assert!(err.detail.unwrap().contains("limit"));
    }

    #[test]
    fn test_route_request_deserialization_defaults() {
        let json = r#"{"from": "Nod", "to": "Brana"}"#;
        let request: RouteRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.from, "Nod");
        assert_eq!(request.to, "Brana");
        assert!(request.avoid_critical_state);
        assert_eq!(request.algorithm, RouteAlgorithm::AStar);
    }
}
