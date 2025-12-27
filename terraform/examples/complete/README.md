# Complete Example: EveFrontier Lambda Deployment

This example demonstrates a complete deployment of the EveFrontier Lambda functions with HTTP API Gateway.

## Prerequisites

1. **AWS CLI** configured with appropriate credentials
2. **Terraform** >= 1.5.0 installed
3. **Rust toolchain** with cross-compilation target:

   ```bash
   rustup target add aarch64-unknown-linux-gnu
   ```

4. **Cross-compilation linker** (for building arm64 binaries on x86_64):

   ```bash
   # Ubuntu/Debian
   sudo apt-get install gcc-aarch64-linux-gnu

   # macOS (via Homebrew)
   brew tap messense/macos-cross-toolchains
   brew install aarch64-unknown-linux-gnu
   ```

## Step 1: Build Lambda Binaries

From the repository root, build the Lambda functions with bundled data:

```bash
# Set up cross-compilation (adjust path for your system)
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc

# Build all Lambda functions for arm64
cargo build --release --target aarch64-unknown-linux-gnu \
  -p evefrontier-lambda-route \
  -p evefrontier-lambda-scout-gates \
  -p evefrontier-lambda-scout-range \
  --features bundle-data
```

## Step 2: Prepare Binaries Directory

Create the binaries directory and copy the built binaries:

```bash
# From this example directory
mkdir -p binaries

# Copy and rename binaries (Lambda expects 'bootstrap' naming)
cp ../../../target/aarch64-unknown-linux-gnu/release/evefrontier-lambda-route binaries/bootstrap-route
cp ../../../target/aarch64-unknown-linux-gnu/release/evefrontier-lambda-scout-gates binaries/bootstrap-scout-gates
cp ../../../target/aarch64-unknown-linux-gnu/release/evefrontier-lambda-scout-range binaries/bootstrap-scout-range
```

## Step 3: Configure Variables

Copy the example variables file and customize:

```bash
cp terraform.tfvars.example terraform.tfvars

# Edit terraform.tfvars with your settings
# - aws_region: Your preferred AWS region
# - environment: dev, staging, or prod
# - binaries_path: Path to binaries (default: ./binaries)
```

## Step 4: Deploy

Initialize and apply the Terraform configuration:

```bash
# Initialize Terraform
terraform init

# Preview changes
terraform plan

# Apply (creates all resources)
terraform apply
```

## Step 5: Test the Deployment

After successful deployment, Terraform outputs the API endpoints:

```bash
# Get the API endpoint
API_ENDPOINT=$(terraform output -raw api_endpoint)

# Test route endpoint
curl -X POST "${API_ENDPOINT}/route" \
  -H "Content-Type: application/json" \
  -d '{"from": "Nod", "to": "Brana", "algorithm": "astar"}'

# Test scout-gates endpoint
curl -X POST "${API_ENDPOINT}/scout-gates" \
  -H "Content-Type: application/json" \
  -d '{"system": "Nod"}'

# Test scout-range endpoint
curl -X POST "${API_ENDPOINT}/scout-range" \
  -H "Content-Type: application/json" \
  -d '{"system": "Nod", "radius_ly": 50}'
```

## Step 6: Clean Up

To destroy all created resources:

```bash
terraform destroy
```

## Customization

### Production Configuration

For production deployments, consider:

1. **Remote State**: Uncomment and configure the S3 backend in `main.tf`
2. **CORS**: Restrict `cors_allowed_origins` to your domain
3. **VPC**: Use `vpc_config` for private deployments
4. **Monitoring**: Set up CloudWatch alarms for errors and latency

### Example Production Variables

```hcl
# In your terraform.tfvars for production
aws_region  = "us-east-1"
environment = "prod"

# In the module call, override defaults:
module "evefrontier" {
  # ...
  
  cors_allowed_origins   = ["https://yourdomain.com"]
  throttling_burst_limit = 500
  throttling_rate_limit  = 200
  log_retention_days     = 90
}
```

## Troubleshooting

### Binary Not Found

Ensure binaries exist and are named correctly:

```bash
ls -la binaries/
# Should show:
# bootstrap-route
# bootstrap-scout-gates
# bootstrap-scout-range
```

### Permission Denied

Check AWS credentials:

```bash
aws sts get-caller-identity
```

### Lambda Timeout

If routes timeout, increase `lambda_timeout_seconds` in the module:

```hcl
module "evefrontier" {
  # ...
  lambda_timeout_seconds = 30
}
```

### Cold Start Issues

First request after deployment may be slow due to cold start. The bundled dataset loads into memory on first invocation.

## Architecture

```
                    ┌─────────────────┐
                    │   API Gateway   │
                    │   (HTTP API)    │
                    └────────┬────────┘
                             │
           ┌─────────────────┼─────────────────┐
           │                 │                 │
           ▼                 ▼                 ▼
    ┌─────────────┐   ┌─────────────┐   ┌─────────────┐
    │    Route    │   │ Scout Gates │   │ Scout Range │
    │   Lambda    │   │   Lambda    │   │   Lambda    │
    └─────────────┘   └─────────────┘   └─────────────┘
           │                 │                 │
           └─────────────────┼─────────────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │  CloudWatch     │
                    │     Logs        │
                    └─────────────────┘
```

## Cost Estimation

Approximate monthly costs (us-east-1). Verify current pricing at [AWS Lambda Pricing](https://aws.amazon.com/lambda/pricing/) and [API Gateway Pricing](https://aws.amazon.com/api-gateway/pricing/):

| Component | Free Tier | Beyond Free Tier (example) |
|-----------|-----------|----------------------------|
| Lambda (1M requests) | 1M requests free | ~$0.20/1M requests |
| Lambda (GB-seconds) | 400K GB-s free | ~$0.0000166667/GB-s |
| API Gateway | 1M requests free | ~$1.00/1M requests |
| CloudWatch Logs | 5GB ingestion free | ~$0.50/GB |

> **Note:** Pricing varies by region and changes over time. Always consult the AWS Pricing Calculator for accurate cost estimates.

For low-traffic use cases, the deployment typically remains within free tier limits.
