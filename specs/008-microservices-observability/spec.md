# Feature Specification: Microservices Observability

**Feature Branch**: `008-microservices-observability`  
**Created**: 2025-12-30  
**Status**: Draft  
**Input**: docs/TODO.md "Docker Microservices & Kubernetes Deployment" section - Observability setup

## Problem Statement

The EVE Frontier microservices (route, scout-gates, scout-range) currently have basic tracing
infrastructure via the `tracing` crate but lack comprehensive observability features:

1. **Metrics**: No Prometheus-compatible metrics endpoints for monitoring request rates, latencies,
   error rates, or business metrics (routes calculated, systems queried).
2. **Structured Logging**: Tracing spans exist but log output format and structured fields need
   documentation and standardization.
3. **Health Checks**: Basic health endpoints exist (`/health/live`, `/health/ready`) but need
   enhancement for dependency checks and detailed status reporting.
4. **Distributed Tracing**: No OpenTelemetry integration for request correlation across services.
5. **Documentation**: No operational runbook for monitoring, alerting, and troubleshooting.

## Goals

1. Add Prometheus metrics endpoint (`/metrics`) to each microservice with standard HTTP metrics
2. Expose custom business metrics (routes_calculated_total, systems_queried_total, etc.)
3. Standardize structured JSON logging with correlation IDs
4. Document the observability stack integration (Prometheus, Grafana, Loki/ELK)
5. Provide example Grafana dashboards and alerting rules
6. Create operational documentation for monitoring and troubleshooting

## Non-Goals

1. Deploying a full observability stack (Prometheus/Grafana) - users bring their own
2. Log aggregation infrastructure (Loki, ELK) - user responsibility
3. APM/tracing backend (Jaeger, Tempo) - user responsibility
4. Real-time alerting configuration - provide examples only
5. Auto-instrumentation (focus on explicit, documented metrics)

## User Scenarios & Testing

### User Story 1 - Prometheus Metrics Scraping (Priority: P1)

A platform operator wants to scrape metrics from EVE Frontier microservices to monitor health,
performance, and usage patterns in their existing Prometheus infrastructure.

**Why this priority**: Metrics are the foundation of observability and enable all other monitoring
use cases. Without metrics, operators cannot measure SLIs/SLOs.

**Independent Test**: Start a service, make requests, and verify `/metrics` endpoint returns valid
Prometheus format with expected metric families.

**Acceptance Scenarios**:

1. **Given** a microservice is running, **When** an operator requests `GET /metrics`, **Then** the
   response is `text/plain; version=0.0.4; charset=utf-8` with Prometheus exposition format.
2. **Given** a microservice is running, **When** requests are made to the API, **Then** the
   `http_requests_total` counter increments with labels for method, path, and status.
3. **Given** a microservice is running, **When** requests are made to the API, **Then** the
   `http_request_duration_seconds` histogram captures latency with method and path labels.
4. **Given** a route is calculated successfully, **When** an operator checks metrics, **Then**
   `evefrontier_routes_calculated_total` increments with algorithm label (bfs, dijkstra, astar).
5. **Given** metrics are being collected, **When** an operator configures Prometheus scrape, **Then**
   all standard service_* and process_* metrics are available.

---

### User Story 2 - Structured JSON Logging (Priority: P2)

A developer or SRE wants to search and analyze service logs in a log aggregation system (Loki, ELK,
CloudWatch Logs) using structured queries.

**Why this priority**: Structured logging enables efficient debugging and incident response, but
metrics (US1) provide the "what's wrong" signal first.

**Independent Test**: Start a service, make requests, and verify stdout contains valid JSON logs
with required fields.

**Acceptance Scenarios**:

1. **Given** a microservice is running with default config, **When** it writes logs to stdout,
   **Then** each log line is valid JSON with `timestamp`, `level`, `message`, and `target` fields.
2. **Given** a request is received, **When** it is logged, **Then** the log includes `request_id`
   (correlation ID), `method`, `path`, and `remote_addr` fields.
3. **Given** an error occurs, **When** it is logged, **Then** the log includes `error` field with
   the error message and `error_type` field with the error variant.
4. **Given** tracing is enabled, **When** a span is created, **Then** the log includes `span_id`
   and `trace_id` fields for distributed tracing correlation.
5. **Given** environment variable `LOG_FORMAT=text` is set, **When** the service starts, **Then**
   logs are in human-readable text format (for local development).

---

### User Story 3 - Grafana Dashboard Integration (Priority: P3)

A platform operator wants to import pre-built Grafana dashboards to visualize EVE Frontier service
health and performance without building dashboards from scratch.

**Why this priority**: Dashboards provide immediate value for visualization but require metrics
(US1) to be implemented first.

**Independent Test**: Import the provided dashboard JSON into Grafana and verify all panels
render with data from a running service.

**Acceptance Scenarios**:

1. **Given** Grafana is running with Prometheus data source, **When** an operator imports the
   provided dashboard JSON, **Then** all panels load without errors.
2. **Given** the dashboard is imported, **When** requests are made to services, **Then** the
   request rate panel shows live data.
3. **Given** the dashboard is imported, **When** errors occur, **Then** the error rate panel
   shows the increase in error count.
4. **Given** the dashboard is imported, **When** routes are calculated, **Then** the business
   metrics panel shows routes by algorithm breakdown.

---

### User Story 4 - Health Check Enhancement (Priority: P3)

A platform operator wants detailed health check endpoints that report not just liveness but also
readiness with dependency status for proper load balancer integration.

**Why this priority**: Enhanced health checks improve deployment reliability but basic health
exists; this is an enhancement.

**Independent Test**: Query health endpoints and verify response format and content.

**Acceptance Scenarios**:

1. **Given** a microservice is healthy, **When** `/health/live` is requested, **Then** the response
   is `200 OK` with `{"status": "healthy"}`.
2. **Given** a microservice is healthy, **When** `/health/ready` is requested, **Then** the response
   is `200 OK` with `{"status": "ready", "checks": {"database": "ok", "spatial_index": "ok"}}`.
3. **Given** the spatial index is not loaded, **When** `/health/ready` is requested, **Then** the
   response is `503 Service Unavailable` with `{"status": "not_ready", "checks": {"spatial_index": "missing"}}`.

---

### Edge Cases

- What happens when metrics endpoint is called before any requests? (Empty histograms OK)
- How does system handle extremely high cardinality labels? (Limit path cardinality)
- What happens when log output is too verbose? (Support log level configuration)
- How does system handle correlation ID propagation from upstream? (Accept X-Request-ID header)

## Requirements

### Functional Requirements

- **FR-001**: Each microservice MUST expose a `/metrics` endpoint returning Prometheus format
- **FR-002**: Metrics MUST include standard HTTP metrics (requests_total, duration_seconds, size_bytes)
- **FR-003**: Metrics MUST include custom business metrics (routes_calculated, systems_queried)
- **FR-004**: Logs MUST be emitted in structured JSON format by default
- **FR-005**: Logs MUST include correlation ID (request_id) for request tracing
- **FR-006**: Log format MUST be configurable via environment variable (json/text)
- **FR-007**: Health endpoints MUST return JSON with status and optional check details
- **FR-008**: Metrics endpoint MUST NOT require authentication (internal network only)
- **FR-009**: Documentation MUST include Grafana dashboard JSON for import
- **FR-010**: Documentation MUST include example alerting rules for common failure modes

### Non-Functional Requirements

- **NFR-001**: Metrics collection overhead MUST be <1ms per request (p99)
- **NFR-002**: Metrics endpoint response time MUST be <100ms
- **NFR-003**: Log serialization MUST NOT significantly impact request latency
- **NFR-004**: Metric cardinality MUST be bounded (no unbounded path labels)

### Key Entities

- **Metric**: A named measurement with labels (counter, gauge, histogram, summary)
- **Log Entry**: Structured record with timestamp, level, message, and context fields
- **Health Check**: Named status check with result (ok, degraded, error) and optional details
- **Correlation ID**: Unique identifier for request tracing across service boundaries

## Success Criteria

### Measurable Outcomes

- **SC-001**: All three microservices expose `/metrics` endpoint with >10 metric families
- **SC-002**: Metrics include both RED (Rate, Errors, Duration) and custom business metrics
- **SC-003**: 100% of request logs include correlation ID field
- **SC-004**: Provided Grafana dashboard imports successfully with 0 panel errors
- **SC-005**: Documentation includes operational runbook with troubleshooting steps
- **SC-006**: Metrics overhead <1ms p99 as measured by benchmark

## Technical Notes (for Plan phase)

### Existing Infrastructure

- `evefrontier-service-shared` crate contains common HTTP infrastructure (axum-based)
- `tracing` and `tracing-subscriber` already in dependencies
- Basic `/health/live` and `/health/ready` endpoints exist in each service
- Helm chart exists with Prometheus scrape annotations capability

### Recommended Dependencies

- `metrics` crate with `metrics-exporter-prometheus` for Prometheus exposition
- `tracing-opentelemetry` for distributed tracing (optional enhancement)
- `axum` middleware for automatic request metrics

### Configuration

- `METRICS_ENABLED`: Enable/disable metrics endpoint (default: true)
- `LOG_FORMAT`: json|text (default: json)
- `LOG_LEVEL`: trace|debug|info|warn|error (default: info)
- `OTEL_EXPORTER_OTLP_ENDPOINT`: OpenTelemetry collector endpoint (optional)
