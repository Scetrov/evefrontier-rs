//! Response wrapper for successful Lambda responses.

use serde::{Deserialize, Serialize};

/// Wrapper for successful Lambda responses with content type metadata.
///
/// This provides symmetry with `ProblemDetails` by including content type
/// information in the response body.
///
/// # Example
///
/// ```
/// use evefrontier_lambda_shared::LambdaResponse;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct RouteResult {
///     hops: usize,
///     distance: f64,
/// }
///
/// let result = RouteResult { hops: 5, distance: 123.4 };
/// let response = LambdaResponse::new(result);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LambdaResponse<T> {
    /// The actual response payload.
    #[serde(flatten)]
    pub data: T,

    /// Content type for this response.
    pub content_type: String,
}

impl<T> LambdaResponse<T> {
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

impl<T> From<T> for LambdaResponse<T> {
    fn from(data: T) -> Self {
        Self::new(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    struct TestData {
        value: i32,
    }

    #[test]
    fn test_response_serialization() {
        let response = LambdaResponse::new(TestData { value: 42 });
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"value\":42"));
        assert!(json.contains("\"content_type\":\"application/json\""));
    }

    #[test]
    fn test_custom_content_type() {
        let response = LambdaResponse::with_content_type(TestData { value: 1 }, "text/plain");
        assert_eq!(response.content_type, "text/plain");
    }
}
