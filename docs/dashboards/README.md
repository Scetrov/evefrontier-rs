# Grafana Dashboards & Alerting Rules

This directory contains Grafana dashboard definitions and Prometheus alerting rules for the EVE
Frontier microservices.

## Contents

| File | Description |
|------|-------------|
| `evefrontier-overview.json` | Main service overview dashboard |
| `alerting-rules.yaml` | Prometheus alerting rules |

## Dashboard: EVE Frontier - Service Overview

A comprehensive dashboard showing:

### Service Health Overview Row
- Service status indicators (UP/DOWN) for Route, Scout Gates, Scout Range
- Overall success rate percentage
- Request rate (requests/second)
- P95 latency

### Request Metrics Row
- Request rate by endpoint (time series)
- Request latency by endpoint (P50, P95, P99)
- Request rate by status code (2xx, 4xx, 5xx stacked)
- Success rate by endpoint

### Business Metrics Row
- Routes calculated per minute by algorithm
- Route failures per minute by reason
- Route hops distribution (P50, P95 by algorithm)
- Systems queried per minute by type

### Resource Usage Row
- Memory usage (RSS) by service
- CPU usage by service

## Installation

### Grafana Dashboard

1. Open Grafana and navigate to **Dashboards → Import**
2. Upload `evefrontier-overview.json` or paste its contents
3. Select your Prometheus data source
4. Click **Import**

Alternatively, use the Grafana API:

```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $GRAFANA_API_KEY" \
  -d @evefrontier-overview.json \
  "$GRAFANA_URL/api/dashboards/db"
```

### Prometheus Alerting Rules

Add the alerting rules to your Prometheus configuration:

```yaml
# prometheus.yml
rule_files:
  - /etc/prometheus/rules/evefrontier-alerting-rules.yaml
```

Then copy the rules file:

```bash
cp alerting-rules.yaml /etc/prometheus/rules/evefrontier-alerting-rules.yaml
```

Reload Prometheus configuration:

```bash
curl -X POST http://localhost:9090/-/reload
```

## Alert Severity Levels

| Severity | Description | Response Time |
|----------|-------------|---------------|
| `critical` | Service outage or severe degradation | Immediate |
| `warning` | Performance degradation or elevated errors | Within 1 hour |
| `info` | Informational, no immediate action required | Review daily |

## Alert Reference

### Critical Alerts

- **HighErrorRate**: >5% of requests returning 5xx errors for 5 minutes
- **ServiceDown**: A service instance is unreachable for 1 minute
- **AllInstancesDown**: All instances of a service are down

### Warning Alerts

- **HighLatency**: P95 latency exceeds 1 second for 10 minutes
- **HighRouteFailureRate**: >10% route calculation failures for 15 minutes
- **Elevated4xxRate**: >20% client errors for 15 minutes
- **HighMemoryUsage**: Memory usage exceeds 80% of limit

### Info Alerts

- **LowTraffic**: Request rate below 0.01 req/s for 30 minutes
- **NoRoutesCalculated**: No routes processed in 1 hour

## Customization

### Adjusting Thresholds

Edit `alerting-rules.yaml` to adjust thresholds:

```yaml
# Example: Change error rate threshold from 5% to 3%
- alert: HighErrorRate
  expr: |
    sum(rate(http_requests_total{status=~"5xx"}[5m]))
    / sum(rate(http_requests_total[5m])) > 0.03  # Changed from 0.05
```

### Adding New Panels

1. Edit `evefrontier-overview.json` directly, or
2. Create the panel in Grafana UI, then export the dashboard JSON

### Dashboard Variables

The dashboard uses a `${datasource}` variable for Prometheus data source selection. This allows
switching between multiple Prometheus instances (e.g., staging vs production).

## Related Documentation

- [Prometheus Metrics Contract](../../specs/008-microservices-observability/contracts/metrics.md)
- [Deployment Guide](../DEPLOYMENT.md)
- [Architecture Overview](../ARCHITECTURE.md)
