//! Structured logging infrastructure for EVE Frontier microservices.
//!
//! This module provides:
//! - [`LoggingConfig`]: Configuration for the logging system
//! - [`init_logging`]: Initialize tracing with JSON or text formatting
//!
//! # Environment Variables
//!
//! - `LOG_FORMAT`: Output format, either `json` (default) or `text`
//! - `RUST_LOG`: Log level filter (default: `info`)
//!
//! # Example
//!
//! ```no_run
//! use evefrontier_service_shared::logging::{LoggingConfig, init_logging};
//!
//! // Initialize logging at startup (reads LOG_FORMAT from environment)
//! let config = LoggingConfig::from_env();
//! init_logging(&config);
//! ```

use serde::{Deserialize, Serialize};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// Log output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// JSON structured logging (default, production).
    #[default]
    Json,
    /// Human-readable text logging (development).
    Text,
}

impl LogFormat {
    /// Parse log format from string.
    ///
    /// Accepts "json", "text", or "pretty" (alias for text).
    /// Returns `Json` for any other value.
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "text" | "pretty" => LogFormat::Text,
            _ => LogFormat::Json,
        }
    }
}

/// Configuration for the logging system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Output format (json or text).
    pub format: LogFormat,
    /// Log level filter (e.g., "info", "debug", "warn").
    pub level: String,
    /// Service name to include in log entries.
    pub service: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            format: LogFormat::Json,
            level: "info".to_string(),
            service: None,
        }
    }
}

impl LoggingConfig {
    /// Create configuration from environment variables.
    ///
    /// - `LOG_FORMAT`: "json" (default) or "text"
    /// - `RUST_LOG`: Log level filter (default: "info")
    /// - `SERVICE_NAME`: Service name for log entries (optional)
    pub fn from_env() -> Self {
        let format = std::env::var("LOG_FORMAT")
            .map(|v| LogFormat::from_str(&v))
            .unwrap_or(LogFormat::Json);

        let level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        let service = std::env::var("SERVICE_NAME").ok();

        Self {
            format,
            level,
            service,
        }
    }

    /// Create a new configuration with the specified service name.
    pub fn with_service(mut self, service: impl Into<String>) -> Self {
        self.service = Some(service.into());
        self
    }
}

/// Initialize the tracing subscriber with the given configuration.
///
/// This function sets up either JSON or text formatting based on the configuration.
/// It should be called once at application startup.
///
/// # JSON Format (default)
///
/// ```json
/// {"timestamp":"2025-12-30T10:00:00Z","level":"INFO","target":"service::main","message":"...","service":"route"}
/// ```
///
/// # Text Format (development)
///
/// ```text
/// 2025-12-30T10:00:00Z  INFO service::main: handling request
/// ```
pub fn init_logging(config: &LoggingConfig) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    let registry = tracing_subscriber::registry().with(filter);

    match config.format {
        LogFormat::Text => {
            registry
                .with(fmt::layer().pretty())
                .init();
        }
        LogFormat::Json => {
            // JSON layer with service field injection
            let json_layer = fmt::layer()
                .json()
                .with_current_span(false)
                .with_span_list(false);

            registry
                .with(json_layer)
                .init();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_format_from_str() {
        assert_eq!(LogFormat::from_str("json"), LogFormat::Json);
        assert_eq!(LogFormat::from_str("JSON"), LogFormat::Json);
        assert_eq!(LogFormat::from_str("text"), LogFormat::Text);
        assert_eq!(LogFormat::from_str("TEXT"), LogFormat::Text);
        assert_eq!(LogFormat::from_str("pretty"), LogFormat::Text);
        assert_eq!(LogFormat::from_str("PRETTY"), LogFormat::Text);
        assert_eq!(LogFormat::from_str("unknown"), LogFormat::Json);
    }

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.format, LogFormat::Json);
        assert_eq!(config.level, "info");
        assert!(config.service.is_none());
    }

    #[test]
    fn test_logging_config_with_service() {
        let config = LoggingConfig::default().with_service("route");
        assert_eq!(config.service, Some("route".to_string()));
    }
}
