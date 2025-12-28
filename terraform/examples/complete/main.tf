# =============================================================================
# Complete Example: EVE Frontier Lambda Deployment
# =============================================================================
# This example demonstrates a complete deployment of the EVE Frontier Lambda
# functions with API Gateway.
# =============================================================================

terraform {
  required_version = ">= 1.5.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = ">= 5.0.0, < 6.0.0"
    }
  }

  # Uncomment and configure for remote state storage
  # backend "s3" {
  #   bucket         = "your-terraform-state-bucket"
  #   key            = "evefrontier/terraform.tfstate"
  #   region         = "us-east-1"
  #   encrypt        = true
  #   dynamodb_table = "terraform-state-lock"
  # }
}

# -----------------------------------------------------------------------------
# Provider Configuration
# -----------------------------------------------------------------------------

provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      Project     = "evefrontier"
      Environment = var.environment
      ManagedBy   = "terraform"
    }
  }
}

# -----------------------------------------------------------------------------
# Variables
# -----------------------------------------------------------------------------

variable "aws_region" {
  description = "AWS region for deployment"
  type        = string
  default     = "us-east-1"
}

variable "environment" {
  description = "Deployment environment"
  type        = string
  default     = "dev"
}

variable "binaries_path" {
  description = "Path to directory containing Lambda binaries"
  type        = string
  default     = "./binaries"
}

# -----------------------------------------------------------------------------
# EVE Frontier Lambda Module
# -----------------------------------------------------------------------------

module "evefrontier" {
  source = "../../modules/evefrontier-lambda"

  # Core configuration
  environment  = var.environment
  project_name = "evefrontier-demo" # Customize this for your deployment

  # Lambda binary paths (must be named 'bootstrap' for custom runtime)
  route_binary_path       = "${var.binaries_path}/bootstrap-route"
  scout_gates_binary_path = "${var.binaries_path}/bootstrap-scout-gates"
  scout_range_binary_path = "${var.binaries_path}/bootstrap-scout-range"

  # Lambda configuration
  lambda_memory_mb       = 512
  lambda_timeout_seconds = 10
  lambda_architecture    = "arm64"

  # Logging
  log_retention_days = 30
  log_level          = "info"

  # API Gateway
  api_stage_name = "v1"

  # CORS configuration:
  # - For development and testing, a wildcard ("*") origin is convenient but NOT SAFE
  #   for production use.
  # - In production, you MUST restrict CORS to explicit, trusted origins (for example,
  #   your frontend URL). The underlying module validates that wildcards are not
  #   allowed when environment is not in: dev, local, test, staging.
  # Example for production: cors_allowed_origins = ["https://your-frontend.example.com"]
  cors_allowed_origins   = ["*"]
  throttling_burst_limit = 100
  throttling_rate_limit  = 50

  # Tags
  tags = {
    Example = "complete"
  }
}

# -----------------------------------------------------------------------------
# Outputs
# -----------------------------------------------------------------------------

output "api_endpoint" {
  description = "Base URL for the API Gateway"
  value       = module.evefrontier.api_endpoint
}

output "route_endpoint" {
  description = "Full URL for the route endpoint"
  value       = module.evefrontier.endpoint_urls.route
}

output "scout_gates_endpoint" {
  description = "Full URL for the scout-gates endpoint"
  value       = module.evefrontier.endpoint_urls.scout_gates
}

output "scout_range_endpoint" {
  description = "Full URL for the scout-range endpoint"
  value       = module.evefrontier.endpoint_urls.scout_range
}

output "log_groups" {
  description = "CloudWatch Log Group names"
  value       = module.evefrontier.log_group_names
}
