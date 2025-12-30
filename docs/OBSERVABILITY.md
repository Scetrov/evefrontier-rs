# Observability Guide

This guide covers the observability features of EVE Frontier microservices, including metrics,
structured logging, and health checks.

## Table of Contents

- [Metrics](#metrics)
- [Logging](#logging)
- [Health Checks](#health-checks)
- [Grafana Dashboard](#grafana-dashboard)
- [Alerting](#alerting)
- [Troubleshooting](#troubleshooting)

---

## Metrics

All microservices expose Prometheus metrics on the `/metrics` endpoint.

### Endpoint

```bash
curl http://localhost:8080/metrics
```

### Available Metrics

#### HTTP Metrics (RED Method)

| Metric | Type | Description |
|--------|------|-------------|
| `http_requests_total` | Counter | Total HTTP requests by method, path, status |
| `http_request_duration_seconds` | Histogram | Request duration in seconds |

**Labels**:
- `method`: HTTP method (`GET`, `POST`)
- `path`: Normalized request path (e.g., `/api/v1/route`)
- `status`: Status code bucket (`2xx`, `4xx`, `5xx`)

#### Business Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `evefrontier_routes_calculated_total` | Counter | Routes successfully calculated |
| `evefrontier_routes_failed_total` | Counter | Route calculations that failed |
| `evefrontier_route_hops` | Histogram | Number of hops in routes |
| `evefrontier_systems_queried_total` | Counter | Systems queried via scout endpoints |
| `evefrontier_neighbors_returned` | Histogram | Neighbors returned by scout queries |

### Example PromQL Queries

```promql
# Request rate per second
rate(http_requests_total[5m])

# P95 latency in milliseconds
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) * 1000

# Success rate percentage
sum(rate(http_requests_total{status="2xx"}[5m])) 
/ sum(rate(http_requests_total[5m])) * 100

# Routes per minute by algorithm
sum(rate(evefrontier_routes_calculated_total[1m])) by (algorithm) * 60

# Average route hops
rate(evefrontier_route_hops_sum[5m]) / rate(evefrontier_route_hops_count[5m])
```

### Prometheus Scrape Configuration

Add to your `prometheus.yml`:

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

---

## Logging

Services emit structured JSON logs by default, suitable for log aggregation systems like ELK,
Loki, or CloudWatch.

### Configuration

| Environment Variable | Default | Description |
|---------------------|---------|-------------|
| `LOG_FORMAT` | `json` | Output format: `json` or `text` |
| `RUST_LOG` | `info` | Log level filter |
| `SERVICE_NAME` | (from binary) | Service name in logs |

### Log Formats

#### JSON Format (Production)

```json
{
  "timestamp": "2025-12-30T10:15:30.123456Z",
  "level": "INFO",
  "target": "evefrontier_service_route",
  "service": "route",
  "message": "Request completed",
  "request_id": "01941234-5678-7abc-def0-123456789abc",
  "path": "/api/v1/route",
  "method": "POST",
  "status": 200,
  "latency_ms": 42.5
}
```

#### Text Format (Development)

```
2025-12-30T10:15:30.123456Z  INFO evefrontier_service_route: Request completed
  request_id: 01941234-5678-7abc-def0-123456789abc
  path: /api/v1/route
  method: POST
  status: 200
  latency_ms: 42.5
```

### Switching Log Format

```bash
# JSON (default, production)
LOG_FORMAT=json ./evefrontier-service-route

# Text (development)
LOG_FORMAT=text ./evefrontier-service-route
```

### Log Level Examples

```bash
# Default (info)
RUST_LOG=info ./evefrontier-service-route

# Debug (verbose)
RUST_LOG=debug ./evefrontier-service-route

# Target-specific levels
RUST_LOG=evefrontier_service_route=debug,tower_http=warn ./evefrontier-service-route
```

### Request Correlation

All request logs include a `request_id` field (UUID v7) for correlation. This ID is:
- Generated for each incoming request
- Propagated through the request lifecycle
- Included in all log entries for that request
- Returned in the `X-Request-ID` response header

---

## Health Checks

Services expose Kubernetes-compatible health check endpoints.

### Endpoints

| Endpoint | Purpose | Checks |
|----------|---------|--------|
| `/health/live` | Liveness probe | Service is running |
| `/health/ready` | Readiness probe | Dependencies are available |

### Liveness Probe

```bash
curl http://localhost:8080/health/live
```

Response (200 OK):
```json
{
  "status": "ok",
  "service": "evefrontier-service-route",
  "version": "0.1.0"
}
```

### Readiness Probe

```bash
curl http://localhost:8080/health/ready
```

Response (200 OK):
```json
{
  "status": "ok",
  "service": "evefrontier-service-route",
  "version": "0.1.0",
  "systems_loaded": 5431,
  "spatial_index_ready": true,
  "checks": {
    "database": {
      "status": "ok",
      "systems_count": 5431
    },
    "spatial_index": {
      "status": "ok"
    }
  }
}
```

Response (503 Service Unavailable):
```json
{
  "status": "not_ready",
  "service": "evefrontier-service-route",
  "version": "0.1.0",
  "systems_loaded": 0,
  "spatial_index_ready": false,
  "checks": {
    "database": {
      "status": "error",
      "message": "no systems loaded"
    },
    "spatial_index": {
      "status": "degraded",
      "message": "spatial index not loaded"
    }
  }
}
```

### Check Status Values

| Status | HTTP Code | Description |
|--------|-----------|-------------|
| `ok` | 200 | All checks passed |
| `degraded` | 200 | Some non-critical checks failed |
| `not_ready` | 503 | Critical checks failed |

### Kubernetes Configuration

```yaml
spec:
  containers:
    - name: route
      livenessProbe:
        httpGet:
          path: /health/live
          port: 8080
        initialDelaySeconds: 5
        periodSeconds: 10
      readinessProbe:
        httpGet:
          path: /health/ready
          port: 8080
        initialDelaySeconds: 10
        periodSeconds: 5
```

---

## Grafana Dashboard

A pre-built Grafana dashboard is available at `docs/dashboards/evefrontier-overview.json`.

### Installation

1. Open Grafana → Dashboards → Import
2. Upload `evefrontier-overview.json`
3. Select your Prometheus data source
4. Click Import

### Dashboard Sections

- **Service Health Overview**: Service status, success rate, RPS, P95 latency
- **Request Metrics**: Request rate and latency by endpoint
- **Business Metrics**: Routes calculated, failures, hops distribution
- **Resource Usage**: Memory and CPU by service

---

## Alerting

Prometheus alerting rules are available at `docs/dashboards/alerting-rules.yaml`.

### Critical Alerts

| Alert | Condition | Response |
|-------|-----------|----------|
| `HighErrorRate` | >5% 5xx errors for 5m | Immediate investigation |
| `ServiceDown` | Service unreachable for 1m | Check pods, restart |
| `AllInstancesDown` | All replicas down | Emergency escalation |

### Warning Alerts

| Alert | Condition | Response |
|-------|-----------|----------|
| `HighLatency` | P95 > 1s for 10m | Performance investigation |
| `HighRouteFailureRate` | >10% failures for 15m | Check data, algorithms |
| `Elevated4xxRate` | >20% 4xx for 15m | Review client errors |

### Installation

```bash
# Add to Prometheus
cp docs/dashboards/alerting-rules.yaml /etc/prometheus/rules/
curl -X POST http://localhost:9090/-/reload
```

---

## Troubleshooting

### Metrics Not Appearing

1. **Check endpoint is accessible**:
   ```bash
   curl -v http://localhost:8080/metrics
   ```

2. **Verify Prometheus scrape config**:
   ```bash
   curl http://prometheus:9090/api/v1/targets
   ```

3. **Check pod annotations**:
   ```bash
   kubectl describe pod <pod-name> | grep prometheus
   ```

### Logs Not Structured

1. **Verify LOG_FORMAT**:
   ```bash
   kubectl exec <pod> -- env | grep LOG_FORMAT
   ```

2. **Check for initialization errors**:
   ```bash
   kubectl logs <pod> | head -20
   ```

### Health Check Failures

1. **Check readiness details**:
   ```bash
   curl http://localhost:8080/health/ready | jq
   ```

2. **Verify data is loaded**:
   ```bash
   curl http://localhost:8080/health/ready | jq '.checks'
   ```

3. **Check pod events**:
   ```bash
   kubectl describe pod <pod-name>
   ```

### Common Issues

| Symptom | Likely Cause | Solution |
|---------|--------------|----------|
| `no systems loaded` | Database not found | Check DATA_DIR or bundled database |
| `spatial index not loaded` | Index file missing | Rebuild with `index-build` command |
| High P95 latency | Large routes | Enable spatial index, adjust timeouts |
| Memory growth | Route caching | Monitor route complexity |

### Useful Commands

```bash
# View service logs with timestamps
kubectl logs -f <pod> --timestamps

# Get Prometheus metrics
curl -s http://localhost:8080/metrics | grep http_requests_total

# Check health with jq
curl -s http://localhost:8080/health/ready | jq '.checks'

# Monitor request rate
watch -n 1 'curl -s http://localhost:8080/metrics | grep http_requests_total'
```

---

## Related Documentation

- [Deployment Guide](DEPLOYMENT.md)
- [Architecture Overview](ARCHITECTURE.md)
- [Metrics Contract](../specs/008-microservices-observability/contracts/metrics.md)
- [Dashboard README](dashboards/README.md)
