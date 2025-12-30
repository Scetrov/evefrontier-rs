//! Response wrapper for successful HTTP responses.

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

/// Wrapper for successful responses with content type metadata.
///
/// This provides symmetry with `ProblemDetails` by including content type
/// information in the response body.
///
/// # Example
///
/// ```
/// use evefrontier_service_shared::ServiceResponse;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct RouteResult {
///     hops: usize,
///     distance: f64,
/// }
///
/// let result = RouteResult { hops: 5, distance: 123.4 };
/// let response = ServiceResponse::new(result);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResponse<T> {
    /// The actual response payload.
    #[serde(flatten)]
    pub data: T,

    /// Content type for this response.
    pub content_type: String,
}

impl<T> ServiceResponse<T> {
    /// Create a new successful response with the default content type.
    pub fn new(data: T) -> Self {
        Self {
            data,
            content_type: "application/json".to_string(),
        }
    }

    /// Create a response with a custom content type.
    pub fn with_content_type(data: T, content_type: impl Into<String>) -> Self {
        Self {
            data,
            content_type: content_type.into(),
        }
    }
}

impl<T> From<T> for ServiceResponse<T> {
    fn from(data: T) -> Self {
        Self::new(data)
    }
}

/// Implement IntoResponse for axum to return ServiceResponse as HTTP responses.
impl<T: Serialize> IntoResponse for ServiceResponse<T> {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestData {
        value: i32,
    }

    #[test]
    fn test_response_serialization() {
        let response = ServiceResponse::new(TestData { value: 42 });
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"value\":42"));
        assert!(json.contains("\"content_type\":\"application/json\""));
    }

    #[test]
    fn test_custom_content_type() {
        let response = ServiceResponse::with_content_type(TestData { value: 1 }, "text/plain");
        assert_eq!(response.content_type, "text/plain");
    }

    #[test]
    fn test_response_from_trait() {
        let data = TestData { value: 99 };
        let response: ServiceResponse<TestData> = data.clone().into();
        assert_eq!(response.data, data);
        assert_eq!(response.content_type, "application/json");
    }

    #[test]
    fn test_response_flatten_serialization() {
        // Verify that #[serde(flatten)] works correctly
        #[derive(Debug, Serialize)]
        struct RouteResult {
            hops: usize,
            route: Vec<String>,
        }

        let result = RouteResult {
            hops: 3,
            route: vec!["Nod".to_string(), "Brana".to_string()],
        };
        let response = ServiceResponse::new(result);
        let json = serde_json::to_string(&response).unwrap();

        // Fields should be at the top level, not nested under "data"
        assert!(json.contains("\"hops\":3"));
        assert!(json.contains("\"route\":["));
        assert!(!json.contains("\"data\":{"));
    }

    #[test]
    fn test_response_deserialization() {
        let json = r#"{"value":42,"content_type":"application/json"}"#;
        let response: ServiceResponse<TestData> = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.value, 42);
        assert_eq!(response.content_type, "application/json");
    }
}
