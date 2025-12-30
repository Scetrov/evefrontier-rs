# Implementation Plan: Microservices Observability

**Branch**: `008-microservices-observability` | **Date**: 2025-12-30 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/008-microservices-observability/spec.md`

## Summary

Add comprehensive observability infrastructure to EVE Frontier microservices including Prometheus
metrics, structured JSON logging with correlation IDs, enhanced health checks, and documentation
for Grafana dashboards and alerting rules. The implementation leverages the existing `tracing`
infrastructure and adds the `metrics` crate with Prometheus exposition.

## Technical Context

**Language/Version**: Rust 1.91.1 (per `.rust-toolchain`)  
**Primary Dependencies**: axum 0.8, tracing, tracing-subscriber (json feature), metrics,
metrics-exporter-prometheus  
**Storage**: N/A (metrics are in-memory with Prometheus scraping)  
**Testing**: cargo test, integration tests with axum test server  
**Target Platform**: Linux containers (x86_64 + aarch64)  
**Project Type**: Rust workspace with multiple microservice crates  
**Performance Goals**: <1ms metrics overhead per request (p99), <100ms /metrics endpoint response  
**Constraints**: Bounded metric cardinality (no unbounded path labels), JSON structured logs  
**Scale/Scope**: 3 microservices, ~10 metric families per service, ~5 dashboard panels

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Test-Driven Development | ✅ PASS | Tests for metrics middleware, log format, health endpoints |
| II. Library-First Architecture | ✅ PASS | Metrics infrastructure goes in evefrontier-service-shared |
| III. Architecture Decision Records | ⚠️ N/A | No ADR needed - enhancement, not architectural change |
| IV. Clean Code & Cognitive Load | ✅ PASS | Middleware pattern keeps complexity contained |
| V. Security-First Development | ✅ PASS | /metrics endpoint internal-only, no sensitive data exposed |
| VI. Testing Tiers | ✅ PASS | Unit tests + integration tests for HTTP layer |
| VII. Refactoring & Tech Debt | ✅ PASS | Adds new capability, no refactoring of existing code |

**Gate Result**: ✅ PASS - All applicable principles satisfied

## Project Structure

### Documentation (this feature)

```text
specs/008-microservices-observability/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── metrics.md       # Prometheus metrics specification
└── tasks.md             # Phase 2 output (NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
crates/
├── evefrontier-service-shared/
│   └── src/
│       ├── lib.rs           # Re-export metrics module
│       ├── health.rs        # Enhanced health checks (existing, modify)
│       ├── metrics.rs       # NEW: Prometheus metrics layer
│       ├── logging.rs       # NEW: Structured JSON logging setup
│       └── middleware.rs    # NEW: Request tracking middleware
├── evefrontier-service-route/
│   └── src/
│       └── main.rs          # Wire in metrics endpoint + middleware
├── evefrontier-service-scout-gates/
│   └── src/
│       └── main.rs          # Wire in metrics endpoint + middleware
└── evefrontier-service-scout-range/
    └── src/
        └── main.rs          # Wire in metrics endpoint + middleware

docs/
├── OBSERVABILITY.md         # NEW: Operational guide for metrics/logging
└── dashboards/
    └── evefrontier.json     # NEW: Grafana dashboard JSON
```

**Structure Decision**: Observability infrastructure (metrics recorder, middleware) lives in
`evefrontier-service-shared` following the existing pattern. Each microservice main.rs wires in
the shared components. Documentation goes in `docs/` following existing convention.

## Complexity Tracking

> No Constitution violations requiring justification.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |

---

## Phase 0: Research Findings

### R-001: Prometheus Metrics in Rust with axum

**Task**: Research best practices for Prometheus metrics in axum services

**Decision**: Use `metrics` crate with `metrics-exporter-prometheus` recorder

**Rationale**:
- `metrics` crate is the de-facto standard for Rust metrics (facade pattern like `log`/`tracing`)
- `metrics-exporter-prometheus` provides HTTP endpoint via PrometheusBuilder
- Integrates well with axum via `axum::routing::get` for `/metrics` endpoint
- Supports histogram, counter, gauge primitives with labels

**Alternatives considered**:
- `prometheus` crate directly: More verbose, less idiomatic Rust, no facade pattern
- `opentelemetry-prometheus`: Adds OTel complexity when we just need Prometheus scraping
- Custom implementation: Unnecessary when standard crates exist

### R-002: Request Metrics Middleware for axum

**Task**: Research axum middleware patterns for request-level metrics

**Decision**: Use `tower_http::trace::TraceLayer` with custom `on_request`/`on_response` callbacks
combined with `metrics::counter!` and `metrics::histogram!` macros

**Rationale**:
- Tower middleware is the standard for axum layers
- `tower_http::trace::TraceLayer` already exists in our dependencies
- Can augment existing trace layer with metrics recording
- Alternatively, create dedicated MetricsLayer for separation of concerns

**Alternatives considered**:
- `axum-prometheus` crate: Abandoned, last update 2 years ago
- Manual metrics in each handler: Duplicates code, error-prone
- OpenTelemetry auto-instrumentation: Adds complexity beyond our needs

### R-003: Structured JSON Logging Configuration

**Task**: Research tracing-subscriber JSON configuration options

**Decision**: Use `tracing_subscriber::fmt::layer().json()` with custom field configuration

**Rationale**:
- Already using `tracing-subscriber` with json feature enabled
- JSON layer supports custom fields via `with_current_span(false)`, `with_span_list(false)`
- Environment variable `LOG_FORMAT` can switch between json/pretty formatters
- Request ID injection via span fields propagates to all logs

**Alternatives considered**:
- `slog` with JSON formatter: Would require rewriting all log statements
- `log4rs` with JSON: Less integration with tracing spans
- Custom JSON serialization: Reinvents the wheel

### R-004: Correlation ID Propagation

**Task**: Research request ID/correlation ID patterns for distributed systems

**Decision**: Use `X-Request-ID` header if present, otherwise generate UUID7 (time-sortable)

**Rationale**:
- `X-Request-ID` is industry standard header for correlation
- UUID7 provides time-sortability for debugging
- Inject into tracing span so all nested logs include it
- Return in response headers for client debugging

**Alternatives considered**:
- W3C Trace Context (traceparent): More complex, requires OTel infrastructure
- Snowflake IDs: Custom implementation overhead
- Timestamp-based IDs (current): Not collision-safe under load

### R-005: Health Check Enhancements

**Task**: Research Kubernetes health check best practices

**Decision**: Keep current `/health/live` simple, enhance `/health/ready` with detailed checks

**Rationale**:
- Liveness should be simple (process running) - current implementation correct
- Readiness should verify dependencies (database loaded, spatial index available)
- JSON response with individual check status enables debugging
- Match existing `HealthStatus` struct, extend with checks map

**Alternatives considered**:
- gRPC health checking: Not applicable to REST services
- Complex liveness probes: Anti-pattern, causes unnecessary restarts
- Third-party health check libraries: Overkill for our simple needs

### R-006: Grafana Dashboard Design

**Task**: Research Grafana dashboard best practices for microservices

**Decision**: Single dashboard with variable-based service selection, 4 key panels

**Rationale**:
- Variables allow single dashboard for all 3 services
- RED method (Rate, Errors, Duration) covers operational essentials
- Business metrics panel for domain-specific insights
- Use Grafana 10.x JSON format for wide compatibility

**Panels**:
1. Request Rate (by service, endpoint)
2. Error Rate (by service, status code)
3. Latency Distribution (histogram heatmap)
4. Business Metrics (routes calculated, systems queried)

**Alternatives considered**:
- Separate dashboard per service: Duplicates configuration
- Pre-built community dashboards: Don't match our custom metrics
- Grafana dashboard-as-code (Jsonnet): Complexity overkill for 4 panels

---

## Phase 1: Design Artifacts

The following artifacts will be generated:

1. **data-model.md**: Metric definitions, log schema, health check schema
2. **contracts/metrics.md**: Prometheus metrics specification
3. **quickstart.md**: Developer guide for local observability testing

See separate files in this directory.
