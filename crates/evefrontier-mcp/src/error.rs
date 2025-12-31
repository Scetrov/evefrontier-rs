//! Error types and RFC 9457-style problem details for the MCP server
//!
//! This module defines a unified error type for the MCP server that
//! can be serialized as RFC 9457 Problem Details for HTTP APIs or
//! MCP error responses.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

/// Result type for MCP operations
pub type Result<T> = std::result::Result<T, Error>;

/// MCP Server error type implementing RFC 9457 Problem Details
#[derive(Debug, Error, Serialize, Deserialize, Clone)]
#[error("{message}")]
pub struct Error {
    /// HTTP status-like code (e.g., 400, 404, 500)
    pub code: i32,

    /// Human-readable error message
    pub message: String,

    /// Machine-readable problem type URI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// Additional error context (e.g., system name, algorithm)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,
}

impl Error {
    /// Create a new error with a code and message
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            r#type: None,
            context: None,
        }
    }

    /// Add a problem type URI
    pub fn with_type(mut self, type_uri: impl Into<String>) -> Self {
        self.r#type = Some(type_uri.into());
        self
    }

    /// Add context information as JSON
    pub fn with_context(mut self, context: Value) -> Self {
        self.context = Some(context);
        self
    }

    /// System not found error
    pub fn system_not_found(system_name: impl Into<String>, suggestions: Vec<String>) -> Self {
        let name = system_name.into();
        Self::new(404, format!("System '{}' not found", name))
            .with_type("https://evefrontier.local/errors/system-not-found")
            .with_context(json!({
                "system_name": name,
                "suggestions": suggestions,
                "message": "Did you mean one of these systems?"
            }))
    }

    /// Route not found error
    pub fn route_not_found(origin: impl Into<String>, destination: impl Into<String>) -> Self {
        Self::new(404, "No route found between the specified systems")
            .with_type("https://evefrontier.local/errors/route-not-found")
            .with_context(json!({
                "origin": origin.into(),
                "destination": destination.into(),
                "message": "Try removing constraints or using a different algorithm"
            }))
    }

    /// Invalid parameter error
    pub fn invalid_param(param: impl Into<String>, reason: impl Into<String>) -> Self {
        let p = param.into();
        Self::new(400, format!("Invalid parameter: {}", p))
            .with_type("https://evefrontier.local/errors/invalid-parameter")
            .with_context(json!({
                "parameter": p,
                "reason": reason.into()
            }))
    }

    /// Internal server error
    pub fn internal(reason: impl Into<String>) -> Self {
        Self::new(500, format!("Internal server error: {}", reason.into()))
            .with_type("https://evefrontier.local/errors/internal-error")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::new(400, "Bad request");
        assert_eq!(err.code, 400);
        assert_eq!(err.message, "Bad request");
    }

    #[test]
    fn test_system_not_found() {
        let err = Error::system_not_found("Unknown", vec!["Nod".to_string(), "Brana".to_string()]);
        assert_eq!(err.code, 404);
        assert!(err.message.contains("Unknown"));
        assert!(err.context.is_some());
    }

    #[test]
    fn test_error_serialization() {
        let err = Error::new(400, "test")
            .with_type("test/type")
            .with_context(json!({"key": "value"}));
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("400"));
        assert!(json.contains("test"));
    }
}
