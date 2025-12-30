# Quickstart: Microservices Observability

**Feature**: 008-microservices-observability  
**Date**: 2025-12-30

## Overview

This guide helps developers quickly test the observability features locally before deployment.

---

## Prerequisites

- Docker and Docker Compose
- curl or httpie for API testing
- Optional: Prometheus + Grafana for visualization

---

## 1. Start Services with Observability

### Using Docker Compose

```bash
# From repository root
docker compose up -d

# Verify services are running
docker compose ps

# Check logs (JSON format)
docker compose logs -f route
```

### Verify Metrics Endpoint

```bash
# Check metrics from route service
curl http://localhost:8080/metrics

# Expected output (excerpt):
# # HELP http_requests_total Total number of HTTP requests received
# # TYPE http_requests_total counter
# http_requests_total{method="GET",path="/health/ready",status="2xx"} 1
```

---

## 2. Generate Traffic

### Route Service

```bash
# Calculate a route
curl -X POST http://localhost:8080/api/v1/route \
  -H "Content-Type: application/json" \
  -H "X-Request-ID: test-123" \
  -d '{"from": "Nod", "to": "Brana", "algorithm": "bfs"}'

# Generate multiple requests for metrics
for i in {1..10}; do
  curl -s -X POST http://localhost:8080/api/v1/route \
    -H "Content-Type: application/json" \
    -d '{"from": "Nod", "to": "Brana"}' > /dev/null
done
```

### Scout Services

```bash
# Scout gates (via Traefik routing)
curl -X POST http://localhost:8080/api/v1/scout/gates \
  -H "Content-Type: application/json" \
  -d '{"system": "Nod"}'

# Scout range
curl -X POST http://localhost:8080/api/v1/scout/range \
  -H "Content-Type: application/json" \
  -d '{"system": "Nod", "radius": 50.0}'
```

---

## 3. Verify Metrics Collection

After generating traffic, check metrics again:

```bash
curl -s http://localhost:8080/metrics | grep -E "http_requests_total|evefrontier_routes"

# Expected:
# http_requests_total{method="POST",path="/api/v1/route",status="2xx"} 11
# evefrontier_routes_calculated_total{algorithm="bfs",service="route"} 11
```

### Key Metrics to Verify

| Metric | After Traffic | Meaning |
|--------|---------------|---------|
| `http_requests_total` | > 0 | Requests are being counted |
| `http_request_duration_seconds_bucket` | Has values | Latency is being recorded |
| `evefrontier_routes_calculated_total` | > 0 | Business metrics working |

---

## 4. Verify Structured Logging

### Check JSON Log Format

```bash
# View logs with jq for pretty printing
docker compose logs route 2>&1 | head -20 | jq .

# Expected fields:
# {
#   "timestamp": "2025-12-30T10:00:00.123456Z",
#   "level": "INFO",
#   "target": "evefrontier_service_route::main",
#   "message": "handling route request",
#   "service": "route",
#   "request_id": "test-123"
# }
```

### Filter by Request ID

```bash
# Find all logs for a specific request
docker compose logs route 2>&1 | grep "test-123"
```

### Switch to Text Logging (Development)

```bash
# Stop services
docker compose down

# Edit docker-compose.yml to add environment variable:
#   environment:
#     - LOG_FORMAT=text

# Restart
docker compose up -d
docker compose logs route
# Logs now in human-readable format
```

---

## 5. Verify Health Checks

### Liveness Probe

```bash
curl http://localhost:8080/health/live | jq .

# Expected:
# {
#   "status": "ok",
#   "service": "route",
#   "version": "0.1.0"
# }
```

### Readiness Probe

```bash
curl http://localhost:8080/health/ready | jq .

# Expected:
# {
#   "status": "ready",
#   "service": "route",
#   "version": "0.1.0",
#   "checks": {
#     "database": {
#       "status": "ok",
#       "systems_loaded": 24505
#     },
#     "spatial_index": {
#       "status": "ok"
#     }
#   }
# }
```

---

## 6. Local Prometheus Setup (Optional)

### Start Prometheus

Create `prometheus.yml`:

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'evefrontier-route'
    static_configs:
      - targets: ['host.docker.internal:8080']
  - job_name: 'evefrontier-scout-gates'
    static_configs:
      - targets: ['host.docker.internal:8081']
  - job_name: 'evefrontier-scout-range'
    static_configs:
      - targets: ['host.docker.internal:8082']
```

Run Prometheus:

```bash
docker run -d --name prometheus \
  -p 9090:9090 \
  -v $(pwd)/prometheus.yml:/etc/prometheus/prometheus.yml \
  prom/prometheus

# Open Prometheus UI
open http://localhost:9090
```

### Query Examples

In Prometheus UI, try these queries:

```promql
# Request rate
rate(http_requests_total[1m])

# P95 latency
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

# Routes by algorithm
sum by (algorithm) (evefrontier_routes_calculated_total)
```

---

## 7. Local Grafana Setup (Optional)

### Start Grafana

```bash
docker run -d --name grafana \
  -p 3000:3000 \
  grafana/grafana

# Default login: admin/admin
open http://localhost:3000
```

### Import Dashboard

1. Go to Dashboards → Import
2. Upload `docs/dashboards/evefrontier.json`
3. Select Prometheus data source
4. Click Import

---

## 8. Troubleshooting

### Metrics Not Appearing

```bash
# Check if metrics endpoint is accessible
curl -v http://localhost:8080/metrics

# Check for errors in logs
docker compose logs route | grep -i error
```

### Logs Not in JSON Format

```bash
# Verify LOG_FORMAT environment variable
docker compose exec route env | grep LOG_FORMAT

# Should be empty (defaults to json) or "json"
```

### Health Check Failing

```bash
# Check detailed readiness status
curl -v http://localhost:8080/health/ready

# Look for specific check failures in response
```

### Request ID Not Propagating

```bash
# Ensure X-Request-ID header is set
curl -X POST http://localhost:8080/api/v1/route \
  -H "Content-Type: application/json" \
  -H "X-Request-ID: my-custom-id-123" \
  -d '{"from": "Nod", "to": "Brana"}'

# Check logs for the custom ID
docker compose logs route | grep "my-custom-id-123"
```

---

## 9. Testing Checklist

Before submitting PR, verify:

- [ ] `curl /metrics` returns Prometheus format
- [ ] `http_requests_total` increments after requests
- [ ] `http_request_duration_seconds` has histogram buckets
- [ ] `evefrontier_routes_calculated_total` increments after route requests
- [ ] Logs are JSON by default
- [ ] Logs include `request_id` field
- [ ] `X-Request-ID` header is respected
- [ ] `/health/live` returns 200 with status "ok"
- [ ] `/health/ready` returns detailed checks

---

## Next Steps

After local verification:

1. Run full test suite: `cargo test --workspace`
2. Run integration tests: `cargo test -p evefrontier-service-shared`
3. Deploy to staging and verify Prometheus scraping
4. Import Grafana dashboard and verify panels populate
