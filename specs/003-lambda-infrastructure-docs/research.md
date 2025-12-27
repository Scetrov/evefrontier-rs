# Research: Lambda Infrastructure Documentation

**Feature**: 003-lambda-infrastructure-docs  
**Created**: 2025-12-27  
**Status**: Complete

---

## Research Questions

### 1. Terraform vs SAM vs CDK

**Decision**: Terraform  
**Rationale**: 
- ADR 0007 explicitly mentions Terraform for infrastructure
- Team familiarity with Terraform (implied by ADR)
- No additional language runtime required (SAM needs Python/Node, CDK needs TypeScript)
- Better multi-cloud portability if needed later
- Mature ecosystem and documentation

**Alternatives Considered**:
- AWS SAM: Simpler for Lambda-only, but less flexible
- AWS CDK: Powerful but adds TypeScript dependency
- Pulumi: Good but smaller community, less documentation

### 2. API Gateway Type

**Decision**: HTTP API (API Gateway v2)  
**Rationale**:
- 70% lower cost than REST API
- Lower latency (5-10ms vs 29ms)
- Simpler configuration for Lambda proxy
- Built-in CORS support
- Sufficient for JSON API endpoints

**Alternatives Considered**:
- REST API: More features (WAF, caching) but overkill for this use case
- ALB: Requires VPC, more complex setup

### 3. Lambda Runtime Configuration

**Decision**: `provided.al2023` (Amazon Linux 2023)  
**Rationale**:
- Custom Rust runtime using `lambda_runtime` crate
- AL2023 is latest supported, better security
- `bootstrap` binary naming convention

**Configuration Defaults**:
| Setting | Default | Rationale |
|---------|---------|-----------|
| Memory | 512 MB | Sufficient for starmap + spatial index |
| Timeout | 10 seconds | Routes compute in <1s typically |
| Architecture | arm64 | Better price/performance |
| Reserved Concurrency | None | Allow auto-scaling |

### 4. IAM Policy Scope

**Decision**: CloudWatch Logs only (minimal)  
**Rationale**:
- Lambda functions don't need S3, DynamoDB, or other AWS services
- Dataset is bundled at build time (no runtime downloads)
- Secrets not required (no API keys in current implementation)

**Expansion Path** (if needed later):
- Secrets Manager: For API keys or rate limiting tokens
- S3: For dynamic dataset updates (not currently planned)
- X-Ray: For distributed tracing (optional enhancement)

### 5. Terraform Module Organization

**Decision**: Single module with all three Lambdas  
**Rationale**:
- Functions share common patterns (runtime, IAM, logging)
- Simpler deployment (one `terraform apply`)
- API Gateway routes naturally grouped

**Module Inputs**:
```hcl
variable "environment" {
  description = "Deployment environment (dev, staging, prod)"
  type        = string
  default     = "dev"
}

variable "lambda_memory_mb" {
  description = "Memory allocation for Lambda functions"
  type        = number
  default     = 512
}

variable "lambda_timeout_seconds" {
  description = "Timeout for Lambda functions"
  type        = number
  default     = 10
}

variable "log_retention_days" {
  description = "CloudWatch log retention period"
  type        = number
  default     = 30
}

variable "cors_allowed_origins" {
  description = "CORS allowed origins"
  type        = list(string)
  default     = ["*"]
}
```

### 6. Binary Packaging

**Decision**: ZIP file with `bootstrap` binary  
**Rationale**:
- Standard Lambda custom runtime packaging
- Simple CI/CD integration
- No container registry required

**Build Commands**:
```bash
# Build for Lambda (ARM64)
cargo build --release --target aarch64-unknown-linux-musl \
  -p evefrontier-lambda-route \
  -p evefrontier-lambda-scout-gates \
  -p evefrontier-lambda-scout-range \
  --features bundle-data

# Package (example for route function)
cp target/aarch64-unknown-linux-musl/release/evefrontier-lambda-route bootstrap
zip lambda-route.zip bootstrap
```

### 7. Documentation Structure

**Decision**: Single `docs/DEPLOYMENT.md` with sections  
**Rationale**:
- Keeps deployment info in one place
- Easy to find and update
- Links to Terraform module README for details

**Outline**:
1. **Prerequisites**: AWS CLI, Terraform, built binaries
2. **Quick Start**: Minimal 5-minute deployment
3. **Configuration Reference**: All variables explained
4. **API Gateway**: Routes, CORS, authentication options
5. **Monitoring**: CloudWatch Logs, metrics, alerts
6. **Operations**: Updates, rollbacks, scaling
7. **Troubleshooting**: Common errors and solutions
8. **Security**: IAM review, VPC options

---

## Best Practices Identified

### Terraform Module Standards

1. **Versioning**: Use semantic versioning for module releases
2. **Descriptions**: All variables and outputs must have descriptions
3. **Defaults**: Provide sensible defaults where possible
4. **Validation**: Add variable validation rules
5. **Examples**: Include complete working example
6. **README**: Document all inputs, outputs, and usage

### Security Considerations

1. **No wildcards in IAM**: Scope to specific resources
2. **Encryption**: CloudWatch logs encrypted by default
3. **No secrets in state**: Use `sensitive = true` for outputs
4. **HTTPS only**: API Gateway enforces HTTPS
5. **CORS restricted**: Document how to restrict origins

### Operational Best Practices

1. **Versioned deployments**: Use Lambda aliases for blue/green
2. **Monitoring**: CloudWatch alarms for errors, duration, throttles
3. **Logging**: Structured JSON logs (already implemented)
4. **Cost control**: Set billing alerts, consider reserved concurrency

---

## Implementation Recommendations

### Phase 1: Core Module (Priority)
- Lambda function resources
- IAM roles and policies
- CloudWatch Log groups
- Basic API Gateway routes

### Phase 2: Documentation (Priority)
- `docs/DEPLOYMENT.md` with quick start
- Terraform module README
- Example tfvars file

### Phase 3: Enhancements (Optional)
- Custom domain support
- WAF integration
- VPC configuration
- X-Ray tracing

---

## External References

- [AWS Lambda Terraform Resource](https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/lambda_function)
- [API Gateway v2 Terraform](https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/apigatewayv2_api)
- [Rust Lambda Custom Runtime](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-custom.html)
- [Terraform Module Best Practices](https://developer.hashicorp.com/terraform/language/modules/develop)
