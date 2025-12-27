# Deploying EveFrontier Lambda Functions

This guide covers deploying the EveFrontier Lambda functions to AWS using Terraform.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Building Lambda Binaries](#building-lambda-binaries)
- [Configuration Reference](#configuration-reference)
- [Deployment](#deployment)
- [API Gateway Configuration](#api-gateway-configuration)
- [Monitoring & Logging](#monitoring--logging)
- [Operations](#operations)
- [Troubleshooting](#troubleshooting)
- [Security Considerations](#security-considerations)

---

## Prerequisites

### Required Tools

| Tool | Version | Purpose |
|------|---------|---------|
| Terraform | >= 1.5.0 | Infrastructure deployment |
| Rust | 1.91+ | Building Lambda binaries |
| AWS CLI | v2 | AWS authentication |
| Cross-compiler | Latest | ARM64 cross-compilation |

### AWS Requirements

- AWS account with appropriate permissions
- IAM user or role with a **custom least-privilege policy** (see below)
- Configured AWS credentials (`aws configure` or environment variables)

#### Deployment IAM Policy

> **Security Note:** Avoid attaching broad AWS-managed policies like `AWSLambda_FullAccess` or
> `IAMFullAccess` to deployment users/roles. These grant far more permissions than needed and
> increase blast radius if credentials are compromised. Use a narrowly scoped custom policy instead.

The deploying principal needs permissions for:
- Managing EveFrontier Lambda functions
- Managing EveFrontier API Gateway resources  
- Creating/managing CloudWatch log groups
- Minimal IAM actions to create the Lambda execution role

Example IAM policy. **Replace `<YOUR_REGION>` with your AWS region (e.g., `us-east-1`) and
`<YOUR_ACCOUNT_ID>` with your 12-digit AWS account ID** before using:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "LambdaFunctionManagement",
      "Effect": "Allow",
      "Action": [
        "lambda:CreateFunction",
        "lambda:UpdateFunctionCode",
        "lambda:UpdateFunctionConfiguration",
        "lambda:DeleteFunction",
        "lambda:GetFunction",
        "lambda:GetFunctionConfiguration",
        "lambda:AddPermission",
        "lambda:RemovePermission",
        "lambda:GetPolicy",
        "lambda:TagResource",
        "lambda:UntagResource"
      ],
      "Resource": "arn:aws:lambda:<YOUR_REGION>:<YOUR_ACCOUNT_ID>:function:evefrontier-*"
    },
    {
      "Sid": "ApiGatewayManagement",
      "Effect": "Allow",
      "Action": [
        "apigateway:GET",
        "apigateway:POST",
        "apigateway:PUT",
        "apigateway:PATCH",
        "apigateway:DELETE",
        "apigateway:TagResource",
        "apigateway:UntagResource"
      ],
      "Resource": [
        "arn:aws:apigateway:<YOUR_REGION>::/apis",
        "arn:aws:apigateway:<YOUR_REGION>::/apis/*"
      ]
    },
    {
      "Sid": "CloudWatchLogsManagement",
      "Effect": "Allow",
      "Action": [
        "logs:CreateLogGroup",
        "logs:DeleteLogGroup",
        "logs:DescribeLogGroups",
        "logs:PutRetentionPolicy",
        "logs:TagLogGroup",
        "logs:UntagLogGroup",
        "logs:ListTagsLogGroup"
      ],
      "Resource": [
        "arn:aws:logs:<YOUR_REGION>:<YOUR_ACCOUNT_ID>:log-group:/aws/lambda/evefrontier-*",
        "arn:aws:logs:<YOUR_REGION>:<YOUR_ACCOUNT_ID>:log-group:/aws/apigateway/evefrontier-*"
      ]
    },
    {
      "Sid": "LambdaExecutionRoleManagement",
      "Effect": "Allow",
      "Action": [
        "iam:CreateRole",
        "iam:GetRole",
        "iam:DeleteRole",
        "iam:TagRole",
        "iam:UntagRole",
        "iam:PutRolePolicy",
        "iam:GetRolePolicy",
        "iam:DeleteRolePolicy",
        "iam:ListRolePolicies",
        "iam:ListAttachedRolePolicies"
      ],
      "Resource": "arn:aws:iam::<YOUR_ACCOUNT_ID>:role/evefrontier-*"
    },
    {
      "Sid": "PassRoleToLambda",
      "Effect": "Allow",
      "Action": "iam:PassRole",
      "Resource": "arn:aws:iam::<YOUR_ACCOUNT_ID>:role/evefrontier-*",
      "Condition": {
        "StringEquals": {
          "iam:PassedToService": "lambda.amazonaws.com"
        }
      }
    }
  ]
}
```

> **⚠️ Security Note:** This policy grants `iam:CreateRole` and `iam:PutRolePolicy` on
> `evefrontier-*` roles combined with `iam:PassRole`. In highly sensitive environments, consider:
> - Using pre-created IAM roles instead of allowing Terraform to create them
> - Applying [IAM permissions boundaries](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies_boundaries.html)
>   to limit what policies can be attached to created roles
> - Restricting the deployment principal to a dedicated CI/CD pipeline with short-lived credentials
>
> These measures reduce the risk of privilege escalation if deployment credentials are compromised.

### Verify Prerequisites

```bash
# Check Terraform version
terraform version
# Should show >= 1.5.0

# Check Rust version
rustc --version
# Should show >= 1.91.0

# Check AWS credentials
aws sts get-caller-identity
# Should show your account details
```

---

## Quick Start

For experienced users who want to deploy quickly:

```bash
# 1. Build Lambda binaries (from repository root)
cargo build --release --target aarch64-unknown-linux-gnu \
  -p evefrontier-lambda-route \
  -p evefrontier-lambda-scout-gates \
  -p evefrontier-lambda-scout-range \
  --features bundle-data

# 2. Prepare binaries
cd terraform/examples/complete
mkdir -p binaries
cp ../../../target/aarch64-unknown-linux-gnu/release/evefrontier-lambda-route binaries/bootstrap-route
cp ../../../target/aarch64-unknown-linux-gnu/release/evefrontier-lambda-scout-gates binaries/bootstrap-scout-gates
cp ../../../target/aarch64-unknown-linux-gnu/release/evefrontier-lambda-scout-range binaries/bootstrap-scout-range

# 3. Configure and deploy
cp terraform.tfvars.example terraform.tfvars
# Edit terraform.tfvars as needed

terraform init
terraform apply

# 4. Test
API=$(terraform output -raw api_endpoint)
curl -X POST "${API}/route" -H "Content-Type: application/json" \
  -d '{"from": "Nod", "to": "Brana"}'
```

---

## Building Lambda Binaries

### Setting Up Cross-Compilation

Lambda runs on Amazon Linux 2023. For ARM64 deployment (recommended), you need cross-compilation.

> **Note on Rust targets**: This guide uses `aarch64-unknown-linux-gnu` which dynamically links
> against glibc. This works because the `provided.al2023` Lambda runtime includes glibc. An
> alternative is `aarch64-unknown-linux-musl` which produces fully static binaries, but requires
> musl toolchain setup and may have compatibility issues with some crates. For most use cases,
> the `gnu` target is simpler and works well with Amazon Linux 2023.

#### Ubuntu/Debian

```bash
# Install ARM64 cross-compiler
sudo apt-get update
sudo apt-get install -y gcc-aarch64-linux-gnu

# Add Rust target
rustup target add aarch64-unknown-linux-gnu

# Configure cargo for cross-compilation
cat >> ~/.cargo/config.toml << 'EOF'
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
EOF
```

#### macOS

```bash
# Install cross-compiler via Homebrew
brew tap messense/macos-cross-toolchains
brew install aarch64-unknown-linux-gnu

# Add Rust target
rustup target add aarch64-unknown-linux-gnu

# Configure cargo
cat >> ~/.cargo/config.toml << 'EOF'
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-unknown-linux-gnu-gcc"
EOF
```

#### Using Docker (Alternative)

```bash
# Build using cross-rs (handles all cross-compilation)
cargo install cross

# Build Lambda functions
cross build --release --target aarch64-unknown-linux-gnu \
  -p evefrontier-lambda-route \
  -p evefrontier-lambda-scout-gates \
  -p evefrontier-lambda-scout-range \
  --features bundle-data
```

### Build Commands

```bash
# From repository root
cd /path/to/evefrontier-rs

# Build for ARM64 (recommended - better price/performance)
cargo build --release --target aarch64-unknown-linux-gnu \
  -p evefrontier-lambda-route \
  -p evefrontier-lambda-scout-gates \
  -p evefrontier-lambda-scout-range \
  --features bundle-data

# OR build for x86_64 (if ARM64 is problematic)
cargo build --release --target x86_64-unknown-linux-gnu \
  -p evefrontier-lambda-route \
  -p evefrontier-lambda-scout-gates \
  -p evefrontier-lambda-scout-range \
  --features bundle-data
```

### Binary Sizes

Expected binary sizes with `bundle-data` feature:

| Binary | Approximate Size |
|--------|-----------------|
| evefrontier-lambda-route | ~15-20 MB |
| evefrontier-lambda-scout-gates | ~15-20 MB |
| evefrontier-lambda-scout-range | ~15-20 MB |

The large size is due to the bundled dataset and spatial index.

---

## Configuration Reference

### Terraform Module Variables

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `environment` | string | `"dev"` | Environment name (dev, staging, prod) |
| `project_name` | string | `"evefrontier"` | Prefix for all resources |
| `route_binary_path` | string | **required** | Path to route Lambda binary |
| `scout_gates_binary_path` | string | **required** | Path to scout-gates Lambda binary |
| `scout_range_binary_path` | string | **required** | Path to scout-range Lambda binary |
| `lambda_memory_mb` | number | `512` | Lambda memory (128-10240 MB) |
| `lambda_timeout_seconds` | number | `10` | Lambda timeout (1-900 seconds) |
| `lambda_architecture` | string | `"arm64"` | CPU architecture (arm64, x86_64) |
| `log_retention_days` | number | `30` | CloudWatch log retention |
| `log_level` | string | `"info"` | Log verbosity (trace, debug, info, warn, error) |
| `api_stage_name` | string | `"v1"` | API Gateway stage name |
| `cors_allowed_origins` | list | `["*"]` | CORS allowed origins |
| `throttling_burst_limit` | number | `100` | API burst limit (req/sec) |
| `throttling_rate_limit` | number | `50` | API rate limit (req/sec) |
| `vpc_config` | object | `null` | Optional VPC configuration |
| `tags` | map | `{}` | Additional resource tags |

### Example Configurations

#### Development

```hcl
environment            = "dev"
lambda_memory_mb       = 512
log_level              = "debug"
log_retention_days     = 7
cors_allowed_origins   = ["*"]
throttling_burst_limit = 50
throttling_rate_limit  = 25
```

#### Production

```hcl
environment            = "prod"
lambda_memory_mb       = 1024
log_level              = "info"
log_retention_days     = 90
cors_allowed_origins   = ["https://yourdomain.com"]
throttling_burst_limit = 500
throttling_rate_limit  = 200
```

---

## Deployment

### Initial Deployment

```bash
# Navigate to example directory
cd terraform/examples/complete

# Create binaries directory and copy built binaries
mkdir -p binaries
cp ../../../target/aarch64-unknown-linux-gnu/release/evefrontier-lambda-route binaries/bootstrap-route
cp ../../../target/aarch64-unknown-linux-gnu/release/evefrontier-lambda-scout-gates binaries/bootstrap-scout-gates
cp ../../../target/aarch64-unknown-linux-gnu/release/evefrontier-lambda-scout-range binaries/bootstrap-scout-range

# Copy and customize variables
cp terraform.tfvars.example terraform.tfvars
# Edit terraform.tfvars with your settings

# Initialize Terraform
terraform init

# Preview changes
terraform plan

# Apply changes
terraform apply
```

### Remote State (Recommended for Production)

Uncomment and configure the S3 backend in `main.tf`:

```hcl
terraform {
  backend "s3" {
    bucket         = "your-terraform-state-bucket"
    key            = "evefrontier/terraform.tfstate"
    region         = "us-east-1"
    encrypt        = true
    dynamodb_table = "terraform-state-lock"
  }
}
```

### Updating Lambda Functions

After rebuilding binaries:

```bash
# Rebuild binaries
cargo build --release --target aarch64-unknown-linux-gnu ...

# Copy new binaries
cp ../../../target/aarch64-unknown-linux-gnu/release/evefrontier-lambda-* binaries/

# Apply changes (Terraform detects binary changes via hash)
terraform apply
```

---

## API Gateway Configuration

### Endpoints

| Method | Path | Lambda Function | Description |
|--------|------|-----------------|-------------|
| POST | `/route` | evefrontier-route | Calculate route between systems |
| POST | `/scout-gates` | evefrontier-scout-gates | Find gate-connected neighbors |
| POST | `/scout-range` | evefrontier-scout-range | Find systems within spatial range |

### Request/Response Examples

See [USAGE.md](./USAGE.md#aws-lambda-functions) for detailed API documentation.

#### Route Endpoint

```bash
curl -X POST "${API_ENDPOINT}/route" \
  -H "Content-Type: application/json" \
  -d '{
    "from": "Nod",
    "to": "Brana",
    "algorithm": "astar",
    "max_jump_ly": 80,
    "avoid_systems": []
  }'
```

#### Scout Gates Endpoint

```bash
curl -X POST "${API_ENDPOINT}/scout-gates" \
  -H "Content-Type: application/json" \
  -d '{
    "system": "Nod"
  }'
```

#### Scout Range Endpoint

```bash
curl -X POST "${API_ENDPOINT}/scout-range" \
  -H "Content-Type: application/json" \
  -d '{
    "system": "Nod",
    "radius_ly": 50,
    "max_temperature": 400.0
  }'
```

### CORS Configuration

Default configuration allows all origins (`*`). For production, restrict to your domain:

```hcl
cors_allowed_origins = ["https://yourdomain.com", "https://api.yourdomain.com"]
```

### Rate Limiting

Default throttling limits:
- **Burst limit**: 100 requests/second
- **Rate limit**: 50 requests/second (sustained)

Adjust based on expected traffic:

```hcl
throttling_burst_limit = 500
throttling_rate_limit  = 200
```

---

## Monitoring & Logging

### CloudWatch Log Groups

The module creates log groups for each Lambda function:

| Log Group | Retention | Content |
|-----------|-----------|---------|
| `/aws/lambda/evefrontier-route-{env}` | Configurable | Route Lambda logs |
| `/aws/lambda/evefrontier-scout-gates-{env}` | Configurable | Scout-gates Lambda logs |
| `/aws/lambda/evefrontier-scout-range-{env}` | Configurable | Scout-range Lambda logs |
| `/aws/apigateway/evefrontier-api-{env}` | Configurable | API Gateway access logs |

### Log Format

Lambda functions use structured JSON logging via `tracing`:

```json
{
  "timestamp": "2024-01-15T10:30:00.000Z",
  "level": "INFO",
  "message": "Route calculated",
  "target": "evefrontier_lambda_route",
  "fields": {
    "from": "Nod",
    "to": "Brana",
    "hops": 5,
    "duration_ms": 42
  }
}
```

### CloudWatch Insights Queries

> **Note:** These queries assume JSON-formatted logs as shown above. Adjust field names and
> patterns based on your actual log output structure.

```sql
-- Find slow requests (adjust field path based on actual log structure)
fields @timestamp, @message
| filter @message like /duration_ms/
| parse @message /\"duration_ms\":\s*(\d+)/ as duration
| filter duration > 1000
| sort @timestamp desc
| limit 100

-- Error rate
fields @timestamp, @message
| filter @message like /ERROR/
| stats count(*) as errors by bin(1h)

-- Cold starts
fields @timestamp, @message
| filter @message like /cold_start/
| stats count(*) as cold_starts by bin(1h)
```

### Recommended Alarms

Create CloudWatch alarms for:

1. **Error Rate**: Alert when Lambda errors exceed threshold
2. **Duration**: Alert when P99 latency exceeds SLA
3. **Throttling**: Alert when API Gateway throttles requests
4. **Memory**: Alert when Lambda memory usage approaches limit

---

## Operations

### Updating Lambda Code

```bash
# 1. Rebuild with new code
cargo build --release --target aarch64-unknown-linux-gnu ...

# 2. Copy binaries
cp target/*/release/evefrontier-lambda-* terraform/examples/complete/binaries/

# 3. Apply changes
cd terraform/examples/complete
terraform apply
```

### Rolling Back

```bash
# Option 1: Terraform state (if not destroyed)
terraform state list
terraform state show module.evefrontier.aws_lambda_function.route

# Option 2: Redeploy previous version
git checkout <previous-commit>
cargo build ...
terraform apply

# Option 3: AWS Console
# Use Lambda's built-in version/alias feature
```

### Scaling

Lambda scales automatically. To adjust:

```hcl
# Reserve concurrency (limits scaling)
lambda_reserved_concurrency = 100

# Or use provisioned concurrency (reduces cold starts)
# Add to module: provisioned_concurrent_executions = 5
```

### Cost Optimization

1. **ARM64 architecture**: Generally cheaper than x86_64 (see [AWS Lambda Pricing](https://aws.amazon.com/lambda/pricing/) for current rates in your region)
2. **Right-size memory**: 512MB is sufficient for most workloads
3. **Log retention**: Reduce `log_retention_days` for dev environments
4. **Provisioned concurrency**: Only if cold starts are unacceptable

---

## Troubleshooting

### Common Issues

#### "Unable to import module 'bootstrap'"

**Cause**: Binary not named `bootstrap` or wrong architecture.

**Solution**:
```bash
# Verify binary name
ls -la binaries/
# Should show: bootstrap-route, bootstrap-scout-gates, bootstrap-scout-range

# Verify architecture
file binaries/bootstrap-route
# Should show: ELF 64-bit LSB pie executable, ARM aarch64
```

#### "Task timed out after X seconds"

**Cause**: Lambda timeout too short or cold start + processing exceeds limit.

**Solution**:
```hcl
lambda_timeout_seconds = 30  # Increase timeout
lambda_memory_mb       = 1024  # More memory = faster CPU
```

#### "AccessDeniedException: User is not authorized"

**Cause**: AWS credentials lack required permissions.

**Solution**: See the [Deployment IAM Policy](#deployment-iam-policy) section for a complete
least-privilege policy template.

**Diagnosing the specific missing permission:**

1. Check AWS CloudTrail for the denied API call:
   ```bash
   aws cloudtrail lookup-events \
     --lookup-attributes AttributeKey=EventName,AttributeValue=<API_CALL> \
     --max-results 5
   ```

2. Look for `errorCode: AccessDenied` or `UnauthorizedAccess` in the event details

3. Add only the specific missing permission to your policy, scoped to the appropriate resources

**Required permissions summary** (see Deployment IAM Policy for full details):

- **Lambda**: `CreateFunction`, `UpdateFunctionCode`, `UpdateFunctionConfiguration`, `DeleteFunction`,
  `GetFunction`, `GetFunctionConfiguration`, `AddPermission`, `RemovePermission`, `GetPolicy`,
  `TagResource`, `UntagResource`
- **API Gateway**: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`
- **CloudWatch Logs**: `CreateLogGroup`, `DeleteLogGroup`, `DescribeLogGroups`, `PutRetentionPolicy`
- **IAM**: `CreateRole`, `GetRole`, `DeleteRole`, `PutRolePolicy`, `GetRolePolicy`, `DeleteRolePolicy`,
  `PassRole` (with condition)

> **Security Note:** Never use wildcard action patterns like `lambda:*` or `iam:*` to resolve
> permission errors. These effectively create admin-level access and increase blast radius if
> credentials are compromised. Always diagnose and add only the specific missing permission.

#### "CORS error in browser"

**Cause**: Origin not in allowed list.

**Solution**:
```hcl
cors_allowed_origins = ["https://yourdomain.com"]
```

#### "Cold start latency is high"

**Cause**: First invocation loads bundled dataset into memory.

**Solutions**:
1. Increase memory (faster CPU allocation)
2. Use provisioned concurrency
3. Implement a "warmer" Lambda that pings periodically

### Debug Mode

Enable debug logging:

```hcl
log_level = "debug"
```

Then check CloudWatch Logs for detailed traces.

---

## Security Considerations

### IAM Least Privilege

The module creates a minimal IAM execution role policy for the Lambda functions. The actual policy
can be found in [`terraform/modules/evefrontier-lambda/iam.tf`](../terraform/modules/evefrontier-lambda/iam.tf).
The policy grants only CloudWatch Logs write permissions:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "CloudWatchLogsWrite",
      "Effect": "Allow",
      "Action": [
        "logs:CreateLogStream",
        "logs:PutLogEvents"
      ],
      "Resource": [
        "arn:aws:logs:<REGION>:<ACCOUNT_ID>:log-group:/aws/lambda/<PROJECT_NAME>-route-<ENV>:*",
        "arn:aws:logs:<REGION>:<ACCOUNT_ID>:log-group:/aws/lambda/<PROJECT_NAME>-scout-gates-<ENV>:*",
        "arn:aws:logs:<REGION>:<ACCOUNT_ID>:log-group:/aws/lambda/<PROJECT_NAME>-scout-range-<ENV>:*"
      ]
    }
  ]
}
```

> **Note:** Log groups are created by Terraform, not Lambda. The Lambda execution role does not
> include `logs:CreateLogGroup` because Terraform manages log group lifecycle. If log groups are
> accidentally deleted, run `terraform apply` to recreate them.
>
> The placeholders `<REGION>`, `<ACCOUNT_ID>`, `<PROJECT_NAME>`, and `<ENV>` are resolved at
> deployment time based on your AWS configuration and Terraform variables.

Lambda functions have **no access** to:
- S3 buckets
- DynamoDB tables
- Secrets Manager
- Other AWS services

### No Secrets

- Dataset is bundled at build time (no runtime downloads)
- No API keys or credentials in Lambda environment
- No database connections
### CORS

For production:

```hcl
cors_allowed_origins = ["https://yourdomain.com"]  # Restrict to your domain
```

### VPC Deployment

For private deployments:

```hcl
vpc_config = {
  subnet_ids         = ["subnet-private-1", "subnet-private-2"]
  security_group_ids = ["sg-lambda-egress"]
}
```

**Note**: VPC deployment requires NAT Gateway for outbound internet access (if needed).

### API Authentication

The module does not include authentication. Options to add:

1. **API Key**: Add `aws_apigatewayv2_api_key` resource
2. **IAM**: Use AWS_IAM authorization
3. **Cognito**: Add JWT authorizer
4. **Lambda Authorizer**: Custom authentication logic

---

## Destroying Resources

To remove all deployed resources:

```bash
cd terraform/examples/complete
terraform destroy
```

**Warning**: This permanently deletes:
- Lambda functions
- API Gateway
- CloudWatch Log Groups (and all logs)
- IAM roles and policies

---

## Next Steps

- [USAGE.md](./USAGE.md) - API usage documentation and SDK examples
- [Module README](../terraform/modules/evefrontier-lambda/README.md) - Terraform module reference
- [Example README](../terraform/examples/complete/README.md) - Step-by-step deployment guide
