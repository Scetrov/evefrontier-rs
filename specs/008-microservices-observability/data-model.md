# Data Model: Microservices Observability

**Feature**: 008-microservices-observability  
**Date**: 2025-12-30  
**Status**: Complete

## Overview

This document defines the data models for the observability infrastructure, including metrics
definitions, log entry schema, and health check response formats.

---

## 1. Prometheus Metrics

### 1.1 Standard HTTP Metrics

These metrics follow the RED method (Rate, Errors, Duration) pattern.

#### `http_requests_total` (Counter)

Total number of HTTP requests received.

| Label | Type | Description | Example Values |
|-------|------|-------------|----------------|
| `method` | string | HTTP method | `GET`, `POST` |
| `path` | string | Request path (normalized) | `/api/v1/route`, `/health/ready` |
| `status` | string | HTTP status code bucket | `2xx`, `4xx`, `5xx` |

**Note**: Path is normalized to prevent cardinality explosion (e.g., `/api/v1/route` not
`/api/v1/route?from=Nod`).

#### `http_request_duration_seconds` (Histogram)

Request duration in seconds.

| Label | Type | Description | Example Values |
|-------|------|-------------|----------------|
| `method` | string | HTTP method | `GET`, `POST` |
| `path` | string | Request path (normalized) | `/api/v1/route`, `/scout/gates` |

**Buckets**: `[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]`

#### `http_request_size_bytes` (Histogram)

Request body size in bytes.

| Label | Type | Description | Example Values |
|-------|------|-------------|----------------|
| `method` | string | HTTP method | `POST` |
| `path` | string | Request path | `/api/v1/route` |

**Buckets**: `[100, 500, 1000, 5000, 10000, 50000]`

#### `http_response_size_bytes` (Histogram)

Response body size in bytes.

| Label | Type | Description | Example Values |
|-------|------|-------------|----------------|
| `method` | string | HTTP method | `POST` |
| `path` | string | Request path | `/api/v1/route` |

**Buckets**: `[100, 500, 1000, 5000, 10000, 50000, 100000]`

### 1.2 Business Metrics

Domain-specific metrics for EVE Frontier operations.

#### `evefrontier_routes_calculated_total` (Counter)

Total routes successfully calculated.

| Label | Type | Description | Example Values |
|-------|------|-------------|----------------|
| `algorithm` | string | Routing algorithm used | `bfs`, `dijkstra`, `astar` |
| `service` | string | Service name | `route` |

#### `evefrontier_routes_failed_total` (Counter)

Total route calculations that failed.

| Label | Type | Description | Example Values |
|-------|------|-------------|----------------|
| `reason` | string | Failure reason | `no_path`, `unknown_system`, `validation_error` |
| `service` | string | Service name | `route` |

#### `evefrontier_route_hops` (Histogram)

Number of hops in successful routes.

| Label | Type | Description | Example Values |
|-------|------|-------------|----------------|
| `algorithm` | string | Routing algorithm used | `bfs`, `dijkstra`, `astar` |

**Buckets**: `[1, 2, 5, 10, 20, 50, 100, 200]`

#### `evefrontier_systems_queried_total` (Counter)

Total systems queried via scout endpoints.

| Label | Type | Description | Example Values |
|-------|------|-------------|----------------|
| `query_type` | string | Type of query | `gates`, `range` |
| `service` | string | Service name | `scout-gates`, `scout-range` |

#### `evefrontier_neighbors_returned` (Histogram)

Number of neighbors returned by scout queries.

| Label | Type | Description | Example Values |
|-------|------|-------------|----------------|
| `query_type` | string | Type of query | `gates`, `range` |

**Buckets**: `[0, 1, 5, 10, 25, 50, 100, 250]`

### 1.3 Runtime Metrics

Standard process metrics (automatically provided by metrics-exporter-prometheus).

- `process_cpu_seconds_total` - Total CPU time
- `process_resident_memory_bytes` - Resident memory size
- `process_virtual_memory_bytes` - Virtual memory size
- `process_open_fds` - Open file descriptors
- `process_start_time_seconds` - Process start timestamp

---

## 2. Structured Log Schema

### 2.1 Base Log Entry

All log entries share these fields.

```json
{
  "timestamp": "2025-12-30T10:00:00.123456Z",
  "level": "INFO",
  "target": "evefrontier_service_route::main",
  "message": "Human readable message",
  "service": "route"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `timestamp` | string (ISO 8601) | Yes | Event timestamp with microsecond precision |
| `level` | string | Yes | Log level: `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR` |
| `target` | string | Yes | Rust module path that emitted the log |
| `message` | string | Yes | Human-readable message |
| `service` | string | Yes | Service identifier: `route`, `scout-gates`, `scout-range` |

### 2.2 Request Log Entry

Additional fields for HTTP request logs.

```json
{
  "timestamp": "2025-12-30T10:00:00.123456Z",
  "level": "INFO",
  "target": "evefrontier_service_shared::middleware",
  "message": "handling request",
  "service": "route",
  "request_id": "01935b4c-1234-7000-8000-000000000001",
  "method": "POST",
  "path": "/api/v1/route",
  "remote_addr": "10.0.0.1:54321"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request_id` | string (UUID) | Yes | Correlation ID for request tracing |
| `method` | string | Yes | HTTP method |
| `path` | string | Yes | Request path |
| `remote_addr` | string | No | Client address (may be omitted for privacy) |

### 2.3 Response Log Entry

Additional fields for HTTP response logs.

```json
{
  "timestamp": "2025-12-30T10:00:00.234567Z",
  "level": "INFO",
  "target": "evefrontier_service_shared::middleware",
  "message": "request completed",
  "service": "route",
  "request_id": "01935b4c-1234-7000-8000-000000000001",
  "method": "POST",
  "path": "/api/v1/route",
  "status": 200,
  "latency_ms": 45.2
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `status` | integer | Yes | HTTP status code |
| `latency_ms` | float | Yes | Request duration in milliseconds |

### 2.4 Error Log Entry

Additional fields for error logs.

```json
{
  "timestamp": "2025-12-30T10:00:00.234567Z",
  "level": "ERROR",
  "target": "evefrontier_service_route::main",
  "message": "route planning failed",
  "service": "route",
  "request_id": "01935b4c-1234-7000-8000-000000000001",
  "error": "No path found between systems",
  "error_type": "NoPath"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `error` | string | Yes | Error message |
| `error_type` | string | No | Error variant/type for categorization |

---

## 3. Health Check Schema

### 3.1 Liveness Response

Simple response for `/health/live` endpoint.

```json
{
  "status": "ok",
  "service": "route",
  "version": "0.1.0"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `status` | string | Yes | Always `"ok"` if process is running |
| `service` | string | Yes | Service name |
| `version` | string | Yes | Service version from build |

### 3.2 Readiness Response (Healthy)

Detailed response for `/health/ready` endpoint when all checks pass.

```json
{
  "status": "ready",
  "service": "route",
  "version": "0.1.0",
  "checks": {
    "database": {
      "status": "ok",
      "systems_loaded": 24505
    },
    "spatial_index": {
      "status": "ok"
    }
  }
}
```

### 3.3 Readiness Response (Not Ready)

Response when one or more checks fail (HTTP 503).

```json
{
  "status": "not_ready",
  "service": "route",
  "version": "0.1.0",
  "checks": {
    "database": {
      "status": "ok",
      "systems_loaded": 24505
    },
    "spatial_index": {
      "status": "error",
      "message": "Spatial index not loaded"
    }
  }
}
```

### 3.4 Check Status Values

| Status | HTTP Code | Description |
|--------|-----------|-------------|
| `ok` | 200 | Check passed |
| `degraded` | 200 | Check passed but with warnings |
| `error` | 503 | Check failed, service not ready |

---

## 4. Rust Type Definitions

### 4.1 Metrics Recording (Pseudo-code)

```rust
/// Standard HTTP metric labels
pub struct HttpMetricLabels {
    pub method: &'static str,
    pub path: String,
    pub status: &'static str,
}

/// Business metric for route calculation
pub struct RouteMetric {
    pub algorithm: &'static str,
    pub hops: u32,
    pub success: bool,
    pub failure_reason: Option<&'static str>,
}

/// Business metric for scout queries
pub struct ScoutMetric {
    pub query_type: &'static str,
    pub neighbors_count: u32,
}
```

### 4.2 Health Check Types

```rust
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Individual health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub status: CheckStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Additional details specific to the check
    #[serde(flatten)]
    pub details: HashMap<String, serde_json::Value>,
}

/// Health check status values
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Ok,
    Degraded,
    Error,
}

/// Complete health response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<HashMap<String, CheckResult>>,
}
```

---

## 5. Configuration

### 5.1 Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `METRICS_ENABLED` | `true` | Enable/disable `/metrics` endpoint |
| `METRICS_PORT` | Same as service | Port for metrics (if separate) |
| `LOG_FORMAT` | `json` | Log format: `json` or `text` |
| `LOG_LEVEL` | `info` | Log level: `trace`, `debug`, `info`, `warn`, `error` |
| `SERVICE_NAME` | (from binary) | Override service name in logs/metrics |

### 5.2 Metric Cardinality Limits

To prevent metric cardinality explosion:

- **Path labels**: Normalized, no query parameters
- **Error messages**: Use error type, not full message
- **System names**: Not included in labels (use logs for details)
- **Maximum unique label combinations**: ~100 per metric family

---

## Summary

This data model defines:
- **13 metric families** (4 HTTP, 5 business, 4 runtime)
- **4 log entry types** (base, request, response, error)
- **3 health check schemas** (live, ready-ok, ready-error)
- **6 configuration options**

All schemas are backward-compatible with existing code and can be implemented incrementally.
