# EVE Frontier Lambda Module

Terraform module for deploying [EVE Frontier](https://github.com/Scetrov/evefrontier-rs) Lambda functions with API Gateway.

## Features

- **Three Lambda Functions**: Route planning, gate scouting, and spatial range queries
- **HTTP API Gateway**: Low-latency API with CORS support
- **Least-Privilege IAM**: CloudWatch Logs only (no unnecessary permissions)
- **Configurable**: Memory, timeout, logging, throttling all customizable
- **ARM64 by Default**: Typically offers better price/performance for Rust workloads (see [AWS Lambda Pricing](https://aws.amazon.com/lambda/pricing/))

## Requirements

| Name | Version |
|------|---------|
| terraform | >= 1.5.0 |
| aws | >= 5.0.0, < 6.0.0 |
| archive | >= 2.4.0 |

## Prerequisites

1. **Built Lambda binaries** with the `bundle-data` feature:

   ```bash
   # Build for arm64 (recommended)
   cargo build --release --target aarch64-unknown-linux-gnu \
     -p evefrontier-lambda-route \
     -p evefrontier-lambda-scout-gates \
     -p evefrontier-lambda-scout-range \
     --features bundle-data

   # Rename binaries to 'bootstrap' for Lambda custom runtime
   cp target/aarch64-unknown-linux-gnu/release/evefrontier-lambda-route ./bootstrap-route
   cp target/aarch64-unknown-linux-gnu/release/evefrontier-lambda-scout-gates ./bootstrap-scout-gates
   cp target/aarch64-unknown-linux-gnu/release/evefrontier-lambda-scout-range ./bootstrap-scout-range
   ```

2. **AWS credentials** configured (via environment variables, AWS profile, or IAM role)

## Usage

### Basic Example

```hcl
module "evefrontier" {
  source = "path/to/modules/evefrontier-lambda"

  environment = "prod"

  # Paths to built Lambda binaries (named 'bootstrap')
  route_binary_path       = "${path.module}/binaries/bootstrap-route"
  scout_gates_binary_path = "${path.module}/binaries/bootstrap-scout-gates"
  scout_range_binary_path = "${path.module}/binaries/bootstrap-scout-range"
}

output "api_endpoint" {
  value = module.evefrontier.api_endpoint
}
```

### Complete Example with All Options

```hcl
module "evefrontier" {
  source = "path/to/modules/evefrontier-lambda"

  # Core configuration
  environment  = "prod"
  project_name = "evefrontier"

  # Lambda configuration
  lambda_memory_mb            = 512
  lambda_timeout_seconds      = 10
  lambda_architecture         = "arm64"
  lambda_reserved_concurrency = -1  # Unreserved (auto-scaling)

  # Binary paths
  route_binary_path       = "${path.module}/binaries/bootstrap-route"
  scout_gates_binary_path = "${path.module}/binaries/bootstrap-scout-gates"
  scout_range_binary_path = "${path.module}/binaries/bootstrap-scout-range"

  # Logging
  log_retention_days = 30
  log_level          = "info"

  # API Gateway
  api_stage_name         = "v1"
  cors_allowed_origins   = ["https://yourdomain.com"]
  throttling_burst_limit = 100
  throttling_rate_limit  = 50

  # Optional: VPC configuration
  # vpc_config = {
  #   subnet_ids         = ["subnet-abc123", "subnet-def456"]
  #   security_group_ids = ["sg-123456"]
  # }

  # Tags
  tags = {
    Team      = "Platform"
    CostCenter = "Engineering"
  }
}
```

## Inputs

| Name | Description | Type | Default | Required |
|------|-------------|------|---------|:--------:|
| `environment` | Deployment environment (dev, staging, prod) | `string` | `"dev"` | no |
| `project_name` | Project name prefix for resources | `string` | `"evefrontier"` | no |
| `route_binary_path` | Path to route Lambda binary | `string` | - | **yes** |
| `scout_gates_binary_path` | Path to scout-gates Lambda binary | `string` | - | **yes** |
| `scout_range_binary_path` | Path to scout-range Lambda binary | `string` | - | **yes** |
| `lambda_memory_mb` | Lambda memory allocation (MB) | `number` | `512` | no |
| `lambda_timeout_seconds` | Lambda timeout (seconds) | `number` | `10` | no |
| `lambda_architecture` | CPU architecture (arm64, x86_64) | `string` | `"arm64"` | no |
| `lambda_reserved_concurrency` | Reserved concurrency (-1 for unreserved) | `number` | `-1` | no |
| `log_retention_days` | CloudWatch log retention | `number` | `30` | no |
| `log_level` | Lambda log level | `string` | `"info"` | no |
| `api_stage_name` | API Gateway stage name | `string` | `"v1"` | no |
| `cors_allowed_origins` | CORS allowed origins | `list(string)` | `["*"]` | no |
| `cors_allowed_methods` | CORS allowed methods | `list(string)` | `["POST", "OPTIONS"]` | no |
| `cors_allowed_headers` | CORS allowed headers | `list(string)` | *(see variables.tf)* | no |
| `cors_max_age_seconds` | CORS max age | `number` | `300` | no |
| `throttling_burst_limit` | API throttling burst limit | `number` | `100` | no |
| `throttling_rate_limit` | API throttling rate limit | `number` | `50` | no |
| `vpc_config` | Optional VPC configuration | `object` | `null` | no |
| `tags` | Additional tags for resources | `map(string)` | `{}` | no |

## Outputs

| Name | Description |
|------|-------------|
| `api_endpoint` | Base URL for API Gateway |
| `api_id` | API Gateway ID |
| `api_execution_arn` | API Gateway execution ARN |
| `route_function_name` | Route Lambda function name |
| `route_function_arn` | Route Lambda function ARN |
| `scout_gates_function_name` | Scout-gates Lambda function name |
| `scout_gates_function_arn` | Scout-gates Lambda function ARN |
| `scout_range_function_name` | Scout-range Lambda function name |
| `scout_range_function_arn` | Scout-range Lambda function ARN |
| `lambda_execution_role_arn` | IAM execution role ARN |
| `log_group_names` | Map of log group names |
| `endpoint_urls` | Complete endpoint URLs |

## API Endpoints

After deployment, the following endpoints are available:

| Method | Path | Description |
|--------|------|-------------|
| POST | `/route` | Calculate route between systems |
| POST | `/scout-gates` | Find gate-connected neighbors |
| POST | `/scout-range` | Find systems within spatial range |

### Example API Calls

```bash
# First, export the API endpoint from Terraform output
export API_ENDPOINT=$(terraform output -raw api_endpoint)

# Calculate a route
curl -X POST "${API_ENDPOINT}/route" \
  -H "Content-Type: application/json" \
  -d '{"from": "Nod", "to": "Brana", "algorithm": "astar"}'

# Find gate neighbors
curl -X POST "${API_ENDPOINT}/scout-gates" \
  -H "Content-Type: application/json" \
  -d '{"system": "Nod"}'

# Find systems within range
curl -X POST "${API_ENDPOINT}/scout-range" \
  -H "Content-Type: application/json" \
  -d '{"system": "Nod", "radius_ly": 50}'
```

## Security Considerations

- **IAM Least Privilege**: Lambda functions only have CloudWatch Logs permissions
- **No Secrets**: No API keys or credentials stored in Lambda configuration
- **CORS**: Configure `cors_allowed_origins` appropriately for production
- **Throttling**: Default limits prevent abuse; adjust for your use case
- **VPC**: Use `vpc_config` for private deployments (requires NAT for outbound)

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     API Gateway (HTTP)                       │
│                                                              │
│  POST /route ──────► Lambda: evefrontier-route              │
│  POST /scout-gates ─► Lambda: evefrontier-scout-gates       │
│  POST /scout-range ─► Lambda: evefrontier-scout-range       │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    CloudWatch Logs                           │
│                                                              │
│  /aws/lambda/evefrontier-route-{env}                        │
│  /aws/lambda/evefrontier-scout-gates-{env}                  │
│  /aws/lambda/evefrontier-scout-range-{env}                  │
│  /aws/apigateway/evefrontier-api-{env}                      │
└─────────────────────────────────────────────────────────────┘
```

## License

MIT License - see [LICENSE](../../../LICENSE) for details.
