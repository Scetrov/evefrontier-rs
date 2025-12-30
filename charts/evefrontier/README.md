# EVE Frontier Helm Chart

Helm chart for deploying EVE Frontier microservices to Kubernetes.

## Overview

This chart deploys three microservices for EVE Frontier pathfinding and exploration:

- **route**: Route planning between star systems
- **scout-gates**: Gate-connected neighbor discovery
- **scout-range**: Spatial range queries within a radius

## Prerequisites

- Kubernetes 1.24+
- Helm 3.8+
- Traefik Ingress Controller (if using IngressRoute)
- Persistent Volume provisioner (for dataset storage)

## Installation

### Add the Helm repository

```bash
helm repo add evefrontier https://evefrontier.github.io/charts
helm repo update
```

### Install the chart

```bash
# Install with default values
helm install evefrontier evefrontier/evefrontier

# Install with custom values
helm install evefrontier evefrontier/evefrontier -f values.yaml

# Install in a specific namespace
helm install evefrontier evefrontier/evefrontier -n evefrontier --create-namespace
```

### Install from local chart

```bash
# From repository root
helm install evefrontier ./charts/evefrontier
```

## Configuration

### Global Settings

| Parameter | Description | Default |
|-----------|-------------|---------|
| `global.imagePullSecrets` | Image pull secrets for private registries | `[]` |
| `global.dataPath` | Path to the dataset inside container | `/data/static_data.db` |

### Service Configuration

Each service (route, scoutGates, scoutRange) has the following configuration:

| Parameter | Description | Default |
|-----------|-------------|---------|
| `<service>.enabled` | Enable the service | `true` |
| `<service>.replicaCount` | Number of replicas | `1` |
| `<service>.image.repository` | Image repository | `ghcr.io/rslater-cs/evefrontier-service-<name>` |
| `<service>.image.tag` | Image tag (defaults to appVersion) | `""` |
| `<service>.image.pullPolicy` | Image pull policy | `IfNotPresent` |
| `<service>.service.type` | Kubernetes service type | `ClusterIP` |
| `<service>.service.port` | Service port | `8080` |
| `<service>.resources.requests.cpu` | CPU request | `100m` |
| `<service>.resources.requests.memory` | Memory request | `128Mi` |
| `<service>.resources.limits.cpu` | CPU limit | `500m` |
| `<service>.resources.limits.memory` | Memory limit | `256Mi` |
| `<service>.nodeSelector` | Node selector for pods | `{}` |
| `<service>.tolerations` | Tolerations for pods | `[]` |
| `<service>.affinity` | Affinity rules for pods | `{}` |

### Ingress Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `ingress.enabled` | Enable ingress | `true` |
| `ingress.className` | Ingress class (`traefik` or `nginx`) | `traefik` |
| `ingress.host` | Hostname for routing | `""` |
| `ingress.tls.enabled` | Enable TLS | `false` |
| `ingress.tls.secretName` | TLS secret name | `""` |
| `ingress.traefik.entryPoint` | Traefik entrypoint | `web` |
| `ingress.traefik.middlewares` | Middleware references | `[]` |
| `ingress.traefik.rateLimit.enabled` | Enable rate limiting | `false` |
| `ingress.traefik.rateLimit.average` | Average rate limit | `100` |
| `ingress.traefik.rateLimit.burst` | Burst rate limit | `50` |
| `ingress.traefik.cors.enabled` | Enable CORS middleware | `false` |

### Persistence Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `persistence.enabled` | Enable persistence | `true` |
| `persistence.existingClaim` | Use existing PVC | `""` |
| `persistence.storageClass` | Storage class | `""` |
| `persistence.accessModes` | Access modes | `["ReadOnlyMany"]` |
| `persistence.size` | Storage size | `1Gi` |
| `persistence.mountPath` | Mount path | `/data` |

### Security Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `serviceAccount.create` | Create service account | `true` |
| `serviceAccount.name` | Service account name | `""` |
| `podSecurityContext.runAsNonRoot` | Run as non-root | `true` |
| `podSecurityContext.runAsUser` | Run as user ID | `65532` |
| `securityContext.allowPrivilegeEscalation` | Allow privilege escalation | `false` |
| `securityContext.readOnlyRootFilesystem` | Read-only root filesystem | `true` |

### Probe Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `probes.liveness.enabled` | Enable liveness probe | `true` |
| `probes.liveness.path` | Liveness probe path | `/health/live` |
| `probes.liveness.initialDelaySeconds` | Initial delay | `5` |
| `probes.readiness.enabled` | Enable readiness probe | `true` |
| `probes.readiness.path` | Readiness probe path | `/health/ready` |

## Examples

### Production deployment with TLS

```yaml
ingress:
  enabled: true
  className: traefik
  host: evefrontier.example.com
  tls:
    enabled: true
    secretName: evefrontier-tls
  traefik:
    entryPoint: websecure
    middlewares:
      - evefrontier-ratelimit
      - evefrontier-cors
    rateLimit:
      enabled: true
      average: 100
      burst: 50
    cors:
      enabled: true
      allowOrigins:
        - "https://app.example.com"

route:
  replicaCount: 3
  resources:
    requests:
      cpu: 200m
      memory: 256Mi
    limits:
      cpu: 1000m
      memory: 512Mi
```

### Development deployment

```yaml
ingress:
  enabled: false

route:
  service:
    type: NodePort

persistence:
  enabled: true
  storageClass: standard
```

### Using existing PVC

```yaml
persistence:
  enabled: true
  existingClaim: my-evefrontier-data
```

## API Endpoints

Once deployed, the following endpoints are available:

### Route Planning

```bash
curl -X POST https://evefrontier.example.com/api/v1/route \
  -H "Content-Type: application/json" \
  -d '{"origin": "Nod", "destination": "Brana"}'
```

### Gate Discovery

```bash
curl -X POST https://evefrontier.example.com/api/v1/scout/gates \
  -H "Content-Type: application/json" \
  -d '{"system": "Nod"}'
```

### Range Query

```bash
curl -X POST https://evefrontier.example.com/api/v1/scout/range \
  -H "Content-Type: application/json" \
  -d '{"system": "Nod", "radius": 50}'
```

## Health Endpoints

All services expose health endpoints:

- `GET /health/live` - Liveness probe (is the service running?)
- `GET /health/ready` - Readiness probe (is the service ready for traffic?)

## Observability

### Prometheus Metrics

All services expose Prometheus metrics at `/metrics`. By default, pod annotations are added for automatic Prometheus discovery.

**Configuration:**

| Parameter | Description | Default |
|-----------|-------------|---------|
| `prometheus.enabled` | Enable Prometheus scraping annotations | `true` |
| `prometheus.port` | Metrics endpoint port | `8080` |
| `prometheus.path` | Metrics endpoint path | `/metrics` |

**Example Prometheus scrape config:**

```yaml
scrape_configs:
  - job_name: 'evefrontier'
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
```

### Logging

Services emit structured JSON logs by default.

**Configuration:**

| Parameter | Description | Default |
|-----------|-------------|---------|
| `config.logLevel` | Log level (trace, debug, info, warn, error) | `info` |
| `config.logFormat` | Log format (json, text) | `json` |

**Example log entry (JSON format):**

```json
{"timestamp":"2025-12-30T10:15:30Z","level":"INFO","service":"route","message":"Request completed","status":200}
```

### Grafana Dashboard

A pre-built Grafana dashboard is available at [`docs/dashboards/evefrontier-overview.json`](../../docs/dashboards/evefrontier-overview.json).

Import it into Grafana to visualize:
- Service health status
- Request rates and latencies
- Error rates
- Route calculations and failures
- Resource usage

### Alerting Rules

Prometheus alerting rules are available at [`docs/dashboards/alerting-rules.yaml`](../../docs/dashboards/alerting-rules.yaml).

For more details, see the [Observability Guide](../../docs/OBSERVABILITY.md).

## Troubleshooting

### Check pod status

```bash
kubectl get pods -l app.kubernetes.io/name=evefrontier
```

### View logs

```bash
kubectl logs -l app.kubernetes.io/name=evefrontier --all-containers
```

### Check events

```bash
kubectl get events --sort-by='.lastTimestamp'
```

### Verify dataset availability

```bash
kubectl exec -it deploy/evefrontier-route -- ls -la /data/
```

## Upgrading

```bash
helm upgrade evefrontier evefrontier/evefrontier -f values.yaml
```

## Uninstalling

```bash
helm uninstall evefrontier
```

**Note**: PVCs are not deleted by default. To remove:

```bash
kubectl delete pvc -l app.kubernetes.io/name=evefrontier
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.
