# ADR 0013: Infrastructure as Code Tooling

## Status

Accepted

## Date

2025-12-27

## Context

The EveFrontier project includes AWS Lambda functions (`evefrontier-lambda-route`,
`evefrontier-lambda-scout-gates`, `evefrontier-lambda-scout-range`) that require deployment
infrastructure. We need to choose an Infrastructure as Code (IaC) tool for:

- Deploying Lambda functions
- Configuring API Gateway
- Managing IAM roles and policies
- Setting up CloudWatch Logs

The main options considered were:

1. **Terraform** - HashiCorp's declarative IaC tool with HCL syntax
2. **AWS SAM (Serverless Application Model)** - AWS-native serverless deployment framework
3. **AWS CDK (Cloud Development Kit)** - Imperative IaC using TypeScript/Python
4. **Pulumi** - Multi-cloud IaC with general-purpose programming languages

## Decision

We will use **Terraform** as the Infrastructure as Code tool for deploying Lambda functions and
related AWS resources.

## Rationale

### Why Terraform

1. **No Additional Runtime Dependencies**: Terraform uses HCL, a declarative configuration language.
   SAM requires Python or Node.js runtime; CDK requires TypeScript/Node.js. Since this is a Rust
   project, adding JavaScript/TypeScript dependencies solely for infrastructure would increase
   complexity.

2. **Multi-Cloud Portability**: While currently AWS-only, Terraform's provider model allows future
   expansion to other clouds or services without changing tooling.

3. **Mature Ecosystem**: Terraform has extensive documentation, a large community, and battle-tested
   AWS provider with comprehensive Lambda support.

4. **State Management**: Terraform's state management provides drift detection and reliable
   infrastructure tracking, which is valuable for production deployments.

5. **Declarative Approach**: HCL's declarative syntax aligns with infrastructure-as-documentation
   principles, making configurations self-documenting and reviewable.

### Why Not SAM

- Adds Python/Node.js dependency to a Rust project
- More opinionated, less flexible for custom configurations
- Primarily optimized for Node.js/Python Lambda functions

### Why Not CDK

- Requires TypeScript/JavaScript runtime and npm ecosystem
- Adds significant dependency footprint
- Imperative approach can be harder to review for infrastructure changes

### Why Not Pulumi

- Smaller community and ecosystem compared to Terraform
- Less documentation and fewer examples for AWS Lambda patterns
- Would require choosing a general-purpose language (adding another runtime)

## Consequences

### Positive

- Single IaC tool for all infrastructure needs
- No additional language runtimes required beyond Rust toolchain
- Industry-standard tooling with broad team familiarity
- Reusable modules can be published to Terraform Registry

### Negative

- HCL learning curve for contributors unfamiliar with Terraform
- Terraform state must be managed (local or remote backend)
- Provider version updates may require configuration changes

### Neutral

- Terraform CLI must be installed for infrastructure deployments
- CI/CD pipelines need Terraform setup for automated deployments

## Implementation

- Terraform module location: `terraform/modules/evefrontier-lambda/`
- Example configurations: `terraform/examples/complete/`
- Deployment documentation: `docs/DEPLOYMENT.md`
- Minimum Terraform version: 1.5.0
- AWS provider version: >= 5.0.0, < 6.0.0

## References

- [Terraform AWS Provider](https://registry.terraform.io/providers/hashicorp/aws/latest)
- [Terraform Module Best Practices](https://developer.hashicorp.com/terraform/language/modules/develop)
- [AWS Lambda Terraform Resource](https://registry.terraform.io/providers/hashicorp/aws/latest/docs/resources/lambda_function)
- Feature specification: `specs/003-lambda-infrastructure-docs/spec.md`
