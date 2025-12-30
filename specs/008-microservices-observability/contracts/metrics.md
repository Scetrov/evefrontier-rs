# Prometheus Metrics Contract

**Feature**: 008-microservices-observability  
**Date**: 2025-12-30  
**Version**: 1.0.0

## Overview

This document specifies the Prometheus metrics exposed by EVE Frontier microservices. Metrics
follow the [Prometheus naming conventions](https://prometheus.io/docs/practices/naming/) and
[OpenMetrics specification](https://openmetrics.io/).

---

## Endpoint Specification

### GET /metrics

Returns metrics in Prometheus exposition format.

**Request**

```http
GET /metrics HTTP/1.1
Host: service:8080
Accept: text/plain
```

**Response**

```http
HTTP/1.1 200 OK
Content-Type: text/plain; version=0.0.4; charset=utf-8

# HELP http_requests_total Total number of HTTP requests
# TYPE http_requests_total counter
http_requests_total{method="POST",path="/api/v1/route",status="2xx"} 1234
...
```

**Status Codes**

| Code | Description |
|------|-------------|
| 200 | Metrics returned successfully |
| 503 | Metrics temporarily unavailable |

---

## Metric Families

### HTTP Metrics

#### http_requests_total

**Type**: Counter  
**Unit**: requests  
**Help**: Total number of HTTP requests received

**Labels**:
- `method` (required): HTTP method (`GET`, `POST`, etc.)
- `path` (required): Normalized request path
- `status` (required): Status code bucket (`2xx`, `4xx`, `5xx`)

**Example**:
```prometheus
# HELP http_requests_total Total number of HTTP requests received
# TYPE http_requests_total counter
http_requests_total{method="POST",path="/api/v1/route",status="2xx"} 1523
http_requests_total{method="POST",path="/api/v1/route",status="4xx"} 42
http_requests_total{method="GET",path="/health/ready",status="2xx"} 8901
```

---

#### http_request_duration_seconds

**Type**: Histogram  
**Unit**: seconds  
**Help**: Duration of HTTP requests in seconds

**Labels**:
- `method` (required): HTTP method
- `path` (required): Normalized request path

**Buckets**: `[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]`

**Example**:
```prometheus
# HELP http_request_duration_seconds Duration of HTTP requests in seconds
# TYPE http_request_duration_seconds histogram
http_request_duration_seconds_bucket{method="POST",path="/api/v1/route",le="0.005"} 120
http_request_duration_seconds_bucket{method="POST",path="/api/v1/route",le="0.01"} 450
http_request_duration_seconds_bucket{method="POST",path="/api/v1/route",le="0.025"} 890
http_request_duration_seconds_bucket{method="POST",path="/api/v1/route",le="0.05"} 1100
http_request_duration_seconds_bucket{method="POST",path="/api/v1/route",le="0.1"} 1400
http_request_duration_seconds_bucket{method="POST",path="/api/v1/route",le="0.25"} 1500
http_request_duration_seconds_bucket{method="POST",path="/api/v1/route",le="0.5"} 1520
http_request_duration_seconds_bucket{method="POST",path="/api/v1/route",le="1.0"} 1523
http_request_duration_seconds_bucket{method="POST",path="/api/v1/route",le="2.5"} 1523
http_request_duration_seconds_bucket{method="POST",path="/api/v1/route",le="5.0"} 1523
http_request_duration_seconds_bucket{method="POST",path="/api/v1/route",le="10.0"} 1523
http_request_duration_seconds_bucket{method="POST",path="/api/v1/route",le="+Inf"} 1523
http_request_duration_seconds_sum{method="POST",path="/api/v1/route"} 32.456
http_request_duration_seconds_count{method="POST",path="/api/v1/route"} 1523
```

---

#### http_request_size_bytes

**Type**: Histogram  
**Unit**: bytes  
**Help**: Size of HTTP request bodies in bytes

**Labels**:
- `method` (required): HTTP method
- `path` (required): Normalized request path

**Buckets**: `[100, 500, 1000, 5000, 10000, 50000]`

---

#### http_response_size_bytes

**Type**: Histogram  
**Unit**: bytes  
**Help**: Size of HTTP response bodies in bytes

**Labels**:
- `method` (required): HTTP method
- `path` (required): Normalized request path

**Buckets**: `[100, 500, 1000, 5000, 10000, 50000, 100000]`

---

### Business Metrics

#### evefrontier_routes_calculated_total

**Type**: Counter  
**Unit**: routes  
**Help**: Total number of routes successfully calculated

**Labels**:
- `algorithm` (required): Algorithm used (`bfs`, `dijkstra`, `astar`)
- `service` (required): Service name (`route`)

**Example**:
```prometheus
# HELP evefrontier_routes_calculated_total Total number of routes successfully calculated
# TYPE evefrontier_routes_calculated_total counter
evefrontier_routes_calculated_total{algorithm="bfs",service="route"} 892
evefrontier_routes_calculated_total{algorithm="dijkstra",service="route"} 423
evefrontier_routes_calculated_total{algorithm="astar",service="route"} 208
```

---

#### evefrontier_routes_failed_total

**Type**: Counter  
**Unit**: routes  
**Help**: Total number of failed route calculations

**Labels**:
- `reason` (required): Failure reason (`no_path`, `unknown_system`, `validation_error`)
- `service` (required): Service name (`route`)

**Example**:
```prometheus
# HELP evefrontier_routes_failed_total Total number of failed route calculations
# TYPE evefrontier_routes_failed_total counter
evefrontier_routes_failed_total{reason="unknown_system",service="route"} 15
evefrontier_routes_failed_total{reason="no_path",service="route"} 3
evefrontier_routes_failed_total{reason="validation_error",service="route"} 24
```

---

#### evefrontier_route_hops

**Type**: Histogram  
**Unit**: hops  
**Help**: Number of hops in calculated routes

**Labels**:
- `algorithm` (required): Algorithm used

**Buckets**: `[1, 2, 5, 10, 20, 50, 100, 200]`

---

#### evefrontier_systems_queried_total

**Type**: Counter  
**Unit**: queries  
**Help**: Total number of system queries via scout endpoints

**Labels**:
- `query_type` (required): Query type (`gates`, `range`)
- `service` (required): Service name (`scout-gates`, `scout-range`)

---

#### evefrontier_neighbors_returned

**Type**: Histogram  
**Unit**: systems  
**Help**: Number of neighbor systems returned by scout queries

**Labels**:
- `query_type` (required): Query type (`gates`, `range`)

**Buckets**: `[0, 1, 5, 10, 25, 50, 100, 250]`

---

### Runtime Metrics

The following process metrics are automatically provided by `metrics-exporter-prometheus`:

| Metric | Type | Description |
|--------|------|-------------|
| `process_cpu_seconds_total` | Counter | Total CPU time spent |
| `process_resident_memory_bytes` | Gauge | Resident memory size |
| `process_virtual_memory_bytes` | Gauge | Virtual memory size |
| `process_open_fds` | Gauge | Number of open file descriptors |
| `process_start_time_seconds` | Gauge | Process start time (Unix epoch) |

---

## PromQL Examples

### Request Rate

```promql
# Requests per second by service
rate(http_requests_total[5m])

# Success rate percentage
sum(rate(http_requests_total{status="2xx"}[5m])) 
/ sum(rate(http_requests_total[5m])) * 100
```

### Latency

```promql
# P50 latency
histogram_quantile(0.50, rate(http_request_duration_seconds_bucket[5m]))

# P95 latency
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

# P99 latency
histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))
```

### Error Rate

```promql
# Error rate per service
sum(rate(http_requests_total{status=~"4xx|5xx"}[5m])) by (path)
/ sum(rate(http_requests_total[5m])) by (path) * 100
```

### Business Metrics

```promql
# Routes per minute by algorithm
sum(rate(evefrontier_routes_calculated_total[1m])) by (algorithm) * 60

# Average route length
rate(evefrontier_route_hops_sum[5m]) / rate(evefrontier_route_hops_count[5m])

# Route failure rate
sum(rate(evefrontier_routes_failed_total[5m])) 
/ (sum(rate(evefrontier_routes_calculated_total[5m])) 
   + sum(rate(evefrontier_routes_failed_total[5m]))) * 100
```

---

## Alerting Rules

### Critical Alerts

```yaml
groups:
  - name: evefrontier-critical
    rules:
      - alert: HighErrorRate
        expr: |
          sum(rate(http_requests_total{status=~"5xx"}[5m])) 
          / sum(rate(http_requests_total[5m])) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value | printf \"%.2f\" }}%"

      - alert: ServiceDown
        expr: up == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Service is down"
```

### Warning Alerts

```yaml
      - alert: HighLatency
        expr: |
          histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 1
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High latency detected"
          description: "P95 latency is {{ $value | printf \"%.2f\" }}s"

      - alert: HighRouteFailureRate
        expr: |
          sum(rate(evefrontier_routes_failed_total[5m])) 
          / (sum(rate(evefrontier_routes_calculated_total[5m])) 
             + sum(rate(evefrontier_routes_failed_total[5m]))) > 0.1
        for: 15m
        labels:
          severity: warning
        annotations:
          summary: "High route failure rate"
```

---

## Scrape Configuration

### Prometheus scrape_configs

```yaml
scrape_configs:
  - job_name: 'evefrontier'
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
        action: replace
        target_label: __metrics_path__
        regex: (.+)
      - source_labels: [__address__, __meta_kubernetes_pod_annotation_prometheus_io_port]
        action: replace
        regex: ([^:]+)(?::\d+)?;(\d+)
        replacement: $1:$2
        target_label: __address__
```

### Pod Annotations

```yaml
metadata:
  annotations:
    prometheus.io/scrape: "true"
    prometheus.io/path: "/metrics"
    prometheus.io/port: "8080"
```

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-12-30 | Initial specification |
