# Implementation Plan: Lambda Infrastructure Documentation

**Feature**: 003-lambda-infrastructure-docs  
**Created**: 2025-12-27  
**Status**: Planning Complete

---

## Technical Context

### Current State Assessment

| Area | Status | Notes |
|------|--------|-------|
| Lambda Functions | ✅ COMPLETE | 3 functions implemented with bundled data |
| Lambda Shared | ✅ COMPLETE | Runtime, tracing, error handling |
| API Contracts | ✅ COMPLETE | Request/response models documented |
| USAGE.md Lambda Section | ✅ COMPLETE | SDK examples, cold-start docs |
| Terraform Module | ❌ MISSING | No IaC for deployment |
| DEPLOYMENT.md | ❌ MISSING | No deployment guide |
| Build Instructions | ⚠️ PARTIAL | In USAGE.md but not complete |

### Existing Infrastructure Documentation

| Document | Content | Gap |
|----------|---------|-----|
| `docs/USAGE.md` | Lambda invocation, SDK examples | No deployment instructions |
| `docs/TODO.md` | Lists Terraform as TODO | Not implemented |
| ADR 0013 | Documents Terraform as IaC choice | Created as part of this feature |

### Technology Stack Confirmed

- **IaC Tool**: Terraform (per ADR 0013)
- **Lambda Runtime**: provided.al2023 (custom Rust)
- **API Gateway**: HTTP API (v2)
- **Architecture**: arm64 (cost-effective)
- **Packaging**: ZIP with bootstrap binary

---

## Constitution Check

| Principle | Status | Implementation |
|-----------|--------|----------------|
| I. TDD | ✅ N/A | Infrastructure code, not application code |
| II. Library-First | ✅ N/A | Documentation task |
| III. ADR Documentation | ✅ COMPLIANT | Aligns with ADR 0013 |
| IV. Clean Code | ✅ APPLICABLE | Terraform follows HCL best practices |
| V. Security-First | ✅ COMPLIANT | IAM least-privilege, no secrets |
| VI. Testing Tiers | ✅ APPLICABLE | `terraform validate` and `plan` |
| VII. Refactoring | ✅ N/A | New code, no refactoring |

---

## Gate Evaluation

### Pre-Implementation Gates

| Gate | Status | Evidence |
|------|--------|----------|
| ADR alignment | ✅ PASSED | ADR 0013 documents Terraform choice |
| Security review | ✅ PASSED | IAM minimal, no secrets |
| Breaking changes | ✅ NONE | New files only |
| Dependencies | ✅ MET | Lambda functions complete |

---

## Phase 0: Research (Complete)

See [research.md](./research.md) for detailed findings.

### Key Decisions

| Decision | Rationale |
|----------|-----------|
| Terraform over SAM/CDK | ADR 0013 rationale, no extra runtime |
| HTTP API over REST API | 70% lower cost, simpler setup |
| Single module with all 3 Lambdas | Simpler deployment, shared patterns |
| arm64 architecture | Better price/performance for Rust |
| 512 MB memory default | Sufficient for bundled dataset |

---

## Phase 1: Terraform Module

### File Structure

```
terraform/
├── modules/
│   └── evefrontier-lambda/
│       ├── main.tf           # Lambda function resources
│       ├── api_gateway.tf    # HTTP API configuration
│       ├── iam.tf            # Roles and policies
│       ├── cloudwatch.tf     # Log groups and retention
│       ├── variables.tf      # Input variables
│       ├── outputs.tf        # Output values
│       └── README.md         # Module documentation
└── examples/
    └── complete/
        ├── main.tf           # Example deployment
        ├── terraform.tfvars.example
        └── README.md         # Example instructions
```

### Implementation Tasks

| Task | File | Description |
|------|------|-------------|
| 1.1 | `variables.tf` | Define all input variables with descriptions |
| 1.2 | `iam.tf` | IAM role and CloudWatch policy |
| 1.3 | `cloudwatch.tf` | Log groups with retention |
| 1.4 | `main.tf` | Lambda function resources (3 functions) |
| 1.5 | `api_gateway.tf` | HTTP API with routes and integrations |
| 1.6 | `outputs.tf` | API endpoint URL, function ARNs |
| 1.7 | `README.md` | Module documentation |
| 1.8 | `examples/complete/` | Working example configuration |

### Variable Definitions

```hcl
# Core
variable "environment" {}
variable "aws_region" {}

# Lambda configuration
variable "lambda_memory_mb" { default = 512 }
variable "lambda_timeout_seconds" { default = 10 }
variable "lambda_architecture" { default = "arm64" }

# Paths to built binaries
variable "route_binary_path" {}
variable "scout_gates_binary_path" {}
variable "scout_range_binary_path" {}

# Logging
variable "log_retention_days" { default = 30 }

# API Gateway
variable "cors_allowed_origins" { default = ["*"] }
variable "stage_name" { default = "v1" }
```

### Output Definitions

```hcl
output "api_endpoint" {}
output "route_function_arn" {}
output "scout_gates_function_arn" {}
output "scout_range_function_arn" {}
output "log_group_names" {}
```

---

## Phase 2: Documentation

### Implementation Tasks

| Task | File | Description |
|------|------|-------------|
| 2.1 | `docs/DEPLOYMENT.md` | Complete deployment guide |
| 2.2 | Module `README.md` | Input/output reference |
| 2.3 | Example `README.md` | Step-by-step walkthrough |
| 2.4 | Update `docs/TODO.md` | Mark task complete |

### DEPLOYMENT.md Structure

```markdown
# Deploying EveFrontier Lambda Functions

## Prerequisites
- AWS account with appropriate permissions
- Terraform 1.5+
- Rust toolchain (for building)
- Built Lambda binaries with `bundle-data` feature

## Quick Start (5 minutes)
1. Build binaries
2. Configure terraform.tfvars
3. terraform init && apply
4. Test endpoints

## Configuration Reference
[All variables with descriptions]

## Building Lambda Binaries
[Cross-compilation for arm64]

## API Gateway Configuration
[Routes, CORS, authentication]

## Monitoring & Logging
[CloudWatch Logs, dashboards]

## Operations
[Updates, rollbacks, scaling]

## Troubleshooting
[Common issues and solutions]

## Security Considerations
[IAM, VPC, secrets management]
```

---

## Phase 3: Validation

### Validation Tasks

| Task | Method | Acceptance Criteria |
|------|--------|---------------------|
| 3.1 | `terraform validate` | No syntax errors |
| 3.2 | `terraform fmt -check` | Properly formatted |
| 3.3 | `terraform plan` with example | Shows expected resources |
| 3.4 | Documentation review | All sections complete |
| 3.5 | Security checklist | IAM minimal, no secrets |

---

## Data Model

N/A - Infrastructure documentation, no data model changes.

---

## API Contracts

No new API contracts. Terraform module inputs/outputs documented in variables.tf and outputs.tf.

---

## Task Summary

| Phase | Tasks | Estimated Effort |
|-------|-------|------------------|
| Phase 1: Terraform Module | 8 tasks | 3 hours |
| Phase 2: Documentation | 4 tasks | 2 hours |
| Phase 3: Validation | 5 tasks | 1 hour |
| **Total** | **17 tasks** | **6 hours** |

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| AWS provider version conflict | Low | Low | Pin provider version |
| Cross-compilation issues | Medium | Medium | Document build requirements |
| User misconfiguration | Medium | Medium | Comprehensive examples |

---

## Next Steps

1. Create Terraform module files (Phase 1)
2. Write DEPLOYMENT.md documentation (Phase 2)
3. Validate with `terraform plan` (Phase 3)
4. Create PR with all changes
5. Update TODO.md to mark complete
