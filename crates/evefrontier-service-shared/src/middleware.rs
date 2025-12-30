//! HTTP middleware for EVE Frontier microservices.
//!
//! This module provides:
//! - [`RequestId`]: Newtype for correlation ID extraction/generation
//! - [`extract_or_generate_request_id`]: Extract X-Request-ID header or generate UUID v7
//! - [`MetricsLayer`]: Tower middleware for recording HTTP metrics
//!
//! # Request ID Propagation
//!
//! The middleware extracts `X-Request-ID` header if present, otherwise generates
//! a new UUID v7 (time-sortable). The ID is injected into tracing spans for
//! correlation across log entries.
//!
//! # Metrics Recording
//!
//! The `MetricsLayer` records:
//! - `http_requests_total`: Counter by method, path, status bucket
//! - `http_request_duration_seconds`: Histogram by method, path
//! - `http_request_size_bytes`: Histogram by method, path
//! - `http_response_size_bytes`: Histogram by method, path

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

use axum::http::{HeaderMap, Request, Response};
use pin_project_lite::pin_project;
use tower::{Layer, Service};
use tracing::{info_span, Span};
use uuid::Uuid;

/// Newtype wrapper for request correlation IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestId(pub String);

impl RequestId {
    /// Create a new request ID from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generate a new UUID v7 request ID.
    pub fn generate() -> Self {
        Self(Uuid::now_v7().to_string())
    }

    /// Get the request ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for RequestId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for RequestId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Extract the request ID from headers or generate a new UUID v7.
///
/// Looks for the `X-Request-ID` header (case-insensitive). If not present
/// or invalid UTF-8, generates a new UUID v7 (time-sortable).
///
/// # Arguments
///
/// * `headers` - HTTP headers to search for X-Request-ID
///
/// # Returns
///
/// A `RequestId` containing either the extracted ID or a newly generated UUID v7.
pub fn extract_or_generate_request_id(headers: &HeaderMap) -> RequestId {
    headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(RequestId::from)
        .unwrap_or_else(RequestId::generate)
}

/// Normalize a request path for metric labels.
///
/// Strips query parameters and normalizes path segments to prevent
/// cardinality explosion in metrics (NFR-004).
///
/// # Arguments
///
/// * `path` - The full request path (may include query string)
///
/// # Returns
///
/// Normalized path without query parameters.
pub fn normalize_path(path: &str) -> &str {
    // Strip query string
    path.split('?').next().unwrap_or(path)
}

/// Convert HTTP status code to bucket label.
///
/// Groups status codes into buckets: "2xx", "3xx", "4xx", "5xx".
fn status_bucket(status: u16) -> &'static str {
    match status {
        200..=299 => "2xx",
        300..=399 => "3xx",
        400..=499 => "4xx",
        500..=599 => "5xx",
        _ => "other",
    }
}

// =============================================================================
// MetricsLayer - Tower middleware for HTTP metrics
// =============================================================================

/// Tower layer for recording HTTP metrics.
///
/// Wraps services to record:
/// - `http_requests_total`: Counter by method, path, status bucket
/// - `http_request_duration_seconds`: Histogram by method, path
/// - `http_request_size_bytes`: Histogram by method, path
/// - `http_response_size_bytes`: Histogram by method, path
#[derive(Debug, Clone)]
pub struct MetricsLayer;

impl<S> Layer<S> for MetricsLayer {
    type Service = MetricsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MetricsMiddleware { inner }
    }
}

/// Middleware service that records HTTP metrics.
#[derive(Debug, Clone)]
pub struct MetricsMiddleware<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for MetricsMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send,
    ReqBody: http_body::Body + Send + 'static,
    ResBody: http_body::Body + Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = MetricsFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let start = Instant::now();

        // Extract request metadata for metrics
        let method = req.method().to_string();
        let path = normalize_path(req.uri().path()).to_string();

        // Record request size
        if let Some(content_length) = req.headers().get(http::header::CONTENT_LENGTH) {
            if let Ok(size) = content_length.to_str().unwrap_or("0").parse::<f64>() {
                metrics::histogram!(
                    "http_request_size_bytes",
                    "method" => method.clone(),
                    "path" => path.clone()
                )
                .record(size);
            }
        }

        // Extract or generate request ID for tracing
        let request_id = extract_or_generate_request_id(req.headers());
        let remote_addr = req
            .extensions()
            .get::<std::net::SocketAddr>()
            .map(|a| a.to_string());

        // Create request span with correlation ID
        let span = info_span!(
            "request",
            request_id = %request_id,
            method = %method,
            path = %path,
            remote_addr = remote_addr.as_deref().unwrap_or("-"),
        );

        {
            let _enter = span.enter();
            tracing::info!("handling request");
        }

        let future = self.inner.call(req);

        MetricsFuture {
            inner: future,
            start,
            method,
            path,
            request_id,
            span,
        }
    }
}

pin_project! {
    /// Future wrapper that records metrics on completion.
    pub struct MetricsFuture<F> {
        #[pin]
        inner: F,
        start: Instant,
        method: String,
        path: String,
        request_id: RequestId,
        span: Span,
    }
}

impl<F, ResBody, E> Future for MetricsFuture<F>
where
    F: Future<Output = Result<Response<ResBody>, E>>,
    ResBody: http_body::Body,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _enter = this.span.enter();

        match this.inner.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(result) => {
                let duration = this.start.elapsed();
                let duration_secs = duration.as_secs_f64();
                let latency_ms = duration.as_secs_f64() * 1000.0;

                match &result {
                    Ok(response) => {
                        let status = response.status().as_u16();
                        let status_label = status_bucket(status);

                        // Record request counter
                        metrics::counter!(
                            "http_requests_total",
                            "method" => this.method.clone(),
                            "path" => this.path.clone(),
                            "status" => status_label
                        )
                        .increment(1);

                        // Record duration histogram
                        metrics::histogram!(
                            "http_request_duration_seconds",
                            "method" => this.method.clone(),
                            "path" => this.path.clone()
                        )
                        .record(duration_secs);

                        // Record response size if available
                        if let Some(content_length) = response.headers().get(http::header::CONTENT_LENGTH) {
                            if let Ok(size) = content_length.to_str().unwrap_or("0").parse::<f64>() {
                                metrics::histogram!(
                                    "http_response_size_bytes",
                                    "method" => this.method.clone(),
                                    "path" => this.path.clone()
                                )
                                .record(size);
                            }
                        }

                        tracing::info!(
                            status = status,
                            latency_ms = latency_ms,
                            "request completed"
                        );
                    }
                    Err(_) => {
                        // Record as 5xx for errors
                        metrics::counter!(
                            "http_requests_total",
                            "method" => this.method.clone(),
                            "path" => this.path.clone(),
                            "status" => "5xx"
                        )
                        .increment(1);

                        metrics::histogram!(
                            "http_request_duration_seconds",
                            "method" => this.method.clone(),
                            "path" => this.path.clone()
                        )
                        .record(duration_secs);

                        tracing::error!(
                            latency_ms = latency_ms,
                            "request failed"
                        );
                    }
                }

                Poll::Ready(result)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_request_id_generate() {
        let id1 = RequestId::generate();
        let id2 = RequestId::generate();

        // UUIDs should be unique
        assert_ne!(id1, id2);

        // Should be valid UUID format (36 chars with hyphens)
        assert_eq!(id1.as_str().len(), 36);
        assert!(id1.as_str().contains('-'));
    }

    #[test]
    fn test_extract_request_id_from_header() {
        let mut headers = HeaderMap::new();
        headers.insert("x-request-id", HeaderValue::from_static("test-123"));

        let id = extract_or_generate_request_id(&headers);
        assert_eq!(id.as_str(), "test-123");
    }

    #[test]
    fn test_extract_request_id_case_insensitive() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Request-ID", HeaderValue::from_static("test-456"));

        let id = extract_or_generate_request_id(&headers);
        assert_eq!(id.as_str(), "test-456");
    }

    #[test]
    fn test_extract_request_id_generates_when_missing() {
        let headers = HeaderMap::new();
        let id = extract_or_generate_request_id(&headers);

        // Should be a valid UUID
        assert_eq!(id.as_str().len(), 36);
    }

    #[test]
    fn test_extract_request_id_generates_when_empty() {
        let mut headers = HeaderMap::new();
        headers.insert("x-request-id", HeaderValue::from_static(""));

        let id = extract_or_generate_request_id(&headers);

        // Should generate a new ID since header is empty
        assert_eq!(id.as_str().len(), 36);
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("/api/v1/route"), "/api/v1/route");
        assert_eq!(normalize_path("/api/v1/route?from=Nod"), "/api/v1/route");
        assert_eq!(normalize_path("/health/ready"), "/health/ready");
        assert_eq!(normalize_path("/"), "/");
    }

    #[test]
    fn test_status_bucket() {
        assert_eq!(status_bucket(200), "2xx");
        assert_eq!(status_bucket(201), "2xx");
        assert_eq!(status_bucket(204), "2xx");
        assert_eq!(status_bucket(301), "3xx");
        assert_eq!(status_bucket(404), "4xx");
        assert_eq!(status_bucket(422), "4xx");
        assert_eq!(status_bucket(500), "5xx");
        assert_eq!(status_bucket(503), "5xx");
    }
}
