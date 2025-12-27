# Feature Specification: Lambda Infrastructure Documentation

**Feature ID**: 003-lambda-infrastructure-docs  
**Created**: 2025-12-27  
**Status**: Draft

---

## Summary

Create comprehensive infrastructure documentation and deployment templates for the AWS Lambda
functions (`evefrontier-lambda-route`, `evefrontier-lambda-scout-gates`,
`evefrontier-lambda-scout-range`). This includes Terraform modules (per ADR 0007), deployment
guides, and operational runbooks.

## Background

The Lambda functions are fully implemented with:

- Request/response models with validation
- RFC 9457 error responses
- Bundled dataset and spatial index at cold start
- JSON tracing for CloudWatch Logs

However, there is no infrastructure-as-code for deployment. Users must manually configure:

- Lambda function resources
- API Gateway integration
- IAM roles and policies
- CloudWatch Logs configuration

## Goals

1. **Terraform modules** for deploying all three Lambda functions
2. **API Gateway configuration** with proper routing and CORS
3. **IAM least-privilege policies** following security best practices
4. **Deployment documentation** in `docs/DEPLOYMENT.md`
5. **Operational runbook** for common tasks (updates, rollbacks, monitoring)

## Non-Goals

- SAM templates (Terraform preferred per ADR 0007)
- CDK (adds TypeScript dependency, unnecessary complexity)
- Multi-region deployment (single region sufficient for initial release)
- Custom domain configuration (user-specific, document as optional)

## Requirements

### Functional Requirements

| ID   | Requirement                                         | Priority |
| ---- | --------------------------------------------------- | -------- |
| FR-1 | Terraform module deploys all three Lambda functions | MUST     |
| FR-2 | API Gateway routes requests to appropriate Lambda   | MUST     |
| FR-3 | IAM roles follow least-privilege principle          | MUST     |
| FR-4 | CloudWatch Logs configured with retention policy    | MUST     |
| FR-5 | Variables for memory, timeout, and environment      | SHOULD   |
| FR-6 | Output values for API endpoint URLs                 | SHOULD   |
| FR-7 | Optional VPC configuration for private deployments  | COULD    |

### Non-Functional Requirements

| ID    | Requirement                                  | Priority |
| ----- | -------------------------------------------- | -------- |
| NFR-1 | Terraform 1.5+ compatibility                 | MUST     |
| NFR-2 | Module follows HashiCorp best practices      | MUST     |
| NFR-3 | Documentation includes troubleshooting guide | SHOULD   |
| NFR-4 | Examples for common deployment scenarios     | SHOULD   |

## Technical Design

### Terraform Module Structure

```
terraform/
├── modules/
│   └── evefrontier-lambda/
│       ├── main.tf           # Lambda resources
│       ├── api_gateway.tf    # API Gateway configuration
│       ├── iam.tf            # IAM roles and policies
│       ├── cloudwatch.tf     # Logging configuration
│       ├── variables.tf      # Input variables
│       ├── outputs.tf        # Output values
│       └── README.md         # Module documentation
└── examples/
    └── complete/
        ├── main.tf           # Example usage
        ├── terraform.tfvars.example
        └── README.md
```

### API Gateway Routes

| Method | Path           | Lambda Function                  |
| ------ | -------------- | -------------------------------- |
| POST   | `/route`       | evefrontier-lambda-route         |
| POST   | `/scout-gates` | evefrontier-lambda-scout-gates   |
| POST   | `/scout-range` | evefrontier-lambda-scout-range   |
| GET    | `/health`      | (optional) health check endpoint |

### IAM Policy (Least Privilege)

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": ["logs:CreateLogGroup", "logs:CreateLogStream", "logs:PutLogEvents"],
      "Resource": "arn:aws:logs:*:*:log-group:/aws/lambda/evefrontier-*"
    }
  ]
}
```

### Documentation Structure

```markdown
# docs/DEPLOYMENT.md

1. Prerequisites (AWS account, Terraform, built binaries)
2. Quick Start (minimal deployment)
3. Configuration Options (variables reference)
4. API Gateway Setup (routes, CORS, authentication)
5. Monitoring & Logging (CloudWatch dashboards)
6. Operational Procedures (updates, rollbacks)
7. Troubleshooting Guide (common issues)
8. Security Considerations (IAM, VPC, secrets)
```

## Test Scenarios

| Scenario                      | Expected Result             |
| ----------------------------- | --------------------------- |
| `terraform validate` passes   | No syntax errors            |
| `terraform plan` with example | Shows expected resources    |
| Module documentation renders  | README.md is complete       |
| IAM policy is minimal         | Only CloudWatch permissions |
| Variables have descriptions   | All inputs documented       |

## Acceptance Criteria

- [ ] Terraform module deploys successfully in AWS
- [ ] API Gateway routes work with curl examples from USAGE.md
- [ ] IAM roles follow least-privilege (no `*` resources except logs)
- [ ] `docs/DEPLOYMENT.md` provides complete deployment guide
- [ ] Example configuration is tested and documented
- [ ] Security review passes (no hardcoded credentials, proper IAM)

## Dependencies

- Completed Lambda function implementations (✅ done)
- Built Lambda binaries with `bundle-data` feature
- AWS account for testing (user responsibility)

## Risks & Mitigations

| Risk                              | Likelihood | Impact | Mitigation               |
| --------------------------------- | ---------- | ------ | ------------------------ |
| AWS API changes                   | Low        | Medium | Pin AWS provider version |
| Terraform version incompatibility | Low        | Low    | Document minimum version |
| User misconfiguration             | Medium     | Medium | Comprehensive examples   |

## Timeline Estimate

| Phase            | Effort      | Duration   |
| ---------------- | ----------- | ---------- |
| Research         | 1 hour      | Day 1      |
| Terraform module | 3 hours     | Day 1-2    |
| Documentation    | 2 hours     | Day 2      |
| Testing          | 1 hour      | Day 2      |
| **Total**        | **7 hours** | **2 days** |

---

## References

- [ADR 0007: DevSecOps Practices](../docs/adrs/0007-devsecops-practices.md)
- [AWS Lambda Terraform Provider](https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/lambda_function)
- [Terraform Module Best Practices](https://developer.hashicorp.com/terraform/language/modules/develop)
- [docs/USAGE.md Lambda section](../docs/USAGE.md#aws-lambda-functions)
