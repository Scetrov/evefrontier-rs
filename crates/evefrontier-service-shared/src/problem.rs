//! RFC 9457 Problem Details for HTTP APIs.
//!
//! Provides structured error responses following the Problem Details standard.
//! See: <https://www.rfc-editor.org/rfc/rfc9457.html>

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use evefrontier_lib::Error as LibError;

/// Problem type URI for unknown system names.
pub const PROBLEM_UNKNOWN_SYSTEM: &str = "/problems/unknown-system";

/// Problem type URI for routes that cannot be found.
pub const PROBLEM_ROUTE_NOT_FOUND: &str = "/problems/route-not-found";

/// Problem type URI for invalid request parameters.
pub const PROBLEM_INVALID_REQUEST: &str = "/problems/invalid-request";

/// Problem type URI for internal server errors.
pub const PROBLEM_INTERNAL_ERROR: &str = "/problems/internal-error";

/// Problem type URI for service unavailable (e.g., missing spatial index).
pub const PROBLEM_SERVICE_UNAVAILABLE: &str = "/problems/service-unavailable";

/// RFC 9457 Problem Details response structure.
///
/// Provides a consistent format for error responses across all microservice endpoints.
///
/// # Example
///
/// ```
/// use evefrontier_service_shared::{ProblemDetails, PROBLEM_UNKNOWN_SYSTEM};
/// use axum::http::StatusCode;
///
/// let problem = ProblemDetails::new(
///     PROBLEM_UNKNOWN_SYSTEM,
///     "Unknown System",
///     StatusCode::NOT_FOUND,
/// )
/// .with_detail("System 'InvalidName' not found. Did you mean: 'Nod', 'Brana'?")
/// .with_request_id("req-12345");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemDetails {
    /// URI reference identifying the problem type (relative).
    #[serde(rename = "type")]
    pub type_uri: String,

    /// Short, human-readable summary of the problem.
    pub title: String,

    /// HTTP status code for this problem.
    pub status: u16,

    /// Human-readable explanation specific to this occurrence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    /// URI reference identifying the specific occurrence (e.g., request ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,

    /// Content type for this response (always "application/problem+json").
    pub content_type: String,
}

impl ProblemDetails {
    /// Create a new ProblemDetails with required fields.
    pub fn new(type_uri: impl Into<String>, title: impl Into<String>, status: StatusCode) -> Self {
        Self {
            type_uri: type_uri.into(),
            title: title.into(),
            status: status.as_u16(),
            detail: None,
            instance: None,
            content_type: "application/problem+json".to_string(),
        }
    }

    /// Add a detailed explanation of this specific problem occurrence.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Add the request identifier for tracing.
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.instance = Some(request_id.into());
        self
    }

    /// Create a 400 Bad Request problem for invalid input.
    pub fn bad_request(detail: impl Into<String>, request_id: impl Into<String>) -> Self {
        Self::new(
            PROBLEM_INVALID_REQUEST,
            "Invalid Request",
            StatusCode::BAD_REQUEST,
        )
        .with_detail(detail)
        .with_request_id(request_id)
    }

    /// Create a 404 Not Found problem for unknown systems.
    pub fn unknown_system(
        name: &str,
        suggestions: &[String],
        request_id: impl Into<String>,
    ) -> Self {
        let detail = if suggestions.is_empty() {
            format!("System '{}' not found", name)
        } else {
            format!(
                "System '{}' not found. Did you mean: {}?",
                name,
                suggestions.join(", ")
            )
        };

        Self::new(
            PROBLEM_UNKNOWN_SYSTEM,
            "Unknown System",
            StatusCode::NOT_FOUND,
        )
        .with_detail(detail)
        .with_request_id(request_id)
    }

    /// Create a 404 Not Found problem for unreachable routes.
    pub fn route_not_found(start: &str, goal: &str, request_id: impl Into<String>) -> Self {
        Self::new(
            PROBLEM_ROUTE_NOT_FOUND,
            "Route Not Found",
            StatusCode::NOT_FOUND,
        )
        .with_detail(format!("No route exists from '{}' to '{}'", start, goal))
        .with_request_id(request_id)
    }

    /// Create a 500 Internal Server Error problem.
    pub fn internal_error(detail: impl Into<String>, request_id: impl Into<String>) -> Self {
        Self::new(
            PROBLEM_INTERNAL_ERROR,
            "Internal Error",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .with_detail(detail)
        .with_request_id(request_id)
    }

    /// Create a 503 Service Unavailable problem.
    pub fn service_unavailable(detail: impl Into<String>, request_id: impl Into<String>) -> Self {
        Self::new(
            PROBLEM_SERVICE_UNAVAILABLE,
            "Service Unavailable",
            StatusCode::SERVICE_UNAVAILABLE,
        )
        .with_detail(detail)
        .with_request_id(request_id)
    }
}

impl std::fmt::Display for ProblemDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            self.title,
            self.detail.as_deref().unwrap_or("")
        )
    }
}

impl std::error::Error for ProblemDetails {}

/// Implement IntoResponse for axum to return ProblemDetails as HTTP responses.
impl IntoResponse for ProblemDetails {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        // Set the content-type header to application/problem+json
        let mut response = Json(&self).into_response();
        response.headers_mut().insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static("application/problem+json"),
        );

        // Update status code
        *response.status_mut() = status;
        response
    }
}

/// Convert library errors to ProblemDetails.
///
/// The `request_id` must be provided separately since library errors don't have it.
pub fn from_lib_error(error: &LibError, request_id: &str) -> ProblemDetails {
    match error {
        LibError::UnknownSystem { name, suggestions } => {
            ProblemDetails::unknown_system(name, suggestions, request_id)
        }
        LibError::RouteNotFound { start, goal } => {
            ProblemDetails::route_not_found(start, goal, request_id)
        }
        LibError::DatasetNotFound { path } => ProblemDetails::service_unavailable(
            format!("Dataset not available at {}", path.display()),
            request_id,
        ),
        LibError::UnsupportedSchema => {
            ProblemDetails::internal_error("Unsupported dataset schema", request_id)
        }
        _ => ProblemDetails::internal_error(error.to_string(), request_id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_problem_details_new() {
        let problem = ProblemDetails::new(
            PROBLEM_UNKNOWN_SYSTEM,
            "Unknown System",
            StatusCode::NOT_FOUND,
        );
        assert_eq!(problem.type_uri, PROBLEM_UNKNOWN_SYSTEM);
        assert_eq!(problem.title, "Unknown System");
        assert_eq!(problem.status, 404);
        assert_eq!(problem.content_type, "application/problem+json");
    }

    #[test]
    fn test_problem_details_with_detail() {
        let problem = ProblemDetails::new(
            PROBLEM_INVALID_REQUEST,
            "Bad Request",
            StatusCode::BAD_REQUEST,
        )
        .with_detail("Missing required field 'from'");

        assert_eq!(
            problem.detail.as_deref(),
            Some("Missing required field 'from'")
        );
    }

    #[test]
    fn test_problem_details_bad_request() {
        let problem = ProblemDetails::bad_request("Invalid JSON", "req-123");
        assert_eq!(problem.status, 400);
        assert_eq!(problem.instance.as_deref(), Some("req-123"));
    }

    #[test]
    fn test_problem_details_unknown_system_with_suggestions() {
        let suggestions = vec!["Nod".to_string(), "Node".to_string()];
        let problem = ProblemDetails::unknown_system("Nodd", &suggestions, "req-456");

        assert_eq!(problem.status, 404);
        assert!(problem.detail.as_deref().unwrap().contains("Nodd"));
        assert!(problem.detail.as_deref().unwrap().contains("Nod, Node"));
    }

    #[test]
    fn test_problem_details_unknown_system_no_suggestions() {
        let problem = ProblemDetails::unknown_system("XYZ", &[], "req-789");

        assert!(problem.detail.as_deref().unwrap().contains("XYZ"));
        assert!(!problem.detail.as_deref().unwrap().contains("Did you mean"));
    }

    #[test]
    fn test_problem_details_serialization() {
        let problem = ProblemDetails::bad_request("Test error", "req-test");
        let json = serde_json::to_string(&problem).unwrap();

        assert!(json.contains("\"type\":\"/problems/invalid-request\""));
        assert!(json.contains("\"title\":\"Invalid Request\""));
        assert!(json.contains("\"status\":400"));
        assert!(json.contains("\"detail\":\"Test error\""));
        assert!(json.contains("\"instance\":\"req-test\""));
    }

    #[test]
    fn test_from_lib_error_unknown_system() {
        let error = LibError::UnknownSystem {
            name: "TestSystem".to_string(),
            suggestions: vec!["Nod".to_string()],
        };
        let problem = from_lib_error(&error, "req-lib");

        assert_eq!(problem.type_uri, PROBLEM_UNKNOWN_SYSTEM);
        assert_eq!(problem.status, 404);
    }

    #[test]
    fn test_from_lib_error_route_not_found() {
        let error = LibError::RouteNotFound {
            start: "A".to_string(),
            goal: "B".to_string(),
        };
        let problem = from_lib_error(&error, "req-route");

        assert_eq!(problem.type_uri, PROBLEM_ROUTE_NOT_FOUND);
        assert!(problem.detail.as_deref().unwrap().contains("A"));
        assert!(problem.detail.as_deref().unwrap().contains("B"));
    }
}
