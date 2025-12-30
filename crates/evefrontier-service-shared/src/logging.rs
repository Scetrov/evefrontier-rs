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
//! - `SERVICE_NAME`: Service name to include in log entries (optional)
//!
//! # Example
//!
//! ```no_run
//! use evefrontier_service_shared::logging::{LoggingConfig, init_logging};
//!
//! // Initialize logging at startup (reads LOG_FORMAT from environment)
//! let config = LoggingConfig::from_env().with_service("route");
//! init_logging(&config);
//! ```

use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

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

impl std::str::FromStr for LogFormat {
    type Err = std::convert::Infallible;

    /// Parse log format from string.
    ///
    /// Accepts "json", "text", or "pretty" (alias for text).
    /// Returns `Json` for any other value.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "text" | "pretty" => LogFormat::Text,
            _ => LogFormat::Json,
        })
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
            .map(|v| v.parse::<LogFormat>().unwrap_or(LogFormat::Json))
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
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

    let registry = tracing_subscriber::registry().with(filter);

    match config.format {
        LogFormat::Text => {
            registry.with(fmt::layer().pretty()).init();
        }
        LogFormat::Json => {
            // JSON layer with service field injection
            if let Some(service) = &config.service {
                // Clone service name for the closure
                let service_name = service.clone();
                let json_layer = fmt::layer()
                    .json()
                    .with_current_span(false)
                    .with_span_list(false)
                    .map_event_format(move |_| {
                        // Custom JSON formatter that includes service field
                        ServiceJsonFormat {
                            service: service_name.clone(),
                        }
                    });

                registry.with(json_layer).init();
            } else {
                let json_layer = fmt::layer()
                    .json()
                    .with_current_span(false)
                    .with_span_list(false);

                registry.with(json_layer).init();
            }
        }
    }
}

/// Custom JSON formatter that includes the service field.
struct ServiceJsonFormat {
    service: String,
}

impl<S, N> tracing_subscriber::fmt::FormatEvent<S, N> for ServiceJsonFormat
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> tracing_subscriber::fmt::FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
        mut writer: tracing_subscriber::fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let metadata = event.metadata();
        let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true);

        // Collect event fields
        let mut visitor = JsonVisitor::default();
        event.record(&mut visitor);

        // Build JSON object
        let mut log = json!({
            "timestamp": timestamp,
            "level": metadata.level().to_string(),
            "target": metadata.target(),
            "service": self.service,
        });

        // Add message if present
        if let Some(message) = visitor.message {
            log["message"] = json!(message);
        }

        // Add other fields
        for (key, value) in visitor.fields {
            log[key] = value;
        }

        // Write JSON line
        writeln!(writer, "{}", log)
    }
}

/// Visitor to collect event fields as JSON values.
#[derive(Default)]
struct JsonVisitor {
    message: Option<String>,
    fields: Vec<(String, serde_json::Value)>,
}

impl tracing::field::Visit for JsonVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{:?}", value));
        } else {
            self.fields
                .push((field.name().to_string(), json!(format!("{:?}", value))));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields.push((field.name().to_string(), json!(value)));
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields.push((field.name().to_string(), json!(value)));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields.push((field.name().to_string(), json!(value)));
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.fields.push((field.name().to_string(), json!(value)));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.fields.push((field.name().to_string(), json!(value)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_format_from_str() {
        assert_eq!("json".parse::<LogFormat>().unwrap(), LogFormat::Json);
        assert_eq!("JSON".parse::<LogFormat>().unwrap(), LogFormat::Json);
        assert_eq!("text".parse::<LogFormat>().unwrap(), LogFormat::Text);
        assert_eq!("TEXT".parse::<LogFormat>().unwrap(), LogFormat::Text);
        assert_eq!("pretty".parse::<LogFormat>().unwrap(), LogFormat::Text);
        assert_eq!("PRETTY".parse::<LogFormat>().unwrap(), LogFormat::Text);
        assert_eq!("unknown".parse::<LogFormat>().unwrap(), LogFormat::Json);
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
