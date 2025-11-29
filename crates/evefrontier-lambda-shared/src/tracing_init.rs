//! Tracing initialization for Lambda functions.
//!
//! Configures JSON-formatted tracing output suitable for CloudWatch Logs.

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize tracing with JSON formatting for CloudWatch Logs.
///
/// This should be called once at the start of the Lambda `main` function,
/// before calling `lambda_runtime::run()`.
///
/// The log level can be controlled via the `RUST_LOG` environment variable.
/// Defaults to `info` if not set.
///
/// # Example
///
/// ```no_run
/// use evefrontier_lambda_shared::init_tracing;
///
/// #[tokio::main]
/// async fn main() -> Result<(), lambda_runtime::Error> {
///     init_tracing();
///     // ... rest of Lambda setup
///     Ok(())
/// }
/// ```
pub fn init_tracing() {
    // Use RUST_LOG env var, defaulting to info level
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Configure JSON formatting for CloudWatch
    let fmt_layer = fmt::layer()
        .json()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .flatten_event(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}

#[cfg(test)]
mod tests {
    // Tracing initialization is global state, so we can't easily test it
    // in unit tests without affecting other tests. Integration tests or
    // manual verification is preferred.
}
