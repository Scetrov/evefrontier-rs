//! Request types and validation for HTTP endpoints.

use serde::{Deserialize, Serialize};

use crate::ProblemDetails;

/// Validation trait for request types.
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
        let req = RouteRequest {
            from: "Nod".to_string(),
            to: "Brana".to_string(),
            algorithm: RouteAlgorithm::AStar,
            max_jump: Some(80.0),
            avoid: vec![],
            avoid_gates: false,
            max_temperature: None,
        };
        assert!(req.validate("test").is_ok());
    }

    #[test]
    fn test_route_request_empty_from() {
        let req = RouteRequest {
            from: "".to_string(),
            to: "Brana".to_string(),
            algorithm: RouteAlgorithm::Bfs,
            max_jump: None,
            avoid: vec![],
            avoid_gates: false,
            max_temperature: None,
        };
        let err = req.validate("test").unwrap_err();
        assert!(err.detail.as_deref().unwrap().contains("'from'"));
    }

    #[test]
    fn test_route_request_empty_to() {
        let req = RouteRequest {
            from: "Nod".to_string(),
            to: "   ".to_string(),
            algorithm: RouteAlgorithm::Bfs,
            max_jump: None,
            avoid: vec![],
            avoid_gates: false,
            max_temperature: None,
        };
        let err = req.validate("test").unwrap_err();
        assert!(err.detail.as_deref().unwrap().contains("'to'"));
    }

    #[test]
    fn test_route_request_negative_max_jump() {
        let req = RouteRequest {
            from: "Nod".to_string(),
            to: "Brana".to_string(),
            algorithm: RouteAlgorithm::Dijkstra,
            max_jump: Some(-10.0),
            avoid: vec![],
            avoid_gates: false,
            max_temperature: None,
        };
        let err = req.validate("test").unwrap_err();
        assert!(err.detail.as_deref().unwrap().contains("'max_jump'"));
    }

    #[test]
    fn test_route_algorithm_serialization() {
        let algo = RouteAlgorithm::AStar;
        let json = serde_json::to_string(&algo).unwrap();
        assert_eq!(json, "\"a-star\"");

        let bfs: RouteAlgorithm = serde_json::from_str("\"bfs\"").unwrap();
        assert_eq!(bfs, RouteAlgorithm::Bfs);
    }

    #[test]
    fn test_scout_gates_request_valid() {
        let req = ScoutGatesRequest {
            system: "Nod".to_string(),
        };
        assert!(req.validate("test").is_ok());
    }

    #[test]
    fn test_scout_gates_request_empty() {
        let req = ScoutGatesRequest {
            system: "".to_string(),
        };
        let err = req.validate("test").unwrap_err();
        assert!(err.detail.as_deref().unwrap().contains("'system'"));
    }

    #[test]
    fn test_scout_range_request_valid() {
        let req = ScoutRangeRequest {
            system: "Nod".to_string(),
            limit: 10,
            radius: Some(50.0),
            max_temperature: Some(8000.0),
        };
        assert!(req.validate("test").is_ok());
    }

    #[test]
    fn test_scout_range_request_zero_limit() {
        let req = ScoutRangeRequest {
            system: "Nod".to_string(),
            limit: 0,
            radius: None,
            max_temperature: None,
        };
        let err = req.validate("test").unwrap_err();
        assert!(err.detail.as_deref().unwrap().contains("'limit'"));
    }

    #[test]
    fn test_scout_range_request_limit_too_high() {
        let req = ScoutRangeRequest {
            system: "Nod".to_string(),
            limit: 101,
            radius: None,
            max_temperature: None,
        };
        let err = req.validate("test").unwrap_err();
        assert!(err.detail.as_deref().unwrap().contains("exceed 100"));
    }

    #[test]
    fn test_scout_range_request_negative_radius() {
        let req = ScoutRangeRequest {
            system: "Nod".to_string(),
            limit: 10,
            radius: Some(-5.0),
            max_temperature: None,
        };
        let err = req.validate("test").unwrap_err();
        assert!(err.detail.as_deref().unwrap().contains("'radius'"));
    }

    #[test]
    fn test_route_request_deserialization_defaults() {
        let json = r#"{"from":"Nod","to":"Brana"}"#;
        let req: RouteRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.algorithm, RouteAlgorithm::AStar); // default
        assert!(req.avoid.is_empty()); // default
        assert!(!req.avoid_gates); // default
    }

    #[test]
    fn test_scout_range_request_deserialization_defaults() {
        let json = r#"{"system":"Nod"}"#;
        let req: ScoutRangeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.limit, 10); // default_limit()
    }
}
