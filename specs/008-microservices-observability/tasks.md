# Tasks: Microservices Observability

**Input**: Design documents from `/specs/008-microservices-observability/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/metrics.md ✅

**Tests**: Tests ARE required per TDD constitution (Red-Green-Refactor cycle mandatory).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Rust workspace**: `crates/evefrontier-service-*/src/`
- **Shared library**: `crates/evefrontier-service-shared/src/`
- **Documentation**: `docs/`
- **Tests**: Inline in modules + `crates/evefrontier-service-shared/src/` test modules

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add dependencies and create module structure

- [X] T001 Add `metrics = "0.23"` and `metrics-exporter-prometheus = "0.15"` dependencies to `crates/evefrontier-service-shared/Cargo.toml`
- [X] T002 Add `uuid = { version = "1.11", features = ["v7"] }` dependency to `crates/evefrontier-service-shared/Cargo.toml`
- [X] T003 [P] Create empty module file `crates/evefrontier-service-shared/src/metrics.rs`
- [X] T004 [P] Create empty module file `crates/evefrontier-service-shared/src/logging.rs`
- [X] T005 [P] Create empty module file `crates/evefrontier-service-shared/src/middleware.rs`
- [X] T006 Add module declarations to `crates/evefrontier-service-shared/src/lib.rs` for metrics, logging, middleware

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before user stories

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T007 Define `MetricsConfig` struct in `crates/evefrontier-service-shared/src/metrics.rs` with fields: enabled (bool), path (String)
- [X] T008 Implement `init_metrics()` function in `crates/evefrontier-service-shared/src/metrics.rs` using PrometheusBuilder
- [X] T009 [P] Define `LoggingConfig` struct in `crates/evefrontier-service-shared/src/logging.rs` with fields: format (json/text), level
- [X] T010 Implement `init_logging()` function in `crates/evefrontier-service-shared/src/logging.rs` with format switching
- [X] T011 [P] Define `RequestId` newtype in `crates/evefrontier-service-shared/src/middleware.rs` for correlation ID extraction
- [X] T012 Implement `extract_or_generate_request_id()` function in `crates/evefrontier-service-shared/src/middleware.rs`: extract `X-Request-ID` header if present, else generate UUID v7
- [X] T013 Export public types from `crates/evefrontier-service-shared/src/lib.rs`: MetricsConfig, LoggingConfig, RequestId, init_metrics, init_logging

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Prometheus Metrics Scraping (Priority: P1) 🎯 MVP

**Goal**: Expose `/metrics` endpoint with HTTP RED metrics and business metrics

**Independent Test**: `curl http://localhost:8080/metrics` returns valid Prometheus format with `http_requests_total` counter

### Tests for User Story 1

- [ ] T014 [P] [US1] Write test `test_metrics_endpoint_returns_prometheus_format` in `crates/evefrontier-service-shared/src/metrics.rs` (MUST FAIL first)
- [ ] T015 [P] [US1] Write test `test_http_request_counter_increments` in `crates/evefrontier-service-shared/src/metrics.rs` (MUST FAIL first)
- [ ] T016 [P] [US1] Write test `test_http_request_duration_histogram_records` in `crates/evefrontier-service-shared/src/metrics.rs` (MUST FAIL first)
- [ ] T017 [P] [US1] Write test `test_business_metric_routes_calculated` in `crates/evefrontier-service-shared/src/metrics.rs` (MUST FAIL first)

### Implementation for User Story 1

- [ ] T018 [US1] Implement `MetricsLayer` tower middleware in `crates/evefrontier-service-shared/src/middleware.rs` for http_requests_total counter (NOTE: normalize path labels - strip query params, use route template not actual path to bound cardinality per NFR-004)
- [ ] T019 [US1] Add http_request_duration_seconds histogram recording to `MetricsLayer` in `crates/evefrontier-service-shared/src/middleware.rs`
- [ ] T019a [P] [US1] Add http_request_size_bytes histogram recording to `MetricsLayer` for request body size
- [ ] T019b [P] [US1] Add http_response_size_bytes histogram recording to `MetricsLayer` for response body size
- [ ] T020 [US1] Implement `metrics_handler()` axum handler in `crates/evefrontier-service-shared/src/metrics.rs` returning Prometheus format
- [ ] T021 [P] [US1] Define business metric helper functions in `crates/evefrontier-service-shared/src/metrics.rs`: `record_route_calculated()`, `record_route_failed()`
- [ ] T021a [P] [US1] Define `record_route_hops()` helper in `crates/evefrontier-service-shared/src/metrics.rs` for evefrontier_route_hops histogram
- [ ] T022 [P] [US1] Define business metric helper functions in `crates/evefrontier-service-shared/src/metrics.rs`: `record_systems_queried()`, `record_neighbors_returned()`
- [ ] T023 [US1] Wire `/metrics` endpoint into `crates/evefrontier-service-route/src/main.rs` router
- [ ] T024 [US1] Wire `MetricsLayer` middleware into `crates/evefrontier-service-route/src/main.rs` router
- [ ] T025 [US1] Add `record_route_calculated()` call to route handler success path in `crates/evefrontier-service-route/src/main.rs`
- [ ] T025a [US1] Add `record_route_hops()` call to route handler success path in `crates/evefrontier-service-route/src/main.rs`
- [ ] T026 [US1] Add `record_route_failed()` call to route handler error path in `crates/evefrontier-service-route/src/main.rs`
- [ ] T027 [P] [US1] Wire `/metrics` endpoint and `MetricsLayer` into `crates/evefrontier-service-scout-gates/src/main.rs`
- [ ] T028 [P] [US1] Wire `/metrics` endpoint and `MetricsLayer` into `crates/evefrontier-service-scout-range/src/main.rs`
- [ ] T029 [US1] Add `record_systems_queried()` call to scout-gates handler in `crates/evefrontier-service-scout-gates/src/main.rs`
- [ ] T030 [US1] Add `record_systems_queried()` call to scout-range handler in `crates/evefrontier-service-scout-range/src/main.rs`

**Checkpoint**: `curl /metrics` returns valid Prometheus data with HTTP and business metrics

---

## Phase 4: User Story 2 - Structured JSON Logging (Priority: P2)

**Goal**: Emit JSON logs with correlation IDs on every request

**Independent Test**: Service logs JSON to stdout with `request_id`, `method`, `path` fields

### Tests for User Story 2

- [ ] T031 [P] [US2] Write test `test_json_log_format_default` in `crates/evefrontier-service-shared/src/logging.rs` (MUST FAIL first)
- [ ] T032 [P] [US2] Write test `test_text_log_format_with_env` in `crates/evefrontier-service-shared/src/logging.rs` (MUST FAIL first)
- [ ] T033 [P] [US2] Write test `test_request_log_includes_correlation_id` in `crates/evefrontier-service-shared/src/middleware.rs` (MUST FAIL first)

### Implementation for User Story 2

- [ ] T034 [US2] Enhance `init_logging()` in `crates/evefrontier-service-shared/src/logging.rs` to read `LOG_FORMAT` env var and configure json/text layers
- [ ] T035 [US2] Add `service` field injection to JSON logs in `crates/evefrontier-service-shared/src/logging.rs`
- [ ] T036 [US2] Implement request span creation with `request_id` field in `MetricsLayer` `crates/evefrontier-service-shared/src/middleware.rs`
- [ ] T037 [US2] Add `method`, `path`, `remote_addr` fields to request span in `crates/evefrontier-service-shared/src/middleware.rs`
- [ ] T038 [US2] Add response logging with `status`, `latency_ms` fields to `MetricsLayer` on_response callback
- [ ] T039 [US2] Replace `init_tracing()` with shared `init_logging()` in `crates/evefrontier-service-route/src/main.rs`
- [ ] T040 [P] [US2] Replace `init_tracing()` with shared `init_logging()` in `crates/evefrontier-service-scout-gates/src/main.rs`
- [ ] T041 [P] [US2] Replace `init_tracing()` with shared `init_logging()` in `crates/evefrontier-service-scout-range/src/main.rs`

**Checkpoint**: Logs are valid JSON with `request_id`, `method`, `path`, `status`, `latency_ms` fields

---

## Phase 5: User Story 3 - Grafana Dashboard Integration (Priority: P3)

**Goal**: Provide ready-to-import Grafana dashboard JSON

**Independent Test**: Import dashboard into Grafana, all panels render without errors

### Implementation for User Story 3 (No tests needed - static JSON file)

- [ ] T042 [P] [US3] Create directory `docs/dashboards/`
- [ ] T043 [US3] Create Grafana dashboard JSON `docs/dashboards/evefrontier.json` with Overview row (request rate, error rate, p95 latency stats)
- [ ] T044 [US3] Add Traffic row to `docs/dashboards/evefrontier.json` with request rate by service and endpoint time series
- [ ] T045 [US3] Add Latency row to `docs/dashboards/evefrontier.json` with p50/p95/p99 time series and latency heatmap
- [ ] T046 [US3] Add Business Metrics row to `docs/dashboards/evefrontier.json` with routes by algorithm and systems queried panels
- [ ] T047 [US3] Add service variable to dashboard for filtering by service name
- [ ] T048 [P] [US3] Create example Prometheus alerting rules in `docs/dashboards/alerting-rules.yaml`

**Checkpoint**: Dashboard imports into Grafana with 0 panel errors

---

## Phase 6: User Story 4 - Health Check Enhancement (Priority: P3)

**Goal**: Return detailed dependency status in /health/ready response

**Independent Test**: `/health/ready` returns `{"status": "ready", "checks": {"database": "ok", "spatial_index": "ok"}}`

### Tests for User Story 4

- [ ] T049 [P] [US4] Write test `test_health_ready_returns_checks_map` in `crates/evefrontier-service-shared/src/health.rs` (MUST FAIL first)
- [ ] T050 [P] [US4] Write test `test_health_ready_503_on_failure` in `crates/evefrontier-service-shared/src/health.rs` (MUST FAIL first)

### Implementation for User Story 4

- [ ] T051 [US4] Add `CheckResult` struct to `crates/evefrontier-service-shared/src/health.rs` per data-model.md
- [ ] T052 [US4] Add `checks: Option<HashMap<String, CheckResult>>` field to `HealthStatus` in `crates/evefrontier-service-shared/src/health.rs`
- [ ] T053 [US4] Update `health_ready()` handler to populate checks map with database status in `crates/evefrontier-service-shared/src/health.rs`
- [ ] T054 [US4] Update `health_ready()` handler to populate checks map with spatial_index status in `crates/evefrontier-service-shared/src/health.rs`
- [ ] T055 [US4] Return HTTP 503 when any check fails in `health_ready()` handler

**Checkpoint**: `/health/ready` returns detailed checks and 503 on failure

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, Helm chart updates, and validation

- [ ] T056 [P] Create `docs/OBSERVABILITY.md` with overview of metrics, logging, and health check features
- [ ] T057 Add metrics endpoint documentation to `docs/OBSERVABILITY.md` with PromQL examples
- [ ] T058 Add logging configuration documentation to `docs/OBSERVABILITY.md` with LOG_FORMAT examples
- [ ] T059 Add troubleshooting section to `docs/OBSERVABILITY.md` per quickstart.md patterns
- [ ] T060 [P] Update `charts/evefrontier/values.yaml` with Prometheus scrape annotations: `prometheus.io/scrape: "true"`, `prometheus.io/port: "8080"`, `prometheus.io/path: "/metrics"`
- [ ] T061 [P] Update `charts/evefrontier/templates/deployment-route.yaml` to include Prometheus annotations from values
- [ ] T062 [P] Update `charts/evefrontier/templates/deployment-scout-gates.yaml` to include Prometheus annotations
- [ ] T063 [P] Update `charts/evefrontier/templates/deployment-scout-range.yaml` to include Prometheus annotations
- [ ] T064 Add observability section to `charts/evefrontier/README.md` with Prometheus integration instructions
- [ ] T065 Run `cargo test --workspace` to verify all tests pass
- [ ] T066 Run quickstart.md validation: start services, verify `/metrics`, verify JSON logs, verify `/health/ready`
- [ ] T067 Update `docs/TODO.md` to mark "Observability setup" checkbox as complete

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - can start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 - BLOCKS all user stories
- **Phase 3 (US1 Metrics)**: Depends on Phase 2
- **Phase 4 (US2 Logging)**: Depends on Phase 2, can run in parallel with Phase 3
- **Phase 5 (US3 Dashboard)**: Depends on Phase 3 (needs metrics spec finalized)
- **Phase 6 (US4 Health)**: Depends on Phase 2, can run in parallel with Phases 3-5
- **Phase 7 (Polish)**: Depends on Phases 3-6 completion

### User Story Independence

- **US1 (Metrics)**: Foundation only - can be deployed as MVP
- **US2 (Logging)**: Foundation only - can be deployed independently
- **US3 (Dashboard)**: Requires US1 metrics to be meaningful
- **US4 (Health)**: Foundation only - can be deployed independently

### Parallel Opportunities

```text
After Phase 2 completes, these can run in parallel:
┌─────────────────────────────────────────────────────────────┐
│  Phase 3 (US1)    │  Phase 4 (US2)   │  Phase 6 (US4)      │
│  Metrics          │  Logging         │  Health Checks      │
│  (Developer A)    │  (Developer B)   │  (Developer C)      │
└─────────────────────────────────────────────────────────────┘
                           ↓
                    Phase 5 (US3)
                    Dashboard
                    (after US1 done)
                           ↓
                    Phase 7 (Polish)
                    All documentation
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T006)
2. Complete Phase 2: Foundational (T007-T013)
3. Complete Phase 3: User Story 1 Metrics (T014-T030)
4. **STOP and VALIDATE**: `curl /metrics` returns Prometheus data
5. Deploy/demo if ready - basic observability available

### Full Implementation

1. Setup + Foundational → Foundation ready
2. US1 Metrics → Test independently → Prometheus scraping works
3. US2 Logging → Test independently → JSON logs with correlation IDs
4. US3 Dashboard → Test with Grafana import → Visualization ready
5. US4 Health → Test independently → Enhanced health checks
6. Polish → Documentation and Helm updates → Production ready

---

## Summary

| Metric | Count |
|--------|-------|
| **Total Tasks** | 72 |
| **Phase 1 (Setup)** | 6 |
| **Phase 2 (Foundational)** | 7 |
| **Phase 3 (US1 Metrics)** | 22 |
| **Phase 4 (US2 Logging)** | 11 |
| **Phase 5 (US3 Dashboard)** | 7 |
| **Phase 6 (US4 Health)** | 7 |
| **Phase 7 (Polish)** | 12 |
| **Parallel Opportunities** | 28 tasks marked [P] |

## Remediation Log

The following issues from specification analysis were addressed:

| Issue ID | Severity | Resolution |
|----------|----------|------------|
| C1 | MEDIUM | Added T019a, T019b for http_request_size_bytes and http_response_size_bytes histograms |
| U2 | MEDIUM | Clarified T012 to extract X-Request-ID header if present, else generate UUID v7 |
| U1 | MEDIUM | Added path normalization note to T018 for NFR-004 bounded cardinality |
| C3 | LOW | Added T021a for record_route_hops() and T025a for calling it in route handler |

**MVP Scope**: Phases 1-3 (35 tasks) delivers Prometheus metrics endpoint with HTTP and business metrics.

**Format Validation**: ✅ All tasks follow `- [ ] [ID] [P?] [Story?] Description with file path` format.
