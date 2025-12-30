# Research: Microservices Observability

**Feature**: 008-microservices-observability  
**Date**: 2025-12-30  
**Status**: Complete

## Overview

This document consolidates research findings for implementing observability infrastructure in the
EVE Frontier microservices. Each section addresses a specific technical question identified during
the planning phase.

---

## R-001: Prometheus Metrics in Rust with axum

### Context

The EVE Frontier microservices need to expose metrics for monitoring. We need to choose the right
crate ecosystem for Prometheus metrics that integrates well with our existing axum + tower stack.

### Research

Evaluated three approaches for Prometheus metrics in Rust:

1. **`metrics` + `metrics-exporter-prometheus`** (Recommended)
   - Facade crate similar to `log`/`tracing` pattern
   - `metrics::counter!`, `metrics::histogram!`, `metrics::gauge!` macros
   - PrometheusBuilder provides HTTP handler for `/metrics` endpoint
   - Labels via `.with_label("key", "value")` or inline `counter!("name", "label" => "value")`
   - Active maintenance, 2.3M downloads

2. **`prometheus` crate**
   - Direct port of Go prometheus client
   - More verbose: requires manual registry, Desc, etc.
   - Thread-safe but requires explicit Arc<> sharing
   - Less idiomatic Rust

3. **`opentelemetry-prometheus`**
   - Full OpenTelemetry SDK with Prometheus export
   - Heavier dependency tree
   - Overkill when we only need Prometheus scraping

### Decision

Use `metrics` crate with `metrics-exporter-prometheus` recorder.

### Rationale

- Idiomatic Rust API with macros
- Minimal boilerplate for common patterns
- Compatible with tower middleware ecosystem
- Active maintenance and large user base

### Implementation Notes

```toml
# Cargo.toml additions
metrics = "0.23"
metrics-exporter-prometheus = "0.15"
```

```rust
use metrics_exporter_prometheus::PrometheusBuilder;

// In main(), before starting server:
let recorder = PrometheusBuilder::new()
    .with_http_listener(([0, 0, 0, 0], 9090))
    .install()
    .expect("failed to install Prometheus recorder");

// In handlers:
metrics::counter!("http_requests_total", "method" => "POST", "path" => "/route").increment(1);
metrics::histogram!("http_request_duration_seconds", "method" => "POST").record(duration);
```

---

## R-002: Request Metrics Middleware for axum

### Context

We need automatic request-level metrics (request count, duration histogram) without manually
instrumenting every handler. This should integrate with the existing tower middleware stack.

### Research

Evaluated three middleware approaches:

1. **Extend existing `TraceLayer`** (Recommended)
   - Already have `tower_http::trace::TraceLayer` configured
   - Can add `on_request` / `on_response` callbacks for metrics
   - Keeps tracing and metrics in sync

2. **Dedicated `MetricsLayer`**
   - Custom tower::Layer implementation
   - Clean separation of concerns
   - More code to maintain

3. **`axum-prometheus` crate**
   - Abandoned (last commit 2022)
   - Uses older prometheus crate directly
   - Not recommended

### Decision

Create a dedicated `MetricsLayer` using tower for clean separation, but leverage the `metrics`
crate macros for actual recording. This keeps metrics isolated from tracing concerns.

### Rationale

- TraceLayer is for distributed tracing spans, not metrics
- Dedicated layer allows enabling/disabling metrics independently
- Cleaner code organization in evefrontier-service-shared

### Implementation Notes

```rust
use std::task::{Context, Poll};
use std::time::Instant;
use tower::{Layer, Service};
use metrics::{counter, histogram};

#[derive(Clone)]
pub struct MetricsLayer;

impl<S> Layer<S> for MetricsLayer {
    type Service = MetricsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MetricsMiddleware { inner }
    }
}

pub struct MetricsMiddleware<S> {
    inner: S,
}

// Implementation records request count and duration
```

---

## R-003: Structured JSON Logging Configuration

### Context

Production logs should be structured JSON for log aggregation systems (Loki, ELK, CloudWatch).
Development logs should be human-readable. We need configurable format switching.

### Research

Current setup uses `tracing_subscriber::fmt::layer().json()` which produces:

```json
{"timestamp":"2025-12-30T10:00:00Z","level":"INFO","target":"route_service","message":"handling request"}
```

Enhancements needed:
- Add `request_id` field to all request-scoped logs
- Add `service` field for multi-service log streams
- Support `LOG_FORMAT=text` for development
- Control verbosity of span/event fields

### Decision

Enhance `init_tracing()` in each service to:
1. Read `LOG_FORMAT` env var (default: `json`)
2. Configure appropriate formatter layer
3. Always include `service` field

### Rationale

- Minimal code change to existing pattern
- Environment-based configuration is 12-factor compliant
- JSON by default is production-safe

### Implementation Notes

```rust
fn init_tracing(service_name: &'static str) {
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "json".to_string());

    let registry = tracing_subscriber::registry().with(filter);

    match log_format.as_str() {
        "text" | "pretty" => {
            registry.with(fmt::layer().pretty()).init();
        }
        _ => {
            registry
                .with(fmt::layer().json().with_current_span(false))
                .init();
        }
    }
}
```

---

## R-004: Correlation ID Propagation

### Context

Requests should have unique IDs for tracing across logs. If the caller provides `X-Request-ID`
header, we should use it; otherwise, generate one.

### Research

Evaluated ID generation strategies:

1. **UUID v7** (Recommended)
   - Time-sortable (timestamp prefix)
   - Unique across distributed systems
   - Standard format recognized by tooling
   - `uuid` crate with `v7` feature

2. **UUID v4**
   - Random, not sortable
   - Standard but less debuggable

3. **Timestamp-based (current)**
   - Simple but collision risk under load
   - Not standard format

4. **Snowflake IDs**
   - Requires configuration (machine ID)
   - Overkill for request correlation

### Decision

Use UUID v7 for generated request IDs, accept `X-Request-ID` header if provided.

### Rationale

- Time-sortable helps debugging
- Standard format integrates with logging tools
- Header propagation supports distributed tracing

### Implementation Notes

```rust
fn extract_or_generate_request_id(headers: &HeaderMap) -> String {
    headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| uuid::Uuid::now_v7().to_string())
}
```

---

## R-005: Health Check Enhancements

### Context

Current health endpoints return basic status. Kubernetes operators need more detail for debugging
readiness issues (which dependency is failing?).

### Research

Best practices for health checks:
- **Liveness**: Simple, fast, no dependencies (is the process alive?)
- **Readiness**: Check critical dependencies (can we serve traffic?)
- **Startup**: For slow initialization (currently not needed)

Current implementation:
- `/health/live` returns `{"status":"ok","service":"...","version":"..."}`
- `/health/ready` returns same plus `systems_loaded`, `spatial_index_ready`

Enhancements:
- Add `checks` map with individual dependency status
- Return 503 if any critical check fails
- Keep response JSON consistent

### Decision

Enhance `HealthStatus` with detailed checks map. Keep liveness simple.

### Rationale

- Backwards compatible (adds fields, doesn't change existing)
- Provides actionable debugging info
- Standard pattern for Kubernetes health checks

### Implementation Notes

```rust
#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub service: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<HashMap<String, CheckStatus>>,
}

#[derive(Serialize)]
pub struct CheckStatus {
    pub status: String, // "ok", "degraded", "error"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
```

---

## R-006: Grafana Dashboard Design

### Context

Provide a ready-to-import Grafana dashboard for EVE Frontier services that shows operational
metrics at a glance.

### Research

Grafana dashboard best practices:
- Use variables for multi-service/instance filtering
- Follow RED method: Rate, Errors, Duration
- Include business metrics for domain insight
- Use appropriate visualizations (stat, time series, heatmap)

Dashboard structure:
1. **Row: Overview**
   - Total request rate (stat)
   - Error rate percentage (stat)
   - P95 latency (stat)

2. **Row: Traffic**
   - Request rate by service (time series)
   - Request rate by endpoint (time series)

3. **Row: Errors**
   - Error rate by service (time series)
   - Errors by status code (bar chart)

4. **Row: Latency**
   - P50/P95/P99 latency (time series)
   - Latency heatmap (heatmap)

5. **Row: Business Metrics**
   - Routes calculated by algorithm (time series)
   - Systems queried (time series)

### Decision

Create single dashboard JSON with variables for service selection. Export as Grafana 10.x format.

### Rationale

- Single dashboard simplifies operator experience
- Variables allow filtering without multiple dashboards
- JSON export is portable across Grafana installations

### File Location

`docs/dashboards/evefrontier.json`

---

## Summary

| Topic | Decision | Key Dependency |
|-------|----------|----------------|
| Metrics crate | `metrics` + `metrics-exporter-prometheus` | metrics 0.23, metrics-exporter-prometheus 0.15 |
| Request middleware | Custom `MetricsLayer` with tower | tower (existing) |
| Logging format | Environment-based json/text | tracing-subscriber (existing) |
| Correlation ID | UUID v7, X-Request-ID header | uuid with v7 feature |
| Health checks | Enhanced checks map | None (extend existing) |
| Dashboard | Single JSON with variables | N/A (static file) |

All research items resolved. No NEEDS CLARIFICATION remaining.
